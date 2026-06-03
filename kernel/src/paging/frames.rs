/*
 * Contains the interface definition for an Interrupt Service Routine (ISR).
 * This must be implemented by a device driver if it needs to handle interrupts.
 * The ISR is registered using the `register()` function in `intdispatcher.rs`.
 *
 * Author: Michael Schoetter, Heinrich Heine University Duesseldorf, 2024-01-21
 *         Fabian Ruhland, Heinrich Heine University Duesseldorf, 2025-10-31
 * License: GPLv3
 */

use core::fmt;
use core::ops::{Add, Sub};
use crate::consts::PAGE_SIZE;
use crate::library::spinlock::Spinlock as Mutex;

pub static FRAME_ALLOCATOR: Mutex<PfListAllocator> = Mutex::new(PfListAllocator::new());

/// Represents a physical address in memory and allows accessing it via pointers.
/// Basic arithmetic operations are implemented for easy address manipulation.
#[derive(Copy, Clone, PartialOrd, PartialEq)]
pub struct PhysAddr(u64);

impl PhysAddr {
    pub fn new(addr: u64) -> Self {
        PhysAddr(addr)
    }

    pub fn raw(&self) -> u64 {
        self.0
    }

    pub fn as_ptr<T>(&self) -> *const T {
        self.0 as *const T
    }

    pub fn as_mut_ptr<T>(&self) -> *mut T {
        self.0 as *mut T
    }
}

impl fmt::Debug for PhysAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Phys(0x{:016x})", self.0)
    }
}

impl From<PhysAddr> for u64 {
    fn from(addr: PhysAddr) -> Self {
        addr.0
    }
}

impl Add<PhysAddr> for PhysAddr {
    type Output = PhysAddr;

    fn add(self, rhs: PhysAddr) -> Self::Output {
        let res = self.0.checked_add(rhs.0).unwrap();
        PhysAddr(res)
    }
}

impl Sub<PhysAddr> for PhysAddr {
    type Output = PhysAddr;

    fn sub(self, rhs: PhysAddr) -> Self::Output {
        let res = self.0.checked_sub(rhs.0).unwrap();
        PhysAddr(res)
    }
}

impl Add<usize> for PhysAddr {
    type Output = PhysAddr;

    fn add(self, rhs: usize) -> Self::Output {
        let res = self.0.checked_add(rhs as u64).unwrap();
        PhysAddr(res)
    }
}

impl Sub<usize> for PhysAddr {
    type Output = PhysAddr;

    fn sub(self, rhs: usize) -> Self::Output {
        let res = self.0.checked_sub(rhs as u64).unwrap();
        PhysAddr(res)
    }
}

impl Add<u64> for PhysAddr {
    type Output = PhysAddr;

    fn add(self, rhs: u64) -> Self::Output {
        let res = self.0.checked_add(rhs).unwrap();
        PhysAddr(res)
    }
}

impl Sub<u64> for PhysAddr {
    type Output = PhysAddr;

    fn sub(self, rhs: u64) -> Self::Output {
        let res = self.0.checked_sub(rhs).unwrap();
        PhysAddr(res)
    }
}

/// A node in the physical frame free list.
/// Contains the size of the free block and a pointer to the next node.
struct PfListNode {
    size: usize,
    next: Option<&'static mut PfListNode>
}

impl PfListNode {
    const fn new(size: usize) -> Self {
        PfListNode { size, next: None }
    }

    fn start_addr(&self) -> PhysAddr {
        PhysAddr::new(self as *const Self as u64)
    }

    fn end_addr(&self) -> PhysAddr {
        self.start_addr() + self.size
    }
}

/// A physical frame allocator that uses a linked list to manage free memory blocks.
/// Memory blocks are always aligned to PAGE_SIZE (4096 bytes).
pub struct PfListAllocator {
    head: PfListNode
}

impl PfListAllocator {
    /// Create a new empty physical frame list allocator.
    pub const fn new() -> PfListAllocator {
        PfListAllocator {
            head: PfListNode::new(0)
        }
    }

    /// Try to allocate a block of 'num_frames' physical frames.
    /// Returns the starting physical address of the allocated block on success.
    /// The found block is filled with zeroes. If the block is larger than requested,
    /// the remaining part is added back to the free list.
    /// If no suitable block is found, returns None.
    pub unsafe fn alloc_block(&mut self, num_frames: usize) -> Option<PhysAddr> {
        todo!("PfListAllocator::alloc_block() is not implemented yet")
    }

    /// Free a previously allocated block of 'num_frames' physical frames starting at 'addr'.
    /// The address must be aligned to PAGE_SIZE (4096 bytes).
    /// The freed block is merged with adjacent free blocks if possible.
    pub unsafe fn free_block(&mut self, addr: PhysAddr, num_frames: usize) {
        todo!("PfListAllocator::free_block() is not implemented yet")
    }

    /// Print the list of free physical memory.
    pub fn dump_free_list(&self) {
        todo!("PfListAllocator::dump_free_list() is not implemented yet")
    }
}