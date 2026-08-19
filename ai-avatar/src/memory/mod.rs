//! Memory trait and message types for conversation persistence.

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Role of a message in the conversation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role {
    System,
    User,
    Assistant,
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::System => write!(f, "system"),
            Role::User => write!(f, "user"),
            Role::Assistant => write!(f, "assistant"),
        }
    }
}

impl std::str::FromStr for Role {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "system" => Ok(Role::System),
            "user" => Ok(Role::User),
            "assistant" => Ok(Role::Assistant),
            _ => anyhow::bail!("unknown role: {}", s),
        }
    }
}

/// A single message in the conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
    pub timestamp: i64,
}

impl Message {
    pub fn new(role: Role, content: impl Into<String>) -> Self {
        Self {
            role,
            content: content.into(),
            timestamp: now_timestamp(),
        }
    }
}

pub mod hybrid;
pub mod sqlite;

pub use hybrid::HybridMemory;

/// Core memory operations.
pub trait Memory: Send {
    /// Persist a message.
    fn save_message(&mut self, message: Message) -> Result<()>;

    /// Retrieve the most recent `limit` messages.
    fn get_history(&self, limit: usize) -> Result<Vec<Message>>;

    /// Retrieve messages with a time offset (e.g. last hour).
    fn get_history_since(&self, since: i64) -> Result<Vec<Message>>;

    /// Clear all history.
    fn clear_history(&mut self) -> Result<()>;

    /// Compact old messages into summaries.
    fn compact(&mut self) -> Result<()>;
}

/// Naive token estimator: chars / 4.
pub fn estimate_tokens(text: &str) -> usize {
    text.chars().count() / 4 + 1
}

pub fn now_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}
