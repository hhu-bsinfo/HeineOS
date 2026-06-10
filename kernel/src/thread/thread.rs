/*
 * CAUTION: This is not meant as a replacement for your existing thread.rs
 *          It just contains additional code that you should copy into your own file.
 */

#[unsafe(naked)]
unsafe extern "C" fn thread_switch(current_stack_ptr: *mut usize, next_stack: usize, next_stack_end: usize, next_pml4: usize) {}

#[repr(C)]
pub struct Thread {
    id: usize,
    is_kernel_thread: bool,
    pml4_table: &'static PageTable,
    kernel_stack: Vec<u64>, // Memory for the kernel stack
    user_stack: Option<Vec<u64, NoOpAllocator>>, // Memory for the user stack
    stack_ptr: usize, // Pointer on the stack to the saved context
    entry: fn(),
}

impl Thread {
    /// Get a pointer to the top of the given stack.
    fn get_top_of_stack<A: Allocator>(stack: &Vec<u64, A>) -> *const u64 {
        unsafe {
            ptr::from_ref(&stack[stack.len() - 1]).offset(1)
        }
    }
}
