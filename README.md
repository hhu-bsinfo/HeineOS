# Lesson 12: Processes

## Learning Goals
1. Compile applications separately from the kernel
2. Protect the kernel against user space accesses
3. Implement a process management structure

## Assignment 12.1: Compiling Applications
The given code adds two new folders to your project's root folder.
This separates the operating system into three parts:

- `kernel`: Contains the whole operating system code
- `apps`: Contains all user space applications
- `usrlib`: Contains library functions that can be used by user space applications (and also the kernel)

Integrate the new structure into your project.
Note, that the file `kernel/src/device/key.rs` has been split into two parts.
One that remains inside the kernel and contains the keyboard buffer and one that only contains the `KeyEvent` struct in [usrlib/src/key.rs](https://github.com/hhu-bsinfo/HeineOS/blob/lesson-12/usrlib/src/key.rs).
This is necessary because we want to pass key events from the kernel to the user space via system calls.
However, the keyboard buffer should still be managed by the kernel and not be directly accessible from user space.

Now, get familiar with the example application in [apps/hello](https://github.com/hhu-bsinfo/HeineOS/blob/lesson-12/apps/hello).
Our operating system should load this application into its own address space and execute it at runtime.
This poses multiple questions:

1. *How can HeineOS access the application's code?*  
   Application binaries are stored inside the initial ramdisk (TAR archive) and can thus be loaded via the filesystem.
2. *What kind of file format do applications use and how can we find the executable code inside the file?*  
   We initially link applications as ELF files. However, the ELF format is quite complex and would need a parser to extract the executable code.
   Instead, we use the program `objcopy` to extract the code from the ELF file and store it inside a so-called *flat binary*, containing only the executable code.
   We just need to make sure that an application's `main()` function is always stored right at the beginning of a flat binary.
   To enforce this, we add the attribute `#[unsafe(link_section = ".main")]` to `main()`, creating an own linker section for the `main()` function.
   Inside the linker script ([apps/link.ld](https://github.com/hhu-bsinfo/HeineOS/blob/lesson-12/apps/link.ld)), we ensure that the `main` section is always linked to the beginning of the code.
3. *How can applications access library functions without duplicating code*
   All library functions that are used by applications should be placed in the `usrlib` folder.
   Each application has a dependency to this folder in its own `Makefile.toml`.
   This way, the `usrlib` code compiled and linked into each application (and the kernel).

After you have integrated the new structure, copy `once.rs`, `spinlock.rs` and `user_api.rs` from your kernel to `usrlib`.
This will require you to update multiple `use` statements in the kernel code.
Afterward, everything should compile and work as before.

## Assignment 12.2: Mapping the Application Image
Application binaries are included in the TAR archive (and thus in our filesystem) in the `apps` folder.
New user threads should now always execute an application instead of a function.
For that, physical memory must be allocated and the application code copied into it.
Afterward, this physical memory must be mapped into the thread's address space.
We always map code at address `0x100_0000_0000` (1 TiB) (see [consts.rs](https://github.com/hhu-bsinfo/HeineOS/blob/lesson-12/kernel/src/consts.rs)).
The `entry` function of a user thread should now always point to this fixed virtual address.
You can use `core::mem::transmute()` to cast this constant value to a `fn()` type variable (*CAUTION: This is unsafe and should only be used in exceptional cases*).

Because applications are now no part of the kernel anymore, they cannot be debugged directly.
The debugger cannot find the appropriate symbols in the kernel file.
To solve this, we can execute the GDB command `add-symbol-file target/heineos_app/debug/hello.elf` in the debug console.
*CAUTION: You should only load one application at a time, as the virtual addresses of different applications overlap, causing confusion in the debugger.*

*Note: It is not possible to return from an application's `main()` function. All applications must terminate themselvses via the system call `usr_thread_exit()`*

## Assignment 12.3: Protecting Kernel Space
Until now, every application has read/write access to the kernel (addresses smaller than 1 TiB).
We now want to protect the kernel against user space accesses, by deleting the `USER_ACCESSIBLE` flag in the corresponding page table entries.
If you have used the functions `kernel_flags()` and `user_flags()` correctly in the last lesson, you only need to edit one line in `user_flags()`.

Afterward, we need to modify the start procedure of a user mode thread.
We cannot call `kickoff_user_thread()` anymore from `thread_user_start()`, as this function resides in kernel memory and is not accessible from user space.
Instead, we can directly place the `entry` address of a user thread onto the prepared stack in `switch_to_user_mode()`.

## Assignment 12.4: Process Management
We now want to implement a process management structure and modify the thread creation code to start a new process with each user thread.

All running processes should be stored in key-value-tree (`BTreeMap`) in [kernel/src/process/process.rs](https://github.com/hhu-bsinfo/HeineOS/blob/lesson-12/kernel/src/process/process.rs).
The key of each entry is the process ID (PID), and the value is the process (struct `Process`) itself.
This way, process information can be retrieved quickly via the PID.
The `BTreeMap` is provided by `alloc` crate (part of the rust standard library).

The kernel also has its own process that can be accessed via `process::kernel_process_id()`.
Since the process also stores a reference to the PML4, the static variable `KERNEL_PAGE_TABLES` in `pages.rs` is no longer needed and should be deleted, along with the function `pages::kernel_page_tables()`.

Next, some modification in `thread.rs` are necessary.
The `Thread` struct should now also store the process ID of the process it belongs to and provide a function to retrieve it.

In `scheduler.rs` a new function `Scheduler::spawn_process(app_path: &str)` should be implemented, that loads and runs an application from the given path.
It should create a new process for the application, create a mapping for the application image and framebuffer and create a new user thread that runs the application.
When an application terminates (i.e., the scheduler cleans up the corresponding thread), the process should be removed from the `BTreeMap`.

Finally, a new system call `usr_process_get_id()` should be introduced that returns the PID of the currently running process.
Test your operating system by calling different system calls from a user space application.

## Optional Assignment: Isolated File Handles
In its current state, the filesystem uses a global map to store open file handles.
This is bad because one process can access file handles of another process by simply trying out different values as file handles.

To solve this problem, the filesystem should use one map per process.
This way, each process can only access its own file handles.

Replace your `TarFs` struct with the following code:
```rust
pub struct TarFs {
    /// The tar archive reference used to read files from.
    archive: TarArchiveRef<'static>,
    /// Maps of open file handles for each process.
    open_handles: Spinlock<BTreeMap<usize, BTreeMap<FileHandle, OpenFile>>>,
    /// The map of next available file handle IDs for each process.
    next_handles: Spinlock<BTreeMap<usize, usize>>
}
```
The `open_handles` map now takes a process ID as the key and has a map of file handles for each process as value.
Furthermore, each process now has its own ID counter to determine the next available file handle ID in `next_handle`.

Replace `Tarfs::next_handle_id()` with the following code:
```rust
/// Generate the next unique file handle ID for the current process.
/// This function is called internally when a new file handle is created.
fn next_handle_id(&self) -> usize {
let pid = scheduler().get_active_pid();
   let mut handles = self.next_handles.lock();
   
   match handles.get(&pid) {
      Some(handle) => *handle,
      None => {
         handles.insert(pid, 1);
         0
      }
   }
}
```
The function now first retrieves the current process ID via `scheduler::get_active_pid()` and then uses the `next_handles` to determine the next available file handle ID for the current process.
If no entry for the current process exists, a new entry is created with the value `1`.

Additionally, we need a function to remove processes from the `open_handles` and `next_handles` map:
```rust
/// Remove the process from the map of open handles and next available handle IDs.
/// This should be called when the process is terminated to free up resources.
pub fn remove_process(&self, pid: usize) {
   self.open_handles.lock().remove(&pid);
   self.next_handles.lock().remove(&pid);
}
```
Call this function whenever a process terminates.

Modify the remaining filesystem functions on your own, so that they work with the new `TarFs` structure.
When a process has no entry in `open_handles` yet, create a new empty map for it.
