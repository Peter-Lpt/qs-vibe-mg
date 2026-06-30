mod commands;
mod errors;
mod models;
mod parsers;
mod utils;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::skills::list_skills,
            commands::skills::preview_skill,
            commands::skills::install_skill,
            commands::skills::delete_skill,
            commands::skills::search_skills,
            commands::skills::get_dashboard_data,
            commands::sync::create_link,
            commands::sync::remove_link,
            commands::sync::check_link_status,
            commands::sync::batch_link,
            commands::sync::batch_unlink,
            commands::sync::sync_agent_to_vibe,
            commands::sync::sync_category_to_vibe,
            commands::sync::remove_sync,
            commands::agents::list_agents,
            commands::agents::add_custom_agent,
            commands::agents::update_agent,
            commands::agents::remove_custom_agent,
            commands::agents::get_skills_tree,
            commands::history::get_history,
            commands::history::undo,
            commands::history::redo,
            commands::config::get_config,
            commands::config::update_config,
            commands::config::set_vibe_skills_path,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
