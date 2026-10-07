use crate::config::ConfigError;
use crate::model::ops::OpError;
use crate::store::format::ParseError;

/// One place that maps every failure to an exit code (spec 4.3, 11.1).
#[derive(Debug, thiserror::Error)]
pub enum TrkError {
    #[error("{0}")]
    Op(#[from] OpError),
    #[error("{0}")]
    Config(#[from] ConfigError),
    #[error("{0}")]
    Parse(#[from] ParseError),
    #[error("store is locked by another trk process")]
    StoreLocked,
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Message(String),
    #[error("{0}")]
    Usage(String),
    #[error("cancelled")]
    Cancelled,
}

impl TrkError {
    pub fn exit_code(&self) -> i32 {
        match self {
            TrkError::Op(OpError::Blocked { .. }) | TrkError::Op(OpError::GoalBlocked { .. }) => 3,
            TrkError::Op(OpError::NoActiveGoal) | TrkError::Op(OpError::NoCurrentTask) => 4,
            TrkError::Usage(_) => 2,
            TrkError::Cancelled => 5,
            _ => 1,
        }
    }

    /// `true` when the error is a blocked-command notice, which is rendered as
    /// an orange warning rather than a red `trk:` error.
    pub fn is_blocked(&self) -> bool {
        matches!(
            self,
            TrkError::Op(OpError::Blocked { .. }) | TrkError::Op(OpError::GoalBlocked { .. })
        )
    }

    /// Message for a blocked command (spec 8.6, 8.15).
    pub fn blocked_warning(&self) -> Option<String> {
        match self {
            TrkError::Op(OpError::Blocked { text, count }) => Some(format!(
                "\"{text}\" has {count} open sub-tasks; finish or drop them first (or use --force)"
            )),
            TrkError::Op(OpError::GoalBlocked { title, count }) => Some(format!(
                "goal \"{title}\" has {count} open tasks; finish or drop them first (or use --force)"
            )),
            _ => None,
        }
    }
}
