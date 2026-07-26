use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use tracing::warn;
use uuid::Uuid;

use crate::errors::VibeError;
use crate::models::dashboard::{
    DashboardAgent, DashboardData, DashboardSkill, DashboardStats, SharedSkillInfo,
};
use crate::models::history::HistoryAction;
use crate::models::skill::{ConflictType, Skill, SkillIssue, SkillSource, SkillUpdateCheck};
use crate::parsers::skill_md::parse_skill_md_full;
use crate::utils::config::{
    build_agents_from_config, load_agents, load_config, project_skill_roots,
};
use crate::utils::datetime;
use crate::utils::fs as vibe_fs;
use crate::utils::fs::{copy_dir_all, copy_skill_dir_all};
use crate::utils::history::record_action;
use crate::utils::origin::{
    build_command_origin, build_git_origin, build_install_origin, git_pull_ff_only, git_status_clean,
    normalize_source_method, probe_git_origin, read_skill_origin, refresh_git_origin,
    run_update_command, trust_level_for, update_status_for, write_skill_origin,
    write_skill_origin_sidecar, SOURCE_METHOD_GIT, SOURCE_METHOD_LOCAL_FOLDER,
    SOURCE_METHOD_MARKETPLACE, SOURCE_METHOD_NPM, SOURCE_METHOD_NPX,
};
use crate::utils::path::vibe_skills_dir;

/// 递归扫描最大深度，超出后截断（P4 环路/深度保护）
const MAX_SCAN_DEPTH: usize = 12;

#[tauri::command]
pub fn list_skills() -> Result<Vec<Skill>, VibeError> {
    let mut map: HashMap<String, SkillEntry> = HashMap::new();

    let vibe_dir = vibe_skills_dir()?;
    let mut hash_cache = crate::utils::hash::load_hash_cache(&vibe_dir);
    let config = load_config()?;
    let agents = build_agents_from_config(&config)?;
    let agent_ids: std::collections::HashSet<String> =
        agents.iter().map(|agent| agent.id.clone()).collect();

    scan_directory(
        &vibe_dir,
        "vibe-lib",
        &mut map,
        false,
        0,
        &mut std::collections::HashSet::new(),
        &mut hash_cache,
        Some(&agent_ids),
    )?;

    for agent in &agents {
        if !agent.detected || !agent.enabled {
            continue;
        }
        let agent_dir = Path::new(&agent.skills_dir);
        scan_directory(
            agent_dir,
            &agent.id,
            &mut map,
            false,
            0,
            &mut std::collections::HashSet::new(),
            &mut hash_cache,
            None,
        )?;

        for scan_dir in &agent.additional_scan_dirs {
            let scan_path = Path::new(scan_dir);
            if !scan_path.exists() || !scan_path.is_dir() {
                continue;
            }
            let source_id = external_source_id(&agent.id, scan_path);
            scan_directory(
                scan_path,
                &source_id,
                &mut map,
                false,
                0,
                &mut std::collections::HashSet::new(),
                &mut hash_cache,
                None,
            )?;
        }
    }

    scan_project_sources(&mut map, &mut hash_cache)?;

    // 注意：不在这里扫描 plugin marketplace
    // Plugin skills 由独立的 list_plugin_skills() 处理
    // 这样可以避免 plugin sources 与 regular sources 合并

    crate::utils::hash::save_hash_cache(&vibe_dir, &hash_cache);

    let mut skills: Vec<Skill> = map
        .into_iter()
        .map(|(id, entry)| {
            let linked_agents = find_linked_agents(&id, &agents);

            // list_skills() 不再扫描 plugin marketplace，所以这里不会有 plugin sources
            let from_plugin = false;
            let plugin_source = None;

            // 检测冲突：多个 source 的 content_hash 不完全相同
            // 注意：list_skills() 不再扫描 plugin marketplace，所以不会有 plugin sources
            let unique_hashes: Vec<&str> = entry.sources
                .iter()
                .map(|s| s.content_hash.as_str())
                .filter(|h| !h.is_empty())
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect();
            let has_conflict = unique_hashes.len() > 1;

            // 检测断链：is_symlink 为 true 但 symlink_target 不存在
            let has_dangling = entry.sources.iter().any(|s| {
                if !s.is_symlink {
                    return false;
                }
                match &s.symlink_target {
                    Some(target) => !vibe_fs::normalize_path(Path::new(target)).exists(),
                    None => true,
                }
            });

            // 检测重复：同文件夹名但 SKILL.md name 不同
            let unique_names: std::collections::HashSet<&str> =
                entry.sources.iter().map(|s| s.name.as_str()).collect();
            let is_duplicate = unique_names.len() > 1;

            // 检测 name 是否为空
            let missing_name = entry.name.is_empty();

            Skill {
                id,
                name: entry.name,
                description: entry.description,
                path: entry.path,
                linked_agents,
                sources: entry.sources,
                license: entry.license,
                compatibility: entry.compatibility,
                metadata: entry.metadata,
                has_scripts: entry.has_scripts,
                has_references: entry.has_references,
                has_assets: entry.has_assets,
                modified_at: entry.modified_at,
                has_conflict,
                has_dangling,
                is_duplicate,
                missing_name,
                from_plugin,
                plugin_source,
                plugin_enabled: None, // regular skills 不需要这个字段
            }
        })
        .collect();

    // 排序：冲突和断链置顶，其余按字母排序
    skills.sort_by(|a, b| {
        let a_issue = a.has_conflict || a.has_dangling;
        let b_issue = b.has_conflict || b.has_dangling;
        b_issue
            .cmp(&a_issue)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });

    // 分离：只返回 regular skills（非 plugin）
    let regular_skills: Vec<Skill> = skills.into_iter()
        .filter(|s| !s.from_plugin)
        .collect();

    Ok(regular_skills)
}

/// 列出所有 plugin skills（独立于 regular skills）
/// Plugin skills 由 plugin 市场管理，不在中心库中
#[tauri::command]
pub fn list_plugin_skills() -> Result<Vec<Skill>, VibeError> {
    let mut map: HashMap<String, SkillEntry> = HashMap::new();

    let config = load_config()?;
    let agents = build_agents_from_config(&config)?;

    let vibe_dir = vibe_skills_dir()?;
    let mut hash_cache = crate::utils::hash::load_hash_cache(&vibe_dir);

    // 只扫描 plugin 市场
    scan_plugin_marketplace_skills(&mut map, &mut hash_cache)?;

    crate::utils::hash::save_hash_cache(&vibe_dir, &hash_cache);

    // 过滤掉已认领的 skill（已在中心库中的）
    let mut skills: Vec<Skill> = map
        .into_iter()
        .map(|(id, entry)| {
            let linked_agents = find_linked_agents(&id, &agents);

            // 提取 plugin 来源信息
            let from_plugin = true; // 这里只处理 plugin skills
            let plugin_source = entry.sources.iter()
                .find(|s| s.source_kind == "marketplace" || s.from.starts_with("claude-plugin:") || s.from.starts_with("codex-plugin:"))
                .and_then(|s| {
                    if s.from.starts_with("claude-plugin:") {
                        Some(s.from.strip_prefix("claude-plugin:").unwrap_or(&s.from).to_string())
                    } else if s.from.starts_with("codex-plugin:") {
                        Some(s.from.strip_prefix("codex-plugin:").unwrap_or(&s.from).to_string())
                    } else {
                        None
                    }
                });

            // Plugin skills 不检测冲突、断链、重复（由 plugin 系统管理）
            let missing_name = entry.name.is_empty();

            // 获取 plugin 启用状态
            let plugin_enabled = entry.sources.iter()
                .find(|s| s.source_kind == "marketplace" || s.from.starts_with("claude-plugin:") || s.from.starts_with("codex-plugin:"))
                .and_then(|s| {
                    if s.from.starts_with("claude-plugin:") {
                        let plugin_name = s.from.strip_prefix("claude-plugin:").unwrap_or("");
                        get_plugin_enabled("claude-plugin", plugin_name)
                    } else if s.from.starts_with("codex-plugin:") {
                        let plugin_name = s.from.strip_prefix("codex-plugin:").unwrap_or("");
                        get_plugin_enabled("codex-plugin", plugin_name)
                    } else {
                        None
                    }
                });

            Skill {
                id,
                name: entry.name,
                description: entry.description,
                path: entry.path,
                linked_agents,
                sources: entry.sources,
                license: entry.license,
                compatibility: entry.compatibility,
                metadata: entry.metadata,
                has_scripts: entry.has_scripts,
                has_references: entry.has_references,
                has_assets: entry.has_assets,
                modified_at: entry.modified_at,
                has_conflict: false,
                has_dangling: false,
                is_duplicate: false,
                missing_name,
                from_plugin,
                plugin_source,
                plugin_enabled,
            }
        })
        .collect();

    // 标记已认领的 skill（已在中心库中的）
    for skill in &mut skills {
        let skill_path = vibe_dir.join(&skill.id);
        // 通过 metadata 标记是否已认领
        if skill.metadata.is_none() {
            skill.metadata = Some(std::collections::HashMap::new());
        }
        if let Some(meta) = &mut skill.metadata {
            meta.insert("adopted".to_string(), skill_path.exists().to_string());
        }
    }

    // 按 plugin_source 分组，然后按名称排序
    skills.sort_by(|a, b| {
        a.plugin_source.as_deref().unwrap_or("")
            .cmp(b.plugin_source.as_deref().unwrap_or(""))
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });

    Ok(skills)
}

