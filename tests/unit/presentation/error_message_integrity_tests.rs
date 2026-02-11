use magicer::application::errors::ApplicationError;
use magicer::domain::errors::{DomainError, ValidationError, MagicError};

#[test]
fn test_error_message_structure_compliance() {
    let err = ApplicationError::BadRequest("invalid filename".to_string());
    assert!(err.to_string().contains("Bad Request:"));
    
    let domain_err = DomainError::ValidationError(ValidationError::ExceedsMaxLength);
    let app_err = ApplicationError::from(domain_err);
    assert_eq!(app_err.to_string(), "Bad Request: Exceeds maximum length");
}

#[test]
fn test_magic_error_propagation() {
    let magic_err = MagicError::AnalysisFailed("corrupt file".to_string());
    let app_err = ApplicationError::from(magic_err);
    assert_eq!(app_err.to_string(), "Unprocessable Entity: Analysis failed: corrupt file");
}

#[test]
fn test_file_not_found_propagation() {
    let domain_err = DomainError::FileNotFound("/tmp/test.txt".to_string());
    let app_err = ApplicationError::from(domain_err);
    assert_eq!(app_err.to_string(), "Not Found: File not found: /tmp/test.txt");
}
