//! 内置 agent 定义（与用户配置分开维护，新增 agent 时只需在这里加一条）。
//!
//! 路径约定以各工具官方文档为准（2026）：
//! - Claude Code `~/.claude/skills`、Codex CLI `~/.codex/skills`、OpenCode `~/.config/opencode/skills`
//! - Gemini CLI `~/.gemini/skills`、Qwen Code `~/.qwen/skills`、Cursor `~/.cursor/skills`
//! - Amp `~/.config/amp/skills`（同时支持 `~/.config/agents/skills` 与 `~/.agents/skills` 别名）
//! - `~/.agents/skills` 为跨工具共享目录（OpenCode/Codex/Cursor/Gemini/Cline/Amp 均原生支持）

use crate::utils::config::AgentConfig;

/// 全部内置 agent 的默认配置。仅用于「配置不存在 / 缺失内置项」时的补全，
/// 不会覆盖用户对已有 agent 的自定义。
pub fn default_agents() -> Vec<AgentConfig> {
    vec![
        AgentConfig {
            id: "claude-code".to_string(),
            name: "Claude Code".to_string(),
            skills_dir: "~/.claude/skills".to_string(),
            kind: "agent".to_string(),
            detect_dir: Some("~/.claude".to_string()),
            additional_scan_dirs: Vec::new(),
            enabled: true,
            auto_detected: true,
        },
        AgentConfig {
            id: "hermes".to_string(),
            name: "Hermes".to_string(),
            skills_dir: hermes_skills_dir(),
            kind: "agent".to_string(),
            detect_dir: Some(hermes_detect_dir()),
            additional_scan_dirs: Vec::new(),
            enabled: true,
            auto_detected: true,
        },
        AgentConfig {
            id: "pi-agent".to_string(),
            name: "Pi Agent".to_string(),
            skills_dir: "~/.pi/agent/skills".to_string(),
            kind: "agent".to_string(),
            detect_dir: Some("~/.pi/agent".to_string()),
            additional_scan_dirs: Vec::new(),
            enabled: true,
            auto_detected: true,
        },
        AgentConfig {
            id: "opencode".to_string(),
            name: "OpenCode".to_string(),
            skills_dir: "~/.config/opencode/skills".to_string(),
            kind: "agent".to_string(),
            detect_dir: Some("~/.config/opencode".to_string()),
            additional_scan_dirs: Vec::new(),
            enabled: true,
            auto_detected: true,
        },
        AgentConfig {
            id: "codex".to_string(),
            name: "Codex CLI".to_string(),
            skills_dir: "~/.codex/skills".to_string(),
            kind: "agent".to_string(),
            detect_dir: Some("~/.codex".to_string()),
            additional_scan_dirs: vec!["~/.agents/skills".to_string()],
            enabled: true,
            auto_detected: true,
        },
        AgentConfig {
            id: "gemini-cli".to_string(),
            name: "Gemini CLI".to_string(),
            skills_dir: "~/.gemini/skills".to_string(),
            kind: "agent".to_string(),
            detect_dir: Some("~/.gemini".to_string()),
            additional_scan_dirs: Vec::new(),
            enabled: true,
            auto_detected: true,
        },
        AgentConfig {
            id: "qwen-code".to_string(),
            name: "Qwen Code".to_string(),
            skills_dir: "~/.qwen/skills".to_string(),
            kind: "agent".to_string(),
            detect_dir: Some("~/.qwen".to_string()),
            additional_scan_dirs: Vec::new(),
            enabled: true,
            auto_detected: true,
        },
        AgentConfig {
            id: "cursor".to_string(),
            name: "Cursor".to_string(),
            skills_dir: "~/.cursor/skills".to_string(),
            kind: "agent".to_string(),
            detect_dir: Some("~/.cursor".to_string()),
            additional_scan_dirs: Vec::new(),
            enabled: true,
            auto_detected: true,
        },
        AgentConfig {
            id: "mimocode".to_string(),
            name: "MiMo Code".to_string(),
            skills_dir: "~/.config/mimocode/skills".to_string(),
            kind: "agent".to_string(),
            detect_dir: Some("~/.config/mimocode".to_string()),
            additional_scan_dirs: Vec::new(),
            enabled: true,
            auto_detected: true,
        },
        AgentConfig {
            id: "agents-shared".to_string(),
            name: "Agents Common".to_string(),
            skills_dir: "~/.agents/skills".to_string(),
            kind: "common".to_string(),
            detect_dir: None,
            additional_scan_dirs: Vec::new(),
            enabled: true,
            auto_detected: true,
        },
    ]
}

/// 根据平台返回 hermes skills 目录
fn hermes_skills_dir() -> String {
    #[cfg(windows)]
    {
        // Windows: %LOCALAPPDATA%\hermes\skills
        if let Some(local) = dirs::data_local_dir() {
            let path = local.join("hermes").join("skills");
            if path.exists() {
                return path.to_string_lossy().to_string();
            }
        }
        "~/.hermes/skills".to_string()
    }
    #[cfg(not(windows))]
    {
        "~/.hermes/skills".to_string()
    }
}

fn hermes_detect_dir() -> String {
    #[cfg(windows)]
    {
        if let Some(local) = dirs::data_local_dir() {
            return local.join("hermes").to_string_lossy().to_string();
        }
        "~/.hermes".to_string()
    }
    #[cfg(not(windows))]
    {
        "~/.hermes".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_agent_ids_are_unique() {
        let agents = default_agents();
        let mut ids = std::collections::HashSet::new();
        for agent in &agents {
            assert!(ids.insert(agent.id.clone()), "duplicate id: {}", agent.id);
        }
    }

    #[test]
    fn builtin_agents_have_valid_paths() {
        for agent in default_agents() {
            assert!(!agent.skills_dir.trim().is_empty(), "empty skills_dir: {}", agent.id);
            assert!(
                agent.detect_dir.as_deref().unwrap_or("").trim().is_empty()
                    || agent.detect_dir.as_deref().unwrap().starts_with('~')
                    || agent.detect_dir.as_deref().unwrap().contains("hermes"),
                "detect_dir should be tilde-relative: {}",
                agent.id
            );
        }
    }
}
