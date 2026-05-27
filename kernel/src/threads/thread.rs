/*
 * CAUTION: This is not meant as a replacement for your existing thread.rs
 *          It just contains additional code that you should copy into your own file.
 */

/// Low-level routine for switching to the next thread.
/// `current_stack_ptr` is a pointer to `stack_ptr` of the next coroutine (where the rsp is saved).
/// `next_stack` is the value of `stack_ptr` of the next thread (the new rsp value).
/// `next_stack_end` points to the top of the next thread's stack (used to set the rsp0 entry of the TSS)
#[unsafe(naked)]
unsafe extern "C" fn thread_switch(current_stack_ptr: *mut usize, next_stack: usize, next_stack_end: usize) {
    // TODO: Add the third parameter `next_stack_end` to your existing implementation
    //       and call `tss_set_rsp0()` at the right position
}

#[unsafe(naked)]
/// Called by `switch_to_usermode()`.
/// Switches to the prepared user stack and uses `iret` to return to ring 3 code.
unsafe extern "C" fn thread_user_start(stack_ptr: usize) {
    naked_asm!(
        "mov rsp, rdi", // Switch stack
        "pop rdi",
        "iretq" // Return to user mode
    )
}

/// Represents a thread in the system.
/// It contains the stack and the entry function.
/// Threads must be registered in the scheduler and are run automatically once the scheduler is started.
#[repr(C)]
pub struct Thread {
    id: usize,
    is_kernel_thread: bool,
    kernel_stack: Vec<u64>, // Memory for the kernel stack
    user_stack: Vec<u64>, // Memory for the user stack
    stack_ptr: usize, // Pointer on the stack to the saved context
    entry: fn(),
}

impl Thread {
    /// Create a new user thread with the given entry function.
    pub fn new_user_thread(entry: fn()) -> Box<Thread> {
        todo!("Thread::new_user_thread() is not implemented yet")
    }

    /// Switch this thread from Ring 0 to Ring 3.
    /// For this, the kernel stack is prepared in a way that an 'iretq' instruction
    /// switches to user mode (Ring 3) and the user stack is used. If this function works correctly,
    /// the thread continues in user mode in the function 'kickoff_user_thread'.
    fn switch_to_usermode(&mut self) {
        todo!("Thread::switch_to_usermode() is not implemented yet")
    }

    /// Called indirectly by using the prepared stack in 'thread_start' and 'thread_switch'.
    fn kickoff_kernel_thread(&mut self) {
        // Set TSS rsp0 to the top of the kernel stack of this thread
        unsafe {
            let rsp0 = Self::get_top_of_stack(&self.kernel_stack);
            tss_set_rsp0(rsp0 as usize);
        }

        if self.is_kernel_thread {
            cpu::enable_int(); // Interrupts are disabled during thread start
            ((*self).entry)();
        } else {
            self.switch_to_usermode();
        }

        scheduler().exit();
    }

    /// Called indirectly by using the prepared stack in 'switch_to_usermode'.
    fn kickoff_user_thread(&self) {
        todo!("Thread::kickoff_user_thread() is not implemented yet");
        loop {} // User threads may currently not exit
    }

    /// Get a pointer to the top of the given stack.
    fn get_top_of_stack(stack: &Vec<u64>) -> *const u64 {
        unsafe {
            ptr::from_ref(&stack[stack.len() - 1]).offset(1)
        }
    }
}
