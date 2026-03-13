# Lesson 1: Input/Output

## Learning Goals
- Get acquainted with the environment and tools
- Familiarize with the rust programming language
- Low-level programming: Implement code to output text via the serial port and to the screen, and to read input from the keyboard

## Assignment 1.1: Set up the environment

### Prerequisites

For building HeineOS, a *rust nightly* toolchain is required. To install rust, use [rustup](https://rustup.rs/).
The toolchain `nightly-2026-03-01` is confirmed to work with HeineOS.
We also need `cargo-make` for Makefile-like build scripts.

```bash
rustup toolchain install nightly-2026-04-01
cargo install --no-default-features cargo-make
```

Furthermore, we need to install the *build-essential* tools, as well as the *Netwide Assembler* (nasm) for building HeineOS.
For debugging purposes, *gdb* should also be installed. Last but not least, QEMU is required to run HeineOS in a virtual machine.

On Ubuntu 24.04 you can install all the above with a single apt command:

```bash
sudo apt install build-essential nasm gdb qemu-system-x86
```

On macOS you can use [Homebrew](https://brew.sh/) to install the required tools:

```bash
brew install x86_64-elf-binutils nasm x86_64-elf-gdb qemu
```

### Building and running HeineOS

You should now be able to build and run HeineOS. Clone the repository and run the following commands:

```bash
git clone git@github.com:hhu-bsinfo/HeineOS.git
cd HeineOS
git checkout lesson-1
cargo make --no-workspace qemu
```

QEMU should start and boot HeineOS, which will do nothing but show a black screen.

### Debugging with IDEs

We recommend using either VSCode or RustRover for development, as we provide debugging configurations for both IDEs.

#### RustRover

To debug with RustRover place a breakpoint anywhere in the code (use a line in `main()` for example) and start the *debug* configuration in the upper right corner.
This will build HeineOS and start QEMU which waits for a debugger to attach.

![Start the debug configuration in RustRover](https://raw.githubusercontent.com/hhu-bsinfo/HeineOS/refs/heads/main/media/lesson-1/rustrover1.png)

Now launch the *Start Debugger* configuration, which will start gdb and attach it to the running QEMU instance.

![Launch the Start Debugger configuration in RustRover](https://raw.githubusercontent.com/hhu-bsinfo/HeineOS/refs/heads/main/media/lesson-1/rustrover2.png)

QEMU should now continue and stop at the breakpoint you set in the first step, allowing you to inspect variables and step through the code.

![Debugging HeineOS in RustRover](https://raw.githubusercontent.com/hhu-bsinfo/HeineOS/refs/heads/main/media/lesson-1/rustrover3.png)

#### VSCode

To debug with VSCode first install the *C/C++ Debug (gdb)* extension from the VSCode marketplace.
It is also recommended to install the *rust-analyzer* extension to get rust language support in VSCode.

Now open the *Run and Debug* tab on the left side.

![Open the Run and Debug tab in VSCode](https://raw.githubusercontent.com/hhu-bsinfo/HeineOS/refs/heads/main/media/lesson-1/vscode1.png)

Then start the *debug* configuration in the upper left corner.

![Start the debug configuration in VSCode](https://raw.githubusercontent.com/hhu-bsinfo/HeineOS/refs/heads/main/media/lesson-1/vscode2.png)

This will build HeineOS and launch QEMU and gdb and attach gdb to the running QEMU instance.
If the build process takes too long, VSCode might receive a timeout. In this case, click on "Debug Anyway", or just try again.
The build process should be faster on the second try.

![Debugging HeineOS in VSCode](https://raw.githubusercontent.com/hhu-bsinfo/HeineOS/refs/heads/main/media/lesson-1/vscode3.png)

## Assignment 1.2: Hello, World!

*This assignment is very guided, to provide an easy introduction. The next assignments will leave you with more freedom to implement things by yourself.*

Our operating system just boots to a black screen and is not able to do anything afterward.
The first thing it needs to learn is communicating with the outside world. The easiest way to do that, is using the *serial port*, also known as *COM* (short for *communication port*).
This is an interface, used in computers for decades. Even today many servers still have one for debugging purposes, and consumer grade mainboards often still provide headers, that can be extended to a serial port using a simple adapter.

[<img src=https://upload.wikimedia.org/wikipedia/commons/thumb/e/ea/Serial_port.jpg/960px-Serial_port.jpg width=320px>](https://commons.wikimedia.org/wiki/File:Serial_port.jpg)

We have configured QEMU to emulate a serial port and redirect all its output to standard out (see [Makefile.toml](https://github.com/hhu-bsinfo/HeineOS/blob/lesson-1/Makefile.toml#L35)).
This way, we can see everything that our operating system writes to the serial port in the terminal where QEMU was started.

An incomplete driver for the serial port is provided in [kernel/src/device/serial.rs](https://github.com/hhu-bsinfo/HeineOS/blob/lesson-1/kernel/src/device/serial.rs).
Implement the `ComPort::write_byte()` function (replace the `todo!()` call) to write the given byte to the serial controller's data port.
To test your implementation, insert the following code at the end of your `main()` function (before the endless loop):

```rust
COM1.lock().write_byte('H' as u8);
```

You also need to import the static variable `COM1` from the `serial` module with:

```rust
use crate::device::serial::COM1;
```

*Note: You are expected to figure out imports by yourself in future assignments. We recommend using an IDE, that imports the necessary modules automatically.*

When you now run `cargo make --no-workspace qemu`, HeineOS should still boot to a black screen, but you should see the letter `H` in the terminal where you started QEMU.

Next, implement the `write_str()` function in `serial.rs` to output an entire string via the serial port using `ComPort::write_byte()`.
Test your implementation by writing *"Hello, World!"* via the serial port in `main()`.

![Hello World from HeineOS](https://raw.githubusercontent.com/hhu-bsinfo/HeineOS/refs/heads/main/media/lesson-1/serial.png)

As you may have noticed, `write_str()` is part of the `fmt::Writer` trait, which allows formatted output to any struct, that implements `write_str()`.
This works by using the `write_fmt()` function in conjunction with the `format_args!()` macro, which is part of the rust `core` library:

```rust
COM1.lock()
.write_fmt(format_args!("The screen resolution is {}x{}!", framebuffer_info.width as usize, framebuffer_info.height as usize))
.unwrap();
```

Notice the `unwrap()` call: `write_fmt()` and `write_str()` return a `Result`, which indicates whether writing the provided string was successful.
The rust compiler will output a warning, if a `Result` is not handled properly. The `unwrap()` function simply checks if the `Result` is `Ok` and panics otherwise.

A panic is an unrecoverable error, which causes the global *panic handler* to be called (see [boot.rs](https://github.com/hhu-bsinfo/HeineOS/blob/lesson-1/kernel/src/boot.rs#L141)).
Our panic handler just outputs an error message via the `error!()` macro and runs an endless loop, effectively halting the system.

The `error!()` macro is part of the [log](https://lib.rs/crates/log) crate, which provides a simple logging interface via the macros `trace!()`, `debug!()`, `info!()`, `warn!()` and `error!()`.
Our goal is to be able to use these logging macros with the serial port to provide a robust logging mechanism for our operating system.

The logger is already initialized at the very beginning of `main()`, but we still need to provide an actual implementation of the `log::Log` trait.
An incomplete implementation is provided in [kernel/src/logger.rs](https://github.com/hhu-bsinfo/HeineOS/blob/lesson-1/kernel/src/logger.rs).
You still need to implement the `Logger::log()` function to actually output the log message via the serial port.  
The function takes a `Record` as argument, which contains the log level in its `metadata` field, the actual log message in its `args` field (usable with the `format_args!()` macro) and the source code location in its `file` and `line` fields.

You are free to format the log message however you like, but a sensible implementation could look like this:

![Logging output from HeineOS](https://raw.githubusercontent.com/hhu-bsinfo/HeineOS/refs/heads/main/media/lesson-1/logger.png)

The time field at the beginning of each log message is just a placeholder for now but can be easily implemented later, once we have implemented a driver for the hardware timer.
The first four log messages in this example are from `boot::exit_uefi_boot_services()`, which takes over complete control of the system from the UEFI BIOS (you don't need to understand this function right now).

We now have a robust logging mechanism, which can be used to debug our operating system.

## Assignment 1.3: Spinlock

The rust compiler guarantees that each variable can only be accessed by an arbitrary number of readers or exactly one writer.
This prevents race conditions at compile time, but it also makes it hard to have global variables.
For example, the naive way to create a global instance of the serial port could look like this:

```rust
pub static COM1: ComPort = ComPort::new(ComBaseAddress::Com1);
```

If we were to write to `COM1` in `main()` using `COM1.write_byte('H' as u8)` similar to the previous assignment, we would get a compiler error, because `COM1` is not mutable.
However, `write_byte()` requires a mutable reference to `self` (which is the `ComPort` struct).

```
error[E0596]: cannot borrow immutable static item `COM1` as mutable
  --> kernel/src/boot.rs:85:5
   |
85 |     COM1.write_byte('H' as u8);
   |     ^^^^ cannot borrow as mutable
   |
  ::: kernel/src/device/serial.rs:13:1
   |
13 | pub static COM1: ComPort = ComPort::new(ComBaseAddress::Com1);
   | ------------------------ this `static` cannot be borrowed as mutable

For more information about this error, try `rustc --explain E0596`.

```

We could declare `COM1` as mutable using the `mut` keyword. The declaration would look like this:

```rust
pub static mut COM1: ComPort = ComPort::new(ComBaseAddress::Com1);
```

However, the code would still not compile because static mutable variables are not allowed in rust.

```
error[E0133]: use of mutable static is unsafe and requires unsafe block
  --> kernel/src/boot.rs:85:5
   |
85 |     COM1.write_byte('H' as u8);
   |     ^^^^ use of mutable static
   |
   = note: mutable statics can be mutated by multiple threads: aliasing violations or data races will cause undefined behavior
```

Static mutable variables are not allowed in rust because they are not thread-safe.
The compiler cannot guarantee that `COM1` is only accessed by one thread at a time.

The solution is to use a locking mechanism, which allows us to have a global variable that is not declared as mutable but still allows mutable access to the variable it protects.
One such mechanism is the *spinlock*, which is implemented in [kernel/src/library/spinlock.rs](https://github.com/hhu-bsinfo/HeineOS/blob/lesson-1/kernel/src/library/spinlock.rs).
It uses `unsafe` blocks internally to provide mutable access to the variable, but ensures that only one reference to the variable is allowed at a time.  
This is done using an `AtomicBool`, which is a boolean variable, that can be atomically set to `true` and `false`.
The provided implementation is not complete and always grants access to the variable without setting or even checking the `AtomicBool`.
This is fine in this early stage of the project, but it breaks the guarantee made by the rust compiler and will lead to problems later on.

To avoid such problems, we complete the implementation of the `Spinlock` struct right now.  
Your task is to implement the `try_lock()`, `lock()`, `unlock()` and `is_locked()` functions.

If the lock is acquired successfully, it returns a `SpinlockGuard` struct, which wraps the variable and provides transparent access to it, via the `Deref` and `DerefMut` traits.
Furthermore, it implements the `Drop` trait to automatically unlock the spinlock when it goes out of scope. See the following example:

```rust
fn test() {
    let mut com1 = COM1.lock();
    com1.write_str("Hello, World!\n").unwrap();
}
```

Notice that the `lock()` function does not return a `SerialPort` reference, but a `SpinlockGuard<SerialPort>` struct, which wraps the `SerialPort` instance.
However, due to the `Deref` and `DerefMut` traits, we can treat the `SpinlockGuard` struct as a `SerialPort` instance and directly call `write_str()` on it.  
In this example `COM1` is locked right at the beginning of the function. The returned guard (stored in `com1`) can then be used as many times as we want to access the `SerialPort` instance.
We do not need to unlock the spinlock manually, because the guard will automatically unlock the spinlock when it goes out of scope.
In this case, the compiler will automatically `drop()` on the guard at the end of the function, which unlocks the spinlock.

*Warning:* You should not use code like this in the `main()` function, as does never return and thus never unlocks the spinlock.
If you want to acquire a lock in the `main()` function, you have three options:

1. If you only need to access the variable once, you can chain the function calls. The compiler will automatically drop the guard after `write_str()` returns:
```rust
COM1.lock().write_str("Hello, World!");
```
2. Open a new scope and store the guard in a variable:
```rust
{
let mut com1 = COM1.lock();
com1.write_str("Hello, World!").unwrap();
}
```
3. Manually call `drop()` on the guard:
```rust
let mut com1 = COM1.lock();
com1.write_str("Hello, World!").unwrap();
drop(com1);
```

The first option is very convenient if you only need to access the variable once.
However, if you need to access the variable multiple times, it is better to use the second or third option, since you lock and unlock the spinlock multiple times otherwise.

As a simple test for your implementation, you can try to lock and unlock `COM1` multiple times in a row and see if the code runs without getting stuck.
Furthermore, you should test provoking a deadlock by trying to lock `COM1` two times in the same scope. The operating system should get stuck in the second `lock()` call, spinning forever.

## Assignment 1.4: Framebuffer and Terminal

The framebuffer is a memory region that allows us to draw pixels on the screen. It actually is part of the video memory on the graphics card that is mapped into the physical address space of the CPU.
Put simply, it is an array of 32-bit values, where each value corresponds to one pixel on the screen. The 32-bit values are split up into eight bits for each color channel (red, blue, green and alpha).
For example, setting the first address in the framebuffer to `0xffffffff` would set the first pixel on the screen to white.

The framebuffer is already set up for us by the bootloader, using a UEFI BIOS service. The bootloader provides us with the framebuffer's base address and size via the [Multiboot2](https://www.gnu.org/software/grub/manual/multiboot2/multiboot.html) protocol.
All multiboot-related code is already implemented in [kernel/src/multiboot.rs](https://github.com/hhu-bsinfo/HeineOS/blob/lesson-1/kernel/src/multiboot.rs).

Code for drawing pixels to the framebuffer is already implemented in [kernel/src/device/framebuffer.rs](https://github.com/hhu-bsinfo/HeineOS/blob/lesson-1/kernel/src/device/framebuffer.rs).
The `Framebuffer` struct is initialized in [boot.rs](https://github.com/hhu-bsinfo/HeineOS/blob/lesson-1/kernel/src/boot.rs#L61) and the framebuffer instance can be accessed via the static function `terminal::framebuffer()`.

To get acquainted with drawing via the framebuffer, try to draw some pixels to the screen using the `Framebuffer::draw_pixel()` function.
Functions for drawing text to the screen are also provided with `Framebuffer::draw_char()` and `Framebuffer::draw_str()`.
These functions use an 8x8 pixel bitmap font, defined in [kernel/src/device/font_8x8.rs](https://github.com/hhu-bsinfo/HeineOS/blob/lesson-1/kernel/src/device/font_8x8.rs).

![Framebuffer drawing example in HeineOS](https://raw.githubusercontent.com/hhu-bsinfo/HeineOS/refs/heads/main/media/lesson-1/framebuffer.png)

Your main task in this assignment is to complete the terminal implementation. The terminal is a simple text-based user interface that outputs text to the screen.
An incomplete implementation is provided in [kernel/src/device/terminal.rs](https://github.com/hhu-bsinfo/HeineOS/blob/lesson-1/kernel/src/device/terminal.rs).
The terminal stores a current position in the framebuffer, where the next character should be drawn. Once it reaches the end of the line, it automatically wraps to the next line.
Furthermore, if the chracter is a newline character (`\n`), it also automatically moves the cursor to the beginning of the next line. The current position is made visible by drawing a *cursor* character.

Implement the empty functions in `terminal.rs` to make the terminal work. Convenience functions for drawing an erasing the cursor at a given position are already provided (`draw_cursor()` and `clear_cursor()`).  
As with the serial port, we get formatted output via the `fmt::Write` trait. Implement the `write_str()` function to output a string to the terminal.

Once you have finished your implementation, you can test it by using the `print!()` and `println!()` macros.
For example, the following code should output five rows of *"Hello, World!"* to the screen:

```rust
for _ in 0..5 {
println!("Hello, World!");
}
```

![Terminal output in HeineOS](https://raw.githubusercontent.com/hhu-bsinfo/HeineOS/refs/heads/main/media/lesson-1/terminal.png)

Now we need to handle the case that the terminal position reaches the last line of the framebuffer.
Naturally, we cannot just continue drawing to the next line, as it would be outside the framebuffer bounds.
Instead, we want to scroll the screen upwards by the number of pixels that a character occupies (in this case eight pixels).
For that, you need to implement the `Framebuffer::scroll_up()` function, which scrolls the entire framebuffer upwards by the given number of pixels.
The lines of pixels at the top of the framebuffer are moved "out of sight" and are not drawn anymore.

The terminal should now call `Framebuffer::scroll_up()` whenever the cursor would go beyond the last line and set the cursor position to the beginning of the last line.

![Terminal scrolling in HeineOS](https://raw.githubusercontent.com/hhu-bsinfo/HeineOS/refs/heads/main/media/lesson-1/terminal.gif)

Once your terminal implementation is finished, implement the `text_demo()` function in [kernel/src/demo/lesson1.rs](https://github.com/hhu-bsinfo/HeineOS/blob/lesson-1/kernel/src/demo/lesson1.rs) to print formatted text to the screen, like this:

![Terminal demo](https://raw.githubusercontent.com/hhu-bsinfo/HeineOS/refs/heads/main/media/lesson-1/text_demo.png)

## Assignment 1.5: Keyboard input

We now have a solid logging mechanism and a working terminal for communicating with the user.
However, we still need to be able to receive input from the user. For that, we need to implement a driver for the keyboard.  
For decades, the *PS/2* controller was the standard for handling keyboard and mouse input. It is still supported by some systems today and can be emulated by QEMU.
It is much simpler than communication with a USB keyboard, which is why we use the *PS/2* controller for HeineOS.

The code decoding keyboard scancodes into key events with ascii characters is already implemented in [kernel/src/device/keyboard.rs](https://github.com/hhu-bsinfo/HeineOS/blob/lesson-1/kernel/src/device/keyboard.rs).
You only need to implement the following functions:
* `Keyboard::try_read_next_byte()`: Check if a new byte from the keyboard is available. If so, pass it to `Keyboard::decode_byte()` and return the decoded key event (or `None` if no key event could be decoded).  
  Checking whether a byte is available works by reading the keyboard's control port and testing the `KeyboardStatus::OUTPUT_BUFFER_FULLL` bit.
  The byte can then be read from the keyboard's data port.
* `Keyboard::poll_key_event()`: Call `Keyboard::try_read_next_byte()` in a loop until a key event is detected and return it.
* `Keyboard::poll_key_press()`: Poll key events from the keyboard until a key press event is detected and return it. Other key events are discarded.

Further information on how the keyboard works is provided by [keyboard.pdf](https://github.com/hhu-bsinfo/HeineOS/blob/main/slides/keyboard.pdf)

Implement the `keyboard_demo()` function in [kernel/src/demo/lesson1.rs](https://github.com/hhu-bsinfo/HeineOS/blob/lesson-1/kernel/src/demo/lesson1.rs) to test your keyboard implementation.
The function should print the key events that are detected to the terminal. An example output may look like this:

![Keyboard demo in HeineOS](https://raw.githubusercontent.com/hhu-bsinfo/HeineOS/refs/heads/main/media/lesson-1/keyboard.png)

## Optional assignment: Keyboard LEDs and repeat rate

Implement the functions `Keyboard::set_led()` and `Keyboard::set_repeat_rate()` to control the keyboard's LEDs and typematic repeat rate.
See [keyboard.pdf](https://github.com/hhu-bsinfo/HeineOS/blob/main/slides/keyboard.pdf) for more information.  
These features should be tested on real hardware, as QEMU does not support setting the keyboard LEDs.