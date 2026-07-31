use crate::errors::VibeError;
use crate::models::agent::Agent;
use crate::utils::config::{
    build_agents_from_config, invalidate_agents_cache, load_agents, load_config, save_config,
    AgentConfig,
};
use crate::utils::fs as vibe_fs;
use crate::utils::path::vibe_skills_dir;
use std::fs;

#[tauri::command]
pub async fn list_agents() -> Result<Vec<Agent>, VibeError> {
    tauri::async_runtime::spawn_blocking(load_agents)
        .await
        .map_err(|error| VibeError::Path(format!("list_agents task failed: {}", error)))?
}

#[tauri::command]
pub async fn add_custom_agent(name: String, skills_dir: String) -> Result<Agent, VibeError> {
    tauri::async_runtime::spawn_blocking(move || add_custom_agent_sync(name, skills_dir))
        .await
        .map_err(|error| VibeError::Path(format!("add_custom_agent task failed: {}", error)))?
}

fn add_custom_agent_sync(name: String, skills_dir: String) -> Result<Agent, VibeError> {
    add_custom_agent_with_options_sync(name, skills_dir, None, Vec::new())
}

#[tauri::command]
pub async fn add_custom_agent_with_options(
    name: String,
    skills_dir: String,
    detect_dir: Option<String>,
    additional_scan_dirs: Vec<String>,
) -> Result<Agent, VibeError> {
    tauri::async_runtime::spawn_blocking(move || {
        add_custom_agent_with_options_sync(name, skills_dir, detect_dir, additional_scan_dirs)
    })
    .await
    .map_err(|error| VibeError::Path(format!("add_custom_agent_with_options task failed: {}", error)))?
}

fn add_custom_agent_with_options_sync(
    name: String,
    skills_dir: String,
    detect_dir: Option<String>,
    additional_scan_dirs: Vec<String>,
) -> Result<Agent, VibeError> {
    let mut config = load_config()?;

    let id = name
        .to_lowercase()
        .replace(|c: char| !c.is_ascii_alphanumeric(), "-")
        .trim_matches('-')
        .to_string();

    if id.is_empty() {
        return Err(VibeError::Config("Invalid agent name".to_string()));
    }

    if config.agents.iter().any(|a| a.id == id) {
        return Err(VibeError::Config(format!(
            "Agent with id '{}' already exists",
            id
        )));
    }

    // M10：校验 skills_dir 为已存在的目录，避免把任意路径（如 C:\）当技能目录深度遍历
    validate_skills_dir(&skills_dir)?;

    let agent_config = AgentConfig {
        id: id.clone(),
        name: name.clone(),
        skills_dir: skills_dir.clone(),
        kind: "agent".to_string(),
        detect_dir: detect_dir.clone().filter(|dir| !dir.trim().is_empty()),
        additional_scan_dirs,
        enabled: true,
        auto_detected: false,
    };

    config.agents.push(agent_config);
    save_config(&config)?;
    invalidate_agents_cache();

    let updated_config = load_config()?;
    let agents = build_agents_from_config(&updated_config)?;
    agents
        .into_iter()
        .find(|a| a.id == id)
        .ok_or_else(|| VibeError::AgentNotFound { agent_id: id })
}

#[tauri::command]
pub async fn update_agent(
    agent_id: String,
    name: Option<String>,
    skills_dir: Option<String>,
    detect_dir: Option<String>,
    additional_scan_dirs: Option<Vec<String>>,
    enabled: Option<bool>,
) -> Result<Agent, VibeError> {
    tauri::async_runtime::spawn_blocking(move || {
        update_agent_sync(agent_id, name, skills_dir, detect_dir, additional_scan_dirs, enabled)
    })
    .await
    .map_err(|error| VibeError::Path(format!("update_agent task failed: {}", error)))?
}

fn update_agent_sync(
    agent_id: String,
    name: Option<String>,
    skills_dir: Option<String>,
    detect_dir: Option<String>,
    additional_scan_dirs: Option<Vec<String>>,
    enabled: Option<bool>,
) -> Result<Agent, VibeError> {
    let mut config = load_config()?;

    let agent_config = config
        .agents
        .iter_mut()
        .find(|a| a.id == agent_id)
        .ok_or_else(|| VibeError::AgentNotFound {
            agent_id: agent_id.clone(),
        })?;

    if let Some(n) = name {
        agent_config.name = n;
    }
    if let Some(d) = skills_dir {
        // M10：同上，更新时也校验
        validate_skills_dir(&d)?;
        agent_config.skills_dir = d;
    }
    if let Some(d) = detect_dir {
        agent_config.detect_dir = if d.trim().is_empty() { None } else { Some(d) };
    }
    if let Some(dirs) = additional_scan_dirs {
        agent_config.additional_scan_dirs = dirs
            .into_iter()
            .map(|dir| dir.trim().to_string())
            .filter(|dir| !dir.is_empty())
            .collect();
    }
    if let Some(e) = enabled {
        agent_config.enabled = e;
    }

    save_config(&config)?;
    invalidate_agents_cache();

    let updated_config = load_config()?;
    let agents = build_agents_from_config(&updated_config)?;
    agents
        .into_iter()
        .find(|a| a.id == agent_id)
        .ok_or_else(|| VibeError::AgentNotFound { agent_id })
}

#[tauri::command]
pub async fn remove_custom_agent(agent_id: String) -> Result<(), VibeError> {
    tauri::async_runtime::spawn_blocking(move || remove_custom_agent_sync(agent_id))
        .await
        .map_err(|error| VibeError::Path(format!("remove_custom_agent task failed: {}", error)))?
}

fn remove_custom_agent_sync(agent_id: String) -> Result<(), VibeError> {
    let mut config = load_config()?;

    let idx = config
        .agents
        .iter()
        .position(|a| a.id == agent_id && !a.auto_detected)
        .ok_or_else(|| VibeError::AgentNotFound {
            agent_id: agent_id.clone(),
        })?;

    config.agents.remove(idx);
    save_config(&config)?;
    invalidate_agents_cache();

    // L5：清理该 agent 在 vibe 库中的镜像目录（~/.vibe-skills/{agent_id}/，内容为指向 agent 源目录的 symlink）
    let vibe_dir = vibe_skills_dir()?;
    let mirror_dir = vibe_dir.join(&agent_id);
    if mirror_dir.is_dir() && is_mirror_dir(&mirror_dir) {
        let _ = fs::remove_dir_all(&mirror_dir);
    }

    Ok(())
}

/// 镜像目录判定：所有直接子项都是链接（或目录为空），避免误删与 agent 同名的真实 skill
fn is_mirror_dir(dir: &std::path::Path) -> bool {
    match fs::read_dir(dir) {
        Ok(entries) => entries.flatten().all(|e| vibe_fs::is_link(&e.path())),
        Err(_) => false,
    }
}

/// M10：校验 skills_dir 为已存在的目录
fn validate_skills_dir(skills_dir: &str) -> Result<(), VibeError> {
    let expanded = crate::utils::path::expand_tilde(skills_dir)?;
    if !expanded.exists() || !expanded.is_dir() {
        return Err(VibeError::Config(format!(
            "Skills directory does not exist or is not a directory: {}",
            skills_dir
        )));
    }
    Ok(())
}
