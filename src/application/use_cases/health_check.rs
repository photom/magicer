use crate::application::errors::ApplicationError;

pub struct HealthCheckUseCase;

impl HealthCheckUseCase {
    pub fn new() -> Self {
        Self
    }

    pub async fn execute(&self) -> Result<(), ApplicationError> {
        Ok(())
    }
}
