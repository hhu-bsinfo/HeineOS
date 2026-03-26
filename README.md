# Aufgabe 5: Preemptive Multithreading

## Learning Goals
1. Understand how preemptive multithreading works
2. Automatically yield the CPU in a fixed interval using the PIT
3. Avoid deadlocks in preemptive multithreading

## Assignment 5.1: Programmable Interval Timer (PIT)
From now on, we will use the PIT to implement a system timer and automatically switch between threads at a fixed interval.
The system time is stored in the variable `SYSTEM_TIME` (in [pit.rs](https://github.com/hhu-bsinfo/HeineOS/blob/lesson-5/kernel/src/device/pit.rs)) and should increment every time the PIT triggers an interrupt.
Use the PIT's counter 0 and mode 3 and load the counter with a suitable value so that the PIT triggers an interrupt every millisecond.
This way, `SYSTEM_TIME` shows how many ticks (i.e., milliseconds) have passed since the timer has been started.

Furthermore, the system time should be visualized on the screen using a rotating spinner.
Use the signs `| / - \` (given in `SPINNER_CHARS`) and change the current sign at a fixed interval (e.g., every 250 ms).
The current sign should be displayed at a fixed position on the screen (e.g., on the right upper corner).
To draw the spinner, you can access the framebuffer via `terminal::framebuffer()`.
However, you need to lock the framebuffer instance before you can draw to it.
This is a potential source for a deadlock, since we are currently inside an interrupt handler.
To avoid this, use `try_lock()` and only draw the spinner if the lock can be acquired.

![System Time Spinner](https://raw.githubusercontent.com/hhu-bsinfo/HeineOS/refs/heads/main/media/lesson-5/spinner.gif)

Finally, implement `pit::plugin()` to initialize the PIT using `TIMER.init(|| { ... })`, set the interrupt interval and register the interrupt in `interrupt/dispatcher.rs`.
Additionally, the PIT interrupts should be allowed in the PIC. Call `pit::plugin()` in `startup.rs` to start the timer.

If you want to, you can now use the system time in your log messages, printing it, for example, at the beginning of each message.

## Assignment 5.2: Waiting using the System Time
Now that we have an incrementing system time, we can use it to implement a waiting function.
The function `pit::wait(ms: usize)` should loop until the given number of milliseconds has passed.

To test your implementation, replace all `delay()` calls in the PC speaker driver with `pit::wait()`.
The `delay()` function cannot be used anymore, since it programs the PIT directly using its counter 0, which would interfere with our system time.

Playing the builtin melodies should work as before when using `pit::wait()` instead of `delay()`.

## Assignment 5.3: Switching Threads using the PIT
In this assignment, the PIT interrupt will be used to switch threads at a fixed interval.

Start by adding a new instance variable to `SchedulerState` called `initialized` to indicate whether the scheduler is already running.
It should set to `false` initially and switched to `true` in `Scheduler::schedule()` (directly before the first thread is started).

In `Scheduler::yield_cpu()`, you should now check whether the scheduler is already running and only switch threads if so.
Furthermore, we must make sure to not cause a deadlock when locking `Scheduler::state`.
This can happen, if the pit interrupt is triggered and initiates a thread switch, while `Scheduler::state` is locked.
In this case, the system would get stuck at acquiring the lock and never return from the interrupt.
Use `try_lock()` to check whether the lock can be acquired and return if it cannot.  
There is still one potential deadlock left: As the scheduler modifies the thread queue in `yield_cpu()`, the allocator is invoked.
If the allocator is currently locked while the pit triggers a thread switch, we also run into a deadlock.
To avoid this, check whether the allocator is currently locked in `yield_cpu()`, using `allocator::global::is_allocator_locked()` and only switch threads if it returns `false`.

Once you have modified the scheduler, call `Scheduler::yield_cpu()` in the PIT's interrupt handler at a fixed interval.
Every 10 ms is a good choice. Smaller values will slow down the system, as more time is spent in the scheduler.
Too large values can cause the system to be unresponsive, as threads are not switched quickly enough.
You are, of course, free to experiment with different values and see how the system behaves.

Finally, there is one deadlock left that we must fix:
Every interrupt is first handled by `dispatch_interrupt()` in `interrupt/dispatcher.rs`, which locks `INT_VECTORS`.
The lock is automatically released after the interrupt handler returns.
However, if the PIT switches threads, we directly return into the next thread from within the PIT's interrupt handler.
Because of this, the lock protecting `INT_VECTORS` is not released.
To fix this, we must call `interrupt::dispatcher::unlock_int_vectors()` before calling `Scheduler::yield_cpu()` inside the PIT's interrupt handler.
This force unlocks `INT_VECTORS`, which would usually be highly unsafe. But since we only do this inside an interrupt handler, which cannot be interrupted, it is safe (although somewhat ugly).

## Assignment 5.4: Preemptive Multithreading Demo
Test your implementation with the multithreading demo from assignment 4.5.
However, remove the `yield_cpu()` call, since we now want to check whether preemptive multithreading works.
Furthermore, an additional thread should be spawned that plays a melody via the PC speaker.

You will probably notice the following behavior:
1. Primarily, only one thread outputs its counter. Only occasionally, we can observe a switch to another thread.
2. Nevertheless, the PC speaker plays a melody, which means that scheduler must be working correctly.
3. The system time spinner seems to be updated irregularly.

This behavior is caused by the fact that our threads all need to acquire a lock on the terminal, which in turn also locks the framebuffer when drawing characters.
Since our demo threads spent most of their time printing to the screen, there is only a tiny time window during which the terminal (and the framebuffer) is not locked.
Because of this, the thread that first acquires the terminal lock keeps it for a long time, as the probability of a thread switch at the exact (short) moment when it releases the lock is only small.
This causes the other two threads to *starve* and is also the reason the system time spinner is drawn less frequently, as the PIT's interrupt handler cannot acquire the lock most of the time.

Try inserting a `pit::wait(100)` call after each time the thread outputs its counter (and the terminal lock is released).
You should now see all three threads output their counter, albeit very slowly.
This works, because now the probability of a thread switch while the thread is inside `pit::wait()` is much higher than while it holds the terminal lock.
However, this is not a good solution, since it wastes a lot of computation time.
A better solution would be, to manually call `scheduler::yield_cpu()` every few iterations (e.g., 10).

In general, this demo is pathological for such kind of behavior.
In real-world scenarios, there would be much more computation done besides drawing to the screen.
However, it shows the pitfalls we have to be aware of when using preemptive multithreading.

Finally, calling `Scheduler::kill()` is not a good idea in a preemptive environment.
If we kill a thread while it holds a lock, the lock will never be released.
For this reason, you do not need to kill threads anymore in the demo. Instead, let them exit when their counter reaches a certain value.

Your demo should now look like this:

![Thread Demo](https://raw.githubusercontent.com/hhu-bsinfo/HeineOS/refs/heads/main/media/lesson-5/threads.gif)

## Assignment 5.5: Freeing Thread Resources
Each thread allocates its stack on the heap. This should be freed automatically when the thread's lifetime ends.
Look at `Scheduler::exit()`: The thread that terminates is taken out of `SchedulerState::active_thread` and **not** inserted into `SchedulerState::ready_queue`.
Usually, the thread should now automatically be dropped (deallocating its stack) once `Scheduler::exit()` returns.
In fact, the compiler inserts the `drop()` call at the end of `Scheduler::exit()`, but since we call `Thread::switch()` beforehand, the implicit `drop()` call is never executed.

A first attempt at fixing this, might be to call `drop()` manually on the thread before calling `Thread::switch()`.
However, there are two problems with this approach:
1. We still need the thread instance during `Thread::switch()`.
2. During the `drop()` call, the heap allocator will get locked. If it was already locked before, we run into a deadlock because the scheduler state is also currently locked, preventing any other thread switches.

To work around these problems, you should introduce a second queue to `SchedulerState` called `terminated_threads`.
In `Scheduler::exit()`, the terminated thread should be inserted into this queue before calling `Thread::switch()`.
This way, all terminated threads are gathered in one place with `terminated_threads` having the ownership of all the instances.  
Now, implement a new function `cleanup_terminated_threads()` in the scheduler that dequeues all threads from `SchedulerState::terminated_threads`, thus freeing their resources.
Note that you do not need to call `drop()` on the threads, just dequeuing them is enough, as Rust will automatically drop them once they have no owner anymore.
Finally, use the idle thread to call `cleanup_terminated_threads()` regularly.

## Optional Assignment: Performance Optimizations
In this optional assignment, we will tune some parts of the code to improve the performance of our operating system.

### Waste less time when waiting
As a first step, yield the CPU manually while waiting in `pit::wait()`.
This way, the CPU spends less time spinning in the wait loop but rather switches to other threads.

### More efficient locks
The same applies for our `Spinlock`. Manually switch to next thread while the lock cannot be acquired in `Spinlock::lock()`.
This should greatly improve the performance of our thread demo.

To see the performance improvements, you can measure the time it takes each thread to count to a specific value (e.g., 1000)
Store the system time before entering the loop in a variable. Then use it to calculate the time it took to count to the desired value.
Print the time it took for each thread at the end of the loop.

Now start the demo one time without the optimized `Spinlock` and one time with the optimizations applied.
In my testing, I could see a 3x performance improvement for the thread demo.

| Without optimized Spinlock                                                                                                                                 | With optimized Spinlock                                                                                                                               |
|------------------------------------------------------------------------------------------------------------------------------------------------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------|
| ![Thread Demo Without Spinlock Optimizations](https://raw.githubusercontent.com/hhu-bsinfo/HeineOS/refs/heads/main/media/lesson-5/threads_unoptimized.png) | ![Thread Demo With Spinlock Optimizations](https://raw.githubusercontent.com/hhu-bsinfo/HeineOS/refs/heads/main/media/lesson-5/threads_optimized.png) |


### Compiler optimizations
Until now, we have used *debug* build to test our system. In such builds, the compiler does not optimize the code as much as it does in *release* builds.
You can use the following command to enable compiler optimization:

```shell
cargo make --no-workspace --profile production qemu
```

This should significantly improve the performance of our system. However, be aware that you cannot debug release builds.
Use debug builds during development and for fixing problems and release builds to test the final system.

While it took the demo threads around 7 seconds to count to 1000 in my testing before, they now count to 150000 in roughly the same time.
This is a huge performance improvement of 150x just by enabling compiler optimizations.
As you will see, especially drawing to the screen is now much faster, and it is highly recommended to test your system with compiler optimizations enabled during the next assignments.

![Thread Demo With Compiler Optimizations](https://raw.githubusercontent.com/hhu-bsinfo/HeineOS/refs/heads/main/media/lesson-5/threads_release.png)