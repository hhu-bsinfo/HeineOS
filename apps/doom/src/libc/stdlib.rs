/*
 * Contains functions from 'stdlib.h'.
 *
 * Author: Fabian Ruhland, Heinrich Heine University Duesseldorf, 2026-06-18
 * License: GPLv3
 */
use alloc::collections::BTreeMap;
use core::alloc::Layout;
use core::cmp::min;
use core::ffi::{c_char, c_double, c_int, c_size_t, c_void, CStr};
use core::ptr;
use core::str::FromStr;
use usrlib::allocator;
use usrlib::spinlock::Spinlock;
use usrlib::user_api::usr_thread_exit;

/// A map to keep track of all allocations.
/// This is needed because the C standard `free()` function takes a pointer as its only parameter.
/// However, the Rust `dealloc()` function requires an additional layout parameter, providing size and alignment.
/// This map uses pointers of allocated memory blocks as keys and the block sizes as values.
/// This way, the layout parameter for deallocations can be reconstructed.
static ALLOCATIONS: Spinlock<BTreeMap<u64, c_size_t>> = Spinlock::new(BTreeMap::new());

#[unsafe(no_mangle)]
/// Allocate memory of the given size.
pub(crate) unsafe extern "C" fn malloc(size: c_size_t) -> *mut c_void {
    let layout = Layout::from_size_align(size, 8).unwrap();
    let ptr = allocator::global::alloc(layout);

    ALLOCATIONS.lock().insert(ptr as u64, size);

    ptr as *mut c_void
}

#[unsafe(no_mangle)]
/// Allocate memory of the given (size * num) and initialize it with zeros.
pub(crate) unsafe extern "C" fn calloc(num: c_size_t, size: c_size_t) -> *mut c_void {
    unsafe {
        let ptr = malloc(num * size);
        ptr::write_bytes(ptr as *mut u8, 0, num * size);

        ptr
    }
}

#[unsafe(no_mangle)]
/// Reallocate the memory of the given pointer to the new size.
/// If the pointer is NULL, a new block of memory of the given size is allocated.
/// If the pointer is not NULL, the old memory is copied to the new block and the old block is freed.
/// If the new size is smaller than the old size, the old memory is truncated.
/// If the new size is larger than the old size, the new memory is zero-filled.
/// In our implementation, we always allocate new memory and do not try to resize the exisiting block.
pub(crate) unsafe extern "C" fn realloc(ptr: *mut c_void, new_size: c_size_t) -> *mut c_void {
    unsafe {
        let new_ptr = malloc(new_size);
        if ptr.is_null() {
            return new_ptr;
        }

        let old_size = *ALLOCATIONS.lock()
            .get(&(ptr as u64))
            .expect("realloc: Invalid pointer");

        let copy_size = min(old_size, new_size);
        ptr::copy_nonoverlapping(ptr as *const u8, new_ptr as *mut u8, copy_size);
        free(ptr);

        new_ptr
    }
}

#[unsafe(no_mangle)]
/// Free the memory block pointed to by the given pointer.
/// This function does nothing if the pointer is NULL.
pub(crate) unsafe extern "C" fn free(ptr: *mut c_void) {
    if ptr.is_null() {
        return;
    }

    let size = ALLOCATIONS.lock()
        .remove(&(ptr as u64))
        .expect("free: Invalid pointer");

    let layout = Layout::from_size_align(size, 8).unwrap();
    allocator::global::dealloc(ptr as *mut u8, layout);
}

#[unsafe(no_mangle)]
/// Calculate the absolute value of an integer.
pub(crate) unsafe extern "C" fn abs(n: c_int) -> c_int {
    n.abs()
}

#[unsafe(no_mangle)]
/// Parse a floating point value from a string.
pub(crate) unsafe extern "C" fn atof(str: *const c_char) -> c_double {
    // Closure to determine if a character is invalid for atof.
    let invalid_char = |c: char| {
        !c.is_digit(10) && c != '+' && c != '-' && c != '.' && c != 'e' && c != 'E'
    };

    unsafe {
        let num_str = CStr::from_ptr(str)
            // Convert C string to Rust string (default to "" if conversion fails)
            .to_str()
            .unwrap_or("")
            // Remove leading whitespace characters
            .trim_start()
            // Remove invalid trailing characters
            .split(invalid_char)
            .next()
            .unwrap();

        // Parse the number, defaulting to 0.0 if parsing fails
        c_double::from_str(num_str).unwrap_or(0.0)
    }
}

#[unsafe(no_mangle)]
/// Parse an integer value from a string.
pub(crate) unsafe extern "C" fn atoi(str: *const c_char) -> c_int {
    // Closure to determine if a character is invalid for atoi.
    let invalid_char = |c : char| {
        !c.is_digit(10) && c != '+' && c != '-'
    };

    unsafe {
        let num_str = CStr::from_ptr(str)
            // Convert C string to Rust string (default to "" if conversion fails)
            .to_str()
            .unwrap_or("")
            // Remove leading whitespace characters
            .trim_start()
            // Remove invalid trailing characters
            .split(invalid_char)
            .next()
            .unwrap();

        // Parse the number, defaulting to 0 if parsing fails
        c_int::from_str(num_str).unwrap_or(0)
    }
}

#[unsafe(no_mangle)]
/// Execute a command.
/// This function is not supported by our implementation.
pub(crate) unsafe extern "C" fn system(_command: *const c_char) -> c_int {
    -1
}

#[unsafe(no_mangle)]
/// Exit the application.
pub(crate) unsafe extern "C" fn exit(_exit_code: c_int) {
    usr_thread_exit();
}
