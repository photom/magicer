use crate::domain::errors::ValidationError;
use std::path::Path;
use async_trait::async_trait;

#[async_trait]
pub trait TemporaryFile: Send + Sync {
    fn path(&self) -> &Path;
    async fn write(&mut self, data: &[u8]) -> Result<(), std::io::Error>;
    async fn sync(&mut self) -> Result<(), std::io::Error>;
}

#[async_trait]
pub trait TempStorageService: Send + Sync {
    async fn create_temp_file(&self) -> Result<Box<dyn TemporaryFile>, std::io::Error>;
}