/// 认领 plugin skill 到中心库
/// 将 plugin skill 复制到 ~/.vibe-skills/，使其成为 library-managed skill
/// 如果已存在，先快照到 .trash/ 再覆盖
#[tauri::command]
pub fn adopt_plugin_skill(skill_id: String) -> Result<Skill, VibeError> {
    let vibe_dir = vibe_skills_dir()?;
    let skill_path = vibe_dir.join(&skill_id);

    // 查找 plugin skill
    let plugin_skills = list_plugin_skills()?;
    let plugin_skill = plugin_skills.iter().find(|s| s.id == skill_id).ok_or_else(|| {
        VibeError::SkillNotFound {
            skill_id: skill_id.clone(),
        }
    })?;

    // 查找 plugin 来源路径
    let plugin_source = plugin_skill.sources.iter().find(|s| {
        s.source_kind == "marketplace"
            || s.from.starts_with("claude-plugin:")
            || s.from.starts_with("codex-plugin:")
    }).ok_or_else(|| {
        VibeError::Path(format!("No plugin source found for skill {}", skill_id))
    })?;

    let source_path = Path::new(&plugin_source.path);
    if !source_path.exists() {
        return Err(VibeError::Path(format!(
            "Plugin source path does not exist: {}",
            source_path.display()
        )));
    }

    // 如果已存在，先快照到 .trash/ 再删除
    if skill_path.exists() {
        let trash_dir = vibe_dir.join(".trash");
        let _ = std::fs::create_dir_all(&trash_dir);
        let timestamp = crate::utils::datetime::chrono_now().replace("-", "").replace(":", "").replace(" ", "_");
        let trash_path = trash_dir.join(format!("{}_{}", skill_id, timestamp));
        let _ = copy_dir_all(&skill_path, &trash_path);
        let _ = std::fs::remove_dir_all(&skill_path);
    }

    // 复制到中心库
    copy_skill_dir_all(source_path, &skill_path)?;

    // 写入 origin 记录，标记来源为 marketplace
    let mut origin = crate::utils::origin::build_install_origin(
        Path::new(&plugin_source.path),
    );
    origin.method = SOURCE_METHOD_MARKETPLACE.to_string();
    origin.installed_by = Some("qs-vibe-adopt".to_string());
    // 清除 command 和 update_command，因为 marketplace 来源不支持命令更新
    origin.command = None;
    origin.update_command = None;
    let _ = write_skill_origin(&skill_path, &origin);

    // 返回新创建的 library skill
    let skills = list_skills()?;
    skills.into_iter().find(|s| s.id == skill_id).ok_or_else(|| {
        VibeError::Path(format!(
            "Failed to find adopted skill '{}' in library",
            skill_id
        ))
    })
}

#[tauri::command]
pub async fn check_skill_update(skill_id: String) -> Result<SkillUpdateCheck, VibeError> {
    tauri::async_runtime::spawn_blocking(move || check_skill_update_sync(skill_id))
        .await
        .map_err(|error| VibeError::Path(format!("Update check task failed: {}", error)))?
}

fn check_skill_update_sync(skill_id: String) -> Result<SkillUpdateCheck, VibeError> {
    let skill = list_skills()?
        .into_iter()
        .find(|skill| skill.id == skill_id)
        .ok_or_else(|| VibeError::SkillNotFound {
            skill_id: skill_id.clone(),
        })?;
    check_skill_update_for_skill(&skill)
}

fn check_skill_update_for_skill(skill: &Skill) -> Result<SkillUpdateCheck, VibeError> {
    let skill_id = skill.id.clone();
    let checked_at = datetime::chrono_now();
    let Some(source) = skill.sources.iter().find(|source| source.from == "vibe-lib") else {
        return Ok(SkillUpdateCheck {
            skill_id,
            method: "unknown".to_string(),
            available: false,
            current_commit: None,
            remote_commit: None,
            checked_at,
            error: Some("Skill is not installed in the library; agent-only copies cannot be checked here".to_string()),
        });
    };
    let Some(origin) = source.origin.as_ref() else {
        return Ok(SkillUpdateCheck {
            skill_id,
            method: "unknown".to_string(),
            available: false,
            current_commit: None,
            remote_commit: None,
            checked_at,
            error: Some("No provenance record is available".to_string()),
        });
    };

    let method = normalize_source_method(&origin.method);
    match method.as_str() {
        SOURCE_METHOD_GIT => check_git_source_update(&skill_id, origin, checked_at),
        SOURCE_METHOD_NPX | SOURCE_METHOD_NPM => {
            check_npm_package_update(&skill_id, origin, checked_at)
        }
        SOURCE_METHOD_MARKETPLACE => {
            check_plugin_update(&skill_id, origin, checked_at)
        }
        _ => Ok(SkillUpdateCheck {
            skill_id,
            method: origin.method.clone(),
            available: false,
            current_commit: None,
            remote_commit: None,
            checked_at,
            error: Some(format!("来源 {} 暂不支持远程更新检测", origin.method)),
        }),
    }
}

fn check_git_source_update(
    skill_id: &str,
    origin: &crate::models::origin::SkillOrigin,
    checked_at: String,
) -> Result<SkillUpdateCheck, VibeError> {
    let Some(source_path) = origin.source_path.as_deref() else {
        return Ok(SkillUpdateCheck {
            skill_id: skill_id.to_string(),
            method: SOURCE_METHOD_GIT.to_string(),
            available: false,
            current_commit: origin.commit.clone(),
            remote_commit: None,
            checked_at,
            error: Some("Git source path is missing".to_string()),
        });
    };
    let path = Path::new(source_path);
    let current_commit = git_output(path, &["rev-parse", "HEAD"]);
    let fetch_output = match run_git_fetch(path) {
        Ok(output) => output,
        Err(message) => {
            warn!(skill = %skill_id, error = %message, "Skill update check failed");
            return Ok(SkillUpdateCheck { skill_id: skill_id.to_string(), method: SOURCE_METHOD_GIT.to_string(), available: false, current_commit, remote_commit: None, checked_at, error: Some(message) });
        }
    };
    if !fetch_output.status.success() {
        let message = String::from_utf8_lossy(&fetch_output.stderr).trim().to_string();
        let message = if message.is_empty() { "Git fetch failed; check remote access and permissions".to_string() } else { message };
        warn!(skill = %skill_id, error = %message, "Skill update check failed");
        return Ok(SkillUpdateCheck { skill_id: skill_id.to_string(), method: SOURCE_METHOD_GIT.to_string(), available: false, current_commit, remote_commit: None, checked_at, error: Some(message) });
    }
    let remote_ref = origin.branch.as_deref().map(|branch| format!("origin/{}", branch)).unwrap_or_else(|| "origin/HEAD".to_string());
    let remote_commit = git_output(path, &["rev-parse", &remote_ref]);
    let error = if remote_commit.is_none() { Some(format!("Remote branch {} is not available", remote_ref)) } else { None };
    Ok(SkillUpdateCheck {
        skill_id: skill_id.to_string(),
        method: SOURCE_METHOD_GIT.to_string(),
        available: current_commit.is_some() && remote_commit.is_some() && current_commit != remote_commit,
        current_commit,
        remote_commit,
        checked_at,
        error,
    })
}

#[tauri::command]
pub async fn check_all_skill_updates() -> Result<Vec<SkillUpdateCheck>, VibeError> {
    tauri::async_runtime::spawn_blocking(|| {
        let skills = list_skills()?;
        tracing::info!(skill_count = skills.len(), "Checking all skill updates");
        let results: Result<Vec<_>, _> = skills
            .iter()
            .map(check_skill_update_for_skill)
            .collect();
        if let Ok(items) = &results {
            let available = items.iter().filter(|item| item.available).count();
            tracing::info!(checked = items.len(), available, "Finished checking skill updates");
        }
        results
    })
    .await
    .map_err(|error| VibeError::Path(format!("Update check task failed: {}", error)))?
}

fn run_git_fetch(path: &Path) -> Result<std::process::Output, String> {
    const FETCH_TIMEOUT: Duration = Duration::from_secs(30);
    let mut child = Command::new("git")
        .arg("-C")
        .arg(path)
        .args(["-c", "credential.interactive=never", "fetch", "--quiet", "--no-tags", "origin"])
        .env("GIT_TERMINAL_PROMPT", "0")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("Git fetch could not start: {}", error))?;
    let started_at = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => return child.wait_with_output().map_err(|error| error.to_string()),
            Ok(None) if started_at.elapsed() < FETCH_TIMEOUT => {
                std::thread::sleep(Duration::from_millis(100));
            }
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!("Git fetch timed out after {} seconds", FETCH_TIMEOUT.as_secs()));
            }
            Err(error) => return Err(format!("Git fetch wait failed: {}", error)),
        }
    }
}

