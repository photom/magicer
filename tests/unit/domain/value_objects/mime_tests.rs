use magicer::domain::value_objects::mime_type::MimeType;

#[test]
fn test_try_from_with_valid_mime_returns_success() {
    let mime = MimeType::try_from("application/pdf");
    assert!(mime.is_ok());
    assert_eq!(mime.unwrap().as_str(), "application/pdf");
}

#[test]
fn test_parts_returns_correct_components() {
    let mime = MimeType::new("text/plain").unwrap();
    assert_eq!(mime.type_part(), "text");
    assert_eq!(mime.subtype(), "plain");
}

#[test]
fn test_categorization_methods_return_correct_booleans() {
    let text = MimeType::new("text/html").unwrap();
    assert!(text.is_text());
    assert!(!text.is_application());
    
    let app = MimeType::new("application/json").unwrap();
    assert!(app.is_application());
    assert!(!app.is_text());
    assert!(app.is_binary());
}

#[test]
fn test_try_from_with_invalid_format_returns_error() {
    let mime = MimeType::try_from("not-a-mime");
    assert!(mime.is_err());
}

#[test]
fn test_try_from_with_empty_string_returns_error() {
    let mime = MimeType::try_from("");
    assert!(mime.is_err());
}
