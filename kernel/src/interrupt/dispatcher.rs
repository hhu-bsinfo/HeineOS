/*
 * Handles dispatching of interrupts to their respective Interrupt Service Routines (ISRs).
 * The main function is `int_disp()` which is called for every interrupt
 * and calls a registered ISR of a device driver (e.g. the keyboard).
 *
 * Author: Michael Schoetter, Heinrich Heine University Duesseldorf, 2022-03-07
 *         Fabian Ruhland, Heinrich Heine University Duesseldorf, 2026-01-14
 * License: GPLv3
 */

use alloc::{boxed::Box, vec::Vec};
use crate::device::cpu;
use crate::interrupt::idt;
use crate::interrupt::idt::InterruptStackFrame;
use crate::interrupt::isr::ISR;
use crate::library::spinlock::Spinlock;

#[derive(Debug, Copy, Clone)]
/// Enumeration of all standardized interrupt vectors.
pub enum InterruptVector {
    // CPU exceptions
    DivisionByZero = 0,
    Debug = 1,
    NonMaskableInterrupt = 2,
    Breakpoint = 3,
    Overflow = 4,
    BoundRangeExceeded = 5,
    InvalidOpcode = 6,
    DeviceNotAvailable = 7,
    DoubleFault = 8,
    CoprocessorSegmentOverrun = 9,
    InvalidTaskStateSegment = 10,
    SegmentNotPresent = 11,
    StackSegmentFault = 12,
    GeneralProtectionFault = 13,
    PageFault = 14,
    X87FloatingPointException = 16,
    AlignmentCheck = 17,
    MachineCheck = 18,
    SimdFloatingPointException = 19,
    VirtualizationException = 20,
    ControlProtectionException = 21,
    HypervisorInjectionException = 28,
    VmmCommunicationException = 29,
    SecurityException = 30,

    // Hardware interrupts
    Pit = 0x20,
    Keyboard = 0x21,
    Cascade = 0x22,
    Com2 = 0x23,
    Com1 = 0x24,
    Lpt2 = 0x25,
    Floppy = 0x26,
    Lpt1 = 0x27,
    Rtc = 0x28,
    Free1 = 0x29,
    Free2 = 0x2a,
    Free3 = 0x2b,
    Mouse = 0x2c,
    Fpu = 0x2d,
    PrimaryAta = 0x2e,
    SecondaryAta = 0x2f,
}

/// Global instance of the interrupt vector map.
static INT_VECTORS: Spinlock<IntVectors> = Spinlock::new(IntVectors::new());

/// Initialize the interrup vector map.
/// The heap must be initialized before calling this function.
pub fn init_interrupt_dispatcher() {
    INT_VECTORS.lock().init();
}

/// Force unlock the interrupt vector map.
/// This should only be used in exceptional cases.
pub unsafe fn unlock_int_vectors() {
    unsafe { INT_VECTORS.force_unlock(); }
}

/// The main interrupt dispatcher.
/// Every interrupt is routed here, if not specified otherwise in the IDT.
pub fn dispatch_interrupt(vector: u8, stack_frame: InterruptStackFrame, error_code: Option<u64>) {
    todo!("dispatch_interrupt() not implemented yet!");
}

/// The Interrupt vector map. Each ISR is registered in this map.
pub struct IntVectors {
    // Each ISR is wrapped in a Box, because the size of the ISRs is not known at compile time.
    map: Vec<Option<Box<dyn ISR>>>,
}

// Tell the compiler that IntVectors is safe to be shared between threads.
// This is ok, since we use a Spinlock to protect the map.
unsafe impl Send for IntVectors {}
unsafe impl Sync for IntVectors {}

impl IntVectors {
    /// Create a new empty ISR map.
    /// `init()` must be called before using the map.
    pub const fn new() -> Self {
        IntVectors { map: Vec::new() }
    }

    /// Fill the ISR map with IDT_SIZE empty Options.
    /// Specific ISRs can be overwritten by calling `register()`.
    pub fn init(&mut self) {
        if !self.map.is_empty() {
            panic!("ISR map is already initialized!");
        }

        for _ in 0..idt::IDT_SIZE {
            self.map.push(None);
        }
    }

    /// Register an ISR.
    /// Interrupts get disabled while registering the ISR to avoid race conditions with `dispatch_interrupt()`.
    pub fn register(vector: InterruptVector, isr: Box<dyn ISR>) {
        todo!("IntVectors::register() not implemented yet!");
    }

    /// Check if an ISR is registered for `vector`. If so, call it.
    pub fn report(&self, vector: u8) -> bool {
        todo!("IntVectors::report() not implemented yet!");
    }
}