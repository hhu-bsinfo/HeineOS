/*
 * CAUTION: This is not meant as a replacement for your existing framebuffer.rs
 *          It just contains additional code that you should copy into your own file.
 */

/// Draw a bitmap image at the specified (x, y) coordinates.
/// If the bitmap does not fully fit within the framebuffer, it is clipped.
pub fn draw_bitmap(&mut self, bitmap: &Bitmap, x: usize, y: usize) {
    // Original bitmap dimensions
    let bmp_width = bitmap.width() as usize;
    let bmp_height = bitmap.height() as usize;

    // Clip the bitmap to the framebuffer dimensions
    let target_width = if x + bmp_width > self.width {
        max(self.width - x, 0)
    } else {
        bmp_width
    };

    let target_height = if y + bmp_height > self.height {
        max(self.height - y, 0)
    } else {
        bmp_height
    };

    todo!("framebuffer::draw_bitmap() is not yet implemented");
}