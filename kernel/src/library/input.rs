/*
 * Utility functions for reading text input from the keyboard.
 *
 * Author: Michael Schoetter, Heinrich Heine University Duesseldorf, 2024-05-06
 *         Fabian Ruhland, Heinrich Heine University Duesseldorf, 2026-01-14
 * License: GPLv3
 */

use crate::device::keyboard::keyboard_buffer;

/// Wait for a key press and return the corresponding ASCII character.
/// If the key pressed does not correspond to an ASCII character (e.g., function keys),
/// the function will continue to wait until a valid ASCII character is pressed.
pub fn read_char() -> char {
    todo!("input::read_char() not implemented yet!");
}

/// Wait until the 'Return' (Enter) key is pressed.
pub fn wait_for_return() {
    while read_char() != '\r' {}
}