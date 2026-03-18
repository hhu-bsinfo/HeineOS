use core::ffi::c_void;
use core::ptr;
use crate::allocator::global::align_up;

unsafe extern "C" {
    /// A symbol defined in the linker script, marking the start of the kernel data segment.
    static ___KERNEL_DATA_START__: c_void;
    /// A symbol defined in the linker script, marking the end of the kernel data segment.
    static ___KERNEL_DATA_END__: c_void;
}

/// Get the start address of the kernel data segment.
/// This works by using a pointer to `___KERNEL_DATA_START__` defined in the linker script and casting it to a usize.
pub fn kernel_start() -> usize {
    unsafe { ptr::from_ref(&___KERNEL_DATA_START__) as usize }
}

/// Get the end address of the kernel data segment.
/// This works by using a pointer to `___KERNEL_DATA_END__` defined in the linker script and casting it to a usize.
pub fn kernel_end() -> usize {
    unsafe { ptr::from_ref(&___KERNEL_DATA_END__) as usize }
}

/// Get the start address of the kernel heap.
/// This is calculated by aligning the end of the kernel data segment to the next page boundary.
/// This is technically not safe, since there could be other data after the kernel end symbol (e.g. the initial ramdisk),
/// but it works for in QEMU when booted via towboot.
/// A more sophisticated approach at managing physical memory will be implemented late on in the development of the OS.
pub fn heap_start() -> usize {
    align_up(kernel_end(), PAGE_SIZE)
}

/// Size of a memory page in bytes (4 KiB).
pub const PAGE_SIZE: usize = 4096;

/// Size of the kernel heap in bytes (16 MiB).
pub const HEAP_SIZE: usize = 16 * 1024 * 1024;

/// Stack size for coroutines and threads in bytes (512 KiB).
pub const STACK_SIZE: usize = 512 * 1024;