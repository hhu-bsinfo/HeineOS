/*
 * A basic round-robin scheduler for cooperative threads.
 * Priorities are not supported.
 *
 * Author: Michael Schoettner, Heinrich Heine University Duesseldorf, 2023-05-15
 *         Fabian Ruhland, Heinrich Heine University Duesseldorf, 2026-01-15
 * License: GPLv3
 */

use alloc::boxed::Box;
use core::fmt::Display;
use core::{fmt, ptr};
use crate::allocator;
use crate::library::once::Once;
use crate::library::queue::LinkedQueue;
use crate::library::spinlock::Spinlock;
use crate::thread::idle_thread::idle_thread;
use crate::thread::thread::Thread;

/// Global scheduler instance
static SCHEDULER: Once<Scheduler> = Once::new();

/// Global access to the scheduler.
pub fn scheduler() -> &'static Scheduler {
    SCHEDULER.init(Scheduler::new)
}

/// Unlock the scheduler state.
/// This function is called from assembly code.
/// Usually, the mutex would be unlocked automatically when going out of scope.
/// However, since we switch to a different thread in `yield_cpu()` and `exit()`,
/// the scope is not left and the mutex remains locked.
/// As a workaround, we provide this function to unlock the scheduler manually.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn unlock_scheduler() {
    unsafe {
        scheduler().state.force_unlock();
    }
}

/// The state of the scheduler.
/// It contains the active thread and the ready queue with all other threads.
/// The state is contained in its own struct so that it can be locked via a mutex.
struct SchedulerState {
    active_thread: Option<Box<Thread>>,
    ready_queue: LinkedQueue<Box<Thread>>
}

/// Represents the scheduler.
/// It is round-robin-based and uses a queue to manage the threads.
pub struct Scheduler {
    state: Spinlock<SchedulerState>,
}

impl Scheduler {
    /// Create a new scheduler instance with an empty ready queue
    /// and an idle thread as the active thread.
    pub fn new() -> Self {
        let state = SchedulerState {
            active_thread: Some(Thread::new(idle_thread)),
            ready_queue: LinkedQueue::new(),
        };

        Scheduler { state: Spinlock::new(state) }
    }

    /// Get the ID of the currently active thread.
    pub fn get_active_tid(&self) -> usize {
        let state = self.state.lock();

        state.active_thread.as_ref().unwrap().id()
    }

    /// Start the scheduler.
    /// This function must only be called once.
    pub fn schedule(&self) {
        let mut state = self.state.lock();

        // The active thread is never None, since we must at least have the idle thread.
        state.active_thread.as_mut().unwrap().start();
    }

    /// Register a new thread in the ready queue.
    pub fn ready(&self, thread: Box<Thread>) {
        let mut state = self.state.lock();

        state.ready_queue.enqueue(thread);
    }

    /// Terminate the current (calling) thread and switch to the next one.
    pub fn exit(&self) {
        let mut state = self.state.lock();

        // The active thread is never None, since we must at least have the idle thread.
        let mut current = state.active_thread.take().unwrap();
        // The idle thread never exits, so there must be at least one thread in the queue.
        let next = state.ready_queue.dequeue().unwrap();

        // Set the dequeued thread as the active thread,
        // overwriting the current one, which we want to exit.
        state.active_thread = Some(next);

        unsafe {
            // Switch to the next thread.
            // `current` still contains the old thread we want to exit,
            // while `state.active_thread` contains the next one.
            Thread::switch(current.as_mut(), state.active_thread.as_mut().unwrap().as_mut());
        }
    }

    /// Yield the CPU and switch to the next thread in the ready queue.
    pub fn yield_cpu(&self) {
        todo!("Scheduler::yield_cpu() is not implemented yet.");
    }

    /// Kill the thread with the given ID by removing it from the ready queue.
    pub fn kill(&self, to_kill_id: usize) {
        todo!("Scheduler::kill() is not implemented yet.");
    }
}

impl Display for Scheduler {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let state = self.state.lock();
        let active = state.active_thread.as_ref().unwrap();

        write!(f, "active: {}, ready: {}", active, state.ready_queue)
    }
}