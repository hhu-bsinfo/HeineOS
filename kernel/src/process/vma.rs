/*
 * Contains the definition of a virtual memory area (VMA).
 *
 * Author: Michael Schoettner, Heinrich Heine University Duesseldorf, 2024-01-05
 *         Fabian Ruhland, Heinrich Heine University Duesseldorf, 2026-06-18
 * License: GPLv3
 */
use core::fmt;

#[derive(Debug)]
pub enum VmaType {
    Code,
    Heap,
    Stack,
    Framebuffer
}

/// Represents a virtual memory area (VMA) in the address space of a process.
pub struct VMA {
    start: u64,
    end: u64,
    typ: VmaType,
}

impl VMA {
    /// Create a new VMA with a start and end address and a given type.
    pub fn new(start: u64, end: u64, typ: VmaType) -> Self {
        VMA { start, end, typ }
    }

    /// Check if this VMA overlaps with another one.
    pub fn overlaps(&self, other: &VMA) -> bool {
        todo!("VMA::overlaps() is not implemented yet!");
    }
}

impl fmt::Debug for VMA {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "VMA {{ start: 0x{:016x}, end: {:#016x}, type: {:?} }}", self.start, self.end, self.typ)
    }
}
