use std::collections::HashMap;
use std::fs;
use std::path::Path;

use serde_yaml::Value;

use crate::errors::VabError;

/// 解析 SKILL.md 文件，返回 (name, description)
#[allow(dead_code)]
pub fn parse_skill_md(path: &Path) -> Result<(String, String), VabError> {
    let content = fs::read_to_string(path).map_err(|e| VabError::InvalidSkillMd {
        reason: format!("Failed to read {}: {}", path.display(), e),
    })?;

    let (name, description, _, _, _, _) = parse_frontmatter_full(&content)?;
    Ok((name, description))
}

/// 解析 SKILL.md 文件，返回全部字段
pub fn parse_skill_md_full(
    path: &Path,
) -> Result<
    (
        String,
        String,
        Option<String>,
        Option<String>,
        Option<HashMap<String, String>>,
        String,
    ),
    VabError,
> {
    let content = fs::read_to_string(path).map_err(|e| VabError::InvalidSkillMd {
        reason: format!("Failed to read {}: {}", path.display(), e),
    })?;

    parse_frontmatter_full(&content)
}

/// 从 SKILL.md 内容中解析 YAML frontmatter，返回全部字段
fn parse_frontmatter_full(
    content: &str,
) -> Result<
    (
        String,
        String,
        Option<String>,
        Option<String>,
        Option<HashMap<String, String>>,
        String,
    ),
    VabError,
> {
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

    // body 是第二个 --- 之后的内容
    let body_start = 3 + end + 3; // skip first --- + yaml + ---
    let body = if body_start < content.len() {
        content[body_start..].trim().to_string()
    } else {
        String::new()
    };

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

    // 提取字段
    let name = get_string_from_yaml(&map, "name").unwrap_or_default();
    let description = get_string_from_yaml(&map, "description").unwrap_or_default();
    let license = get_string_from_yaml(&map, "license");
    let compatibility = get_string_from_yaml(&map, "compatibility");

    // 提取 metadata (HashMap)
    let metadata = get_metadata_from_yaml(&map, "metadata");

    Ok((name, description, license, compatibility, metadata, body))
}

/// 从 YAML Mapping 中提取字符串值
fn get_string_from_yaml(map: &serde_yaml::Mapping, key: &str) -> Option<String> {
    map.get(&Value::String(key.to_string()))
        .and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => None,
        })
}

/// 从 YAML Mapping 中提取 metadata HashMap
fn get_metadata_from_yaml(
    map: &serde_yaml::Mapping,
    key: &str,
) -> Option<HashMap<String, String>> {
    map.get(&Value::String(key.to_string()))
        .and_then(|v| match v {
            Value::Mapping(m) => {
                let mut result = HashMap::new();
                for (k, v) in m {
                    if let (Value::String(key), Value::String(val)) = (k, v) {
                        result.insert(key.clone(), val.clone());
                    }
                }
                if result.is_empty() {
                    None
                } else {
                    Some(result)
                }
            }
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

        let (name, desc, _, _, _, body) = parse_frontmatter_full(content).unwrap();
        assert_eq!(name, "my-skill");
        assert_eq!(desc, "A test skill");
        assert!(body.contains("Content here"));
    }

    #[test]
    fn test_parse_frontmatter_missing_fields() {
        let content = r#"---
name: my-skill
---
# Content"#;

        let (name, desc, _, _, _, _) = parse_frontmatter_full(content).unwrap();
        assert_eq!(name, "my-skill");
        assert_eq!(desc, "");
    }

    #[test]
    fn test_parse_no_frontmatter() {
        let content = "# Just markdown";
        assert!(parse_frontmatter_full(content).is_err());
    }

    #[test]
    fn test_parse_with_metadata() {
        let content = r#"---
name: test
description: Test skill
license: MIT
metadata:
  author: test-org
  version: "2.0"
---
Body"#;

        let (name, _, license, _, metadata, _) = parse_frontmatter_full(content).unwrap();
        assert_eq!(name, "test");
        assert_eq!(license, Some("MIT".to_string()));
        let meta = metadata.unwrap();
        assert_eq!(meta.get("author").unwrap(), "test-org");
        assert_eq!(meta.get("version").unwrap(), "2.0");
    }
}
