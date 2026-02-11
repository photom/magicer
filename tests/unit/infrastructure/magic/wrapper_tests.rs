use magicer::infrastructure::magic::wrapper::MagicCookie;
use magicer::infrastructure::magic::ffi::MAGIC_MIME_TYPE;

#[test]
fn test_magic_cookie_lifecycle() {
    let cookie = MagicCookie::open(MAGIC_MIME_TYPE).expect("Failed to open magic cookie");
    cookie.load(None).expect("Failed to load default database");
    // Drop happens automatically
}

#[test]
fn test_magic_cookie_analyze_buffer() {
    let cookie = MagicCookie::open(MAGIC_MIME_TYPE).expect("Failed to open magic cookie");
    cookie.load(None).expect("Failed to load default database");
    
    let png_header = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    let result = cookie.buffer(&png_header).expect("Failed to analyze buffer");
    assert_eq!(result, "image/png");
}

#[test]
fn test_magic_cookie_send_sync() {
    let cookie = MagicCookie::open(MAGIC_MIME_TYPE).expect("Failed to open magic cookie");
    let cookie = std::sync::Arc::new(cookie);
    
    let c = cookie.clone();
    let handle = std::thread::spawn(move || {
        let png_header = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        c.buffer(&png_header).unwrap();
    });
    handle.join().unwrap();
}

#[test]
fn test_magic_cookie_error_handling() {
    let cookie = MagicCookie::open(MAGIC_MIME_TYPE).expect("Failed to open magic cookie");
    let result = cookie.load(Some("non_existent.mgc"));
    assert!(result.is_err());
}
