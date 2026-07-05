use std::fs;
use std::path::Path;

use sha2::Digest;
use tracing::warn;

use crate::errors::VabError;
use crate::models::history::HistoryAction;
use crate::models::sync::SyncResult;
use crate::utils::config::{build_agents_from_config, load_config};
use crate::utils::fs as vibe_fs;
use crate::utils::history::record_action;
use crate::utils::path::vibe_skills_dir;

#[tauri::command]
pub fn create_link(skill_id: String, agent_id: String) -> Result<(), VabError> {
    tracing::info!("create_link: skill={}, agent={}", skill_id, agent_id);

    let vibe_dir = vibe_skills_dir()?;
    let skill_path = vibe_dir.join(&skill_id);

    if !skill_path.exists() {
        tracing::error!("create_link: skill not found in vibe-lib: {}", skill_id);
        return Err(VabError::SkillNotFound { skill_id });
    }

    let config = load_config()?;
    let agents = build_agents_from_config(&config)?;
    let agent =
        agents
            .iter()
            .find(|a| a.id == agent_id)
            .ok_or_else(|| {
                tracing::error!("create_link: agent not found: {}", agent_id);
                VabError::AgentNotFound {
                    agent_id: agent_id.clone(),
                }
            })?;

    let agent_skills_dir = Path::new(&agent.skills_dir);
    let link_path = agent_skills_dir.join(&skill_id);

    tracing::info!("create_link: creating symlink {} -> {}", link_path.display(), skill_path.display());
    vibe_fs::create_symlink(&skill_path, &link_path)?;
    tracing::info!("create_link: success");

    if let Err(e) = record_action(
        HistoryAction::Link,
        &skill_id,
        Some(&agent_id),
        Some("symlink"),
    ) {
        warn!("Failed to record Link action: {}", e);
    }

    Ok(())
}

#[tauri::command]
pub fn remove_link(skill_id: String, agent_id: String) -> Result<(), VabError> {
    tracing::info!("remove_link: skill={}, agent={}", skill_id, agent_id);

    let config = load_config()?;
    let agents = build_agents_from_config(&config)?;
    let agent =
        agents
            .iter()
            .find(|a| a.id == agent_id)
            .ok_or_else(|| {
                tracing::error!("remove_link: agent not found: {}", agent_id);
                VabError::AgentNotFound {
                    agent_id: agent_id.clone(),
                }
            })?;

    let agent_skills_dir = Path::new(&agent.skills_dir);
    let link_path = agent_skills_dir.join(&skill_id);

    if !vibe_fs::is_link(&link_path) {
        tracing::error!("remove_link: link not found: {}", link_path.display());
        return Err(VabError::LinkNotFound {
            skill_id,
            agent_id,
        });
    }

    tracing::info!("remove_link: removing symlink at {}", link_path.display());
    vibe_fs::remove_symlink(&link_path)?;
    tracing::info!("remove_link: success");

    if let Err(e) = record_action(
        HistoryAction::Unlink,
        &skill_id,
        Some(&agent_id),
        Some("symlink"),
    ) {
        warn!("Failed to record Unlink action: {}", e);
    }

    Ok(())
}

