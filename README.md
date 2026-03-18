# Lesson 2: Memory Management & PC Speaker

## Learning Goals
1. Understand how a heap allocator works and is implemented in Rust
2. Low-level programming: Use the PC speaker and PIT to play sounds

A detailed overview on how to implement a heap allocator in Rust is given by Phil Oppermann in his fantastic [blog article](https://os.phil-opp.com/allocator-designs/).  
General information on memory management can be found in the [slides](https://github.com/hhu-bsinfo/HeineOS/blob/main/slides/memory.pdf).

## Assignment 2.1: Bump Allocator
In this assignment, you will implement a basic *bump allocator*, to understand the integration of a heap allocator into our operating system.  
This allocator only knows the beginning and end of the heap, and stores the current heap position in the variable `next`.
Each allocation simply increases the value of `next` by the size of the allocation, if it fits in the remaining heap space.
*Deallocating memory is not supported.*

![Bump Allocator](https://raw.githubusercontent.com/hhu-bsinfo/HeineOS/refs/heads/main/media/lesson-2/bump.png)

The heap starts right after the kernel image and is 16 MiB in size (see [consts.rs](https://github.com/hhu-bsinfo/HeineOS/blob/lesson-2/kernel/src/consts.rs)).
Integrate the given code into your kernel. You need to declare the `allocator` module in `boot.rs`:

```rust
mod allocator;
```

*Note: As with imports, you are expected to figure out module declarations by yourself in future assignments.*

Furthermore, we need to import the `alloc` module in `boot.rs` to gain access to Rust library structures that rely on a heap allocator (e.g., `Box` or `Vec`):

```rust
extern crate alloc;
```

Make sure to call the `init_allocator()` function in your boot code.
To test your bump allocator, implement a demo function in [demo/lesson2.rs](https://github.com/hhu-bsinfo/HeineOS/blob/lesson-2/kernel/src/demo/lesson2.rs)

The bump allocator is implemented in [allocator/bump.rs](https://github.com/hhu-bsinfo/HeineOS/blob/lesson-2/kernel/src/allocator/bump.rs).
The integration into the Rust runtime is done through the `GloballAlloc` trait.
The heap allocator instance is stored in a static variable in [allocator/global.rs](https://github.com/hhu-bsinfo/HeineOS/blob/lesson-2/kernel/src/allocator/global.rs).
Make sure to call `global::init_allocator()` in your boot code to initialize the heap allocator.

To test your bump allocator, implement a demo function in [demo/lesson2.rs](https://github.com/hhu-bsinfo/HeineOS/blob/lesson-2/kernel/src/demo/lesson2.rs).
Use `Box::new()` to allocate a struct on the heap and `Vec::new()` to allocate a dynamically growing array-like structure on the heap.

Notice that there is no classic `free()` function or `delete` operator in Rust.

*Note: Rust requires pointers to be aligned as specified by the `layout` parameter of the `alloc()` function. Use the given function `align_up()` in [allocator/global.rs](https://github.com/hhu-bsinfo/HeineOS/blob/lesson-2/kernel/src/allocator/global.rs) to align a pointer.*

## Assignment 2.2: List-based Allocator
In this assignment, you will implement an improved allocator based on a linked list, which is able to also deallocate memory.
For this implementation, all free memory blocks are linked together to form a single linked list.

At the start, there is only one large chunk of free memory comprising the entire heap.
During initialization, this large chunk should be inserted into the list, as its first and only element.

![List-Allocator](https://raw.githubusercontent.com/hhu-bsinfo/HeineOS/refs/heads/main/media/lesson-2/list1.png)

The struct `LinkedListAllocator` stores the start and end of the heap (`heap_start` and `heap_end`) as well as a dummy `ListNode` with length 0 (`head`).
The dummy head is only used to store the initial heap position in the list. After initialization, a single `ListNode` is present in the heap, its length equaling the heap size.

### Allocation
When allocating a memory block, the list must be searched for a suitable block.
It is enough to just take the first block that is large enough to hold the requested memory.
If the remaining space is large enough to store metadata for a list node, it should be cut off and reinserted into the list.

### Deallocation
A freed block should be inserted back into the list.
In principle, it is fine to just insert it at the beginning of the list.
Optionally, it can be checked whether adjacent free blocks also fit and can be merged. To do this, the list must be searched.

To test the allocator, the function `dump_free_list()` should be implemented to print the entire list on the screen.
Additionally, the demo application should be extended to test the deallocation of memory blocks.

The following image shows the heap with two free and three occupied blocks.

![List-Allocator](https://raw.githubusercontent.com/hhu-bsinfo/HeineOS/refs/heads/main/media/lesson-2/list2.png)

To use the list allocator, you need to change the static `ALLOCATOR` variable in [allocator/global.rs](https://github.com/hhu-bsinfo/HeineOS/blob/lesson-2/kernel/src/allocator/global.rs) to instantiate a `LinkedListAllocator` instead of a `BumpAllocator`.

*Note: As with the bump allocator, pointers must be aligned.*

![Heap Demo](https://raw.githubusercontent.com/hhu-bsinfo/HeineOS/refs/heads/main/media/lesson-2/heap.png)

## Assignment 2.3: PC Speaker
In this assignment, your goal is to play sounds on the PC speaker.
This is a very basic device that can only play a single beep tone at a time.
The most challenging part is to implement the `delay()` function to use the PIT (Programmable Interval Timer) to wait for a given amount of time.
This is essential to play tone sequences on the speaker, as we use it to hold a frequency for a given amount of time.

Since we do not have interrupts or a system time at the moment, we use channel 0 of the PIT. You need to configure it to count down to 0 in a fixed amount of time (e.g., 1 ms).
For this purpose, mode 2 (rate generator) should be used for channel 0. Once the counter reaches 0, the configured value is reloaded to start counting down again.
To wait for longer periods of time, you need to check the counter in an endless loop and count the number of times it reaches 0 (e.g., 100 times to wait for 100 ms).
Since it is probable that you will not read the counter at the exact time it reaches 0, you need to detect when the counter has already been reloaded and has started counting down again.

All in all, this is a rather hacky solution and will be replaced in the future, once we have a proper system time.
However, it is a good task to get familiar with low-level programming and the PIT, which will be useful in the future.

A good resource for learning how to use the PIT is [OSDev](http://wiki.osdev.org/Programmable_Interval_Timer).
Furthermore, a detailed description of the PIT is given in [8254.pdf](https://github.com/hhu-bsinfo/HeineOS/blob/main/slides/8254.pdf).

## Optional Assignment: Optimized Heap Allocator
As an optional extension to your linked list allocator, you can implement merging of adjacent free blocks.
This is important to reduce fragmentation in the heap and will also optimize the allocation process, as it reduces the number of elements in the list.

We recommend that you adapt your deallocation function to insert freed blocks back into the list in such a way that they are sorted by their start address.
Instead of just inserting them at the beginning of the list, you need to walk the list and insert them in the correct position.
Then you can check if a freshly inserted block is adjacent to its predecessor and/or successor and merge them if so.
