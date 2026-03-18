/*
 * Contains a demo for heap allocations.
 *
 * Author: Michael Schoetter, Heinrich Heine University Duesseldorf
 *         Fabian Ruhland, Heinrich Heine University Duesseldorf, 2026-01-14
 * License: GPLv3
 */

use alloc::boxed::Box;
use alloc::vec::Vec;
use crate::allocator;
use crate::device::key::Scancode;
use crate::device::keyboard::KEYBOARD;
use crate::device::speaker;
use crate::device::speaker::SPEAKER;
use crate::device::terminal::terminal;

/// A simple heap demo, allocating and freeing memory on the heap.
/// The allocator state is dumped before and after each operation.
pub fn heap_demo() {
    todo!("lesson2::heap_demo() is not implemented yet.")
}

/// A demo that plays songs via the PC speaker.
pub fn speaker_demo() {
    todo!("lesson2::speaker_demo() is not implemented yet.")
}