fn git_output(path: &Path, args: &[&str]) -> Option<String> {
    let output = Command::new("git").arg("-C").arg(path).args(args).output().ok()?;
    if !output.status.success() { return None; }
    let value = String::from_utf8(output.stdout).ok()?.trim().to_string();
    (!value.is_empty()).then_some(value)
}

/// 从 npx/npm 命令中提取包名
/// 例如: "npx @anthropic-ai/claude-code-skills@latest install foo" → Some("@anthropic-ai/claude-code-skills")
///        "npm exec skills -- install foo" → Some("skills")
fn extract_package_name_from_command(command: &str) -> Option<String> {
    let parts: Vec<&str> = command.split_whitespace().collect();
    if parts.len() < 2 {
        return None;
    }

    // 跳过 npx/npm 部分
    let cmd_part = if parts[0] == "npx" || parts[0] == "npm" {
        if parts[0] == "npm" && parts.len() > 1 && (parts[1] == "exec" || parts[1] == "run") {
            if parts.len() < 3 { return None; }
            parts[2]
        } else {
            parts[1]
        }
    } else {
        return None;
    };

    // 去除 @scope/ 和 @version 后缀
    let name = if cmd_part.starts_with('@') {
        // scoped package: @scope/name@version → @scope/name
        let at_idx = cmd_part[1..].find('@').map(|i| i + 1);
        match at_idx {
            Some(idx) => &cmd_part[..idx],
            None => cmd_part,
        }
    } else {
        // unscoped: name@version → name
        cmd_part.split('@').next().unwrap_or(cmd_part)
    };

    Some(name.to_string())
}

/// 检查 npm 包是否有更新
/// 执行 `npm view <package> version` 获取远程最新版本
/// 与 origin.version 比较
fn check_npm_package_update(
    skill_id: &str,
    origin: &crate::models::origin::SkillOrigin,
    checked_at: String,
) -> Result<SkillUpdateCheck, VibeError> {
    let method = origin.method.clone();

    // 获取包名：优先使用 origin.package_name，否则从命令中提取
    let package_name = origin.package_name.clone()
        .or_else(|| origin.command.as_ref().and_then(|cmd| extract_package_name_from_command(cmd)));

    let Some(package_name) = package_name else {
        return Ok(SkillUpdateCheck {
            skill_id: skill_id.to_string(),
            method,
            available: false,
            current_commit: None,
            remote_commit: None,
            checked_at,
            error: Some("无法从命令中提取包名".to_string()),
        });
    };

    // 执行 npm view <package> version
    let output = Command::new("npm")
        .args(["view", &package_name, "version"])
        .env("CI", "1")
        .env("NPM_CONFIG_YES", "true")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();

    match output {
        Ok(output) if output.status.success() => {
            let latest_version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let current_version = origin.version.clone().unwrap_or_default();

            let available = !latest_version.is_empty()
                && !current_version.is_empty()
                && latest_version != current_version;

            Ok(SkillUpdateCheck {
                skill_id: skill_id.to_string(),
                method,
                available,
                current_commit: if current_version.is_empty() { None } else { Some(current_version) },
                remote_commit: if latest_version.is_empty() { None } else { Some(latest_version) },
                checked_at,
                error: None,
            })
        }
        Ok(output) => {
            let error = String::from_utf8_lossy(&output.stderr).trim().to_string();
            Ok(SkillUpdateCheck {
                skill_id: skill_id.to_string(),
                method,
                available: false,
                current_commit: origin.version.clone(),
                remote_commit: None,
                checked_at,
                error: Some(if error.is_empty() { "npm view failed".to_string() } else { error }),
            })
        }
        Err(e) => Ok(SkillUpdateCheck {
            skill_id: skill_id.to_string(),
            method,
            available: false,
            current_commit: origin.version.clone(),
            remote_commit: None,
            checked_at,
            error: Some(format!("npm 命令执行失败: {}", e)),
        }),
    }
}

/// 检查 Plugin skill 是否有更新
/// 比较 plugin cache 目录的内容哈希与中心库中的内容哈希
fn check_plugin_update(
    skill_id: &str,
    origin: &crate::models::origin::SkillOrigin,
    checked_at: String,
) -> Result<SkillUpdateCheck, VibeError> {
    let method = origin.method.clone();

    // Plugin 来源需要有 source_path（plugin cache 中的路径）
    let Some(source_path) = origin.source_path.as_deref() else {
        return Ok(SkillUpdateCheck {
            skill_id: skill_id.to_string(),
            method,
            available: false,
            current_commit: None,
            remote_commit: None,
            checked_at,
            error: Some("Plugin 来源缺少 source_path".to_string()),
        });
    };

    let plugin_dir = Path::new(source_path);
    if !plugin_dir.exists() {
        return Ok(SkillUpdateCheck {
            skill_id: skill_id.to_string(),
            method,
            available: false,
            current_commit: None,
            remote_commit: None,
            checked_at,
            error: Some("Plugin 来源目录不存在".to_string()),
        });
    }

    // 计算 plugin cache 目录的内容哈希
    let plugin_hash = crate::utils::hash::dir_hash(plugin_dir);

    // 获取中心库中的内容哈希
    let vibe_dir = vibe_skills_dir()?;
    let skill_path = vibe_dir.join(skill_id);
    let library_hash = if skill_path.exists() {
        crate::utils::hash::dir_hash(&skill_path)
    } else {
        String::new()
    };

    let available = !plugin_hash.is_empty() && !library_hash.is_empty() && plugin_hash != library_hash;

    Ok(SkillUpdateCheck {
        skill_id: skill_id.to_string(),
        method,
        available,
        current_commit: if library_hash.is_empty() { None } else { Some(library_hash) },
        remote_commit: if plugin_hash.is_empty() { None } else { Some(plugin_hash) },
        checked_at,
        error: None,
    })
}

/// 更新 Plugin skill
/// 重新扫描 plugin cache，将最新文件复制到中心库
/// 如果同一 marketplace 有多个 plugin，一起更新
fn update_plugin_skill(
    skill_path: &Path,
    skill_id: &str,
    origin: &mut crate::models::origin::SkillOrigin,
) -> Result<(), VibeError> {
    let Some(source_path) = origin.source_path.clone() else {
        return Err(VibeError::Path(format!(
            "Skill {} 缺少 Plugin 来源路径，无法更新",
            skill_id
        )));
    };
    let plugin_source_path = Path::new(&source_path);
    if !plugin_source_path.exists() {
        return Err(VibeError::Path(format!(
            "Plugin 来源路径不存在：{}",
            plugin_source_path.display()
        )));
    }

    // 清空中心库内容并复制最新文件
    if vibe_fs::is_link(skill_path) {
        vibe_fs::remove_symlink(skill_path)?;
    } else if skill_path.exists() {
        vibe_fs::clear_skill_dir_contents(skill_path)?;
    }

    copy_skill_dir_all(plugin_source_path, skill_path)?;

    // 更新 origin 的 last_checked_at
    origin.last_checked_at = Some(datetime::chrono_now());
    write_skill_origin(skill_path, origin)?;

    Ok(())
}

/// 批量更新同一 marketplace 的所有 plugin skills
/// 当一个 plugin 更新时，同一来源的其他 plugin 也会被更新
#[tauri::command]
pub fn update_plugin_skills_from_marketplace(marketplace: String) -> Result<Vec<String>, VibeError> {
    let vibe_dir = vibe_skills_dir()?;
    let all_skills = list_skills()?;
    let mut updated_ids = Vec::new();

    // 找到所有来自该 marketplace 的 plugin skills
    let marketplace_skills: Vec<&Skill> = all_skills.iter().filter(|s| {
        s.sources.iter().any(|source| {
            source.origin.as_ref().map_or(false, |o| {
                normalize_source_method(&o.method) == SOURCE_METHOD_MARKETPLACE
                    && o.source_path.as_ref().map_or(false, |p| {
                        // 检查 source_path 是否包含 marketplace 名称
                        p.contains(&marketplace)
                    })
            })
        })
    }).collect();

    for skill in marketplace_skills {
        let skill_path = vibe_dir.join(&skill.id);
        let Some(source) = skill.sources.iter().find(|s| s.from == "vibe-lib") else {
            continue;
        };
        let Some(origin) = source.origin.as_ref() else {
            continue;
        };

        let mut origin_clone = origin.clone();
        match update_plugin_skill(&skill_path, &skill.id, &mut origin_clone) {
            Ok(()) => {
                updated_ids.push(skill.id.clone());
            }
            Err(e) => {
                warn!(skill = %skill.id, error = %e, "Plugin skill update failed");
            }
        }
    }

    Ok(updated_ids)
}

