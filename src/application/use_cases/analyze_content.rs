use crate::application::errors::ApplicationError;
use crate::domain::entities::magic_result::MagicResult;
use crate::domain::repositories::magic_repository::MagicRepository;
use crate::domain::services::temp_storage::TempStorageService;
use crate::domain::value_objects::filename::WindowsCompatibleFilename;
use crate::domain::value_objects::request_id::RequestId;
use crate::infrastructure::config::server_config::ServerConfig;
use futures_util::{Stream, StreamExt};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

pub struct AnalyzeContentUseCase {
    magic_repo: Arc<dyn MagicRepository>,
    temp_storage: Arc<dyn TempStorageService>,
    config: Arc<ServerConfig>,
}

impl AnalyzeContentUseCase {
    pub fn new(
        magic_repo: Arc<dyn MagicRepository>,
        temp_storage: Arc<dyn TempStorageService>,
        config: Arc<ServerConfig>,
    ) -> Self {
        Self {
            magic_repo,
            temp_storage,
            config,
        }
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
        .map_err(|_| ApplicationError::Timeout)??;

        Ok(MagicResult::new(
            request_id,
            filename,
            mime_type,
            description,
        ))
    }

    pub async fn execute_stream<S, E>(
        &self,
        request_id: RequestId,
        filename: WindowsCompatibleFilename,
        mut stream: S,
    ) -> Result<MagicResult, ApplicationError>
    where
        S: Stream<Item = Result<bytes::Bytes, E>> + Unpin + Send,
        E: std::fmt::Display,
    {
        let threshold = self.config.analysis.large_file_threshold_mb * 1024 * 1024;
        let buffer_size = self.config.analysis.write_buffer_size_kb * 1024;

        let mut buffer = Vec::with_capacity(buffer_size.min(threshold));
        let mut is_large = false;
        let mut temp_file = None;

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.map_err(|e| ApplicationError::BadRequest(e.to_string()))?;

            if !is_large {
                if buffer.len() + chunk.len() > threshold {
                    // Check disk space before switching to file
                    let free_space = self
                        .config
                        .get_free_space_mb(&self.config.analysis.temp_dir);
                    if free_space < self.config.analysis.min_free_space_mb {
                        return Err(ApplicationError::InternalError(
                            "Insufficient storage space for analysis".to_string(),
                        ));
                    }

                    is_large = true;
                    let mut tf = self
                        .temp_storage
                        .create_temp_file()
                        .await
                        .map_err(|e| {
                            ApplicationError::InternalError(format!(
                                "Failed to create temp file: {}",
                                e
                            ))
                        })?;

                    tf.write(&buffer).await.map_err(|e| {
                        ApplicationError::InternalError(format!(
                            "Failed to write buffer to temp file: {}",
                            e
                        ))
                    })?;

                    tf.write(&chunk).await.map_err(|e| {
                        ApplicationError::InternalError(format!(
                            "Failed to write chunk to temp file: {}",
                            e
                        ))
                    })?;

                    temp_file = Some(tf);
                    buffer.clear();
                    buffer.shrink_to_fit();
                } else {
                    buffer.extend_from_slice(&chunk);
                }
            } else {
                if let Some(tf) = temp_file.as_mut() {
                    tf.write(&chunk).await.map_err(|e| {
                        ApplicationError::InternalError(format!(
                            "Failed to write chunk to temp file: {}",
                            e
                        ))
                    })?;
                }
            }
        }

        if is_large {
            if let Some(mut tf) = temp_file {
                tf.sync().await.map_err(|e| {
                    ApplicationError::InternalError(format!("Failed to sync temp file: {}", e))
                })?;

                self.execute_from_file(request_id, filename, tf.path())
                    .await
            } else {
                unreachable!()
            }
        } else {
            self.execute(request_id, filename, &buffer).await
        }
    }

    pub async fn execute_from_file(
        &self,
        request_id: RequestId,
        filename: WindowsCompatibleFilename,
        path: &Path,
    ) -> Result<MagicResult, ApplicationError> {
        let timeout_secs = self.config.server.timeouts.analysis_timeout_secs;

        let (mime_type, description) = timeout(
            Duration::from_secs(timeout_secs),
            self.magic_repo.analyze_file(path),
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
