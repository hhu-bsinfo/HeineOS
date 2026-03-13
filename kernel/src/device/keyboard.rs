/*
 * A driver for the PS/2 keyboard.
 *
 * Author: Michael Schoetter, Heinrich Heine University Duesseldorf, 2024-05-06
 *         Fabian Ruhland, Heinrich Heine University Duesseldorf, 2026-01-14
 * License: GPLv3
 */

use bitflags::bitflags;
use crate::device::cpu::IoPort;
use crate::device::key::{KeyEvent, KeyModifiers};
use crate::library::spinlock::Spinlock;

/// The global keyboard instance protected by a spinlock.
/// This instance can be used to poll key events from the keyboard. Process the key event
pub static KEYBOARD: Spinlock<Keyboard> = Spinlock::new(Keyboard::new());

/// Driver struct for the PS/2 keyboard.
/// The keyboard may send multiple bytes for a single key event (e.g. with modifier keys).
/// This struct works like a state machine, gathering bytes until a complete key event is decoded.
pub struct Keyboard {
    /// Current prefix byte (if any) for the next key event.
    /// Some keys send a prefix byte (0xe0 or 0xe1) before the actual scancode.
    prefix: u8,
    /// The key event that is currently being decoded.
    gather: KeyEvent,
    /// Current status of the keyboard LEDs (NumLock, CapsLock, ScrollLock).
    leds: LedStatus,
    /// I/O port for keyboard control commands and status.
    control_port: IoPort,
    /// I/O port for keyboard data (scancodes).
    data_port: IoPort
}

/// Translation table to convert scancodes to ASCII codes for normal keys (no modifiers).
/// Index is the scancode, value is the ASCII code (0 if no ASCII code).
static NORMAL_TAB: [u8;89] =
    [
        0, 0, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 225, 39, 8, 0, 113,
        119, 101, 114, 116, 122, 117, 105, 111, 112, 129, 43, 13, 0, 97,
        115, 100, 102, 103, 104, 106, 107, 108, 148, 132, 94, 0, 35, 121,
        120, 99, 118, 98, 110, 109, 44, 46, 45, 0, 42, 0, 32, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 45, 0, 0, 0, 43, 0, 0, 0, 0,
        0, 0, 0, 60, 0, 0
    ];

/// Translation table to convert scancodes to ASCII codes for keys pressed with the Shift modifier.
/// Index is the scancode, value is the ASCII code (0 if no ASCII code).
static SHIFT_TAB: [u8;89] =
    [
        0, 0, 33, 34, 21, 36, 37, 38, 47, 40, 41, 61, 63, 96, 0, 0, 81,
        87, 69, 82, 84, 90, 85, 73, 79, 80, 154, 42, 0, 0, 65, 83, 68,
        70, 71, 72, 74, 75, 76, 153, 142, 248, 0, 39, 89, 88, 67, 86, 66,
        78, 77, 59, 58, 95, 0, 0, 0, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 62, 0, 0
    ];

/// Translation table to convert scancodes to ASCII codes for keys pressed with the Alt modifier.
/// Index is the scancode, value is the ASCII code (0 if no ASCII code).
static ALT_TAB: [u8; 89] =
    [
        0, 0, 0, 253, 0, 0, 0, 0, 123, 91, 93, 125, 92, 0, 0, 0, 64, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 126, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 230, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 124, 0, 0
    ];

/// Translation table to convert scancodes to ASCII codes for the numeric keypad when NumLock is active.
/// Index is the scancode minus 71, value is the ASCII code.
static ASC_NUM_TAB:[u8; 13] = [ 55, 56, 57, 45, 52, 53, 54, 43, 49, 50, 51, 48, 44 ];

/// Translation table to convert scancodes to scancodes for the numeric keypad when NumLock is active.
/// Index is the scancode minus 71, value is the scancode.
static SCAN_NUM_TAB: [u8; 13] = [  8, 9, 10, 53, 5, 6, 7, 27, 2, 3, 4, 11, 51 ];

bitflags! {
    /// LED status flags for the keyboard.
    struct LedStatus: u8 {
        const NUM_LOCK = 1;
        const CAPS_LOCK = 2;
        const SCROLL_LOCK = 4;
    }
}

bitflags! {
    /// Status flags for the keyboard controller.
    /// These flags can be read from the keyboard control port.
    struct KeyboardStatus: u8 {
        const OUTPUT_BUFFER_FULL = 0x01;
        const INPUT_BUFFER_FULL = 0x02;
        const AUXILIARY_DEVICE = 0x20;
    }
}

#[repr(u16)]
/// I/O port addresses for the keyboard controller.
enum KeyboardRegister {
    /// Control port (status and commands).
    Control = 0x64,
    /// Data port (scancodes and data).
    Data = 0x60
}

/// Commands that can be sent to the keyboard controller.
enum KeyboardCommand {
    /// Set keyboard LED status.
    SetLed = 0xed,
    /// Set keyboard repeat rate.
    SetSpeed = 0xf3,
    /// Reset the CPU via the keyboard controller.
    CpuReset = 0xfe
}

