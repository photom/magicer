use crate::application::use_cases::analyze_content::AnalyzeContentUseCase;
use crate::application::use_cases::analyze_path::AnalyzePathUseCase;
use crate::application::use_cases::health_check::HealthCheckUseCase;
use crate::domain::repositories::magic_repository::MagicRepository;
use crate::domain::services::authentication_service::AuthenticationService;
use crate::domain::services::sandbox_service::SandboxService;
use crate::domain::services::temp_storage::TempStorageService;
use crate::infrastructure::config::server_config::ServerConfig;
use std::sync::Arc;

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
        temp_storage: Arc<dyn TempStorageService>,
        auth_service: Arc<dyn AuthenticationService>,
        config: Arc<ServerConfig>,
    ) -> Self {
        let timeout = config.server.timeouts.analysis_timeout_secs;
        Self {
            analyze_content_use_case: AnalyzeContentUseCase::new(
                magic_repo.clone(),
                temp_storage,
                config.clone(),
            ),
            analyze_path_use_case: AnalyzePathUseCase::new(magic_repo, sandbox, timeout),
            health_check_use_case: HealthCheckUseCase::new(),
            auth_service,
            config,
        }
    }
}
