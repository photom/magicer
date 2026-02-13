use crate::infrastructure::errors::InfrastructureError;
use std::path::Path;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use crate::infrastructure::filesystem::temp_file_handler::TempFileHandler;
use futures_util::StreamExt;
use axum::body::BodyDataStream;

pub struct StreamWriter;

impl StreamWriter {
    pub async fn write_stream_to_temp(
        mut stream: BodyDataStream,
        base_dir: &Path,
        _request_id: &str,
    ) -> Result<TempFileHandler, InfrastructureError> {
        let temp_file = TempFileHandler::new_empty(base_dir)?;
        let mut file = OpenOptions::new()
            .write(true)
            .open(temp_file.path())
            .await
            .map_err(InfrastructureError::Io)?;

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.map_err(|e| InfrastructureError::Internal(e.to_string()))?;
            file.write_all(&chunk).await.map_err(InfrastructureError::Io)?;
        }

        file.flush().await.map_err(InfrastructureError::Io)?;
        file.sync_all().await.map_err(InfrastructureError::Io)?;

        Ok(temp_file)
    }
}
