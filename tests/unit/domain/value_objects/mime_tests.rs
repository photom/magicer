use magicer::domain::value_objects::mime_type::MimeType;

#[test]
fn test_mime_type_valid_accepted() {
    let mime = MimeType::try_from("application/pdf");
    assert!(mime.is_ok());
    assert_eq!(mime.unwrap().as_str(), "application/pdf");
}

#[test]
fn test_mime_type_parts() {
    let mime = MimeType::new("text/plain").unwrap();
    assert_eq!(mime.type_part(), "text");
    assert_eq!(mime.subtype(), "plain");
}

#[test]
fn test_mime_type_categorization() {
    let text = MimeType::new("text/html").unwrap();
    assert!(text.is_text());
    assert!(!text.is_application());
    
    let app = MimeType::new("application/json").unwrap();
    assert!(app.is_application());
    assert!(!app.is_text());
    assert!(app.is_binary());
}

#[test]
fn test_mime_type_invalid_format_rejected() {
    let mime = MimeType::try_from("not-a-mime");
    assert!(mime.is_err());
}

#[test]
fn test_mime_type_empty_rejected() {
    let mime = MimeType::try_from("");
    assert!(mime.is_err());
}
