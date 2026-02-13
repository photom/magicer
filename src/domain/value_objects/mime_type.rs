use crate::domain::errors::ValidationError;

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct MimeType {
    type_part: String,
    subtype_part: String,
}

impl MimeType {
    pub fn new(mime_str: &str) -> Result<Self, ValidationError> {
        if mime_str.is_empty() {
            return Err(ValidationError::EmptyValue);
        }
        let parts: Vec<&str> = mime_str.split('/').collect();
        if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
            return Err(ValidationError::InvalidCharacter);
        }

        Ok(Self {
            type_part: parts[0].to_string(),
            subtype_part: parts[1].to_string(),
        })
    }

    pub fn from_parts(type_part: String, subtype_part: String) -> Result<Self, ValidationError> {
        if type_part.is_empty() || subtype_part.is_empty() {
            return Err(ValidationError::EmptyValue);
        }
        Ok(Self {
            type_part,
            subtype_part,
        })
    }

    pub fn type_part(&self) -> &str {
        &self.type_part
    }

    pub fn subtype(&self) -> &str {
        &self.subtype_part
    }

    pub fn as_str(&self) -> String {
        format!("{}/{}", self.type_part, self.subtype_part)
    }

    pub fn is_text(&self) -> bool {
        self.type_part == "text"
    }

    pub fn is_binary(&self) -> bool {
        !self.is_text()
    }

    pub fn is_application(&self) -> bool {
        self.type_part == "application"
    }
}

impl TryFrom<&str> for MimeType {
    type Error = ValidationError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<String> for MimeType {
    type Error = ValidationError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(&value)
    }
}
