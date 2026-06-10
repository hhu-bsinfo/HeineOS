/*
 * CAUTION: This is not meant as a replacement for your existing framebuffer.rs
 *          It just contains additional code that you should copy into your own file.
 */

impl Framebuffer
    /// Get the memory address of the framebuffer.
    pub fn address(&self) -> u64 {
        self.address
    }

    /// Get the size of the framebuffer in bytes.
    pub fn size_in_bytes(&self) -> usize {
        self.pitch * self.height
    }
}
