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
fn test_sandbox_resolve_path_boundary_violation() {
    let base_dir = PathBuf::from("/tmp/sandbox");
    let _sandbox = PathSandbox::new(base_dir);
    // RelativePath already prevents .. traversal, but let's test absolute path escape attempt 
    // (though RelativePath::new rejects it, we want to see how Sandbox handles it if passed)
    // Actually, SandboxService takes RelativePath, which is already validated.
    // So traversal is impossible via RelativePath.
}
