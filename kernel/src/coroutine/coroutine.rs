/*
 * Contains functions to create, start, switch and end coroutines.
 *
 * Author: Michael Schoettner, Heinrich Heine University Duesseldorf, 2023-05-15
 *         Fabian Ruhland, Heinrich Heine University Duesseldorf, 2026-01-15
 * License: GPLv3
 */

use alloc::boxed::Box;
use alloc::vec::Vec;
use core::arch::naked_asm;
use core::ptr;
use core::sync::atomic::{AtomicUsize, Ordering};
use crate::consts;
use crate::device::cpu;

/// Atomic counter for coroutine ids.
static COROUTINE_ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// Generate a new coroutine id by incrementing the counter.
fn next_id() -> usize {
    COROUTINE_ID_COUNTER.fetch_add(1, Ordering::SeqCst)
}

/// Low-level routine for starting a coroutine.
#[unsafe(naked)]
unsafe extern "C" fn coroutine_start(stack_ptr: usize) {
    naked_asm!(
        // TODO: Implement assembly code for starting a coroutine
        "ret"
    )
}

/// Low-level routine for switching to the next coroutine.
/// `current_stack_ptr` is a pointer to `stack_ptr` of the current coroutine (where the rsp is saved).
/// `next_stack` is the value of `stack_ptr` of the next coroutine (the new rsp value).
#[unsafe(naked)]
unsafe extern "C" fn coroutine_switch(current_stack_ptr: *mut usize, next_stack: usize) {
    naked_asm!(
        // TODO: Implement assembly code for switching coroutines
        "ret"
    )
}

/// Represents a coroutine in the system.
/// It contains the stack, the entry function, and a pointer to the next coroutine.
/// Coroutines must be chained via `set_next()` and form a circular linked list.
/// To start the coroutine, use `start()`. Once started, coroutines cannot be exited
/// and the entry function must not return.
pub struct Coroutine {
    id: usize,
    stack: Vec<u64>,  // Memory for the stack
    stack_ptr: usize, // Pointer on the stack to the saved context
    entry: fn(&mut Coroutine),
    next: *mut Coroutine,
}

impl Coroutine {
    /// Create a new coroutine with the given entry function.
    pub fn new(entry: fn(&mut Coroutine)) -> Box<Coroutine> {
        let mut stack = Vec::<u64>::with_capacity(consts::STACK_SIZE / 8);
        for _ in 0..stack.capacity() {
            stack.push(0);
        }

        let stack_ptr = ptr::from_ref(&stack[stack.capacity() - 1]) as usize;

        let mut coroutine = Box::new(
            Coroutine { id: next_id(), stack, stack_ptr, entry, next: ptr::null_mut() }
        );

        coroutine.prepare_stack();
        coroutine
    }

    /// Start the coroutine.
    /// Once started, coroutines cannot be exited.
    /// May only be called once.
    pub fn start(&mut self) {
        todo!("Coroutine::start() is not implemented yet.");
    }

    /// Switch to the next coroutine.
    pub fn switch(&mut self) {
        todo!("Coroutine::switch() is not implemented yet.");
    }

    /// Get the id of the coroutine.
    pub fn id(&self) -> usize {
        self.id
    }

    /// Set the next coroutine.
    pub fn set_next(&mut self, next: &mut Coroutine) {
        todo!("Coroutine::set_next() is not implemented yet.");
    }

    /// Prepare the stack of a newly created coroutine in a way that it can be used
    /// to return to the 'kickoff' function with the coroutine itself as parameter.
    /// The prepared stack is used in 'coroutine_start' to start the first coroutine.
    /// Other coroutines are started by 'coroutine_switch' with the prepared stack.
    fn prepare_stack(&mut self) {
        let kickoff = (Coroutine::kickoff as *const ()) as u64;
        let coroutine = ptr::from_mut(self) as u64;
        let length = self.stack.len();

        self.stack[length - 1] = kickoff; // Address of 'kickoff' -> Used as return address
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
        self.stack[length - 15] = coroutine; // rdi -> First parameter for 'kickoff'
        self.stack[length - 16] = 0; // rbp
        self.stack[length - 17] = 0x2; // rflags (IE = 0); interrupts disabled

        self.stack_ptr = self.stack_ptr - (size_of::<u64>() * 16);
    }

    /// Called indirectly by using the prepared stack in 'coroutine_start' and 'coroutine_switch'.
    fn kickoff(&mut self) {
        // Interrupts are disabled during coroutine start, so we need to re-enable them here
        cpu::enable_int();
        (self.entry)(self);

        panic!("Coroutine {} finished!", self.id());
    }
}
