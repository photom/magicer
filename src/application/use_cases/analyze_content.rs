use crate::application::errors::ApplicationError;
use crate::domain::entities::magic_result::MagicResult;
use crate::domain::repositories::magic_repository::MagicRepository;
use crate::domain::value_objects::filename::WindowsCompatibleFilename;
use crate::domain::value_objects::request_id::RequestId;
use crate::infrastructure::config::server_config::ServerConfig;
use crate::infrastructure::filesystem::mmap::MmapHandler;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

pub struct AnalyzeContentUseCase {
    magic_repo: Arc<dyn MagicRepository>,
    config: Arc<ServerConfig>,
}

impl AnalyzeContentUseCase {
    pub fn new(magic_repo: Arc<dyn MagicRepository>, config: Arc<ServerConfig>) -> Self {
        Self { magic_repo, config }
    }

    pub async fn execute(
        &self,
        request_id: RequestId,
        filename: WindowsCompatibleFilename,
        data: &[u8],
    ) -> Result<MagicResult, ApplicationError> {
        if data.is_empty() {
            return Err(ApplicationError::BadRequest(
                "Content cannot be empty".to_string(),
            ));
        }

        let timeout_secs = self.config.server.timeouts.analysis_timeout_secs;

        let (mime_type, description) = timeout(
            Duration::from_secs(timeout_secs),
            self.magic_repo.analyze_buffer(data, filename.as_str()),
        )
        .await
        .map_err(|_| ApplicationError::InternalError("Analysis timed out".to_string()))??;

        Ok(MagicResult::new(
            request_id,
            filename,
            mime_type,
            description,
        ))
    }

    pub async fn execute_from_file(
        &self,
        request_id: RequestId,
        filename: WindowsCompatibleFilename,
        path: &Path,
    ) -> Result<MagicResult, ApplicationError> {
        let timeout_secs = self.config.server.timeouts.analysis_timeout_secs;

        let result = match MmapHandler::new(&std::fs::File::open(path).unwrap()) {
            Ok(mmap) => timeout(
                Duration::from_secs(timeout_secs),
                self.magic_repo
                    .analyze_buffer(mmap.as_slice(), filename.as_str()),
            )
            .await
            .map_err(|_| ApplicationError::InternalError("Analysis timed out".to_string()))?,
            Err(e) if self.config.analysis.mmap_fallback_enabled => {
                tracing::warn!("Mmap failed, falling back to buffer: {}", e);
                let data = std::fs::read(path)
                    .map_err(|e| ApplicationError::InternalError(e.to_string()))?;
                timeout(
                    Duration::from_secs(timeout_secs),
                    self.magic_repo.analyze_buffer(&data, filename.as_str()),
                )
                .await
                .map_err(|_| ApplicationError::InternalError("Analysis timed out".to_string()))?
            }
            Err(e) => {
                return Err(ApplicationError::InternalError(format!(
                    "Failed to mmap temp file: {}",
                    e
                )))
            }
        }?;

        let (mime_type, description) = result;

        Ok(MagicResult::new(
            request_id,
            filename,
            mime_type,
            description,
        ))
    }
}
