/*
 * Defines a `KeyEvent` struct, consisting of an ascii- and scan-code as well as modifiers.
 * Additionally, it provides helper functions for manipulating the modifiers.
 *
 * Author: Michael Schoetter, Heinrich Heine University Duesseldorf, 2024-02-06
 *         Fabian Ruhland, Heinrich Heine University Duesseldorf, 2026-01-14
 * License: GPLv3
 */

use bitflags::bitflags;

bitflags! {
    #[derive(Copy, Clone, Debug)]
    /// Key modifier bitflags.
    /// The modifier field in the `Key` struct can be a combination of these flags.
    pub struct KeyModifiers: u8 {
        const SHIFT = 1;
        const ALT_LEFT = 2;
        const ALT_RIGHT = 4;
        const CTRL_LEFT = 8;
        const CTRL_RIGHT = 16;
        const CAPS_LOCK = 32;
        const NUM_LOCK = 64;
        const SCROLL_LOCK = 128;
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
/// Key scancodes.
pub enum Scancode {
    Escape = 0x01,
    One = 0x02,
    Two = 0x03,
    Three = 0x04,
    Four = 0x05,
    Five = 0x06,
    Six = 0x07,
    Seven = 0x08,
    Eight = 0x09,
    Nine = 0x0a,
    Zero = 0x0b,
    Q = 0x10,
    W = 0x11,
    E = 0x12,
    R = 0x13,
    T = 0x14,
    Z = 0x15,
    U = 0x16,
    I = 0x17,
    O = 0x18,
    P = 0x19,
    A = 0x1e,
    S = 0x1f,
    D = 0x20,
    F = 0x21,
    G = 0x22,
    H = 0x23,
    J = 0x24,
    K = 0x25,
    L = 0x26,
    Y = 0x2c,
    X = 0x2d,
    C = 0x2e,
    V = 0x2f,
    B = 0x30,
    N = 0x31,
    M = 0x32,
    Plus = 0x1b,
    Minus = 0x35,
    Enter = 0x1c,
    Space = 0x39,
    Up = 0x48,
    Down = 0x50,
    Left = 0x4b,
    Right = 0x4d,
    Home = 0x47,
    End = 0x4f,
    PageUp = 0x49,
    PageDown = 0x51,
    Insert = 0x52,
    Del = 0x53,
    Tab = 0x0f,
    Backspace = 0x0e,
    F1 = 0x3b,
    F2 = 0x3c,
    F3 = 0x3d,
    F4 = 0x3e,
    F5 = 0x3f,
    F6 = 0x40,
    F7 = 0x41,
    F8 = 0x42,
    F9 = 0x43,
    F10 = 0x44,
    F11 = 0x57,
    F12 = 0x58,
}

impl TryFrom<u8> for Scancode {
    type Error = ();

    /// Convert a u8 value to a Scancode enum variant.
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x01 => Ok(Scancode::Escape),
            0x02 => Ok(Scancode::One),
            0x03 => Ok(Scancode::Two),
            0x04 => Ok(Scancode::Three),
            0x05 => Ok(Scancode::Four),
            0x06 => Ok(Scancode::Five),
            0x07 => Ok(Scancode::Six),
            0x08 => Ok(Scancode::Seven),
            0x09 => Ok(Scancode::Eight),
            0x0a => Ok(Scancode::Nine),
            0x0b => Ok(Scancode::Zero),
            0x0f => Ok(Scancode::Tab),
            0x0e => Ok(Scancode::Backspace),
            0x10 => Ok(Scancode::Q),
            0x11 => Ok(Scancode::W),
            0x12 => Ok(Scancode::E),
            0x13 => Ok(Scancode::R),
            0x14 => Ok(Scancode::T),
            0x15 => Ok(Scancode::Z),
            0x16 => Ok(Scancode::U),
            0x17 => Ok(Scancode::I),
            0x18 => Ok(Scancode::O),
            0x19 => Ok(Scancode::P),
            0x1b => Ok(Scancode::Plus),
            0x1c => Ok(Scancode::Enter),
            0x1e => Ok(Scancode::A),
            0x1f => Ok(Scancode::S),
            0x20 => Ok(Scancode::D),
            0x21 => Ok(Scancode::F),
            0x22 => Ok(Scancode::G),
            0x23 => Ok(Scancode::H),
            0x24 => Ok(Scancode::J),
            0x25 => Ok(Scancode::K),
            0x26 => Ok(Scancode::L),
            0x2c => Ok(Scancode::Y),
            0x2d => Ok(Scancode::X),
            0x2e => Ok(Scancode::C),
            0x2f => Ok(Scancode::V),
            0x30 => Ok(Scancode::B),
            0x31 => Ok(Scancode::N),
            0x32 => Ok(Scancode::M),
            0x35 => Ok(Scancode::Minus),
            0x39 => Ok(Scancode::Space),
            0x47 => Ok(Scancode::Home),
            0x48 => Ok(Scancode::Up),
            0x4b => Ok(Scancode::Left),
            0x4d => Ok(Scancode::Right),
            0x4f => Ok(Scancode::End),
            0x49 => Ok(Scancode::PageUp),
            0x50 => Ok(Scancode::Down),
            0x51 => Ok(Scancode::PageDown),
            0x52 => Ok(Scancode::Insert),
            0x53 => Ok(Scancode::Del),
            0x3b => Ok(Scancode::F1),
            0x3c => Ok(Scancode::F2),
            0x3d => Ok(Scancode::F3),
            0x3e => Ok(Scancode::F4),
            0x3f => Ok(Scancode::F5),
            0x40 => Ok(Scancode::F6),
            0x41 => Ok(Scancode::F7),
            0x42 => Ok(Scancode::F8),
            0x43 => Ok(Scancode::F9),
            0x44 => Ok(Scancode::F10),
            0x57 => Ok(Scancode::F11),
            0x58 => Ok(Scancode::F12),
            _ => Err(()),
        }
    }
}

