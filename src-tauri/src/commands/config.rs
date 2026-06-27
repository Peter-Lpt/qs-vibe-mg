use crate::errors::VabError;
use crate::utils::config::{load_config, save_config, Config};

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
