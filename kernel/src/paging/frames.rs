/*
 * CAUTION: This is not meant as a replacement for your existing frames.rs
 *          It just contains additional code that you should copy into your own file.
 */

/// A physical frame allocator that uses a linked list to manage free memory blocks.
/// Memory blocks are always aligned to PAGE_SIZE (4096 bytes).
pub struct PfListAllocator {
    head: PfListNode,
    max_addr: PhysAddr
}

impl PfListAllocator {
    /// Create a new empty physical frame list allocator.
    pub const fn new() -> PfListAllocator {
        PfListAllocator {
            head: PfListNode::new(0),
            max_addr: PhysAddr::new(0)
        }
    }

    /// Get the maximum physical address ever inserted into the allocator via `free_block()`.
    pub fn max_phys_addr(&self) -> PhysAddr {
        self.max_addr
    }
}
