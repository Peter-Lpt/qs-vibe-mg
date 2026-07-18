use serde::{Deserialize, Serialize};

use crate::models::origin::SkillOrigin;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    /// 文件夹名，作为唯一标识
    pub id: String,
    /// SKILL.md frontmatter 中的 name（可能为空）
    pub name: String,
    /// SKILL.md frontmatter 中的 description
    pub description: String,
    /// ~/.vibe-skills/{id} 绝对路径
    pub path: String,
    /// 已关联的 agent id 列表
    pub linked_agents: Vec<String>,
    /// skill 来源：vibe-lib（中心库）或各 agent id
    pub sources: Vec<SkillSource>,
    /// license
    pub license: Option<String>,
    /// compatibility
    pub compatibility: Option<String>,
    /// metadata key-value pairs
    pub metadata: Option<std::collections::HashMap<String, String>>,
    /// 是否包含 scripts/ 目录
    pub has_scripts: bool,
    /// 是否包含 references/ 目录
    pub has_references: bool,
    /// 是否包含 assets/ 目录
    pub has_assets: bool,
    /// 最后修改时间
    pub modified_at: String,
    /// 是否存在同名冲突（多个 source 的 content_hash 不同）
    pub has_conflict: bool,
    /// 是否存在断链（symlink 目标不存在）
    pub has_dangling: bool,
    /// 是否为重复条目（同文件夹名但 SKILL.md name 不同）
    pub is_duplicate: bool,
    /// 是否缺少 name 字段（frontmatter 中没有 name 或 name 为空）
    pub missing_name: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillSource {
    /// 来源标识：vibe-lib 或 agent id（如 claude-code）
    pub from: String,
    /// 来源类型：library / agent / project / external
    #[serde(default = "default_source_kind")]
    pub source_kind: String,
    /// 该来源下的绝对路径
    pub path: String,
    /// 该来源下 SKILL.md 中的 name
    pub name: String,
    /// 该来源下 SKILL.md 中的 description
    pub description: String,
    /// 是否为 symlink 或 junction
    pub is_symlink: bool,
    /// symlink 目标路径（如有）
    pub symlink_target: Option<String>,
    /// SKILL.md 内容的 SHA-256 hex hash
    pub content_hash: String,
    /// 该来源目录最后修改时间
    #[serde(default)]
    pub modified_at: String,
    /// 该来源的安装/导入来源记录
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin: Option<SkillOrigin>,
    /// 来源可信度：explicit / inferred / unknown
    #[serde(default = "default_trust_level")]
    pub trust_level: String,
    /// 更新状态：auto_update / best_effort / unknown
    #[serde(default = "default_update_status")]
    pub update_status: String,
}

fn default_source_kind() -> String {
    "agent".to_string()
}

fn default_trust_level() -> String {
    "unknown".to_string()
}

fn default_update_status() -> String {
    "unknown".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictType {
    /// 同名但 SKILL.md 内容不同
    SameNameDiffContent,
    /// symlink 指向已删除路径
    DanglingLink,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillIssue {
    pub skill_id: String,
    pub issue_type: ConflictType,
    pub description: String,
}
