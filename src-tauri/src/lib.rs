mod commands;
mod errors;
mod models;
mod parsers;
mod utils;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            // Skills
            commands::skills::list_skills,
            commands::skills::preview_skill,
            commands::skills::install_skill,
            commands::skills::delete_skill,
            // Sync / Links
            commands::sync::create_link,
            commands::sync::remove_link,
            commands::sync::check_link_status,
            commands::sync::batch_link,
            commands::sync::batch_unlink,
            // Agents
            commands::agents::list_agents,
            commands::agents::add_custom_agent,
            commands::agents::remove_custom_agent,
            // History
            commands::history::get_history,
            commands::history::undo,
            commands::history::redo,
            // Config
            commands::config::get_config,
            commands::config::update_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
