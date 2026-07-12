use std::path::PathBuf;

use crate::errors::VibeError;

/// 获取 vibe-skills 目录路径（可配置）
pub fn vibe_skills_dir() -> Result<PathBuf, VibeError> {
    let default_dir = default_vibe_skills_dir()?;
    let config_path = default_dir.join(".vibe-config.json");

    if let Some(path) = read_vibe_skills_path_from_config(&config_path)? {
        let expanded = expand_tilde(&path)?;
        if expanded.exists() {
            return Ok(expanded);
        }
    }

    Ok(default_dir)
}

/// 从配置文件读取 vibe_skills_path 字段
fn read_vibe_skills_path_from_config(config_path: &std::path::Path) -> Result<Option<String>, VibeError> {
    if !config_path.exists() {
        return Ok(None);
    }

    let content = std::fs::read_to_string(config_path)?;
    let config: serde_json::Value =
        serde_json::from_str(&content).map_err(|e| VibeError::Config(e.to_string()))?;

    Ok(config
        .get("vibe_skills_path")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string()))
}

/// 默认的 vibe-skills 目录路径（~/.vibe-skills/）
fn default_vibe_skills_dir() -> Result<PathBuf, VibeError> {
    let home = dirs::home_dir()
        .ok_or_else(|| VibeError::Path("Cannot determine home directory".to_string()))?;
    Ok(home.join(".vibe-skills"))
}

/// 展开 ~ 为用户主目录
pub fn expand_tilde(path: &str) -> Result<PathBuf, VibeError> {
    if path.starts_with("~/") || path.starts_with("~\\") {
        let home = dirs::home_dir()
            .ok_or_else(|| VibeError::Path("Cannot determine home directory".to_string()))?;
        Ok(home.join(&path[2..]))
    } else {
        Ok(PathBuf::from(path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_tilde() {
        let result = expand_tilde("~/test").unwrap();
        assert!(result.to_string_lossy().contains("test"));
        assert!(!result.to_string_lossy().starts_with("~"));
    }

    #[test]
    fn test_expand_no_tilde() {
        let result = expand_tilde("/absolute/path").unwrap();
        assert_eq!(result.to_string_lossy(), "/absolute/path");
    }
}
