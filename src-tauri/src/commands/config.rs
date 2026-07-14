use std::fs;

use serde::{Deserialize, Serialize};

use crate::errors::VibeError;
use crate::utils::config::{invalidate_agents_cache, load_config, save_config, Config};
use crate::utils::fs::copy_dir_all;
use crate::utils::history::load_history;
use crate::utils::path::{expand_tilde, vibe_skills_dir};

/// 获取配置
#[tauri::command]
pub fn get_config() -> Result<Config, VibeError> {
    load_config()
}

/// 更新配置
#[tauri::command]
pub fn update_config(
    theme: Option<String>,
    locale: Option<String>,
    sync_mode_default: Option<String>,
    max_history: Option<u32>,
) -> Result<Config, VibeError> {
    let mut config = load_config()?;

    if let Some(t) = theme {
        config.ui.theme = t;
    }
    if let Some(l) = locale {
        config.ui.locale = l;
    }
    if let Some(s) = sync_mode_default {
        config.sync_mode_default = s;
    }
    if let Some(m) = max_history {
        config.history.max_entries = m;
    }

    save_config(&config)?;
    Ok(config)
}

/// 设置 vibe-skills 目录路径，可选迁移旧数据
#[tauri::command]
pub fn set_vibe_skills_path(new_path: String, migrate: bool) -> Result<Config, VibeError> {
    let expanded = expand_tilde(&new_path)?;
    let old_dir = vibe_skills_dir()?;

    let mut config = load_config()?;

    // 如果需要迁移，将旧目录内容复制到新目录
    if migrate && old_dir.exists() {
        if !expanded.exists() {
            fs::create_dir_all(&expanded)?;
        }

        // 迁移配置文件
        let old_config_path = old_dir.join(".vibe-config.json");
        let new_config_path = expanded.join(".vibe-config.json");
        if old_config_path.exists() && !new_config_path.exists() {
            fs::copy(&old_config_path, &new_config_path)?;
        }

        // 迁移历史文件
        let old_history_path = old_dir.join(".vibe-history.json");
        let new_history_path = expanded.join(".vibe-history.json");
        if old_history_path.exists() && !new_history_path.exists() {
            fs::copy(&old_history_path, &new_history_path)?;
        }

        // 迁移 skill 目录
        for entry in fs::read_dir(&old_dir)? {
            let entry = entry?;
            let path = entry.path();
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            // 跳过配置和历史文件
            if name.starts_with('.') {
                continue;
            }

            let dest = expanded.join(&name);
            if !dest.exists() {
                if path.is_dir() {
                    copy_dir_all(&path, &dest)?;
                } else {
                    fs::copy(&path, &dest)?;
                }
            }
        }
    }

    // 更新配置
    config.vibe_skills_path = Some(new_path);
    save_config(&config)?;
    invalidate_agents_cache(); // vibe 目录可能变更，agent 缓存失效（P5）

    Ok(config)
}

#[derive(Serialize, Deserialize)]
pub struct ExportData {
    pub config: Config,
    pub history: crate::models::history::HistoryStore,
}

/// Export config and history as JSON string
#[tauri::command]
pub fn export_data() -> Result<String, VibeError> {
    let config = load_config()?;
    let history = load_history()?;
    let data = ExportData { config, history };
    serde_json::to_string_pretty(&data).map_err(|e| VibeError::History(e.to_string()))
}

/// Import config and history from JSON string
#[tauri::command]
pub fn import_data(json: String) -> Result<(), VibeError> {
    let data: ExportData =
        serde_json::from_str(&json).map_err(|e| VibeError::History(e.to_string()))?;
    save_config(&data.config)?;
    crate::utils::history::save_history(&data.history)?;
    invalidate_agents_cache();
    Ok(())
}

/// Write content to a file path (used by export)（P6：拒绝 `..` 字符串逃逸）
#[tauri::command]
pub fn write_file_to_path(path: String, content: String) -> Result<(), VibeError> {
    if path.contains("..") {
        return Err(VibeError::Path(
            "write_file_to_path 不允许路径包含 '..'".to_string(),
        ));
    }
    fs::write(&path, content).map_err(VibeError::Io)
}

/// Read content from a file path (used by import)（P6：拒绝 `..` 字符串逃逸）
#[tauri::command]
pub fn read_file_from_path(path: String) -> Result<String, VibeError> {
    if path.contains("..") {
        return Err(VibeError::Path(
            "read_file_from_path 不允许路径包含 '..'".to_string(),
        ));
    }
    fs::read_to_string(&path).map_err(VibeError::Io)
}
