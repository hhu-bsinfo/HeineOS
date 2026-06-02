/*
 * Module: user_api
 *
 * Description: All system calls available to user programs are defined in this module.
 *
 * Author: Stefan Lankes, RWTH Aachen University
 *         Licensed under the Apache License, Version 2.0 or MIT license, at your option.
 *
 *         Michael Schoettner, Heinrich Heine University Duesseldorf, 14.09.2023
 *         Fabian Ruhland, Heinrich Heine University Duesseldorf, 15.10.2025
 */

use core::arch::asm;

/// System call numbers available to user programs.
#[repr(u64)]
pub enum SyscallFunction {
    HelloWorld,
    NumSyscalls // Last entry to count number of syscalls
}

/// Test system call printing "Hello, World!" to the serial console.
pub fn usr_hello_world() {
    unsafe { syscall0(SyscallFunction::HelloWorld); }
}

// TODO: Implement further system calls

/// Perform a system call with 0 parameters.
#[inline(always)]
pub unsafe fn syscall0(syscall: SyscallFunction) -> u64 {
    let mut ret: u64;
    unsafe {
        asm!(
        "int 0x80",
        inlateout("rax") syscall as u64 => ret,
        options(preserves_flags, nostack)
        );
    }
    ret
}
// TODO: Implement system call wrapper functions with parameters