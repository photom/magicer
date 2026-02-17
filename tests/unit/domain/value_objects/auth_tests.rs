use magicer::domain::value_objects::auth::BasicAuthCredentials;

#[test]
fn test_new_with_valid_credentials_returns_success() {
    // codeql[rust/hard-coded-cryptographic-value]: suppress
    let creds = BasicAuthCredentials::new("user", "pass");
    assert!(creds.is_ok());
    let creds = creds.unwrap();
    // codeql[rust/hard-coded-cryptographic-value]: suppress
    assert_eq!(creds.username(), "user");
    // codeql[rust/hard-coded-cryptographic-value]: suppress
    assert!(creds.verify("user", "pass"));
}

#[test]
fn test_verify_with_correct_credentials_returns_true() {
    // codeql[rust/hard-coded-cryptographic-value]: suppress
    let creds = BasicAuthCredentials::new("admin", "secret").unwrap();
    // codeql[rust/hard-coded-cryptographic-value]: suppress
    assert!(creds.verify("admin", "secret"));
    // codeql[rust/hard-coded-cryptographic-value]: suppress
    assert!(!creds.verify("admin", "wrong"));
    // codeql[rust/hard-coded-cryptographic-value]: suppress
    assert!(!creds.verify("wrong", "secret"));
}

#[test]
fn test_new_with_colon_in_username_returns_error() {
    // codeql[rust/hard-coded-cryptographic-value]: suppress
    let creds = BasicAuthCredentials::new("user:name", "pass");
    assert!(creds.is_err());
}

#[test]
fn test_new_with_empty_username_returns_error() {
    // codeql[rust/hard-coded-cryptographic-value]: suppress
    let creds = BasicAuthCredentials::new("", "pass");
    assert!(creds.is_err());
}
