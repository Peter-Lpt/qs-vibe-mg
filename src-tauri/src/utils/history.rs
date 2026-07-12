use std::fs;
use std::path::Path;

use uuid::Uuid;

use crate::errors::VibeError;
use crate::models::agent::Agent;
use crate::models::history::{HistoryAction, HistoryEntry, HistoryStore};
use crate::utils::config::{build_agents_from_config, load_config};
use crate::utils::datetime;
use crate::utils::fs as vibe_fs;
use crate::utils::path::vibe_skills_dir;

const HISTORY_FILE: &str = ".vibe-history.json";

fn history_path() -> Result<std::path::PathBuf, VibeError> {
    Ok(vibe_skills_dir()?.join(HISTORY_FILE))
}

/// 加载历史记录
pub fn load_history() -> Result<HistoryStore, VibeError> {
    let path = history_path()?;
    if !path.exists() {
        return Ok(HistoryStore::default());
    }

    let content = fs::read_to_string(&path)?;
    let store: HistoryStore =
        serde_json::from_str(&content).map_err(|e| VibeError::History(e.to_string()))?;
    Ok(store)
}

/// 保存历史记录
pub fn save_history(store: &HistoryStore) -> Result<(), VibeError> {
    let path = history_path()?;
    let dir = vibe_skills_dir()?;
    if !dir.exists() {
        fs::create_dir_all(&dir)?;
    }

    let content =
        serde_json::to_string_pretty(store).map_err(|e| VibeError::History(e.to_string()))?;
    fs::write(&path, content)?;
    Ok(())
}

/// 记录一条操作历史
pub fn record_action(
    action: HistoryAction,
    skill_id: &str,
    agent_id: Option<&str>,
    mode: Option<&str>,
) -> Result<HistoryEntry, VibeError> {
    record_action_with_skills(action, skill_id, None, agent_id, mode)
}

/// 记录一条操作历史（批量/同步操作可携带受影响的 skill 列表）
pub fn record_action_with_skills(
    action: HistoryAction,
    skill_id: &str,
    skill_ids: Option<Vec<String>>,
    agent_id: Option<&str>,
    mode: Option<&str>,
) -> Result<HistoryEntry, VibeError> {
    let mut store = load_history()?;

    // 超出限制时删除最旧的
    let config = load_config()?;
    let max = config.history.max_entries as usize;
    while store.entries.len() >= max {
        store.entries.remove(0);
    }

    let entry = HistoryEntry {
        id: Uuid::new_v4().to_string(),
        timestamp: datetime::chrono_now(),
        action,
        skill_id: skill_id.to_string(),
        skill_ids: skill_ids.unwrap_or_default(),
        agent_id: agent_id.map(|s| s.to_string()),
        mode: mode.map(|s| s.to_string()),
        undone: false,
    };

    store.entries.push(entry.clone());
    save_history(&store)?;
    Ok(entry)
}

/// 获取最后一条未撤销的记录（用于撤销）
pub fn last_active_for_undo() -> Result<Option<HistoryEntry>, VibeError> {
    let store = load_history()?;
    Ok(store.entries.iter().rev().find(|e| !e.undone).cloned())
}

/// 获取最后一条已撤销的记录（用于重做）
pub fn last_undone_for_redo() -> Result<Option<HistoryEntry>, VibeError> {
    let store = load_history()?;
    Ok(store.entries.iter().rev().find(|e| e.undone).cloned())
}

/// 标记记录为已撤销
pub fn mark_undone(id: &str, undone: bool) -> Result<(), VibeError> {
    let mut store = load_history()?;
    if let Some(entry) = store.entries.iter_mut().find(|e| e.id == id) {
        entry.undone = undone;
    }
    save_history(&store)
}

/// 清空所有历史记录
pub fn clear_history() -> Result<(), VibeError> {
    // 先写入空 store，写入成功再清空内存（防止IO失败导致状态不一致）
    let empty_store = HistoryStore::default();
    save_history(&empty_store)?;
    Ok(())
}

/// 根据 agent_id 解析 Agent 对象
pub fn resolve_agent(agent_id: &str) -> Result<Agent, VibeError> {
    let config = load_config()?;
    let agents = build_agents_from_config(&config)?;
    agents
        .into_iter()
        .find(|a| a.id == agent_id)
        .ok_or_else(|| VibeError::AgentNotFound {
            agent_id: agent_id.to_string(),
        })
}

// ===== 原子文件操作 =====

fn do_link(skill_id: &str, agent: &Agent) -> Result<(), VibeError> {
    let vibe_dir = vibe_skills_dir()?;
    let skill_path = vibe_dir.join(skill_id);
    let link_path = Path::new(&agent.skills_dir).join(skill_id);
    if skill_path.exists() {
        vibe_fs::create_symlink(&skill_path, &link_path)?;
    }
    Ok(())
}

