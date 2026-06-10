/*
 * Contains the interface definition for an Interrupt Service Routine (ISR).
 * This must be implemented by a device driver if it needs to handle interrupts.
 * The ISR is registered using the `register()` function in `intdispatcher.rs`.
 *
 * Author: Michael Schoetter, Heinrich Heine University Duesseldorf, 2024-01-24
 *         Fabian Ruhland, Heinrich Heine University Duesseldorf, 2026-05-27
 * License: GPLv3
 */
use alloc::vec::Vec;
use bitflags::bitflags;
use core::arch::asm;
use core::{fmt, ptr};
use crate::allocator::noop::NoOpAllocator;
use crate::consts::{PAGE_SIZE, STACK_SIZE, USER_STACK_VIRT_START};
use crate::device::terminal::framebuffer;
use crate::library::once::Once;
use crate::paging::frames::{PhysAddr, FRAME_ALLOCATOR};

/// Global reference to kernel page tables.
/// These are initialized once during boot and remain valid as long as the operating system runs.
static KERNEL_PAGE_TABLES: Once<&'static mut PageTable> = Once::new();

/// Get a reference to the kernel page tables.
/// During the first call to this function, the pages tables are created and initialized.
/// CAUTION: This means, that this function should only be called
///          after the page frame allocator has been initialized.
pub fn kernel_page_tables() -> &'static mut PageTable {
    KERNEL_PAGE_TABLES.init(|| {
        create_kernel_mapping()
    });

    // This is somewhat tricky:
    // The `Once` stores a mutable reference to the page tables and hands out a const reference to this mutable reference.
    // We need to convert the const reference to a mutable one, by calling `get_mut()`.
    unsafe {
        KERNEL_PAGE_TABLES.get_mut().unwrap()
    }
}

/// Number of entries in a page table.
const PAGE_TABLE_ENTRIES: usize = 512;

bitflags! {
    #[derive(Debug)]
    /// Flags for a page table entry.
    pub struct PageFlags: u64 {
        const PRESENT = 1 << 0;
        const WRITEABLE = 1 << 1;
        const USER_ACCESSIBLE = 1 << 2;
        const WRITE_THROUGH = 1 << 3;
        const CACHE_DISABLE = 1 << 4;
        const ACCESSED = 1 << 5;
        const DIRTY = 1 << 6;
        const HUGE_PAGE = 1 << 7;
        const GLOBAL = 1 << 8;
    }
}

impl PageFlags {
    /// Flags used for page table entries referring to kernel memory (i.e., not accessible from user space).
    fn kernel_flags() -> Self {
        PageFlags::PRESENT | PageFlags::WRITEABLE | PageFlags::USER_ACCESSIBLE
    }

    /// Flags used for page table entries referring to user space memory (e.g., user stack, application code).
    fn user_flags() -> Self {
        PageFlags::PRESENT | PageFlags::WRITEABLE | PageFlags::USER_ACCESSIBLE
    }
}

#[repr(transparent)]
#[derive(Copy, Clone)]
/// Represents an entry inside a page table.
/// Each page table is made up of 512 of these 64-bit wide entries.
pub struct PageTableEntry(u64);

impl PageTableEntry {
    /// Create a new page table entry from the given physical address and flags
    fn new(addr: PhysAddr, flags: PageFlags) -> Self {
        let addr: u64 = addr.into();
        Self(addr | flags.bits())
    }

    /// Overwrite the physical address and flags of the page table entry
    pub fn set(&mut self, addr: PhysAddr, flags: PageFlags) {
        *self = PageTableEntry::new(addr, flags);
    }

    /// Get the physical address that the page table entry refers to
    pub fn addr(&self) -> PhysAddr {
        PhysAddr::new(self.0 & 0x000f_ffff_ffff_f000)
    }

    /// Get the flags of the page table entry
    pub fn flags(&self) -> PageFlags {
        PageFlags::from_bits_truncate(self.0)
    }
}

impl fmt::Debug for PageTableEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[addr={:?}, flags={:?}]", self.addr(), self.flags())
    }
}

