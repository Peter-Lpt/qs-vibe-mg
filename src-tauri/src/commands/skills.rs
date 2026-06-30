use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::errors::VabError;
use crate::models::dashboard::{
    DashboardAgent, DashboardData, DashboardSkill, DashboardStats, SharedSkillInfo,
};
use crate::models::skill::{Skill, SkillSource};
use crate::parsers::skill_md::parse_skill_md_full;
use crate::utils::config::{build_agents_from_config, load_config};
use crate::utils::fs as vibe_fs;
use crate::utils::history::record_action;
use crate::utils::path::vibe_skills_dir;
use crate::models::history::HistoryAction;

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
            }
        })
        .collect();

    skills.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
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
        .filter(|s| s.name.to_lowercase().contains(&q))
        .collect();

    Ok(results)
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
    // 搜索所有可能的位置：vibe-skills + 各 agent 目录
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
        // 递归搜索子目录
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

    let _ = record_action(HistoryAction::Install, &name, None, None);

    let modified_at = get_modified_at(&dest);
    Ok(Skill {
        id: name.clone(),
        name,
        description,
        path: dest.to_string_lossy().to_string(),
        linked_agents: Vec::new(),
        sources: vec![SkillSource {
            from: "vibe-lib".to_string(),
            path: dest.to_string_lossy().to_string(),
        }],
        license,
        compatibility,
        metadata,
        has_scripts: dest.join("scripts").is_dir(),
        has_references: dest.join("references").is_dir(),
        has_assets: dest.join("assets").is_dir(),
        modified_at,
    })
}

#[tauri::command]
pub fn delete_skill(skill_id: String) -> Result<(), VabError> {
    let skill_path = vibe_skills_dir()?.join(&skill_id);

    if !skill_path.exists() {
        return Err(VabError::SkillNotFound { skill_id });
    }

    let config = load_config()?;
    let agents = build_agents_from_config(&config)?;
    for agent in &agents {
        let link_path = Path::new(&agent.skills_dir).join(&skill_id);
        if vibe_fs::is_link(&link_path) {
            let _ = vibe_fs::remove_symlink(&link_path);
        }
    }

    fs::remove_dir_all(&skill_path)?;

    let _ = record_action(HistoryAction::Delete, &skill_id, None, None);

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
            // 找到 skill，解析并加入 map
            let (name, description, license, compatibility, metadata, _body) =
                parse_skill_md_full(&skill_md_path)
                    .unwrap_or_else(|_| (id.clone(), String::new(), None, None, None, String::new()));

            let source = SkillSource {
                from: source_id.to_string(),
                path: path.to_string_lossy().to_string(),
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
            // 不是 skill 目录，递归扫描子目录
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

fn copy_dir_all(src: &Path, dst: &Path) -> Result<(), VabError> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let dest = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dest)?;
        } else {
            fs::copy(entry.path(), &dest)?;
        }
    }
    Ok(())
}

fn get_modified_at(path: &Path) -> String {
    use std::time::UNIX_EPOCH;

    fs::metadata(path)
        .and_then(|m| m.modified())
        .map(|t| {
            let duration = t.duration_since(UNIX_EPOCH).unwrap_or_default();
            let secs = duration.as_secs();
            let days = secs / 86400;
            let time_of_day = secs % 86400;
            let hours = time_of_day / 3600;
            let minutes = (time_of_day % 3600) / 60;
            let seconds = time_of_day % 60;
            let (year, month, day) = days_to_ymd(days);
            format!(
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
                year, month, day, hours, minutes, seconds
            )
        })
        .unwrap_or_default()
}

fn days_to_ymd(mut days: u64) -> (u64, u64, u64) {
    days += 719468;
    let era = days / 146097;
    let doe = days % 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}