#[tauri::command]
pub fn search_skills(query: String) -> Result<Vec<Skill>, VibeError> {
    let all_skills = list_skills()?;
    if query.trim().is_empty() {
        return Ok(all_skills);
    }

    let q = query.to_lowercase();
    let results: Vec<Skill> = all_skills
        .into_iter()
        .filter(|s| {
            s.name.to_lowercase().contains(&q)
                || s.description.to_lowercase().contains(&q)
                || s.id.to_lowercase().contains(&q)
        })
        .collect();

    Ok(results)
}

#[tauri::command]
pub fn detect_issues() -> Result<Vec<SkillIssue>, VibeError> {
    let skills = list_skills()?;
    let mut issues = Vec::new();

    for skill in skills {
        if skill.has_conflict {
            let source_names: Vec<String> = skill
                .sources
                .iter()
                .map(|s| {
                    let agent_name = if s.from == "vibe-lib" {
                        "Vibe Library"
                    } else {
                        &s.from
                    };
                    format!("{} ({})", s.name, agent_name)
                })
                .collect();
            issues.push(SkillIssue {
                skill_id: skill.id.clone(),
                issue_type: ConflictType::SameNameDiffContent,
                description: format!("同名 skill 有不同内容: {}", source_names.join(", ")),
            });
        }

        if skill.has_dangling {
            let broken_sources: Vec<String> = skill
                .sources
                .iter()
                .filter(|s| s.is_symlink)
                .filter_map(|s| s.symlink_target.as_ref())
                .cloned()
                .collect();
            issues.push(SkillIssue {
                skill_id: skill.id.clone(),
                issue_type: ConflictType::DanglingLink,
                description: format!("断链指向已删除路径: {}", broken_sources.join(", ")),
            });
        }
    }

    Ok(issues)
}

#[tauri::command]
pub fn get_dashboard_data() -> Result<DashboardData, VibeError> {
    let config = load_config()?;
    let agents = build_agents_from_config(&config)?;
    let vibe_dir = vibe_skills_dir()?;
    let mut truncated = false;

    let mut agent_skills: HashMap<String, Vec<(String, String)>> = HashMap::new();
    let mut all_skill_agents: HashMap<String, Vec<String>> = HashMap::new();

    for agent in &agents {
        if !agent.detected {
            continue;
        }
        let skills_dir = Path::new(&agent.skills_dir);
        if !skills_dir.exists() {
            continue;
        }

        let mut skills = Vec::new();
        collect_skills_recursive(
            skills_dir,
            &mut skills,
            &mut all_skill_agents,
            &agent.id,
            &vibe_dir,
            0,
            &mut std::collections::HashSet::new(),
            &mut truncated,
        );

        agent_skills.insert(agent.id.clone(), skills);
    }

    let shared_skills: Vec<SharedSkillInfo> = all_skill_agents
        .iter()
        .filter(|(_, agent_ids)| agent_ids.len() > 1)
        .map(|(skill_id, agent_ids)| {
            let skill_name = agent_skills
                .values()
                .flatten()
                .find(|(id, _)| id == skill_id)
                .map(|(_, name)| name.clone())
                .unwrap_or_else(|| skill_id.clone());

            SharedSkillInfo {
                skill_id: skill_id.clone(),
                skill_name,
                agent_ids: agent_ids.clone(),
            }
        })
        .collect();

    let mut total_skills: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut per_agent_count: HashMap<String, usize> = HashMap::new();

    let dashboard_agents: Vec<DashboardAgent> = agents
        .iter()
        .filter(|a| a.detected)
        .map(|agent| {
            let skills = agent_skills.get(&agent.id).cloned().unwrap_or_default();
            let skill_count = skills.len();
            per_agent_count.insert(agent.id.clone(), skill_count);

            let mut dashboard_skills: Vec<DashboardSkill> = skills
                .iter()
                .map(|(skill_id, skill_name)| {
                    total_skills.insert(skill_id.clone());
                    let shared_with: Vec<String> = all_skill_agents
                        .get(skill_id)
                        .map(|ids| ids.iter().filter(|id| *id != &agent.id).cloned().collect())
                        .unwrap_or_default();

                    DashboardSkill {
                        skill_id: skill_id.clone(),
                        skill_name: skill_name.clone(),
                        shared_with,
                    }
                })
                .collect();

            dashboard_skills.sort_by(|a, b| {
                b.shared_with.len().cmp(&a.shared_with.len()).then(
                    a.skill_name
                        .to_lowercase()
                        .cmp(&b.skill_name.to_lowercase()),
                )
            });

            DashboardAgent {
                agent_id: agent.id.clone(),
                agent_name: agent.name.clone(),
                skill_count,
                skills: dashboard_skills,
            }
        })
        .collect();

    let mut vibe_skills = Vec::new();
    let mut vibe_truncated = false;
    if vibe_dir.exists() {
        collect_vibe_skills(
            &vibe_dir,
            &mut vibe_skills,
            &all_skill_agents,
            &mut total_skills,
            0,
            &mut std::collections::HashSet::new(),
            &mut vibe_truncated,
        );
    }

    let mut all_agents = dashboard_agents;
    if !vibe_skills.is_empty() {
        all_agents.insert(
            0,
            DashboardAgent {
                agent_id: "vibe-lib".to_string(),
                agent_name: "VIBE Library".to_string(),
                skill_count: vibe_skills.len(),
                skills: vibe_skills,
            },
        );
    }

    let stats = DashboardStats {
        total_skills: total_skills.len(),
        shared_count: shared_skills.len(),
        per_agent_count,
    };

    Ok(DashboardData {
        agents: all_agents,
        shared_skills,
        stats,
        truncated: truncated || vibe_truncated,
    })
}

fn collect_skills_recursive(
    dir: &Path,
    skills: &mut Vec<(String, String)>,
    all_skill_agents: &mut HashMap<String, Vec<String>>,
    agent_id: &str,
    vibe_dir: &Path,
    depth: usize,
    visited: &mut std::collections::HashSet<std::path::PathBuf>,
    truncated: &mut bool,
) {
    if depth > MAX_SCAN_DEPTH || !visited.insert(vibe_fs::normalize_path(dir)) {
        *truncated = true;
        return;
    }
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            if name.starts_with('.') {
                continue;
            }

            // 跳过指向 vibe-lib 的 symlink，避免重复计数
            if vibe_fs::is_link(&path) {
                if let Ok(target) = vibe_fs::read_link_target(&path) {
                    if target.starts_with(vibe_dir) {
                        continue;
                    }
                }
            }

            let skill_md_path = path.join("SKILL.md");
            if skill_md_path.exists() {
                let id = name.clone();
                let skill_name = parse_skill_md_full(&skill_md_path)
                    .map(|(n, _, _, _, _, _)| n)
                    .unwrap_or_else(|_| id.clone());

                all_skill_agents
                    .entry(id.clone())
                    .or_default()
                    .push(agent_id.to_string());
                skills.push((id, skill_name));
            } else {
                collect_skills_recursive(
                    &path,
                    skills,
                    all_skill_agents,
                    agent_id,
                    vibe_dir,
                    depth + 1,
                    visited,
                    truncated,
                );
            }
        }
    }
}

fn collect_vibe_skills(
    dir: &Path,
    vibe_skills: &mut Vec<DashboardSkill>,
    all_skill_agents: &HashMap<String, Vec<String>>,
    total_skills: &mut std::collections::HashSet<String>,
    depth: usize,
    visited: &mut std::collections::HashSet<std::path::PathBuf>,
    truncated: &mut bool,
) {
    if depth > MAX_SCAN_DEPTH || !visited.insert(vibe_fs::normalize_path(dir)) {
        *truncated = true;
        return;
    }
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let id = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            if id.starts_with('.') || id == ".vibe-config.json" || id == ".vibe-history.json" {
                continue;
            }

            let skill_md_path = path.join("SKILL.md");
            if skill_md_path.exists() {
                let name = parse_skill_md_full(&skill_md_path)
                    .map(|(n, _, _, _, _, _)| n)
                    .unwrap_or_else(|_| id.clone());

                total_skills.insert(id.clone());

                let shared_with: Vec<String> = all_skill_agents
                    .get(&id)
                    .map(|ids| ids.clone())
                    .unwrap_or_default();

                vibe_skills.push(DashboardSkill {
                    skill_id: id,
                    skill_name: name,
                    shared_with,
                });
            } else {
                collect_vibe_skills(
                    &path,
                    vibe_skills,
                    all_skill_agents,
                    total_skills,
                    depth + 1,
                    visited,
                    truncated,
                );
            }
        }
    }
}