#[repr(transparent)]
/// A page table made up of 512 entries.
pub struct PageTable {
    entries: [PageTableEntry; PAGE_TABLE_ENTRIES],
}

impl PageTable {
    /// Create a page table by allocating a 4 KiB page frame.
    /// A page frame is exactly the size of a page table (512 entries * 64 byte = 4096 byte = 4 KiB).
    /// The address of the page frame is converted to a `PageTable` reference and returned.
    /// CAUTION: Page tables are not dropped automatically.
    fn new<'a>() -> Option<&'a mut PageTable> {
        unsafe {
            FRAME_ALLOCATOR.lock()
                .alloc_block(1)?
                .as_mut_ptr::<PageTable>()
                .as_mut()
        }
    }

    /// Get the start address of this page table.
    pub fn as_phys_addr(&self) -> PhysAddr {
        PhysAddr::new(ptr::from_ref(self) as u64)
    }

    /// Recursive function to map memory inside a virtual address space.
    /// If kernel is `true`, an identity mapping of `virt_addr` is created (virtual address = physical address).
    /// This is used to map the kernel (and physical memory) into an address space.
    /// If kernel is `false`, new page frames are allocated and mapped to `virt_addr`.
    /// The `level` should start at 4 and is decreased with each recursion until the last page table level is reached.
    /// Missing page tables are allocated as needed.
    fn map(&mut self, virt_addr: u64, num_pages: usize, level: usize, kernel: bool) -> usize {
        todo!("PageTable::map() is not implemented yet!")
    }
}

/// Read the CR3 register and return a reference to the currently active page map level 4 (PML4).
pub fn read_cr3() -> &'static mut PageTable {
    let value: u64;
    unsafe {
        asm!("mov {}, cr3", out(reg) value)
    }

    unsafe {
        PhysAddr::new(value & 0xffff_ffff_ffff_f000)
            .as_mut_ptr::<PageTable>()
            .as_mut()
            .unwrap()
    }
}

/// Load the given page map level 4 (PML4) into the CR3 register.
pub unsafe fn write_cr3(pml4: &PageTable) {
    let addr: u64 = ptr::from_ref(pml4) as u64;
    unsafe {
        asm!("mov cr3, {}", in(reg) addr);
    }
}

/// Create a new Page Map Level 4 (PML4) and initialize it with a mapping of the full physical memory.
/// CAUTION: This only maps real physical memory. Device memory like the framebuffer must be mapped manually afterward.
pub fn create_kernel_mapping() -> &'static mut PageTable {
    let max_phys_addr = FRAME_ALLOCATOR.lock().max_phys_addr();
    let num_pages = (max_phys_addr.raw() as usize + PAGE_SIZE - 1) / PAGE_SIZE;

    let pml4 = PageTable::new().expect("Failed to allocate frame for PML4!");
    pml4.map(0, num_pages, 4, true);

    pml4
}

/// Create an identity (1:1) mapping for the framebuffer in the given virtual address space.
pub unsafe fn map_framebuffer(pml4_table: &mut PageTable) {
    let framebuffer = framebuffer().lock();
    let num_pages = (framebuffer.size_in_bytes() + PAGE_SIZE - 1) / PAGE_SIZE;

    pml4_table.map(framebuffer.address(), num_pages, 4, true);
}

/// Create a mapping for the user stack in the given virtual address space.
/// The stack is mapped to the address range [USER_STACK_VIRT_START, USER_STACK_VIRT_END).
/// Physical page frames for the stack are automatically allocated.
pub unsafe fn map_user_stack(pml4_table: &mut PageTable) -> Vec<u64, NoOpAllocator> {
    let stack_size = STACK_SIZE / size_of::<u64>();

    todo!("pages::map_user_stack() is not fully implemented yet!");

    unsafe {
        Vec::<u64, NoOpAllocator>::from_raw_parts_in(
            USER_STACK_VIRT_START as *mut u64,
            stack_size,
            stack_size,
            NoOpAllocator::new()
        )
    }
}