use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::errors::VabError;
use crate::models::skill::{Skill, SkillSource};
use crate::parsers::skill_md::parse_skill_md;
use crate::utils::config::{build_agents_from_config, load_config};
use crate::utils::fs as vab_fs;
use crate::utils::path::vab_skills_dir;

/// 扫描所有 skill 来源：~/.vab-skills/ + 各 agent 目录，合并去重
#[tauri::command]
pub fn list_skills() -> Result<Vec<Skill>, VabError> {
    let mut map: HashMap<String, SkillEntry> = HashMap::new();

    // 1. 扫描 ~/.vab-skills/
    let vab_dir = vab_skills_dir()?;
    scan_directory(&vab_dir, "vab-lib", &mut map)?;

    // 2. 扫描所有 agent 的 skills 目录
    let config = load_config()?;
    let agents = build_agents_from_config(&config)?;

    for agent in &agents {
        if !agent.detected {
            continue;
        }
        let agent_dir = Path::new(&agent.skills_dir);
        scan_directory(agent_dir, &agent.id, &mut map)?;
    }

    // 3. 构建结果
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
            }
        })
        .collect();

    skills.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(skills)
}

/// 临时结构，用于合并
struct SkillEntry {
    name: String,
    description: String,
    path: String,
    sources: Vec<SkillSource>,
}

/// 扫描一个目录下的所有子文件夹作为 skill
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
        let (name, description) = if skill_md_path.exists() {
            parse_skill_md(&skill_md_path).unwrap_or_else(|_| (id.clone(), String::new()))
        } else {
            (id.clone(), String::new())
        };

        let source = SkillSource {
            from: source_id.to_string(),
            path: path.to_string_lossy().to_string(),
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
            });
    }

    Ok(())
}

/// 检查指定 skill 关联了哪些 agent
fn find_linked_agents(skill_id: &str, agents: &[crate::models::agent::Agent]) -> Vec<String> {
    let mut linked = Vec::new();

    for agent in agents {
        if !agent.detected {
            continue;
        }
        let link_path = Path::new(&agent.skills_dir).join(skill_id);
        if vab_fs::is_link(&link_path) {
            if let Ok(target) = vab_fs::read_link_target(&link_path) {
                if let Ok(vab_dir) = vab_skills_dir() {
                    let expected = vab_dir.join(skill_id);
                    if target == expected {
                        linked.push(agent.id.clone());
                    }
                }
            }
        }
    }

    linked
}