/// Responses from the keyboard controller (we only care about ACK).
enum KeyboardResponse {
    /// Acknowledge response.
    Ack = 0xfa
}

/// Break bit in the scancode indicating key release.
const BREAK_BIT: u8 = 0x80;
/// First prefix byte for extended keys.
const PREFIX1: u8 = 0xe0;
/// Second prefix byte for certain special keys.
const PREFIX2: u8 = 0xe1;

impl Keyboard {
    /// Create a new keyboard driver instance.
    pub const fn new() -> Keyboard {
        Keyboard {
            prefix: 0,
            gather: KeyEvent::new(),
            leds: LedStatus::empty(),
            control_port: IoPort::new(KeyboardRegister::Control as u16),
            data_port: IoPort::new(KeyboardRegister::Data as u16)
        }
    }

    /// If a byte is available from the keyboard, read and decode it.
    /// If a complete key event has been decoded, it is returned.
    /// If no byte is available or the key event is not complete yet, None is returned.
    fn try_read_next_byte(&mut self) -> Option<KeyEvent> {
        todo!("keyboard::try_read_next_byte() not implemented yet");
    }

    /// Poll the keyboard for the next key event (press or release).
    /// This function blocks until a complete key event has been received and decoded.
    ///
    /// CAUTION: This function must not be used anymore, once the keyboard interrupt handler is active,
    /// because it directly reads from the keyboard controller and thus interferes with the interrupt handler.
    pub fn poll_key_event(&mut self) -> KeyEvent {
        todo!("keyboard::poll_key_event() not implemented yet");
    }

    /// Poll the keyboard for the next key press event.
    /// This function blocks until a key press event has been received and decoded,
    /// discarding any key release events.
    pub fn poll_key_press(&mut self) -> KeyEvent {
        todo!("keyboard::poll_key_press() not implemented yet");
    }

    /// Set the repeat rate of the keyboard (determined by the speed and delay).
    ///
    /// The speed determines how fast repeated keys are sent.
    /// Valid values are between 0 (very fast) and 31 (very slow).
    ///
    /// The delay determines how long a key must be pressed before the keyboard starts repeating it.
    /// Valid values are between 0 (minimum delay) and 3 (maximum delay).
    /// 0 = 250ms, 1 = 500ms, 2 = 750ms, 3 = 1000ms
    pub fn set_repeat_rate(&mut self, delay: u8, speed: u8) {
        todo!("keyboard::set_repeat_rate() not implemented yet");
    }

    /// Turn on or off the specified keyboard LED.
    fn set_led(&mut self, led: LedStatus, on: bool) {
        todo!("keyboard::set_led() not implemented yet");
    }

