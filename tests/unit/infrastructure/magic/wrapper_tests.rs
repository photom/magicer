use magicer::infrastructure::magic::wrapper::MagicCookie;
use magicer::infrastructure::magic::ffi::MAGIC_MIME_TYPE;

fn find_magic_db() -> Option<String> {
    // Check for compiled magic file in target directory
    let patterns = [
        "target/debug/build/magicer-*/out/install/share/misc/magic.mgc",
        "target/release/build/magicer-*/out/install/share/misc/magic.mgc",
        "/usr/share/misc/magic.mgc",
    ];
    
    for pattern in patterns {
        if let Ok(paths) = glob::glob(pattern) {
            if let Some(Ok(path)) = paths.into_iter().next() {
                return Some(path.to_string_lossy().to_string());
            }
        }
    }
    None
}

#[test]
fn test_magic_cookie_lifecycle() {
    let cookie = MagicCookie::open(MAGIC_MIME_TYPE).expect("Failed to open magic cookie");
    cookie.load(find_magic_db().as_deref()).expect("Failed to load magic database");
}

#[test]
fn test_magic_cookie_analyze_buffer() {
    let db_path = find_magic_db();
    let cookie = MagicCookie::open(MAGIC_MIME_TYPE).expect("Failed to open magic cookie");
    cookie.load(db_path.as_deref()).expect("Failed to load magic database");
    
    let shell_script = b"#!/bin/sh\necho hello";
    let result = cookie.buffer(shell_script).expect("Failed to analyze buffer");
    assert_eq!(result, "text/x-shellscript");
}

#[test]
fn test_magic_cookie_send_sync() {
    let cookie = MagicCookie::open(MAGIC_MIME_TYPE).expect("Failed to open magic cookie");
    cookie.load(find_magic_db().as_deref()).expect("Failed to load magic database");
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
