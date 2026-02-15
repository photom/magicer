use std::sync::Arc;
use std::path::PathBuf;
use futures_util::future::BoxFuture;
use magicer::application::use_cases::analyze_path::AnalyzePathUseCase;
use magicer::domain::repositories::magic_repository::MagicRepository;
use magicer::domain::services::sandbox_service::SandboxService;
use magicer::domain::value_objects::request_id::RequestId;
use magicer::domain::value_objects::filename::WindowsCompatibleFilename;
use magicer::domain::value_objects::path::RelativePath;
use magicer::domain::value_objects::mime_type::MimeType;
use magicer::domain::errors::{MagicError, ValidationError};
use magicer::application::errors::ApplicationError;

use magicer::infrastructure::config::server_config::ServerConfig;

struct FakeMagicRepo;
impl MagicRepository for FakeMagicRepo {
    fn analyze_buffer<'a>(&'a self, _data: &'a [u8], _filename: &'a str) -> BoxFuture<'a, Result<(MimeType, String), MagicError>> {
        Box::pin(async {
            Ok((MimeType::try_from("application/pdf").unwrap(), "PDF document".to_string()))
        })
    }
}

struct FakeSandbox {
    root: PathBuf,
}
impl SandboxService for FakeSandbox {
    fn resolve_path(&self, path: &RelativePath) -> Result<PathBuf, ValidationError> {
        Ok(self.root.join(path.as_str()))
    }
}

#[tokio::test]
async fn test_analyze_path_success() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root = temp_dir.path().to_path_buf();
    let upload_dir = root.join("uploads");
    std::fs::create_dir_all(&upload_dir).unwrap();
    let file_path = upload_dir.join("test.pdf");
    std::fs::write(&file_path, b"%PDF-1.4").unwrap();

    let repo: Arc<dyn MagicRepository> = Arc::new(FakeMagicRepo);
    let sandbox: Arc<dyn SandboxService> = Arc::new(FakeSandbox { root });
    let config = Arc::new(ServerConfig::default());
    let timeout = config.server.timeouts.analysis_timeout_secs;
    let use_case = AnalyzePathUseCase::new(repo, sandbox, timeout);
    let request_id = RequestId::generate();
    let filename = WindowsCompatibleFilename::new("test.pdf").unwrap();
    let path = RelativePath::new("uploads/test.pdf").unwrap();
    
    let result = use_case.execute(request_id, filename, path).await.unwrap();
    
    assert_eq!(result.mime_type().as_str(), "application/pdf");
    assert_eq!(result.description(), "PDF document");
}

struct BoundaryViolatingSandbox;
impl SandboxService for BoundaryViolatingSandbox {
    fn resolve_path(&self, _path: &RelativePath) -> Result<PathBuf, ValidationError> {
        Err(ValidationError::PathTraversal)
    }
}

#[tokio::test]
async fn test_analyze_path_outside_sandbox_rejected() {
    let repo: Arc<dyn MagicRepository> = Arc::new(FakeMagicRepo);
    let sandbox: Arc<dyn SandboxService> = Arc::new(BoundaryViolatingSandbox);
    let config = Arc::new(ServerConfig::default());
    let timeout = config.server.timeouts.analysis_timeout_secs;
    let use_case = AnalyzePathUseCase::new(repo, sandbox, timeout);
    let request_id = RequestId::generate();
    let filename = WindowsCompatibleFilename::new("test.pdf").unwrap();
    let path = RelativePath::new("test.pdf").unwrap();
    
    let result = use_case.execute(request_id, filename, path).await;
    assert!(result.is_err());
}

struct NotFoundSandbox {
    root: PathBuf,
}
impl SandboxService for NotFoundSandbox {
    fn resolve_path(&self, _path: &RelativePath) -> Result<PathBuf, ValidationError> {
        Ok(self.root.join("non_existent_file"))
    }
}

#[tokio::test]
async fn test_analyze_path_not_found() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo: Arc<dyn MagicRepository> = Arc::new(FailingMagicRepo);
    let sandbox: Arc<dyn SandboxService> = Arc::new(NotFoundSandbox { root: temp_dir.path().to_path_buf() });
    let config = Arc::new(ServerConfig::default());
    let timeout = config.server.timeouts.analysis_timeout_secs;
    let use_case = AnalyzePathUseCase::new(repo, sandbox, timeout);
    let request_id = RequestId::generate();
    let filename = WindowsCompatibleFilename::new("test.pdf").unwrap();
    let path = RelativePath::new("missing.pdf").unwrap();
    
    let result = use_case.execute(request_id, filename, path).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, ApplicationError::NotFound(_)));
}

struct SlowMagicRepo;
impl MagicRepository for SlowMagicRepo {
    fn analyze_buffer<'a>(&'a self, _data: &'a [u8], _filename: &'a str) -> BoxFuture<'a, Result<(MimeType, String), MagicError>> {
        Box::pin(async { 
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            Ok((MimeType::try_from("application/pdf").unwrap(), "PDF document".to_string())) 
        })
    }
}

#[tokio::test]
async fn test_analyze_path_timeout() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root = temp_dir.path().to_path_buf();
    let file_path = root.join("test.pdf");
    std::fs::write(&file_path, b"%PDF-1.4").unwrap();

    let repo: Arc<dyn MagicRepository> = Arc::new(SlowMagicRepo);
    let sandbox: Arc<dyn SandboxService> = Arc::new(FakeSandbox { root });
    let timeout = 1; // 1 second timeout
    let use_case = AnalyzePathUseCase::new(repo, sandbox, timeout);
    let request_id = RequestId::generate();
    let filename = WindowsCompatibleFilename::new("test.pdf").unwrap();
    let path = RelativePath::new("test.pdf").unwrap();
    
    let result = use_case.execute(request_id, filename, path).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.status_code(), axum::http::StatusCode::GATEWAY_TIMEOUT);
}

struct FailingMagicRepo;
impl MagicRepository for FailingMagicRepo {
    fn analyze_buffer<'a>(&'a self, _data: &'a [u8], _filename: &'a str) -> BoxFuture<'a, Result<(MimeType, String), MagicError>> {
        Box::pin(async { Err(MagicError::AnalysisFailed("fail".to_string())) })
    }
}
