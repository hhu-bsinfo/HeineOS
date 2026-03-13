/*
 * Driver to emulate a simple terminal for text output on a framebuffer.
 * This module also implements the `print!()` and `println!()` macros for formatted output.
 *
 * Author: Fabian Ruhland, Heinrich Heine University Duesseldorf, 2026-01-07
 * License: GPLv3
 */

use core::fmt;
use core::fmt::Write;
use crate::device::{font_8x8, framebuffer};
use crate::device::framebuffer::Framebuffer;
use crate::library::once::Once;
use crate::library::spinlock::Spinlock;

/// Global terminal instance protected by a spinlock.
/// This instance is initialized once during kernel startup.
/// After initialization, it can be accessed via the `terminal()` function.
static TERMINAL: Once<Spinlock<Terminal>> = Once::new();

/// Initialize the global terminal instance with the given framebuffer.
/// This function should be called once during kernel startup.
/// After calling this function, the terminal can be accessed via the `terminal()` function.
pub fn init_terminal(framebuffer: Framebuffer) {
    TERMINAL.init(|| Spinlock::new(Terminal::new(framebuffer)));
}

/// Get a reference to the global terminal instance.
/// This function panics if the terminal has not been initialized yet.
pub fn terminal() -> &'static Spinlock<Terminal> {
    TERMINAL.get().expect("Terminal not initialized")
}

/// Get access to the framebuffer used by the global terminal instance.
/// This way, other parts of the kernel can access the framebuffer for graphics output.
/// The framebuffer is protected by a spinlock, so the caller must lock it before use.
/// While the framebuffer is locked, no other thread can access it, and the terminal is effectively paused.
/// This function panics if the terminal has not been initialized yet.
pub fn framebuffer() -> &'static Spinlock<Framebuffer> {
    // This looks unsafe, but is actually safe because the `framebuffer()` method
    // does not mutate the terminal instance, and the framebuffer is protected by a spinlock.
    unsafe { terminal().inner().framebuffer() }
}

/// Default text foreground color.
const DEFAULT_FG_COLOR: u32 = framebuffer::GREEN;
/// Default text background color.
const DEFAULT_BG_COLOR: u32 = framebuffer::BLACK;

/// A simple terminal driver for text output on a framebuffer.
/// It supports basic character output and a static cursor.
/// The terminal takes ownership of the framebuffer.
pub struct Terminal {
    /// Number of columns in the terminal (based on framebuffer width and font width).
    cols: usize,
    /// Number of rows in the terminal (based on framebuffer height and font height).
    rows: usize,
    /// Current cursor position (column, row) where the next character will be printed.
    pos: (usize, usize),
    /// The framebuffer used for rendering text.
    /// Protected by a spinlock to allow safe concurrent access.
    /// This allows safe access to the framebuffer from multiple threads via the `framebuffer()` method.
    framebuffer: Spinlock<Framebuffer>
}

impl Terminal {
    /// Create a new Terminal instance with the given framebuffer.
    /// The terminal calculates its size based on the framebuffer dimensions and the font size.
    pub fn new(mut framebuffer: Framebuffer) -> Terminal {
        let cols = framebuffer.width / font_8x8::CHAR_WIDTH;
        let rows = framebuffer.height / font_8x8::CHAR_HEIGHT;

        framebuffer.clear();

        Terminal {
            cols,
            rows,
            pos: (0, 0),
            framebuffer: Spinlock::new(framebuffer)
        }
    }

    /// Get direct access to the framebuffer.
    /// The framebuffer is protected by a spinlock, so the caller must lock it before use.
    /// While the framebuffer is locked, no other thread can access it, and the terminal is effectively paused.
    pub fn framebuffer(&self) -> &Spinlock<Framebuffer> {
        &self.framebuffer
    }

    /// Get the current cursor position (column, row).
    pub fn pos(&self) -> (usize, usize) {
        todo!("terminal::pos() not implemented yet");
    }

    /// Set the cursor position to the given position.
    /// If the position is out of bounds, it is clamped to the terminal size.
    pub fn set_pos(&mut self, col: usize, row: usize) {
        todo!("terminal::set_pos() not implemented yet");
    }

