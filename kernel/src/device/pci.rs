/*
 * Code to access the PCI bus.
 * The bus can be scanned for devices, and the configuration space of each device can be read and written.
 *
 * Author: Fabian Ruhland, Heinrich Heine University Duesseldorf, 2026-04-02
 * License: GPLv3
 */
use alloc::vec::Vec;
use crate::device::cpu::IoPort;
use crate::library::once::Once;
use crate::library::spinlock::Spinlock;

// PCI constants
const MAX_DEVICES_PER_BUS: u8 = 32;
const MAX_FUNCTIONS_PER_DEVICE: u8 = 8;
const INVALID_VENDOR_ID: u16 = 0xffff;

// PCI configuration space ports
const CONFIG_ADDRESS_PORT: u16 = 0x0cf8;
const CONFIG_DATA_PORT: u16 = 0x0cfc;

/// Global PCI bus instance, used to access the PCI configuration space.
static PCI_BUS: Once<PciBus> = Once::new();

/// Global access to the PCI bus.
/// On its first call, it initializes the `PCI_BUS` instance and scans the PCI bus for devices.
pub fn pci_bus() -> &'static PciBus {
    PCI_BUS.init(|| {
        let mut pci = PciBus::new();
        pci.scan();

        pci
    })
}

/// Representation of the PCI bus.
/// See the OSDev wiki for details: https://wiki.osdev.org/PCI
pub struct PciBus {
    registers: Spinlock<PciBusRegisters>,
    devices: Vec<PciDevice>
}

/// Registers used for PCI configuration space access.
/// They are capsuled in a separate struct, so they can be guarded by a mutex in `PciBus`.
struct PciBusRegisters {
    config_address_port: IoPort,
    config_data_port: IoPort
}

/// Representation of a PCI device.
/// The `bus`, `device`, and `function` fields are used to identify the device in the PCI configuration space.
pub struct PciDevice {
    bus: u8,
    device: u8,
    function: u8
}

/// Bit flags for the command register of a PCI device.
#[repr(u16)]
pub enum Command {
    IoEnable = 0x0001, // Enable I/O space access
    MemEnable = 0x0002, // Enable memory space access
    BusMasterEnable = 0x0004, // Enable bus mastering
    SpecialCycle = 0x0008, // Special cycle
    MemoryWriteInvalidate = 0x0010, // Memory write and invalidate
    PaletteSnoop = 0x0020, // Palette snooping
    ParityErrorResponse = 0x0040, // Parity error response
    SERREnable = 0x0100, // SERR# enable
    FastBackToBackEnable = 0x0200, // Fast back-to-back transactions enable
    InterruptDisable = 0x0400 // Interrupt disable
}

/// Bit flags for the status register of a PCI device.
#[repr(u16)]
pub enum Status {
    InterruptStatus = 0x0008, // Interrupt status
    CapabilitiesList = 0x0010, // Capabilities list
    MasterDataParityError = 0x0100, // Master data parity error
    SignaledTargetAbort = 0x0800, // Signaled target abort
    ReceivedTargetAbort = 0x1000, // Received target abort
    ReceivedMasterAbort = 0x2000, // Received master abort
    SignaledSystemError = 0x4000, // Signaled system error
    DetectedParityError = 0x8000 // Detected parity error
}

/// PCI configuration space registers for a device.
/// These can be used as offset parameters in the read and write methods of `PciDevice`.
#[repr(u8)]
pub enum Register {
    VendorId = 0x00,
    DeviceId = 0x02,
    Command = 0x04,
    Status = 0x06,
    Revision = 0x08,
    ProgrammingInterface = 0x09,
    Subclass = 0x0a,
    Class = 0x0b,
    CacheLineSize = 0x0c,
    MasterLatencyTimer = 0x0d,
    HeaderType = 0x0e,
    Bist = 0x0f,
    BaseAddress0 = 0x10,
    BaseAddress1 = 0x14,
    BaseAddress2 = 0x18,
    BaseAddress3 = 0x1C,
    BaseAddress4 = 0x20,
    BaseAddress5 = 0x24,
    CardbusCisPointer = 0x28,
    SubsystemVendorId = 0x2c,
    SubsystemId = 0x2e,
    ExpansionRomBaseAddress = 0x30,
    CapabilitiesPointer = 0x34,
    InterruptLine = 0x3c,
    InterruptPin = 0x3d,
    MinGrant = 0x3e,
    MaxLatency = 0x3f,
    SecondaryBus = 0x19
}

