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
            commands::skills::list_skills,
            commands::agents::list_agents,
            commands::sync::create_link,
            commands::sync::remove_link,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
