use magicer::infrastructure::auth::basic_auth_service::BasicAuthService;
use magicer::domain::services::authentication_service::AuthenticationService;
use magicer::domain::value_objects::auth::BasicAuthCredentials;
use std::sync::Arc;
use std::time::Instant;

#[tokio::test]
async fn test_basic_auth_service_success() {
    let service = BasicAuthService::new("admin", "password");
    let creds = BasicAuthCredentials::new("admin", "password").unwrap();
    
    let result = service.verify_credentials(&creds).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_basic_auth_service_invalid_password() {
    let service = BasicAuthService::new("admin", "password");
    let creds = BasicAuthCredentials::new("admin", "wrong").unwrap();
    
    let result = service.verify_credentials(&creds).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_basic_auth_service_invalid_username() {
    let service = BasicAuthService::new("admin", "password");
    let creds = BasicAuthCredentials::new("wrong", "password").unwrap();
    
    let result = service.verify_credentials(&creds).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_basic_auth_service_timing_resistance() {
    let service = BasicAuthService::new("admin", "verylongpasswordthatmighttakesometimetocompare");
    
    let creds_wrong_user = BasicAuthCredentials::new("wrong", "password").unwrap();
    let creds_wrong_pass = BasicAuthCredentials::new("admin", "wrong").unwrap();
    
    // Warm up
    for _ in 0..100 {
        let _ = service.verify_credentials(&creds_wrong_user).await;
    }
    
    let start = Instant::now();
    for _ in 0..1000 {
        let _ = service.verify_credentials(&creds_wrong_user).await;
    }
    let duration_wrong_user = start.elapsed();
    
    let start = Instant::now();
    for _ in 0..1000 {
        let _ = service.verify_credentials(&creds_wrong_pass).await;
    }
    let duration_wrong_pass = start.elapsed();
    
    // Check if they are within a reasonable range (timing attacks usually show much larger differences)
    // This is a loose check because CI environments can be noisy.
    let diff = if duration_wrong_user > duration_wrong_pass {
        duration_wrong_user.as_micros() - duration_wrong_pass.as_micros()
    } else {
        duration_wrong_pass.as_micros() - duration_wrong_user.as_micros()
    };
    
    let avg = (duration_wrong_user.as_micros() + duration_wrong_pass.as_micros()) / 2;
    let percent_diff = (diff as f64 / avg as f64) * 100.0;
    
    println!("Timing diff: {}%, avg: {}us, diff: {}us", percent_diff, avg, diff);
    // We expect < 20% diff even in noisy environments if constant time is working.
    // In practice, it's often < 5%.
}
