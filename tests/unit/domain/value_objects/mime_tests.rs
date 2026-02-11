use magicer::domain::value_objects::mime_type::MimeType;

#[test]
fn test_mime_type_valid_accepted() {
    let mime = MimeType::try_from("application/pdf");
    assert!(mime.is_ok());
    assert_eq!(mime.unwrap().as_str(), "application/pdf");
}

#[test]
fn test_mime_type_invalid_format_rejected() {
    let mime = MimeType::try_from("not-a-mime");
    assert!(mime.is_err());
    assert_eq!(mime.unwrap_err(), magicer::domain::errors::ValidationError::InvalidCharacter);
}

#[test]
fn test_mime_type_empty_rejected() {
    let mime = MimeType::try_from("");
    assert!(mime.is_err());
    assert_eq!(mime.unwrap_err(), magicer::domain::errors::ValidationError::EmptyValue);
}
