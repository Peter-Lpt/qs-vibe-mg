use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillsTreeNode {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub skill_count: usize,
    pub synced: bool,
    pub synced_count: usize,
    pub children: Vec<SkillsTreeNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub synced_count: usize,
    pub errors: Vec<String>,
}
