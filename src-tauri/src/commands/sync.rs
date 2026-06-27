use std::path::Path;

use crate::errors::VabError;
use crate::models::history::HistoryAction;
use crate::utils::config::{build_agents_from_config, load_config};
use crate::utils::fs as vab_fs;
use crate::utils::history::record_action;
use crate::utils::path::vab_skills_dir;

/// 创建 symlink: agent.skills_dir/{skill_id} → ~/.vab-skills/{skill_id}
#[tauri::command]
pub fn create_link(skill_id: String, agent_id: String) -> Result<(), VabError> {
    let vab_dir = vab_skills_dir()?;
    let skill_path = vab_dir.join(&skill_id);

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

    vab_fs::create_symlink(&skill_path, &link_path)?;

    // 记录历史
    let _ = record_action(
        HistoryAction::Link,
        &skill_id,
        Some(&agent_id),
        Some("symlink"),
    );

    Ok(())
}

/// 删除 symlink
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

    if !vab_fs::is_link(&link_path) {
        return Err(VabError::LinkNotFound {
            skill_id,
            agent_id,
        });
    }

    vab_fs::remove_symlink(&link_path)?;

    // 记录历史
    let _ = record_action(
        HistoryAction::Unlink,
        &skill_id,
        Some(&agent_id),
        Some("symlink"),
    );

    Ok(())
}

/// 检查链接状态
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

    if !link_path.exists() && !vab_fs::is_link(&link_path) {
        return Ok("none".to_string());
    }

    if vab_fs::is_link(&link_path) {
        if let Ok(target) = vab_fs::read_link_target(&link_path) {
            let vab_dir = vab_skills_dir()?;
            let expected = vab_dir.join(&skill_id);
            if target == expected {
                return Ok("valid".to_string());
            }
        }
        return Ok("broken".to_string());
    }

    // Exists but not a link (regular directory)
    Ok("copy".to_string())
}

/// 批量关联
#[tauri::command]
pub fn batch_link(skill_ids: Vec<String>, agent_id: String) -> Result<Vec<String>, VabError> {
    let mut errors = Vec::new();

    for skill_id in &skill_ids {
        if let Err(e) = create_link(skill_id.clone(), agent_id.clone()) {
            errors.push(format!("{}: {}", skill_id, e));
        }
    }

    if !errors.is_empty() {
        // Still record batch action
        let _ = record_action(
            HistoryAction::BatchLink,
            &skill_ids.join(","),
            Some(&agent_id),
            Some("symlink"),
        );
    } else {
        let _ = record_action(
            HistoryAction::BatchLink,
            &skill_ids.join(","),
            Some(&agent_id),
            Some("symlink"),
        );
    }

    Ok(errors)
}

/// 批量取消关联
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
