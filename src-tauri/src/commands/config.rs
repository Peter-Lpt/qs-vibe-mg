use std::fs;

use crate::errors::VabError;
use crate::utils::config::{load_config, save_config, Config};
use crate::utils::fs::copy_dir_all;
use crate::utils::path::{expand_tilde, vibe_skills_dir};

/// 获取配置
#[tauri::command]
pub fn get_config() -> Result<Config, VabError> {
    load_config()
}

/// 更新配置
#[tauri::command]
pub fn update_config(
    theme: Option<String>,
    locale: Option<String>,
    sync_mode_default: Option<String>,
    max_history: Option<u32>,
) -> Result<Config, VabError> {
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
pub fn set_vibe_skills_path(new_path: String, migrate: bool) -> Result<Config, VabError> {
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

    Ok(config)
}
