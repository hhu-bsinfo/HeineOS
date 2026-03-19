/*
 * Contains functions to create, start, switch and end threads.
 *
 * Author: Michael Schoettner, Heinrich Heine University Duesseldorf, 2023-05-15
 *         Fabian Ruhland, Heinrich Heine University Duesseldorf, 2026-01-15
 * License: GPLv3
 */

use alloc::boxed::Box;
use alloc::vec::Vec;
use core::{fmt, ptr};
use core::arch::naked_asm;
use core::fmt::Display;
use core::sync::atomic::AtomicUsize;
use crate::consts::STACK_SIZE;
use crate::device::cpu;
use crate::thread::scheduler::scheduler;

static THREAD_ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub fn next_id() -> usize {
    THREAD_ID_COUNTER.fetch_add(1, core::sync::atomic::Ordering::SeqCst)
}

/// Low-level routine for starting a thread.
#[unsafe(naked)]
unsafe extern "C" fn thread_start(stack_ptr: usize) {
    naked_asm!(
        // TODO: Implement assembly code for starting a thread
        "ret"
    )
}

/// Low-level routine for switching to the next thread.
/// `current_stack_ptr` is a pointer to `stack_ptr` of the next coroutine (where the rsp is saved).
/// `next_stack` is the value of `stack_ptr` of the next thread (the new rsp value).
#[unsafe(naked)]
unsafe extern "C" fn thread_switch(current_stack_ptr: *mut usize, next_stack: usize) {
    naked_asm!(
        // TODO: Implement assembly code for switching threads
        "ret"
    )
}

/// Represents a thread in the system.
/// It contains the stack and the entry function.
/// Threads must be registered in the scheduler and are run automatically once the scheduler is started.
#[repr(C)]
pub struct Thread {
    id: usize,
    stack: Vec<u64>,  // Memory for the stack
    stack_ptr: usize, // Pointer on the stack to the saved context
    entry: fn(),
}

impl Thread {
    /// Create a new thread with the given entry function.
    pub fn new(entry: fn()) -> Box<Thread> {
        // Allocate memory for the stack and initialize it to zero
        let mut stack = Vec::<u64>::with_capacity(STACK_SIZE / 8);
        for _ in 0..stack.capacity() {
            stack.push(0);
        }

        // Set the stack pointer to the top of the stack
        let stack_ptr = ptr::from_ref(&stack[stack.capacity() - 1]) as usize;

        // Create a new thread object
        let mut thread = Box::new(
            Thread { id: next_id(), stack, stack_ptr, entry }
        );

        // Prepare the stack for the thread so it can be started via `thread_start()`
        thread.prepare_stack();
        thread
    }

    /// Start the thread.
    /// This function is only once by the scheduler.
    /// The scheduler does further thread switching via `switch()`.
    pub fn start(&mut self) {
        todo!("Thread::start() is not implemented yet.");
    }

    /// Switch from the `current` thread to the `next` thread.
    /// This function is called by the scheduler to switch between threads.
    pub unsafe fn switch(current: *mut Thread, next: *mut Thread) {
        todo!("Thread::switch() is not implemented yet.");
    }

    /// Get the ID of the thread.
    pub fn id(&self) -> usize {
        self.id
    }

    /// Prepare the stack of a newly created thread in a way that it can be used
    /// to return to the 'kickoff' function with the thread itself as parameter.
    /// The prepared stack is used in 'thread_start' to start the first thread.
    /// Other threads are started by 'thread_switch' with the prepared stack.
    fn prepare_stack(&mut self) {
        let kickoff = (Thread::kickoff as *const()) as u64;
        let thread = ptr::from_mut(self) as u64;
        let length = self.stack.len();

        self.stack[length - 1] = kickoff; // Address of 'kickoff'
        self.stack[length - 2] = 0; // r8
        self.stack[length - 3] = 0; // r9
        self.stack[length - 4] = 0; // r10
        self.stack[length - 5] = 0; // r11
        self.stack[length - 6] = 0; // r12
        self.stack[length - 7] = 0; // r13
        self.stack[length - 8] = 0; // r14
        self.stack[length - 9] = 0; // r15
        self.stack[length - 10] = 0; // rax
        self.stack[length - 11] = 0; // rbx
        self.stack[length - 12] = 0; // rcx
        self.stack[length - 13] = 0; // rdx
        self.stack[length - 14] = 0; // rsi
        self.stack[length - 15] = thread; // rdi -> First parameter for 'kickoff'
        self.stack[length - 16] = 0; // rbp
        self.stack[length - 17] = 0x2; // rflags (IE = 0); interrupts disabled

        self.stack_ptr = self.stack_ptr - (size_of::<u64>() * 16);
    }

    /// Called indirectly by using the prepared stack in 'thread_start' and 'thread_switch'.
    fn kickoff(&self) {
        // Interrupts are disabled during thread start, so we need to re-enable them here
        cpu::enable_int();
        ((*self).entry)();

        scheduler().exit();
    }
}

impl Display for Thread {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "T{}", self.id)
    }
}