use crate::application::errors::ApplicationError;
use crate::domain::entities::magic_result::MagicResult;
use crate::domain::repositories::magic_repository::MagicRepository;
use crate::domain::services::sandbox_service::SandboxService;
use crate::domain::value_objects::filename::WindowsCompatibleFilename;
use crate::domain::value_objects::path::RelativePath;
use crate::domain::value_objects::request_id::RequestId;
use crate::infrastructure::config::server_config::ServerConfig;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

pub struct AnalyzePathUseCase {
    magic_repo: Arc<dyn MagicRepository>,
    sandbox: Arc<dyn SandboxService>,
    config: Arc<ServerConfig>,
}

impl AnalyzePathUseCase {
    pub fn new(
        magic_repo: Arc<dyn MagicRepository>,
        sandbox: Arc<dyn SandboxService>,
        config: Arc<ServerConfig>,
    ) -> Self {
        Self {
            magic_repo,
            sandbox,
            config,
        }
    }

    pub async fn execute(
        &self,
        request_id: RequestId,
        filename: WindowsCompatibleFilename,
        path: RelativePath,
    ) -> Result<MagicResult, ApplicationError> {
        let resolved_path = self.sandbox.resolve_path(&path)?;

        let timeout_secs = self.config.server.timeouts.analysis_timeout_secs;

        let (mime_type, description) = timeout(
            Duration::from_secs(timeout_secs),
            self.magic_repo.analyze_file(&resolved_path),
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
}
