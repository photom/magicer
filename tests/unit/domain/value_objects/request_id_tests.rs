use magicer::domain::value_objects::request_id::RequestId;
use uuid::Uuid;

#[test]
fn test_request_id_generate() {
    let request_id = RequestId::generate();
    let s = request_id.as_str();
    assert!(Uuid::parse_str(s).is_ok());
}

#[test]
fn test_request_id_from_valid_uuid() {
    let valid_uuid = Uuid::new_v4().to_string();
    let request_id = RequestId::try_from(valid_uuid.as_str());
    assert!(request_id.is_ok());
    assert_eq!(request_id.unwrap().as_str(), valid_uuid);
}

#[test]
fn test_request_id_from_invalid_uuid() {
    let invalid_uuid = "not-a-uuid";
    let request_id = RequestId::try_from(invalid_uuid);
    assert!(request_id.is_err());
    assert_eq!(request_id.unwrap_err(), magicer::domain::errors::ValidationError::InvalidCharacter);
}