    /// Decode a single byte from the keyboard.
    /// If the current key event has not been fully decoded yet, the driver state is updated and false is returned.
    /// If a complete key event has been decoded, true is returned. In that case, the decoded key can be found in the `gather` field.
    fn decode_byte(&mut self, mut code: u8) -> bool {
        // Keys that are new in the MF II keyboard (compared to the old AT keyboard) send a prefix byte first.
        if code == PREFIX1 || code == PREFIX2 {
            // If the code is a prefix, store it and wait for the next byte.
            self.prefix = code;
            return false;
        }

        // Store the current prefix and clear it in the keyboard state.
        // The prefix is only valid for the next key, so we clear it after processing.
        let prefix = self.prefix;
        self.prefix = 0;

        // The break bit indicates whether a key is pressed or released.
        // If it is set, the key is released, otherwise it is pressed.
        if (code & BREAK_BIT) != 0 {
            // Break bit is set -> Key released
            // Remove the break bit to get the actual scancode.
            code &= !BREAK_BIT;

            // Check modifier keys (Shift, Alt, Control).
            // If a modifier key is released, remove it from the modifier flags.
            match code {
                42 | 54 => {
                    self.gather.remove_modifiers(KeyModifiers::SHIFT);
                },
                56 => {
                    if prefix == PREFIX1 {
                        self.gather.remove_modifiers(KeyModifiers::ALT_RIGHT);
                    } else {
                        self.gather.remove_modifiers(KeyModifiers::ALT_LEFT);
                    }
                },
                29 => {
                    if prefix == PREFIX1 {
                        self.gather.remove_modifiers(KeyModifiers::CTRL_RIGHT);
                    } else {
                        self.gather.remove_modifiers(KeyModifiers::CTRL_LEFT);
                    }
                },
                58 | 70 => {
                    // Ignore key releases of toggle keys (CapsLock, ScrollLock).
                },
                69 => {
                    if self.gather.modifiers().contains(KeyModifiers::CTRL_LEFT) { // Break Key
                        // On old keyboards, the Break function could only be reached via Ctrl+NumLock.
                        // Modern MF-II keyboards send this code combination when Break is meant.
                        // The Break key normally does not deliver an ASCII code, but looking it up does not hurt.
                        // In any case, the key is now complete.
                        Keyboard::parse_ascii_code(code, prefix, &mut self.gather);
                        self.gather.set_pressed(false);
                        return true;
                    } else { // NumLock
                        // Ignore key releases of toggle keys (NumLock).
                    }
                }
                _ => {
                    // For any other key, parse the ascii code and set the key as released.
                    // We now have parsed a complete key (release), so we return true.
                    Keyboard::parse_ascii_code(code, prefix, &mut self.gather);
                    self.gather.set_pressed(false);
                    return true;
                }
            }
        } else {
            // Break bit is not set -> Key pressed
            // Check modifier keys (Shift, Alt, Control).
            // If a modifier key is pressed, insert it into the modifier flags.
            match code {
                42 | 54 => {
                    self.gather.insert_modifiers(KeyModifiers::SHIFT);
                },
                56 => {
                    if prefix == PREFIX1 {
                        self.gather.insert_modifiers(KeyModifiers::ALT_RIGHT);
                    } else {
                        self.gather.insert_modifiers(KeyModifiers::ALT_LEFT);
                    }
                },
                29 => {
                    if prefix == PREFIX1 {
                        self.gather.insert_modifiers(KeyModifiers::CTRL_RIGHT);
                    } else {
                        self.gather.insert_modifiers(KeyModifiers::CTRL_LEFT);
                    }
                },
                58 => {
                    self.gather.toggle_modifiers(KeyModifiers::CAPS_LOCK);
                    self.set_led(LedStatus::CAPS_LOCK, self.gather.modifiers().contains(KeyModifiers::CAPS_LOCK));
                },
                70 => {
                    self.gather.toggle_modifiers(KeyModifiers::SCROLL_LOCK);
                    self.set_led(LedStatus::SCROLL_LOCK, self.gather.modifiers().contains(KeyModifiers::SCROLL_LOCK));
                },
                69 => {
                    if self.gather.modifiers().contains(KeyModifiers::CTRL_LEFT) { // Break Key
                        // On old keyboards, the Break function could only be reached via Ctrl+NumLock.
                        // Modern MF-II keyboards send this code combination when Break is meant.
                        // The Break key normally does not deliver an ASCII code, but looking it up does not hurt.
                        // In any case, the key is now complete.
                        Keyboard::parse_ascii_code(code, prefix, &mut self.gather);
                        self.gather.set_pressed(true);
                        return true;
                    } else { // NumLock
                        self.gather.toggle_modifiers(KeyModifiers::NUM_LOCK);
                        self.set_led(LedStatus::NUM_LOCK, self.gather.modifiers().contains(KeyModifiers::NUM_LOCK));
                    }
                }
                _ => {
                    // For any other key, parse the ascii code and set the key as pressed.
                    Keyboard::parse_ascii_code(code, prefix, &mut self.gather);
                    self.gather.set_pressed(true);
                    return true;
                }
            }
        }

        false
    }

    /// Parse the ASCII code for the given scancode and modifier keys.
    /// The parsed ASCII code and scancode are stored in the given `KeyEvent` struct.
    /// This function handles special cases and chooses the correct translation table
    /// based on the modifier keys set in the `KeyEvent` struct.
    fn parse_ascii_code(code: u8, prefix: u8, key: &mut KeyEvent) {
        // Choose the right table based on the modifier bits.
        // For simplicity, NumLock takes precedence over Alt, Shift and CapsLock.
        // There is no separate table for Ctrl.
        if key.modifiers().contains(KeyModifiers::NUM_LOCK) && prefix == 0 && code >= 71 && code <= 83 {
            // If NumLock is enabled and one of the keys of the separate number block (codes 0x47-0x53) is pressed,
            // the ASCII and scancodes of the corresponding number keys should be delivered instead of the scancodes of the cursor keys.
            // The keys of the cursor block (prefix == prefix1) should of course still be able to be used for cursor control.
            // By the way, they still send a shift, but that should not matter.
            key.set_ascii(ASC_NUM_TAB[(code - 71) as usize]);
            key.set_scancode(SCAN_NUM_TAB[(code - 71) as usize]);
        } else if key.modifiers().contains(KeyModifiers::ALT_RIGHT) {
            key.set_ascii(ALT_TAB[code as usize]);
            key.set_scancode(code);
        } else if key.modifiers().contains(KeyModifiers::SHIFT) {
            key.set_ascii(SHIFT_TAB[code as usize]);
            key.set_scancode(code);
        } else if key.modifiers().contains(KeyModifiers::CAPS_LOCK) {
            // CapsLock is only active for the letters A-Z and 0-9.
            if (code >= 16 && code <= 26) || (code >= 30 && code<= 40) || (code >= 44 && code <= 50) {
                key.set_ascii(SHIFT_TAB[code as usize]);
                key.set_scancode(code);
            } else {
                key.set_ascii(NORMAL_TAB[code as usize]);
                key.set_scancode(code);
            }
        } else {
            // No modifier keys active, use normal table.
            key.set_ascii(NORMAL_TAB[code as usize]);
            key.set_scancode(code);
        }
    }
}