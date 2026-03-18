/*
 * Contains the Interrupt Descriptor Table (IDT) and its entries.
 * The IDT stores 256 entries, each of which points to an interrupt handler.
 * When an interrupt occurs, the CPU looks up the corresponding entry in the IDT and calls the handler function.
 *
 * In HeineOS, all entries point to the `intdispatcher::dispatch_interrupt()` function,
 * which is responsible for dispatching the interrupt to a registered handler.
 *
 * The IDT is loaded into the CPU using the `lidt` instruction,
 * which is wrapped in the `load()` function of the `Idt` struct, for convenience.
 *
 * Author: Fabian Ruhland, Heinrich Heine University Duesseldorf, 2026-01-14
 * License: GPLv3
 */

use core::arch::asm;
use core::ptr;
use crate::interrupt::dispatcher::dispatch_interrupt;
use crate::library::once::Once;

/// Static instance of the Interrupt Descriptor Table (IDT).
/// Wrapped inside a Once, because Idt::new() is not const.
static IDT: Once<Idt> = Once::new();

/// Global access to the IDT via a static reference.
pub fn idt() -> &'static Idt {
    IDT.init(|| Idt::new())
}

/// The IDT has 256 entries.
pub const IDT_SIZE: usize = 256;

#[derive(Debug)]
#[repr(C, packed)]
/// Context that is pushed onto the stack automatically
/// by the CPU when an interrupt occurs.
pub struct InterruptStackFrame {
    pub instruction_pointer: u64,
    pub code_segment: u64,
    pub flags: u64,
    pub stack_pointer: u64,
    pub stack_segment: u64,
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
/// Structure of an entry in the IDT.
/// See the OSDev-Wiki for more details:
/// https://wiki.osdev.org/Interrupt_Descriptor_Table#Structure_on_x86-64
pub struct IdtEntry {
    offset_low: u16,
    selector: u16,
    options: u16,
    offset_mid: u16,
    offset_high: u32,
    reserved: u32,
}

#[repr(C, packed)]
/// The IDT itself is just a packed array of 256 IDT entries.
pub struct Idt {
    entries: [IdtEntry; IDT_SIZE]
}

#[repr(C, packed)]
/// The IDT descriptor is used to load the IDT into the CPU.
/// It contains the address of the IDT and its size.
struct IdtDescriptor {
    limit: u16,
    address: u64,
}

impl IdtDescriptor {
    /// Create a new IDT descriptor for a given IDT.
    fn new(idt: &Idt) -> IdtDescriptor {
        IdtDescriptor {
            limit: (size_of::<Idt>() - 1) as u16, // Limit is the size of the IDT - 1
            address: ptr::from_ref(idt) as u64 // Address just points to the beginning of the IDT
        }
    }
}

impl IdtEntry {
    /// Create a new IDT entry for an interrupt handler at the given offset.
    /// Each entry has the same selector and options:
    /// The selector is the second entry in the GDT (kernel code segment) -> 2 * 8 = 16.
    /// The options are always 'Present', 'DPL=0' and '64-bit interrupt gate'.
    const fn new(offset: u64) -> IdtEntry {
        todo!("IdtEntry::new() not implemented yet.");
    }

    /// Create a new IDT entry for an interrupt handler function.
    /// The function must be marked as 'extern "x86-interrupt"'.
    pub fn without_error_code(handler: extern "x86-interrupt" fn(InterruptStackFrame)) -> IdtEntry {
        IdtEntry::new(handler as u64)
    }

