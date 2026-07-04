use std::collections::HashMap;
use std::fs;
use std::path::Path;

use sha2::{Digest, Sha256};
use tracing::warn;

use crate::errors::VabError;
use crate::models::dashboard::{
    DashboardAgent, DashboardData, DashboardSkill, DashboardStats, SharedSkillInfo,
};
use crate::models::skill::{ConflictType, Skill, SkillIssue, SkillSource};
use crate::parsers::skill_md::parse_skill_md_full;
use crate::utils::config::{build_agents_from_config, load_config};
use crate::utils::datetime;
use crate::utils::fs as vibe_fs;
use crate::utils::fs::copy_dir_all;
use crate::utils::history::record_action;
use crate::utils::path::vibe_skills_dir;
use crate::models::history::HistoryAction;

/// 计算文件内容的 SHA-256 hex hash
fn content_hash(path: &Path) -> String {
    let Ok(content) = fs::read_to_string(path) else {
        return String::new();
    };
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[tauri::command]
pub fn list_skills() -> Result<Vec<Skill>, VabError> {
    let mut map: HashMap<String, SkillEntry> = HashMap::new();

    let vibe_dir = vibe_skills_dir()?;
    scan_directory(&vibe_dir, "vibe-lib", &mut map)?;

    let config = load_config()?;
    let agents = build_agents_from_config(&config)?;

    for agent in &agents {
        if !agent.detected {
            continue;
        }
        let agent_dir = Path::new(&agent.skills_dir);
        scan_directory(agent_dir, &agent.id, &mut map)?;
    }

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
                    Some(target) => !Path::new(target).exists(),
                    None => true,
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
                has_conflict,
                has_dangling,
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
pub fn search_skills(query: String) -> Result<Vec<Skill>, VabError> {
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
pub fn detect_issues() -> Result<Vec<SkillIssue>, VabError> {
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
                description: format!(
                    "同名 skill 有不同内容: {}",
                    source_names.join(", ")
                ),
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
                description: format!(
                    "断链指向已删除路径: {}",
                    broken_sources.join(", ")
                ),
            });
        }
    }

    Ok(issues)
}

#[tauri::command]
pub fn get_dashboard_data() -> Result<DashboardData, VabError> {
    let config = load_config()?;
    let agents = build_agents_from_config(&config)?;
    let vibe_dir = vibe_skills_dir()?;

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
        collect_skills_recursive(skills_dir, skills_dir, &mut skills, &mut all_skill_agents, &agent.id);

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
                        .map(|ids| {
                            ids.iter()
                                .filter(|id| *id != &agent.id)
                                .cloned()
                                .collect()
                        })
                        .unwrap_or_default();

                    DashboardSkill {
                        skill_id: skill_id.clone(),
                        skill_name: skill_name.clone(),
                        shared_with,
                    }
                })
                .collect();

            dashboard_skills.sort_by(|a, b| {
                b.shared_with.len().cmp(&a.shared_with.len())
                    .then(a.skill_name.to_lowercase().cmp(&b.skill_name.to_lowercase()))
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
    if vibe_dir.exists() {
        collect_vibe_skills(&vibe_dir, &mut vibe_skills, &all_skill_agents, &mut total_skills);
    }

    let mut all_agents = dashboard_agents;
    if !vibe_skills.is_empty() {
        all_agents.insert(
            0,
            DashboardAgent {
                agent_id: "vibe-lib".to_string(),
                agent_name: "VAB Library".to_string(),
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
    })
}

fn collect_skills_recursive(
    base_dir: &Path,
    dir: &Path,
    skills: &mut Vec<(String, String)>,
    all_skill_agents: &mut HashMap<String, Vec<String>>,
    agent_id: &str,
) {
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
                collect_skills_recursive(base_dir, &path, skills, all_skill_agents, agent_id);
            }
        }
    }
}

fn collect_vibe_skills(
    dir: &Path,
    vibe_skills: &mut Vec<DashboardSkill>,
    all_skill_agents: &HashMap<String, Vec<String>>,
    total_skills: &mut std::collections::HashSet<String>,
) {
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
                collect_vibe_skills(&path, vibe_skills, all_skill_agents, total_skills);
            }
        }
    }
}

#[tauri::command]
pub fn preview_skill(skill_id: String) -> Result<String, VabError> {
    let vibe_dir = vibe_skills_dir()?;
    let vibe_path = vibe_dir.join(&skill_id).join("SKILL.md");
    if vibe_path.exists() {
        return fs::read_to_string(&vibe_path).map_err(VabError::Io);
    }

    let config = load_config()?;
    let agents = build_agents_from_config(&config)?;
    for agent in &agents {
        if !agent.detected {
            continue;
        }
        let agent_path = Path::new(&agent.skills_dir).join(&skill_id).join("SKILL.md");
        if agent_path.exists() {
            return fs::read_to_string(&agent_path).map_err(VabError::Io);
        }
        if let Ok(content) = find_skill_md_recursive(&Path::new(&agent.skills_dir), &skill_id) {
            return Ok(content);
        }
    }

    Err(VabError::SkillNotFound { skill_id })
}