#[tauri::command]
pub fn batch_link(skill_ids: Vec<String>, agent_id: String) -> Result<Vec<String>, VabError> {
    let mut errors = Vec::new();

    for skill_id in &skill_ids {
        if let Err(e) = create_link(skill_id.clone(), agent_id.clone()) {
            errors.push(format!("{}: {}", skill_id, e));
        }
    }

    if let Err(e) = record_action(
        HistoryAction::BatchLink,
        &skill_ids.join(","),
        Some(&agent_id),
        Some("symlink"),
    ) {
        warn!("Failed to record BatchLink action: {}", e);
    }

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

    if let Err(e) = record_action(
        HistoryAction::BatchUnlink,
        &skill_ids.join(","),
        Some(&agent_id),
        Some("symlink"),
    ) {
        warn!("Failed to record BatchUnlink action: {}", e);
    }

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

    if let Err(e) = record_action(
        HistoryAction::BatchLink,
        &format!("agent:{}", agent_id),
        Some(&agent_id),
        Some("symlink"),
    ) {
        warn!("Failed to record sync_agent_to_vibe action: {}", e);
    }

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

    if let Err(e) = record_action(
        HistoryAction::BatchLink,
        &format!("category:{}:{}", agent_id, category_path),
        Some(&agent_id),
        Some("symlink"),
    ) {
        warn!("Failed to record sync_category_to_vibe action: {}", e);
    }

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

    if let Err(e) = record_action(
        HistoryAction::BatchUnlink,
        &action_desc,
        Some(&agent_id),
        Some("symlink"),
    ) {
        warn!("Failed to record remove_sync action: {}", e);
    }

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

    if let Err(e) = record_action(
        HistoryAction::BatchUnlink,
        &format!("remove-sync-skills:{}:{}", agent_id, skill_names.len()),
        Some(&agent_id),
        Some("symlink"),
    ) {
        warn!("Failed to record remove_sync_skills action: {}", e);
    }

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

/// 将 agent 的 skill 同步到技能库（复制 + 创建 symlink）
#[tauri::command]
pub fn sync_to_vibe(skill_id: String, agent_id: String) -> Result<(), VabError> {
    tracing::info!("sync_to_vibe: skill={}, agent={}", skill_id, agent_id);

    let config = load_config()?;
    let agents = build_agents_from_config(&config)?;
    let agent = agents
        .iter()
        .find(|a| a.id == agent_id)
        .ok_or_else(|| {
            tracing::error!("sync_to_vibe: agent not found: {}", agent_id);
            VabError::AgentNotFound {
                agent_id: agent_id.clone(),
            }
        })?;

    let agent_skills_dir = Path::new(&agent.skills_dir);
    let source_path = agent_skills_dir.join(&skill_id);

    if !source_path.exists() {
        tracing::error!("sync_to_vibe: source not found: {}", source_path.display());
        return Err(VabError::SkillNotFound {
            skill_id: skill_id.clone(),
        });
    }

    // 如果是 symlink，读取真实路径
    let real_source = if vibe_fs::is_link(&source_path) {
        let target = vibe_fs::read_link_target(&source_path)?;
        tracing::info!("sync_to_vibe: source is symlink, target={}", target.display());
        target
    } else {
        tracing::info!("sync_to_vibe: source is real file");
        source_path.clone()
    };

    let vibe_dir = vibe_skills_dir()?;
    let vibe_path = vibe_dir.join(&skill_id);

    // 如果技能库已有此 skill，检查内容是否一致
    if vibe_path.exists() {
        tracing::info!("sync_to_vibe: skill already exists in vibe-lib, checking hash...");
        let source_hash = compute_dir_hash(&real_source);
        let vibe_hash = compute_dir_hash(&vibe_path);

        if source_hash != vibe_hash {
            tracing::warn!("sync_to_vibe: hash mismatch, conflict detected");
            return Err(VabError::Conflict {
                skill_id: skill_id.clone(),
                details: "技能库已有同名 skill，内容不同".to_string(),
            });
        }

        tracing::info!("sync_to_vibe: hash match, replacing with symlink");
        // 内容一致，只需创建 symlink
        if vibe_fs::is_link(&source_path) {
            // 删除旧 symlink（指向非 vibe-lib 的位置）
            vibe_fs::remove_symlink(&source_path)?;
        } else {
            // 删除 agent 的独立副本
            fs::remove_dir_all(&source_path)?;
        }

        vibe_fs::create_symlink(&vibe_path, &source_path)?;
        tracing::info!("sync_to_vibe: symlink created successfully");

        record_action(
            HistoryAction::Link,
            &skill_id,
            Some(&agent_id),
            Some("sync_to_vibe"),
        )
        .ok();

        return Ok(());
    }

    // 技能库没有此 skill，复制过去
    tracing::info!("sync_to_vibe: copying to vibe-lib: {} -> {}", real_source.display(), vibe_path.display());
    vibe_fs::copy_dir_all(&real_source, &vibe_path)?;

    // 如果源是 symlink，删除旧 symlink；如果是独立副本，删除副本
    if vibe_fs::is_link(&source_path) {
        vibe_fs::remove_symlink(&source_path)?;
    } else {
        fs::remove_dir_all(&source_path)?;
    }

    // 创建新 symlink 指向技能库
    vibe_fs::create_symlink(&vibe_path, &source_path)?;
    tracing::info!("sync_to_vibe: sync completed successfully");

    record_action(
        HistoryAction::Link,
        &skill_id,
        Some(&agent_id),
        Some("sync_to_vibe"),
    )
    .ok();

    Ok(())
}

/// 重新链接：删除旧 symlink，创建新 symlink 指向技能库
#[tauri::command]
pub fn relink(skill_id: String, agent_id: String) -> Result<(), VabError> {
    tracing::info!("relink: skill={}, agent={}", skill_id, agent_id);

    let config = load_config()?;
    let agents = build_agents_from_config(&config)?;
    let agent = agents
        .iter()
        .find(|a| a.id == agent_id)
        .ok_or_else(|| {
            tracing::error!("relink: agent not found: {}", agent_id);
            VabError::AgentNotFound {
                agent_id: agent_id.clone(),
            }
        })?;

    let agent_skills_dir = Path::new(&agent.skills_dir);
    let link_path = agent_skills_dir.join(&skill_id);
    let vibe_dir = vibe_skills_dir()?;
    let vibe_path = vibe_dir.join(&skill_id);

    // 技能库必须有此 skill
    if !vibe_path.exists() {
        tracing::error!("relink: skill not found in vibe-lib: {}", skill_id);
        return Err(VabError::SkillNotFound {
            skill_id: skill_id.clone(),
        });
    }

    // 删除旧的 symlink（如果存在）
    if vibe_fs::is_link(&link_path) {
        tracing::info!("relink: removing old symlink at {}", link_path.display());
        vibe_fs::remove_symlink(&link_path)?;
    }

    // 创建新 symlink 指向技能库
    tracing::info!("relink: creating symlink {} -> {}", link_path.display(), vibe_path.display());
    vibe_fs::create_symlink(&vibe_path, &link_path)?;
    tracing::info!("relink: relink completed successfully");

    record_action(
        HistoryAction::Link,
        &skill_id,
        Some(&agent_id),
        Some("relink"),
    )
    .ok();

    Ok(())
}

/// 计算目录内容的 hash（用于比较）
fn compute_dir_hash(dir: &Path) -> String {
    if !dir.exists() {
        return String::new();
    }
    let mut hasher = sha2::Sha256::new();
    hash_dir_recursive(dir, &mut hasher);
    format!("{:x}", hasher.finalize())
}

fn hash_dir_recursive(dir: &Path, hasher: &mut sha2::Sha256) {
    use sha2::Digest;

    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    let mut sorted: Vec<_> = entries.filter_map(|e| e.ok()).collect();
    sorted.sort_by_key(|e| e.file_name());

    for entry in sorted {
        let path = entry.path();
        let name = entry.file_name();

        if path.is_dir() {
            hasher.update(b"dir:");
            let name_str = name.to_string_lossy();
            hasher.update(name_str.as_bytes());
            hasher.update(b"\n");
            hash_dir_recursive(&path, hasher);
        } else if let Ok(content) = fs::read(&path) {
            hasher.update(b"file:");
            let name_str = name.to_string_lossy();
            hasher.update(name_str.as_bytes());
            hasher.update(b":");
            hasher.update(&content);
            hasher.update(b"\n");
        }
    }
}