#[tauri::command]
pub fn preview_skill(skill_id: String) -> Result<String, VibeError> {
    let vibe_dir = vibe_skills_dir()?;
    let vibe_path = vibe_dir.join(&skill_id).join("SKILL.md");
    if vibe_path.exists() {
        return fs::read_to_string(&vibe_path).map_err(VibeError::Io);
    }

    let agents = load_agents()?;
    for agent in &agents {
        if !agent.detected {
            continue;
        }
        let agent_path = Path::new(&agent.skills_dir)
            .join(&skill_id)
            .join("SKILL.md");
        if agent_path.exists() {
            return fs::read_to_string(&agent_path).map_err(VibeError::Io);
        }
        if let Ok(content) = find_skill_md_recursive(
            &Path::new(&agent.skills_dir),
            &skill_id,
            0,
            &mut std::collections::HashSet::new(),
        ) {
            return Ok(content);
        }
        for scan_dir in &agent.additional_scan_dirs {
            if let Ok(content) = find_skill_md_recursive(
                Path::new(scan_dir),
                &skill_id,
                0,
                &mut std::collections::HashSet::new(),
            ) {
                return Ok(content);
            }
        }
    }

    Err(VibeError::SkillNotFound { skill_id })
}

/// 按路径预览 SKILL.md 内容（P6：沙箱到 vibe 目录与已配置 agent 目录）
#[tauri::command]
pub fn preview_skill_at_path(path: String) -> Result<String, VibeError> {
    let skill_path = Path::new(&path);
    if !skill_path.exists() {
        return Err(VibeError::SkillNotFound { skill_id: path });
    }

    // 仅允许读取 vibe 库或某个 agent skills 目录内的文件（调用方传入的是已扫描的 source.path）
    let vibe_dir = vibe_skills_dir()?;
    let agents = load_agents()?;
    let config = load_config()?;
    let target = vibe_fs::normalize_path(skill_path);
    let allowed = vibe_fs::is_path_within(&target, &vibe_dir)
        || agents
            .iter()
            .any(|a| vibe_fs::is_path_within(&target, Path::new(&a.skills_dir)))
        || agents.iter().any(|a| {
            a.additional_scan_dirs
                .iter()
                .any(|dir| vibe_fs::is_path_within(&target, Path::new(dir)))
        })
        || project_skill_roots(&config)
            .iter()
            .any(|root| vibe_fs::is_path_within(&target, root));
    if !allowed {
        return Err(VibeError::Path(
            "preview_skill_at_path 仅允许读取 vibe 目录或 agent 目录内的文件".to_string(),
        ));
    }

    let skill_md_path = if skill_path.join("SKILL.md").exists() {
        skill_path.join("SKILL.md")
    } else {
        skill_path.to_path_buf()
    };

    fs::read_to_string(&skill_md_path).map_err(VibeError::Io)
}

fn find_skill_md_recursive(
    dir: &Path,
    skill_id: &str,
    depth: usize,
    visited: &mut std::collections::HashSet<std::path::PathBuf>,
) -> Result<String, VibeError> {
    if depth > MAX_SCAN_DEPTH || !visited.insert(vibe_fs::normalize_path(dir)) {
        return Err(VibeError::SkillNotFound {
            skill_id: skill_id.to_string(),
        });
    }
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        if name == skill_id {
            let skill_md = path.join("SKILL.md");
            if skill_md.exists() {
                return fs::read_to_string(&skill_md).map_err(VibeError::Io);
            }
        }
        if name.starts_with('.') {
            continue;
        }
        if let Ok(content) = find_skill_md_recursive(&path, skill_id, depth + 1, visited) {
            return Ok(content);
        }
    }
    Err(VibeError::SkillNotFound {
        skill_id: skill_id.to_string(),
    })
}

#[tauri::command]
pub fn install_skill(source_path: String, reference: bool) -> Result<Skill, VibeError> {
    install_skill_from_path(Path::new(&source_path), reference)
}

#[tauri::command]
pub fn install_skill_from_source(
    source_mode: String,
    source_value: String,
    reference: bool,
) -> Result<Skill, VibeError> {
    let mode = source_mode.trim().to_ascii_lowercase();
    match mode.as_str() {
        "folder" | "local-folder" | "local_folder" => {
            install_skill_from_path(Path::new(&source_value), reference)
        }
        "git" | "git-url" | "git_url" => install_skill_from_git_url(&source_value, reference),
        "command" => install_skill_from_command(&source_value, reference),
        _ => Err(VibeError::Path(format!(
            "Unsupported install source mode: {}",
            source_mode
        ))),
    }
}

fn install_skill_from_path(source: &Path, reference: bool) -> Result<Skill, VibeError> {
    if !source.exists() {
        return Err(VibeError::InvalidSkillMd {
            reason: format!("Source path does not exist: {}", source.display()),
        });
    }

    let install_root = locate_skill_root(source)?;
    let origin = probe_git_origin(source)
        .map(|probe| build_git_origin(source, &probe))
        .unwrap_or_else(|| build_install_origin(source));
    install_skill_from_materialized_source(&install_root, reference, origin)
}

fn install_skill_from_git_url(git_url: &str, reference: bool) -> Result<Skill, VibeError> {
    let source_root = managed_install_source_dir("git")?;
    let result = (|| {
        if let Some(parent) = source_root.parent() {
            fs::create_dir_all(parent)?;
        }
        clone_git_repo(git_url, &source_root)?;

        let install_root = locate_skill_root(&source_root)?;
        let probe = probe_git_origin(&source_root).ok_or_else(|| {
            VibeError::Path(format!(
                "Unable to read Git provenance from {}",
                source_root.display()
            ))
        })?;
        let origin = build_git_origin(&source_root, &probe);
        install_skill_from_materialized_source(&install_root, reference, origin)
    })();

    if result.is_err() {
        let _ = remove_path(&source_root);
    }

    result
}

fn install_skill_from_command(command: &str, reference: bool) -> Result<Skill, VibeError> {
    let source_root = managed_install_source_dir("command")?;
    let result = (|| {
        if let Some(parent) = source_root.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::create_dir_all(&source_root)?;
        run_update_command(command, Some(&source_root))?;

        let install_root = locate_skill_root(&source_root)?;
        let origin = build_command_origin(&source_root, command);
        install_skill_from_materialized_source(&install_root, reference, origin)
    })();

    if result.is_err() {
        let _ = remove_path(&source_root);
    }

    result
}

fn install_skill_from_materialized_source(
    install_root: &Path,
    reference: bool,
    origin: crate::models::origin::SkillOrigin,
) -> Result<Skill, VibeError> {
    let skill_md = install_root.join("SKILL.md");
    if !skill_md.exists() {
        return Err(VibeError::InvalidSkillMd {
            reason: format!(
                "Source directory does not contain SKILL.md: {}",
                install_root.display()
            ),
        });
    }

    let (name, description, license, compatibility, metadata, _body) =
        parse_skill_md_full(&skill_md)?;

    let vibe_dir = vibe_skills_dir()?;
    let dest = vibe_dir.join(&name);

    if dest.exists() {
        return Err(VibeError::SkillAlreadyExists { skill_id: name });
    }

    if reference {
        let report = vibe_fs::create_symlink_with_report(install_root, &dest)?;
        if let Some(warning) = report.warning {
            warn!("Reference install fallback: {}", warning);
        }
    } else {
        copy_skill_dir_all(install_root, &dest)?;
    }

    if reference {
        write_skill_origin_sidecar(&dest, &origin)?;
    } else {
        write_skill_origin(&dest, &origin)?;
    }

    let trust_level = trust_level_for(Some(&origin));
    let update_status = update_status_for(Some(&origin), Some(&dest));

    if let Err(e) = record_action(HistoryAction::Install, &name, None, None) {
        warn!("Failed to record Install action: {}", e);
    }

    let modified_at = get_modified_at(&dest);
    let hash = crate::utils::hash::dir_hash_into(
        &mut crate::utils::hash::load_hash_cache(&vibe_dir),
        &dest,
    );

    Ok(Skill {
        id: name.clone(),
        name: name.clone(),
        description,
        path: dest.to_string_lossy().to_string(),
        linked_agents: Vec::new(),
        sources: vec![SkillSource {
            from: "vibe-lib".to_string(),
            source_kind: "library".to_string(),
            path: dest.to_string_lossy().to_string(),
            name,
            description: String::new(),
            is_symlink: false,
            symlink_target: None,
            content_hash: hash,
            modified_at: modified_at.clone(),
            trust_level,
            update_status,
            origin: Some(origin),
        }],
        license,
        compatibility,
        metadata,
        has_scripts: dest.join("scripts").is_dir(),
        has_references: dest.join("references").is_dir(),
        has_assets: dest.join("assets").is_dir(),
        modified_at,
        has_conflict: false,
        has_dangling: false,
        is_duplicate: false,
        missing_name: false,
        from_plugin: false,
        plugin_source: None,
        plugin_enabled: None,
    })
}

fn locate_skill_root(start: &Path) -> Result<std::path::PathBuf, VibeError> {
    if start.join("SKILL.md").exists() {
        return Ok(start.to_path_buf());
    }

    let mut stack = vec![(start.to_path_buf(), 0usize)];
    while let Some((dir, depth)) = stack.pop() {
        if depth > 4 {
            continue;
        }
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            if path.join("SKILL.md").exists() {
                return Ok(path);
            }
            stack.push((path, depth + 1));
        }
    }

    Err(VibeError::InvalidSkillMd {
        reason: format!("Unable to locate SKILL.md under {}", start.display()),
    })
}

