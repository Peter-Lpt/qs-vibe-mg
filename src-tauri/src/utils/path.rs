use std::path::PathBuf;

use crate::errors::VabError;

/// 获取 ~/.vab-skills/ 目录路径
pub fn vab_skills_dir() -> Result<PathBuf, VabError> {
    let home = dirs::home_dir()
        .ok_or_else(|| VabError::Path("Cannot determine home directory".to_string()))?;
    Ok(home.join(".vab-skills"))
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
