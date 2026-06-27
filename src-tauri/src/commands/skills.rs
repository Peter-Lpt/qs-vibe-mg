use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::errors::VabError;
use crate::models::skill::{Skill, SkillSource};
use crate::parsers::skill_md::parse_skill_md_full;
use crate::utils::config::{build_agents_from_config, load_config};
use crate::utils::fs as vab_fs;
use crate::utils::history::{record_action};
use crate::utils::path::vab_skills_dir;
use crate::models::history::HistoryAction;

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

/// 预览 SKILL.md 内容
#[tauri::command]
pub fn preview_skill(skill_id: String) -> Result<String, VabError> {
    let skill_path = vab_skills_dir()?.join(&skill_id).join("SKILL.md");

    if !skill_path.exists() {
        return Err(VabError::SkillNotFound { skill_id });
    }

    fs::read_to_string(&skill_path).map_err(VabError::Io)
}

/// 安装 skill（从外部路径复制到 ~/.vab-skills/）
#[tauri::command]
pub fn install_skill(source_path: String) -> Result<Skill, VabError> {
    let source = Path::new(&source_path);
    if !source.exists() {
        return Err(VabError::InvalidSkillMd {
            reason: format!("Source path does not exist: {}", source_path),
        });
    }

    // 解析 SKILL.md 获取 name
    let skill_md = source.join("SKILL.md");
    if !skill_md.exists() {
        return Err(VabError::InvalidSkillMd {
            reason: "Source directory does not contain SKILL.md".to_string(),
        });
    }

    let (name, description, license, compatibility, metadata, _body) =
        parse_skill_md_full(&skill_md)?;

    // 使用 name 作为目录名
    let vab_dir = vab_skills_dir()?;
    let dest = vab_dir.join(&name);

    if dest.exists() {
        return Err(VabError::SkillAlreadyExists { skill_id: name });
    }

    // 复制整个目录
    copy_dir_all(source, &dest)?;

    // 记录历史
    let _ = record_action(HistoryAction::Install, &name, None, None);

    // 返回安装的 skill 信息
    let modified_at = get_modified_at(&dest);
    Ok(Skill {
        id: name.clone(),
        name,
        description,
        path: dest.to_string_lossy().to_string(),
        linked_agents: Vec::new(),
        sources: vec![SkillSource {
            from: "vab-lib".to_string(),
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

/// 删除 skill（从 ~/.vab-skills/ 删除）
#[tauri::command]
pub fn delete_skill(skill_id: String) -> Result<(), VabError> {
    let skill_path = vab_skills_dir()?.join(&skill_id);

    if !skill_path.exists() {
        return Err(VabError::SkillNotFound { skill_id });
    }

    // 先删除所有 agent 的 symlink
    let config = load_config()?;
    let agents = build_agents_from_config(&config)?;
    for agent in &agents {
        let link_path = Path::new(&agent.skills_dir).join(&skill_id);
        if vab_fs::is_link(&link_path) {
            let _ = vab_fs::remove_symlink(&link_path);
        }
    }

    // 删除 skill 目录
    fs::remove_dir_all(&skill_path)?;

    // 记录历史
    let _ = record_action(HistoryAction::Delete, &skill_id, None, None);

    Ok(())
}

/// 临时结构，用于合并
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
        let (name, description, license, compatibility, metadata, _body) =
            if skill_md_path.exists() {
                parse_skill_md_full(&skill_md_path)
                    .unwrap_or_else(|_| (id.clone(), String::new(), None, None, None, String::new()))
            } else {
                (id.clone(), String::new(), None, None, None, String::new())
            };

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

/// 递归复制目录
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

/// 获取目录的最后修改时间
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
