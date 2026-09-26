//! Game log entries with audience tagging.
//!
//! This replaces the old `network::broadcast` / `network::broadcast_others`.
//! Instead of pushing a message out over the wire in the middle of game logic,
//! the engine records a [`LogEntry`] with an [`Audience`] tag. The host (TUI /
//! future network backend) decides how to route it.

/// The audience a log entry is meant for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Audience {
    /// The entry is meant for everyone.
    Everyone,
    /// The entry is meant for everyone *except* the player with this id.
    ///
    /// This is the old `broadcast_others` behaviour.
    Not(u32),
}

/// A single entry in the game log.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LogEntry {
    /// The round number at which the entry was created.
    pub round: u32,
    /// Who the entry is meant for.
    pub audience: Audience,
    /// The (already formatted, uncolored) text of the entry.
    pub text: String,
}

impl LogEntry {
    /// Creates a new log entry for everyone.
    pub fn everyone(round: u32, text: impl Into<String>) -> Self {
        Self {
            round,
            audience: Audience::Everyone,
            text: text.into(),
        }
    }

    /// Creates a new log entry for everyone except `player_id`.
    pub fn others(round: u32, player_id: u32, text: impl Into<String>) -> Self {
        Self {
            round,
            audience: Audience::Not(player_id),
            text: text.into(),
        }
    }

    /// Returns `true` if this entry should be shown to the player with `viewer_id`.
    pub fn visible_to(&self, viewer_id: u32) -> bool {
        match self.audience {
            Audience::Everyone => true,
            Audience::Not(id) => id != viewer_id,
        }
    }
}
