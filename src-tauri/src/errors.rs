use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum VabError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Skill not found: {skill_id}")]
    SkillNotFound { skill_id: String },

    #[error("Agent not found: {agent_id}")]
    AgentNotFound { agent_id: String },

    #[error("Invalid SKILL.md: {reason}")]
    InvalidSkillMd { reason: String },

    #[error("Link already exists: {skill_id} -> {agent_id}")]
    LinkAlreadyExists { skill_id: String, agent_id: String },

    #[error("Config error: {0}")]
    Config(String),

    #[error("Path error: {0}")]
    Path(String),
}

impl serde::Serialize for VabError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
