impl BootInfo {
    /// Create a copy of the boot information on the heap (wrapped in a `Box`).
    pub fn copy_on_heap(&self) -> Box<BootInfo> {
        let buffer = global::alloc(Layout::from_size_align(self.total_size as usize, 8).unwrap());

        unsafe {
            buffer.copy_from_nonoverlapping(ptr::from_ref(self) as *const u8, self.total_size as usize);
            Box::from_raw(buffer as *mut BootInfo)
        }
    }
}
