use std::fs;

use crate::errors::VibeError;
use crate::utils::config::{
    invalidate_agents_cache,
    load_config,
    normalize_project_roots,
    save_config,
    suggest_project_roots as build_project_root_suggestions,
    Config,
    ProjectRootSuggestion,
};
use crate::utils::fs::copy_dir_all;
use crate::utils::path::{expand_tilde, vibe_skills_dir};

/// 获取配置
#[tauri::command]
pub async fn get_config() -> Result<Config, VibeError> {
    tauri::async_runtime::spawn_blocking(load_config)
        .await
        .map_err(|error| VibeError::Path(format!("get_config task failed: {}", error)))?
}

#[tauri::command]
pub async fn suggest_project_roots() -> Result<Vec<ProjectRootSuggestion>, VibeError> {
    tauri::async_runtime::spawn_blocking(|| Ok(build_project_root_suggestions()))
        .await
        .map_err(|error| VibeError::Path(format!("suggest_project_roots task failed: {}", error)))?
}

/// 更新配置
#[tauri::command]
pub async fn update_config(
    theme: Option<String>,
    locale: Option<String>,
    sync_mode_default: Option<String>,
    max_history: Option<u32>,
    project_roots: Option<Vec<String>>,
    auto_check_updates: Option<bool>,
) -> Result<Config, VibeError> {
    tauri::async_runtime::spawn_blocking(move || {
        update_config_sync(theme, locale, sync_mode_default, max_history, project_roots, auto_check_updates)
    })
    .await
    .map_err(|error| VibeError::Path(format!("update_config task failed: {}", error)))?
}

fn update_config_sync(
    theme: Option<String>,
    locale: Option<String>,
    sync_mode_default: Option<String>,
    max_history: Option<u32>,
    project_roots: Option<Vec<String>>,
    auto_check_updates: Option<bool>,
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
        // H4：钳制下限，避免 record_action 对空历史 remove(0) panic
        config.history.max_entries = m.max(1);
    }
    if let Some(roots) = project_roots {
        config.project_roots = normalize_project_roots(roots);
    }
    if let Some(auto) = auto_check_updates {
        config.ui.auto_check_updates = auto;
    }

    save_config(&config)?;
    Ok(config)
}

/// 设置 vibe-skills 目录路径，可选迁移旧数据
#[tauri::command]
pub async fn set_vibe_skills_path(new_path: String, migrate: bool) -> Result<Config, VibeError> {
    tauri::async_runtime::spawn_blocking(move || set_vibe_skills_path_sync(new_path, migrate))
        .await
        .map_err(|error| VibeError::Path(format!("set_vibe_skills_path task failed: {}", error)))?
}

fn set_vibe_skills_path_sync(new_path: String, migrate: bool) -> Result<Config, VibeError> {
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

        // 迁移 .trash 回收站（含可恢复的删除快照），避免迁移后历史删除无法找回
        let old_trash = old_dir.join(".trash");
        let new_trash = expanded.join(".trash");
        if old_trash.exists() && !new_trash.exists() {
            copy_dir_all(&old_trash, &new_trash)?;
        }

        // 迁移 skill 目录
        for entry in fs::read_dir(&old_dir)? {
            let entry = entry?;
            let path = entry.path();
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            // 跳过配置和历史文件（.trash 已在上面单独迁移）
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
