use crate::domain::errors::{MagicError, ValidationError};

#[derive(Debug)]
pub enum ApplicationError {
    Domain(MagicError),
    Validation(ValidationError),
    Internal(String),
}

impl From<MagicError> for ApplicationError {
    fn from(err: MagicError) -> Self {
        Self::Domain(err)
    }
}

impl From<ValidationError> for ApplicationError {
    fn from(err: ValidationError) -> Self {
        Self::Validation(err)
    }
}
