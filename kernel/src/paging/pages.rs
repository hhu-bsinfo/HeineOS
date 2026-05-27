use bitflags::bitflags;
use core::arch::asm;
use crate::device::cpu;
use crate::paging::pages;

/// Number of entries in a page table.
const PAGE_TABLE_ENTRIES: usize = 512;

bitflags! {
    #[derive(Debug)]
    /// Flags for a page table entry.
    pub struct PageFlags: u64 {
        const PRESENT = 1 << 0;
        const WRITEABLE = 1 << 1;
        const USER = 1 << 2;
        const WRITE_THROUGH = 1 << 3;
        const CACHE_DISABLE = 1 << 4;
        const ACCESSED = 1 << 5;
        const DIRTY = 1 << 6;
        const HUGE_PAGE = 1 << 7;
        const GLOBAL = 1 << 8;
    }
}

/// Read the CR3 register and return a pointer to the currently active page map level 4 (PML4).
pub fn read_cr3() -> *mut u64 {
    let value: u64;
    unsafe {
        asm!("mov {}, cr3", out(reg) value)
    }

    (value & 0xffff_ffff_ffff_f000) as *mut u64
}

/// The UEFI/bootloader has already set up an identity (1:1) page mapping.
/// However, in this mapping the USER bit is probably not set,
/// so that all pages are only accessible in kernel mode (ring 0).
///
/// For our first user mode threads (lesson 8), we need the kernel to be accessible from ring 3.
/// This function iterates through all page tables (starting with page map level 4
/// pointed to by the CR3 register) and sets the user bit in each one.
///
/// We will set up our own paging environment later on (lesson 11)
/// and won't need this function anymore afterward.
pub fn setup_initial_paging() {
    // Disable write protection bit in CR0. The page tables are probably read-only.
    // If this bit is set, even the kernel cannot modify read-only pages.
    let mut cr0 = cpu::read_cr0();
    cr0.remove(cpu::Cr0Flags::WRITE_PROTECT);
    cpu::write_cr0(cr0);

    unsafe {
        // CR3 register contains the address of the root page table (Page Map Level 4)
        let pml4 = pages::read_cr3();
        // Recursively iterate through all page tables and set the user bit.
        setup_initial_page_map(pml4, 4);
    }

    // Set WRITE_PROTECT bit in CR0 again.
    // Even if it was not set before, it is better to set it now.
    let mut cr0 = cpu::read_cr0();
    cr0.insert(cpu::Cr0Flags::WRITE_PROTECT);
    cpu::write_cr0(cr0);
}

/// Iterate through all entries in the page map pointed to by `map` and set the user bit.
/// This function calls itself recursively with the next level page map until the last level is reached.
pub unsafe fn setup_initial_page_map(map: *mut u64, level: usize) {
    for i in 0..512 {
        unsafe {
            // Read the entry at the current index and check if it is not zero.
            // If it is zero, we can skip it as it is not a valid page table entry.
            let entry = map.offset(i).read();
            if entry != 0 {
                // The entry is valid, so we read the flags and set the user bit.
                let mut flags = PageFlags::from_bits_truncate(entry);
                flags.insert(PageFlags::USER);

                // Write the modified entry back to the page table.
                map.offset(i).write(entry | flags.bits());

                // Check if we are at the last level of the page map.
                if level > 1 && !flags.contains(PageFlags::HUGE_PAGE) {
                    // If not, extract the address of the next level page map from the entry.
                    let next_page_map = (entry & 0x000ffffffffff000) as *mut u64;
                    // Recursively call this function with the next level page map.
                    setup_initial_page_map(next_page_map, level - 1);
                }
            }
        }
    }
}