mod commands;
mod errors;
mod models;
mod parsers;
mod utils;

use tracing_subscriber::{fmt, EnvFilter};

fn init_logger() {
    let log_dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("vibe-skills-manager")
        .join("logs");

    std::fs::create_dir_all(&log_dir).ok();

    let file_appender = tracing_appender::rolling::daily(&log_dir, "app.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    // 保持 _guard 在整个程序运行期间存活
    std::mem::forget(_guard);

    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_writer(non_blocking)
        .with_ansi(false)
        .init();

    tracing::info!("Logger initialized, log dir: {:?}", log_dir);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    init_logger();
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::skills::list_skills,
            commands::skills::preview_skill,
            commands::skills::preview_skill_at_path,
            commands::skills::install_skill,
            commands::skills::delete_skill,
            commands::skills::search_skills,
            commands::skills::detect_issues,
            commands::skills::get_dashboard_data,
            commands::sync::create_link,
            commands::sync::remove_link,
            commands::sync::batch_link,
            commands::sync::batch_unlink,
            commands::sync::sync_agent_to_vibe,
            commands::sync::sync_category_to_vibe,
            commands::sync::remove_sync,
            commands::sync::remove_sync_skills,
            commands::sync::sync_to_vibe,
            commands::sync::relink,
            commands::sync::batch_skill_action,
            commands::agents::list_agents,
            commands::agents::add_custom_agent,
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
            commands::config::update_config,
            commands::config::set_vibe_skills_path,
            commands::config::export_data,
            commands::config::import_data,
            commands::config::write_file_to_path,
            commands::config::read_file_from_path,
            commands::logger::log_message,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
