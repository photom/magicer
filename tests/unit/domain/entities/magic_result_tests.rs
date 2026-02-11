use magicer::domain::entities::magic_result::MagicResult;
use magicer::domain::value_objects::request_id::RequestId;
use magicer::domain::value_objects::filename::WindowsCompatibleFilename;
use magicer::domain::value_objects::mime_type::MimeType;

#[test]
fn test_magic_result_creation() {
    let request_id = RequestId::generate();
    let filename = WindowsCompatibleFilename::new("test.txt").unwrap();
    let mime_type = MimeType::try_from("application/octet-stream").unwrap();
    let description = "Binary data".to_string();

    let result = MagicResult::new(
        request_id.clone(),
        filename.clone(),
        mime_type.clone(),
        description.clone(),
    );

    assert_eq!(result.request_id(), &request_id);
    assert_eq!(result.filename(), &filename);
    assert_eq!(result.mime_type(), &mime_type);
    assert_eq!(result.description(), &description);
    assert!(result.encoding().is_none());
}

#[test]
fn test_magic_result_with_encoding() {
    let request_id = RequestId::generate();
    let filename = WindowsCompatibleFilename::new("test.txt").unwrap();
    let mime_type = MimeType::try_from("text/plain").unwrap();
    let result = MagicResult::new(request_id, filename, mime_type, "text".to_string())
        .with_encoding(Some("utf-8".to_string()));
    
    assert_eq!(result.encoding(), Some("utf-8"));
}

#[test]
fn test_magic_result_identity() {
    let request_id = RequestId::generate();
    let filename = WindowsCompatibleFilename::new("test.txt").unwrap();
    let mime_type = MimeType::try_from("text/plain").unwrap();
    
    let result1 = MagicResult::new(request_id.clone(), filename.clone(), mime_type.clone(), "text".to_string());
    let result2 = MagicResult::new(request_id, filename, mime_type, "text".to_string());
    
    assert_ne!(result1.id(), result2.id());
    assert_ne!(result1, result2);
}
