# Lesson 11: Paging

## Learning Goals
1. Implement a basic paging mechanism
2. Execute user threads in their own address spaces


## Slides for this assignment
- Lecture 11: [Paging](https://github.com/hhu-bsinfo/HeineOS/blob/main/slides/lecture11_paging.pdf)
- Lecture 12: [Kernel Isolation](https://github.com/hhu-bsinfo/HeineOS/blob/main/slides/lecture12_isolation.pdf)

## Assignment 11.1: Kernel Page Tables
We use a 4-level paging with only 4 KiB pages (not huge 2 MiB pages).
The whole physical memory should be identity mapped (1:1).
This way, the kernel always access all physical addresses.
For that, we first need to determine the highest physical address.
In the given code, the `PfListAllocator` struct in [frames.rs](https://github.com/hhu-bsinfo/HeineOS/blob/lesson-11/kernel/src/paging/frames.rs) is extended with a new variable `max_addr`.
Modify your `free_block()` function to check if the end address of each inserted block is larger than `max_addr` and if so, update `max_addr` accordingly.

Afterward, you can implement the function `PageTable::map(&mut self, virt_addr: u64, num_pages: usize, kernel: bool, device: bool)` [pages.rs](https://github.com/hhu-bsinfo/HeineOS/blob/lesson-11/kernel/src/paging/pages.rs).
Start by replacing your `pages.rs` with the given one.
The function `pages::setup_initial_paging()` is not needed anymore after this assignment, and thus is missing the new `pages.rs`.
Remove the call to this function in `main()`. It will be replaced later on in this assignment.

`PageTable::map()` should map physical memory into a virtual address space.
A recursive implementation is recommended, but an iterative solution is also possible.
If the parameter `kernel` is set to `true`, an identity mapping (1:1) should be created.
In this case, the given virtual address `virt_addr` is the same as the physical address that is entered into the page tables on the lowest level.
If page tables on the higher levels do not exist yet, they should be created by allocating a new page frame (or using `PageTable::new()` which also allocates a page frame).
On the lowest level, no page frames need to be allocated, as the physical address is the same as `virt_addr`.
The case that `kernel` is set to `false` is handled in assignment 11.2.

For now, we leave page table entries accessible from ring 3.
This means that the `USER_ACCESSIBLE` flag should always be set.
This is necessary for now, as all our code still (even for the user threads) is still part of the kernel, but will change in a later lesson.
Furthermore, all page table entries should have the `PRESENT` and `WRITABLE` flags set.
You can use the convenience methods `PageFlags::kernel_flags()` and `PageFlags::user_flags()` to retrieve a set of the right flags for your use case.
If the parameter `device` is set to true, the `CACHE_DISABLE` should be set as well, but only on lowest level page tables.
This is used to disable caching completely for device memory (e.g. the framebuffer).

The framebuffer is also part of the physical memory, but its address is usually above the real existing memory.
For example, in QEMU we emulate 512 MiB of RAM and the framebuffer starts at 2 GiB.
This means, we need to map the framebuffer separately, if we want to be able to access it in an address space.
This is already implemented in the function `pages::map_framebuffer()`.

The function `pages::create_kernel_mapping()` creates a new virtual address spaces and uses `PageTable::map()` to identity map the whole physical memory into it.
A reference to Page Map Level 4 of the address is returned.
An address space for the kernel is stored in the static variable `pages::KERNEL_PAGE_TABLES` and can be retrieved using `pages::kernel_page_tables()`.
Replace the call to `pages::setup_initial_paging()` in your `main()` function with a call to `pages::kernel_page_tables()` and load the kernel address space using `pages::write_cr3()`.
Do not forget to also map the framebuffer into the kernel address space using `pages::map_framebuffer()`.
Otherwise, your operating system will crash as soon as it starts drawing pixels.
Make sure, that and interrupts are not activated yet, as they make debugging paging errors harder.

If anything goes wrong when creating the page tables, your kernel will very likely crash right after loading the new address space.
In this case, your best option is to pause the right before loading the CR3 register using a debugger, and look at the page tables in memory.

If this works, you should test the null pointer access.
Actually, a null pointer cannot be easily accessed in rust, as the rust runtime will catch such accesses and panic.
However, you can just another address below 4096.
Any access to the first page should cause a page fault (interrupt 14).
The address that causes the page fault is then stored in the CR2 register.
Implement a page fault handler that reads this address and prints it in a panic message.
You must register your page fault handler at the correct position in `idt.rs`.

*Note: Detailed information for this task is provided in the [Intel Software Developer’s Manual](https://www.intel.com/content/www/us/en/developer/articles/technical/intel-sdm.html) Volume 3 in chapter 5.5: 4-Level and 5-Level Paging*.

## Assignment 11.2: Page Tables for User Threads
We can now start isolating the user mode threads. For now, we only isolate the user stacks.
Each thread will have its own address space, with its user stack mapped into it.
The user stack should always be mapped to the same address (64 TiB, see [consts.rs](https://github.com/hhu-bsinfo/HeineOS/blob/lesson-11/kernel/src/consts.rs)).
It should not be allocated on the kernel heap anymore.
For this purpose, a mapping needs to be created in the thread's address space using `pages::map_user_stack()`.
This function should call `PageTable::map()` with the parameter `kernel` set to `false`.
In this case `PageTable::map()` must not create an identity mapping on the lowest level page table, but should allocate a new page frame for each level 1 page table entry.

As you may have noticed `pages::map_user_stack()` returns a `Vec<u64, NoOpAllocator>` instead of normal `Vec<u64>`.
This is due to the fact, that a `Vec` always frees its memory when it is dropped.
In our case, a thread (and with it kernel and user stack) is dropped in `scheduler::cleanup_terminated_threads()` after it has exited (see assignment 5.5).
However, a user stack's memory must not be freed into the kernel heap, as all user stacks refer to same virtual address outside the kernel.
This would break the kernel heap and cause the operating system to crash sooner or later.
To avoid this, we use the `NoOpAllocator` from [kernel/src/allocator/noop.rs](https://github.com/hhu-bsinfo/HeineOS/blob/lesson-11/kernel/src/allocator/noop.rs).
This allocator does nothing and always return an error result when trying to allocate memory.
Freeing memory will just log a message but also does nothing related to the freed memory.
The user stack `Vec` is created using `Vec::from_raw_parts_in()`, which create a `Vec` from a pointer and an allocator.
Now, when a user stack is dropped, no memory is freed at all, as the `NoOpAllocator` does nothing.
However, we still need to free the physical memory occupied by the user stack. This is done in a later assignment.

When a new user thread is created, it should now create a new address space for this thread and map the user stack into it using `pages::map_user_stack()`.
The Page Map Level 4 of this address space must be stored in `Thread` struct, which now contains a new variable `pml4_table` (see the [given code]https://github.com/hhu-bsinfo/HeineOS/blob/lesson-11/kernel/src/thread/thread.rs).
For kernel threads, the user stack should be set to `None` and `pml4_table` to the kernel address space.
Furthermore, the framebuffer must be mapped into each user thread.

When switching threads, we must now also switch to the address space of the next thread.
For that, the function `thread_switch()` gets a fourth parameter `next_pml4` (stored in the `rcx` register).
Additionally, `Thread::start()` must also load the address space of the starting thread.

Test this assignment using two (or more) user threads.
You can, for example, use threads that print a counter, like in previous assignments.
Use the debugger to verify, that the threads actually run in different address spaces and their user stack are located at the same virtual address.

![Virtual Address Space Mapping](https://raw.githubusercontent.com/hhu-bsinfo/HeineOS/refs/heads/main/media/lesson-11/mapping.png)

*Final Notes:*
- *Code isolation follows in the next lesson. We already have system calls, but for now, we can still call kernel function from user mode threads*
- *Heap isolation also follows in a later assignment. For now, allocations done by user mode threads still us the kernel heap.*
- *This stil works, because we do not (yet) protect the pages containing the kernel code and data.*
