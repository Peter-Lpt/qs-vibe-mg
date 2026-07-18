use std::fs;
use std::path::{Path, PathBuf};

use crate::errors::VibeError;
use crate::models::origin::SkillOrigin;
use crate::utils::datetime::chrono_now;

const ORIGIN_FILE: &str = ".vibe-origin.json";

pub fn origin_file_path(skill_dir: &Path) -> PathBuf {
    skill_dir.join(ORIGIN_FILE)
}

pub fn build_install_origin(source_path: &Path) -> SkillOrigin {
    SkillOrigin {
        method: "local-folder".to_string(),
        provider: None,
        url: None,
        commit: None,
        installed_at: chrono_now(),
        installed_by: Some("qs-vibe".to_string()),
        trust_level: "explicit".to_string(),
        source_path: Some(source_path.to_string_lossy().to_string()),
        command: Some(format!("local-folder {}", source_path.to_string_lossy())),
        update_command: None,
        last_checked_at: None,
    }
}

pub fn trust_level_for(origin: Option<&SkillOrigin>) -> String {
    origin
        .map(|o| {
            if o.trust_level.trim().is_empty() {
                "explicit".to_string()
            } else {
                o.trust_level.clone()
            }
        })
        .unwrap_or_else(|| "unknown".to_string())
}

pub fn update_status_for(origin: Option<&SkillOrigin>) -> String {
    let Some(origin) = origin else {
        return "unknown".to_string();
    };

    if origin
        .update_command
        .as_ref()
        .is_some_and(|cmd| !cmd.trim().is_empty())
    {
        return "auto_update".to_string();
    }

    if matches!(
        origin.method.as_str(),
        "git" | "github" | "gitee" | "gitlab"
    ) && origin
        .url
        .as_ref()
        .is_some_and(|url| !url.trim().is_empty())
    {
        return "auto_update".to_string();
    }

    "best_effort".to_string()
}

pub fn read_skill_origin(skill_dir: &Path) -> Option<SkillOrigin> {
    let origin_path = origin_file_path(skill_dir);
    let content = fs::read_to_string(origin_path).ok()?;
    serde_json::from_str::<SkillOrigin>(&content).ok()
}

pub fn write_skill_origin(skill_dir: &Path, origin: &SkillOrigin) -> Result<(), VibeError> {
    let origin_path = origin_file_path(skill_dir);
    let content =
        serde_json::to_string_pretty(origin).map_err(|e| VibeError::Config(e.to_string()))?;
    fs::write(origin_path, content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_install_origin_sets_explicit_trust() {
        let origin = build_install_origin(Path::new("F:/skill-source"));
        assert_eq!(origin.method, "local-folder");
        assert_eq!(origin.trust_level, "explicit");
        assert_eq!(origin.installed_by.as_deref(), Some("qs-vibe"));
        assert_eq!(update_status_for(Some(&origin)), "best_effort");
    }

    #[test]
    fn update_status_requires_origin_for_unknown() {
        assert_eq!(update_status_for(None), "unknown");
    }

    #[test]
    fn update_status_allows_explicit_update_command() {
        let mut origin = build_install_origin(Path::new("F:/skill-source"));
        origin.update_command = Some("git pull".to_string());
        assert_eq!(update_status_for(Some(&origin)), "auto_update");
    }
}
