use futures_util::future::BoxFuture;
use crate::domain::errors::AuthenticationError;
use crate::domain::services::authentication_service::AuthenticationService;
use crate::domain::value_objects::auth::BasicAuthCredentials;
use subtle::ConstantTimeEq;

pub struct BasicAuthService {
    expected_username: String,
    expected_password: String,
}

impl BasicAuthService {
    pub fn new(username: &str, password: &str) -> Self {
        Self {
            expected_username: username.to_string(),
            expected_password: password.to_string(),
        }
    }
}

impl AuthenticationService for BasicAuthService {
    fn verify_credentials<'a>(&'a self, credentials: &'a BasicAuthCredentials) -> BoxFuture<'a, Result<(), AuthenticationError>> {
        Box::pin(async move {
            let username_matches = self.expected_username.as_bytes().ct_eq(credentials.username().as_bytes());
            let password_matches = self.expected_password.as_bytes().ct_eq(credentials.password().as_bytes());

            if (username_matches & password_matches).into() {
                Ok(())
            } else {
                Err(AuthenticationError::InvalidCredentials)
            }
        })
    }
}
