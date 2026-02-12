use magicer::domain::value_objects::path::RelativePath;
use magicer::domain::errors::ValidationError;

#[test]
fn test_new_with_valid_path_returns_success() {
    let path = RelativePath::new("uploads/file.txt");
    assert!(path.is_ok());
    assert_eq!(path.unwrap().as_str(), "uploads/file.txt");
}

#[test]
fn test_new_with_absolute_path_returns_error() {
    let path = RelativePath::new("/etc/passwd");
    assert!(path.is_err());
    assert_eq!(path.unwrap_err(), ValidationError::AbsolutePath);
}

#[test]
fn test_new_with_traversal_returns_error() {
    let paths = ["../etc/passwd", "data/../file.txt", "..", "./.."];
    for p in paths {
        let path = RelativePath::new(p);
        assert!(path.is_err(), "Should reject traversal: {}", p);
        assert_eq!(path.unwrap_err(), ValidationError::PathTraversal);
    }
}

#[test]
fn test_new_with_double_slash_returns_error() {
    let path = RelativePath::new("data//file.txt");
    assert!(path.is_err());
    assert_eq!(path.unwrap_err(), ValidationError::InvalidPath);
}

#[test]
fn test_new_with_dot_suffix_returns_error() {
    let paths = ["data/.", ".", "./"];
    for p in paths {
        let path = RelativePath::new(p);
        assert!(path.is_err(), "Should reject path ending with dot: {}", p);
        assert_eq!(path.unwrap_err(), ValidationError::InvalidPath);
    }
}

#[test]
fn test_new_with_leading_space_returns_error() {
    let path = RelativePath::new(" data/file.txt");
    assert!(path.is_err());
    assert_eq!(path.unwrap_err(), ValidationError::InvalidPath);
}
