use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    /// 唯一标识，如 claude-code
    pub id: String,
    /// 显示名，如 Claude Code
    pub name: String,
    /// skills 目录路径（绝对路径）
    pub skills_dir: String,
    #[serde(default = "default_kind")]
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detect_dir: Option<String>,
    #[serde(default)]
    pub additional_scan_dirs: Vec<String>,
    #[serde(default)]
    pub tool_detected: bool,
    /// 是否检测到已安装
    pub detected: bool,
    /// 是否启用
    pub enabled: bool,
    /// 是否为自动检测（非用户自定义）
    pub auto_detected: bool,
    /// 已关联的 skill id 列表
    pub linked_skills: Vec<String>,
}

fn default_kind() -> String {
    "agent".to_string()
}