fn managed_install_source_dir(kind: &str) -> Result<std::path::PathBuf, VibeError> {
    let vibe_dir = vibe_skills_dir()?;
    Ok(vibe_dir
        .join(".sources")
        .join(kind)
        .join(Uuid::new_v4().to_string()))
}

fn clone_git_repo(url: &str, dest: &Path) -> Result<(), VibeError> {
    let output = Command::new("git")
        .args(["clone", "--depth", "1", url])
        .arg(dest)
        .output()
        .map_err(VibeError::Io)?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    Err(VibeError::Path(format!(
        "Git clone failed for {}: {}",
        url,
        if stderr.is_empty() {
            "check the URL, authentication, or network access".to_string()
        } else {
            stderr
        }
    )))
}

fn remove_path(path: &Path) -> Result<(), VibeError> {
    if !path.exists() && !vibe_fs::is_link(path) {
        return Ok(());
    }

    if vibe_fs::is_link(path) {
        vibe_fs::remove_symlink(path)?;
    } else if path.is_dir() {
        fs::remove_dir_all(path)?;
    } else {
        fs::remove_file(path)?;
    }
    Ok(())
}

#[tauri::command]
pub fn update_skill(skill_id: String, force: bool) -> Result<Skill, VibeError> {
    let vibe_dir = vibe_skills_dir()?;
    let skill_path = vibe_dir.join(&skill_id);
    if !skill_path.exists() && !vibe_fs::is_link(&skill_path) {
        return Err(VibeError::SkillNotFound { skill_id });
    }

    let mut origin = read_skill_origin(&skill_path).ok_or_else(|| {
        VibeError::Path(format!(
            "Skill {} 缺少来源记录，无法自动更新",
            skill_path.display()
        ))
    })?;

    let method = normalize_source_method(&origin.method);
    match method.as_str() {
        SOURCE_METHOD_GIT => update_from_git_source(&skill_path, &skill_id, &mut origin, force)?,
        SOURCE_METHOD_LOCAL_FOLDER => {
            let Some(source_path) = origin.source_path.clone() else {
                return Err(VibeError::Path(format!(
                    "Skill {} 缺少 source_path，无法自动更新",
                    skill_id
                )));
            };
            let source_path = Path::new(&source_path);
            if probe_git_origin(source_path).is_some() {
                update_from_git_source(&skill_path, &skill_id, &mut origin, force)?;
            } else if origin
                .update_command
                .as_ref()
                .is_some_and(|cmd| !cmd.trim().is_empty())
            {
                update_from_command_source(&skill_path, &skill_id, &mut origin)?;
            } else {
                return Err(VibeError::Path(format!(
                    "Skill {} 是本地目录安装，当前只能手动重新安装或切换到 Git 来源",
                    skill_id
                )));
            }
        }
        SOURCE_METHOD_NPM | SOURCE_METHOD_NPX => {
            if origin
                .update_command
                .as_ref()
                .is_some_and(|cmd| !cmd.trim().is_empty())
            {
                update_from_command_source(&skill_path, &skill_id, &mut origin)?;
            } else {
                return Err(VibeError::Path(format!(
                    "Skill {} 当前来源为 {}，暂不支持自动更新，请重新安装或重装来源包",
                    skill_id, method
                )));
            }
        }
        SOURCE_METHOD_MARKETPLACE => {
            update_plugin_skill(&skill_path, &skill_id, &mut origin)?;
        }
        _ => {
            return Err(VibeError::Path(format!(
                "Skill {} 来源未知，无法自动更新",
                skill_id
            )));
        }
    }

    list_skills()?
        .into_iter()
        .find(|skill| skill.id == skill_id)
        .ok_or(VibeError::SkillNotFound { skill_id })
}

fn update_from_git_source(
    skill_path: &Path,
    skill_id: &str,
    origin: &mut crate::models::origin::SkillOrigin,
    force: bool,
) -> Result<(), VibeError> {
    let Some(source_path) = origin.source_path.clone() else {
        return Err(VibeError::Path(format!(
            "Skill {} 缺少 Git 源路径，无法更新",
            skill_id
        )));
    };
    let source_path = Path::new(&source_path);
    if !source_path.exists() {
        return Err(VibeError::Path(format!(
            "Git 源路径不存在：{}",
            source_path.display()
        )));
    }
    if vibe_fs::normalize_path(source_path) == vibe_fs::normalize_path(skill_path) {
        return Err(VibeError::Path(format!(
            "Git 源路径与中心库路径相同：{}",
            source_path.display()
        )));
    }

    if !force && !git_status_clean(source_path)? {
        return Err(VibeError::Conflict {
            skill_id: skill_id.to_string(),
            details: "Git 源仓库存在未提交修改，请先处理本地变更或传入 force".to_string(),
        });
    }

    git_pull_ff_only(source_path)?;
    let probe = probe_git_origin(source_path).ok_or_else(|| {
        VibeError::Path(format!("无法读取 Git 源信息：{}", source_path.display()))
    })?;

    if vibe_fs::is_link(skill_path) {
        refresh_git_origin(origin, &probe);
        write_skill_origin_sidecar(skill_path, origin)?;
        return Ok(());
    }

    let temp_dir = skill_path.with_file_name(format!(".{}.update-tmp", skill_id));
    if temp_dir.exists() {
        if vibe_fs::is_link(&temp_dir) {
            vibe_fs::remove_symlink(&temp_dir)?;
        } else if temp_dir.is_dir() {
            fs::remove_dir_all(&temp_dir)?;
        } else {
            fs::remove_file(&temp_dir)?;
        }
    }

    copy_skill_dir_all(source_path, &temp_dir)?;
    if vibe_fs::is_link(skill_path) {
        vibe_fs::remove_symlink(skill_path)?;
    } else if skill_path.exists() {
        fs::remove_dir_all(skill_path)?;
    }
    fs::rename(&temp_dir, skill_path)?;

    refresh_git_origin(origin, &probe);
    write_skill_origin(skill_path, origin)?;
    Ok(())
}

fn update_from_command_source(
    skill_path: &Path,
    skill_id: &str,
    origin: &mut crate::models::origin::SkillOrigin,
) -> Result<(), VibeError> {
    let Some(source_path) = origin.source_path.clone() else {
        return Err(VibeError::Path(format!(
            "Skill {} 缺少 source_path，无法执行更新命令",
            skill_id
        )));
    };
    let source_path = Path::new(&source_path);

    let Some(command) = origin.update_command.clone() else {
        return Err(VibeError::Path(format!(
            "Skill {} 缺少 update_command，无法自动更新",
            skill_id
        )));
    };

    run_update_command(&command, Some(source_path))?;

    if vibe_fs::is_link(skill_path) {
        origin.last_checked_at = Some(crate::utils::datetime::chrono_now());
        if origin.trust_level.trim().is_empty() {
            origin.trust_level = "explicit".to_string();
        }
        write_skill_origin_sidecar(skill_path, origin)?;
        return Ok(());
    }

    if !source_path.exists() {
        return Err(VibeError::Path(format!(
            "更新命令执行后仍未找到来源目录：{}",
            source_path.display()
        )));
    }

    vibe_fs::clear_skill_dir_contents(skill_path)?;
    copy_skill_dir_all(source_path, skill_path)?;
    origin.last_checked_at = Some(crate::utils::datetime::chrono_now());
    if origin.trust_level.trim().is_empty() {
        origin.trust_level = "explicit".to_string();
    }
    write_skill_origin(skill_path, origin)?;
    Ok(())
}

#[tauri::command]
pub fn delete_library_skill(skill_id: String) -> Result<(), VibeError> {
    let vibe_dir = vibe_skills_dir()?;
    let skill_path = vibe_dir.join(&skill_id);

    if !skill_path.exists() {
        return Err(VibeError::SkillNotFound { skill_id });
    }

    let trash_dir = vibe_dir.join(".trash").join(&skill_id);
    if trash_dir.exists() {
        fs::remove_dir_all(&trash_dir)?;
    }
    copy_dir_all(&skill_path, &trash_dir)?;

    let agents = load_agents()?;
    for agent in &agents {
        let link_path = Path::new(&agent.skills_dir).join(&skill_id);
        if vibe_fs::is_link(&link_path) {
            let _ = vibe_fs::remove_symlink(&link_path);
        }
    }

    fs::remove_dir_all(&skill_path)?;

    if let Err(e) = record_action(HistoryAction::Delete, &skill_id, None, None) {
        warn!("Failed to record Delete action: {}", e);
    }

    Ok(())
}

#[tauri::command]
pub fn delete_skill(skill_id: String) -> Result<(), VibeError> {
    delete_library_skill(skill_id)
}

/// Restore a deleted skill from trash snapshot
pub fn restore_from_trash(skill_id: &str) -> Result<(), VibeError> {
    let vibe_dir = vibe_skills_dir()?;
    let trash_dir = vibe_dir.join(".trash").join(skill_id);
    let restore_to = vibe_dir.join(skill_id);

    if !trash_dir.exists() {
        return Err(VibeError::History(format!(
            "No snapshot found for skill '{}'",
            skill_id
        )));
    }

    copy_dir_all(&trash_dir, &restore_to)?;
    fs::remove_dir_all(&trash_dir)?;

    Ok(())
}

