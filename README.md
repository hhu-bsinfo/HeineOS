# Lesson 3: Interrupts

## Learning Goals

1. Understand the functionality of the Interrupt Descriptor Table (IDT)
2. Understand the functionality of the Programmable Interrupt Controller (PIC)
3. Implement interrupt dispatching using the keyboard as the first interrupt-based device

## Assignment 3.1: Interrupt Descriptor Table (IDT)
In this assignment you will learn how to load the IDT and test it using manual interrupts.

Most of the required code is already implemented in [kernel/interrupt/idt.rs](https://github.com/hhu-bsinfo/HeineOS/blob/lesson-3/kernel/src/interrupt/idt.rs).
Our IDT has 256 entries, with each entry pointing to a function that should be called when the corresponding interrupt occurs.
In HeineOS, all entries point to the same function `int_disp()` in [kernel/interrupt/dispatcher.rs](https://github.com/hhu-bsinfo/HeineOS/blob/lesson-3/kernel/src/interrupt/dispatcher.rs), which handles dispatching interrupts to their appropriate handlers (e.g., device drivers or exception handlers).
Additionally, each entry has some flags that must be set correctly (`IdtEntry::options`).

Your task is to implement the `IdtEntry::new()` function, which creates a new IDT entry.
The parameter `offset` represents the address of the function to be called and must be split into three parts (`IdtEntry::offset_low`, `IdtEntry::offset_mid`, `IdtEntry::offset_high`).
Furthermore, each entry must always have the options `Present`, `DPL = 0` and `64-Bit Interrupt Gate` set.
For more information about the IDT entry structure, see the [OSDev Wiki](https://wiki.osdev.org/Interrupt_Descriptor_Table#Structure_on_x86-64).

Now load your IDT in `startup.rs` by calling `idt().load()`. Afterward, `int_disp()` should be called whenever an interrupt occurs.
To test this, insert code to output a log message with the triggered interrupt number via the serial port in `int_disp()`.
To manually trigger an interrupt, we can use the x86 instruction `int` in `startup.rs`:

```rust
unsafe {
asm!("int 100");
}
```

This code should result in `int_disp()` being called with the parameter `vector = 100` and you should see your log message.

**Notes:**
- *The IDT requires handler functions to be marked as `extern x86-interrupt`.
  This tells the compiler that these are not normal functions and the machine code to be generated needs to be slightly different (e.g., using `iret` instead of `ret` to return from the function).*

## Assignment 3.2: Programmable Interrupt Controller (PIC)
Now that the basic interrupt handling is implemented, we can move on to activating hardware interrupts and test them via the keyboard.

Start by implementing the empty functions in [kernel/device/pic.rs](https://github.com/hhu-bsinfo/HeineOS/blob/lesson-3/kernel/src/device/pic.rs) (`allow()`, `forbid()` and `status()`).
Afterward, complement your existing keyboard driver with the additional code given in [keyboard.rs](https://github.com/hhu-bsinfo/HeineOS/blob/lesson-3/kernel/src/device/keyboard.rs) and [key.rs](https://github.com/hhu-bsinfo/HeineOS/blob/lesson-3/kernel/src/device/key.rs).
You should now implement the `plugin()` function in `keyboard.rs` to enable the keyboard IRQ on the PIC.
The interrupt service routine (ISR) of the keyboard can be left empty for now, and its registration with the interrupt dispatcher will also be done later.

Information on programming the PIC is available in the [OSDev Wiki](https://wiki.osdev.org/8259_PIC) and a detailed description of the chip is given in [8259A.pdf](https://github.com/hhu-bsinfo/HeineOS/blob/main/slides/8259A.pdf).

Now call the PIC's `init()` function in `startup.rs` and allow the keyboard interrupt with `keyboard::plugin()`.
Finally, call `cpu::enable_int()` to enable hardware interrupts.

When you now boot your operating system, you should see a log message from `int_disp()` whenever you press or release a key on the keyboard.
However, this only works a few times (or even only a single time) before the keyboard buffer is filled up.
Once the buffer is full, the keyboard controller will not send any more interrupts.
This will be fixed in the next assignment by reading scancodes from the keyboard in its interrupt handler.

An example logging output of HeineOS after this assignment could look like this:

```
[0.000][INF][boot.rs@085] Welcome to HeineOS!
[0.000][DBG][boot.rs@161] EFI image is located at: 0x1f016498
[0.000][DBG][boot.rs@177] EFI system table is located at: 0x1f5ec018
[0.000][INF][boot.rs@187] Exiting UEFI boot services...
[0.000][DBG][impl_.rs@352] Boot services are exited. Memory map won't be freed using the UEFI boot services allocator.
[0.000][INF][boot.rs@096] Initializing heap allocator
[0.000][INF][list.rs@066] List allocator initialized: 0x1b14000 - 0x2b14000 (Size: 16777216 bytes)
[0.000][INF][boot.rs@103] Initializing IDT
[0.000][INF][boot.rs@106] Initializing PIC
[0.000][INF][boot.rs@109] Initializing keyboard
[0.000][INF][boot.rs@109] Enabling interrupts
[0.000][INF][boot.rs@112] Boot sequence finished
[0.000][DBG][dispatcher.rs@085] Handling interrupt vector 33
```

**Notes:**
- *During the handling of an interrupt, you do not need to worry about unwanted other interrupts.
  The processor will automatically disable hardware interrupts when it starts handling the interrupt, and will only enable them again when the interrupt handler routine returns.
  Furthermore, we only use one processor core.*
- *Be aware that interrupt handling can only work correctly while HeineOS is still running. You should never return from the `main()` function. An operating system does not just end like a normal program does :-)*  
  ![One does not simply return from main() in OS development](https://i.imgflip.com/an114v.jpg)

## Assignment 3.3: Forwarding interrupts to device drivers
In this assignment, we will create an infrastructure to forward interrupts from `int_disp()` to previouisly registered interrupt service routines (ISRs) from device drivers.

To achieve this, a driver must implement the [ISR](https://github.com/hhu-bsinfo/HeineOS/blob/lesson-3/kernel/src/interrupt/isr.rs) trait and register it with the interrupt dispatcher.
The `ISR` trait consists of only a single function named `trigger()`, which should be called by `int_disp()` when the appropriate interrupt occurs.
Keep in mind that the interrupt dispatcher works with *vector numbers* instead of *IRQ numbers* like the PIC.
The first 32 interrupt vectors (0–31) are reserved for CPU exceptions. `Pic::init()` maps hardware interrupts to the vector numbers 32–47.
For example, the keyboard uses IRQ 1, which corresponds to interrupt vector 33.

The `interrupt::dispatcher` module stores the ISRs in a `Vec`, which is initialized with 256 `Option` instances of the value `None`.
This allows us to register the ISR of a driver at a given index using the `IntVectors::register()` function.

The function `IntVectors::report()` should be called from `int_disp()` to call the `trigger()` function of a previously registered ISR (if existing).
If no ISR is registered for the given interrupt vector, an error message should be printed and the system should be stopped (i.e., `panic!()`).
Remove the manual test interrupt in `startup.rs` as it would otherwise cause this error.

To call `IntVectors::report()` safely, the `Spinlock` wrapping the `INT_VECTORS` global variable must be acquired.
Usually, it is a bad idea to acquire a lock while an interrupt is being handled, as it may cause a deadlock if the lock is already acquired.
The only other point in our kernel, where `INT_VECTORS` is being locked, is the `IntVectors::register()` function.
Make sure to disable interrupts in `IntVectors::register()` when registering the ISR and enable them again before returning.
This can be achieved using `cpu::without_interrupts()`. Now, we can be sure that the lock is not held when calling `IntVectors::report()` from `int_disp()`.

The `keyboard::plugin()` function should now be extended to register an instance of `KeyboardISR` with the interrupt dispatcher.
The corresponding vector number is defined in the `InterruptVector` enum in `dispatcher.rs`.
Furthermore, the keyboard interrupt handler should now log a message via the serial port whenever it is triggered.

Finally, initialize `INT_VECTORS` in `startup.rs` by calling `init_interrupt_dispatcher()`.

An example logging output of HeineOS after this assignment could look like this:

```
[0.000][INF][boot.rs@085] Welcome to HeineOS!
[0.000][DBG][boot.rs@161] EFI image is located at: 0x1f016498
[0.000][DBG][boot.rs@177] EFI system table is located at: 0x1f5ec018
[0.000][INF][boot.rs@187] Exiting UEFI boot services...
[0.000][DBG][impl_.rs@352] Boot services are exited. Memory map won't be freed using the UEFI boot services allocator.
[0.000][INF][boot.rs@096] Initializing heap allocator
[0.000][INF][list.rs@066] List allocator initialized: 0x1b12000 - 0x2b12000 (Size: 16777216 bytes)
[0.000][INF][boot.rs@100] Initializing interrupt dispatcher
[0.000][INF][boot.rs@103] Initializing IDT
[0.000][INF][boot.rs@106] Initializing PIC
[0.000][INF][boot.rs@109] Initializing keyboard
[0.000][INF][boot.rs@112] Enabling interrupts
[0.000][INF][boot.rs@112] Boot sequence finished
[0.000][DBG][dispatcher.rs@085] Handling interrupt vector 33
[0.000][DBG][keyboard.rs@152] Keyboard interrupt handler triggered
```

## Assignment 3.4: Query key events using interrupts
As the final step in this lesson, the keyboards interrupt handler should now read all pending scancodes from the keyboard by calling `Keyboard::try_read_next_byte()`.
Furthermore, the read bytes should be decoded using `Keyboard::decode_byte()` and if a key event has been successfully decoded, it should be stored inside the global key event queue `KEYBOARD_BUFFER`.
This queue stores all key events decoded during interrupt handling for later use. It can be accessed by demos using the `keyboard::keyboard_buffer()` function.
The queue implementation is provided by the crate [nolock](https://lib.rs/crates/nolock), which contains data structures that are thread-safe without using locks.
Thus, they are perfect for use in interrupt handlers, where locks are not allowed. A dependency to this crate is added in the [Cargo.toml](https://github.com/hhu-bsinfo/HeineOS/blob/lesson-3/kernel/Cargo.toml) file.

However, we still need to lock the global `KEYBOARD` instance in `KeyboardISR::trigger()` to be able to call `Keyboard::try_read_next_byte()`.
This is fine, as the whole key handling process is now fully handled by the interrupt handler. No other code in your operating system should access the `KEYBOARD` instance anymore (you can even make it private by removing the `pub` keyword).

Replace any calls in your demo code to `Keyboard::poll_key_event()` and `Keyboard::poll_key_press()` by corresponding function calls from `KeyEventQueue`.
You can even delete the two functions from `keyboard.rs` if you want.

Furthermore, the new file [library/input.rs](https://github.com/hhu-bsinfo/HeineOS/blob/lesson-3/kernel/src/library/input.rs) provides convenience functions for reading ASCII characters from the keyboard buffer.
As your last task in this assignment, implement the function `input::read_char()` to wait for a key press that produces a printable ASCII character and return it.

**Notes:**
- *Usually, at least one byte should be readable from the keyboard buffer during interrupt handling.
  However, sometimes an IRQ1 is triggered directly after booting in QEMU, even without any key press.
  In this case, the `KeyboardStatus::OUTPUT_BUFFER_FULL` bit is not set in the status register.
  Our `try_read_next_byte()` should already handle this case correctly.*
- *The PS/2 mouse is also handled by the keyboard controller but uses IRQ12.
  Since we do not have a handler for IRQ12, there may be pending data from the mouse when handling IRQ1.
  This can be detected by checking the `KeyboardStatus::AUXILIARY_DEVICE` bit in the status register. Such data should be discarded.*

## Optional Assignment: Kernel Options
Similar to program arguments for normal applications, the bootloader can pass a string with user-defined options to our kernel.
This is useful for configuring the operating system.

### Setting the Log Level

In this assignment, we want to allow the user to set the kernel's log level via the bootloader's command line.
To enable this, open [loader/towboot.toml](https://github.com/hhu-bsinfo/HeineOS/blob/lesson-1/loader/towboot.toml) and add new line containing `argv = "log_level=DBG" under `[entries.heineos]`.
For example, your towboot configuration file should now look like this:

```
[entries]
  [entries.heineos]
    name = "HeineOS"
    image = "kernel.elf"
    modules = [ { image = "initrd.tar", argv = "initrd.tar" } ]
    argv = "log_level=DBG"
```

Now, in your `main()` function, you can use the following code to check for the presence of command line options:

```rust
if let Some(cmdline) = multiboot.find_tag::<multiboot::CommandLineTag>(multiboot::TagType::CommandLine) {
let command_line = cmdline.as_str();
info!("Command line: '{}'", command_line);
}
```

This should print the command line string to the serial port. Next, you can parse the command line string to extract the given log level.
The function `LevelFilter::from_str()` can be used to convert a string (e.g., `info` or `debug`) to the corresponding `LevelFilter` enum value.

Finally, call `log::set_max_level()` with the extracted value to set the maximum log level for the kernel.
If you now boot your operating system with the command line option `log_level=info`, you should see only log messages with level `info` or higher.
All `debug!()` and `trace!()` messages will be ignored. You can even turn off logging completely with `log_level=off`.

### Logging to the terminal

While outputting log messages to the serial port is useful when working with QEMU, it can be cumbersome when working with real hardware.
Oftentimes, a modern computer does not have a serial port exposed, and even if it does, you still need another computer with a serial port to connect it to and view the output.
For this scenario, it would be much more convenient to output log messages to the terminal instead, so they can be viewed directly on the screen.

A first attempt at this is straightforward: Edit the `logger::log()` function in [kernel/src/logger.rs](https://github.com/hhu-bsinfo/HeineOS/blob/lesson-1/kernel/src/logger.rs) to not only write the given message to the serial port, but also to the terminal using `print!()` or `println!()`.
However, as your operating system will get more complex over time, the number of log messages during boot may become quite large, and printing them to the terminal might slow down the boot process.
It would be nice if we could enable or disable this feature via a command line option.

To achieve this, we first need to implement functionality to turn terminal logging on or off.
This can be done by adding a new variable to the `Logger` struct [kernel/src/logger.rs](https://github.com/hhu-bsinfo/HeineOS/blob/lesson-1/kernel/src/logger.rs) with the type `AtomicBool`.
This variable will be used to toggle terminal logging on and off. Furthermore, implement a new function `Logger::enable_terminal_logging(&self, enabled: bool)` to set the value of this variable.
*Notice that we do not need a mutable reference to `self` here, as we use an atomic variable, which is inherently thread-safe and can be set by using a const reference.*

Finally, you can introduce a new command line option to enable or disable terminal logging (e.g., `log_to_terminal=true`).
If the option is present, parse it as you did for the log level and call `Logger::enable_terminal_logging()` with the corresponding value.

![Terminal logging](https://raw.githubusercontent.com/hhu-bsinfo/HeineOS/refs/heads/main/media/lesson-3/logging.png)