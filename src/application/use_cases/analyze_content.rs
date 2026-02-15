use crate::application::errors::ApplicationError;
use crate::domain::entities::magic_result::MagicResult;
use crate::domain::repositories::magic_repository::MagicRepository;
use crate::domain::services::temp_storage::{TempStorageService, TemporaryFile};
use crate::domain::value_objects::filename::WindowsCompatibleFilename;
use crate::domain::value_objects::request_id::RequestId;
use crate::infrastructure::config::server_config::ServerConfig;
use crate::infrastructure::filesystem::mmap::MmapHandler;
use futures_util::{Stream, StreamExt};
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

    pub async fn analyze_in_memory<S, E>(
        &self,
        request_id: RequestId,
        filename: WindowsCompatibleFilename,
        stream: S,
    ) -> Result<MagicResult, ApplicationError>
    where
        S: Stream<Item = Result<bytes::Bytes, E>> + Unpin + Send,
        E: std::fmt::Display,
    {
        let buffer = self.stream_to_buffer(stream).await?;
        if buffer.is_empty() {
            return Err(ApplicationError::BadRequest(
                "Content cannot be empty".to_string(),
            ));
        }
        self.perform_analysis(request_id, filename, &buffer).await
    }

    pub async fn analyze_to_temp_file<S, E>(
        &self,
        request_id: RequestId,
        filename: WindowsCompatibleFilename,
        stream: S,
    ) -> Result<MagicResult, ApplicationError>
    where
        S: Stream<Item = Result<bytes::Bytes, E>> + Unpin + Send,
        E: std::fmt::Display,
    {
        let mut tf = self.stream_to_file(stream).await?;
        tf.sync().await.map_err(|e| {
            ApplicationError::InternalError(format!("Failed to sync temp file: {}", e))
        })?;

        let file = std::fs::File::open(tf.path()).map_err(|e| {
            ApplicationError::InternalError(format!("Failed to open file for analysis: {}", e))
        })?;

        let mmap = MmapHandler::new(&file).map_err(|e| {
            ApplicationError::InternalError(format!("Failed to mmap file for analysis: {}", e))
        })?;

        if mmap.as_slice().is_empty() {
            return Err(ApplicationError::BadRequest(
                "Content cannot be empty".to_string(),
            ));
        }

        self.perform_analysis(request_id, filename, mmap.as_slice())
            .await
    }

    async fn perform_analysis(
        &self,
        request_id: RequestId,
        filename: WindowsCompatibleFilename,
        data: &[u8],
    ) -> Result<MagicResult, ApplicationError> {
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

    async fn stream_to_buffer<S, E>(&self, mut stream: S) -> Result<Vec<u8>, ApplicationError>
    where
        S: Stream<Item = Result<bytes::Bytes, E>> + Unpin + Send,
        E: std::fmt::Display,
    {
        let mut buffer = Vec::new();
        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.map_err(|e| ApplicationError::BadRequest(e.to_string()))?;
            buffer.extend_from_slice(&chunk);
        }
        Ok(buffer)
    }

    async fn stream_to_file<S, E>(
        &self,
        mut stream: S,
    ) -> Result<Box<dyn TemporaryFile>, ApplicationError>
    where
        S: Stream<Item = Result<bytes::Bytes, E>> + Unpin + Send,
        E: std::fmt::Display,
    {
        let mut tf = self.init_temp_file().await?;
        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.map_err(|e| ApplicationError::BadRequest(e.to_string()))?;
            tf.write(&chunk).await.map_err(|e| {
                ApplicationError::InternalError(format!("Failed to write chunk: {}", e))
            })?;
        }
        Ok(tf)
    }

    async fn init_temp_file(&self) -> Result<Box<dyn TemporaryFile>, ApplicationError> {
        let free_space = self
            .config
            .get_free_space_mb(&self.config.analysis.temp_dir);
        if free_space < self.config.analysis.min_free_space_mb {
            return Err(ApplicationError::InsufficientStorage(format!(
                "Insufficient storage space for analysis: {}MB available, but {}MB required",
                free_space, self.config.analysis.min_free_space_mb
            )));
        }

        self.temp_storage.create_temp_file().await.map_err(|e| {
            ApplicationError::InternalError(format!("Failed to create temp file: {}", e))
        })
    }
}
