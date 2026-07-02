/*
 * Contains functions from 'ctype.h'.
 *
 * Author: Fabian Ruhland, Heinrich Heine University Duesseldorf, 2026-06-18
 * License: GPLv3
 */
use core::ffi::c_int;

#[unsafe(no_mangle)]
/// Return 1 if the character is a whitespace character, 0 otherwise.
pub(crate) unsafe extern "C" fn isspace(c: c_int) -> c_int {
    todo!("isspace() has not been implemented yet!");
}

#[unsafe(no_mangle)]
/// Convert a character to uppercase.
/// If the character has no uppercase representation, the given character itself is returned.
pub(crate) unsafe extern "C" fn toupper(c: c_int) -> c_int {
    todo!("toupper() has not been implemented yet!");
}