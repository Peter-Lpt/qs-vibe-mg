use crate::errors::VibeError;
use crate::models::agent::Agent;
use crate::models::sync::SkillsTreeNode;
use crate::utils::config::{
    build_agents_from_config, invalidate_agents_cache, load_agents, load_config, save_config,
    AgentConfig,
};
use crate::utils::fs as vibe_fs;
use crate::utils::path::vibe_skills_dir;
use std::fs;
use std::path::Path;

#[tauri::command]
pub fn list_agents() -> Result<Vec<Agent>, VibeError> {
    load_agents()
}

#[tauri::command]
pub fn add_custom_agent(name: String, skills_dir: String) -> Result<Agent, VibeError> {
    add_custom_agent_with_options(name, skills_dir, None, Vec::new())
}

#[tauri::command]
pub fn add_custom_agent_with_options(
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
pub fn update_agent(
    agent_id: String,
    name: Option<String>,
    skills_dir: Option<String>,
    detect_dir: Option<String>,
    additional_scan_dirs: Option<Vec<String>>,
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
pub fn remove_custom_agent(agent_id: String) -> Result<(), VibeError> {
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

    Ok(())
}

#[tauri::command]
pub fn get_skills_tree(agent_id: String) -> Result<SkillsTreeNode, VibeError> {
    let agents = load_agents()?;
    let agent = agents
        .iter()
        .find(|a| a.id == agent_id)
        .ok_or_else(|| VibeError::AgentNotFound {
            agent_id: agent_id.clone(),
        })?;

    let skills_dir = Path::new(&agent.skills_dir);
    if !skills_dir.exists() {
        return Ok(SkillsTreeNode {
            name: agent.name.clone(),
            path: agent.skills_dir.clone(),
            is_dir: true,
            skill_count: 0,
            synced: false,
            synced_count: 0,
            children: Vec::new(),
            link_target: None,
            is_source_link: false,
            truncated: false,
        });
    }

    let vibe_dir = vibe_skills_dir()?;
    let target_dir = vibe_dir.join(&agent_id);

    let root = build_tree_node(
        skills_dir,
        skills_dir,
        &target_dir,
        0,
        &mut std::collections::HashSet::new(),
    );
    Ok(root)
}

/// 递归扫描最大深度，超出后返回 `truncated=true` 的节点（P4）
const MAX_SCAN_DEPTH: usize = 12;

/// 递归构建树节点，带深度上限与链接环保护
fn build_tree_node(
    dir: &Path,
    base_dir: &Path,
    target_dir: &Path,
    depth: usize,
    visited: &mut std::collections::HashSet<std::path::PathBuf>,
) -> SkillsTreeNode {
    let name = dir
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    // 深度超限或遇到已访问目录（链接环）→ 返回截断节点，避免栈溢出/无限递归
    if depth > MAX_SCAN_DEPTH || !visited.insert(vibe_fs::normalize_path(dir)) {
        return SkillsTreeNode {
            name,
            path: dir.to_string_lossy().to_string(),
            is_dir: true,
            skill_count: 0,
            synced: false,
            synced_count: 0,
            children: Vec::new(),
            link_target: None,
            is_source_link: false,
            truncated: true,
        };
    }

    let mut children = Vec::new();
    let mut skill_count = 0;
    let mut synced_count = 0;
    let mut truncated = false;

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let child_name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            if child_name.starts_with('.') {
                continue;
            }

            let has_skill_md = path.join("SKILL.md").exists();

            if has_skill_md {
                skill_count += 1;
                let relative = path.strip_prefix(base_dir).unwrap_or(&path);
                let sync_target = target_dir.join(relative);
                let synced = vibe_fs::is_link(&sync_target);
                if synced {
                    synced_count += 1;
                }

                // 检测源目录本身是否是 symlink
                let is_source_link = vibe_fs::is_link(&path);
                let link_target = if is_source_link {
                    vibe_fs::read_link_target(&path)
                        .ok()
                        .map(|p| p.to_string_lossy().to_string())
                } else {
                    None
                };

                children.push(SkillsTreeNode {
                    name: child_name,
                    path: path.to_string_lossy().to_string(),
                    is_dir: true,
                    skill_count: 1,
                    synced,
                    synced_count: if synced { 1 } else { 0 },
                    children: Vec::new(),
                    link_target,
                    is_source_link,
                    truncated: false,
                });
            } else {
                let child = build_tree_node(&path, base_dir, target_dir, depth + 1, visited);
                if child.truncated {
                    truncated = true;
                }
                skill_count += child.skill_count;
                synced_count += child.synced_count;
                children.push(child);
            }
        }
    }

    let relative = dir.strip_prefix(base_dir).unwrap_or(dir);
    let sync_target = target_dir.join(relative);
    let synced = vibe_fs::is_link(&sync_target) || (sync_target.exists() && synced_count > 0);

    let is_source_link = vibe_fs::is_link(dir) && dir != base_dir;
    let link_target = if is_source_link {
        vibe_fs::read_link_target(dir)
            .ok()
            .map(|p| p.to_string_lossy().to_string())
    } else {
        None
    };

    SkillsTreeNode {
        name,
        path: dir.to_string_lossy().to_string(),
        is_dir: true,
        skill_count,
        synced,
        synced_count,
        children,
        link_target,
        is_source_link,
        truncated,
    }
}
