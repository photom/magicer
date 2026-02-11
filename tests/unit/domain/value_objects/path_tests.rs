use magicer::domain::value_objects::path::RelativePath;
use magicer::domain::errors::ValidationError;

#[test]
fn test_path_valid_accepted() {
    let path = RelativePath::new("uploads/file.txt");
    assert!(path.is_ok());
    assert_eq!(path.unwrap().as_str(), "uploads/file.txt");
}

#[test]
fn test_path_absolute_rejected() {
    let path = RelativePath::new("/etc/passwd");
    assert!(path.is_err());
    assert_eq!(path.unwrap_err(), ValidationError::AbsolutePath);
}

#[test]
fn test_path_traversal_rejected() {
    let paths = ["../etc/passwd", "data/../file.txt", "..", "./.."];
    for p in paths {
        let path = RelativePath::new(p);
        assert!(path.is_err(), "Should reject traversal: {}", p);
        assert_eq!(path.unwrap_err(), ValidationError::PathTraversal);
    }
}

#[test]
fn test_path_double_slash_rejected() {
    let path = RelativePath::new("data//file.txt");
    assert!(path.is_err());
    assert_eq!(path.unwrap_err(), ValidationError::InvalidPath);
}

#[test]
fn test_path_ends_with_dot_rejected() {
    let paths = ["data/.", ".", "./"];
    for p in paths {
        let path = RelativePath::new(p);
        assert!(path.is_err(), "Should reject path ending with dot: {}", p);
        assert_eq!(path.unwrap_err(), ValidationError::InvalidPath);
    }
}

#[test]
fn test_path_leading_space_rejected() {
    let path = RelativePath::new(" data/file.txt");
    assert!(path.is_err());
    assert_eq!(path.unwrap_err(), ValidationError::InvalidPath);
}
