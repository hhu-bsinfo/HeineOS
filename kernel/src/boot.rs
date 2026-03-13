/*
 * Contains the entry point for the kernel, as well as all necessary module declarations.
 *
 * Author: Fabian Ruhland, Heinrich Heine University Duesseldorf, 2026-01-07
 * License: GPLv3
 */

#![no_std]
#![feature(abi_x86_interrupt)]
#![feature(unsafe_cell_access)]

// Silence compiler warnings.
// This is done to avoid overwhelming compiler output when building the OS at the beginning.
// As you move on with the course, the warnings for unused functions or parameters will become less relevant,
// as you will be implementing more and more of the kernel. You can delete the following lines to re-enable the warnings.
#![allow(dead_code)]
#![allow(unreachable_code)]
#![allow(unused_variables)]

use log::{debug, error, info};
use uefi::mem::memory_map::MemoryMapOwned;
use crate::device::framebuffer::Framebuffer;
use crate::device::terminal;
use crate::logger::Logger;

#[macro_use]
mod device;
mod library;
mod logger;
mod multiboot;
mod demo;

unsafe extern "C" {
    fn load_gdt();
}

/// Global logger instance. This instance is initialized at the start of the kernel.
/// After initialization, it is used by the `log` crate to log messages via macros like `info!()` and `error!()`.
static LOGGER: Logger = Logger::new();

#[unsafe(no_mangle)]
/// The kernel entry point.
/// This function is called from `boot.asm` after the bare minimum setup is done.
/// It sets up all necessary kernel components and then starts the scheduler.
pub extern "C" fn main(multiboot_magic: u32, multiboot: &multiboot::BootInfo) -> ! {
    // The first thing to do is to initialize the logger.
    // Afterward, we can use logging macros like `info!()` and `error!()` and panic messages will also be logged.
    if log::set_logger(&LOGGER).is_err() {
        panic!("Failed to initialize logger");
    }
    log::set_max_level(log::LevelFilter::Debug);

    // Check if the bootloader passed the correct multiboot magic number.
    // If not, panic immediately as we cannot rely on the multiboot information.
    if multiboot_magic != multiboot::MULTIBOOT2_MAGIC {
        panic!("Invalid multiboot magic number: {:#x}", multiboot_magic);
    }

    // Initialize the framebuffer. Afterward, we can draw to the screen.
    // We take the framebuffer information the bootloader provided via multiboot.
    let framebuffer_info = multiboot
        .find_tag::<multiboot::FramebufferInfo>(multiboot::TagType::FramebufferInfo)
        .expect("Missing framebuffer info");

    let framebuffer = Framebuffer::from_multiboot(framebuffer_info)
        .expect("Failed to initialize framebuffer");

    // Initialize the terminal for text output.
    // The terminal takes ownership of the framebuffer, so we cannot use it directly anymore after this point.
    // If you want to experiment with the framebuffer, do it before this line or comment this line out.
    // However, the `print!()` and `println!()` macros will not work then.
    terminal::init_terminal(framebuffer);

    // Exit UEFI boot services. At this point, the UEFI boot services are still active.
    // By exiting them, the UEFI BIOS frees up resources and hands over full control to the kernel.
    // Furthermore, we get the memory map, which we need to check which memory regions are free to use.
    let _ = exit_uefi_boot_services(multiboot);

    // Load the Global Descriptor Table (code in boot.asm)
    unsafe { load_gdt(); }

    // TODO: Call your demo code here.

    // Endless loop, as we cannot return from main().
    loop {}
}

/// Exit UEFI boot services.
/// When the kernel start, the UEFI boot services are still active.
/// This function exits the UEFI boot services and returns the memory map.
///
/// The memory map contains information about the system memory layout,
/// showing which regions are free, reserved, or used by hardware components.
fn exit_uefi_boot_services(multiboot: &multiboot::BootInfo) -> MemoryMapOwned {
    // Check if the bootloader terminated the EFI boot services.
    // The bootloader provides this information via a multiboot tag.
    // If this tag is not present, the boot services were terminated and we cannot proceed.
    if multiboot.find_tag::<multiboot::EfiBootServicesNotTerminatedTag>(multiboot::TagType::EfiBootServicesNotTerminated).is_none() {
        panic!("EFI boot services were terminated by the bootloader");
    }

    // Retrieve the EFI image handle from the multiboot information.
    // The map_or_else() function takes two closures:
    // The first one is called if the tag is not found, causing a panic.
    // The second one is called if the tag is found, where we set the image handle and system table.
    if let Some(image_handle_tag) = multiboot.find_tag::<multiboot::Efi64BitImageHandleTag>(multiboot::TagType::Efi64BitImageHandlePointer) {
        // The tag is found, we can log the image handle address.
        debug!("EFI image is located at: {:#x}", image_handle_tag.as_ptr() as usize);

        // We use the `uefi` crate to communicate with the UEFI firmware.
        // For that, we first need to set the EFI image handle and system table pointers in the `uefi` crate.
        unsafe {
            let image_handle = uefi::Handle::from_ptr(image_handle_tag.as_ptr()).expect("Failed to get EFI image handle");
            uefi::boot::set_image_handle(image_handle);
        }

        // Now we retrieve the EFI system table pointer from the multiboot information.
        multiboot.find_tag::<multiboot::Efi64BitSystemTableTag>(multiboot::TagType::Efi64BitSystemTablePointer)
            .map_or_else(|| {
                // If the tag is not found, panic with an error message.
                panic!("Missing EFI system table pointer tag");
            },|efi_system_table_tag| {
                // The tag is found, we can log the system table address and set it in the `uefi` crate.
                debug!("EFI system table is located at: {:#x}", efi_system_table_tag.as_ptr() as usize);
                unsafe { uefi::table::set_system_table(efi_system_table_tag.as_ptr()); }
            });
    } else {
        // If the tag is not found, panic with an error message.
        panic!("Missing EFI image handle pointer tag")
    }

    // If we reach this point, both the EFI image handle and system table have been set successfully.
    // We can now exit the UEFI boot services and retrieve the memory map.
    info!("Exiting UEFI boot services...");
    unsafe { uefi::boot::exit_boot_services(None) }
}

#[panic_handler]
/// The panic handler for the kernel.
/// It logs the panic information and enters an infinite loop, halting the system.
fn panic(info: &core::panic::PanicInfo) -> ! {
    error!("Kernel panic: {}", info);
    loop {}
}