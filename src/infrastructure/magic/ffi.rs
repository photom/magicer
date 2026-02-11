use std::os::raw::{c_char, c_int, c_void};

pub type MagicT = *mut c_void;

pub const MAGIC_NONE: c_int = 0x000000;
pub const MAGIC_MIME_TYPE: c_int = 0x000010;
pub const MAGIC_ERROR: c_int = 0x000200;

// #[link(name = "magic")]
extern "C" {
    pub fn magic_open(flags: c_int) -> MagicT;
    pub fn magic_close(ms: MagicT);
    pub fn magic_error(ms: MagicT) -> *const c_char;
    pub fn magic_load(ms: MagicT, filename: *const c_char) -> c_int;
    pub fn magic_buffer(ms: MagicT, buf: *const c_void, nb: usize) -> *const c_char;
    pub fn magic_file(ms: MagicT, filename: *const c_char) -> *const c_char;
}
