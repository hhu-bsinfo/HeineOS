# Lesson 9: System Calls

## Learning Goals
1. Understand how system calls work and allow us to switch from ring 3 to ring 0 in a controlled manner
2. Implement several system calls to expose kernel APIs to user mode threads

## Slides for this assignment
- Lecture 10: [System Calls](https://github.com/hhu-bsinfo/HeineOS/blob/main/slides/lecture10_system_calls.pdf)

## Assignment 9.1: Interrupt Descriptor Table (IDT)
A fundamental approach to executing system calls is to trigger a software interrupt (e.g., 0x80 in Linux).
This automatically switches the CPU to ring 0. However, we need a corresponding interrupt handler in our IDT.

Currently, our IDT is filled with interrupt gates (256 entries), which all have their privilege level (DPL) set to 0
and point the same function `dispatch_interrupt()`. To realize system calls via software interrupts,
we need a trap gate with DPL = 3 to allow user mode threads to trigger this interrupt.
For this, we overwrite the existing IDT entry at index (vector number) 0x80.

Implement a new function `new_trap_gate()` for the struct `IdtEntry` in [kernel/src/interrupts/idt.rs](https://github.com/hhu-bsinfo/HeineOS/blob/lesson-9/kernel/src/interrupts/idt.rs), that creates a new trap gate with DPL = 3.

*CAUTION: In the given code, the existing function `IdtEntry::new()` has been renamed to `IdtEntry::new_interrupt_gate()`*

Afterward, a new trap gate must be installed in the IDT.
Use the function `IdtEntry::syscall_gate()` to create a new trap gate with DPL = 3, pointing to the function `syscall::dispatcher::dispatch_syscall()`.

*Note: Detailed information for this task is provided in the [Intel Software Developer’s Manual](https://www.intel.com/content/www/us/en/developer/articles/technical/intel-sdm.html) Volume 3 in chapter 7.11: IDT Descriptors*

## Assignment 9.2: System Call Handler
Next, we need to implement the syscall handler `dispatch_syscall()` in [kernel/src/syscall/dispatcher.rs](https://github.com/hhu-bsinfo/HeineOS/blob/lesson-9/kernel/src/syscall/dispatcher.rs).
Implement the missing code in `dispatch_syscall()`, so that a system call is executed in the following way:
1. An application writes the system call number to the `rax` and triggers a software interrupt via the instruction `int x80` afterward.
2. Our new IDT entry ensures that `dispatch_syscall()` is called directly after the interrupt occurs (the CPU automatically switches to ring 0).
This function must first store all registers (except `rax`) on the stack and then uses the value in `rax` as index into the system call table.
Afterward, the corresponding function is called.
3. After the system call function has returned, all registers (except `rax`) must be restored from the stack.
The return value remains in `rax` and can be processed by the application. Now, the syscall dispatcher returns into user mode using the instruction `iretq`.

The general user mode API for all system calls is located in [kernel/src/syscall/user_api.rs](https://github.com/hhu-bsinfo/HeineOS/blob/lesson-9/kernel/src/syscall/user_api.rs).
The given code already contains the example system call `usr_hello_world()`, which logs a message to the serial port.

The kernel space counterpart to `usr_hello_world()` is called `sys_hello_world()` and located in [kernel/src/syscall/functions/](https://github.com/hhu-bsinfo/HeineOS/blob/lesson-9/kernel/src/syscall/functions/).
All further system calls should also be implemented in this folder. Thematically related system calls can be grouped together in the same file.

To test system calls, you should use the code given in [kernel/src/demo/lesson9.rs](https://github.com/hhu-bsinfo/HeineOS/blob/lesson-9/kernel/src/demo/lesson9.rs),
which creates a new user mode thread executing the function `syscall_test_thread()`. For now, this only calls `usr_hello_world()` and enters an infinite loop.
If all goes well, you should see the message printed by `sys_hello_world()` in the log output on the serial port.
*Important: Writing to the serial port uses I/O-port accesses, which are not allowed in user mode.
However, the CPU switches to ring 0 because of the system call, allowing us to execute the corresponding code.*

## Assignment 9.3: Further System Calls
After the first system call is working and has been tested successfully, you should implement the following additional system calls:
 * `usr_thread_yield()` -> `sys_thread_yield()`: Yield the CPU and switch to the next thread
 * `usr_thread_exit()` -> `sys_thread_exit()`: Terminate the current thread.
Once this system call works, the function `kickoff_user_thread()` in `thread.rs` should be modified to call
`usr_thread_exit()` at the end of a user mode thread, instead of spinning in an infinite loop.
 * `usr_thread_get_id() -> usize` -> `sys_thread_get_id() -> u64`: Return the ID of the current thread.
 * `usr_get_system_time() -> usize` -> `sys_get_system_time() -> u64`: Return the current system time in milliseconds.
 * `usr_print(msg: &str)` -> `sys_print(buffer: *const u8, len: usize)`: Print a string to the terminal.
The `&str` parameter has to be deconstructed into a raw pointer and a length, which can be passed to the kernel space function in registers.
 * `usr_get_key_event() -> Option<KeyEvent>` -> `sys_get_key_event(event: *mut KeyEvent) -> u64`: Get the next key event from the keyboard queue.
This system call is more complext than the others, because a `KeyEvent` cannot be returned in a register (it is too large).
Because of this, `usr_get_key_event()` must create an empty `KeyEvent` struct and pass a pointer to it to `sys_get_key_event()`.
The kernel function must then check if there is a key event available, and if so, write it to the given pointer.
The `u64` return value indicates whether a key event was available.

In the given code, the function `user_api::syscall0(syscall: SyscallFunction)` is implemented to perform a system call without any parameters.
To support system calls with parameters, you should implement further functions like `syscall1(syscall: SyscallFunction, arg1: u64)`
and `syscall2(syscall: SyscallFunction, arg1: u64, arg2: u64)`. The parameters should be passed according to the `System V AMD64 ABI`.
Up to five parameters can be supported this way.

*Note: Detailed information for passing parameters in registers is provided in the [System V AMD 64 Architecture Processor Supplement](https://cs61.seas.harvard.edu/site/pdf/x86-64-abi-20210928.pdf) in chapter 3.2.3: Parameter Passing*.

Test your system call in `lesson9::syscall_test_thread()` like the following:
Start by printing the current thread ID to the terminal. Then, read keyboard input in a loop until the return key is pressed.
Afterward, print the line that has just been entered to the terminal together with the current system time.

*Note: Some of the system calls seem superfluous now, as it is still possible to directly call kernel functions.
However, once we have implemented a proper protection mechanism via paging, user mode threads will not be able anymore to access kernel space memory (and code).*

![System Call Test](https://raw.githubusercontent.com/hhu-bsinfo/HeineOS/refs/heads/main/media/lesson-9/system_calls.png)
