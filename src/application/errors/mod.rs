use crate::domain::errors::DomainError;
use std::fmt;

#[derive(Debug, Clone)]
pub enum ApplicationError {
    BadRequest(String),
    Unauthorized(String),
    Forbidden(String),
    NotFound(String),
    UnprocessableEntity(String),
    InternalError(String),
    Timeout,
}

impl ApplicationError {
    pub fn status_code(&self) -> axum::http::StatusCode {
        match self {
            Self::BadRequest(_) => axum::http::StatusCode::BAD_REQUEST,
            Self::Unauthorized(_) => axum::http::StatusCode::UNAUTHORIZED,
            Self::Forbidden(_) => axum::http::StatusCode::FORBIDDEN,
            Self::NotFound(_) => axum::http::StatusCode::NOT_FOUND,
            Self::UnprocessableEntity(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Self::InternalError(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Self::Timeout => axum::http::StatusCode::GATEWAY_TIMEOUT,
        }
    }
}

impl fmt::Display for ApplicationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadRequest(msg) => write!(f, "Bad Request: {}", msg),
            Self::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            Self::Forbidden(msg) => write!(f, "Forbidden: {}", msg),
            Self::NotFound(msg) => write!(f, "Not Found: {}", msg),
            Self::UnprocessableEntity(msg) => write!(f, "Unprocessable Entity: {}", msg),
            Self::InternalError(msg) => write!(f, "Internal Error: {}", msg),
            Self::Timeout => write!(f, "Timeout"),
        }
    }
}

impl From<DomainError> for ApplicationError {
    fn from(err: DomainError) -> Self {
        match err {
            DomainError::ValidationError(e) => Self::BadRequest(e.to_string()),
            DomainError::MagicError(e) => match e {
                crate::domain::errors::MagicError::FileNotFound(path) => {
                    Self::NotFound(format!("File not found: {}", path))
                }
                _ => Self::UnprocessableEntity(e.to_string()),
            },
            DomainError::FileNotFound(path) => Self::NotFound(format!("File not found: {}", path)),
            DomainError::PermissionDenied(path) => {
                Self::Forbidden(format!("Permission denied: {}", path))
            }
            DomainError::ConfigurationError(msg) => Self::InternalError(msg),
        }
    }
}

impl From<crate::domain::errors::MagicError> for ApplicationError {
    fn from(err: crate::domain::errors::MagicError) -> Self {
        match err {
            crate::domain::errors::MagicError::FileNotFound(path) => {
                Self::NotFound(format!("File not found: {}", path))
            }
            _ => Self::UnprocessableEntity(err.to_string()),
        }
    }
}

impl From<crate::domain::errors::ValidationError> for ApplicationError {
    fn from(err: crate::domain::errors::ValidationError) -> Self {
        Self::BadRequest(err.to_string())
    }
}
