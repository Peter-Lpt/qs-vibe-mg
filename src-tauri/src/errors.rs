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

    #[error("Link not found: {skill_id} -> {agent_id}")]
    LinkNotFound { skill_id: String, agent_id: String },

    #[error("Config error: {0}")]
    Config(String),

    #[error("Path error: {0}")]
    Path(String),

    #[error("History error: {0}")]
    History(String),

    #[error("No operation to undo")]
    NothingToUndo,

    #[error("No operation to redo")]
    NothingToRedo,

    #[error("Can only undo the most recent undoable entry")]
    UndoNotLatest,

    #[error("Can only redo the most recent redoable entry")]
    RedoNotLatest,

    #[error("History entry not found: {id}")]
    HistoryEntryNotFound { id: String },

    #[error("History entry is already undone: {id}")]
    AlreadyUndone { id: String },

    #[error("History entry is not undone: {id}")]
    NotUndone { id: String },

    #[error("Skill already exists: {skill_id}")]
    SkillAlreadyExists { skill_id: String },

    #[error("Conflict: {skill_id} - {details}")]
    Conflict {
        skill_id: String,
        details: String,
    },
}

impl serde::Serialize for VabError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