    /// Clear the terminal screen and reset the cursor position to the top-left corner.
    pub fn clear(&mut self) {
        todo!("terminal::clear() not implemented yet");
    }

    /// Draw a character with the default colors at the current cursor position and advance the cursor.
    /// A newline character moves the cursor to the beginning of the next line.
    /// If the cursor reaches the end of the terminal, the screen scrolls up.
    pub fn put_char(&mut self, c: char) {
        todo!("terminal::put_char() not implemented yet");
    }

    /// Draw a colored character at the current cursor position and advance the cursor.
    /// A newline character moves the cursor to the beginning of the next line.
    /// If the cursor reaches the end of the terminal, the screen scrolls up using `framebuffer::scroll_up()`
    pub fn put_char_colored(&mut self, c: char, fg_color: u32, bg_color: u32) {
        todo!("terminal::put_char_colored() not implemented yet");
    }

    /// Draw the cursor at the given position by drawing a white space character in the default foreground color.
    fn draw_cursor(pos: (usize, usize), framebuffer: &mut Framebuffer) {
        let x = pos.0 * font_8x8::CHAR_WIDTH;
        let y = pos.1 * font_8x8::CHAR_HEIGHT;

        framebuffer.draw_char(' ', x, y, DEFAULT_FG_COLOR, DEFAULT_FG_COLOR);
    }

    /// Clear the cursor at the given position by drawing a white space character in the default background color.
    fn clear_cursor(pos: (usize, usize), framebuffer: &mut Framebuffer) {
        let x = pos.0 * font_8x8::CHAR_WIDTH;
        let y = pos.1 * font_8x8::CHAR_HEIGHT;

        framebuffer.draw_char(' ', x, y, DEFAULT_BG_COLOR, DEFAULT_BG_COLOR);
    }
}

// Implement the `fmt::Write` trait for the Terminal to support formatted output.
// We only need to implement the `write_str()` method, which writes a string to the terminal.
// This allows formatted output via the `write_fmt()` method provided by the `fmt::Write` trait.
impl Write for Terminal {
    /// Write a string to the COM port by iterating over each byte in the string and writing it using `put_char()`.
    fn write_str(&mut self, s: &str) -> fmt::Result {
        // TODO: Write the string using put_char()
        Ok(())
    }
}

#[macro_export]
/// Print a formatted string to the CGA text buffer.
/// This macro locks the CGA instance and writes the formatted string to it.
macro_rules! print {
    ($($arg:tt)*) => ({
        let mut terminal = $crate::device::terminal::terminal().lock();
        $crate::terminal::print(&mut terminal, format_args!($($arg)*));
    });
}

#[macro_export]
/// Print a formatted string to the CGA text buffer with a newline.
/// This is a convenience macro, wrapping the `print!` macro to add a newline at the end.
/// This macro locks the CGA instance and writes the formatted string to it.
macro_rules! println {
    ($fmt:expr) => (print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*));
}

#[macro_export]
/// Print a formatted string to the terminal.
/// This macro is similar to `print!()`, but it takes a mutable reference to a terminal instance,
/// instead of locking the global terminal instance. This is useful for performing multiple writes
/// without locking and unlocking the terminal instance each time.
macro_rules! print_terminal {
    ($cga:expr, $($arg:tt)*) => ({
        $crate::device::terminal::print($cga, format_args!($($arg)*));
    });
}

#[macro_export]
/// Print a formatted string to the terminal text buffer with a newline.
/// This macro is similar to `println!()`, but it takes a mutable reference to a terminal instance,
/// instead of locking the global terminal instance. This is useful for performing multiple writes
/// without locking and unlocking the CGA instance each time.
macro_rules! println_terminal {
    ($cga:expr, $fmt:expr) => (print_cga!($cga, concat!($fmt, "\n")));
    ($cga:expr, $fmt:expr, $($arg:tt)*) => (print_cga!($cga, concat!($fmt, "\n"), $($arg)*));
}

/// Helper function for the print macros.
pub fn print(terminal: &mut Terminal, args: fmt::Arguments) {
    terminal.write_fmt(args).expect("Failed to write to terminal");
}