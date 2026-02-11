use std::fmt;
use std::io;

#[derive(Debug)]
pub enum InfrastructureError {
    Io(io::Error),
    InvalidConfig(String),
    MaxRetriesExceeded(String),
}

impl fmt::Display for InfrastructureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => write!(f, "I/O error: {}", err),
            Self::InvalidConfig(msg) => write!(f, "Invalid configuration: {}", msg),
            Self::MaxRetriesExceeded(msg) => write!(f, "Max retries exceeded: {}", msg),
        }
    }
}

impl From<io::Error> for InfrastructureError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

impl std::error::Error for InfrastructureError {}
