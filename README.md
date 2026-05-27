# Lesson 8: User Mode Threads

This is the first assignment of the course "Isolation and Protection in Operating Systems" (Isolation und Schutz in Betriebssystemen).
If you have made it through the first course "OS Development" (Betriebssystementwicklung) you can continue with your existing code.
If you have not heard the first course yet, or do not want to use your existing code, a working operating system is provided at the semester start.

## Learning Goals
1. Understand how application code can be executed in user mode (ring 3) on x86 platforms
2. Prohibit user mode threads from executing privileged instructions

## Assignment 8.1: Global Descriptor Table (GDT)
If you continue developing your operating system from lessons 1–7, start by replacing your `boot.asm` file with the one provided in the [given code](https://github.com/hhu-bsinfo/HeineOS/blob/lesson-8/kernel/src/boot.asm).

Our operating system already has a GDT, it only contains entries for the *kernel* code and data segments (ring 0).
Your task is to add two additional entries for *user mode* code and data segments (ring 3).
The GDT is located at the label `gdt`. Furthermore, the limit field of the GDT descriptor needs to be adjusted.
You can find the GDT descriptor at the label `gdt_descriptor`.

It is not really possible to test the changes done in this assignment, but the operating system should still work as before.

## Assignment 8.2: Task State Segment (TSS)
Next, we need a TSS that points to the kernel stack. If a thread runs in ring 3 and is interrupted,
the CPU loads the stack pointer from the `rsp0` entry of the loaded TSS. After the interrupt is fully handled,
the user mode stack pointer is restored.

In the given `boot.asm` file, a TSS is already defined (without I/O-bitmap, as we do not want to allow any I/O-port accesses in user mode)
at the label `tss`. Since we only use a single CPU core, one TSS is enough for our operating system.

For the TSS to be usable, you need to add a TSS descriptor to the GDT.
Most information can be entered directly into the TSS descriptor (e.g., flags, limit),
but the base address needs to be set at runtime and should be zeroed for now.

*CAUTION: The TSS descriptor has double the size of a normal GDT entry*

The TSS address is defined by the `tss` label, but the descriptor splits the address into three parts,
making it impossible to directly enter it into the GDT descriptor.
Because of this, we set the base address at runtime using the function `tss_set_base_address`.
Your task is to implement this function in assembly language, so that splits the TSS address into three parts
and writes each part at the correct position in the TSS descriptor.
To get the TSS descriptor address, you should define a label (e.g., `tss_descriptor`) at the corresponding position in the GDT.

Next, we need a function to set the `rsp0` entry of the TSS. Implement `tss_set_rsp0` in assembly language.
Note that this function is already called in the `start` function of `boot.asm`.
Later we will call this function from our Rust code to set the stack pointer during each thread switch.

At last, you need to load the TSS register (TSSR) by using the instruction `ltr`.
The correct position to execute this instruction is already marked by a comment in `boot.asm`.

Memory for the initial stack is defined by the label `init_stack`. This stack is used for the boot process until the scheduler is started.
Afterward, each thread has its own kernel and user stacks (allocated on the heap), and the initial stack is no longer needed.

Again, this assignment is not really testable, but the operating system should still work as before.
You can verify that the TSS is loaded correctly by opening `View` -> `compatmonitor0` in QEMU and typing `info registers`.
This prints all CPU registers, including the Task Register (`TR`), allowing you to check if the shown values are plausible.

![CPU registers shown in QEMU](https://raw.githubusercontent.com/hhu-bsinfo/HeineOS/refs/heads/main/media/lesson-8/qemu_registers.png).

*Note: Detailed information for this task is provided in the [Intel Software Developer’s Manual](https://www.intel.com/content/www/us/en/developer/articles/technical/intel-sdm.html) Volume 3 in Chapter 9.2: Task Management Data Structures*

## Assignment 8.3: User Mode Threads
User mode threads (running in ring 3) always have two stacks: One for user mode and one for kernel mode.
For the sake of simplicity, we always allocate two stacks for each thread, even for pure kernel threads.
To achieve this, the struct `Thread` in `kernel/thread/thread.rs` needs to be extended with a new `Vec<u64>` for the user stack
and a `bool` flag to indicate whether the thread is running in user mode. The existing `Thread::new()` function
should be renamed to `Thread::new_kernel_thread()` and only be used to create pure kernel threads.
A new function `Thread::new_user_thread()` should be implemented, which sets the `is_kernel_thread` flag to `false`.
Furhtermore, `Thread::kickoff()` should now be named `Thread::kickoff_kernel_thread()` and `Thread::prepare_stack()` should be renamed to `Thread::prepare_kernel_stack()`.

The function `thread_switch()` gets an addidiontal third parameter called `next_stack_end`, which contains the next stack's end address.
This is the address, that the `rsp0` entry of the TSS should point to.
As it is the third parameter, it is passed to `thread_switch()` in the `rdx` register (as specified by the System V ABI).
You must call `tss_set_rsp0()` inside of `thread_switch()`, directly before setting `rsp` to the next thread's stack.
When calling `tss_set_rsp0()`, the address should be passed in the `rdi` register.

We do not have to care about managing the user mode stack pointer when switching between threads
because it is automatically saved and restored by the CPU when an interrupt occurs in ring 3.

*CAUTION: This means that user mode threads are currently not allowed to call `Scheduler::yield_cpu()` and `Scheduler::exit()` manually.
If you call `Scheduler::yield_cpu()` in your `pit::wait()` function, you also cannot call `pit::wait()` in user mode threads.
This will be resolved at a later point, when we implement system calls for user mode threads.*

The existing thread start mechanism using `Thread::prepare_kernel_stack()` and `Thread::start()` remains unchanged.
This means every thread starts in kernel mode (ring 0). However, the function `Thread::kickoff_kernel_thread()` must be
extended to call `tss_set_rsp0()` right at its beginning, so that the stack pointer in the TSS is set correctly for the freshly started thread.

Next, you have to implement the function `Thread::switch_to_usermode()`. It should use the assmebly function `thread_user_start()`
to switch from ring 0 to ring 3 via the `iretq` instruction (there is no other way to achieve this).
`Thread::switch_to_usermode()` must build a stack frame on the kernel stack, that looks exactly like an interrupt with privilege level change has occured.
The stack frame must not contain an error code. This sould cause the `iretq` instruction to return into `Thread::kickoff_user_thread()` with the CPU running in ring 3.

Before you can actually run a thread in user mode, you need to call the new function `setup_initial_paging()`, given in [kernel/paging/pages.rs](https://github.com/hhu-bsinfo/HeineOS/blob/lesson-8/kernel/paging/pages.rs).
The UEFI/bootloader has already set up initial page tables for us, with the whole memory mapped 1:1 (identity mapping).
Because of this, we did not have to care about paging until now. However, the page tables are set up in a way that only the kernel is allowed to access memory.
For user mode threads to be able to access memory, the `USER` flag must be set in each page table entry.
The function `setup_initial_paging()` does this for you and is already fully implemented.
You have to call it inside the kernel's `main()` function, right after `load_gdt()`.
This is only a temporary solution, as we will implement our own page tables in a later assignment.

It is best to test your changes with only a single thread running. For example, you could let the idle thread run in user mode
(of course, you need to reverse this change and let the idle thread run in kernel mode again, once you are done with this first test).

To test if a thread is running in user mode, you have two options:
 * Try accessing an I/O-port -> This should result in a General Protection Fault (GPF)
 * Set a breakpoint in `kickoff_user_thread()` and check if `RPL` is set to 3 in the `CS` register using the GDB command `info registers cs`.

*Note: Detailed information for this task is provided in the [Intel Software Developer’s Manual](https://www.intel.com/content/www/us/en/developer/articles/technical/intel-sdm.html) Volume 3 in Chapter 7.12: Exception and Interrupt Handling*

### Test Scenario

In [demo/lesson8.rs] you already find a test program that is ready to use, which runs a kernel and a user mode thread.
Each thread fills its own line in the terminal either with the character `K` or `U`.
For an extended test scenario, try starting multiple user threads and check if the ring changes are still working correctly.
Use the debugger to check if the kernel thread is still running in ring 0 and the user threads are still running in ring 3 after multiple loop iterations.

| ![User-Thread Test 1](https://raw.githubusercontent.com/hhu-bsinfo/HeineOS/refs/heads/main/media/lesson-8/user_threads1.png) |
|:--:|
| One kernel and one user mode thread |

| ![User-Thread Test 2](https://raw.githubusercontent.com/hhu-bsinfo/HeineOS/refs/heads/main/media/lesson-8/user_threads2.png) |
|:--:|
| Two kernel and three user mode threads |