# Lesson 13: Managing virtual address spaces

## Learning Goals
1. Abstract virtual address spaces through *Virtual Memory Areas* (VMAs).
2. Create heaps for user space applications.
3. User stacks are not fully allocated from the start but should grow on demand.
4. Clean up process resources when an application exits.
5. Convert Peanut-GB to a user space application.

## Assignment 13.1: Virtual Memory Areas (VMAs)
In this assignment, virtual address spaces will be managed through VMAs (similar to Linux).
A VMA is a region of memory in a virtual address space with a start and end address (both page aligned) and a type (Code, Stack, Heap, Framebuffer).
Each VMA belongs to exactly one process.

Paging is still used to realize separate address spaces and protect the kernel memory.

All VMAs of a process should be stored inside a `Vec` in the `Process` struct.
Implement the new function `process::add_vma(process_id: usize, vma: VMA)` to add a VMA to a process.
This function should check whether the given VMA overlaps with an existing one in the process and return an error if it does.
Otherwise, it should add the VMA to the process's VMA list.

Each process (except the kernel process) should have VMAs for the framebuffer, user code and user stack after it is fully initialized and the application thread is created.

For debugging purposes, a new system call `usr_process_dump_vmas()` should be implemented to log all VMAs of the current process to the serial port.

Further information on VMAs in Linux is provided here: [https://manybutfinite.com/post/how-the-kernel-manages-your-memory/](https://manybutfinite.com/post/how-the-kernel-manages-your-memory/).

## Assignment 13.2: A heap for processes
In this assignment, each process should get its own heap. The heap should be managed as a VMA in the process's virtual address space.
We utilize our existing kernel heap allocator and now use it for bothe the kernel and user space applications.

Start by moving the folder `kernel/allocator` to `usrlib/`.
Apply the necessary changes to your kernel code so that it compiles and runs as before (mostly import statements need to be updated).

The memory for the user space heap should be mapped by a new system call `usr_process_map_heap(heap_start_address: u64, heap_size: usize)`.
This requires implementing the new function `pages::map_user_heap(pml4_table: &mut PageTable, user_heap_start: u64, user_heap_size: usize)` in `pages.rs`.
The page frames for the heap should be allocated via the physical page frame allocator.

Test the user space heap by calling `usr_map_heap()` in an application and allocating some memory (e.g., by creating a `Box` or `Vec`).

## Assignment 13.3: Dynamically growing stacks
Until now, user stacks were allocated with a fixed size.
This should now change so that the stack can grow on demand.
This requires no changes in user mode, but the kernel code needs to be modified.

Modify the function `pages::map_user_stack()` in a way that only the highest page of the stack is allocated and mapped (the VMA should still cover the whole stack size).
If the stack grows beyond a size of 4 KiB, a page fault should be triggered.
Test this behavior by creating a user space application that allocates a lot of stack memory.
A recursive function with a lot of parameters is a good choice for this.
An example application for this test could be a recursive fibonacci function with unused parameters.
Instead of parameters, you could create a large slice in each recursive call.

Afterward, extend your page fault handler so that it checks if the fault address is within the user stack.
If it does, a page frame should be allocated and mapped to the corresponding page.
A good idea would be to implement a helper function `pages::check_and_grow_user_stack()` for this functionality.

Your recursive test application should now work as expected.
It still causes page faults on the stack, but these are now handled properly by allocating and mapping new frames for the stack.

## Assignment 13.4: Clean up process resources
At this point, we have a working thread and process management system.
However, when a process terminates, the physical memory occupied by it is not freed.
Over time, this will cause the operating system to run out of physical memory.

The rust runtime frees (*drops*) the process struct when it is removed from the process `BTreeMap`.
However, since it only contains a reference to the PML4, the page tables are not freed.
In fact, since we manage the page tables completely on our own using unsage code, the rust runtime has no chance to properly free them.
To fix this, we can implement the `Drop` trait for the `Process` struct.
This trait only contains a single function `drop()`, which is called when a corresponding struct is dropped.
In this function, we can free the page tables and the physical memory occupied by the process.

Start by implementing the `Drop` trait for the `Process` struct in `process.rs`.
As a first test, print a log message with the process ID and name to see if the function is called when an application exits.

Now, implement the new function `PageTable::drop_frames(level: usize)` that recursively iterates over all page tables (an iterative solution is also possible, though not recommended).
On the last level, where physical addresses are mapped, the function should free all page frames whose page table entries have the `USER_ACCESSIBLE` flag set.
We explicitly do **not** want to free kernel memory, since it is still required by other processes and, of course, the kernel itself.
Additionally, the page frames occupied by the page tables themselves should also be freed.

Implement the public function `pages::free_page_tables(pml4_table: &mut PageTable)` to call the `drop_frames()` function on the given page table.
All that is left to do is to call this function in the `Drop` implementation of `Process`.

Test your implementation by creating and executing a user space application that terminates after a short time.
Use `PfListAllocator::dump_free_list()` to check if the page frames are freed correctly.

## Assignment 13.5: Porting Peanut-GB to user space
Until now, we have only tested small user space applications.
In this assignment, we will port the Game Boy emulator *Peanut-GB* (see [lesson 6](https://github.com/hhu-bsinfo/HeineOS/tree/lesson-6)) to user space.

For Peanut-GB to work properly, we need access to the framebuffer, which is currently only accessible from the kernel (it is mapped in each process, but kernel privileges are required to access it).
Our solution is that each process allocates its own framebuffer on its own heap.
Of course, pixel data set in this framebuffer is not displayed on the screen, as it lies on the user heap.
To display the framebuffer, we will implement a system call that copies the framebuffer data from the user heap to the real framebuffer.

Start by copying `framebuffer.rs` and `font_8x8.rs` from the kernel to `usrlib/`.
You need to delete the function `Framebuffer::from_multiboot()`, as it relies on the `multiboot` crate, which is not available in user space.
Perform any changes necessary to make your operating system compile and run again.

Now, implement the new system call `usr_get_framebuffer_info()` that returns the framebuffer width, height and pitch.
Next, implement the system call `usr_framebuffer_flush(fb: &mut Framebuffer, x: usize, y: usize, width: usize, height: usize)` that copies part of the given framebuffer to the real framebuffer.
The given parameters `x`, `y`, `width` and `height` specify the area of the framebuffer that should be copied.
Test your implementation in a small user space application, that uses a `Vec` as its framebuffer memory:
```rust
let fb_info = usr_get_framebuffer_info();
let fb_vec = Vec::<u8>::with_capacity(fb_info.pitch() * fb_info.height());
let mut fb = Framebuffer::new(fb_info.width(), fb_info.height(), fb_info.pitch(), fb_vec.as_ptr() as u64);
```
You can now draw to the created framebuffer and call `usr_framebuffer_flush()` to flush it to the screen.  
*Note: `Framebuffer` only stores the address of the framebuffer memory, so you need to make sure, that `Vec` lives as long as the framebuffer.
This is why `Framebuffer::new()` is an *unsafe* function.*

Next, you need to implement system calls for the filesystem.
The required system calls are `usr_file_open()`, `usr_file_close()`, `usr_file_read()`, `usr_file_size()` and `usr_file_seek()`.
The corresponding `sys` functions in the kernel should just call the appropriate filesystem functions.
If a filesystem operation fails, return -1 from the kernel. Otherwise, return the resulting value of the operation.
In your `usr` functions, you can then check whether the return value is positive or negative and return a corresponding `Result`.
Since the `usr` functions pass a `usize` as the file descriptor, we need a way to convert it to a `FileHandle` in the kernel.
The given code extends `TarFs` with new functions `FileHandle::new()` and `FileHandle::as_usize()` for that purpose.  
*Note: Since all function in `TarFs` return a `usize`, you need to cast them to `isize` before returning them, and back to `usize` in user space.*

Next, move your `peanut-gb.rs` file from the kernel to the `peanut_gb` application source folder.
It is a good idea to now comment out any code that calls kernel functions so that the application compiles before you continue (even though it does not do anything meaningful in this state).
You can now start to replace kernel calls with user space functions (i.e., system calls).
Peanut-GB requires access to the filesystem, framebuffer, keyboard and timer.
All of these should now be accessible from user space via system calls.

*Note: If you implemented the optional assignment from lesson 6 (save games via the serial port), you need an additional system call to access the serial port in user space.
Of course, you can also just drop the save game feature and implement the rest of the application without it.*

## Optional Assignment: Can it run Doom?
The video game *Doom* is famous for being able to run on everything:

![Doom running on a digital camera](https://64.media.tumblr.com/1bcc6ae941de44f0490f71ffd955eda4/tumblr_mv9wuyBMlA1smreryo2_1280.jpg)

Of course, our operating system would not be a real operating system when it cannot even run Doom.
Luckily, porting Doom is pretty easy nowadays.
The Doom fork [doomgeneric](https://github.com/ozkl/doomgeneric) has made modification to the original source, making it very portable.
It only needs a C standard library and the port must implement a handful of functions for drawing to the screen, reading key events and measuring time.

![Doom meme](https://images7.memedroid.com/images/UPLOADED883/651d98cf2709d.jpeg)

The hardest thing for us is to provide a C standard library.
To our luck, Doom only depends on a few functions (e.g. for file access), so that we do not need to implement the full library.

Copy the `apps/doom/` directory from the given code into your project.
Additionally, you need the *doomgeneric* source code.
You can either download it from [GitHub](https://github.com/ozkl/doomgeneric) or add the repository as a submodule to your project:
```bash
cd apps/doom/src
git submodule add https://github.com/ozkl/doomgeneric
```
Your directory structure should look like this:

![Doom directory structure](https://raw.githubusercontent.com/hhu-bsinfo/HeineOS/refs/heads/main/media/lesson-13/doom_directory.png)

You should now be able to compile the Doom application.
Starting it will immediately cause a panic, because Doom calls a C standard library function that is not yet implemented.

Before you start implementing the missing functions, you should get a copy of a valid Doom *WAD* file.
These files contain all assets and level data for the game.
The WAD-file of Doom's free shareware version (containing only the first episode) is available at [doomwiki.org](https://doomwiki.org/wiki/DOOM1.WAD).
Copy the file to your `initrd/` folder and name it `doom.wad`.

Furthermore, you need to make sure that your filesystem does not hand out the file handles 0, 1, and 2, as these are reserved for standard out, standard error, and standard in.
It is best to let the file handle id counter in `TarFs` start with the value 3.

All C standard library functions are implemented in [apps/doom/libc](https://github.com/hhu-bsinfo/HeineOS/blob/lesson-13/application/doom/src/libc).
Look at the Rust files: Some functions are already implemented for you, but some contain `todo!()` calls and need to be implemented by you.
Start by implementing `putch()` and `puts()` in `stdio.rs`.
These functions enable Doom to print strings to the terminal.
If you now start Doom again, you should see some messages, before running into the next `todo!()`.
Continue implementing the missing functions one after another.

In [doom.rs](https://github.com/hhu-bsinfo/HeineOS/blob/lesson-13/application/doom/src/doom.rs) you need to implement the functions `DG_Init()`, `DG_DrawFrame()`, `DG_GetKey()`, `DG_SleepMs()` and `DG_GetTicksMs()`.
These are mostly self-explanatory. However, there is one problem with `DG_DrawFrame()`:
Doom sets up its own framebuffer pointed to by `DG_ScreenBuffer`.
In `DG_DrawFrame`, you need to flush that buffer to the screen.
You can, of course, create a `Framebuffer` instance that wraps `DG_ScreenBuffer` and use `usr_framebuffer_flush()`.
However, the resolution that Doom renders at will probably not match the screen resolution (it is smaller).  
The best solution would be to modify your `usr_framebuffer_flush()` system call so that it works when the source and target buffers have different resolutions.  
A much more simple, albeit unflexible, way is to set the render resolution of Doom to match your screen resolution.
This can be done by adding the two comilation parameters `-DDOOMGENERIC_RESX=800` and `-DDOOMGENERIC_RESY=600` to the task `compile-doom` in the application's `Makefile.toml`.
However, if you change the screen resolution in `kernel/boot.asm` you also need to update these parameters.

Once you have implemented all missing functions, your operating system should finally be able to run Doom!
