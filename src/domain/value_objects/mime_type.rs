use crate::domain::errors::ValidationError;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct MimeType(String);

impl MimeType {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<&str> for MimeType {
    type Error = ValidationError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err(ValidationError::EmptyValue);
        }
        if !value.contains('/') {
            return Err(ValidationError::InvalidCharacter);
        }
        Ok(Self(value.to_string()))
    }
}

impl TryFrom<String> for MimeType {
    type Error = ValidationError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}
