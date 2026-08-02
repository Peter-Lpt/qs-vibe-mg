use std::fs;

use serde::{Deserialize, Serialize};

use crate::errors::VibeError;
use crate::models::agent::Agent;
use crate::utils::builtin_agents::default_agents;
use crate::utils::path::{display_path, expand_tilde, vibe_skills_dir};

const CONFIG_FILE: &str = ".vibe-config.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub version: u32,
    #[serde(default = "default_sync_mode")]
    pub sync_mode_default: String,
    #[serde(default = "default_agents")]
    pub agents: Vec<AgentConfig>,
    #[serde(default)]
    pub project_roots: Vec<String>,
    #[serde(default)]
    pub ui: UiConfig,
    #[serde(default)]
    pub history: HistoryConfig,
    /// 自定义 vibe-skills 目录路径（None 表示使用默认 ~/.vibe-skills/）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vibe_skills_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_locale")]
    pub locale: String,
    /// 关闭按钮行为：minimize_to_tray 或 close
    #[serde(default = "default_close_behavior")]
    pub close_behavior: String,
    /// 启动时后台静默检查技能更新（默认开启）
    #[serde(default = "default_true")]
    pub auto_check_updates: bool,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            locale: default_locale(),
            close_behavior: default_close_behavior(),
            auto_check_updates: default_true(),
        }
    }
}

fn default_close_behavior() -> String {
    "ask".to_string()
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
    #[serde(default = "default_agent_kind")]
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detect_dir: Option<String>,
    #[serde(default)]
    pub additional_scan_dirs: Vec<String>,
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

fn default_agent_kind() -> String {
    "agent".to_string()
}

fn default_project_roots() -> Vec<String> {
    Vec::new()
}

pub fn normalize_project_roots(roots: Vec<String>) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut normalized = Vec::new();

    for root in roots {
        let trimmed = root.trim();
        if trimmed.is_empty() {
            continue;
        }
        let value = trimmed.replace('\\', "/");
        if seen.insert(value.clone()) {
            normalized.push(value);
        }
    }

    normalized
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectRootSuggestion {
    pub path: String,
    pub is_current: bool,
    pub matched_dirs: Vec<String>,
}

fn normalize_agent_kind(id: &str, kind: &str) -> String {
    if id == "agents-shared" || id == "agents-common" {
        "common".to_string()
    } else if kind.trim().is_empty() {
        default_agent_kind()
    } else {
        kind.to_string()
    }
}


/// 读取配置文件，不存在则返回默认配置；损坏时回退默认而非中断（P5）。
/// 已有配置会补齐缺失的内置 agent（仅 auto_detected 项，不覆盖用户自定义）。
pub fn load_config() -> Result<Config, VibeError> {
    let config_path = vibe_skills_dir()?.join(CONFIG_FILE);

    if !config_path.exists() {
        let config = default_config();
        save_config(&config)?;
        return Ok(config);
    }

    let content = match fs::read_to_string(&config_path) {
        Ok(c) => c,
        Err(e) => return Err(VibeError::Config(e.to_string())),
    };

    let mut config = match serde_json::from_str::<Config>(&content) {
        Ok(config) => config,
        Err(e) => {
            tracing::warn!("Config corrupt, falling back to default: {}", e);
            let backup_path = config_path.with_extension("json.bak");
            let _ = fs::copy(&config_path, &backup_path);
            let config = default_config();
            let _ = save_config(&config);
            return Ok(config);
        }
    };

    ensure_builtin_agents(&mut config)?;
    Ok(config)
}

/// 同步内置 agent：补齐缺失的新内置，并移除已下架的内置（auto_detected 项）。
fn ensure_builtin_agents(config: &mut Config) -> Result<(), VibeError> {
    let builtin = default_agents();
    let builtin_ids: std::collections::HashSet<&str> =
        builtin.iter().map(|agent| agent.id.as_str()).collect();
    let mut changed = false;

    let before = config.agents.len();
    config.agents.retain(|agent| {
        !(agent.auto_detected && !builtin_ids.contains(agent.id.as_str()))
    });
    if config.agents.len() != before {
        changed = true;
    }

    let existing: std::collections::HashSet<&str> =
        config.agents.iter().map(|agent| agent.id.as_str()).collect();
    let missing: Vec<AgentConfig> = builtin
        .into_iter()
        .filter(|agent| agent.auto_detected && !existing.contains(agent.id.as_str()))
        .collect();
    if !missing.is_empty() {
        tracing::info!(
            "Adding missing built-in agents: {:?}",
            missing.iter().map(|a| &a.id).collect::<Vec<_>>()
        );
        config.agents.extend(missing);
        changed = true;
    }

    if changed {
        save_config(config)
    } else {
        Ok(())
    }
}

/// 默认配置（集中构造，供 load/save 共用）
pub fn default_config() -> Config {
    Config {
        version: 1,
        sync_mode_default: default_sync_mode(),
        agents: default_agents(),
        project_roots: default_project_roots(),
        ui: UiConfig::default(),
        history: HistoryConfig::default(),
        vibe_skills_path: None,
    }
}

/// 保存配置文件（P5：临时文件 + 原子 rename，避免中途写入损坏）
pub fn save_config(config: &Config) -> Result<(), VibeError> {
    let dir = vibe_skills_dir()?;
    if !dir.exists() {
        fs::create_dir_all(&dir)?;
    }

    let config_path = dir.join(CONFIG_FILE);
    let content =
        serde_json::to_string_pretty(config).map_err(|e| VibeError::Config(e.to_string()))?;

    let tmp = dir.join(format!("{}.tmp", CONFIG_FILE));
    fs::write(&tmp, &content)?;
    fs::rename(&tmp, &config_path)?;
    Ok(())
}

/// 从配置构建 Agent 列表（检测是否已安装）
pub fn build_agents_from_config(config: &Config) -> Result<Vec<Agent>, VibeError> {
    let mut agents = Vec::new();

    for ac in &config.agents {
        let skills_dir = expand_tilde(&ac.skills_dir)?;
        let detected = skills_dir.exists();
        let detect_dir = ac.detect_dir.as_ref().and_then(|dir| expand_tilde(dir).ok());
        let additional_scan_dirs = ac
            .additional_scan_dirs
            .iter()
            .filter_map(|dir| expand_tilde(dir).ok())
            .filter(|dir| dir.exists() && dir.is_dir())
            .map(|dir| dir.to_string_lossy().to_string())
            .collect();
        let tool_detected = detect_dir
            .as_ref()
            .map(|dir| dir.exists())
            .unwrap_or(detected);

        let linked_skills = if detected {
            scan_linked_skills(&skills_dir)
        } else {
            Vec::new()
        };

        agents.push(Agent {
            id: ac.id.clone(),
            name: ac.name.clone(),
            skills_dir: display_path(&skills_dir),
            kind: normalize_agent_kind(&ac.id, &ac.kind),
            detect_dir: detect_dir.map(|dir| display_path(&dir)),
            additional_scan_dirs,
            tool_detected,
            detected,
            enabled: ac.enabled,
            auto_detected: ac.auto_detected,
            linked_skills,
        });
    }

    Ok(agents)
}

pub fn project_skill_roots(config: &Config) -> Vec<std::path::PathBuf> {
    let mut roots = Vec::new();
    for root in &config.project_roots {
        if let Ok(expanded) = expand_tilde(root) {
            if expanded.exists() && expanded.is_dir() {
                roots.push(expanded);
            }
        }
    }

    roots
}

pub fn suggest_project_roots() -> Vec<ProjectRootSuggestion> {
    let cwd = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(_) => return Vec::new(),
    };

    let mut suggestions = Vec::new();
    let mut seen = std::collections::HashSet::new();
    let mut current = Some(cwd.as_path());

    while let Some(dir) = current {
        let normalized = dir.to_string_lossy().replace('\\', "/");
        if !seen.insert(normalized.clone()) {
            break;
        }

        let mut matched_dirs = Vec::new();
        for relative in [".claude/skills", ".agents/skills", ".codex/skills", ".github/skills", "skills"] {
            if dir.join(relative).is_dir() {
                matched_dirs.push(relative.to_string());
            }
        }

        let has_git = dir.join(".git").exists();
        if dir == cwd.as_path() || has_git || !matched_dirs.is_empty() {
            suggestions.push(ProjectRootSuggestion {
                path: normalized,
                is_current: dir == cwd.as_path(),
                matched_dirs,
            });
        }

        current = dir.parent();
    }

    suggestions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_project_roots_trims_and_deduplicates() {
        let roots = normalize_project_roots(vec![
            "  F:\\workspace\\demo\\repo  ".to_string(),
            "F:/workspace/demo/repo".to_string(),
            "".to_string(),
            "D:\\workspace\\other".to_string(),
        ]);

        assert_eq!(roots, vec![
            "F:/workspace/demo/repo".to_string(),
            "D:/workspace/other".to_string(),
        ]);
    }
}

