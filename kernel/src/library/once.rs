/*
 * An implementation of a simple "Once" type for one-time initialization.
 * A "Once" allows you to initialize a value exactly once, and then access it safely afterward.
 * This is useful for lazy initialization of global or static data.
 *
 * Author: Fabian Ruhland, Heinrich Heine University Duesseldorf, 2026-01-07
 * License: GPLv3
 */

use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use core::sync::atomic::{AtomicBool, Ordering};

/// A simple "Once" type for one-time initialization.
/// It allows you to initialize a value exactly once, and then access it safely afterward.
pub struct Once<T> {
    /// Indicates whether the value has been initialized.
    /// Since it is atomic, it can be safely accessed from multiple threads.
    /// Even if multiple threads call `init` simultaneously, only one will succeed in initializing the value.
    initialized: AtomicBool,
    /// The value to be initialized, stored in an `UnsafeCell` to allow mutable access.
    /// We use `MaybeUninit` to represent an uninitialized value.
    value: UnsafeCell<MaybeUninit<T>>,
}

unsafe impl<T> Send for Once<T> {}
unsafe impl<T> Sync for Once<T> {}

impl<T> Once<T> {
    pub const fn new() -> Self {
        Once {
            initialized: AtomicBool::new(false),
            value: UnsafeCell::new(MaybeUninit::uninit())
        }
    }

    pub fn get(&self) -> Option<&T> {
        // Check if the value has been initialized
        match self.initialized.load(Ordering::Acquire) {
            true => unsafe { Some(self.value.as_ref_unchecked().assume_init_ref()) },
            false => None // Value is not initialized yet
        }
    }

    pub fn init<F>(&self, init: F) -> &T where F: FnOnce() -> T {
        // Set the initialized flag to true atomically using `swap()`.
        // This returns the previous value, so we can check if it was already initialized.
        // If the previous value was false, we proceed with initialization.
        if !self.initialized.swap(true, Ordering::Acquire) {
            // Previous value of `initialized` was false, so this is the first time `init()` is called.
            // We can safely initialize the value now.
            unsafe { self.value.get().write(MaybeUninit::new(init())); }
        }

        // Return a reference to the initialized value.
        unsafe { self.value.as_ref_unchecked().assume_init_ref() }
    }
}