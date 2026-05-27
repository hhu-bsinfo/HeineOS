use crate::device::pit;
use crate::device::terminal::terminal;
use crate::thread::scheduler::scheduler;
use crate::thread::thread::Thread;

pub fn user_thread_demo() {
    let k1 = Thread::new_kernel_thread(kernel_thread);
    let u1 = Thread::new_user_thread(user_thread);

    let scheduler = scheduler();
    scheduler.ready(k1);
    scheduler.ready(u1);

    terminal().lock().clear();
    scheduler.schedule();
}

fn kernel_thread() {
    let id = scheduler().get_active_tid();
    let cols = terminal().lock().size().0;

    for i in 0..cols - 1 {
        {
            let mut terminal = terminal().lock();
            terminal.set_pos(i, id);
            terminal.put_char('K');
        }

        pit::wait(1000);
    }
}

fn user_thread() {
    let id = scheduler().get_active_tid();
    let cols = terminal().lock().size().0;

    for i in 0..cols - 1 {
        {
            let mut terminal = terminal().lock();
            terminal.set_pos(i, id);
            terminal.put_char('U');
        }

        // User threads may not yield the CPU manually and thus cannot call pit::wait(),
        // as it calls Scheduler::yield_cpu() internally.
        // Instead, we implement a simple busy-wait loop here.
        let start = pit::system_time();
        while pit::system_time() - start < 1000 {
            core::hint::spin_loop();
        }
    }
}
