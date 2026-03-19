# Lesson 4: Cooperative Multitasking

## Learning Goals
1. Refresh your assembly knowledge
2. Understand the process of switching between coroutines
3. Learn the difference between coroutines and threads
4. Understand how a scheduler works

*It is recommended to read the [assembler crash course](https://github.com/hhu-bsinfo/HeineOS/blob/main/slides/asm.pdf) first.*

## Assignment 4.1: Coroutines
In this assignment, you will implement **coroutines** using Rust and assembly language. We use coroutines as a preliminary step towards multithreading.

Start by looking at the new file [coroutine/coroutine.rs](https://github.com/hhu-bsinfo/HeineOS/blob/lesson-4/kernel/src/coroutine/coroutine.rs).
Your first task is to implement the functions `coroutine_start()` und `coroutine_switch()`. Since these are `naked`, they may only contain assembly code.
It is not possible to access Rust variables by their name from within assembly code. All parameters must be read from the corresponding CPU registers.
All assembly instructions must be entered as strings, separated by commas, inside the `naked_asm!()` macro.

The state of a coroutine must be saved on the stack. This includes all CPU registers. A pointer to the last stack entry should be stored in the `Coroutine::stack_ptr` field.

Afterward, implement the remaining empty methods in `coroutine.rs`. Coroutines are chained together using the `next` field in the `struct Coroutine`.

Test your coroutines by implementing the test functions in the file [demo/lesson4.rs](https://github.com/hhu-bsinfo/HeineOS/blob/lesson-4/kernel/src/demo/lesson4.rs).
Your test should create three coroutines that are chained together. Each of them should increment its own counter variable and print it at a fixed position on the screen.
A coroutine should switch to the next one after each iteration. Because they are chained together, forming a cycle, the coroutines switch in a round-robin fashion, and it looks like the counters are incremented in parallel.
To set the cursor position, you need to lock the terminal instance temporarily. You should use the macro `print_terminal!()` to print the counter using the locked terminal reference.
Make sure to unlock the terminal instance before switching to the next coroutine. Otherwise, the next coroutine will get stuck at acquiring the terminal lock, resulting in a deadlock.

The demo should look like this (the braced numbers show the coroutine IDs):

![Coroutine Demo](https://raw.githubusercontent.com/hhu-bsinfo/HeineOS/refs/heads/main/media/lesson-4/coroutines.png)

Further information on the coroutine implementation can be found in the [coroutine slides](https://github.com/hhu-bsinfo/HeineOS/blob/main/slides/coroutine.pdf),

## Assignment 4.2: Queue
Before we can implement a scheduler for threads, we need to implement a queue.
We use a linked list for this, which always removes the first element from the beginning of the list and always appends at the end.
When using this for a scheduler, the next thread to be executed is always the one that is at the head of the list, and the thread that was just executed is always appended to the end.

Implementing a linked list in Rust is challenging, which is why you only need to implement the `remove()` function.

The queue implementation is given in the file [library/queue.rs](https://github.com/hhu-bsinfo/HeineOS/blob/lesson-4/kernel/src/library/queue.rs).

## Assignment 4.3: From Coroutines to Threads
Look at the given code in [thread/thread.rs](https://github.com/hhu-bsinfo/HeineOS/blob/lesson-4/kernel/src/thread/thread.rs).
It is very similar to the coroutine implementation, and you can copy over most of your code from assignment 4.1.
You only need to adapt the function names. Notice that the `next` field is missing in the `Thread` struct, since we will manage the threads in a separate queue instead of directly linking them together.

Implement all empty functions in `thread.rs` using your coroutine code. *You cannot test this right now, since we haven't implemented a scheduler yet.*

## Assignment 4.4: Scheduler
In this assigment, you will implement a basic scheduler for threads. All threads are managed in a *ready Queue* (see assignment 4.2) and are switched in a round-robin fashion.
This is still a cooperative multitasking scheduler, so the threads need to manually yield the CPU to other threads by calling `Scheduler::yield_cpu()`.
The scheduler will not support priorities or other advanced features. The current thread is always stored in `SchedulerState::active_thread`, while all wating threads are stored in `SchedulerState::ready_queue`.
The given code also includes an implementation for an [idle thread](https://github.com/hhu-bsinfo/HeineOS/blob/lesson-4/kernel/src/thread/idle_thread.rs), which should always be registered with the scheduler, to ensure that at least one thread is always running.

Notice how all methods of the scheduler are called with a const `&self` reference, although they alter the state of the scheduler (e.g., enqueue and dequeue threads from the ready queue).
This is realized by wrapping the scheduler's variables `ready_queue` and `active_thread` in a separate struct called `SchedulerState` and protecting this with a `Spinlock`.
This way, a const reference is enough to access and modify the scheduler's state without breaking the Rust compiler's contract, as the spinlock allows only one thread to access the scheduler's state at a time.

This causes a problem with the `yield_cpu()` and `exit()` methods: Usually, the spinlock is released automatically when a function returns.
However, in these two functions, we switch to another thread, meaning that we do not return from the function directly and the scheduler state remains locked.
Any further call to one of the scheduler's methods would result in a deadlock. To prevent this, the assembly code in `thread_start()` and `thread_switch()` must be modified to unlock the spinlock directly after setting the `rsp` register, by calling `unlock_scheduler()`.

Implement the empty functions in [thread/scheduler.rs](https://github.com/hhu-bsinfo/HeineOS/blob/lesson-4/kernel/src/thread/scheduler.rs).
When a thread switches via `yield_cpu()`, the currently active thread should be enqueued at the end of the ready queue and the new active thread should be stored in `SchedulerState::active_thread`.
*Notice that you cannot access the formerly active thread anymore after enqueuing it in the ready queue. Because of this, you should store a pointer to this thread in a local variable, as you need to pass it to `thread_switch()`.*

Test your scheduler implementation by only readying the idle thread before moving on the next assignment.
Implement a basic text output in the idle thread to see if it works.

## Assignment 4.5 Multithreading Demo
Start by creating a very basic thread that only prints a message and terminates itself. Afterward, only the idle thread should be running.

As a more advanced test, implement the counter demo from assignment 4.1 using threads instead of coroutines.
Extend the demo, by letting one of the counter threads kill the ones by calling `Scheduler::kill()` after a certain amount of increments.
The last remaining thread should terminate itself by calling `Scheduler::exit()` when it reaches a certain counter value.
Afterward, only the idle thread should remain running.

*Caution: Calling `Scheduler::kill()` is generally not recommended, since it can lead to deadlocks, if the killed thread is currently holding a lock.*

![Thread Demo](https://raw.githubusercontent.com/hhu-bsinfo/HeineOS/refs/heads/main/media/lesson-4/threads.png)

## Optional Assignment: A nicer Font
The 8x8 font included in HeineOS does the job, but it is rather small and only contains basic ASCII characters.
In this assignment, you may switch to a nicer 8x16 font supporting Unicode characters. The font is provided by the [unifont](https://lib.rs/crates/unifont) crate.

Start by adding a dependency to the `unifont` crate in `kernel/Cargo.toml` under the `[dependencies]` section:
```toml
unifont = "1.1.0"
```

Next, you may want to specify the font's size as constant in `device/framebuffer.rs`:
```rust
pub const CHAR_WIDTH: usize = 8;
pub const CHAR_HEIGHT: usize = 16;
```

Now, replace the `draw_char()` implementation with a new one, which uses the `unifont` crate to draw the character.
The crate provides the function `unifont::get_glyph(c: char)` to get the glyph for a given character.
The glyph is a struct containing the actual bitmap data. Use a loop in conjunction with the `Glyph::get_pixel()` function to iterate over the pixels and draw them to the framebuffer.

Notice that glyphs can either `Halfwidth` or `Fullwidth`, depending on the character.
A `Fullwidth` character is 16 pixels wide instead of 8. As we do no support fonts with different widths, we can simply ignore `Fullwidth` glyphs.
These are mainly used for emojis and other special characters. Implementing a `draw_string()` function that supports `Fullwidth` would not be difficult.
However, supporting these glyphs in the terminal is not a trivial task, which is why we support only the normal `Halfwidth` glyphs in this assignment.
However, you are of course free to experiment with `Fullwidth` glyphs as much as you like.

As a last step, all usages of `font8x8::CHAR_WIDTH` and `font8x8::CHAR_HEIGHT` in your operating system must be replaced with `framebuffer::CHAR_WIDTH` and `framebuffer::CHAR_HEIGHT`.

![Unifont Demo](https://raw.githubusercontent.com/hhu-bsinfo/HeineOS/refs/heads/main/media/lesson-4/unifont.png)

### Supporting multiple fonts
It is possible to support both the old 8x8 font and the new font and select the one to use at compile time.
This can be done by introducing a *feature* flag in `Cargo.toml` and using conditional compilation in `framebuffer.rs`.

Start by modifying `Cargo.toml` to include a new feature flag and making the `unifont` dependency optional:
```toml
unifont = { version = "1.1.0", optional = true }

[features]
unifont = ["dep:unifont"]
```

Now, the `unifont` crate will only be compiled if the `unifont` feature flag is enabled.

In `framebuffer.rs`, we need to check if the `unifont` feature is enabled and include different code depending on the result.
This can be achieved by using the `cfg` attribute and needs to be done in three places:

1. Include the `font8x8` module only if the `unifont` feature is not enabled:
```rust
#[cfg(not(feature = "unifont"))]
use crate::device::font_8x8;
```

2. Define the `CHAR_WIDTH` and `CHAR_HEIGHT` constants depending on the feature flag:
```rust
#[cfg(feature = "unifont")]
pub const CHAR_WIDTH: usize = 8;
#[cfg(not(feature = "unifont"))]
pub const CHAR_WIDTH: usize = font_8x8::CHAR_WIDTH;

#[cfg(feature = "unifont")]
pub const CHAR_HEIGHT: usize = 16;
#[cfg(not(feature = "unifont"))]
pub const CHAR_HEIGHT: usize = font_8x8::CHAR_HEIGHT;
```

3. Define two variants of the `draw_char()` function, depending on the feature flag:
```rust
#[cfg(feature = "unifont")]
/// Draw a single character at the specified (x, y) coordinates with the given foreground and background colors.
/// If the character does not fit fully within the framebuffer, it is not drawn.
/// This implementation uses the font provided by `unifont` crate.
pub fn draw_char(&mut self, c: char, x: usize, y: usize, fg_color: u32, bg_color: u32) {
    ...
}

#[cfg(not(feature = "unifont"))]
/// Get the pixel data for a character from the `font_8x8` font data.
fn get_char_pixels(c: char) -> &'static [u8] {
    ...
}

#[cfg(not(feature = "unifont"))]
/// Draw a single character at the specified (x, y) coordinates with the given foreground and background colors.
/// If the character does not fit fully within the framebuffer, it is not drawn.
/// This implementation uses the font provided by the `font_8x8` module.
pub fn draw_char(&mut self, c: char, x: usize, y: usize, fg_color: u32, bg_color: u32) {
    ...
}
```

To enable the feature flag during compilation, you need to edit the arguments for the `compile` task in `kernel/Makefile.toml`.
Simply add `"--features", "unifont"`, to the list of arguments and perform a clean build.
To switch back to the old font, remove the `--features` argument and perform a clean build.