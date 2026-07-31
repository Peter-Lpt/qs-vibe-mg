mod commands;
mod errors;
mod models;
mod parsers;
mod utils;

use std::sync::atomic::{AtomicU32, Ordering};

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, Runtime,
};
use tracing_subscriber::{fmt, EnvFilter};

// 全局更新异常计数
static UPDATE_ERROR_COUNT: AtomicU32 = AtomicU32::new(0);

// L6：持有 non_blocking writer 的 guard，避免 std::mem::forget 泄漏；OnceLock 保证只初始化一次
static LOG_GUARD: std::sync::OnceLock<tracing_appender::non_blocking::WorkerGuard> =
    std::sync::OnceLock::new();

fn init_logger() {
    let log_dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("qs-vibe-mg")
        .join("logs");

    std::fs::create_dir_all(&log_dir).ok();

    let file_appender = tracing_appender::rolling::daily(&log_dir, "app.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    let _ = LOG_GUARD.set(guard);

    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_writer(non_blocking)
        .with_ansi(false)
        .init();

    tracing::info!("Logger initialized, log dir: {:?}", log_dir);
}

/// 构建托盘菜单
fn build_tray_menu<R: Runtime>(app: &tauri::AppHandle<R>) -> tauri::Result<Menu<R>> {
    let error_count = UPDATE_ERROR_COUNT.load(Ordering::Relaxed);

    let update_error_i = if error_count > 0 {
        MenuItem::with_id(
            app,
            "update_errors",
            format!("更新异常 ({})", error_count),
            true,
            None::<&str>,
        )?
    } else {
        MenuItem::with_id(app, "update_errors", "更新异常", false, None::<&str>)?
    };

    let show_i = MenuItem::with_id(app, "show", "显示窗口", true, None::<&str>)?;
    let quit_i = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;

    Menu::with_items(app, &[&update_error_i, &show_i, &quit_i])
}

/// 更新托盘菜单（当前端检测到异常数量变化时调用）
#[tauri::command]
fn update_tray_menu<R: Runtime>(app: tauri::AppHandle<R>, error_count: u32) -> Result<(), String> {
    UPDATE_ERROR_COUNT.store(error_count, Ordering::Relaxed);

    if let Some(tray) = app.tray_by_id("main-tray") {
        let menu = build_tray_menu(&app).map_err(|e| e.to_string())?;
        tray.set_menu(Some(menu)).map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    init_logger();
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // 创建系统托盘菜单
            let menu = build_tray_menu(app.handle())?;

            // 创建系统托盘图标
            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("QS VIBE - AI Skills Manager")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_tray_icon_event(|tray, event| {
                    match event {
                        // 左键点击托盘图标：显示/聚焦窗口
                        TrayIconEvent::Click {
                            button: MouseButton::Left,
                            button_state: MouseButtonState::Up,
                            ..
                        } => {
                            let app = tray.app_handle();
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.unminimize();
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        _ => {}
                    }
                })
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.unminimize();
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "update_errors" => {
                        // 显示窗口并通知前端筛选异常
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.unminimize();
                            let _ = window.show();
                            let _ = window.set_focus();
                            // 发送事件到前端，触发异常筛选
                            let _ = window.emit("tray-filter-update-errors", ());
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .build(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::skills::list_skills,
            commands::skills::list_plugin_skills,
            commands::skills::adopt_plugin_skill,
            commands::skills::update_plugin_skills_from_marketplace,
            commands::skills::check_skill_update,
            commands::skills::check_all_skill_updates,
            commands::skills::preview_skill,
            commands::skills::preview_skill_at_path,
            commands::skills::install_skill,
            commands::skills::install_skill_from_source,
            commands::skills::update_skill,
            commands::skills::delete_library_skill,
            commands::skills::detect_issues,
            commands::skills::get_dashboard_data,
            commands::sync::create_link,
            commands::sync::remove_link,
            commands::sync::detach_keep_local_copy,
            commands::sync::remove_agent_skill_copy,
            commands::sync::batch_link,
            commands::sync::batch_unlink,
            commands::sync::sync_agent_to_vibe,
            commands::sync::sync_category_to_vibe,
            commands::sync::remove_sync,
            commands::sync::remove_sync_skills,
            commands::sync::sync_to_vibe,
            commands::sync::relink,
            commands::sync::replace_with_library,
            commands::sync::batch_skill_action,
            commands::agents::list_agents,
            commands::agents::add_custom_agent,
            commands::agents::add_custom_agent_with_options,
            commands::agents::update_agent,
            commands::agents::remove_custom_agent,
            commands::agents::get_skills_tree,
            commands::history::get_history,
            commands::history::undo,
            commands::history::redo,
            commands::history::undo_by_id,
            commands::history::redo_by_id,
            commands::history::clear_history,
            commands::config::get_config,
            commands::config::suggest_project_roots,
            commands::config::update_config,
            commands::config::set_vibe_skills_path,
            commands::config::export_data,
            commands::config::import_data,
            commands::config::write_file_to_path,
            commands::config::read_file_from_path,
            commands::logger::log_message,
            update_tray_menu,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
