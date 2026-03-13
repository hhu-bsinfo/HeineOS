/*
 * Driver for a linear framebuffer in 32-bit RGB format.
 *
 * Author: Michael Schoetter, Heinrich Heine University Duesseldorf, 2023-06-26
 *         Fabian Ruhland, Heinrich Heine University Duesseldorf, 2026-01-07
 * License: GPLv3
 */
use crate::device::font_8x8;
use crate::multiboot;

/// Represents a linear framebuffer for graphics output.
/// The framebuffer is expected to be in 32-bit RGB format.
pub struct Framebuffer {
    /// The width of the framebuffer in pixels.
    pub width: usize,
    /// The height of the framebuffer in pixels.
    pub height: usize,
    /// The number of bytes per row of pixels.
    /// This may be greater than (width * 4) due to padding.
    pitch: usize,
    /// A pointer to the start of the framebuffer memory.
    address: u64,
}

/// Create a 32-bit color value from red, green, and blue components.
/// Each component is an 8-bit value (0-255).
/// The resulting color is in the format 0x00RRGGBB.
pub const fn color(red: u8, green: u8, blue: u8) -> u32 {
    ((red as u32) << 16) | ((green as u32) << 8) | (blue as u32)
}

// ANSI colors
pub const BLACK: u32 = color(0, 0, 0);
pub const RED: u32 = color(170, 0, 0);
pub const GREEN: u32 = color(0, 170, 0);
pub const YELLOW: u32 = color(170, 170, 0);
pub const BROWN: u32 = color(170, 85, 0);
pub const BLUE: u32 = color(0, 0, 170);
pub const MAGENTA: u32 = color(170, 0, 170);
pub const CYAN: u32 = color(0, 170, 170);
pub const WHITE: u32 = color(170, 170, 170);

impl Framebuffer {
    /// Create a new Framebuffer instance.
    /// This function is unsafe because the caller must ensure that the provided
    /// buffer pointer is valid and points to a memory region large enough to hold
    /// the framebuffer data.
    pub const unsafe fn new(width: usize, height: usize, pitch: usize, address: u64) -> Framebuffer {
        Framebuffer { width, height, pitch, address }
    }

    /// Create a Framebuffer from multiboot framebuffer information.
    /// Returns None if the framebuffer type is not supported or if the bits per pixel is not 32.
    /// This function is safe, because it assumes the multiboot information is valid.
    pub const fn from_multiboot(info: &multiboot::FramebufferInfo) -> Option<Framebuffer> {
        match info.typ {
            multiboot::FramebufferType::RGB => {
                if info.bpp != 32 {
                    None
                } else {
                    Some(Framebuffer {
                        width: info.width as usize,
                        height: info.height as usize,
                        pitch: info.pitch as usize,
                        address: info.address,
                    })
                }
            },
            _ => None,
        }
    }

    /// Get the width of the framebuffer in pixels.
    pub fn width(&self) -> usize {
        self.width
    }

    /// Get the height of the framebuffer in pixels.
    pub fn height(&self) -> usize {
        self.height
    }

    /// Clear the framebuffer by filling it with black pixels.
    pub fn clear(&mut self) {
        let buffer = self.address as *mut u8;
        unsafe { buffer.write_bytes(0, self.pitch * self.height); }
    }

    /// Draw a pixel at the specified (x, y) coordinates with the given color.
    /// This method checks the bounds of the framebuffer before drawing
    /// and omits drawing if the coordinates are out of bounds.
    pub fn draw_pixel(&mut self, x: usize, y: usize, color: u32) {
        if x < self.width && y < self.height {
            unsafe { self.draw_pixel_unchecked(x, y, color); }
        }
    }

    /// Draw a pixel at the specified (x, y) coordinates with the given color.
    /// This method does not check the bounds of the framebuffer.
    /// This is faster than `draw_pixel` but the caller must ensure that the coordinates are valid.
    /// Drawing outside the framebuffer may lead to undefined behavior.
    pub unsafe fn draw_pixel_unchecked(&mut self, x: usize, y: usize, color: u32) {
        let offset = y * self.pitch + x * 4;

        let buffer = self.address as *mut u8;
        unsafe { buffer.add(offset).cast::<u32>().write_volatile(color); }
    }

    /// Get the pixel data for a character from the font data.
    fn get_char_pixels(c: char) -> &'static [u8] {
        let char_mem_size = (font_8x8::CHAR_WIDTH + (8 >> 1)) / 8 * font_8x8::CHAR_HEIGHT;
        let start = char_mem_size * c as usize;
        let end = start + char_mem_size;

        &font_8x8::DATA[start..end]
    }

    /// Draw a single character at the specified (x, y) coordinates with the given foreground and background colors.
    /// If the character does not fit fully within the framebuffer, it is not drawn.
    pub fn draw_char(&mut self, c: char, x: usize, y: usize, fg_color: u32, bg_color: u32) {
        let char_width  = font_8x8::CHAR_WIDTH;
        let char_height = font_8x8::CHAR_HEIGHT;
        if x + char_width > self.width || y + char_height > self.height {
            return;
        }

        let width_byte = (char_width + 7) / 8;
        let char_pixels = Framebuffer::get_char_pixels(c);
        let mut pixel_index = 0;

        for y_offset in 0..char_height {
            let mut x = x;
            let y = y + y_offset;

            for _ in 0..width_byte {
                for bit in (0..8).rev() {
                    if ((1 << bit) & char_pixels[pixel_index]) == 0 {
                        // Safe because we already checked bounds above
                        unsafe { self.draw_pixel_unchecked(x, y, bg_color); }
                    } else {
                        // Safe because we already checked bounds above
                        unsafe { self.draw_pixel_unchecked(x, y, fg_color); }
                    }

                    x += 1;
                }
            }

            pixel_index += 1;
        }
    }

    /// Draw a string at the specified (x, y) coordinates with the given foreground and background colors.
    pub fn draw_str(&mut self, str: &str, x: usize, y: usize, fg_color: u32, bg_color: u32) {
        let mut x = x;

        for c in str.chars() {
            self.draw_char(c, x, y, fg_color, bg_color);
            x += font_8x8::CHAR_WIDTH;
        }
    }

    /// Scroll the framebuffer content up by the specified number of lines.
    /// The freed space at the bottom is cleared to black.
    pub fn scroll_up(&mut self, lines: usize) {
        todo!("framebuffer::scroll_up() not implemented yet");
    }
}