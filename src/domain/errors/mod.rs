use std::fmt;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ValidationError {
    InvalidCharacter,
    ExceedsMaxLength,
    EmptyValue,
    AbsolutePath,
    PathTraversal,
    InvalidPath,
    FileNotFound,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidCharacter => write!(f, "Invalid character"),
            Self::ExceedsMaxLength => write!(f, "Exceeds maximum length"),
            Self::EmptyValue => write!(f, "Empty value"),
            Self::AbsolutePath => write!(f, "Absolute path not allowed"),
            Self::PathTraversal => write!(f, "Path traversal not allowed"),
            Self::InvalidPath => write!(f, "Invalid path"),
            Self::FileNotFound => write!(f, "File or directory not found"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum MagicError {
    AnalysisFailed(String),
    DatabaseLoadFailed(String),
    FileNotFound(String),
}

impl fmt::Display for MagicError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AnalysisFailed(msg) => write!(f, "Analysis failed: {}", msg),
            Self::DatabaseLoadFailed(msg) => write!(f, "Database load failed: {}", msg),
            Self::FileNotFound(path) => write!(f, "File not found: {}", path),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum AuthenticationError {
    InvalidCredentials,
    InternalError(String),
}

impl fmt::Display for AuthenticationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidCredentials => write!(f, "Invalid credentials"),
            Self::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum DomainError {
    ValidationError(ValidationError),
    MagicError(MagicError),
    FileNotFound(String),
    PermissionDenied(String),
    ConfigurationError(String),
}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ValidationError(e) => write!(f, "Validation error: {}", e),
            Self::MagicError(e) => write!(f, "Magic error: {}", e),
            Self::FileNotFound(path) => write!(f, "File not found: {}", path),
            Self::PermissionDenied(path) => write!(f, "Permission denied: {}", path),
            Self::ConfigurationError(msg) => write!(f, "Configuration error: {}", msg),
        }
    }
}

impl From<ValidationError> for DomainError {
    fn from(err: ValidationError) -> Self {
        Self::ValidationError(err)
    }
}

impl From<MagicError> for DomainError {
    fn from(err: MagicError) -> Self {
        Self::MagicError(err)
    }
}
