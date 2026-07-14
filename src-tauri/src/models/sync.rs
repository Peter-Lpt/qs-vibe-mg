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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link_target: Option<String>,
    pub is_source_link: bool,
    /// 子树因超出最大深度或遇到链接环而被截断（P4 环路/深度保护）
    #[serde(default)]
    pub truncated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub synced_count: usize,
    pub errors: Vec<String>,
}
