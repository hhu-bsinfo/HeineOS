/*
 * Contains functions from 'sys/stat.h'.
 *
 * Author: Fabian Ruhland, Heinrich Heine University Duesseldorf, 2026-06-18
 * License: GPLv3
 */
use core::ffi::{c_char, c_int, c_uint};
use crate::libc::errno::{set_errno, Errno};

#[unsafe(no_mangle)]
/// Create a new directory.
/// This function is not supported by our implementation.
pub(crate) unsafe extern "C" fn mkdir(_path: *const c_char, _mode: c_uint) -> c_int {
    set_errno(Errno::EACCES);
    -1
}