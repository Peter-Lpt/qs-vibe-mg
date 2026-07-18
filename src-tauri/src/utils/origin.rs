use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::errors::VibeError;
use crate::models::origin::SkillOrigin;
use crate::utils::datetime::chrono_now;

const ORIGIN_FILE: &str = ".vibe-origin.json";
const ORIGIN_SIDECAR_DIR: &str = ".vibe-origin";
pub const SOURCE_METHOD_LOCAL_FOLDER: &str = "local-folder";
pub const SOURCE_METHOD_GIT: &str = "git";
pub const SOURCE_METHOD_NPM: &str = "npm";
pub const SOURCE_METHOD_NPX: &str = "npx";
pub const SOURCE_METHOD_MARKETPLACE: &str = "marketplace";
pub const UPDATE_STATUS_AUTO: &str = "auto_update";
pub const UPDATE_STATUS_BEST_EFFORT: &str = "best_effort";
pub const UPDATE_STATUS_UNKNOWN: &str = "unknown";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitProbe {
    pub remote_url: String,
    pub commit: String,
    pub branch: Option<String>,
}

pub fn origin_file_path(skill_dir: &Path) -> PathBuf {
    skill_dir.join(ORIGIN_FILE)
}

pub fn origin_sidecar_file_path(skill_dir: &Path) -> Option<PathBuf> {
    let parent = skill_dir.parent()?;
    let name = skill_dir.file_name()?.to_string_lossy().to_string();
    Some(parent.join(ORIGIN_SIDECAR_DIR).join(format!("{}.json", name)))
}

pub fn build_install_origin(source_path: &Path) -> SkillOrigin {
    if let Some(probe) = probe_git_origin(source_path) {
        return build_git_origin(source_path, &probe);
    }

    SkillOrigin {
        method: SOURCE_METHOD_LOCAL_FOLDER.to_string(),
        provider: None,
        url: None,
        commit: None,
        branch: None,
        installed_at: chrono_now(),
        installed_by: Some("qs-vibe".to_string()),
        trust_level: "explicit".to_string(),
        source_path: Some(source_path.to_string_lossy().to_string()),
        command: Some(format!("local-folder {}", source_path.to_string_lossy())),
        update_command: None,
        last_checked_at: None,
    }
}

