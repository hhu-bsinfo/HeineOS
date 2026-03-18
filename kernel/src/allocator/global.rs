/*
 * Global heap allocator functions used by the Rust compiler for dynamic memory allocation.
 *
 * A QEMU VM with OVMF and HeineOS booted via towboot roughly has the following memory layout:
 *   0KB to 640KB: free
 * 640KB to 1MB:   devices
 *   1MB to 8MB:   free
 *   8MB to 9MB:   ACPI
 *   9MB to 24MB:  Boot Services
 *   25M: Our kernel binary is loaded here by the bootloader
 *   ...: Free memory, that we can use for the heap
 *   ...: The initial ramdisk is loaded near the end of physical memory
 *        (we cannot rely on this assumption, which is why we will use the UEFI memory map later)
 *
 * Author: Philipp Oppermann, https://os.phil-opp.com/allocator-designs/
 *         Fabian Ruhland, Heinrich Heine University Duesseldorf, 2026-01-13
 */

use alloc::alloc::Layout;
use crate::allocator::list::BumpAllocator;
use crate::library::spinlock::{Spinlock, SpinlockGuard};

#[global_allocator]
/// Global heap allocator instance, used by the Rust compiler for dynamic memory allocation.
static ALLOCATOR: Locked<BumpAllocator> = Locked::new(BumpAllocator::new());

/// Helper function used in `bump.rs` and `list.rs`. Rust requires pointers to be aligned.
pub fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}

/// Check if the heap allocator is currently locked.
pub fn is_allocator_locked() -> bool {
    ALLOCATOR.inner.is_locked()
}

/// Initialize the heap allocator.
pub fn init_allocator(heap_start: usize, heap_size: usize) {
    unsafe {
        ALLOCATOR.lock().init(heap_start, heap_size);
    }
}

/// Allocates memory from the heap. Compiler generates code calling this function.
pub fn alloc(layout: Layout) -> *mut u8 {
    unsafe {
        ALLOCATOR.lock().alloc(layout)
    }
}

/// Deallocates memory from the heap. Compiler generates code calling this function.
pub fn dealloc(ptr: *mut u8, layout: Layout) {
    unsafe {
        ALLOCATOR.lock().dealloc(ptr, layout)
    }
}

/// Dump heap free list. Must be called by own program.
/// Can be used for debugging the heap allocator.
pub fn dump_free_list() {
    ALLOCATOR.lock().dump_free_list();
}

/// A wrapper around `Spinlock` to allow for trait implementations.
/// Required for implementing `GlobalAlloc` in `bump.rs` and `list.rs`.
pub struct Locked<A> {
    inner: Spinlock<A>,
}

impl<A> Locked<A> {
    /// Create a new `Locked` instance wrapping the given inner value.
    pub const fn new(inner: A) -> Self {
        Locked {
            inner: Spinlock::new(inner),
        }
    }

    /// Lock the inner value and return a guard for it.
    pub fn lock(&self) -> SpinlockGuard<'_, A> {
        self.inner.lock()
    }
}
