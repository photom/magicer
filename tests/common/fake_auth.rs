use magicer::domain::services::authentication_service::AuthenticationService;
use magicer::domain::value_objects::auth::BasicAuthCredentials;
use magicer::domain::errors::AuthenticationError;
use futures_util::future::BoxFuture;

pub struct FakeAuth;

impl AuthenticationService for FakeAuth {
    fn verify_credentials<'a>(&'a self, _credentials: &'a BasicAuthCredentials) -> BoxFuture<'a, Result<(), AuthenticationError>> {
        Box::pin(async { Ok(()) })
    }
}
