use crate::domain::errors::AuthenticationError;
use crate::domain::value_objects::auth::BasicAuthCredentials;
use futures_util::future::BoxFuture;

pub trait AuthenticationService: Send + Sync {
    fn verify_credentials<'a>(
        &'a self,
        credentials: &'a BasicAuthCredentials,
    ) -> BoxFuture<'a, Result<(), AuthenticationError>>;
}
