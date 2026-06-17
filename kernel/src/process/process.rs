/*
 * Contains functions to create and manage processes.
 *
 * Author: Michael Schoettner, Heinrich Heine University Duesseldorf, 2023-12-29
 *         Fabian Ruhland, Heinrich Heine University Duesseldorf, 2026-06-11
 * License: GPLv3
 */

use alloc::collections::BTreeMap;
use alloc::string::String;
use core::ptr;
use core::sync::atomic::{AtomicUsize, Ordering};
use log::info;
use usrlib::spinlock::Spinlock;
use usrlib::once::Once;
use crate::paging::pages;
use crate::paging::pages::PageTable;

/// Map of all active processes.
/// The key is the process ID and the value is the process itself.
static PROCESSES: Spinlock<BTreeMap<usize, Process>> = Spinlock::new(BTreeMap::new());

/// Atomic counter for generating unique process IDs.
static NEXT_PID: AtomicUsize = AtomicUsize::new(0);

/// ID of the kernel process.
/// This is initialized lazily the first time the kernel process is accessed.
static KERNEL_PROCESS_ID: Once<usize> = Once::new();

/// Get the ID of the kernel process.
/// The kernel process is created and added to the list of active processes the first time this function is called.
/// CAUTION: This function should only be called after the page frame allocator and the kernel heap have been initialized.
pub fn kernel_process_id() -> usize {
    *KERNEL_PROCESS_ID.init(|| {
        let kernel_process = Process::new("kernel");
        let id = kernel_process.id();
        add_process(kernel_process);

        id
    })
}

/// Represents a process with a unique ID and a name.
pub struct Process {
    id: usize,
    name: String,
    address_space: &'static mut PageTable
}

impl Process {
    /// Create a new process with the given name.
    /// A unique ID is generated and assigned to the process.
    /// This function does not add the process to the list of active processes.
    /// This has to be done manually by calling `add_process()`.
    pub fn new(name: &str) -> Self {
        let pid = NEXT_PID.fetch_add(1, Ordering::SeqCst);
        let pml4 = pages::create_kernel_mapping();
        unsafe { pages::map_framebuffer(pml4); }

        Process { id: pid, name: String::from(name), address_space: pml4 }
    }

    /// Get the ID of the process.
    /// The ID is unique across all processes.
    pub fn id(&self) -> usize {
        self.id
    }

    /// Get the name of the process.
    /// The name is a string that can be used to identify the process.
    /// The name is not unique across all processes.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get a reference to the address space of the process.
    /// The reference has a static lifetime, which is unsafe.
    /// The caller is responsible for ensuring that the address space reference does not outlive the process.
    pub unsafe fn address_space(&self) -> &'static mut PageTable {
        unsafe {
            // We cannot simply return the PML4 reference, as it is owned by the process and the reference might outlive the process.
            // Instead, we cast it to a pointer and back to a reference to work around the borrow checker.
            // This is highly unsafe, and the caller must ensure that the process is still alive while working with the address space.
            ptr::from_ref(self.address_space).cast_mut().as_mut().unwrap()
        }
    }
}

/// Add a process to the list of active processes.
pub fn add_process(process: Process) {
    todo!("process::add_process() is not implemented yet!");
}

/// Remove a process from the list of active processes.
/// The removed proces is returned.
pub fn remove_process(process_id: usize) -> Option<Process> {
    todo!("process::remove_process() is not implemented yet!");
}

/// Get the name of a process by its ID.
pub fn get_app_name(process_id: usize) -> Option<String> {
    todo!("process::get_app_name() is not implemented yet!");
}

/// Get a reference to the address space of a process by its ID.
/// The reference has static lifetime, which is unsafe.
/// The caller is responsible for ensuring that the address space reference does not outlive the process.
pub unsafe fn get_address_space(process_id: usize) -> Option<&'static mut PageTable> {
    todo!("process::get_address_space() is not implemented yet!");
}
