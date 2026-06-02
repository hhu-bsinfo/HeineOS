/*
 * Contains a demo for testing system calls.
 *
 * Author: Fabian Ruhland, Heinrich Heine University Duesseldorf, 2026-05-28
 * License: GPLv3
 */

use crate::syscall::user_api::usr_hello_world;
use crate::thread::scheduler::scheduler;
use crate::thread::thread::Thread;

/// Start the syscall demo.
/// It reads keyboard input and prints to the terminal from a user thread.
pub fn syscall_demo() {
    let thread = Thread::new_user_thread(syscall_test_thread);
    let scheduler = scheduler();

    scheduler.ready(thread);
    scheduler.schedule();
}

/// Thread function for the syscall demo.
/// It reads a line from the keyboard, and once the user presses 'Enter',
/// the given line and the current system time are printed to the terminal.
fn syscall_test_thread() {
    usr_hello_world();

    loop {
        // TODO: Extend demo to show more system calls.
    }
}