use std::collections::HashMap;
use std::fs;
use std::path::Path;

use tracing::warn;

use crate::errors::VibeError;
use crate::models::dashboard::{
    DashboardAgent, DashboardData, DashboardSkill, DashboardStats, SharedSkillInfo,
};
use crate::models::history::HistoryAction;
use crate::models::skill::{ConflictType, Skill, SkillIssue, SkillSource};
use crate::parsers::skill_md::parse_skill_md_full;
use crate::utils::config::{
    build_agents_from_config, load_agents, load_config, project_skill_roots,
};
use crate::utils::datetime;
use crate::utils::fs as vibe_fs;
use crate::utils::fs::copy_dir_all;
use crate::utils::history::record_action;
use crate::utils::origin::{
    build_install_origin, read_skill_origin, trust_level_for, update_status_for, write_skill_origin,
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
        if !agent.detected {
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

    crate::utils::hash::save_hash_cache(&vibe_dir, &hash_cache);

    let mut skills: Vec<Skill> = map
        .into_iter()
        .map(|(id, entry)| {
            let linked_agents = find_linked_agents(&id, &agents);

            // 检测冲突：多个 source 的 content_hash 不完全相同
            let unique_hashes: Vec<&str> = entry
                .sources
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

    Ok(skills)
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
pub fn install_skill(source_path: String) -> Result<Skill, VibeError> {
    let source = Path::new(&source_path);
    if !source.exists() {
        return Err(VibeError::InvalidSkillMd {
            reason: format!("Source path does not exist: {}", source_path),
        });
    }

    let skill_md = source.join("SKILL.md");
    if !skill_md.exists() {
        return Err(VibeError::InvalidSkillMd {
            reason: "Source directory does not contain SKILL.md".to_string(),
        });
    }

    let (name, description, license, compatibility, metadata, _body) =
        parse_skill_md_full(&skill_md)?;

    let vibe_dir = vibe_skills_dir()?;
    let dest = vibe_dir.join(&name);

    if dest.exists() {
        return Err(VibeError::SkillAlreadyExists { skill_id: name });
    }

    copy_dir_all(source, &dest)?;

    let origin = build_install_origin(source);
    write_skill_origin(&dest, &origin)?;

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
            trust_level: trust_level_for(Some(&origin)),
            update_status: update_status_for(Some(&origin)),
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
    })
}

#[tauri::command]
pub fn delete_skill(skill_id: String) -> Result<(), VibeError> {
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
                trust_level: trust_level_for(origin.as_ref()),
                update_status: update_status_for(origin.as_ref()),
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
                trust_level: trust_level_for(origin.as_ref()),
                update_status: update_status_for(origin.as_ref()),
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
