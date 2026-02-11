use std::sync::Arc;
use crate::application::use_cases::analyze_content::AnalyzeContentUseCase;
use crate::application::use_cases::analyze_path::AnalyzePathUseCase;
use crate::application::use_cases::health_check::HealthCheckUseCase;
use crate::domain::repositories::magic_repository::MagicRepository;
use crate::domain::services::sandbox_service::SandboxService;
use crate::domain::services::authentication_service::AuthenticationService;
use crate::infrastructure::config::server_config::ServerConfig;

pub struct AppState {
    pub analyze_content_use_case: AnalyzeContentUseCase,
    pub analyze_path_use_case: AnalyzePathUseCase,
    pub health_check_use_case: HealthCheckUseCase,
    pub auth_service: Arc<dyn AuthenticationService>,
    pub config: Arc<ServerConfig>,
}

impl AppState {
    pub fn new(
        magic_repo: Arc<dyn MagicRepository>,
        sandbox: Arc<dyn SandboxService>,
        auth_service: Arc<dyn AuthenticationService>,
        config: Arc<ServerConfig>,
    ) -> Self {
        Self {
            analyze_content_use_case: AnalyzeContentUseCase::new(magic_repo.clone(), config.clone()),
            analyze_path_use_case: AnalyzePathUseCase::new(magic_repo, sandbox),
            health_check_use_case: HealthCheckUseCase::new(),
            auth_service,
            config,
        }
    }
}
