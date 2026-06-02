/*
 * Contains the 'Hello World' test system call.
 *
 * Author: Fabian Ruhland, Heinrich Heine University Duesseldorf, 2026-05-26
 * License: GPLv3
 */

use log::info;
use crate::thread::scheduler::scheduler;

/// Print 'Hello, world' and the current thread ID to the kernel log.
pub extern "C" fn sys_hello_world() {
    info!("Hello, world (from syscall, TID: {})!", scheduler().get_active_tid());
}