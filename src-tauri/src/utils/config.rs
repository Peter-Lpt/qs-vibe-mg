use std::fs;

use serde::{Deserialize, Serialize};

use crate::errors::VabError;
use crate::models::agent::Agent;
use crate::utils::path::{expand_tilde, vab_skills_dir};

const CONFIG_FILE: &str = ".vab-config.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub version: u32,
    #[serde(default = "default_sync_mode")]
    pub sync_mode_default: String,
    #[serde(default = "default_agents")]
    pub agents: Vec<AgentConfig>,
    #[serde(default)]
    pub ui: UiConfig,
    #[serde(default)]
    pub history: HistoryConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_locale")]
    pub locale: String,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            locale: default_locale(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryConfig {
    #[serde(default = "default_max_entries")]
    pub max_entries: u32,
    #[serde(default = "default_snapshot_max_size_mb")]
    pub snapshot_max_size_mb: u64,
}

impl Default for HistoryConfig {
    fn default() -> Self {
        Self {
            max_entries: default_max_entries(),
            snapshot_max_size_mb: default_snapshot_max_size_mb(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub id: String,
    pub name: String,
    pub skills_dir: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_true")]
    pub auto_detected: bool,
}

fn default_sync_mode() -> String {
    "symlink".to_string()
}

fn default_theme() -> String {
    "system".to_string()
}

fn default_locale() -> String {
    "zh".to_string()
}

fn default_max_entries() -> u32 {
    50
}

fn default_snapshot_max_size_mb() -> u64 {
    100
}

fn default_true() -> bool {
    true
}

pub fn default_agents() -> Vec<AgentConfig> {
    vec![
        AgentConfig {
            id: "claude-code".to_string(),
            name: "Claude Code".to_string(),
            skills_dir: "~/.claude/skills".to_string(),
            enabled: true,
            auto_detected: true,
        },
        AgentConfig {
            id: "hermes".to_string(),
            name: "Hermes".to_string(),
            skills_dir: "~/.hermes/skills".to_string(),
            enabled: true,
            auto_detected: true,
        },
        AgentConfig {
            id: "pi-agent".to_string(),
            name: "Pi Agent".to_string(),
            skills_dir: "~/.pi/agent/skills".to_string(),
            enabled: true,
            auto_detected: true,
        },
        AgentConfig {
            id: "opencode".to_string(),
            name: "OpenCode".to_string(),
            skills_dir: "~/.config/opencode/skills".to_string(),
            enabled: true,
            auto_detected: true,
        },
        AgentConfig {
            id: "codex".to_string(),
            name: "Codex CLI".to_string(),
            skills_dir: "~/.codex/skills".to_string(),
            enabled: true,
            auto_detected: true,
        },
        AgentConfig {
            id: "mimocode".to_string(),
            name: "MiMo Code".to_string(),
            skills_dir: "~/.config/mimocode/skills".to_string(),
            enabled: true,
            auto_detected: true,
        },
        AgentConfig {
            id: "agents-shared".to_string(),
            name: "Shared".to_string(),
            skills_dir: "~/.agents/skills".to_string(),
            enabled: true,
            auto_detected: true,
        },
    ]
}

/// 读取配置文件，不存在则返回默认配置
pub fn load_config() -> Result<Config, VabError> {
    let config_path = vab_skills_dir()?.join(CONFIG_FILE);

    if !config_path.exists() {
        let config = Config {
            version: 1,
            sync_mode_default: default_sync_mode(),
            agents: default_agents(),
            ui: UiConfig::default(),
            history: HistoryConfig::default(),
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

        // Scan linked skills
        let linked_skills = if detected {
            scan_linked_skills(&skills_dir)
        } else {
            Vec::new()
        };

        agents.push(Agent {
            id: ac.id.clone(),
            name: ac.name.clone(),
            skills_dir: skills_dir.to_string_lossy().to_string(),
            detected,
            enabled: ac.enabled,
            auto_detected: ac.auto_detected,
            linked_skills,
        });
    }

    Ok(agents)
}

/// 扫描 agent skills 目录中的 symlink，返回关联的 skill id 列表
fn scan_linked_skills(skills_dir: &std::path::Path) -> Vec<String> {
    use crate::utils::fs as vab_fs;
    use crate::utils::path::vab_skills_dir;

    let mut linked = Vec::new();
    let vab_dir = match vab_skills_dir() {
        Ok(d) => d,
        Err(_) => return linked,
    };

    if !skills_dir.exists() {
        return linked;
    }

    if let Ok(entries) = fs::read_dir(skills_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if vab_fs::is_link(&path) {
                if let Ok(target) = vab_fs::read_link_target(&path) {
                    // Check if target is under vab-skills dir
                    if let Ok(stripped) = target.strip_prefix(&vab_dir) {
                        if let Some(skill_id) = stripped.file_name() {
                            linked.push(skill_id.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }
    }

    linked
}