    /// Create a new IDT entry for an interrupt handler function with an error code.
    /// The function must be marked as 'extern "x86-interrupt"'.
    /// This is only used for some CPU exceptions (e.g. Page Faults).
    /// See the OSDev wiki for a full list of exceptions: https://wiki.osdev.org/Exceptions
    pub fn with_error_code(handler: extern "x86-interrupt" fn(InterruptStackFrame, error_code: u64)) -> IdtEntry {
        IdtEntry::new(handler as u64)
    }
}

#[macro_export]
/// Macro to create an IDT entry for a given interrupt number and handler function.
/// The macro automatically creates a wrapper function for the handler,
/// which is marked as 'extern "x86-interrupt"' and determines, whether an error code is needed for the handler, or not.
/// This way normal Rust functions can be used as interrupt handlers.
/// Furthermore, the interrupt vector number is passed to the handler function.
/// The signature of an interrupt handler is:
/// fn handler(vector: u8, stack_frame: InterruptStackFrame, error_code: Option<u64>)
macro_rules! interrupt_handler {
    ($int_num:expr, $handler:expr) => {{
        match $int_num {
            0x08 | 0x0a | 0x0b | 0x0c | 0x0d | 0x0e | 0x11 | 0x15 | 0x1d | 0x1e => {
                // These interrupts push an error code onto the stack
                extern "x86-interrupt" fn wrapper(stack_frame: InterruptStackFrame, error_code: u64) {
                    $handler($int_num, stack_frame, Some(error_code));
                }

                IdtEntry::with_error_code(wrapper)
            },
            _ => {
                // These interrupts do not push an error code onto the stack
                extern "x86-interrupt" fn wrapper(stack_frame: InterruptStackFrame) {
                    $handler($int_num, stack_frame, None);
                }

                IdtEntry::without_error_code(wrapper)
            }
        }
    }};
}

impl Idt {
    /// Create a new IDT with all entries set to the default handler `dispatch_interrupt()`.
    pub fn new() -> Idt {
        Idt {
            entries: [
                interrupt_handler!(0x00, dispatch_interrupt),
                interrupt_handler!(0x01, dispatch_interrupt),
                interrupt_handler!(0x02, dispatch_interrupt),
                interrupt_handler!(0x03, dispatch_interrupt),
                interrupt_handler!(0x04, dispatch_interrupt),
                interrupt_handler!(0x05, dispatch_interrupt),
                interrupt_handler!(0x06, dispatch_interrupt),
                interrupt_handler!(0x07, dispatch_interrupt),
                interrupt_handler!(0x08, dispatch_interrupt),
                interrupt_handler!(0x09, dispatch_interrupt),
                interrupt_handler!(0x0a, dispatch_interrupt),
                interrupt_handler!(0x0b, dispatch_interrupt),
                interrupt_handler!(0x0c, dispatch_interrupt),
                interrupt_handler!(0x0d, dispatch_interrupt),
                interrupt_handler!(0x0e, dispatch_interrupt),
                interrupt_handler!(0x0f, dispatch_interrupt),
                interrupt_handler!(0x10, dispatch_interrupt),
                interrupt_handler!(0x11, dispatch_interrupt),
                interrupt_handler!(0x12, dispatch_interrupt),
                interrupt_handler!(0x13, dispatch_interrupt),
                interrupt_handler!(0x14, dispatch_interrupt),
                interrupt_handler!(0x15, dispatch_interrupt),
                interrupt_handler!(0x16, dispatch_interrupt),
                interrupt_handler!(0x17, dispatch_interrupt),
                interrupt_handler!(0x18, dispatch_interrupt),
                interrupt_handler!(0x19, dispatch_interrupt),
                interrupt_handler!(0x1a, dispatch_interrupt),
                interrupt_handler!(0x1b, dispatch_interrupt),
                interrupt_handler!(0x1c, dispatch_interrupt),
                interrupt_handler!(0x1d, dispatch_interrupt),
                interrupt_handler!(0x1e, dispatch_interrupt),
                interrupt_handler!(0x1f, dispatch_interrupt),
                interrupt_handler!(0x20, dispatch_interrupt),
                interrupt_handler!(0x21, dispatch_interrupt),
                interrupt_handler!(0x22, dispatch_interrupt),
                interrupt_handler!(0x23, dispatch_interrupt),
                interrupt_handler!(0x24, dispatch_interrupt),
                interrupt_handler!(0x25, dispatch_interrupt),
                interrupt_handler!(0x26, dispatch_interrupt),
                interrupt_handler!(0x27, dispatch_interrupt),
                interrupt_handler!(0x28, dispatch_interrupt),
                interrupt_handler!(0x29, dispatch_interrupt),
                interrupt_handler!(0x2a, dispatch_interrupt),
                interrupt_handler!(0x2b, dispatch_interrupt),
                interrupt_handler!(0x2c, dispatch_interrupt),
                interrupt_handler!(0x2d, dispatch_interrupt),
                interrupt_handler!(0x2e, dispatch_interrupt),
                interrupt_handler!(0x2f, dispatch_interrupt),
                interrupt_handler!(0x30, dispatch_interrupt),
                interrupt_handler!(0x31, dispatch_interrupt),
                interrupt_handler!(0x32, dispatch_interrupt),
                interrupt_handler!(0x33, dispatch_interrupt),
                interrupt_handler!(0x34, dispatch_interrupt),
                interrupt_handler!(0x35, dispatch_interrupt),
                interrupt_handler!(0x36, dispatch_interrupt),
                interrupt_handler!(0x37, dispatch_interrupt),
                interrupt_handler!(0x38, dispatch_interrupt),
                interrupt_handler!(0x39, dispatch_interrupt),
                interrupt_handler!(0x3a, dispatch_interrupt),
                interrupt_handler!(0x3b, dispatch_interrupt),
                interrupt_handler!(0x3c, dispatch_interrupt),
                interrupt_handler!(0x3d, dispatch_interrupt),
                interrupt_handler!(0x3e, dispatch_interrupt),
                interrupt_handler!(0x3f, dispatch_interrupt),
                interrupt_handler!(0x40, dispatch_interrupt),
                interrupt_handler!(0x41, dispatch_interrupt),
                interrupt_handler!(0x42, dispatch_interrupt),
                interrupt_handler!(0x43, dispatch_interrupt),
                interrupt_handler!(0x44, dispatch_interrupt),
                interrupt_handler!(0x45, dispatch_interrupt),
                interrupt_handler!(0x46, dispatch_interrupt),
                interrupt_handler!(0x47, dispatch_interrupt),
                interrupt_handler!(0x48, dispatch_interrupt),
                interrupt_handler!(0x49, dispatch_interrupt),
                interrupt_handler!(0x4a, dispatch_interrupt),
                interrupt_handler!(0x4b, dispatch_interrupt),
                interrupt_handler!(0x4c, dispatch_interrupt),
                interrupt_handler!(0x4d, dispatch_interrupt),
                interrupt_handler!(0x4e, dispatch_interrupt),
                interrupt_handler!(0x4f, dispatch_interrupt),
                interrupt_handler!(0x50, dispatch_interrupt),
                interrupt_handler!(0x51, dispatch_interrupt),
                interrupt_handler!(0x52, dispatch_interrupt),
                interrupt_handler!(0x53, dispatch_interrupt),
                interrupt_handler!(0x54, dispatch_interrupt),
                interrupt_handler!(0x55, dispatch_interrupt),
                interrupt_handler!(0x56, dispatch_interrupt),
                interrupt_handler!(0x57, dispatch_interrupt),
                interrupt_handler!(0x58, dispatch_interrupt),
                interrupt_handler!(0x59, dispatch_interrupt),
                interrupt_handler!(0x5a, dispatch_interrupt),
                interrupt_handler!(0x5b, dispatch_interrupt),
                interrupt_handler!(0x5c, dispatch_interrupt),
                interrupt_handler!(0x5d, dispatch_interrupt),
                interrupt_handler!(0x5e, dispatch_interrupt),
                interrupt_handler!(0x5f, dispatch_interrupt),
                interrupt_handler!(0x60, dispatch_interrupt),
                interrupt_handler!(0x61, dispatch_interrupt),
                interrupt_handler!(0x62, dispatch_interrupt),
                interrupt_handler!(0x63, dispatch_interrupt),
                interrupt_handler!(0x64, dispatch_interrupt),
                interrupt_handler!(0x65, dispatch_interrupt),
                interrupt_handler!(0x66, dispatch_interrupt),
                interrupt_handler!(0x67, dispatch_interrupt),
                interrupt_handler!(0x68, dispatch_interrupt),
                interrupt_handler!(0x69, dispatch_interrupt),
                interrupt_handler!(0x6a, dispatch_interrupt),
                interrupt_handler!(0x6b, dispatch_interrupt),
                interrupt_handler!(0x6c, dispatch_interrupt),
                interrupt_handler!(0x6d, dispatch_interrupt),
                interrupt_handler!(0x6e, dispatch_interrupt),
                interrupt_handler!(0x6f, dispatch_interrupt),
                interrupt_handler!(0x70, dispatch_interrupt),
                interrupt_handler!(0x71, dispatch_interrupt),
                interrupt_handler!(0x72, dispatch_interrupt),
                interrupt_handler!(0x73, dispatch_interrupt),
                interrupt_handler!(0x74, dispatch_interrupt),
                interrupt_handler!(0x75, dispatch_interrupt),
                interrupt_handler!(0x76, dispatch_interrupt),
                interrupt_handler!(0x77, dispatch_interrupt),
                interrupt_handler!(0x78, dispatch_interrupt),
                interrupt_handler!(0x79, dispatch_interrupt),
                interrupt_handler!(0x7a, dispatch_interrupt),
                interrupt_handler!(0x7b, dispatch_interrupt),
                interrupt_handler!(0x7c, dispatch_interrupt),
                interrupt_handler!(0x7d, dispatch_interrupt),
                interrupt_handler!(0x7e, dispatch_interrupt),
                interrupt_handler!(0x7f, dispatch_interrupt),
                interrupt_handler!(0x80, dispatch_interrupt),
                interrupt_handler!(0x81, dispatch_interrupt),
                interrupt_handler!(0x82, dispatch_interrupt),
                interrupt_handler!(0x83, dispatch_interrupt),
                interrupt_handler!(0x84, dispatch_interrupt),
                interrupt_handler!(0x85, dispatch_interrupt),
                interrupt_handler!(0x86, dispatch_interrupt),
                interrupt_handler!(0x87, dispatch_interrupt),
                interrupt_handler!(0x88, dispatch_interrupt),
                interrupt_handler!(0x89, dispatch_interrupt),
                interrupt_handler!(0x8a, dispatch_interrupt),
                interrupt_handler!(0x8b, dispatch_interrupt),
                interrupt_handler!(0x8c, dispatch_interrupt),
                interrupt_handler!(0x8d, dispatch_interrupt),
                interrupt_handler!(0x8e, dispatch_interrupt),
                interrupt_handler!(0x8f, dispatch_interrupt),
                interrupt_handler!(0x90, dispatch_interrupt),
                interrupt_handler!(0x91, dispatch_interrupt),
                interrupt_handler!(0x92, dispatch_interrupt),
                interrupt_handler!(0x93, dispatch_interrupt),
                interrupt_handler!(0x94, dispatch_interrupt),
                interrupt_handler!(0x95, dispatch_interrupt),
                interrupt_handler!(0x96, dispatch_interrupt),
                interrupt_handler!(0x97, dispatch_interrupt),
                interrupt_handler!(0x98, dispatch_interrupt),
                interrupt_handler!(0x99, dispatch_interrupt),
                interrupt_handler!(0x9a, dispatch_interrupt),
                interrupt_handler!(0x9b, dispatch_interrupt),
                interrupt_handler!(0x9c, dispatch_interrupt),
                interrupt_handler!(0x9d, dispatch_interrupt),
                interrupt_handler!(0x9e, dispatch_interrupt),
                interrupt_handler!(0x9f, dispatch_interrupt),
                interrupt_handler!(0xa0, dispatch_interrupt),
                interrupt_handler!(0xa1, dispatch_interrupt),
                interrupt_handler!(0xa2, dispatch_interrupt),
                interrupt_handler!(0xa3, dispatch_interrupt),
                interrupt_handler!(0xa4, dispatch_interrupt),
                interrupt_handler!(0xa5, dispatch_interrupt),
                interrupt_handler!(0xa6, dispatch_interrupt),
                interrupt_handler!(0xa7, dispatch_interrupt),
                interrupt_handler!(0xa8, dispatch_interrupt),
                interrupt_handler!(0xa9, dispatch_interrupt),
                interrupt_handler!(0xaa, dispatch_interrupt),
                interrupt_handler!(0xab, dispatch_interrupt),
                interrupt_handler!(0xac, dispatch_interrupt),
                interrupt_handler!(0xad, dispatch_interrupt),
                interrupt_handler!(0xae, dispatch_interrupt),
                interrupt_handler!(0xaf, dispatch_interrupt),
                interrupt_handler!(0xb0, dispatch_interrupt),
                interrupt_handler!(0xb1, dispatch_interrupt),
                interrupt_handler!(0xb2, dispatch_interrupt),
                interrupt_handler!(0xb3, dispatch_interrupt),
                interrupt_handler!(0xb4, dispatch_interrupt),
                interrupt_handler!(0xb5, dispatch_interrupt),
                interrupt_handler!(0xb6, dispatch_interrupt),
                interrupt_handler!(0xb7, dispatch_interrupt),
                interrupt_handler!(0xb8, dispatch_interrupt),
                interrupt_handler!(0xb9, dispatch_interrupt),
                interrupt_handler!(0xba, dispatch_interrupt),
                interrupt_handler!(0xbb, dispatch_interrupt),
                interrupt_handler!(0xbc, dispatch_interrupt),
                interrupt_handler!(0xbd, dispatch_interrupt),
                interrupt_handler!(0xbe, dispatch_interrupt),
                interrupt_handler!(0xbf, dispatch_interrupt),
                interrupt_handler!(0xc0, dispatch_interrupt),
                interrupt_handler!(0xc1, dispatch_interrupt),
                interrupt_handler!(0xc2, dispatch_interrupt),
                interrupt_handler!(0xc3, dispatch_interrupt),
                interrupt_handler!(0xc4, dispatch_interrupt),
                interrupt_handler!(0xc5, dispatch_interrupt),
                interrupt_handler!(0xc6, dispatch_interrupt),
                interrupt_handler!(0xc7, dispatch_interrupt),
                interrupt_handler!(0xc8, dispatch_interrupt),
                interrupt_handler!(0xc9, dispatch_interrupt),
                interrupt_handler!(0xca, dispatch_interrupt),
                interrupt_handler!(0xcb, dispatch_interrupt),
                interrupt_handler!(0xcc, dispatch_interrupt),
                interrupt_handler!(0xcd, dispatch_interrupt),
                interrupt_handler!(0xce, dispatch_interrupt),
                interrupt_handler!(0xcf, dispatch_interrupt),
                interrupt_handler!(0xd0, dispatch_interrupt),
                interrupt_handler!(0xd1, dispatch_interrupt),
                interrupt_handler!(0xd2, dispatch_interrupt),
                interrupt_handler!(0xd3, dispatch_interrupt),
                interrupt_handler!(0xd4, dispatch_interrupt),
                interrupt_handler!(0xd5, dispatch_interrupt),
                interrupt_handler!(0xd6, dispatch_interrupt),
                interrupt_handler!(0xd7, dispatch_interrupt),
                interrupt_handler!(0xd8, dispatch_interrupt),
                interrupt_handler!(0xd9, dispatch_interrupt),
                interrupt_handler!(0xda, dispatch_interrupt),
                interrupt_handler!(0xdb, dispatch_interrupt),
                interrupt_handler!(0xdc, dispatch_interrupt),
                interrupt_handler!(0xdd, dispatch_interrupt),
                interrupt_handler!(0xde, dispatch_interrupt),
                interrupt_handler!(0xdf, dispatch_interrupt),
                interrupt_handler!(0xe0, dispatch_interrupt),
                interrupt_handler!(0xe1, dispatch_interrupt),
                interrupt_handler!(0xe2, dispatch_interrupt),
                interrupt_handler!(0xe3, dispatch_interrupt),
                interrupt_handler!(0xe4, dispatch_interrupt),
                interrupt_handler!(0xe5, dispatch_interrupt),
                interrupt_handler!(0xe6, dispatch_interrupt),
                interrupt_handler!(0xe7, dispatch_interrupt),
                interrupt_handler!(0xe8, dispatch_interrupt),
                interrupt_handler!(0xe9, dispatch_interrupt),
                interrupt_handler!(0xea, dispatch_interrupt),
                interrupt_handler!(0xeb, dispatch_interrupt),
                interrupt_handler!(0xec, dispatch_interrupt),
                interrupt_handler!(0xed, dispatch_interrupt),
                interrupt_handler!(0xee, dispatch_interrupt),
                interrupt_handler!(0xef, dispatch_interrupt),
                interrupt_handler!(0xf0, dispatch_interrupt),
                interrupt_handler!(0xf1, dispatch_interrupt),
                interrupt_handler!(0xf2, dispatch_interrupt),
                interrupt_handler!(0xf3, dispatch_interrupt),
                interrupt_handler!(0xf4, dispatch_interrupt),
                interrupt_handler!(0xf5, dispatch_interrupt),
                interrupt_handler!(0xf6, dispatch_interrupt),
                interrupt_handler!(0xf7, dispatch_interrupt),
                interrupt_handler!(0xf8, dispatch_interrupt),
                interrupt_handler!(0xf9, dispatch_interrupt),
                interrupt_handler!(0xfa, dispatch_interrupt),
                interrupt_handler!(0xfb, dispatch_interrupt),
                interrupt_handler!(0xfc, dispatch_interrupt),
                interrupt_handler!(0xfd, dispatch_interrupt),
                interrupt_handler!(0xfe, dispatch_interrupt),
                interrupt_handler!(0xff, dispatch_interrupt),
            ]
        }
    }

    /// Overwrite an entry in the IDT with a new IDT entry.
    pub fn set_entry(&mut self, index: usize, entry: IdtEntry) {
        self.entries[index] = entry;
    }

    /// Load the IDT into the CPU.
    pub fn load(&self) {
        let idt_descriptor = IdtDescriptor::new(self);
        unsafe {
            asm!(
            "lidt [{}]",
            in(reg) &idt_descriptor,
            options(nostack)
            );
        }
    }
}
