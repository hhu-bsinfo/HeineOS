/*
 * CAUTION: This is not meant as a replacement for your existing idt.rs
 *          It just contains additional code that you should copy into your own file.
 */

impl IdtEntry {
    /// Create a new IDT entry for a trap gate at the given offset.
    /// This is used for system calls, which should be accessible from user mode (DPL=3).
    const fn new_trap_gate(offset: u64) -> IdtEntry {
        todo!("IdtEntry::new_trap_gate() is not implemented yet!");
    }

    /// Create a new IDT entry for a syscall handler function.
    /// This entry is a trap gate and has DPL=3, so it can be called from user mode.
    pub fn syscall_gate(handler: extern "C" fn()) -> IdtEntry {
        IdtEntry::new_trap_gate(handler as u64)
    }
}
