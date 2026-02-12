use magicer::domain::value_objects::auth::BasicAuthCredentials;

#[test]
fn test_new_with_valid_credentials_returns_success() {
    let creds = BasicAuthCredentials::new("user", "pass");
    assert!(creds.is_ok());
    let creds = creds.unwrap();
    assert_eq!(creds.username(), "user");
    assert!(creds.verify("user", "pass"));
}

#[test]
fn test_verify_with_correct_credentials_returns_true() {
    let creds = BasicAuthCredentials::new("admin", "secret").unwrap();
    assert!(creds.verify("admin", "secret"));
    assert!(!creds.verify("admin", "wrong"));
    assert!(!creds.verify("wrong", "secret"));
}

#[test]
fn test_new_with_colon_in_username_returns_error() {
    let creds = BasicAuthCredentials::new("user:name", "pass");
    assert!(creds.is_err());
}

#[test]
fn test_new_with_empty_username_returns_error() {
    let creds = BasicAuthCredentials::new("", "pass");
    assert!(creds.is_err());
}
