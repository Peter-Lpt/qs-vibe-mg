use std::path::PathBuf;

use crate::errors::VabError;

/// 获取 vibe-skills 目录路径（可配置）
pub fn vibe_skills_dir() -> Result<PathBuf, VabError> {
    // 尝试从配置文件读取自定义路径
    let config_path = default_vibe_skills_dir()?.join(".vibe-config.json");
    if config_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(config) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(path) = config.get("vibe_skills_path").and_then(|v| v.as_str()) {
                    let expanded = expand_tilde(path)?;
                    if expanded.exists() {
                        return Ok(expanded);
                    }
                }
            }
        }
    }
    default_vibe_skills_dir()
}

/// 默认的 vibe-skills 目录路径（~/.vibe-skills/）
fn default_vibe_skills_dir() -> Result<PathBuf, VabError> {
    let home = dirs::home_dir()
        .ok_or_else(|| VabError::Path("Cannot determine home directory".to_string()))?;
    Ok(home.join(".vibe-skills"))
}

/// 展开 ~ 为用户主目录
pub fn expand_tilde(path: &str) -> Result<PathBuf, VabError> {
    if path.starts_with("~/") || path.starts_with("~\\") {
        let home = dirs::home_dir()
            .ok_or_else(|| VabError::Path("Cannot determine home directory".to_string()))?;
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