/// PCI class codes, used to identify the type of device.
#[repr(u8)]
pub enum Class {
    Unclassified = 0x00,
    MassStorage = 0x01,
    NetworkController = 0x02,
    DisplayController = 0x03,
    MultimediaController = 0x04,
    MemoryController = 0x05,
    Bridge = 0x06,
    SimpleCommunicationController = 0x07,
    BaseSystemPeripheral = 0x08,
    InputDeviceController = 0x09,
    DockingStation = 0x0a,
    Processor = 0x0b,
    SerialBusController = 0x0c,
    WirelessController = 0x0d,
    IntelligentController = 0x0e,
    SatelliteCommunicationController = 0x0f,
    EncryptionController = 0x10,
    SignalProcessingController = 0x11,
    ProcessingAccelerator = 0x12,
    NonEssentialInstrumentation = 0x13,
    CoProcessor = 0x40,
    Unassigned = 0xff
}

/// Subclass codes for the `Bridge` class.
#[repr(u8)]
enum Bridge {
    Host = 0x00,
    Isa = 0x01,
    Eisa = 0x02,
    Mca = 0x03,
    PciToPci = 0x04,
    Pcmcia = 0x05,
    Nubus = 0x06,
    Cardbus = 0x07,
    Raceway = 0x08,
    PciToPciHost = 0x09,
    InfinibandToPciHost = 0x0a,
    Other = 0x80
}

impl PciBus {
    /// Create a new PCI bus instance
    const fn new() -> Self {
        PciBus {
            registers: Spinlock::new(PciBusRegisters {
                config_address_port: IoPort::new(CONFIG_ADDRESS_PORT),
                config_data_port: IoPort::new(CONFIG_DATA_PORT)
            }),
            devices: Vec::new()
        }
    }

    /// Get an iterator over the PCI devices on the bus.
    pub fn iter(&self) -> impl Iterator<Item = &PciDevice> {
        self.devices.iter()
    }

    /// Scan the PCI bus for devices.
    /// The PCI host controller can itself be a multi-function device,
    /// with each function representing a unique "bus" in the PCI hierarchy.
    fn scan(&mut self) {
        // Check the header type of the host controller. If it is a multi-function device,
        // there are multiple host controllers available at bus 0, device 0, function 0-7.
        // Otherwise, there is only one host controller at bus 0, device 0, function 0.
        let header_type = self.read8(0, 0, 0, Register::HeaderType as u8);
        if header_type & 0x80 == 0 {
            // Single-function device, scan only bus 0, device 0
            self.scan_bus(0);
        } else {
            // Multi-function device, scan all functions
            for function in 0..MAX_FUNCTIONS_PER_DEVICE {
                let vendor_id = self.read16(0, 0, function, Register::VendorId as u8);
                if vendor_id != INVALID_VENDOR_ID {
                    break;
                }

                self.scan_bus(function);
            }
        }
    }

    /// Search for devices on a specific at specified bus number.
    fn scan_bus(&mut self, bus: u8) {
        for device in 0..MAX_DEVICES_PER_BUS {
            self.check_device(bus, device)
        }
    }

    /// Check if a device exists at the specified bus and device number.
    fn check_device(&mut self, bus: u8, device: u8) {
        let vendor_id = self.read16(bus, device, 0, Register::VendorId as u8);
        if vendor_id == INVALID_VENDOR_ID {
            // No device found at this address
            return;
        }

        self.check_function(bus, device, 0);

        let header_type = self.read8(bus, device, 0, Register::HeaderType as u8);
        if header_type & 0x80 != 0 {
            // Multi-function device, check all functions
            for function in 1..MAX_FUNCTIONS_PER_DEVICE {
                let vendor_id = self.read16(bus, device, function, Register::VendorId as u8);
                if vendor_id == INVALID_VENDOR_ID {
                    continue;
                }

                self.check_function(bus, device, function);
            }
        }
    }

