/*
 * Contains demos for the PCI bus scan and reading the MAC address of a RTL8139 Ethernet card.
 *
 * Author: Fabian Ruhland, Heinrich Heine University Duesseldorf, 2026-04-02
 * License: GPLv3
 */
use crate::device::cpu::IoPort;
use crate::device::pci::{pci_bus, Command};
use crate::device::terminal::terminal;
use crate::library::input;

pub fn print_pci_devices() {
    terminal().lock().clear();
    println!("PCI Demo:\n");

    for device in pci_bus().iter() {
        println!("Found PCI device {:04x}:{:04x}", device.read_vendor_id(), device.read_device_id());
    }

    println!("\nPress 'Enter' to exit...");
    input::wait_for_return();
}

pub fn rtl8139_demo() {
    terminal().lock().clear();
    println!("RTL8139 Demo:\n");

    let rtl8139 = pci_bus().iter().find(|device| {
        device.read_vendor_id() == 0x10ec && device.read_device_id() == 0x8139
    });

    if let Some(rtl8139) = rtl8139 {
        println!("Found RTL8139 device!");

        // Read the I/O base address from BAR0
        let bar0 = rtl8139.read_bar(0);
        if bar0 & 0x1 == 0 {
            // The address in BAR0 is a 32-bit memory-mapped I/O address.
            // This means that the registers are accessed via memory addresses instead of I/O ports.
            // The card emulated by QEMU uses 16-bit I/O ports,
            // so this code path is never executed in QEMU and is just here as a showcase.
            let mmio_base = bar0 & 0xfffffff0;
            println!("RTL8139 MMIO base address: 0x{:x}", mmio_base);

            // Enable MMIO access by setting the correct command bits in the PCI command register
            rtl8139.write_command(rtl8139.read_command() | Command::MemEnable as u16);

            // Read mac address from the RTL8139 registers -> Always at offset 0x00-0x05
            // MMIO access is done via volatile reads to ensure the compiler does not optimize them away
            let mac_address_ptr = (mmio_base) as *const u8;
            let mac_address = unsafe {[
                mac_address_ptr.add(0).read_volatile(),
                mac_address_ptr.add(1).read_volatile(),
                mac_address_ptr.add(2).read_volatile(),
                mac_address_ptr.add(3).read_volatile(),
                mac_address_ptr.add(4).read_volatile(),
                mac_address_ptr.add(5).read_volatile()
            ]};
            println!("MAC address: {:x?}", mac_address);
        } else {
            // The address in BAR0 is a 16-bit I/O port address
            let io_base = (bar0 & 0xfffc) as u16;
            println!("RTL8139 I/O base address: 0x{:x}", io_base);

            // Enable I/O access by setting the correct command bits in the PCI command register
            rtl8139.write_command(rtl8139.read_command() | Command::IoEnable as u16);

            // Read mac address from the RTL8139 registers -> Always at offset 0x00-0x05
            let mac_address = unsafe {[
                IoPort::new(io_base + 0).inb(),
                IoPort::new(io_base + 1).inb(),
                IoPort::new(io_base + 2).inb(),
                IoPort::new(io_base + 3).inb(),
                IoPort::new(io_base + 4).inb(),
                IoPort::new(io_base + 5).inb()
            ]};
            println!("MAC address: {:x?}", mac_address);
        }
    } else {
        println!("No RTL8139 device found!");
    }

    println!("\nPress 'Enter' to exit...");
    input::wait_for_return();
}