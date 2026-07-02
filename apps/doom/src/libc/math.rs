/*
 * Contains functions from 'math.h'.
 *
 * Author: Fabian Ruhland, Heinrich Heine University Duesseldorf, 2026-06-18
 * License: GPLv3
 */

use core::ffi::c_double;

#[unsafe(no_mangle)]
/// Return the absolute value of a double.
pub(crate) unsafe extern "C" fn fabs(a: c_double) -> c_double {
    todo!("fabs() has not been implemented yet!");   
}