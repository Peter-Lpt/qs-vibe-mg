use std::fs;
use std::path::Path;

use crate::errors::VabError;
use crate::models::history::HistoryAction;
use crate::models::sync::SyncResult;
use crate::utils::config::{build_agents_from_config, load_config};
use crate::utils::fs as vibe_fs;
use crate::utils::history::record_action;
use crate::utils::path::vibe_skills_dir;

#[tauri::command]
pub fn create_link(skill_id: String, agent_id: String) -> Result<(), VabError> {
    let vibe_dir = vibe_skills_dir()?;
    let skill_path = vibe_dir.join(&skill_id);

    if !skill_path.exists() {
        return Err(VabError::SkillNotFound { skill_id });
    }

    let config = load_config()?;
    let agents = build_agents_from_config(&config)?;
    let agent =
        agents
            .iter()
            .find(|a| a.id == agent_id)
            .ok_or_else(|| VabError::AgentNotFound {
                agent_id: agent_id.clone(),
            })?;

    let agent_skills_dir = Path::new(&agent.skills_dir);
    let link_path = agent_skills_dir.join(&skill_id);

    vibe_fs::create_symlink(&skill_path, &link_path)?;

    let _ = record_action(
        HistoryAction::Link,
        &skill_id,
        Some(&agent_id),
        Some("symlink"),
    );

    Ok(())
}

#[tauri::command]
pub fn remove_link(skill_id: String, agent_id: String) -> Result<(), VabError> {
    let config = load_config()?;
    let agents = build_agents_from_config(&config)?;
    let agent =
        agents
            .iter()
            .find(|a| a.id == agent_id)
            .ok_or_else(|| VabError::AgentNotFound {
                agent_id: agent_id.clone(),
            })?;

    let agent_skills_dir = Path::new(&agent.skills_dir);
    let link_path = agent_skills_dir.join(&skill_id);

    if !vibe_fs::is_link(&link_path) {
        return Err(VabError::LinkNotFound {
            skill_id,
            agent_id,
        });
    }

    vibe_fs::remove_symlink(&link_path)?;

    let _ = record_action(
        HistoryAction::Unlink,
        &skill_id,
        Some(&agent_id),
        Some("symlink"),
    );

    Ok(())
}

#[tauri::command]
pub fn check_link_status(skill_id: String, agent_id: String) -> Result<String, VabError> {
    let config = load_config()?;
    let agents = build_agents_from_config(&config)?;
    let agent =
        agents
            .iter()
            .find(|a| a.id == agent_id)
            .ok_or_else(|| VabError::AgentNotFound {
                agent_id: agent_id.clone(),
            })?;

    let link_path = Path::new(&agent.skills_dir).join(&skill_id);

    if !link_path.exists() && !vibe_fs::is_link(&link_path) {
        return Ok("none".to_string());
    }

    if vibe_fs::is_link(&link_path) {
        if let Ok(target) = vibe_fs::read_link_target(&link_path) {
            let vibe_dir = vibe_skills_dir()?;
            let expected = vibe_dir.join(&skill_id);
            if target == expected {
                return Ok("valid".to_string());
            }
        }
        return Ok("broken".to_string());
    }

    Ok("copy".to_string())
}

#[tauri::command]
pub fn batch_link(skill_ids: Vec<String>, agent_id: String) -> Result<Vec<String>, VabError> {
    let mut errors = Vec::new();

    for skill_id in &skill_ids {
        if let Err(e) = create_link(skill_id.clone(), agent_id.clone()) {
            errors.push(format!("{}: {}", skill_id, e));
        }
    }

    let _ = record_action(
        HistoryAction::BatchLink,
        &skill_ids.join(","),
        Some(&agent_id),
        Some("symlink"),
    );

    Ok(errors)
}

#[tauri::command]
pub fn batch_unlink(skill_ids: Vec<String>, agent_id: String) -> Result<Vec<String>, VabError> {
    let mut errors = Vec::new();

    for skill_id in &skill_ids {
        if let Err(e) = remove_link(skill_id.clone(), agent_id.clone()) {
            errors.push(format!("{}: {}", skill_id, e));
        }
    }

    let _ = record_action(
        HistoryAction::BatchUnlink,
        &skill_ids.join(","),
        Some(&agent_id),
        Some("symlink"),
    );

    Ok(errors)
}

/// 将 agent 的所有 skills 层级同步到 ~/.vibe-skills/{agent_id}/
#[tauri::command]
pub fn sync_agent_to_vibe(agent_id: String) -> Result<SyncResult, VabError> {
    let config = load_config()?;
    let agents = build_agents_from_config(&config)?;
    let agent = agents
        .iter()
        .find(|a| a.id == agent_id)
        .ok_or_else(|| VabError::AgentNotFound {
            agent_id: agent_id.clone(),
        })?;

    let source_dir = Path::new(&agent.skills_dir);
    if !source_dir.exists() {
        return Err(VabError::Path(format!(
            "Source directory does not exist: {}",
            agent.skills_dir
        )));
    }

    let vibe_dir = vibe_skills_dir()?;
    let target_dir = vibe_dir.join(&agent_id);

    let mut result = SyncResult {
        synced_count: 0,
        errors: Vec::new(),
    };

    sync_directory_recursive(source_dir, source_dir, &target_dir, &mut result)?;

    let _ = record_action(
        HistoryAction::BatchLink,
        &format!("agent:{}", agent_id),
        Some(&agent_id),
        Some("symlink"),
    );

    Ok(result)
}

