use std::path::PathBuf;
use magicer::domain::services::sandbox_service::SandboxService;
use magicer::infrastructure::filesystem::sandbox::PathSandbox;
use magicer::domain::value_objects::path::RelativePath;

#[test]
fn test_sandbox_resolve_path_success() {
    let base_dir = PathBuf::from("/tmp/sandbox");
    let sandbox = PathSandbox::new(base_dir.clone());
    let relative_path = RelativePath::new("file.txt").unwrap();
    
    let result = sandbox.resolve_path(&relative_path);
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), base_dir.join("file.txt"));
}

#[test]
fn test_sandbox_within_boundary() {
    let base_dir = PathBuf::from("/tmp/sandbox");
    let sandbox = PathSandbox::new(base_dir.clone());
    let relative_path = RelativePath::new("docs/test.txt").unwrap();
    
    let result = sandbox.resolve_path(&relative_path).unwrap();
    assert!(result.starts_with(&base_dir));
}