pub fn build_git_origin(source_path: &Path, probe: &GitProbe) -> SkillOrigin {
    SkillOrigin {
        method: SOURCE_METHOD_GIT.to_string(),
        provider: infer_provider_from_url(&probe.remote_url),
        url: Some(probe.remote_url.clone()),
        commit: Some(probe.commit.clone()),
        branch: probe.branch.clone(),
        installed_at: chrono_now(),
        installed_by: Some("qs-vibe".to_string()),
        trust_level: "explicit".to_string(),
        source_path: Some(source_path.to_string_lossy().to_string()),
        command: Some(format!("git-source {}", source_path.to_string_lossy())),
        update_command: Some("git pull --ff-only".to_string()),
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

#[allow(dead_code)]
pub fn source_method_for(origin: Option<&SkillOrigin>) -> String {
    origin
        .map(|o| normalize_source_method(&o.method))
        .unwrap_or_else(|| UPDATE_STATUS_UNKNOWN.to_string())
}

pub fn update_status_for(origin: Option<&SkillOrigin>, skill_dir: Option<&Path>) -> String {
    let Some(origin) = origin else {
        return UPDATE_STATUS_UNKNOWN.to_string();
    };

    if origin
        .update_command
        .as_ref()
        .is_some_and(|cmd| !cmd.trim().is_empty())
    {
        return UPDATE_STATUS_AUTO.to_string();
    }

    let method = normalize_source_method(&origin.method);

    if method == SOURCE_METHOD_GIT {
        if origin
            .url
            .as_ref()
            .is_some_and(|url| !url.trim().is_empty())
            || skill_dir.and_then(probe_git_origin).is_some()
        {
            return UPDATE_STATUS_AUTO.to_string();
        }
        return UPDATE_STATUS_BEST_EFFORT.to_string();
    }

    if matches!(
        method.as_str(),
        SOURCE_METHOD_NPM | SOURCE_METHOD_NPX | SOURCE_METHOD_MARKETPLACE
    ) {
        return UPDATE_STATUS_BEST_EFFORT.to_string();
    }

    if method == SOURCE_METHOD_LOCAL_FOLDER {
        if skill_dir.and_then(probe_git_origin).is_some() {
            return UPDATE_STATUS_AUTO.to_string();
        }
        return UPDATE_STATUS_BEST_EFFORT.to_string();
    }

    UPDATE_STATUS_UNKNOWN.to_string()
}

pub fn normalize_source_method(method: &str) -> String {
    match method.trim().to_ascii_lowercase().as_str() {
        "local-folder" | "local_folder" | "folder" | "local" => SOURCE_METHOD_LOCAL_FOLDER.to_string(),
        "git" | "github" | "gitee" | "gitlab" => SOURCE_METHOD_GIT.to_string(),
        "npm" => SOURCE_METHOD_NPM.to_string(),
        "npx" => SOURCE_METHOD_NPX.to_string(),
        "marketplace" | "market" => SOURCE_METHOD_MARKETPLACE.to_string(),
        other if other.is_empty() => UPDATE_STATUS_UNKNOWN.to_string(),
        other => other.to_string(),
    }
}

pub fn infer_provider_from_url(url: &str) -> Option<String> {
    let lower = url.to_ascii_lowercase();
    if lower.contains("github.com") {
        Some("github".to_string())
    } else if lower.contains("gitee.com") {
        Some("gitee".to_string())
    } else if lower.contains("gitlab.com") {
        Some("gitlab".to_string())
    } else {
        None
    }
}

pub fn probe_git_origin(path: &Path) -> Option<GitProbe> {
    if !path.exists() {
        return None;
    }

    let remote_url = run_git(path, ["remote", "get-url", "origin"])?;
    let commit = run_git(path, ["rev-parse", "HEAD"])?;
    let branch = run_git(path, ["rev-parse", "--abbrev-ref", "HEAD"]);

    Some(GitProbe {
        remote_url,
        commit,
        branch,
    })
}

pub fn git_status_clean(path: &Path) -> Result<bool, VibeError> {
    let output = Command::new("git")
        .arg("-C")
        .arg(path)
        .args(["status", "--porcelain"])
        .output()
        .map_err(VibeError::Io)?;

    if !output.status.success() {
        return Err(VibeError::Path(format!(
            "无法检查 Git 工作区状态：{}",
            path.display()
        )));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().is_empty())
}

pub fn git_pull_ff_only(path: &Path) -> Result<(), VibeError> {
    let output = Command::new("git")
        .arg("-C")
        .arg(path)
        .args(["pull", "--ff-only"])
        .output()
        .map_err(VibeError::Io)?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    Err(VibeError::Path(format!(
        "Git 拉取失败：{}。{}",
        path.display(),
        if stderr.is_empty() {
            "请检查远端、权限或本地冲突".to_string()
        } else {
            stderr
        }
    )))
}

pub fn run_update_command(command: &str, cwd: Option<&Path>) -> Result<(), VibeError> {
    let mut cmd = if cfg!(windows) {
        let mut c = Command::new("cmd");
        c.arg("/C").arg(command);
        c
    } else {
        let mut c = Command::new("sh");
        c.arg("-lc").arg(command);
        c
    };

    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }

    let output = cmd.output().map_err(VibeError::Io)?;
    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    Err(VibeError::Path(format!(
        "更新命令执行失败：{}。{}",
        command,
        if stderr.is_empty() {
            "请检查命令可用性、权限或来源配置".to_string()
        } else {
            stderr
        }
    )))
}

pub fn refresh_git_origin(origin: &mut SkillOrigin, probe: &GitProbe) {
    origin.method = SOURCE_METHOD_GIT.to_string();
    origin.provider = infer_provider_from_url(&probe.remote_url).or_else(|| origin.provider.clone());
    origin.url = Some(probe.remote_url.clone());
    origin.commit = Some(probe.commit.clone());
    origin.branch = probe.branch.clone();
    origin.update_command = Some("git pull --ff-only".to_string());
    origin.last_checked_at = Some(chrono_now());
    if origin.trust_level.trim().is_empty() {
        origin.trust_level = "explicit".to_string();
    }
}

fn run_git<const N: usize>(path: &Path, args: [&str; N]) -> Option<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(path)
        .args(args)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let text = String::from_utf8(output.stdout).ok()?;
    let trimmed = text.trim().to_string();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

pub fn read_skill_origin(skill_dir: &Path) -> Option<SkillOrigin> {
    if let Some(origin) = read_skill_origin_direct_or_sidecar(skill_dir) {
        return Some(origin);
    }

    if let Ok(target) = crate::utils::fs::read_link_target(skill_dir) {
        if let Some(origin) = read_skill_origin_direct_or_sidecar(&target) {
            return Some(origin);
        }
    }

    None
}

pub fn write_skill_origin(skill_dir: &Path, origin: &SkillOrigin) -> Result<(), VibeError> {
    write_skill_origin_to_path(&origin_file_path(skill_dir), origin)
}

pub fn write_skill_origin_sidecar(skill_dir: &Path, origin: &SkillOrigin) -> Result<(), VibeError> {
    let path = origin_sidecar_file_path(skill_dir).ok_or_else(|| {
        VibeError::Path(format!(
            "无法为 {} 生成 provenance 侧边车路径",
            skill_dir.display()
        ))
    })?;
    write_skill_origin_to_path(&path, origin)
}

fn write_skill_origin_to_path(path: &Path, origin: &SkillOrigin) -> Result<(), VibeError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let content =
        serde_json::to_string_pretty(origin).map_err(|e| VibeError::Config(e.to_string()))?;
    fs::write(path, content)?;
    Ok(())
}

