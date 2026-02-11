use magicer::domain::value_objects::auth::BasicAuthCredentials;

#[test]
fn test_credentials_valid_accepted() {
    let creds = BasicAuthCredentials::new("user", "pass");
    assert!(creds.is_ok());
    let creds = creds.unwrap();
    assert_eq!(creds.username(), "user");
    assert!(creds.verify("user", "pass"));
}

#[test]
fn test_credentials_verify() {
    let creds = BasicAuthCredentials::new("admin", "secret").unwrap();
    assert!(creds.verify("admin", "secret"));
    assert!(!creds.verify("admin", "wrong"));
    assert!(!creds.verify("wrong", "secret"));
}

#[test]
fn test_credentials_no_colon_allowed() {
    let creds = BasicAuthCredentials::new("user:name", "pass");
    assert!(creds.is_err());
}

#[test]
fn test_credentials_empty_username_rejected() {
    let creds = BasicAuthCredentials::new("", "pass");
    assert!(creds.is_err());
}
