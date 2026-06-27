use crate::errors::VabError;
use crate::models::agent::Agent;
use crate::utils::config::{build_agents_from_config, load_config};

/// 获取所有 agent 列表（含检测状态）
#[tauri::command]
pub fn list_agents() -> Result<Vec<Agent>, VabError> {
    let config = load_config()?;
    build_agents_from_config(&config)
}
