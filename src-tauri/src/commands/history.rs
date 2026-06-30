use std::fs;
use std::path::Path;

use crate::errors::VabError;
use crate::models::history::{HistoryAction, HistoryEntry};
use crate::utils::config::{build_agents_from_config, load_config};
use crate::utils::fs as vibe_fs;
use crate::utils::history::{last_undone_entry, last_undone_entry_for_redo, load_history, mark_undone};
use crate::utils::path::vibe_skills_dir;

/// 获取操作历史
#[tauri::command]
pub fn get_history() -> Result<Vec<HistoryEntry>, VabError> {
    let store = load_history()?;
    Ok(store.entries)
}

/// 撤销最后一个操作
#[tauri::command]
pub fn undo() -> Result<HistoryEntry, VabError> {
    let entry = last_undone_entry()?.ok_or(VabError::NothingToUndo)?;

    // 执行逆操作
    match entry.action {
        HistoryAction::Link => {
            // 逆操作：删除 symlink
            if let (Some(ref agent_id), ..) = (&entry.agent_id, &entry.mode) {
                let config = load_config()?;
                let agents = build_agents_from_config(&config)?;
                if let Some(agent) = agents.iter().find(|a| a.id == *agent_id) {
                    let link_path = Path::new(&agent.skills_dir).join(&entry.skill_id);
                    let _ = vibe_fs::remove_symlink(&link_path);
                }
            }
        }
        HistoryAction::Unlink => {
            // 逆操作：重新创建 symlink
            if let Some(ref agent_id) = entry.agent_id {
                let vibe_dir = vibe_skills_dir()?;
                let skill_path = vibe_dir.join(&entry.skill_id);
                let config = load_config()?;
                let agents = build_agents_from_config(&config)?;
                if let Some(agent) = agents.iter().find(|a| a.id == *agent_id) {
                    let link_path = Path::new(&agent.skills_dir).join(&entry.skill_id);
                    if skill_path.exists() {
                        let _ = vibe_fs::create_symlink(&skill_path, &link_path);
                    }
                }
            }
        }
        HistoryAction::Install => {
            // 逆操作：删除 skill
            let skill_path = vibe_skills_dir()?.join(&entry.skill_id);
            if skill_path.exists() {
                let _ = fs::remove_dir_all(&skill_path);
            }
        }
        HistoryAction::Delete => {
            // 逆操作：需要从 snapshot 恢复，但 v0.1 暂不支持完整 snapshot
            // 标记为已撤销但无法恢复
            return Err(VabError::History(
                "Cannot undo delete without snapshot".to_string(),
            ));
        }
        HistoryAction::BatchLink => {
            // 逆操作：批量删除 symlink
            if let Some(ref agent_id) = entry.agent_id {
                let skill_ids: Vec<&str> = entry.skill_id.split(',').collect();
                let config = load_config()?;
                let agents = build_agents_from_config(&config)?;
                if let Some(agent) = agents.iter().find(|a| a.id == *agent_id) {
                    for skill_id in skill_ids {
                        let link_path = Path::new(&agent.skills_dir).join(skill_id);
                        let _ = vibe_fs::remove_symlink(&link_path);
                    }
                }
            }
        }
        HistoryAction::BatchUnlink => {
            // 逆操作：批量重新创建 symlink
            if let Some(ref agent_id) = entry.agent_id {
                let skill_ids: Vec<&str> = entry.skill_id.split(',').collect();
                let vibe_dir = vibe_skills_dir()?;
                let config = load_config()?;
                let agents = build_agents_from_config(&config)?;
                if let Some(agent) = agents.iter().find(|a| a.id == *agent_id) {
                    for skill_id in skill_ids {
                        let skill_path = vibe_dir.join(skill_id);
                        let link_path = Path::new(&agent.skills_dir).join(skill_id);
                        if skill_path.exists() {
                            let _ = vibe_fs::create_symlink(&skill_path, &link_path);
                        }
                    }
                }
            }
        }
    }

    mark_undone(&entry.id, true)?;
    let mut undone = entry;
    undone.undone = true;
    Ok(undone)
}

/// 重做最后一个已撤销的操作
#[tauri::command]
pub fn redo() -> Result<HistoryEntry, VabError> {
    let entry = last_undone_entry_for_redo()?.ok_or(VabError::NothingToRedo)?;

    // 执行正向操作
    match entry.action {
        HistoryAction::Link => {
            if let Some(ref agent_id) = entry.agent_id {
                let vibe_dir = vibe_skills_dir()?;
                let skill_path = vibe_dir.join(&entry.skill_id);
                let config = load_config()?;
                let agents = build_agents_from_config(&config)?;
                if let Some(agent) = agents.iter().find(|a| a.id == *agent_id) {
                    let link_path = Path::new(&agent.skills_dir).join(&entry.skill_id);
                    if skill_path.exists() {
                        vibe_fs::create_symlink(&skill_path, &link_path)?;
                    }
                }
            }
        }
        HistoryAction::Unlink => {
            if let Some(ref agent_id) = entry.agent_id {
                let config = load_config()?;
                let agents = build_agents_from_config(&config)?;
                if let Some(agent) = agents.iter().find(|a| a.id == *agent_id) {
                    let link_path = Path::new(&agent.skills_dir).join(&entry.skill_id);
                    let _ = vibe_fs::remove_symlink(&link_path);
                }
            }
        }
        HistoryAction::Install => {
            // Cannot redo install without source
            return Err(VabError::History(
                "Cannot redo install operation".to_string(),
            ));
        }
        HistoryAction::Delete => {
            let skill_path = vibe_skills_dir()?.join(&entry.skill_id);
            if skill_path.exists() {
                let _ = fs::remove_dir_all(&skill_path);
            }
        }
        HistoryAction::BatchLink => {
            if let Some(ref agent_id) = entry.agent_id {
                let skill_ids: Vec<&str> = entry.skill_id.split(',').collect();
                let vibe_dir = vibe_skills_dir()?;
                let config = load_config()?;
                let agents = build_agents_from_config(&config)?;
                if let Some(agent) = agents.iter().find(|a| a.id == *agent_id) {
                    for skill_id in skill_ids {
                        let skill_path = vibe_dir.join(skill_id);
                        let link_path = Path::new(&agent.skills_dir).join(skill_id);
                        if skill_path.exists() {
                            let _ = vibe_fs::create_symlink(&skill_path, &link_path);
                        }
                    }
                }
            }
        }
        HistoryAction::BatchUnlink => {
            if let Some(ref agent_id) = entry.agent_id {
                let skill_ids: Vec<&str> = entry.skill_id.split(',').collect();
                let config = load_config()?;
                let agents = build_agents_from_config(&config)?;
                if let Some(agent) = agents.iter().find(|a| a.id == *agent_id) {
                    for skill_id in skill_ids {
                        let link_path = Path::new(&agent.skills_dir).join(skill_id);
                        let _ = vibe_fs::remove_symlink(&link_path);
                    }
                }
            }
        }
    }

    mark_undone(&entry.id, false)?;
    let mut redone = entry;
    redone.undone = false;
    Ok(redone)
}
