use crate::errors::VibeError;
use crate::models::history::HistoryEntry;
use crate::utils::history::{
    clear_history as utils_clear_history, last_active_for_undo, last_undone_for_redo,
    load_history, mark_undone, perform_redo, perform_undo,
};

/// 获取操作历史
#[tauri::command]
pub async fn get_history() -> Result<Vec<HistoryEntry>, VibeError> {
    tauri::async_runtime::spawn_blocking(get_history_sync)
        .await
        .map_err(|error| VibeError::Path(format!("get_history task failed: {}", error)))?
}

fn get_history_sync() -> Result<Vec<HistoryEntry>, VibeError> {
    let store = load_history()?;
    Ok(store.entries)
}

/// 清空所有历史记录
#[tauri::command]
pub async fn clear_history() -> Result<(), VibeError> {
    tauri::async_runtime::spawn_blocking(utils_clear_history)
        .await
        .map_err(|error| VibeError::Path(format!("clear_history task failed: {}", error)))?
}

/// 撤销最后一个操作
#[tauri::command]
pub async fn undo() -> Result<HistoryEntry, VibeError> {
    tauri::async_runtime::spawn_blocking(undo_sync)
        .await
        .map_err(|error| VibeError::Path(format!("undo task failed: {}", error)))?
}

fn undo_sync() -> Result<HistoryEntry, VibeError> {
    let entry = last_active_for_undo()?.ok_or(VibeError::NothingToUndo)?;

    perform_undo(&entry)?;

    mark_undone(&entry.id, true)?;
    let mut undone = entry;
    undone.undone = true;
    Ok(undone)
}

/// 重做最后一个已撤销的操作
#[tauri::command]
pub async fn redo() -> Result<HistoryEntry, VibeError> {
    tauri::async_runtime::spawn_blocking(redo_sync)
        .await
        .map_err(|error| VibeError::Path(format!("redo task failed: {}", error)))?
}

fn redo_sync() -> Result<HistoryEntry, VibeError> {
    let entry = last_undone_for_redo()?.ok_or(VibeError::NothingToRedo)?;

    perform_redo(&entry)?;

    mark_undone(&entry.id, false)?;
    let mut redone = entry;
    redone.undone = false;
    Ok(redone)
}

/// 按ID撤销（堆栈模式：只允许操作最新的未撤销记录）
#[tauri::command]
pub async fn undo_by_id(id: String) -> Result<HistoryEntry, VibeError> {
    tauri::async_runtime::spawn_blocking(move || undo_by_id_sync(id))
        .await
        .map_err(|error| VibeError::Path(format!("undo_by_id task failed: {}", error)))?
}

fn undo_by_id_sync(id: String) -> Result<HistoryEntry, VibeError> {
    let store = load_history()?;
    let entry = store
        .entries
        .iter()
        .find(|e| e.id == id)
        .ok_or_else(|| VibeError::HistoryEntryNotFound {
            id: id.clone(),
        })?;

    if entry.undone {
        return Err(VibeError::AlreadyUndone { id });
    }

    // 堆栈模式验证：必须是最后一个未撤销的记录
    let last_undoable = store.entries.iter().rev().find(|e| !e.undone);
    match last_undoable {
        Some(last) if last.id == id => {}
        _ => return Err(VibeError::UndoNotLatest),
    }

    perform_undo(entry)?;

    mark_undone(&entry.id, true)?;
    let mut undone = entry.clone();
    undone.undone = true;
    Ok(undone)
}

/// 按ID重做（堆栈模式：只允许操作最新的已撤销记录）
#[tauri::command]
pub async fn redo_by_id(id: String) -> Result<HistoryEntry, VibeError> {
    tauri::async_runtime::spawn_blocking(move || redo_by_id_sync(id))
        .await
        .map_err(|error| VibeError::Path(format!("redo_by_id task failed: {}", error)))?
}

fn redo_by_id_sync(id: String) -> Result<HistoryEntry, VibeError> {
    let store = load_history()?;
    let entry = store
        .entries
        .iter()
        .find(|e| e.id == id)
        .ok_or_else(|| VibeError::HistoryEntryNotFound {
            id: id.clone(),
        })?;

    if !entry.undone {
        return Err(VibeError::NotUndone { id });
    }

    // 堆栈模式验证：必须是最后一个已撤销的记录
    let last_redoable = store.entries.iter().rev().find(|e| e.undone);
    match last_redoable {
        Some(last) if last.id == id => {}
        _ => return Err(VibeError::RedoNotLatest),
    }

    perform_redo(entry)?;

    mark_undone(&entry.id, false)?;
    let mut redone = entry.clone();
    redone.undone = false;
    Ok(redone)
}