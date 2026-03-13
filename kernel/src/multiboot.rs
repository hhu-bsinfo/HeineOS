/*
 * Contains functions to parse Multiboot2 information structures.
 * The bootloader passes information about the system and boot process
 * to the kernel via these structures.
 *
 * Multiboot2 defines various tags. The ones implemented here are:
 * - Command line
 * - Bootloader name
 * - Modules (e.g., initrd)
 * - Framebuffer information
 *
 * Author: Fabian Ruhland, Heinrich Heine University Duesseldorf, 2026-01-07
 * License: GPLv3
 */

use core::ffi::{c_void, CStr};
use core::ptr;

/// The Multiboot2 magic number passed by the bootloader.
pub const MULTIBOOT2_MAGIC: u32 = 0x36d76289;

#[repr(C, packed)]
/// The Multiboot2 boot information structure.
/// This structure contains various tags with information about the boot process.
/// The tags can be iterated over using the `find_tag()` method.
pub struct BootInfo {
    total_size: u32,
    reserved: u32,
    tags: [TagHeader; 0] // Flexible array member
}

#[repr(u32)]
#[derive(Copy, Clone, PartialEq)]
/// The different types of Multiboot2 tags.
pub enum TagType {
    CommandLine = 1,
    BootLoaderName = 2,
    Module = 3,
    FramebufferInfo = 8,
    Efi64BitSystemTablePointer = 12,
    EfiBootServicesNotTerminated = 18,
    Efi64BitImageHandlePointer = 20
}

#[repr(C, packed)]
/// The header that precedes each Multiboot2 tag.
/// It contains the type and size of the tag.
pub struct TagHeader {
    typ: TagType,
    size: u32
}

#[repr(C, packed)]
/// A Multiboot2 tag containing the command line passed to the kernel.
/// The command line is stored as a null-terminated C string,
/// which can be accessed via the `as_str()` method.
pub struct CommandLineTag {
    header: TagHeader,
    command_line: [u8; 0] // Flexible array member
}

#[repr(C, packed)]
/// A Multiboot2 tag containing the name of the bootloader.
/// The name is stored as a null-terminated C string,
/// which can be accessed via the `as_str()` method.
pub struct BootLoaderNameTag {
    header: TagHeader,
    name: [u8; 0] // Flexible array member
}

#[repr(C, packed)]
/// A Multiboot2 tag containing information about a loaded module.
/// The module name is stored as a null-terminated C string,
/// which can be accessed via the `name()` method.
pub struct ModuleTag {
    header: TagHeader,
    start_address: u32,
    end_address: u32,
    string: [u8; 0] // Flexible array member
}

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Debug)]
/// The different types of framebuffers supported by Multiboot2.
pub enum FramebufferType {
    Indexed = 0,
    RGB = 1,
    EGAText = 2
}

#[repr(C, packed)]
/// A Multiboot2 tag containing information about the framebuffer.
/// This includes the address, pitch, dimensions, bits per pixel, and type of the framebuffer.
pub struct FramebufferInfo {
    header: TagHeader,
    pub address: u64,
    pub pitch: u32,
    pub width: u32,
    pub height: u32,
    pub bpp: u8,
    pub typ: FramebufferType
}

#[repr(C, packed)]
/// A Multiboot2 tag containing a pointer to the EFI 64-bit system table.
pub struct Efi64BitSystemTableTag {
    header: TagHeader,
    address: u64
}

#[repr(C, packed)]
/// A Multiboot2 tag indicating that EFI boot services were not terminated.
/// This does not contain any additional data. It is used as a flag.
pub struct EfiBootServicesNotTerminatedTag {
    header: TagHeader
}

#[repr(C, packed)]
/// A Multiboot2 tag containing a pointer to the EFI 64-bit image handle.
pub struct Efi64BitImageHandleTag {
    header: TagHeader,
    address: u64
}

impl BootInfo {
    /// Find a specific Multiboot2 tag by its type.
    /// Returns a reference to the tag if found, or `None` if not found.
    pub fn find_tag<T>(&self, tag_type: TagType) -> Option<&T> {
        let base_addr = ptr::from_ref(self) as usize;
        let mut offset = size_of::<BootInfo>();

        // Iterate through all tags
        while offset < self.total_size as usize {
            // Calculate pointer to current tag by adding the current offset to the base pointer
            let tag_ptr = (base_addr + offset) as *const TagHeader;
            // Get reference to current tag from the raw pointer
            let tag = unsafe { tag_ptr.as_ref().unwrap() };

            let current_tag_type = tag.typ;
            let current_tag_size = tag.size;

            if current_tag_type == tag_type {
                // Found the desired tag -> Cast it to the desired type and return it
                let tag = unsafe { (tag_ptr as *const T).as_ref().unwrap() };
                return Some(tag);
            }

            if current_tag_size == 0 {
                // Found a tag with size 0, which is invalid -> Stop searching
                break;
            }

            // Add tag size to offset and align to 8 bytes to find the next tag
            offset += ((tag.size + 7) & !7) as usize;
        }
        None
    }
}

/// Some Multiboot2 tags have a dynamically sized null-terminated string at the end.
/// This function extracts that string and converts it to a Rust `&str`.
fn convert_str<'a, T>(tag_header: &'a TagHeader, string: &[u8; 0]) -> &'a str {
    let ptr = string as *const u8;
    let len = tag_header.size as usize - size_of::<T>();
    let slice = unsafe { core::slice::from_raw_parts(ptr, len) };

    CStr::from_bytes_with_nul(slice)
        .expect("Multiboot: Invalid C string")
        .to_str()
        .expect("Multiboot: C string contains invalid UTF-8")
}

impl CommandLineTag {
    /// Get the command line as a Rust `&str`.
    pub fn as_str(&self) -> &str {
        convert_str::<CommandLineTag>(&self.header, &self.command_line)
    }
}

impl BootLoaderNameTag {
    /// Get the bootloader name as a Rust `&str`.
    pub fn as_str(&self) -> &str {
        convert_str::<BootLoaderNameTag>(&self.header, &self.name)
    }
}

impl ModuleTag {
    /// Get the module name as a Rust `&str`.
    pub fn name(&self) -> &str {
        convert_str::<ModuleTag>(&self.header, &self.string)
    }

    /// Get the size of the module in bytes.
    pub fn size(&self) -> usize {
        (self.end_address - self.start_address) as usize
    }

    /// Get the module data as a byte slice.
    pub fn as_slice(&self) -> &'static [u8] {
        let ptr = self.start_address as *const u8;
        let size = self.size();
        unsafe { core::slice::from_raw_parts(ptr, size) }
    }
}

impl Efi64BitSystemTableTag {
    /// Get a raw pointer to the EFI system table.
    pub fn as_ptr(&self) -> *const uefi_raw::table::system::SystemTable {
        self.address as *const uefi_raw::table::system::SystemTable
    }
}

impl Efi64BitImageHandleTag {
    /// Get a raw pointer to the EFI image handle.
    pub fn as_ptr(&self) -> *mut c_void {
        self.address as *mut c_void
    }
}