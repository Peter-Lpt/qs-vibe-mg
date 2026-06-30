use std::fs;

use uuid::Uuid;

use crate::errors::VabError;
use crate::models::history::{HistoryAction, HistoryEntry, HistoryStore};
use crate::utils::config::load_config;
use crate::utils::path::vibe_skills_dir;

const HISTORY_FILE: &str = ".vibe-history.json";

fn history_path() -> Result<std::path::PathBuf, VabError> {
    Ok(vibe_skills_dir()?.join(HISTORY_FILE))
}

/// 加载历史记录
pub fn load_history() -> Result<HistoryStore, VabError> {
    let path = history_path()?;
    if !path.exists() {
        return Ok(HistoryStore::default());
    }

    let content = fs::read_to_string(&path)?;
    let store: HistoryStore =
        serde_json::from_str(&content).map_err(|e| VabError::History(e.to_string()))?;
    Ok(store)
}

/// 保存历史记录
pub fn save_history(store: &HistoryStore) -> Result<(), VabError> {
    let path = history_path()?;
    let dir = vibe_skills_dir()?;
    if !dir.exists() {
        fs::create_dir_all(&dir)?;
    }

    let content =
        serde_json::to_string_pretty(store).map_err(|e| VabError::History(e.to_string()))?;
    fs::write(&path, content)?;
    Ok(())
}

/// 记录一条操作历史
pub fn record_action(
    action: HistoryAction,
    skill_id: &str,
    agent_id: Option<&str>,
    mode: Option<&str>,
) -> Result<HistoryEntry, VabError> {
    let mut store = load_history()?;

    // 超出限制时删除最旧的
    let config = load_config()?;
    let max = config.history.max_entries as usize;
    while store.entries.len() >= max {
        store.entries.remove(0);
    }

    let entry = HistoryEntry {
        id: Uuid::new_v4().to_string(),
        timestamp: chrono_now(),
        action,
        skill_id: skill_id.to_string(),
        agent_id: agent_id.map(|s| s.to_string()),
        mode: mode.map(|s| s.to_string()),
        undone: false,
    };

    store.entries.push(entry.clone());
    save_history(&store)?;
    Ok(entry)
}

/// 获取最后一条未撤销的记录
pub fn last_undone_entry() -> Result<Option<HistoryEntry>, VabError> {
    let store = load_history()?;
    Ok(store.entries.iter().rev().find(|e| !e.undone).cloned())
}

/// 获取最后一条已撤销的记录
pub fn last_undone_entry_for_redo() -> Result<Option<HistoryEntry>, VabError> {
    let store = load_history()?;
    Ok(store.entries.iter().rev().find(|e| e.undone).cloned())
}

/// 标记记录为已撤销
pub fn mark_undone(id: &str, undone: bool) -> Result<(), VabError> {
    let mut store = load_history()?;
    if let Some(entry) = store.entries.iter_mut().find(|e| e.id == id) {
        entry.undone = undone;
    }
    save_history(&store)
}

/// 获取当前时间的 ISO 8601 格式字符串
fn chrono_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();
    // Simple ISO 8601 formatting
    let days = secs / 86400;
    let time_of_day = secs % 86400;
    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let seconds = time_of_day % 60;

    // Days since epoch to Y-M-D (simplified, good enough for display)
    let (year, month, day) = days_to_ymd(days);
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month, day, hours, minutes, seconds
    )
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
