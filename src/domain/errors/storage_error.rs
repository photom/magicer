use std::fmt;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum StorageError {
    InsufficientSpace {
        available_mb: u64,
        required_mb: u64,
        path: String,
    },
    IoError(String),
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InsufficientSpace {
                available_mb,
                required_mb,
                path,
            } => write!(
                f,
                "Insufficient storage space at {}: {}MB available, but {}MB required",
                path, available_mb, required_mb
            ),
            Self::IoError(msg) => write!(f, "Storage I/O error: {}", msg),
        }
    }
}
