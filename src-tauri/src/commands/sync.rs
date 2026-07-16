use std::fs;
use std::path::{Path, PathBuf};

use tracing::warn;

use crate::errors::VibeError;
use crate::models::agent::Agent;
use crate::models::history::HistoryAction;
use crate::models::sync::SyncResult;
use crate::utils::config::{invalidate_agents_cache, load_agents};
use crate::utils::fs as vibe_fs;
use crate::utils::hash::dir_hash;
use crate::utils::history::{record_action, record_action_with_skills, record_action_with_source};
use crate::utils::path::vibe_skills_dir;

/// 仅创建符号链接（链接方向：vibe-lib → agent 目录），不记录历史
fn link_skill(skill_id: &str, agent: &Agent) -> Result<(), VibeError> {
    let vibe_dir = vibe_skills_dir()?;
    let skill_path = vibe_dir.join(skill_id);
    if !skill_path.exists() {
        return Err(VibeError::SkillNotFound {
            skill_id: skill_id.to_string(),
        });
    }
    let link_path = Path::new(&agent.skills_dir).join(skill_id);
    vibe_fs::create_symlink(&skill_path, &link_path)?;
    invalidate_agents_cache();
    Ok(())
}

/// 仅移除符号链接（链接方向：vibe-lib → agent 目录），不记录历史
fn unlink_skill(skill_id: &str, agent: &Agent, source_path: Option<&str>) -> Result<(), VibeError> {
    let link_path = resolve_agent_skill_path(agent, skill_id, source_path, false)?;
    if !vibe_fs::is_link(&link_path) {
        return Err(VibeError::LinkNotFound {
            skill_id: skill_id.to_string(),
            agent_id: agent.id.clone(),
        });
    }
    vibe_fs::remove_symlink(&link_path)?;
    invalidate_agents_cache();
    Ok(())
}

fn resolve_agent_skill_path(
    agent: &Agent,
    skill_id: &str,
    source_path: Option<&str>,
    require_skill_md: bool,
) -> Result<PathBuf, VibeError> {
    let agent_skills_dir = Path::new(&agent.skills_dir);

    if let Some(path) = source_path.filter(|p| !p.trim().is_empty()) {
        let candidate = PathBuf::from(path);
        let normalized_candidate = vibe_fs::normalize_path(&candidate);
        let normalized_agent_dir = vibe_fs::normalize_path(agent_skills_dir);
        if !normalized_candidate.starts_with(&normalized_agent_dir) {
            return Err(VibeError::Path(format!(
                "Source path is outside agent skills directory: {}",
                path
            )));
        }
        if !require_skill_md
            || candidate.join("SKILL.md").exists()
            || vibe_fs::is_link(&candidate)
        {
            return Ok(candidate);
        }
    }

    let direct = agent_skills_dir.join(skill_id);
    if direct.exists() || vibe_fs::is_link(&direct) {
        if !require_skill_md || direct.join("SKILL.md").exists() || vibe_fs::is_link(&direct) {
            return Ok(direct);
        }
    }

    find_skill_path_recursive(agent_skills_dir, skill_id).ok_or_else(|| VibeError::SkillNotFound {
        skill_id: skill_id.to_string(),
    })
}

