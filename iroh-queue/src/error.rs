use thiserror::Error;

#[derive(Debug, Error)]
pub enum QueueError {
    #[error("job not found: {0}")]
    JobNotFound(uuid::Uuid),

    #[error("job already claimed: {0}")]
    AlreadyClaimed(uuid::Uuid),

    #[error("serialization error: {0}")]
    Serialization(#[from] postcard::Error),

    #[error("connection error: {0}")]
    Connection(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("gossip error: {0}")]
    Gossip(String),

    #[error("timeout")]
    Timeout,

    #[error("shutdown")]
    Shutdown,
}

pub type Result<T> = std::result::Result<T, QueueError>;
