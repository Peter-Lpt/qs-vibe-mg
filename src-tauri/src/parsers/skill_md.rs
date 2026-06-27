use std::fs;
use std::path::Path;

use serde_yaml::Value;

use crate::errors::VabError;

/// 解析 SKILL.md 文件，提取 frontmatter 中的 name 和 description
/// 返回 (name, description)
pub fn parse_skill_md(path: &Path) -> Result<(String, String), VabError> {
    let content = fs::read_to_string(path).map_err(|e| VabError::InvalidSkillMd {
        reason: format!("Failed to read {}: {}", path.display(), e),
    })?;

    parse_frontmatter(&content)
}

/// 从 SKILL.md 内容中解析 YAML frontmatter
fn parse_frontmatter(content: &str) -> Result<(String, String), VabError> {
    let content = content.trim();

    // frontmatter 必须以 --- 开头
    if !content.starts_with("---") {
        return Err(VabError::InvalidSkillMd {
            reason: "Missing frontmatter (must start with ---)".to_string(),
        });
    }

    // 找到第二个 ---
    let after_first = &content[3..];
    let end = after_first
        .find("---")
        .ok_or_else(|| VabError::InvalidSkillMd {
            reason: "Missing closing --- in frontmatter".to_string(),
        })?;

    let yaml_str = &after_first[..end].trim();

    // 解析 YAML
    let value: Value = serde_yaml::from_str(yaml_str).map_err(|e| VabError::InvalidSkillMd {
        reason: format!("YAML parse error: {}", e),
    })?;

    let map = match value {
        Value::Mapping(m) => m,
        _ => {
            return Err(VabError::InvalidSkillMd {
                reason: "Frontmatter must be a YAML mapping".to_string(),
            });
        }
    };

    // 提取 name
    let name = get_string_from_yaml(&map, "name").unwrap_or_default();

    // 提取 description
    let description = get_string_from_yaml(&map, "description").unwrap_or_default();

    Ok((name, description))
}

/// 从 YAML Mapping 中提取字符串值
fn get_string_from_yaml(map: &serde_yaml::Mapping, key: &str) -> Option<String> {
    map.get(&Value::String(key.to_string()))
        .and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => None,
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_frontmatter() {
        let content = r#"---
name: my-skill
description: A test skill
---
# Content here"#;

        let (name, desc) = parse_frontmatter(content).unwrap();
        assert_eq!(name, "my-skill");
        assert_eq!(desc, "A test skill");
    }

    #[test]
    fn test_parse_frontmatter_missing_fields() {
        let content = r#"---
name: my-skill
---
# Content"#;

        let (name, desc) = parse_frontmatter(content).unwrap();
        assert_eq!(name, "my-skill");
        assert_eq!(desc, "");
    }

    #[test]
    fn test_parse_no_frontmatter() {
        let content = "# Just markdown";
        assert!(parse_frontmatter(content).is_err());
    }
}
