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
        command: None,
        update_command: None,
        last_checked_at: None,
    }
}

pub fn read_skill_origin(skill_dir: &Path) -> Option<SkillOrigin> {
    let origin_path = origin_file_path(skill_dir);
    let content = fs::read_to_string(origin_path).ok()?;
    serde_json::from_str::<SkillOrigin>(&content).ok()
}

pub fn write_skill_origin(skill_dir: &Path, origin: &SkillOrigin) -> Result<(), VibeError> {
    let origin_path = origin_file_path(skill_dir);
    let content = serde_json::to_string_pretty(origin)
        .map_err(|e| VibeError::Config(e.to_string()))?;
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
    }
}