/// 扫描 agent skills 目录中的 symlink，返回关联的 skill id 列表（P2 亦供 skills 命令统一调用）
pub fn scan_linked_skills(skills_dir: &std::path::Path) -> Vec<String> {
    use crate::utils::fs as vibe_fs;
    use crate::utils::path::vibe_skills_dir;

    let mut linked = Vec::new();
    let vibe_dir = match vibe_skills_dir() {
        Ok(d) => d,
        Err(_) => return linked,
    };

    if !skills_dir.exists() {
        return linked;
    }

    if let Ok(entries) = fs::read_dir(skills_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if vibe_fs::is_link(&path) {
                if let Ok(target) = vibe_fs::read_link_target(&path) {
                    if let Ok(stripped) = target.strip_prefix(&vibe_dir) {
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

// ── P5：agent 列表缓存 ───────────────────────────────────────────────────
// 避免每次命令重复解析配置并 `exists()` 探测各 agent 目录。仅在 agent 配置变更或
// 链接状态变更后失效（invalidate_agents_cache）。
struct AgentsCache {
    dir: std::path::PathBuf,
    agents: Vec<crate::models::agent::Agent>,
}

static AGENTS_CACHE: std::sync::OnceLock<std::sync::Mutex<Option<AgentsCache>>> =
    std::sync::OnceLock::new();

fn cache_cell() -> &'static std::sync::Mutex<Option<AgentsCache>> {
    AGENTS_CACHE.get_or_init(|| std::sync::Mutex::new(None))
}

/// 使 agent 缓存失效（add/update/remove agent、set_vibe_skills_path、所有链接变更后调用）
pub fn invalidate_agents_cache() {
    *cache_cell().lock().unwrap() = None;
}

/// 读取 agent 列表；命中缓存且 vibe 目录未变时直接返回（P5）
pub fn load_agents() -> Result<Vec<crate::models::agent::Agent>, VibeError> {
    let dir = vibe_skills_dir()?;
    {
        let guard = cache_cell().lock().unwrap();
        if let Some(c) = guard.as_ref() {
            if c.dir == dir {
                return Ok(c.agents.clone());
            }
        }
    }
    let config = load_config()?;
    let agents = build_agents_from_config(&config)?;
    *cache_cell().lock().unwrap() = Some(AgentsCache { dir, agents: agents.clone() });
    Ok(agents)
}
