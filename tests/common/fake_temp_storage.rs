use magicer::domain::services::temp_storage::{TempStorageService, TemporaryFile};
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct FakeTempFile {
    path: PathBuf,
}

impl FakeTempFile {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

#[async_trait]
impl TemporaryFile for FakeTempFile {
    fn path(&self) -> &Path {
        &self.path
    }

    async fn write(&mut self, data: &[u8]) -> Result<(), std::io::Error> {
        // In fake implementation, we just write to the real FS at the fake path
        // because our tests run in a real environment (sandbox)
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open(&self.path)
            .await?;
        file.write_all(data).await?;
        file.sync_all().await
    }

    async fn sync(&mut self) -> Result<(), std::io::Error> {
        Ok(())
    }
}

pub struct FakeTempStorageService {
    base_path: PathBuf,
    counter: AtomicUsize,
}

impl FakeTempStorageService {
    pub fn new(base_path: PathBuf) -> Self {
        std::fs::create_dir_all(&base_path).unwrap();
        Self {
            base_path,
            counter: AtomicUsize::new(0),
        }
    }

    pub fn counter(&self) -> usize {
        self.counter.load(Ordering::SeqCst)
    }

    pub fn reset(&self) {
        self.counter.store(0, Ordering::SeqCst);
    }
}

#[async_trait]
impl TempStorageService for FakeTempStorageService {
    async fn create_temp_file(&self) -> Result<Box<dyn TemporaryFile>, std::io::Error> {
        let id = self.counter.fetch_add(1, Ordering::SeqCst);
        let path = self.base_path.join(format!("temp_{}", id));
        Ok(Box::new(FakeTempFile::new(path)))
    }
}
