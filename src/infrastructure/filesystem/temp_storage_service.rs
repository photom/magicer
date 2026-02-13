use crate::domain::services::temp_storage::{TempStorageService, TemporaryFile};
use crate::infrastructure::filesystem::temp_file_handler::TempFileHandler;
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs::{File, OpenOptions};
use tokio::io::AsyncWriteExt;

pub struct FsTempFile {
    handler: TempFileHandler,
    file: Option<File>,
}

impl FsTempFile {
    pub async fn new(base_dir: &Path) -> Result<Self, std::io::Error> {
        let handler = TempFileHandler::new_empty(base_dir)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
        
        let file = OpenOptions::new()
            .write(true)
            .open(handler.path())
            .await?;

        Ok(Self {
            handler,
            file: Some(file),
        })
    }
}

#[async_trait]
impl TemporaryFile for FsTempFile {
    fn path(&self) -> &Path {
        self.handler.path()
    }

    async fn write(&mut self, data: &[u8]) -> Result<(), std::io::Error> {
        if let Some(file) = &mut self.file {
            file.write_all(data).await
        } else {
            Err(std::io::Error::new(std::io::ErrorKind::Other, "File not open"))
        }
    }

    async fn sync(&mut self) -> Result<(), std::io::Error> {
        if let Some(mut file) = self.file.take() {
            file.sync_all().await?;
            Ok(())
        } else {
            Ok(())
        }
    }
}

pub struct FsTempStorageService {
    temp_dir: PathBuf,
}

impl FsTempStorageService {
    pub fn new(temp_dir: PathBuf) -> Self {
        Self { temp_dir }
    }
}

#[async_trait]
impl TempStorageService for FsTempStorageService {
    async fn create_temp_file(&self) -> Result<Box<dyn TemporaryFile>, std::io::Error> {
        let file = FsTempFile::new(&self.temp_dir).await?;
        Ok(Box::new(file))
    }
}
