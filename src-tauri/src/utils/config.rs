use std::fs;

use serde::{Deserialize, Serialize};

use crate::errors::VabError;
use crate::models::agent::Agent;
use crate::utils::path::{expand_tilde, vab_skills_dir};

const CONFIG_FILE: &str = ".vab-config.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub version: u32,
    #[serde(default = "default_agents")]
    pub agents: Vec<AgentConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub id: String,
    pub name: String,
    pub skills_dir: String,
    pub enabled: bool,
}

fn default_agents() -> Vec<AgentConfig> {
    vec![
        AgentConfig {
            id: "claude-code".to_string(),
            name: "Claude Code".to_string(),
            skills_dir: "~/.claude/skills".to_string(),
            enabled: true,
        },
        AgentConfig {
            id: "hermes".to_string(),
            name: "Hermes".to_string(),
            skills_dir: "~/.hermes/skills".to_string(),
            enabled: true,
        },
        AgentConfig {
            id: "pi-agent".to_string(),
            name: "Pi Agent".to_string(),
            skills_dir: "~/.pi/agent/skills".to_string(),
            enabled: true,
        },
        AgentConfig {
            id: "opencode".to_string(),
            name: "OpenCode".to_string(),
            skills_dir: "~/.config/opencode/skills".to_string(),
            enabled: true,
        },
        AgentConfig {
            id: "codex".to_string(),
            name: "Codex CLI".to_string(),
            skills_dir: "~/.codex/skills".to_string(),
            enabled: true,
        },
        AgentConfig {
            id: "mimocode".to_string(),
            name: "MiMo Code".to_string(),
            skills_dir: "~/.config/mimocode/skills".to_string(),
            enabled: true,
        },
        AgentConfig {
            id: "agents-shared".to_string(),
            name: "Shared".to_string(),
            skills_dir: "~/.agents/skills".to_string(),
            enabled: true,
        },
    ]
}

/// 读取配置文件，不存在则返回默认配置
pub fn load_config() -> Result<Config, VabError> {
    let config_path = vab_skills_dir()?.join(CONFIG_FILE);

    if !config_path.exists() {
        let config = Config {
            version: 1,
            agents: default_agents(),
        };
        save_config(&config)?;
        return Ok(config);
    }

    let content = fs::read_to_string(&config_path)?;
    let config: Config =
        serde_json::from_str(&content).map_err(|e| VabError::Config(e.to_string()))?;
    Ok(config)
}

/// 保存配置文件
pub fn save_config(config: &Config) -> Result<(), VabError> {
    let dir = vab_skills_dir()?;
    if !dir.exists() {
        fs::create_dir_all(&dir)?;
    }

    let config_path = dir.join(CONFIG_FILE);
    let content =
        serde_json::to_string_pretty(config).map_err(|e| VabError::Config(e.to_string()))?;
    fs::write(&config_path, content)?;
    Ok(())
}

/// 从配置构建 Agent 列表（检测是否已安装）
pub fn build_agents_from_config(config: &Config) -> Result<Vec<Agent>, VabError> {
    let mut agents = Vec::new();

    for ac in &config.agents {
        if !ac.enabled {
            continue;
        }

        let skills_dir = expand_tilde(&ac.skills_dir)?;
        let detected = skills_dir.exists();

        agents.push(Agent {
            id: ac.id.clone(),
            name: ac.name.clone(),
            skills_dir: skills_dir.to_string_lossy().to_string(),
            detected,
            enabled: ac.enabled,
        });
    }

    Ok(agents)
}
