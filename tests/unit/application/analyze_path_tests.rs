use std::sync::Arc;
use std::path::{Path, PathBuf};
use futures_util::future::BoxFuture;
use magicer::application::use_cases::analyze_path::AnalyzePathUseCase;
use magicer::domain::repositories::magic_repository::MagicRepository;
use magicer::domain::services::sandbox_service::SandboxService;
use magicer::domain::value_objects::request_id::RequestId;
use magicer::domain::value_objects::filename::WindowsCompatibleFilename;
use magicer::domain::value_objects::path::RelativePath;
use magicer::domain::value_objects::mime_type::MimeType;
use magicer::domain::errors::{MagicError, ValidationError};

struct FakeMagicRepo;
impl MagicRepository for FakeMagicRepo {
    fn analyze_buffer<'a>(&'a self, _data: &'a [u8], _filename: &'a str) -> BoxFuture<'a, Result<(MimeType, String), MagicError>> {
        Box::pin(async { Ok((MimeType::try_from("application/octet-stream").unwrap(), "data".to_string())) })
    }
    fn analyze_file<'a>(&'a self, _path: &'a Path) -> BoxFuture<'a, Result<(MimeType, String), MagicError>> {
        Box::pin(async {
            Ok((MimeType::try_from("application/pdf").unwrap(), "PDF document".to_string()))
        })
    }
}

struct FakeSandbox;
impl SandboxService for FakeSandbox {
    fn resolve_path(&self, path: &RelativePath) -> Result<PathBuf, ValidationError> {
        Ok(PathBuf::from("/sandbox").join(path.as_str()))
    }
}

#[tokio::test]
async fn test_analyze_path_success() {
    let repo: Arc<dyn MagicRepository> = Arc::new(FakeMagicRepo);
    let sandbox: Arc<dyn SandboxService> = Arc::new(FakeSandbox);
    let use_case = AnalyzePathUseCase::new(repo, sandbox);
    let request_id = RequestId::generate();
    let filename = WindowsCompatibleFilename::new("test.pdf").unwrap();
    let path = RelativePath::new("uploads/test.pdf").unwrap();
    
    let result = use_case.execute(request_id, filename, path).await.unwrap();
    
    assert_eq!(result.mime_type().as_str(), "application/pdf");
    assert_eq!(result.description(), "PDF document");
}
