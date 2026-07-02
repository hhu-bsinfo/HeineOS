/*
 * CAUTION: This is not meant as a replacement for your existing process.rs
 *          It just contains additional code that you should copy into your own file.
 */

/// Represents a process with a unique ID and a name.
pub struct Process {
    id: usize,
    name: String,
    address_space: &'static mut PageTable,
    vmas: Vec<VMA>,
}

impl Process {
    /// Create a new process with the given name.
    /// A unique ID is generated and assigned to the process.
    /// This function does not add the process to the list of active processes.
    /// This has to be done manually by calling `add_process()`.
    pub fn new(name: &str) -> Self {
        todo!("Process::new() has not been implemented yet!");
    }

    /// Add a new virtual memory area (VMA) to the process.
    pub fn add_vma(&mut self, vma: VMA) -> Result<(), &'static str> {
        todo!("Process::add_vma() has not been implemented yet!");
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        todo!("Process::drop() has not been implemented yet!");
    }
}

/// Add a new virtual memory area (VMA) to the process with the given ID.
/// This function returns `Ok` on success or `Err` if the process could not be found.
pub fn add_vma(process_id: usize, vma: VMA) -> Result<(), &'static str> {
    todo!("process::add_vma() has not been implemented yet!");
}

/// Dump all virtual memory areas (VMAs) of a process to the log.
pub fn dump_vmas(process_id: usize) {
    todo!("process::dump_vmas() has not been implemented yet!");
}
