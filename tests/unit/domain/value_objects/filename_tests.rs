use magicer::domain::value_objects::filename::WindowsCompatibleFilename;
use magicer::domain::errors::ValidationError;

#[test]
fn test_filename_valid_accepted() {
    let filename = WindowsCompatibleFilename::new("test.txt");
    assert!(filename.is_ok());
    assert_eq!(filename.unwrap().as_str(), "test.txt");
}

#[test]
fn test_filename_with_slash_rejected() {
    let filename = WindowsCompatibleFilename::new("folder/file.txt");
    assert!(filename.is_err());
    assert_eq!(filename.unwrap_err(), ValidationError::InvalidCharacter);
}

#[test]
fn test_filename_with_null_byte_rejected() {
    let filename = WindowsCompatibleFilename::new("file\0.txt");
    assert!(filename.is_err());
    assert_eq!(filename.unwrap_err(), ValidationError::InvalidCharacter);
}

#[test]
fn test_filename_too_long_rejected() {
    let long_string = "a".repeat(311);
    let filename = WindowsCompatibleFilename::new(&long_string);
    assert!(filename.is_err());
    assert_eq!(filename.unwrap_err(), ValidationError::ExceedsMaxLength);
}

#[test]
fn test_filename_max_length_accepted() {
    let max_string = "a".repeat(310);
    let filename = WindowsCompatibleFilename::new(&max_string);
    assert!(filename.is_ok());
}

#[test]
fn test_filename_empty_rejected() {
    let filename = WindowsCompatibleFilename::new("");
    assert!(filename.is_err());
    assert_eq!(filename.unwrap_err(), ValidationError::EmptyValue);
}

#[test]
fn test_filename_unicode_accepted() {
    let unicode_name = "файл_测试_🎉.txt";
    let filename = WindowsCompatibleFilename::new(unicode_name);
    assert!(filename.is_ok());
    assert_eq!(filename.unwrap().as_str(), unicode_name);
}

#[test]
fn test_filename_with_reserved_characters_rejected() {
    let reserved = ['\\', ':', '*', '?', '"', '<', '>', '|'];
    for &c in &reserved {
        let name = format!("file{}.txt", c);
        let filename = WindowsCompatibleFilename::new(&name);
        assert!(filename.is_err(), "Should reject character: {}", c);
        assert_eq!(filename.unwrap_err(), ValidationError::InvalidCharacter);
    }
}