/// Struct representing a key event (press or release of a key).
#[derive(Copy, Clone, Debug)]
pub struct KeyEvent {
    /// ASCII character (may be None for non-printable keys)
    ascii: Option<char>,
    /// Scancode of the key (see `Scancode` enum)
    scancode: Option<Scancode>,
    /// Combination of `KeyModifier` flags
    modifiers: KeyModifiers, // Modifier
    /// Whether the key is pressed (true) or released (false)
    pressed: bool
}

impl KeyEvent {
    /// Create a new key event with the given ASCII code, scancode and modifiers.
    pub const fn new() -> KeyEvent {
        KeyEvent { ascii : None, scancode: None, modifiers: KeyModifiers::empty(), pressed: false }
    }

    /// Set the ascii code to the given value.
    /// The given byte is converted to a char. If the conversion fails, it is set to None.
    pub fn set_ascii(&mut self, ascii: u8) {
        if ascii != 0 {
            self.ascii = char::from_u32(ascii as u32);
        }
    }

    /// Set the scancode to the given value.
    pub fn set_scancode(&mut self, scancode: u8) {
        self.scancode = match Scancode::try_from(scancode) {
            Ok(code) => Some(code),
            Err(_) => None
        };
    }

    /// Insert modifiers into the key event's modifier field.
    pub fn insert_modifiers(&mut self, modifiers: KeyModifiers) {
        self.modifiers.insert(modifiers);
    }

    /// Remove modifiers from the key event's modifier field.
    pub fn remove_modifiers(&mut self, modifiers: KeyModifiers) {
        self.modifiers.remove(modifiers);
    }

    /// Toggle modifiers in the key event's modifier field (i.e. set them if they are not set, and unset them if they are set).
    pub fn toggle_modifiers(&mut self, modifiers: KeyModifiers) {
        self.modifiers.toggle(modifiers);
    }

    /// Set the pressed state of the key event.
    pub fn set_pressed(&mut self, pressed: bool) {
        self.pressed = pressed;
    }

    /// Get the ascii code of the key event.
    pub fn ascii(&self) -> Option<char> {
        self.ascii
    }

    /// Get the scancode of the key event.
    pub fn scancode(&self) -> Option<Scancode> {
        self.scancode
    }

    /// Get the modifiers of the key event.
    pub fn modifiers(&self) -> KeyModifiers {
        self.modifiers
    }

    /// Check if the key is pressed.
    pub fn pressed(&self) -> bool {
        self.pressed
    }
}