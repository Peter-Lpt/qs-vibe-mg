use crate::errors::VabError;
use crate::models::agent::Agent;
use crate::utils::config::{build_agents_from_config, load_config, save_config, AgentConfig};

/// 获取所有 agent 列表（含检测状态）
#[tauri::command]
pub fn list_agents() -> Result<Vec<Agent>, VabError> {
    let config = load_config()?;
    build_agents_from_config(&config)
}

/// 添加自定义 agent
#[tauri::command]
pub fn add_custom_agent(name: String, skills_dir: String) -> Result<Agent, VabError> {
    let mut config = load_config()?;

    // 生成 id（小写、连字符）
    let id = name
        .to_lowercase()
        .replace(|c: char| !c.is_ascii_alphanumeric(), "-")
        .trim_matches('-')
        .to_string();

    if id.is_empty() {
        return Err(VabError::Config("Invalid agent name".to_string()));
    }

    // 检查是否已存在
    if config.agents.iter().any(|a| a.id == id) {
        return Err(VabError::Config(format!(
            "Agent with id '{}' already exists",
            id
        )));
    }

    let agent_config = AgentConfig {
        id: id.clone(),
        name: name.clone(),
        skills_dir: skills_dir.clone(),
        enabled: true,
        auto_detected: false,
    };

    config.agents.push(agent_config);
    save_config(&config)?;

    let skills_dir_expanded = crate::utils::path::expand_tilde(&skills_dir)?;
    let detected = skills_dir_expanded.exists();

    Ok(Agent {
        id,
        name,
        skills_dir: skills_dir_expanded.to_string_lossy().to_string(),
        detected,
        enabled: true,
        auto_detected: false,
        linked_skills: Vec::new(),
    })
}

/// 移除自定义 agent
#[tauri::command]
pub fn remove_custom_agent(agent_id: String) -> Result<(), VabError> {
    let mut config = load_config()?;

    // 只能移除自定义 agent
    let idx = config
        .agents
        .iter()
        .position(|a| a.id == agent_id && !a.auto_detected)
        .ok_or_else(|| VabError::AgentNotFound {
            agent_id: agent_id.clone(),
        })?;

    config.agents.remove(idx);
    save_config(&config)?;

    Ok(())
}