fn do_unlink(skill_id: &str, agent: &Agent) -> Result<(), VibeError> {
    let link_path = Path::new(&agent.skills_dir).join(skill_id);
    let _ = vibe_fs::remove_symlink(&link_path);
    Ok(())
}

fn do_delete_skill(skill_id: &str) -> Result<(), VibeError> {
    let skill_path = vibe_skills_dir()?.join(skill_id);
    if skill_path.exists() {
        fs::remove_dir_all(&skill_path)?;
    }
    Ok(())
}

// ===== 顶层分发函数 =====

/// 受影响的 skill 列表：批量/同步操作优先用 `skill_ids`，否则回退到单值 `skill_id`
fn affected_skills(entry: &HistoryEntry) -> Vec<String> {
    if !entry.skill_ids.is_empty() {
        entry.skill_ids.clone()
    } else {
        vec![entry.skill_id.clone()]
    }
}

/// 执行撤销操作（内部使用，被 undo() 和 undo_by_id() 共用）
pub fn perform_undo(entry: &HistoryEntry) -> Result<(), VibeError> {
    match entry.action {
        HistoryAction::Link => {
            let agent = entry.agent_id.as_ref().ok_or_else(|| {
                VibeError::History("Link entry missing agent_id".to_string())
            })?;
            let agent = resolve_agent(agent)?;
            do_unlink(&entry.skill_id, &agent)
        }
        HistoryAction::Unlink => {
            let agent = entry.agent_id.as_ref().ok_or_else(|| {
                VibeError::History("Unlink entry missing agent_id".to_string())
            })?;
            let agent = resolve_agent(agent)?;
            let vibe_dir = vibe_skills_dir()?;
            let skill_path = vibe_dir.join(&entry.skill_id);
            if skill_path.exists() {
                do_link(&entry.skill_id, &agent)
            } else {
                Ok(())
            }
        }
        HistoryAction::Install => do_delete_skill(&entry.skill_id),
        HistoryAction::Delete => {
            use crate::commands::skills::restore_from_trash;
            restore_from_trash(&entry.skill_id)
        }
        HistoryAction::BatchLink => {
            let agent = entry.agent_id.as_ref().ok_or_else(|| {
                VibeError::History("BatchLink entry missing agent_id".to_string())
            })?;
            let agent = resolve_agent(agent)?;
            for skill_id in affected_skills(entry) {
                do_unlink(&skill_id, &agent)?;
            }
            Ok(())
        }
        HistoryAction::BatchUnlink => {
            let agent = entry.agent_id.as_ref().ok_or_else(|| {
                VibeError::History("BatchUnlink entry missing agent_id".to_string())
            })?;
            let agent = resolve_agent(agent)?;
            let vibe_dir = vibe_skills_dir()?;
            for skill_id in affected_skills(entry) {
                let skill_path = vibe_dir.join(&skill_id);
                if skill_path.exists() {
                    do_link(&skill_id, &agent)?;
                }
            }
            Ok(())
        }
    }
}

/// 执行重做操作（内部使用，被 redo() 和 redo_by_id() 共用）
pub fn perform_redo(entry: &HistoryEntry) -> Result<(), VibeError> {
    match entry.action {
        HistoryAction::Link => {
            let agent = entry.agent_id.as_ref().ok_or_else(|| {
                VibeError::History("Link entry missing agent_id".to_string())
            })?;
            let agent = resolve_agent(agent)?;
            let vibe_dir = vibe_skills_dir()?;
            let skill_path = vibe_dir.join(&entry.skill_id);
            if skill_path.exists() {
                do_link(&entry.skill_id, &agent)
            } else {
                Ok(())
            }
        }
        HistoryAction::Unlink => {
            let agent = entry.agent_id.as_ref().ok_or_else(|| {
                VibeError::History("Unlink entry missing agent_id".to_string())
            })?;
            let agent = resolve_agent(agent)?;
            do_unlink(&entry.skill_id, &agent)
        }
        HistoryAction::Install => Err(VibeError::History(
            "Cannot redo install operation".to_string(),
        )),
        HistoryAction::Delete => {
            use crate::commands::skills::move_to_trash;
            move_to_trash(&entry.skill_id)
        }
        HistoryAction::BatchLink => {
            let agent = entry.agent_id.as_ref().ok_or_else(|| {
                VibeError::History("BatchLink entry missing agent_id".to_string())
            })?;
            let agent = resolve_agent(agent)?;
            let vibe_dir = vibe_skills_dir()?;
            for skill_id in affected_skills(entry) {
                let skill_path = vibe_dir.join(&skill_id);
                if skill_path.exists() {
                    do_link(&skill_id, &agent)?;
                }
            }
            Ok(())
        }
        HistoryAction::BatchUnlink => {
            let agent = entry.agent_id.as_ref().ok_or_else(|| {
                VibeError::History("BatchUnlink entry missing agent_id".to_string())
            })?;
            let agent = resolve_agent(agent)?;
            for skill_id in affected_skills(entry) {
                do_unlink(&skill_id, &agent)?;
            }
            Ok(())
        }
    }
}