/// 将 agent 的 skill 同步到技能库（复制 + 创建 symlink），不记录历史
/// 当 force=true 时，如果技能库已有同名 skill 且内容不同，会用 agent 的版本覆盖技能库的版本
fn sync_to_vibe_impl(
    skill_id: &str,
    agent: &Agent,
    force: bool,
    source_path: Option<&str>,
) -> Result<(), VibeError> {
    let source_path = resolve_agent_skill_path(agent, skill_id, source_path, true)?;

    // 如果是 symlink，读取真实路径
    let real_source = if vibe_fs::is_link(&source_path) {
        vibe_fs::read_link_target(&source_path)?
    } else {
        source_path.clone()
    };

    let vibe_dir = vibe_skills_dir()?;
    let vibe_path = vibe_dir.join(skill_id);

    // 如果技能库已有此 skill，检查内容是否一致
    if vibe_path.exists() {
        let source_hash = dir_hash(&real_source);
        let vibe_hash = dir_hash(&vibe_path);

        if source_hash != vibe_hash {
            if !force {
                return Err(VibeError::Conflict {
                    skill_id: skill_id.to_string(),
                    details: "技能库已有同名 skill，内容不同".to_string(),
                });
            }
            // force=true：用 agent 的版本覆盖技能库
            tracing::info!(
                "sync_to_vibe_impl: force overwriting vibe library copy of {}",
                skill_id
            );
            fs::remove_dir_all(&vibe_path)?;
            vibe_fs::copy_dir_all(&real_source, &vibe_path)?;
        }

        // 内容一致（或已强制覆盖），只需创建 symlink
        if vibe_fs::is_link(&source_path) {
            vibe_fs::remove_symlink(&source_path)?;
        } else {
            fs::remove_dir_all(&source_path)?;
        }

        vibe_fs::create_symlink(&vibe_path, &source_path)?;
        invalidate_agents_cache();
        return Ok(());
    }

    // 技能库没有此 skill，复制过去
    vibe_fs::copy_dir_all(&real_source, &vibe_path)?;

    // 如果源是 symlink，删除旧 symlink；如果是独立副本，删除副本
    if vibe_fs::is_link(&source_path) {
        vibe_fs::remove_symlink(&source_path)?;
    } else {
        fs::remove_dir_all(&source_path)?;
    }

    // 创建新 symlink 指向技能库
    vibe_fs::create_symlink(&vibe_path, &source_path)?;
    invalidate_agents_cache();
    Ok(())
}

/// 重新链接：如果技能库没有则先同步，然后创建 symlink 指向技能库，不记录历史
fn relink_impl(skill_id: &str, agent: &Agent, source_path: Option<&str>) -> Result<(), VibeError> {
    let agent_skills_dir = Path::new(&agent.skills_dir);
    let link_path = resolve_agent_skill_path(agent, skill_id, source_path, false)?;
    let vibe_dir = vibe_skills_dir()?;
    let vibe_path = vibe_dir.join(skill_id);

    // 技能库没有此 skill，先从 agent 复制过去
    if !vibe_path.exists() {
        let real_source = if vibe_fs::is_link(&link_path) {
            vibe_fs::read_link_target(&link_path)?
        } else if link_path.exists() && link_path.join("SKILL.md").exists() {
            link_path.clone()
        } else {
            find_skill_path_recursive(agent_skills_dir, skill_id).ok_or_else(|| {
                VibeError::SkillNotFound {
                    skill_id: skill_id.to_string(),
                }
            })?
        };

        vibe_fs::copy_dir_all(&real_source, &vibe_path)?;
    }

    // 删除旧的 symlink（如果存在）
    if vibe_fs::is_link(&link_path) {
        vibe_fs::remove_symlink(&link_path)?;
    }

    // 创建新 symlink 指向技能库
    vibe_fs::create_symlink(&vibe_path, &link_path)?;
    invalidate_agents_cache();
    Ok(())
}

fn replace_with_library_impl(
    skill_id: &str,
    agent: &Agent,
    source_path: Option<&str>,
) -> Result<(), VibeError> {
    let vibe_dir = vibe_skills_dir()?;
    let vibe_path = vibe_dir.join(skill_id);
    if !vibe_path.exists() {
        return Err(VibeError::SkillNotFound {
            skill_id: skill_id.to_string(),
        });
    }

    let link_path = resolve_agent_skill_path(agent, skill_id, source_path, false)?;
    if vibe_fs::is_link(&link_path) {
        vibe_fs::remove_symlink(&link_path)?;
    } else if link_path.is_dir() {
        fs::remove_dir_all(&link_path)?;
    } else if link_path.exists() {
        fs::remove_file(&link_path)?;
    }
    vibe_fs::create_symlink(&vibe_path, &link_path)?;
    invalidate_agents_cache();
    Ok(())
}

