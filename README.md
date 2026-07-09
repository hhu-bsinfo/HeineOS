# Lesson 10: Memory Management for Physical Memory

## Learning Goals
1. Learn how the kernel detects physical memory
2. Implement a memory allocator for physical page frames

## Assignment 10.1: Detect Usable Physical Memory
Before we can start implementing paging, we need to detect the usable physical memory.
Right now, we do not know how much physical memory is available and what parts of it are free.
Some parts of the physical memory may be occupied by the BIOS and, of course, our kernel code and data.

This information comes in the form of a *memory map* and is provided by the UEFI BIOS once the UEFI Boot Services are exited.
This is done by the function `exit_uefi_boot_services()` in `boot.rs`.
Other operating systems may rely on the bootloader to exit the UEFI Boot Services and provide the memory map, but in HeinOS, we do this manually.

In the [given code](https://github.com/hhu-bsinfo/HeineOS/blob/lesson-9/kernel/src/boot.rs), the function `init_physical_memory_allocator()` is already fully implemented.
It takes a reference to the memory map and initializes the physical memory allocator with the blocks that are marked as available.
The kernel image is actually not reserved in the memory map but is located inside an available block.
The function `init_physical_memory_allocator()` takes this into account and slices the available blocks accordingly.

Your task is to look at the code carefully and understand how the memory map is used to initialize the physical memory allocator.
Apart from that, you just have to call `init_physical_memory_allocator()` in `main()`, after the UEFI Boot Services have been exited, the GDT is loaded and initial pagin is set up.

*Note: Detailed information about the memory map provided by the UEFI BIOS can be found in the [UEFI specification](https://uefi.org/specifications) in chapter 7.2.3: EFI_BOOT_SERVICES.GetMemoryMap().*

## Assignment 10.2: An Allocator for Physical Page Frames
The physical memory must be managed in 4 KiB page frames. As a basis for the allocator, we can use the existing list allocator used for the kernel heap.
Of course, the allocator needs to be modified. Instead of arbitrary memory block, we now want to link free page frames, where each block may consist of multiple consecutive frames.
The metadata for free blocks is stored directly inside the free page frames.
We do not need metadata for used blocks, as all frames are 4 KiB in size and referenced by a page table entry, after we have implemented paging.

When freeing a block of physical memory, it should be inserted into the list in a sorted order.
The free blocks should always be sorted by their address.
If a freed block is adjacent to its predecessor and/or successor, they should be merged into a single block.
This prevents fragmentation of the physical memory into many small blocks.

Implement a test function that checks if the allocator works correctly.
Like with the heap allocator, a `dump_free_list()` that print the free list to the terminal is helpful.

*Important Notes:*
- *The allocator must not allocate an already reserved page frame. Make sure your allocator works correctly before continuing with the next assignments. `assert!()` might be useful here.*
- *It is recommended to fill allocated page frames with zeroes. This helps to detect illegal pointer accesses, as null pointer accesses can be detected via paging. However, this comes at a performance cost and is not required for this assignment.*

## Assignment 10.3: Use Physical Memory for the Kernel Heap
Now that we have a working allocator for physical memory, we can use it to allocate a block of memory for the kernel heap.
Right now, we just put the kernel heap at a static address and just assume (or better, *hope*) that the memory there is available.
Modify the `main()` method to allocate a block of contiguous physical memory and use it to initialize the kernel heap.
