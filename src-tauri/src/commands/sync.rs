use std::path::Path;

use crate::errors::VabError;
use crate::utils::config::{build_agents_from_config, load_config};
use crate::utils::fs as vab_fs;
use crate::utils::path::vab_skills_dir;

/// 创建 symlink: ~/.vab-skills/{skill_id} -> {agent.skills_dir}/{skill_id}
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

    vab_fs::create_symlink(&skill_path, &link_path)
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

    vab_fs::remove_symlink(&link_path)
}
