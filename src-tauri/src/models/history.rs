use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: String,
    /// ISO 8601 格式时间戳
    pub timestamp: String,
    pub action: HistoryAction,
    pub skill_id: String,
    /// 受影响 skill 列表（批量/同步操作使用）；单条操作为空，回退到 `skill_id`
    #[serde(default)]
    pub skill_ids: Vec<String>,
    pub agent_id: Option<String>,
    /// 精确操作路径。嵌套 skill 或断链清理时用于撤销/重做定位原链接位置。
    #[serde(default)]
    pub source_path: Option<String>,
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