#[tauri::command]
pub fn create_link(skill_id: String, agent_id: String) -> Result<(), VibeError> {
    tracing::info!("create_link: skill={}, agent={}", skill_id, agent_id);

    let agents = load_agents()?;
    let agent = agents.iter().find(|a| a.id == agent_id).ok_or_else(|| {
        tracing::error!("create_link: agent not found: {}", agent_id);
        VibeError::AgentNotFound {
            agent_id: agent_id.clone(),
        }
    })?;

    link_skill(&skill_id, agent)?;
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
pub fn remove_link(
    skill_id: String,
    agent_id: String,
    source_path: Option<String>,
) -> Result<(), VibeError> {
    tracing::info!("remove_link: skill={}, agent={}", skill_id, agent_id);

    let agents = load_agents()?;
    let agent = agents.iter().find(|a| a.id == agent_id).ok_or_else(|| {
        tracing::error!("remove_link: agent not found: {}", agent_id);
        VibeError::AgentNotFound {
            agent_id: agent_id.clone(),
        }
    })?;

    unlink_skill(&skill_id, agent, source_path.as_deref())?;
    tracing::info!("remove_link: success");

    if let Err(e) = record_action_with_source(
        HistoryAction::Unlink,
        &skill_id,
        None,
        Some(&agent_id),
        Some("symlink"),
        source_path.as_deref(),
    ) {
        warn!("Failed to record Unlink action: {}", e);
    }

    Ok(())
}

#[tauri::command]
pub fn batch_link(skill_ids: Vec<String>, agent_id: String) -> Result<Vec<String>, VibeError> {
    let agents = load_agents()?;
    let agent = agents.iter().find(|a| a.id == agent_id).ok_or_else(|| {
        VibeError::AgentNotFound {
            agent_id: agent_id.clone(),
        }
    })?;

    let mut errors = Vec::new();
    let mut linked = Vec::new();

    for skill_id in &skill_ids {
        match link_skill(skill_id, agent) {
            Ok(()) => linked.push(skill_id.clone()),
            Err(e) => errors.push(format!("{}: {}", skill_id, e)),
        }
    }

    // 仅记录一条批量历史（携带实际受影响的 skill 列表），避免逐条重复记录
    if !linked.is_empty() {
        if let Err(e) = record_action_with_skills(
            HistoryAction::BatchLink,
            &linked.join(","),
            Some(linked),
            Some(&agent_id),
            Some("symlink"),
        ) {
            warn!("Failed to record BatchLink action: {}", e);
        }
    }

    Ok(errors)
}

#[tauri::command]
pub fn batch_unlink(skill_ids: Vec<String>, agent_id: String) -> Result<Vec<String>, VibeError> {
    let agents = load_agents()?;
    let agent = agents.iter().find(|a| a.id == agent_id).ok_or_else(|| {
        VibeError::AgentNotFound {
            agent_id: agent_id.clone(),
        }
    })?;

    let mut errors = Vec::new();
    let mut unlinked = Vec::new();

    for skill_id in &skill_ids {
        match unlink_skill(skill_id, agent, None) {
            Ok(()) => unlinked.push(skill_id.clone()),
            Err(e) => errors.push(format!("{}: {}", skill_id, e)),
        }
    }

    if !unlinked.is_empty() {
        if let Err(e) = record_action_with_skills(
            HistoryAction::BatchUnlink,
            &unlinked.join(","),
            Some(unlinked),
            Some(&agent_id),
            Some("symlink"),
        ) {
            warn!("Failed to record BatchUnlink action: {}", e);
        }
    }

    Ok(errors)
}

/// 将 agent 的所有 skills 层级同步到 ~/.vibe-skills/{agent_id}/
#[tauri::command]
pub fn sync_agent_to_vibe(agent_id: String) -> Result<SyncResult, VibeError> {
    let agents = load_agents()?;
    let agent = agents
        .iter()
        .find(|a| a.id == agent_id)
        .ok_or_else(|| VibeError::AgentNotFound {
            agent_id: agent_id.clone(),
        })?;

    let source_dir = Path::new(&agent.skills_dir);
    if !source_dir.exists() {
        return Err(VibeError::Path(format!(
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

    sync_directory_recursive(source_dir, &target_dir, &mut result)?;

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
) -> Result<SyncResult, VibeError> {
    let agents = load_agents()?;
    let agent = agents
        .iter()
        .find(|a| a.id == agent_id)
        .ok_or_else(|| VibeError::AgentNotFound {
            agent_id: agent_id.clone(),
        })?;

    let source_dir = Path::new(&agent.skills_dir);
    let category_dir = source_dir.join(&category_path);

    if !category_dir.exists() {
        return Err(VibeError::Path(format!(
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

    sync_directory_recursive(&category_dir, &target_dir, &mut result)?;

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
pub fn remove_sync(agent_id: String, path: Option<String>) -> Result<(), VibeError> {
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
    source_dir: &Path,
    target_dir: &Path,
    result: &mut SyncResult,
) -> Result<(), VibeError> {
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
            sync_directory_recursive(&path, &link_target, result)?;
        }
    }

    Ok(())
}

/// 按 skill 名称列表删除目标端 symlink
#[tauri::command]
pub fn remove_sync_skills(
    agent_id: String,
    skill_names: Vec<String>,
) -> Result<SyncResult, VibeError> {
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
fn remove_symlinks_recursive(dir: &Path) -> Result<usize, VibeError> {
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

/// 将 agent 的 skill 同步到技能库（命令入口，记录单条 Link 历史）
#[tauri::command]
pub fn sync_to_vibe(
    skill_id: String,
    agent_id: String,
    force: bool,
    source_path: Option<String>,
) -> Result<(), VibeError> {
    tracing::info!("sync_to_vibe: skill={}, agent={}, force={}", skill_id, agent_id, force);

    let agents = load_agents()?;
    let agent = agents.iter().find(|a| a.id == agent_id).ok_or_else(|| {
        tracing::error!("sync_to_vibe: agent not found: {}", agent_id);
        VibeError::AgentNotFound {
            agent_id: agent_id.clone(),
        }
    })?;

    sync_to_vibe_impl(&skill_id, agent, force, source_path.as_deref())?;
    tracing::info!("sync_to_vibe: sync completed successfully");

    if let Err(e) = record_action_with_source(
        HistoryAction::Link,
        &skill_id,
        Some(vec![skill_id.clone()]),
        Some(&agent_id),
        Some("sync_to_vibe"),
        source_path.as_deref(),
    ) {
        warn!("Failed to record Link action: {}", e);
    }

    Ok(())
}

/// 重新链接（命令入口，记录单条 Link 历史）
#[tauri::command]
pub fn relink(
    skill_id: String,
    agent_id: String,
    source_path: Option<String>,
) -> Result<(), VibeError> {
    tracing::info!("relink: skill={}, agent={}", skill_id, agent_id);

    let agents = load_agents()?;
    let agent = agents.iter().find(|a| a.id == agent_id).ok_or_else(|| {
        tracing::error!("relink: agent not found: {}", agent_id);
        VibeError::AgentNotFound {
            agent_id: agent_id.clone(),
        }
    })?;

    relink_impl(&skill_id, agent, source_path.as_deref())?;
    tracing::info!("relink: relink completed successfully");

    if let Err(e) = record_action_with_source(
        HistoryAction::Link,
        &skill_id,
        Some(vec![skill_id.clone()]),
        Some(&agent_id),
        Some("relink"),
        source_path.as_deref(),
    ) {
        warn!("Failed to record Link action: {}", e);
    }

    Ok(())
}

#[tauri::command]
pub fn replace_with_library(
    skill_id: String,
    agent_id: String,
    source_path: Option<String>,
) -> Result<(), VibeError> {
    tracing::info!("replace_with_library: skill={}, agent={}", skill_id, agent_id);

    let agents = load_agents()?;
    let agent = agents.iter().find(|a| a.id == agent_id).ok_or_else(|| {
        tracing::error!("replace_with_library: agent not found: {}", agent_id);
        VibeError::AgentNotFound {
            agent_id: agent_id.clone(),
        }
    })?;

    replace_with_library_impl(&skill_id, agent, source_path.as_deref())?;

    if let Err(e) = record_action_with_source(
        HistoryAction::Link,
        &skill_id,
        Some(vec![skill_id.clone()]),
        Some(&agent_id),
        Some("replace_with_library"),
        source_path.as_deref(),
    ) {
        warn!("Failed to record Link action: {}", e);
    }

    Ok(())
}

/// 递归查找 skill 路径
fn find_skill_path_recursive(dir: &Path, skill_id: &str) -> Option<PathBuf> {
    if !dir.exists() {
        return None;
    }

    // 直接检查当前目录
    let direct = dir.join(skill_id);
    if (direct.exists() && direct.join("SKILL.md").exists()) || vibe_fs::is_link(&direct) {
        return Some(direct);
    }

    // 递归搜索子目录
    for entry in fs::read_dir(dir).ok()? {
        let entry = entry.ok()?;
        let path = entry.path();
        let name = path.file_name().map(|n| n.to_string_lossy().to_string());
        if name.as_deref() == Some(skill_id)
            && ((path.exists() && path.join("SKILL.md").exists()) || vibe_fs::is_link(&path))
        {
            return Some(path);
        }
        if path.is_dir() {
            if let Some(found) = find_skill_path_recursive(&path, skill_id) {
                return Some(found);
            }
        }
    }

    None
}

/// 批量操作：对同一个 skill 执行多个 agent 的操作（每个 agent 记录独立历史，撤销/重做精确生效）
#[tauri::command]
pub fn batch_skill_action(
    skill_id: String,
    agent_ids: Vec<String>,
    action: String,
) -> Result<SyncResult, VibeError> {
    tracing::info!(
        "batch_skill_action: skill={}, agents={}, action={}",
        skill_id,
        agent_ids.join(","),
        action
    );

    let agents = load_agents()?;

    let mut result = SyncResult {
        synced_count: 0,
        errors: Vec::new(),
    };

    for agent_id in &agent_ids {
        let agent = match agents.iter().find(|a| a.id == *agent_id) {
            Some(a) => a,
            None => {
                result.errors.push(format!("{}: agent not found", agent_id));
                continue;
            }
        };

        let op_result = match action.as_str() {
            "link" => link_skill(&skill_id, agent),
            "unlink" => unlink_skill(&skill_id, agent, None),
            "sync_to_vibe" => sync_to_vibe_impl(&skill_id, agent, false, None),
            "replace_with_link" => sync_to_vibe_impl(&skill_id, agent, false, None),
            "replace_with_library" => replace_with_library_impl(&skill_id, agent, None),
            "relink" => relink_impl(&skill_id, agent, None),
            "remove_dangling" => unlink_skill(&skill_id, agent, None),
            _ => {
                result.errors.push(format!("Unknown action: {}", action));
                continue;
            }
        };

        match op_result {
            Ok(()) => {
                result.synced_count += 1;
                // 链接方向（创建链接）记为 Link，移除方向记为 Unlink
                let history_action = match action.as_str() {
                    "link" | "relink" | "sync_to_vibe" | "replace_with_link" => HistoryAction::Link,
                    _ => HistoryAction::Unlink,
                };
                if let Err(e) = record_action(
                    history_action,
                    &skill_id,
                    Some(agent_id),
                    Some(&action),
                ) {
                    warn!("Failed to record batch action: {}", e);
                }
            }
            Err(e) => result.errors.push(format!("{}: {}", agent_id, e)),
        }
    }

    Ok(result)
}
