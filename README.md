# Lesson 6: Filesystem & Porting a Game Boy Emulator

## Learning Goals
1. Implement a basic read-only filesystem
2. Learn how to parse and display bitmap files
3. Port a Game Boy emulator (Peanut-GB) to HeineOS that can load a ROM image from the filesystem


## Assignment 6.1: Filesystem
In this assignment, you will implement a basic read-only filesystem.
To get files into the system, we use the bootloader's ability to load *modules* into memory besides the kernel.
A module can be any arbitrary file and must be bundled inside the OS image.
The bootloader allocates memory for the file and loads it there. The allocated address is passed to the kernel via the [Multiboot2](https://www.gnu.org/software/grub/manual/multiboot2/multiboot.html) protocol.

We use a TAR archive as a module. This way, we can easily bundle multiple files into a single module.
Other operating systems use similar techniques, bundling multiple files required for booting into a single archive.
This archive is oftentimes called an *initrd* (initial ram disk).
Our initrd is built automatically during the build process from the contents of the `initrd` directory.
The final TAR archive is stored in the `loader` directory as `initrd.tar` and included in the OS image `HeineOS.img` together with the kernel.

We use the crate [tar-no-std](https://lib.rs/crates/tar-no-std) to parse the TAR archive and access the files it contains.
The filesystem code is located in the [filesystem/tarfs.rs](https://github.com/hhu-bsinfo/HeineOS/blob/lesson-6/kernel/src/filesystem/tarfs.rs).

As a first step, you should find the initial ramdisk in memory and initialize the filesystem with it.
Start by searching for the initrd in `boot.rs` using `multiboot.find_tag::<multiboot::ModuleTag>(multiboot::TagType::Module)`.
If a module is found, you can call `as_slice()` to get its contents as a slice of bytes.
This can then be used to create a `TarArchiveRef` object, which is need to initialize the filesystem with `tarfs::init_filesystem()`.

Now, implement the empty functions in `tarfs.rs` to make the filesystem work.
Start with `TarFs::open()`, which should search the tar archive for a file at the given path.
The `TarArchiveRef` struct provides an iterator over all files in the archive via its `entries()` function.
The name of each file is already its full path, so you should easily be able to find the file you are looking for.
If the file is found, a `FileHandle`, which is just a unique identifier for accessing the file, should be created and returned.
Furthermore, an `OpenFile` instance must be created, referencing the entry in the tar archive and the current read position (initially 0).
The caller can use the returned handle to access the file using `TarFs::read()`, `TarFs::seek()` and `TarFs::size()`. Calling `TarFs::close()` should invalidate the handle (i.e., remove it from the internal map of open files).

Next, implement the remaining filesystem functions.
Reading a file should copy its contents into the provided buffer, respecting the read position and also updating the read position depending on the number of bytes read.
Seeking a file updates the read position of the appropriate `OpenFile` instance.

Finally, test your implementation by reading a simple text file from the filesystem and printing its contents to the screen.

![Filesystem overview](https://raw.githubusercontent.com/hhu-bsinfo/HeineOS/refs/heads/main/media/lesson-6/tarfs.png)

## Assignment 6.2: Bitmap Images
Now that we have a working filesystem, we have a way to get useful files into the system.
In this assignment, you will implement a basic bitmap image loader, which enables you to display a bitmap image on the screen.
This is, for example, useful for game development, as bitmap images are often used to represent sprites.

The given code in [library/bitmap.rs](https://github.com/hhu-bsinfo/HeineOS/blob/lesson-6/kernel/src/filesystem/tarfs.rs) already contains all necessary structs.
You only need to implement the function `Bitmap::from_bytes()` to parse a bitmap file from a slice of bytes.
A detailed description of the BMP file format can be found on [Wikipedia](https://en.wikipedia.org/wiki/BMP_file_format).
It is enough to support simple Windows BMP files with 24-bit color depth and no compression.
You can use the provided `color()` function to convert the extracted 8-bit color values to a 32-bit color value, compatible with the framebuffer.
Be aware that each row of pixel data is padded to a 4-byte boundary.

To be able to display the bitmap image, implement the function `Framebuffer::draw_bitmap()`.
As the color data is already in the correct format, you can copy the pixel data to the framebuffer, row by row.

The initrd already contains a bitmap image file called [heine.bmp](https://github.com/hhu-bsinfo/HeineOS/blob/lesson-6/initrd/heine.bmp), that you can use to test your implementation.

![HeineOS showing a bitmap image](https://raw.githubusercontent.com/hhu-bsinfo/HeineOS/refs/heads/main/media/lesson-6/bitmap.png)

## Assignment 6.3: Porting a Game Boy Emulator
[Peanut-GB](https://github.com/deltabeard/Peanut-GB) is a Game Boy emulator written in pure C.
The whole code is contained in a single header file, making it easy to port to different platforms.
In this assignment, you will port Peanut-GB to HeineOS, using the filesystem to load a ROM image and the framebuffer to display the game screen.

For testing purposes, the ROM file `2048.gb` is already included in the initrd in the `roms` directory.
This is an open source reimplementation of the popular game 2048. The source is available on [GitHub](https://github.com/Sanqui/2048-gb).
We will not provide any proprietary ROM files, as they are copyrighted by their creators.
However, the emulator is able to play any ROM file that is compatible with the original Game Boy.

Start by looking at the code in [demo/lesson6/peanut-gb.rs](https://github.com/hhu-bsinfo/HeineOS/blob/lesson-6/kernel/src/demo/lesson6/peanut_gb.rs).
It begins with declarations for functions that are implemented in the C code. By declaring them as `extern "C"` here, we can call them from Rust.
Notice how parameters and return values have special types, corresponding to the C types (e.g., `c_int`, `c_long`, `c_void`).

Further down in the file, you will find more functions marked as `extern "C"`, containing *TODO* comments.
These are functions that are called from the C code but implemented in Rust.

While it is easy to call C functions from Rust, and vice versa, the case looks different for structs and enums.
If C code defines a struct that we need to use in Rust, we have no choice but to also declare it in Rust (the same applies to enums).
Rust provides the `#[repr(C)]` attribute to tell the compiler that a struct or enum should look exactly as if it was defined in C, when compiled.
The provided code already defines the enums `GbError` and `GbInitError`, corresponding the error code enums returned by the C functions.  
However, the most important struct used by the C code is the `gb_s` struct, which contains the entire emulator state.
Redeclaring it in Rust would be a lot of work. Fortunately, we can work around this, as the C functions only want a pointer to the struct as a parameter.
We only need to allocate memory for it and pass a pointer to it to the C code. Notice, how the function declaratations in Rust use the `c_void` type for the struct pointer.
The size of the struct is provided by the C function `gb_size()` in [demo/lesson6/peanut-gb.c](https://github.com/hhu-bsinfo/HeineOS/blob/lesson-6/kernel/src/demo/lesson6/peanut_gb.c).  
The only time when we need to access a field of the struct is when passing joypad button states to the C code.
For this purpose, the function `gb_get_joypad_ptr()` in `peanut_gb.c` returns a pointer to the joypad state field of the passed struct.
This is an 8-bit wide field, where each bit corresponds to a button of the Game Boy's joypad.
Setting a bit to 1 means that the corresponding button is not pressed, while setting a bit to 0 means that the button is pressed.

Before starting implementing the emulator, make sure to replace your old `kernel/Makefile.toml`, with the one provided in this assignment.
The new file contains additional build tasks for compiling the C code.

Your main task is to implement the `play()` function.
Start by loading the ROM file's content from the given path into the static variable `ROM`.
Next, allocate a memory for the `gb_s` struct. You can use `Vec::<u8>::with_capacity()` to allocate a buffer of the correct size.
A pointer to the `Vec`'s data can be acquired by calling `Vec::as_mut_ptr()`.
Now, call the C function `gb_init()` to initialize the emulator, passing the pointer to the struct, as well as the corresponding function pointers.

As a first test of your implementation, you can now add a log message to `gb_rom_read()`, logging the read address and call `gb_run_frame()` in an infinite loop after the emulator has been initialized.
If you run the emulator, you should see your log message appear a few times in the terminal, before the emulator panics because of a checksum error:

```text
[1.259][DBG][peanut_gb.rs@168] ROM read at 0x0134
[1.259][DBG][peanut_gb.rs@168] ROM read at 0x0135
[1.259][DBG][peanut_gb.rs@168] ROM read at 0x0136
[1.259][DBG][peanut_gb.rs@168] ROM read at 0x0137
[1.259][DBG][peanut_gb.rs@168] ROM read at 0x0138
[1.260][DBG][peanut_gb.rs@168] ROM read at 0x0139
[1.260][DBG][peanut_gb.rs@168] ROM read at 0x013a
[1.260][DBG][peanut_gb.rs@168] ROM read at 0x013b
[1.260][DBG][peanut_gb.rs@168] ROM read at 0x013c
[1.260][DBG][peanut_gb.rs@168] ROM read at 0x013d
[1.260][DBG][peanut_gb.rs@168] ROM read at 0x013e
[1.260][DBG][peanut_gb.rs@168] ROM read at 0x013f
[1.261][DBG][peanut_gb.rs@168] ROM read at 0x0140
[1.261][DBG][peanut_gb.rs@168] ROM read at 0x0141
[1.261][DBG][peanut_gb.rs@168] ROM read at 0x0142
[1.261][DBG][peanut_gb.rs@168] ROM read at 0x0143
[1.261][DBG][peanut_gb.rs@168] ROM read at 0x0144
[1.261][DBG][peanut_gb.rs@168] ROM read at 0x0145
[1.261][DBG][peanut_gb.rs@168] ROM read at 0x0146
[1.262][DBG][peanut_gb.rs@168] ROM read at 0x0147
[1.262][DBG][peanut_gb.rs@168] ROM read at 0x0148
[1.262][DBG][peanut_gb.rs@168] ROM read at 0x0149
[1.262][DBG][peanut_gb.rs@168] ROM read at 0x014a
[1.262][DBG][peanut_gb.rs@168] ROM read at 0x014b
[1.262][DBG][peanut_gb.rs@168] ROM read at 0x014c
[1.262][DBG][peanut_gb.rs@168] ROM read at 0x014d
[1.263][ERR][boot.rs@258] Kernel panic: panicked at kernel/src/demo/lesson6/peanut_gb.rs:257:13:
Failed to initialize PeanutGB (Error: InvalidChecksum)

```

Implement `gb_rom_read()` to return the correct byte of the ROM file and run the emulator again.
The emulator should now run without errors, and you should see an infinite stream of log messages in your terminal.
The ROM is now actually played by the emulator, we just do not see anything on the screen yet.

To display the game screen, implement the function `lcd_draw_line()`.
This function is called by the emulator's C code for each line of the screen.
The emulator provides a pointer to the line's data, which is an array of 160 bytes.
We only need to look at the first two bits of each byte, as the original Game Boy can only display four colors (or better, four shades of gray).
The provided code already includes a color palette with the `PALETTE` constant. You can use the 2-bit color values as indices into the palette.
After implementing this function, make sure to call `gb_init_lcd()` before entering the infinite loop in `play()`.
Furthermore, remove the log message from `gb_rom_read()`, as it significantly slows down the emulator.
You should now see the game's title screen.

![Peanut-GB Screenshot](https://raw.githubusercontent.com/hhu-bsinfo/HeineOS/refs/heads/main/media/lesson-6/gameboy1.png)

Next, we should make sure that the emulator runs at the correct speed.
We want it to run at 60 Hz, so we need to call `gb_run_frame()` 60 times per second.
Use `pit::system_time()` to measure the time taken by the last call to `gb_run_frame()` and calculate the time to wait until the next frame should be drawn.
Use `pit::wait()` to wait for the calculated time. The desired frame time is already provided in the `MS_PER_FRAME` constant.

Finally, only user input is missing. Check if the user has pressed a key using `KeyEventQueue::pop_key_event()` in every iteration of the loop.
If a key has been pressed or released, and it corresponds to a button of the Game Boy's joypad, set/unset the corresponding bit in the emulator's joypad state.

You should now be able to run the emulator and play the game! However, when running a debug build of HeineOS, performance is probably very slow.
Try running a release build instead by using `cargo make --no-workspace --profile production qemu`, as explained in the optional assignment of [Lesson 5](https://github.com/hhu-bsinfo/HeineOS/tree/lesson-5).
This way, full emulation speed should be achievable (depending on your hardware).

If you like, you can scale the game screen by 2x or even 4x, so it is not displayed so tiny anymore.
To do this, you only need to adapt the `lcd_draw_line()` function.
Here is a screenshot with 2x scaling and the Game Boy screen centered in the framebuffer:

![Peanut-GB Screenshot](https://raw.githubusercontent.com/hhu-bsinfo/HeineOS/refs/heads/main/media/lesson-6/gameboy2.png)

## Optional Assignment: Save Games
Many Game Boy Games include a small amount of battery-backed RAM, which is used to store game progress.
A few games (e.g., Pokémon) also use this RAM for some of their sprites, so these only work correctly if the RAM is also emulated.

Emulating the RAM is pretty easy, as you just need to implement the functions `gb_cart_ram_read()` and `gb_cart_ram_write()`.
Start by adding a new static variable, holding a `Vec` protected by a `Spinlock`. This `Vec` will contain the contents of the RAM.
In your `play()` function, call `gb_get_save_size_s()` after calling `gb_init()` to get the size of the RAM.
Notice that `gb_get_save_size_s()` does not return the size directly, but wants a pointer to a `c_size_t` variable.
It writes the size into this variable and returns an error code (0 on success).
Initialize the RAM `Vec` with the correct number of zeros.

The `c_size_t` type is still experimental in Rust and must be enabled manually.
Add the following line to the top of your `boot.rs` file:

```rust
#![feature(c_size_t)]
```

That's it, in theory. The RAM is now emulated, but it is not very useful yet.
We need a way to save the contents of the RAM to a file that can be loaded when starting the game again.
HeineOS does not support writing files, but we can make use of the serial port to export the contents of the RAM to a file on the host computer.

First, add two new arguments to QEMU in your `Makefile.toml`:

```text
"-serial", "none",
"-serial", "file:gameboy.sav",
```

You should now pass three `-serial` arguments to QEMU in total, each of which causes QEMU to emulate an additional serial port.
Our operating system uses the first serial port for logging.
The UEFI BIOS uses the second serial port, also for logging purposes.
The third serial port is not used at all until now, so we can safely use it for saving the contents of the RAM.
As you can see in the arguments above, QEMU will write all data written to the third serial port to a file called `gameboy.sav`.

Next, add a new global variable to `device/serial.rs`, used to access the third serial port:

```rust
pub static COM3: Spinlock<ComPort> = Spinlock::new(ComPort::new(ComBaseAddress::Com3));
```

Now, we need a way to exit the emulation loop so that we can write the contents of the RAM to the file after the user exits the game.
Check if the user has pressed the `Escape` key and break out of the loop if this is the case.
At the end of the `play()` function, iterate over all bytes in the RAM and write them to the serial port one by one.

You should now see the file `gameboy.sav` on your host computer.
Check its size to make sure that the save data has actually been written to it.
Copy the file to `initrd/roms/`. In your `play()` function, you can now load the save file in addition to the ROM file.
Copy the contents of the save file to your RAM `Vec` before entering the emulation loop.
To make sure, that the initrd is definitely rebuilt with the new file, you can delete `loader/initrd.img` and `HeineOS.img` before restarting.
The game should now see the contents of the save file in its RAM, allowing the user to save and load game progress (if supported by the game).