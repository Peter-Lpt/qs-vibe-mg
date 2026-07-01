use crate::errors::VabError;
use crate::models::history::HistoryEntry;
use crate::utils::history::{
    clear_history as utils_clear_history, last_undone_entry, last_undone_entry_for_redo,
    load_history, mark_undone, perform_redo, perform_undo,
};

/// 获取操作历史
#[tauri::command]
pub fn get_history() -> Result<Vec<HistoryEntry>, VabError> {
    let store = load_history()?;
    Ok(store.entries)
}

/// 清空所有历史记录
#[tauri::command]
pub fn clear_history() -> Result<(), VabError> {
    utils_clear_history()
}

/// 撤销最后一个操作
#[tauri::command]
pub fn undo() -> Result<HistoryEntry, VabError> {
    let entry = last_undone_entry()?.ok_or(VabError::NothingToUndo)?;

    perform_undo(&entry)?;

    mark_undone(&entry.id, true)?;
    let mut undone = entry;
    undone.undone = true;
    Ok(undone)
}

/// 重做最后一个已撤销的操作
#[tauri::command]
pub fn redo() -> Result<HistoryEntry, VabError> {
    let entry = last_undone_entry_for_redo()?.ok_or(VabError::NothingToRedo)?;

    perform_redo(&entry)?;

    mark_undone(&entry.id, false)?;
    let mut redone = entry;
    redone.undone = false;
    Ok(redone)
}

/// 按ID撤销（堆栈模式：只允许操作最新的未撤销记录）
#[tauri::command]
pub fn undo_by_id(id: String) -> Result<HistoryEntry, VabError> {
    let store = load_history()?;
    let entry = store
        .entries
        .iter()
        .find(|e| e.id == id)
        .ok_or_else(|| VabError::HistoryEntryNotFound {
            id: id.clone(),
        })?;

    if entry.undone {
        return Err(VabError::AlreadyUndone { id });
    }

    // 堆栈模式验证：必须是最后一个未撤销的记录
    let last_undoable = store.entries.iter().rev().find(|e| !e.undone);
    match last_undoable {
        Some(last) if last.id == id => {}
        _ => return Err(VabError::UndoNotLatest),
    }

    perform_undo(entry)?;

    mark_undone(&entry.id, true)?;
    let mut undone = entry.clone();
    undone.undone = true;
    Ok(undone)
}

/// 按ID重做（堆栈模式：只允许操作最新的已撤销记录）
#[tauri::command]
pub fn redo_by_id(id: String) -> Result<HistoryEntry, VabError> {
    let store = load_history()?;
    let entry = store
        .entries
        .iter()
        .find(|e| e.id == id)
        .ok_or_else(|| VabError::HistoryEntryNotFound {
            id: id.clone(),
        })?;

    if !entry.undone {
        return Err(VabError::NotUndone { id });
    }

    // 堆栈模式验证：必须是最后一个已撤销的记录
    let last_redoable = store.entries.iter().rev().find(|e| e.undone);
    match last_redoable {
        Some(last) if last.id == id => {}
        _ => return Err(VabError::RedoNotLatest),
    }

    perform_redo(entry)?;

    mark_undone(&entry.id, false)?;
    let mut redone = entry.clone();
    redone.undone = false;
    Ok(redone)
}