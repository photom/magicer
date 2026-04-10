use crate::domain::errors::ValidationError;
use std::fmt;
use uuid::Uuid;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct RequestId(String);

impl fmt::Display for RequestId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl RequestId {
    pub fn generate() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    pub fn parse(s: &str) -> Result<Self, ValidationError> {
        if Uuid::parse_str(s).is_err() {
            return Err(ValidationError::InvalidCharacter);
        }
        Ok(Self(s.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for RequestId {
    type Error = ValidationError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if Uuid::parse_str(&value).is_err() {
            return Err(ValidationError::InvalidCharacter);
        }
        Ok(Self(value))
    }
}

impl TryFrom<&str> for RequestId {
    type Error = ValidationError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from(value.to_string())
    }
}
