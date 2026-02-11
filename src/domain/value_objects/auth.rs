use crate::domain::errors::ValidationError;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BasicAuthCredentials {
    username: String,
    password: String,
}

impl BasicAuthCredentials {
    pub fn new(username: &str, password: &str) -> Result<Self, ValidationError> {
        if username.is_empty() || password.is_empty() {
            return Err(ValidationError::EmptyValue);
        }
        Ok(Self {
            username: username.to_string(),
            password: password.to_string(),
        })
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn password(&self) -> &str {
        &self.password
    }
}