/// Move a skill to trash (for redo of undo-delete)
pub fn move_to_trash(skill_id: &str) -> Result<(), VibeError> {
    let vibe_dir = vibe_skills_dir()?;
    let skill_path = vibe_dir.join(skill_id);
    let trash_dir = vibe_dir.join(".trash").join(skill_id);

    if !skill_path.exists() {
        return Ok(());
    }

    if trash_dir.exists() {
        fs::remove_dir_all(&trash_dir)?;
    }
    copy_dir_all(&skill_path, &trash_dir)?;
    fs::remove_dir_all(&skill_path)?;

    Ok(())
}

struct SkillEntry {
    name: String,
    description: String,
    path: String,
    sources: Vec<SkillSource>,
    license: Option<String>,
    compatibility: Option<String>,
    metadata: Option<HashMap<String, String>>,
    has_scripts: bool,
    has_references: bool,
    has_assets: bool,
    modified_at: String,
}

fn source_kind_for(source_id: &str) -> String {
    if source_id == "vibe-lib" {
        "library".to_string()
    } else if source_id.starts_with("project:") {
        "project".to_string()
    } else if source_id.starts_with("external:") {
        "external".to_string()
    } else if source_id.starts_with("claude-plugin:") || source_id.starts_with("codex-plugin:") {
        "marketplace".to_string()
    } else {
        "agent".to_string()
    }
}

fn external_source_id(agent_id: &str, dir: &Path) -> String {
    format!(
        "external:{}:{}",
        agent_id,
        dir.to_string_lossy().replace('\\', "/")
    )
}

fn scan_project_sources(
    map: &mut HashMap<String, SkillEntry>,
    hash_cache: &mut crate::utils::hash::HashCache,
) -> Result<(), VibeError> {
    let config = load_config()?;

    for root in project_skill_roots(&config) {
        let root_id = format!("project:{}", root.to_string_lossy().replace('\\', "/"));
        for relative in [
            ".claude/skills",
            ".agents/skills",
            ".codex/skills",
            ".github/skills",
            "skills",
        ] {
            let skill_root = root.join(relative);
            if !skill_root.exists() || !skill_root.is_dir() {
                continue;
            }

            scan_directory(
                &skill_root,
                &root_id,
                map,
                false,
                0,
                &mut std::collections::HashSet::new(),
                hash_cache,
                None,
            )?;
        }
    }
    Ok(())
}

/// 读取 Claude Code 的 enabledPlugins 配置
/// 从 ~/.claude/settings.json 读取 enabledPlugins 字段
/// 返回 Map<plugin_key, enabled>，plugin_key 格式为 "name@marketplace"
fn read_claude_enabled_plugins() -> Result<HashMap<String, bool>, VibeError> {
    let home_dir = dirs::home_dir().ok_or_else(|| VibeError::Path("Home directory not found".to_string()))?;
    let settings_path = home_dir.join(".claude").join("settings.json");

    if !settings_path.exists() {
        return Ok(HashMap::new());
    }

    let content = fs::read_to_string(&settings_path)?;
    let settings: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| VibeError::Path(format!("Failed to parse Claude settings.json: {}", e)))?;

    let mut result = HashMap::new();

    if let Some(enabled_plugins) = settings.get("enabledPlugins").and_then(|v| v.as_object()) {
        for (key, value) in enabled_plugins {
            if let Some(enabled) = value.as_bool() {
                result.insert(key.clone(), enabled);
            }
        }
    }

    Ok(result)
}

/// 读取 Codex 的 plugin enabled 配置
/// 从 ~/.codex/config.toml 读取 [plugins."name@marketplace"] 段落
/// 返回 Map<plugin_key, enabled>
fn read_codex_plugin_enabled() -> Result<HashMap<String, bool>, VibeError> {
    let home_dir = dirs::home_dir().ok_or_else(|| VibeError::Path("Home directory not found".to_string()))?;
    let config_path = home_dir.join(".codex").join("config.toml");

    if !config_path.exists() {
        return Ok(HashMap::new());
    }

    let content = fs::read_to_string(&config_path)?;
    let config: toml::Value = toml::from_str(&content)
        .map_err(|e| VibeError::Path(format!("Failed to parse Codex config.toml: {}", e)))?;

    let mut result = HashMap::new();

    if let Some(plugins) = config.get("plugins").and_then(|v| v.as_table()) {
        for (key, value) in plugins {
            if let Some(enabled) = value.get("enabled").and_then(|v| v.as_bool()) {
                result.insert(key.clone(), enabled);
            }
        }
    }

    Ok(result)
}

/// 获取 plugin 的启用状态
/// 根据 plugin_type 和 plugin_name 查询对应配置
/// 返回 Some(true) = 已启用, Some(false) = 已禁用, None = 未配置
fn get_plugin_enabled(plugin_type: &str, plugin_name: &str) -> Option<bool> {
    match plugin_type {
        "claude-plugin" => {
            // Claude Code: 查询 ~/.claude/settings.json 的 enabledPlugins
            // plugin_key 格式为 "name@marketplace"
            if let Ok(enabled_map) = read_claude_enabled_plugins() {
                // 遍历所有 key，查找匹配的 plugin
                for (key, enabled) in &enabled_map {
                    // key 格式: "name@marketplace"
                    if let Some(at_pos) = key.find('@') {
                        let name = &key[..at_pos];
                        if name == plugin_name {
                            return Some(*enabled);
                        }
                    }
                }
            }
            // 未配置时返回 None（默认启用）
            None
        }
        "codex-plugin" => {
            // Codex: 查询 ~/.codex/config.toml 的 [plugins."name@marketplace"]
            if let Ok(enabled_map) = read_codex_plugin_enabled() {
                for (key, enabled) in &enabled_map {
                    if let Some(at_pos) = key.find('@') {
                        let name = &key[..at_pos];
                        if name == plugin_name {
                            return Some(*enabled);
                        }
                    }
                }
            }
            None
        }
        _ => None,
    }
}

/// 扫描 Claude Plugin 和 Codex Plugin 市场安装的 skill
/// Claude 目录结构: ~/.claude/plugins/cache/{marketplace}/{plugin-name}/{version}/skills/{skill-name}/SKILL.md
/// Codex 目录结构: ~/.codex/plugins/cache/{plugin-name}/{category}/{version}/skills/{skill-name}/SKILL.md
fn scan_plugin_marketplace_skills(
    map: &mut HashMap<String, SkillEntry>,
    hash_cache: &mut crate::utils::hash::HashCache,
) -> Result<(), VibeError> {
    let home_dir = dirs::home_dir().ok_or_else(|| VibeError::Path("Home directory not found".to_string()))?;

    // 扫描 Claude Plugin
    let claude_plugins_dir = home_dir.join(".claude").join("plugins").join("cache");
    scan_plugin_directory(&claude_plugins_dir, "claude-plugin", map, hash_cache)?;

    // 扫描 Codex Plugin
    let codex_plugins_dir = home_dir.join(".codex").join("plugins").join("cache");
    scan_plugin_directory(&codex_plugins_dir, "codex-plugin", map, hash_cache)?;

    Ok(())
}

/// 扫描插件目录中的 skill
/// 支持两种目录结构：
/// 1. {marketplace}/{plugin-name}/{version}/skills/ (Claude Plugin)
/// 2. {plugin-name}/{category}/{version}/skills/ (Codex Plugin)
fn scan_plugin_directory(
    plugins_dir: &Path,
    plugin_type: &str,
    map: &mut HashMap<String, SkillEntry>,
    hash_cache: &mut crate::utils::hash::HashCache,
) -> Result<(), VibeError> {
    if !plugins_dir.exists() || !plugins_dir.is_dir() {
        return Ok(());
    }

    // 遍历第一层目录（marketplace 或 plugin-name）
    for first_level_entry in fs::read_dir(plugins_dir)? {
        let first_level_entry = first_level_entry?;
        let first_level_path = first_level_entry.path();
        if !first_level_path.is_dir() {
            continue;
        }
        let first_level_name = first_level_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        // 遍历第二层目录（plugin-name 或 category）
        for second_level_entry in fs::read_dir(&first_level_path)? {
            let second_level_entry = second_level_entry?;
            let second_level_path = second_level_entry.path();
            if !second_level_path.is_dir() {
                continue;
            }

            // 检查是否有 skills 目录（直接在第二层）
            let skills_dir = second_level_path.join("skills");
            if skills_dir.exists() && skills_dir.is_dir() {
                let source_id = format!("{}:{}", plugin_type, first_level_name);
                scan_directory(
                    &skills_dir,
                    &source_id,
                    map,
                    false,
                    0,
                    &mut std::collections::HashSet::new(),
                    hash_cache,
                    None,
                )?;
                continue;
            }

            // 遍历第三层目录（version）
            for third_level_entry in fs::read_dir(&second_level_path)? {
                let third_level_entry = third_level_entry?;
                let third_level_path = third_level_entry.path();
                if !third_level_path.is_dir() {
                    continue;
                }

                let skills_dir = third_level_path.join("skills");
                if !skills_dir.exists() || !skills_dir.is_dir() {
                    continue;
                }

                let source_id = format!("{}:{}", plugin_type, first_level_name);
                scan_directory(
                    &skills_dir,
                    &source_id,
                    map,
                    false,
                    0,
                    &mut std::collections::HashSet::new(),
                    hash_cache,
                    None,
                )?;
            }
        }
    }

    Ok(())
}

