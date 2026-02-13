use crate::domain::errors::ValidationError;
use subtle::ConstantTimeEq;

#[derive(Debug, Clone)]
pub struct BasicAuthCredentials {
    username: String,
    password: String,
}

impl BasicAuthCredentials {
    pub fn new(username: &str, password: &str) -> Result<Self, ValidationError> {
        if username.is_empty() || password.is_empty() {
            return Err(ValidationError::EmptyValue);
        }
        if username.contains(':') {
            return Err(ValidationError::InvalidCharacter);
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

    pub fn verify(&self, username: &str, password: &str) -> bool {
        let u_match = self.username.as_bytes().ct_eq(username.as_bytes());
        let p_match = self.password.as_bytes().ct_eq(password.as_bytes());

        (u_match & p_match).into()
    }
}

impl PartialEq for BasicAuthCredentials {
    fn eq(&self, other: &Self) -> bool {
        self.verify(&other.username, &other.password)
    }
}

impl Eq for BasicAuthCredentials {}
