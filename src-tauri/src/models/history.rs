use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: String,
    /// ISO 8601 格式时间戳
    pub timestamp: String,
    pub action: HistoryAction,
    pub skill_id: String,
    pub agent_id: Option<String>,
    /// "symlink" | "junction" | "copy"
    pub mode: Option<String>,
    pub undone: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum HistoryAction {
    Link,
    Unlink,
    Install,
    Delete,
    BatchLink,
    BatchUnlink,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryStore {
    pub version: u32,
    pub entries: Vec<HistoryEntry>,
}

impl Default for HistoryStore {
    fn default() -> Self {
        Self {
            version: 1,
            entries: Vec::new(),
        }
    }
}
