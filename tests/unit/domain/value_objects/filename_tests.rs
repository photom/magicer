use magicer::domain::value_objects::filename::WindowsCompatibleFilename;
use magicer::domain::errors::ValidationError;

#[test]
fn test_new_with_valid_name_returns_success() {
    let filename = WindowsCompatibleFilename::new("test.txt");
    assert!(filename.is_ok());
    assert_eq!(filename.unwrap().as_str(), "test.txt");
}

#[test]
fn test_new_with_slash_returns_error() {
    let filename = WindowsCompatibleFilename::new("folder/file.txt");
    assert!(filename.is_err());
    assert_eq!(filename.unwrap_err(), ValidationError::InvalidCharacter);
}

#[test]
fn test_new_with_null_byte_returns_error() {
    let filename = WindowsCompatibleFilename::new("file\0.txt");
    assert!(filename.is_err());
    assert_eq!(filename.unwrap_err(), ValidationError::InvalidCharacter);
}

#[test]
fn test_new_with_too_long_name_returns_error() {
    let long_string = "a".repeat(311);
    let filename = WindowsCompatibleFilename::new(&long_string);
    assert!(filename.is_err());
    assert_eq!(filename.unwrap_err(), ValidationError::ExceedsMaxLength);
}

#[test]
fn test_new_with_max_length_name_returns_success() {
    let max_string = "a".repeat(310);
    let filename = WindowsCompatibleFilename::new(&max_string);
    assert!(filename.is_ok());
}

#[test]
fn test_new_with_empty_name_returns_error() {
    let filename = WindowsCompatibleFilename::new("");
    assert!(filename.is_err());
    assert_eq!(filename.unwrap_err(), ValidationError::EmptyValue);
}

#[test]
fn test_new_with_unicode_name_returns_success() {
    let unicode_name = "файл_测试_🎉.txt";
    let filename = WindowsCompatibleFilename::new(unicode_name);
    assert!(filename.is_ok());
    assert_eq!(filename.unwrap().as_str(), unicode_name);
}

#[test]
fn test_new_with_windows_reserved_characters_returns_success() {
    let reserved = ['\\', ':', '*', '?', '"', '<', '>', '|'];
    for &c in &reserved {
        let name = format!("file{}.txt", c);
        let filename = WindowsCompatibleFilename::new(&name);
        assert!(filename.is_ok(), "Should accept character: {}", c);
    }
}
