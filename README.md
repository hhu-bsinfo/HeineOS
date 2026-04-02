# Lesson 7: Implement your own Application or OS Component

## Learning Goals
1. Implementing an application for your operating system
2. Alternatively, implement a new component or driver for your operating system

## Example Subjects
- Complex graphics demo (e.g., showcasing simple software rendered 3D graphics)
- Retro game (e.g., Snake, Tetris, Pac-Man, Pong, ...)
- A basic Shell that can start all demos and has further builtin commands (e.g., clear, time, meminfo ...)
- Extensions to the scheduler (e.g., sleep and join, priorities, ...)
- Device driver (e.g., IDE disk controller, RTL8139 network controller, ...)  
  *Does not have to be fully functional*

## Optional Assignment: PCI support
The given file [device/pci.rs](https://github.com/hhu-bsinfo/HeineOS/blob/lesson-7/kernel/src/device/pci.rs) contains code to scan the PCI bus for devices.
It can be tested with the demo code provided in [demo/lesson7.rs](https://github.com/hhu-bsinfo/HeineOS/blob/lesson-7/kernel/src/demo/lesson7.rs).

The demo function `lesson7::print_pci_devices()` prints the Vendor and Device IDs of all PCI devices to the screen.

![PCI Bus Scan](https://raw.githubusercontent.com/hhu-bsinfo/HeineOS/refs/heads/main/media/lesson-7/pci.png)

The demo function `lesson7::rtl8139_demo()` tries to find a Realtek RTL8139 network controller and prints its MAC address to the screen.
QEMU is already configured to emulate a Realtek RTL8139 network controller by our `Makefile.toml`.

![RTL8139](https://raw.githubusercontent.com/hhu-bsinfo/HeineOS/refs/heads/main/media/lesson-7/rtl8139.png)

The given code is already fully functional and can be used as a starting point for implementing your own device driver.
The demos showcase how to search for PCI devices and how to access their registers.
For further information about a device itself, you need to do some research. A good starting point is the [OSDev Wiki](https://wiki.osdev.org/).
The specification and programming guides of many older devices are available on the Internet.