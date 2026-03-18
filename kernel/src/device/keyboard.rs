/*
 * CAUTION: This is not meant as a replacement for your existing keyboard.rs
 *          It just contains additional code that you should copy into your own file.
 */

/// Global key event buffer.
/// Each key is pushed to this queue by the interrupt handler and can be retrieved at a later time by the user.
/// Wrapped inside a Once, because the Queue cannot be created inside a const function.
static KEYBOARD_BUFFER: Once<KeyEventQueue> = Once::new();

/// Global access to the key buffer.
/// Usage: let key_buffer = keyboard::keyboard_buffer();
///        let key = key_buffer.pop_key_event();
pub fn keyboard_buffer() -> &'static KeyEventQueue {
    KEYBOARD_BUFFER.init(KeyEventQueue::new)
}

/// Interrupt handler struct for the keyboard.
struct KeyboardISR;

impl ISR for KeyboardISR {
    /// Keyboard interrupt handler.
    /// This function reads the next byte from the keyboard and decodes it into a key event.
    fn trigger(&self) {
        todo!("KeyboardISR::trigger() not implemented yet!");
    }
}

/// Register the keyboard interrupt handler with the interrupt dispatcher
/// and enable keyboard interrupts at the PIC.
pub fn plugin() {
    todo!("Keyboard::plugin() not implemented yet!");
}