/// 将 agent 的特定分类同步到 ~/.vibe-skills/{agent_id}/{category}/
#[tauri::command]
pub fn sync_category_to_vibe(
    agent_id: String,
    category_path: String,
) -> Result<SyncResult, VabError> {
    let config = load_config()?;
    let agents = build_agents_from_config(&config)?;
    let agent = agents
        .iter()
        .find(|a| a.id == agent_id)
        .ok_or_else(|| VabError::AgentNotFound {
            agent_id: agent_id.clone(),
        })?;

    let source_dir = Path::new(&agent.skills_dir);
    let category_dir = source_dir.join(&category_path);

    if !category_dir.exists() {
        return Err(VabError::Path(format!(
            "Category directory does not exist: {}",
            category_path
        )));
    }

    let vibe_dir = vibe_skills_dir()?;
    let target_dir = vibe_dir.join(&agent_id).join(&category_path);

    let mut result = SyncResult {
        synced_count: 0,
        errors: Vec::new(),
    };

    sync_directory_recursive(source_dir, &category_dir, &target_dir, &mut result)?;

    let _ = record_action(
        HistoryAction::BatchLink,
        &format!("category:{}:{}", agent_id, category_path),
        Some(&agent_id),
        Some("symlink"),
    );

    Ok(result)
}

/// 移除软连接
#[tauri::command]
pub fn remove_sync(agent_id: String, path: Option<String>) -> Result<(), VabError> {
    let vibe_dir = vibe_skills_dir()?;
    let target_base = vibe_dir.join(&agent_id);

    if !target_base.exists() {
        return Ok(());
    }

    let action_desc = match &path {
        Some(p) => format!("remove-sync:{}:{}", agent_id, p),
        None => format!("remove-sync:{}:all", agent_id),
    };

    match &path {
        Some(p) => {
            let target = target_base.join(p);
            if target.exists() {
                let _ = remove_symlinks_recursive(&target)?;
            }
        }
        None => {
            let _ = remove_symlinks_recursive(&target_base)?;
            let _ = fs::remove_dir(&target_base);
        }
    }

    let _ = record_action(
        HistoryAction::BatchUnlink,
        &action_desc,
        Some(&agent_id),
        Some("symlink"),
    );

    Ok(())
}

/// 递归同步目录：对每个 skill 创建软连接
fn sync_directory_recursive(
    _base_source: &Path,
    source_dir: &Path,
    target_dir: &Path,
    result: &mut SyncResult,
) -> Result<(), VabError> {
    if !target_dir.exists() {
        fs::create_dir_all(target_dir)?;
    }

    for entry in fs::read_dir(source_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        if name.starts_with('.') {
            continue;
        }

        let has_skill_md = path.join("SKILL.md").exists();
        let link_target = target_dir.join(&name);

        if has_skill_md {
            if vibe_fs::is_link(&link_target) {
                result.synced_count += 1;
                continue;
            }
            if link_target.exists() {
                result.synced_count += 1;
                continue;
            }

            match vibe_fs::create_symlink(&path, &link_target) {
                Ok(()) => result.synced_count += 1,
                Err(e) => result.errors.push(format!("{}: {}", name, e)),
            }
        } else {
            sync_directory_recursive(_base_source, &path, &link_target, result)?;
        }
    }

    Ok(())
}

/// 按 skill 名称列表删除目标端 symlink
#[tauri::command]
pub fn remove_sync_skills(agent_id: String, skill_names: Vec<String>) -> Result<SyncResult, VabError> {
    let vibe_dir = vibe_skills_dir()?;
    let target_base = vibe_dir.join(&agent_id);

    let mut result = SyncResult {
        synced_count: 0,
        errors: Vec::new(),
    };

    for name in &skill_names {
        let target = target_base.join(name);
        if !target.exists() {
            continue;
        }
        if vibe_fs::is_link(&target) {
            match vibe_fs::remove_symlink(&target) {
                Ok(()) => result.synced_count += 1,
                Err(e) => result.errors.push(format!("{}: {}", name, e)),
            }
        } else if target.is_dir() {
            match fs::remove_dir_all(&target) {
                Ok(()) => result.synced_count += 1,
                Err(e) => result.errors.push(format!("{}: {}", name, e)),
            }
        }
    }

    let _ = record_action(
        HistoryAction::BatchUnlink,
        &format!("remove-sync-skills:{}:{}", agent_id, skill_names.len()),
        Some(&agent_id),
        Some("symlink"),
    );

    Ok(result)
}

/// 递归移除软连接，返回移除数量
fn remove_symlinks_recursive(dir: &Path) -> Result<usize, VabError> {
    if !dir.exists() {
        return Ok(0);
    }

    let mut count = 0;

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if vibe_fs::is_link(&path) {
            vibe_fs::remove_symlink(&path)?;
            count += 1;
        } else if path.is_dir() {
            count += remove_symlinks_recursive(&path)?;
        }
    }

    // 如果目录空了，尝试删除
    if count > 0 {
        let _ = fs::remove_dir(dir);
    }

    Ok(count)
}
