use magicer::application::use_cases::health_check::HealthCheckUseCase;

#[tokio::test]
async fn test_health_check_success() {
    let use_case = HealthCheckUseCase::new();
    let result = use_case.execute().await;
    
    assert!(result.is_ok());
}