fn read_skill_origin_from_path(path: &Path) -> Option<SkillOrigin> {
    let content = fs::read_to_string(path).ok()?;
    serde_json::from_str::<SkillOrigin>(&content).ok()
}

fn read_skill_origin_direct_or_sidecar(skill_dir: &Path) -> Option<SkillOrigin> {
    if let Some(origin) = read_skill_origin_from_path(&origin_file_path(skill_dir)) {
        return Some(origin);
    }
    if let Some(sidecar_path) = origin_sidecar_file_path(skill_dir) {
        if let Some(origin) = read_skill_origin_from_path(&sidecar_path) {
            return Some(origin);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_install_origin_sets_explicit_trust() {
        let origin = build_install_origin(Path::new("F:/skill-source"));
        assert_eq!(origin.method, SOURCE_METHOD_LOCAL_FOLDER);
        assert_eq!(origin.trust_level, "explicit");
        assert_eq!(origin.installed_by.as_deref(), Some("qs-vibe"));
        assert_eq!(update_status_for(Some(&origin), None), UPDATE_STATUS_BEST_EFFORT);
    }

    #[test]
    fn update_status_requires_origin_for_unknown() {
        assert_eq!(update_status_for(None, None), UPDATE_STATUS_UNKNOWN);
    }

    #[test]
    fn update_status_allows_explicit_update_command() {
        let mut origin = build_install_origin(Path::new("F:/skill-source"));
        origin.update_command = Some("git pull".to_string());
        assert_eq!(update_status_for(Some(&origin), None), UPDATE_STATUS_AUTO);
    }

    #[test]
    fn normalize_source_method_maps_aliases() {
        assert_eq!(normalize_source_method("github"), SOURCE_METHOD_GIT);
        assert_eq!(normalize_source_method("npx"), SOURCE_METHOD_NPX);
        assert_eq!(normalize_source_method("market"), SOURCE_METHOD_MARKETPLACE);
    }

    #[test]
    fn build_git_origin_uses_git_method() {
        let probe = GitProbe {
            remote_url: "https://github.com/example/skill.git".to_string(),
            commit: "abc123".to_string(),
            branch: Some("main".to_string()),
        };
        let origin = build_git_origin(Path::new("F:/skill-source"), &probe);
        assert_eq!(origin.method, SOURCE_METHOD_GIT);
        assert_eq!(origin.provider.as_deref(), Some("github"));
        assert_eq!(
            origin.url.as_deref(),
            Some("https://github.com/example/skill.git")
        );
        assert_eq!(origin.commit.as_deref(), Some("abc123"));
        assert_eq!(origin.branch.as_deref(), Some("main"));
        assert_eq!(origin.update_command.as_deref(), Some("git pull --ff-only"));
    }

    #[test]
    fn write_and_read_sidecar_origin() {
        let dir = std::env::temp_dir().join(format!(
            "vibe_origin_sidecar_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let skill_dir = dir.join("skill");
        fs::create_dir_all(&skill_dir).unwrap();

        let mut origin = build_install_origin(Path::new("F:/skill-source"));
        origin.method = SOURCE_METHOD_GIT.to_string();
        write_skill_origin_sidecar(&skill_dir, &origin).unwrap();

        let loaded = read_skill_origin(&skill_dir).unwrap();
        assert_eq!(loaded.method, SOURCE_METHOD_GIT);
        assert_eq!(loaded.source_path.as_deref(), Some("F:/skill-source"));

        let _ = fs::remove_dir_all(&dir);
    }
}
