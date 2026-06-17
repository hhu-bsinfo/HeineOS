/*
 * A simple application to test user space functionality.
 *
 * Author: Fabian Ruhland, Heinrich Heine University Duesseldorf, 2026-06-17
 * License: GPLv3
 */
#![no_std]

use core::panic::PanicInfo;
use usrlib::user_api::{usr_hello_world, usr_thread_exit};
use usrlib::println;

#[unsafe(link_section = ".main")]
#[unsafe(no_mangle)]
fn main() {
    usr_hello_world();

    usr_thread_exit();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    usr_thread_exit();
}
