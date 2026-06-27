use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    /// 文件夹名，作为唯一标识
    pub id: String,
    /// SKILL.md frontmatter 中的 name
    pub name: String,
    /// SKILL.md frontmatter 中的 description
    pub description: String,
    /// ~/.vab-skills/{id} 绝对路径
    pub path: String,
    /// 已关联的 agent id 列表
    pub linked_agents: Vec<String>,
    /// skill 来源：vab-lib（中心库）或各 agent id
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillSource {
    /// 来源标识：vab-lib 或 agent id（如 claude-code）
    pub from: String,
    /// 该来源下的绝对路径
    pub path: String,
}
