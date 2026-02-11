use std::ffi::{CStr, CString};
use std::ptr;
use std::sync::Mutex;
use crate::infrastructure::magic::ffi::*;
use crate::domain::errors::MagicError;

pub struct MagicCookie {
    inner: Mutex<MagicT>,
}

unsafe impl Send for MagicCookie {}
unsafe impl Sync for MagicCookie {}

impl MagicCookie {
    pub fn open(flags: i32) -> Result<Self, MagicError> {
        let cookie = unsafe { magic_open(flags) };
        if cookie.is_null() {
            return Err(MagicError::AnalysisFailed("Failed to initialize magic cookie".to_string()));
        }
        Ok(Self { inner: Mutex::new(cookie) })
    }

    pub fn load(&self, path: Option<&str>) -> Result<(), MagicError> {
        let c_path = match path {
            Some(p) => Some(CString::new(p).map_err(|_| MagicError::DatabaseLoadFailed("Invalid path".to_string()))?),
            None => None,
        };
        
        let path_ptr = match &c_path {
            Some(p) => p.as_ptr(),
            None => ptr::null(),
        };

        let lock = self.inner.lock().unwrap();
        let result = unsafe { magic_load(*lock, path_ptr) };
        
        if result != 0 {
            let err = self.get_error(*lock);
            return Err(MagicError::DatabaseLoadFailed(err));
        }
        Ok(())
    }

    pub fn buffer(&self, data: &[u8]) -> Result<String, MagicError> {
        let lock = self.inner.lock().unwrap();
        let result = unsafe { magic_buffer(*lock, data.as_ptr() as *const _, data.len()) };
        
        if result.is_null() {
            let err = self.get_error(*lock);
            return Err(MagicError::AnalysisFailed(err));
        }
        
        let c_str = unsafe { CStr::from_ptr(result) };
        Ok(c_str.to_string_lossy().into_owned())
    }

    pub fn file(&self, path: &str) -> Result<String, MagicError> {
        let c_path = CString::new(path).map_err(|_| MagicError::FileNotFound("Invalid path".to_string()))?;
        let lock = self.inner.lock().unwrap();
        let result = unsafe { magic_file(*lock, c_path.as_ptr()) };
        
        if result.is_null() {
            let err = self.get_error(*lock);
            return Err(MagicError::AnalysisFailed(err));
        }
        
        let c_str = unsafe { CStr::from_ptr(result) };
        Ok(c_str.to_string_lossy().into_owned())
    }

    fn get_error(&self, ms: MagicT) -> String {
        let err = unsafe { magic_error(ms) };
        if err.is_null() {
            "Unknown magic error".to_string()
        } else {
            let c_str = unsafe { CStr::from_ptr(err) };
            c_str.to_string_lossy().into_owned()
        }
    }
}

impl Drop for MagicCookie {
    fn drop(&mut self) {
        let lock = self.inner.lock().unwrap();
        unsafe { magic_close(*lock) };
    }
}
