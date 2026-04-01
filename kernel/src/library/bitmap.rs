/*
 * Bitmap Image Loader for 24-bit and 32-bit uncompressed BMP files.
 *
 * Author: Fabian Ruhland, Heinrich Heine University Duesseldorf, 2026-01-14
 * License: GPLv3
 */

use alloc::vec;
use alloc::vec::Vec;
use crate::filesystem::tarfs::{filesystem, FsError};
use crate::library::bitmap::Compression::BitFields;

/// Represents a bitmap image loaded from a BMP file.
/// The pixel data is stored as a vector of 32-bit color values in ARGB format.
/// The color format matches the framebuffer color format,
/// so the bitmap data can be copied directly to the framebuffer.
pub struct Bitmap {
    header: BitmapFileHeader,
    pixel_data: Vec<u32>,
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
/// The BMP file header structure (see: https://en.wikipedia.org/wiki/BMP_file_format)
struct BitmapFileHeader {
    /// Signature must be 'BM' (0x42, 0x4D).
    signature: [u8; 2],
    /// The size of the whole BMP file in bytes.
    file_size: u32,
    /// Reserved (unused).
    reserved: u32,
    /// The offset to the start of the pixel data.
    data_offset: u32,
    /// The BMP info header.
    info_header: BitmapInfoHeader,
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
/// The BMP info header structure (see: https://en.wikipedia.org/wiki/BMP_file_format)
struct BitmapInfoHeader {
    /// The size of this header (should be 40 bytes for BITMAPINFOHEADER).
    header_size: u32,
    /// The width of the bitmap in pixels.
    width: i32,
    /// The height of the bitmap in pixels.
    height: i32,
    /// The number of color planes (must be 1).
    color_planes: u16,
    /// The number of bits per pixel (only 24 is supported).
    bits_per_pixel: u16,
    /// The compression method used (Only the uncompressed formats `None` and `BitFields` are supported).
    compression: Compression,
    /// The size of the raw bitmap data.
    image_size: u32,
    /// The horizontal resolution (pixels per meter, unused by HeineOS).
    x_pixels_per_meter: i32,
    /// The vertical resolution (pixels per meter, unused by HeineOS).
    y_pixels_per_meter: i32,
    /// The number of colors in the color palette (unused by HeineOS).
    colors_used: u32,
    /// The number of important colors (unused by HeineOS).
    important_colors: u32,
}

#[repr(u32)]
#[derive(Copy, Clone, PartialEq)]
enum Compression {
    None = 0,
    BitFields = 3,
}

/// Create a 32-bit ARGB color value from individual red, green, blue, and alpha components.
fn color(r: u8, g: u8, b: u8, a: u8) -> u32 {
    ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

impl Bitmap {
    /// Read a bitmap image from a file at the given path.
    /// Returns `Ok(Some(Bitmap))` if the file was read and parsed successfully.
    /// Returns `Ok(None)` if the file is not a valid or supported BMP image.
    /// Returns `Err(FsError)` if there was an error reading the file.
    pub fn read_from_file(path: &str) -> Result<Option<Bitmap>, FsError> {
        let filesystem = filesystem();
        let file = filesystem.open(path)?;
        let size = filesystem.size(file)?;

        let mut bmp_data = vec![0u8; size];
        filesystem.read(file, &mut bmp_data)?;

        Ok(Bitmap::from_bytes(&bmp_data))
    }

    /// Parse a bitmap image from the given byte slice.
    /// Returns `Some(Bitmap)` if the data represents a valid and supported BMP image.
    /// Returns `None` if the data is not a valid or supported BMP image.
    pub fn from_bytes(data: &[u8]) -> Option<Bitmap> {
        // Get a reference to the BMP file header inside the given byte slice
        let header_slice = &data[..size_of::<BitmapFileHeader>()];
        let header = unsafe {
            &*(header_slice.as_ptr() as *const BitmapFileHeader)
        };

        todo!("Bitmap::from_bytes() is not yet implemented");
    }

    /// Get the width of the bitmap in pixels.
    pub fn width(&self) -> u32 {
        self.header.info_header.width as u32
    }

    /// Get the height of the bitmap in pixels.
    pub fn height(&self) -> u32 {
        self.header.info_header.height as u32
    }

    /// Get a reference to the pixel data of the bitmap.
    /// The data is in 32-bit ARGB format, matching the framebuffer color format.
    pub fn pixel_data(&self) -> &[u32] {
        &self.pixel_data
    }
}