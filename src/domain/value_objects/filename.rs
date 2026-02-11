use crate::domain::errors::ValidationError;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct WindowsCompatibleFilename(String);

const MAX_LENGTH: usize = 310;

impl WindowsCompatibleFilename {
    pub fn new(filename: &str) -> Result<Self, ValidationError> {
        if filename.is_empty() {
            return Err(ValidationError::EmptyValue);
        }
        if filename.len() > MAX_LENGTH {
            return Err(ValidationError::ExceedsMaxLength);
        }
        
        let reserved = ['/', '\\', ':', '*', '?', '"', '<', '>', '|', '\0'];
        if filename.chars().any(|c| reserved.contains(&c)) {
            return Err(ValidationError::InvalidCharacter);
        }
        
        Ok(Self(filename.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
