use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardData {
    pub agents: Vec<DashboardAgent>,
    pub shared_skills: Vec<SharedSkillInfo>,
    pub stats: DashboardStats,
    /// 扫描时因深度上限/链接环被截断（P4）
    #[serde(default)]
    pub truncated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardAgent {
    pub agent_id: String,
    pub agent_name: String,
    pub skill_count: usize,
    pub skills: Vec<DashboardSkill>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardSkill {
    pub skill_id: String,
    pub skill_name: String,
    pub shared_with: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedSkillInfo {
    pub skill_id: String,
    pub skill_name: String,
    pub agent_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardStats {
    pub total_skills: usize,
    pub shared_count: usize,
    pub per_agent_count: HashMap<String, usize>,
}
