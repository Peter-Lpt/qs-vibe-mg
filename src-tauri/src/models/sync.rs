use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub synced_count: usize,
    pub errors: Vec<String>,
    #[serde(default)]
    pub warnings: Vec<String>,
}
