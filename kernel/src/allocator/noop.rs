use core::alloc::{AllocError, Allocator, Layout};
use core::ptr::NonNull;
use log::debug;

/// An allocator that does nothing.
/// Trying to allocate memory always return an error result.
/// Deallocating memory has no effect.
/// It is used as allocator for user stacks, which are created using `Vec::from_raw_parts_in()`.
/// When they are dropped, the `Vec`'s memory is freed. However, since user stacks
/// have a fixed virtual address, the memory cannot be freed to the kernel heap.
/// Instead, we use this allocator, leaving the memory untouched.
/// The actual freeing is done later by deallocating the physical memory occupied by the address space.
pub struct NoOpAllocator;

impl NoOpAllocator {
    /// Create a new NoOpAllocator.
    pub const fn new() -> Self {
        NoOpAllocator {}
    }
}

unsafe impl Allocator for NoOpAllocator {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        Err(AllocError)
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        debug!("NullAllocator: Deallocating {:p} with layout {:?} (ignored)", ptr.as_ptr(), layout);
    }
}
