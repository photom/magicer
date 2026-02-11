use magicer::domain::services::authentication_service::AuthenticationService;
use magicer::infrastructure::auth::basic_auth_service::BasicAuthService;
use magicer::domain::value_objects::auth::BasicAuthCredentials;

#[tokio::test]
async fn test_auth_service_success() {
    let service = BasicAuthService::new("admin", "secret");
    let creds = BasicAuthCredentials::new("admin", "secret").unwrap();
    
    let result = service.verify_credentials(&creds).await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_auth_service_invalid_credentials() {
    let service = BasicAuthService::new("admin", "secret");
    let creds = BasicAuthCredentials::new("admin", "wrong").unwrap();
    
    let result = service.verify_credentials(&creds).await;
    
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), magicer::domain::errors::AuthenticationError::InvalidCredentials);
}