/// 递归扫描目录，找到所有包含 SKILL.md 的子目录
/// symlink_only=true 时跳过真实文件（仅扫描 symlink/junction）
fn scan_directory(
    dir: &Path,
    source_id: &str,
    map: &mut HashMap<String, SkillEntry>,
    symlink_only: bool,
    depth: usize,
    visited: &mut std::collections::HashSet<std::path::PathBuf>,
    hash_cache: &mut crate::utils::hash::HashCache,
    ignored_root_dirs: Option<&std::collections::HashSet<String>>,
) -> Result<(), VibeError> {
    if !dir.exists() {
        return Ok(());
    }

    if depth > MAX_SCAN_DEPTH || !visited.insert(vibe_fs::normalize_path(dir)) {
        return Ok(());
    }

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if !path.is_dir() && !vibe_fs::is_link(&path) {
            continue;
        }

        let id = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        if id.starts_with('.') {
            continue;
        }

        // Legacy agent mirror directories (`~/.vibe-skills/{agent_id}/...`) are
        // not center-library skill entities. Keeping them out of vibe-lib scan
        // avoids merging mirrored agent links into the canonical skill list.
        if depth == 0
            && source_id == "vibe-lib"
            && ignored_root_dirs
                .map(|ignored| ignored.contains(&id))
                .unwrap_or(false)
        {
            continue;
        }

        let is_link = vibe_fs::is_link(&path);
        if source_id.starts_with("project:") && is_link {
            continue;
        }
        let symlink_target = if is_link {
            vibe_fs::read_link_target(&path)
                .ok()
                .map(|p| p.to_string_lossy().to_string())
        } else {
            None
        };

        let is_broken_link = is_link
            && symlink_target
                .as_ref()
                .map(|target| !vibe_fs::normalize_path(Path::new(target)).exists())
                .unwrap_or(true);

        if is_broken_link {
            let modified_at = get_modified_at(&path);
            let origin = read_skill_origin(&path);
            let trust_level = trust_level_for(origin.as_ref());
            let update_status = update_status_for(origin.as_ref(), Some(&path));
            let source = SkillSource {
                from: source_id.to_string(),
                source_kind: source_kind_for(source_id),
                path: path.to_string_lossy().to_string(),
                name: id.clone(),
                description: String::new(),
                is_symlink: true,
                symlink_target,
                content_hash: String::new(),
                modified_at: modified_at.clone(),
                trust_level,
                update_status,
                origin,
            };

            map.entry(id.clone())
                .and_modify(|e| {
                    e.sources.push(source.clone());
                })
                .or_insert_with(|| SkillEntry {
                    name: id.clone(),
                    description: String::new(),
                    path: path.to_string_lossy().to_string(),
                    sources: vec![source],
                    license: None,
                    compatibility: None,
                    metadata: None,
                    has_scripts: false,
                    has_references: false,
                    has_assets: false,
                    modified_at,
                });
            continue;
        }

        let skill_md_path = path.join("SKILL.md");
        if skill_md_path.exists() {
            // agent 目录：只保留 symlink，跳过真实文件
            if symlink_only && !is_link {
                continue;
            }

            let (name, description, license, compatibility, metadata, _body) =
                parse_skill_md_full(&skill_md_path).unwrap_or_else(|_| {
                    (id.clone(), String::new(), None, None, None, String::new())
                });

            // P1：哈希缓存——三元组未变时复用真哈希，避免重复读文件
            let hash = crate::utils::hash::dir_hash_into(hash_cache, &path);
            let modified_at = get_modified_at(&path);
            let origin = read_skill_origin(&path);
            let trust_level = trust_level_for(origin.as_ref());
            let update_status = update_status_for(origin.as_ref(), Some(&path));

            let source = SkillSource {
                from: source_id.to_string(),
                source_kind: source_kind_for(source_id),
                path: path.to_string_lossy().to_string(),
                name: name.clone(),
                description: description.clone(),
                is_symlink: is_link,
                symlink_target,
                content_hash: hash,
                modified_at: modified_at.clone(),
                trust_level,
                update_status,
                origin,
            };

            map.entry(id.clone())
                .and_modify(|e| {
                    e.sources.push(source.clone());
                })
                .or_insert_with(|| SkillEntry {
                    name,
                    description,
                    path: path.to_string_lossy().to_string(),
                    sources: vec![source],
                    license,
                    compatibility,
                    metadata,
                    has_scripts: path.join("scripts").is_dir(),
                    has_references: path.join("references").is_dir(),
                    has_assets: path.join("assets").is_dir(),
                    modified_at,
                });
        } else {
            scan_directory(
                &path,
                source_id,
                map,
                symlink_only,
                depth + 1,
                visited,
                hash_cache,
                ignored_root_dirs,
            )?;
        }
    }

    Ok(())
}

fn find_linked_agents(skill_id: &str, agents: &[crate::models::agent::Agent]) -> Vec<String> {
    let mut linked = Vec::new();

    for agent in agents {
        if !agent.detected {
            continue;
        }
        // P2：统一复用 scan_linked_skills，避免 Windows junction 归一化分歧
        let linked_for_agent =
            crate::utils::config::scan_linked_skills(Path::new(&agent.skills_dir));
        if linked_for_agent.iter().any(|id| id == skill_id) {
            linked.push(agent.id.clone());
        }
    }

    linked
}

fn get_modified_at(path: &Path) -> String {
    fs::metadata(path)
        .and_then(|m| m.modified())
        .map(datetime::system_time_to_iso)
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn git(path: &Path, args: &[&str]) {
        let output = Command::new("git")
            .arg("-C")
            .arg(path)
            .args(args)
            .output()
            .expect("git should start");
        assert!(
            output.status.success(),
            "git {:?} failed: {}",
            args,
            String::from_utf8_lossy(&output.stderr)
        );
    }

    #[test]
    fn mock_git_update_is_detected() {
        let root = std::env::temp_dir().join(format!("qs-vibe-mock-update-{}", Uuid::new_v4()));
        let source = root.join("source");
        let clone = root.join("clone");
        let destination = root.join("library-skill");
        fs::create_dir_all(&source).unwrap();
        fs::write(
            source.join("SKILL.md"),
            "---\nname: mock-skill\ndescription: initial\n---\n",
        )
        .unwrap();

        git(&source, &["init", "-b", "main"]);
        git(&source, &["config", "user.email", "mock@example.com"]);
        git(&source, &["config", "user.name", "mock"]);
        git(&source, &["add", "."]);
        git(&source, &["commit", "-m", "initial"]);
        let output = Command::new("git")
            .args(["clone", "-q"])
            .arg(&source)
            .arg(&clone)
            .output()
            .unwrap();
        assert!(output.status.success());
        fs::create_dir_all(&destination).unwrap();
        fs::copy(source.join("SKILL.md"), destination.join("SKILL.md")).unwrap();

        fs::write(
            source.join("SKILL.md"),
            "---\nname: mock-skill\ndescription: updated\n---\n",
        )
        .unwrap();
        git(&source, &["add", "."]);
        git(&source, &["commit", "-m", "update"]);

        let mut origin = crate::models::origin::SkillOrigin {
            method: SOURCE_METHOD_GIT.to_string(),
            provider: Some("local".to_string()),
            url: Some(source.to_string_lossy().to_string()),
            commit: git_output(&clone, &["rev-parse", "HEAD"]),
            branch: Some("main".to_string()),
            installed_at: datetime::chrono_now(),
            installed_by: Some("test".to_string()),
            trust_level: "explicit".to_string(),
            source_path: Some(clone.to_string_lossy().to_string()),
            command: None,
            update_command: Some("git pull --ff-only".to_string()),
            refresh_command: Some("git pull --ff-only".to_string()),
            package_name: None,
            version: None,
            sync_mode: Some("copy".to_string()),
            last_checked_at: None,
        };

        let result = check_git_source_update("mock-skill", &origin, datetime::chrono_now()).unwrap();
        assert!(result.available);
        assert!(result.current_commit.is_some());
        assert!(result.remote_commit.is_some());
        assert_ne!(result.current_commit, result.remote_commit);

        update_from_git_source(&destination, "mock-skill", &mut origin, false).unwrap();
        let updated_skill = fs::read_to_string(destination.join("SKILL.md")).unwrap();
        assert!(updated_skill.contains("description: updated"));

        fs::remove_dir_all(root).unwrap();
    }
}
