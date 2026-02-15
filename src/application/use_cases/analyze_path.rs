use crate::application::errors::ApplicationError;
use crate::domain::entities::magic_result::MagicResult;
use crate::domain::repositories::magic_repository::MagicRepository;
use crate::domain::services::sandbox_service::SandboxService;
use crate::domain::value_objects::filename::WindowsCompatibleFilename;
use crate::domain::value_objects::path::RelativePath;
use crate::domain::value_objects::request_id::RequestId;
use crate::infrastructure::filesystem::mmap::MmapHandler;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

pub struct AnalyzePathUseCase {
    magic_repo: Arc<dyn MagicRepository>,
    sandbox: Arc<dyn SandboxService>,
    analysis_timeout_secs: u64,
}

impl AnalyzePathUseCase {
    pub fn new(
        magic_repo: Arc<dyn MagicRepository>,
        sandbox: Arc<dyn SandboxService>,
        analysis_timeout_secs: u64,
    ) -> Self {
        Self {
            magic_repo,
            sandbox,
            analysis_timeout_secs,
        }
    }

    pub async fn execute(
        &self,
        request_id: RequestId,
        filename: WindowsCompatibleFilename,
        path: RelativePath,
    ) -> Result<MagicResult, ApplicationError> {
        let resolved_path = self.sandbox.resolve_path(&path)?;

        let file = std::fs::File::open(&resolved_path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                ApplicationError::NotFound(resolved_path.to_string_lossy().to_string())
            } else {
                ApplicationError::InternalError(format!("Failed to open file for analysis: {}", e))
            }
        })?;

        let mmap = MmapHandler::new(&file).map_err(|e| {
            ApplicationError::InternalError(format!("Failed to mmap file for analysis: {}", e))
        })?;

        let (mime_type, description) = timeout(
            Duration::from_secs(self.analysis_timeout_secs),
            self.magic_repo
                .analyze_buffer(mmap.as_slice(), filename.as_str()),
        )
        .await
        .map_err(|_| ApplicationError::Timeout)??;

        Ok(MagicResult::new(
            request_id,
            filename,
            mime_type,
            description,
        ))
    }
}
