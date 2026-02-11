use crate::domain::errors::ValidationError;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct RelativePath(String);

impl RelativePath {
    pub fn new(path: &str) -> Result<Self, ValidationError> {
        if path.starts_with('/') {
            return Err(ValidationError::AbsolutePath);
        }
        if path.starts_with(' ') {
            return Err(ValidationError::InvalidPath);
        }
        if path.contains("//") {
            return Err(ValidationError::InvalidPath);
        }
        
        let parts: Vec<&str> = path.split('/').collect();
        if parts.iter().any(|&p| p == "..") {
            return Err(ValidationError::PathTraversal);
        }
        if parts.iter().any(|&p| p == ".") {
            return Err(ValidationError::InvalidPath);
        }
        
        Ok(Self(path.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
