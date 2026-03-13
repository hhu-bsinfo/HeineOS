/*
 * A basic driver for the serial port, only supporting output.
 *
 * Author: Michael Schoetter, Heinrich Heine University Duesseldorf, 2023-03-07
 *         Fabian Ruhland, Heinrich Heine University Duesseldorf, 2026-01-07
 */

use core::fmt;
use crate::library::spinlock::Spinlock;
use crate::device::cpu::IoPort;

/// Standard COM port for kernel output via the logger
pub static COM1: Spinlock<ComPort> = Spinlock::new(ComPort::new(ComBaseAddress::Com1));

#[allow(dead_code)]
#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u16)]
/// Standardized IO-Port base addresses for the first four COM ports.
/// We usually only use `Com1`.
pub enum ComBaseAddress {
    Com1 = 0x3f8,
    Com2 = 0x2f8,
    Com3 = 0x3e8,
    Com4 = 0x2e8,
}

/// Struct representing a COM port
pub struct ComPort {
    /// IO-port where output is written to
    data_port: IoPort
}

impl ComPort {
    /// Create a new COM port
    pub const fn new(base_addr: ComBaseAddress) -> ComPort {
        ComPort {
            data_port: IoPort::new(base_addr as u16)
        }
    }

    /// Write a single byte to the COM port.
    pub fn write_byte(&mut self, byte: u8) {
        todo!("ComPort::write_byte() not implemented yet");
    }
}

// Implement the `fmt::Write` trait for the serial port to support formatted output.
// We only need to implement the `write_str()` method, which writes a string to the serial port.
// This allows formatted output via the `write_fmt()` method provided by the `fmt::Write` trait.
impl fmt::Write for ComPort {
    /// Write a string to the COM port by iterating over each byte in the string and writing it using `write_byte()`.
    fn write_str(&mut self, s: &str) -> fmt::Result {
        // TODO: Write the string using write_byte()
        Ok(())
    }
}
