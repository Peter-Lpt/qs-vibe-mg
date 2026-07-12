use crate::errors::VibeError;

#[tauri::command]
pub fn log_message(level: String, message: String) -> Result<(), VibeError> {
    match level.as_str() {
        "error" => tracing::error!(target: "frontend", "{}", message),
        "warn" => tracing::warn!(target: "frontend", "{}", message),
        "info" => tracing::info!(target: "frontend", "{}", message),
        "debug" => tracing::debug!(target: "frontend", "{}", message),
        _ => tracing::info!(target: "frontend", "{}", message),
    }
    Ok(())
}