    /// Check a specific function of a device.
    /// If the function is a PCI-to-PCI bridge, it scans the secondary bus.
    /// Otherwise, it adds the device to the list of devices.
    fn check_function(&mut self, bus: u8, device: u8, function: u8) {
        let base_class = self.read8(bus, device, function, Register::Class as u8);
        let sub_class = self.read8(bus, device, function, Register::Subclass as u8);

        if base_class == Class::Bridge as u8 && sub_class == Bridge::PciToPci as u8 {
            // This is a PCI-to-PCI bridge, we need to scan the secondary bus
            let secondary_bus = self.read8(bus, device, function, Register::SecondaryBus as u8);
            self.scan_bus(secondary_bus);
        } else {
            // Print basic information about the device
            let device = PciDevice::new(bus, device, function);
            self.devices.push(device);
        }
    }

    /// Calculate an address in the PCI configuration space for a specific device.
    const fn calc_config_address(bus: u8, device: u8, function: u8, offset: u8) -> u32 {
        (1 << 31) // Enable bit
            | ((bus as u32) << 16) // Bus number
            | ((device as u32) << 11) // Device number
            | ((function as u32) << 8) // Function number
            | ((offset as u32) & 0xfc) // Register offset (must be multiple of 4)
    }

    /// Read a 32-bit value from the PCI configuration space.
    fn read32(&self, bus: u8, device: u8, function: u8, offset: u8) -> u32 {
        let address = PciBus::calc_config_address(bus, device, function, offset);
        let mut registers = self.registers.lock();

        unsafe {
            registers.config_address_port.outdw(address);
            registers.config_data_port.indw()
        }
    }

    /// Write a 32-bit value to the PCI configuration space.
    fn write32(&self, bus: u8, device: u8, function: u8, offset: u8, value: u32) {
        let address = PciBus::calc_config_address(bus, device, function, offset);
        let mut registers = self.registers.lock();

        unsafe {
            registers.config_address_port.outdw(address);
            registers.config_data_port.outdw(value);
        }
    }

    /// Read a 16-bit value from the PCI configuration space.
    fn read16(&self, bus: u8, device: u8, function: u8, offset: u8) -> u16 {
        ((self.read32(bus, device, function, offset) >> ((offset & 0x02) * 8)) & 0xffff) as u16
    }

    /// Write a 16-bit value to the PCI configuration space.
    fn write16(&self, bus: u8, device: u8, function: u8, offset: u8, value: u16) {
        let mut data = self.read32(bus, device, function, offset);
        data &= !(0xffff << ((offset & 0x02) * 8)); // Clear the bits we want to write
        data |= (value as u32) << ((offset & 0x02) * 8); // Set the new value

        self.write32(bus, device, function, offset, data);
    }

    /// Read an 8-bit value from the PCI configuration space.
    fn read8(&self, bus: u8, device: u8, function: u8, offset: u8) -> u8 {
        ((self.read32(bus, device, function, offset) >> ((offset & 0x03) * 8)) & 0xff) as u8
    }

    /// Write an 8-bit value to the PCI configuration space.
    fn write8(&self, bus: u8, device: u8, function: u8, offset: u8, value: u8) {
        let mut data = self.read32(bus, device, function, offset);
        data &= !(0xff << ((offset & 0x03) * 8)); // Clear the bits we want to write
        data |= (value as u32) << ((offset & 0x03) * 8); // Set the new value

        self.write32(bus, device, function, offset, data);
    }
}

impl PciDevice {
    /// Create a new PCI device instance.
    const fn new(bus: u8, device: u8, function: u8) -> Self {
        PciDevice {
            bus,
            device,
            function
        }
    }

