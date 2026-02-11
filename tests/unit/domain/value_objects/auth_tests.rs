use magicer::domain::value_objects::auth::BasicAuthCredentials;

#[test]
fn test_credentials_valid_accepted() {
    let creds = BasicAuthCredentials::new("user", "pass");
    assert!(creds.is_ok());
    let creds = creds.unwrap();
    assert_eq!(creds.username(), "user");
    assert_eq!(creds.password(), "pass");
}

#[test]
fn test_credentials_empty_username_rejected() {
    let creds = BasicAuthCredentials::new("", "pass");
    assert!(creds.is_err());
    assert_eq!(creds.unwrap_err(), magicer::domain::errors::ValidationError::EmptyValue);
}

#[test]
fn test_credentials_empty_password_rejected() {
    let creds = BasicAuthCredentials::new("user", "");
    assert!(creds.is_err());
    assert_eq!(creds.unwrap_err(), magicer::domain::errors::ValidationError::EmptyValue);
}
