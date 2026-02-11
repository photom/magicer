#[derive(Debug, PartialEq, Eq)]
pub enum ValidationError {
    InvalidCharacter,
    ExceedsMaxLength,
    EmptyValue,
    AbsolutePath,
    PathTraversal,
    InvalidPath,
}

#[derive(Debug, PartialEq, Eq)]
pub enum MagicError {
    AnalysisFailed(String),
    DatabaseLoadFailed(String),
    FileNotFound(String),
}

#[derive(Debug, PartialEq, Eq)]
pub enum AuthenticationError {
    InvalidCredentials,
    InternalError(String),
}