    /// Read an 8-bit value from the PCI configuration space at the specified offset relative to the device.
    pub unsafe fn read8(&self, offset: u8) -> u8 {
        pci_bus().read8(self.bus, self.device, self.function, offset)
    }

    /// Write an 8-bit value to the PCI configuration space at the specified offset relative to the device.
    pub unsafe fn write8(&self, offset: u8, value: u8) {
        pci_bus().write8(self.bus, self.device, self.function, offset, value);
    }

    /// Read a 16-bit value from the PCI configuration space at the specified offset relative to the device.
    pub unsafe fn read16(&self, offset: u8) -> u16 {
        pci_bus().read16(self.bus, self.device, self.function, offset)
    }

    /// Write a 16-bit value to the PCI configuration space at the specified offset relative to the device.
    pub unsafe fn write16(&self, offset: u8, value: u16) {
        pci_bus().write16(self.bus, self.device, self.function, offset, value);
    }

    /// Read a 32-bit value to the PCI configuration space at the specified offset relative to the device.
    pub unsafe fn read32(&self, offset: u8) -> u32 {
        pci_bus().read32(self.bus, self.device, self.function, offset)
    }

    /// Write a 32-bit value to the PCI configuration space at the specified offset relative to the device.
    pub unsafe fn write32(&self, offset: u8, value: u32) {
        pci_bus().write32(self.bus, self.device, self.function, offset, value);
    }

    /// Read the vendor ID of the PCI device.
    pub fn read_vendor_id(&self) -> u16 {
        unsafe { self.read16(Register::VendorId as u8) }
    }

    /// Read the device ID of the PCI device.
    pub fn read_device_id(&self) -> u16 {
        unsafe { self.read16(Register::DeviceId as u8) }
    }

    /// Read the class code of the PCI device.
    pub fn read_class(&self) -> u8 {
        unsafe { self.read8(Register::Class as u8) }
    }

    /// Read the subclass code of the PCI device.
    pub fn read_subclass(&self) -> u8 {
        unsafe { self.read8(Register::Subclass as u8) }
    }

    /// Read the revision number of the PCI device.
    pub fn read_revision(&self) -> u8 {
        unsafe { self.read8(Register::Revision as u8) }
    }

    /// Read the programming interface of the PCI device.
    pub fn read_programming_interface(&self) -> u8 {
        unsafe { self.read8(Register::ProgrammingInterface as u8) }
    }

    /// Read the subsystem vendor ID of the PCI device.
    pub fn read_subsystem_vendor_id(&self) -> u16 {
        unsafe { self.read16(Register::SubsystemVendorId as u8) }
    }

    /// Read the subsystem ID of the PCI device.
    pub fn read_subsystem_id(&self) -> u16 {
        unsafe { self.read16(Register::SubsystemId as u8) }
    }

    /// Read the interrupt pin of the PCI device.
    pub fn read_interrupt_line(&self) -> u8 {
        unsafe { self.read8(Register::InterruptLine as u8) }
    }

    /// Read the command register of the PCI device.
    pub fn read_command(&self) -> u16 {
        unsafe { self.read16(Register::Command as u8) }
    }

    /// Write the command register of the PCI device.
    pub fn write_command(&self, value: u16) {
        unsafe { self.write16(Register::Command as u8, value); }
    }

    /// Read a base address register (BAR) of the PCI device.
    /// The `index` parameter specifies which BAR to read (0-5).
    pub fn read_bar(&self, index: u8) -> u32 {
        assert!(index < 6, "Invalid BAR index: {}", index);
        let offset = Register::BaseAddress0 as u8 + index * 4;
        unsafe { self.read32(offset) }
    }

    /// Write a value to a base address register (BAR) of the PCI device.
    /// The `index` parameter specifies which BAR to write (0-5).
    pub fn write_bar(&self, index: u8, value: u32) {
        assert!(index < 6, "Invalid BAR index: {}", index);
        let offset = Register::BaseAddress0 as u8 + index * 4;
        unsafe { self.write32(offset, value); }
    }
}