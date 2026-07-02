# Assignment 14: Implement your own Application or OS Component
To conclude this course, you should again implement an application or component for your operating system.
Of course, you can continue working on your project from [lesson 7](https://github.com/hhu-bsinfo/HeineOS/tree/lesson-7).
In that case, applications implemented in lesson 7 should now be ported to user space.

## Example subjects
- **User mode applications with multiple threads:** Currently, each application can only have a single thread executing the `main()` function.
  It would be desirable for an operating system to support multiple user threads per application.
- **Scheduler extensions:** Many extensions to the scheduler are possible.
  For example, it does not support sleeping or joining (waiting for another thread to end) right now.
  Furthermore, priorities could be introduced. You could also experiment with different scheduling algorithms.
- **Terminal:** A terminal application that can start other applications from the filesystem.
  The terminal itself should, of course, run in user space and start other applications via a system call.
  Furthermore, a mechanism for waiting for another process to exit is required.
  For demonstration purposes, multiple small applications or demos, that can be started from the terminal, should be implemented.
- **Filesystem extensions:** Currently, all open file handles are stored in a global map in `TarFs`.
  This is a problem, because any process can access any open file, even if it was opened by another process, by simply trying out different values as file handles.
  The solution is to have one table per process, either as part of the `Process` struct, or by mapping the process id to the map of open files for the process in `TarFs`.  
  Another extension to the filesystem would be creating and writing virtual (in memory) files in addition to the read-only TAR files.
