/*
 * CAUTION: This is not meant as a replacement for your existing boot.rs
 *          It just contains additional code that you should copy into your own file.
 */

/// Initialize the physical memory allocator.
/// This function iterates through the memory map and inserts all available blocks into the page frame allocator.
/// It should only be called once during startup, after the UEFI boot services have been exited.
fn init_physical_memory_allocator(memory_map: &MemoryMapOwned) {
    let kernel_end = kernel_end() as u64;
    let mut allocator = FRAME_ALLOCATOR.lock();

    for entry in memory_map.entries().filter(|entry| entry.ty == MemoryType::CONVENTIONAL) {
        let mut start = entry.phys_start;
        let mut end = entry.phys_start + entry.page_count * PAGE_SIZE as u64;

        // Check for overlap with kernel memory
        if start < kernel_end {
            start = kernel_end;
            if start >= end {
                continue;
            }
        }

        // Align start and end address to 4096 bytes
        if start % (PAGE_SIZE as u64) != 0 {
            start = (start / (PAGE_SIZE as u64) + 1) * (PAGE_SIZE as u64);
        }
        if end % (PAGE_SIZE as u64) != 0 {
            end = (end / (PAGE_SIZE as u64)) * (PAGE_SIZE as u64);
        }

        // Insert block into physical memory allocator
        let num_frames = ((end - start) / (PAGE_SIZE as u64)) as usize;
        if num_frames > 0 {
            debug!("Inserting physical memory block (Addr: 0x{:x}, Size: {} frames)", start, num_frames);
            unsafe {
                allocator.free_block(PhysAddr::new(start), num_frames);
            }
        }
    }
}