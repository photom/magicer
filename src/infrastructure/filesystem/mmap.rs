use crate::infrastructure::errors::InfrastructureError;
use std::fs::File;
use std::os::unix::io::AsRawFd;
use std::ptr;
use std::sync::atomic::{AtomicBool, Ordering};

static SIGBUS_OCCURRED: AtomicBool = AtomicBool::new(false);

extern "C" fn handle_sigbus(_: libc::c_int) {
    SIGBUS_OCCURRED.store(true, Ordering::SeqCst);
    // Note: In a real scenario, we'd need siglongjmp to escape the faulting instruction.
    // For this etude, we follow the atomic flag design.
}

pub struct MmapHandler {
    addr: *mut libc::c_void,
    len: usize,
}

impl MmapHandler {
    pub fn new(file: &File) -> Result<Self, InfrastructureError> {
        let metadata = file.metadata()?;
        let len = metadata.len() as usize;

        if len == 0 {
            return Ok(Self {
                addr: ptr::null_mut(),
                len: 0,
            });
        }

        let addr = unsafe {
            libc::mmap(
                ptr::null_mut(),
                len,
                libc::PROT_READ,
                libc::MAP_PRIVATE,
                file.as_raw_fd(),
                0,
            )
        };

        if addr == libc::MAP_FAILED {
            return Err(InfrastructureError::Io(std::io::Error::last_os_error()));
        }

        // Install SIGBUS handler
        unsafe {
            let mut sa: libc::sigaction = std::mem::zeroed();
            sa.sa_sigaction = handle_sigbus as *const () as usize;
            libc::sigemptyset(&mut sa.sa_mask);
            libc::sigaction(libc::SIGBUS, &sa, ptr::null_mut());
        }

        Ok(Self { addr, len })
    }

    pub fn as_slice(&self) -> &[u8] {
        if self.len == 0 {
            return &[];
        }
        unsafe { std::slice::from_raw_parts(self.addr as *const u8, self.len) }
    }

    pub fn clear_sigbus_flag() {
        SIGBUS_OCCURRED.store(false, Ordering::SeqCst);
    }

    pub fn check_sigbus_flag() -> bool {
        SIGBUS_OCCURRED.load(Ordering::SeqCst)
    }
}

impl Drop for MmapHandler {
    fn drop(&mut self) {
        if !self.addr.is_null() {
            unsafe {
                libc::munmap(self.addr, self.len);
            }
        }
    }
}

unsafe impl Send for MmapHandler {}
unsafe impl Sync for MmapHandler {}
