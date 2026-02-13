use crate::infrastructure::errors::InfrastructureError;
use rand::{distributions::Alphanumeric, Rng};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

pub struct TempFileHandler {
    path: PathBuf,
    cleaned_up: bool,
}

impl TempFileHandler {
    pub fn create_temp_file(data: &[u8], base_dir: &Path) -> Result<Self, InfrastructureError> {
        if !base_dir.exists() {
            fs::create_dir_all(base_dir)?;
        }

        let mut retries = 0;
        const MAX_RETRIES: u32 = 10;

        loop {
            let filename = Self::generate_unique_filename();
            let path = base_dir.join(filename);

            match fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
            {
                Ok(mut file) => {
                    file.write_all(data)?;
                    file.sync_all()?;

                    // Set permissions to 0600 on Unix
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        let mut perms = file.metadata()?.permissions();
                        perms.set_mode(0o600);
                        fs::set_permissions(&path, perms)?;
                    }

                    return Ok(Self {
                        path,
                        cleaned_up: false,
                    });
                }
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                    retries += 1;
                    if retries >= MAX_RETRIES {
                        return Err(InfrastructureError::MaxRetriesExceeded(
                            "Failed to generate unique temp filename".to_string(),
                        ));
                    }
                    continue;
                }
                Err(e) => return Err(InfrastructureError::Io(e)),
            }
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn cleanup(&mut self) -> Result<(), InfrastructureError> {
        if !self.cleaned_up {
            if self.path.exists() {
                fs::remove_file(&self.path)?;
            }
            self.cleaned_up = true;
        }
        Ok(())
    }

    fn generate_unique_filename() -> String {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let uuid = Uuid::new_v4().simple().to_string();
        let random: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(8)
            .map(char::from)
            .collect();

        format!("temp_{}_{}_{}.tmp", timestamp, uuid, random)
    }
}

impl Drop for TempFileHandler {
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}