fn find_skill_md_recursive(dir: &Path, skill_id: &str) -> Result<String, VabError> {
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
                return fs::read_to_string(&skill_md).map_err(VabError::Io);
            }
        }
        if name.starts_with('.') {
            continue;
        }
        if let Ok(content) = find_skill_md_recursive(&path, skill_id) {
            return Ok(content);
        }
    }
    Err(VabError::SkillNotFound {
        skill_id: skill_id.to_string(),
    })
}

#[tauri::command]
pub fn install_skill(source_path: String) -> Result<Skill, VabError> {
    let source = Path::new(&source_path);
    if !source.exists() {
        return Err(VabError::InvalidSkillMd {
            reason: format!("Source path does not exist: {}", source_path),
        });
    }

    let skill_md = source.join("SKILL.md");
    if !skill_md.exists() {
        return Err(VabError::InvalidSkillMd {
            reason: "Source directory does not contain SKILL.md".to_string(),
        });
    }

    let (name, description, license, compatibility, metadata, _body) =
        parse_skill_md_full(&skill_md)?;

    let vibe_dir = vibe_skills_dir()?;
    let dest = vibe_dir.join(&name);

    if dest.exists() {
        return Err(VabError::SkillAlreadyExists { skill_id: name });
    }

    copy_dir_all(source, &dest)?;

    if let Err(e) = record_action(HistoryAction::Install, &name, None, None) {
        warn!("Failed to record Install action: {}", e);
    }

    let modified_at = get_modified_at(&dest);
    let hash = content_hash(&dest.join("SKILL.md"));

    Ok(Skill {
        id: name.clone(),
        name: name.clone(),
        description,
        path: dest.to_string_lossy().to_string(),
        linked_agents: Vec::new(),
        sources: vec![SkillSource {
            from: "vibe-lib".to_string(),
            path: dest.to_string_lossy().to_string(),
            name,
            description: String::new(),
            is_symlink: false,
            symlink_target: None,
            content_hash: hash,
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
    })
}

#[tauri::command]
pub fn delete_skill(skill_id: String) -> Result<(), VabError> {
    let vibe_dir = vibe_skills_dir()?;
    let skill_path = vibe_dir.join(&skill_id);

    if !skill_path.exists() {
        return Err(VabError::SkillNotFound { skill_id });
    }

    let trash_dir = vibe_dir.join(".trash").join(&skill_id);
    if trash_dir.exists() {
        fs::remove_dir_all(&trash_dir)?;
    }
    copy_dir_all(&skill_path, &trash_dir)?;

    let config = load_config()?;
    let agents = build_agents_from_config(&config)?;
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
pub fn restore_from_trash(skill_id: &str) -> Result<(), VabError> {
    let vibe_dir = vibe_skills_dir()?;
    let trash_dir = vibe_dir.join(".trash").join(skill_id);
    let restore_to = vibe_dir.join(skill_id);

    if !trash_dir.exists() {
        return Err(VabError::History(format!(
            "No snapshot found for skill '{}'",
            skill_id
        )));
    }

    copy_dir_all(&trash_dir, &restore_to)?;
    fs::remove_dir_all(&trash_dir)?;

    Ok(())
}

/// Move a skill to trash (for redo of undo-delete)
pub fn move_to_trash(skill_id: &str) -> Result<(), VabError> {
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

/// 递归扫描目录，找到所有包含 SKILL.md 的子目录
fn scan_directory(
    dir: &Path,
    source_id: &str,
    map: &mut HashMap<String, SkillEntry>,
) -> Result<(), VabError> {
    if !dir.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        let id = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        if id.starts_with('.') {
            continue;
        }

        let skill_md_path = path.join("SKILL.md");
        if skill_md_path.exists() {
            let (name, description, license, compatibility, metadata, _body) =
                parse_skill_md_full(&skill_md_path)
                    .unwrap_or_else(|_| (id.clone(), String::new(), None, None, None, String::new()));

            let is_symlink = vibe_fs::is_link(&path);
            let symlink_target = if is_symlink {
                vibe_fs::read_link_target(&path)
                    .ok()
                    .map(|p| p.to_string_lossy().to_string())
            } else {
                None
            };

            let hash = content_hash(&skill_md_path);

            let source = SkillSource {
                from: source_id.to_string(),
                path: path.to_string_lossy().to_string(),
                name: name.clone(),
                description: description.clone(),
                is_symlink,
                symlink_target,
                content_hash: hash,
            };

            let modified_at = get_modified_at(&path);

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
            scan_directory(&path, source_id, map)?;
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
        let link_path = Path::new(&agent.skills_dir).join(skill_id);
        if vibe_fs::is_link(&link_path) {
            if let Ok(target) = vibe_fs::read_link_target(&link_path) {
                if let Ok(vibe_dir) = vibe_skills_dir() {
                    let expected = vibe_dir.join(skill_id);
                    if target == expected {
                        linked.push(agent.id.clone());
                    }
                }
            }
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
