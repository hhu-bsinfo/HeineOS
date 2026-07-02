/*
 * Frontend for doomgeneric.
 * Required standard C functions are implemented in `libc`.
 *
 * Author: Fabian Ruhland, Heinrich Heine University Duesseldorf, 2026-06-18
 * License: GPLv3
 */
#![no_std]
#![feature(c_size_t)]
#![feature(c_variadic)]

extern crate alloc;

mod libc;

use alloc::ffi::CString;
use core::ffi::{c_char, c_int};
use core::panic::PanicInfo;
use usrlib::allocator::global::init_allocator;
use usrlib::framebuffer::Framebuffer;
use usrlib::key::Scancode;
use usrlib::once::Once;
use usrlib::spinlock::Spinlock;
use usrlib::user_api::{usr_get_system_time, usr_process_map_heap, usr_thread_exit, USER_HEAP_SIZE, USER_HEAP_VIRT_START};
use usrlib::println;

unsafe extern "C" {
    /// The buffer that Doom renders to.
    /// Its size is `DOOMGENERIC_RESX` * `DOOMGENERIC_RESY` * 4 bytes.
    /// In `DG_DrawFrame()` this buffer should be flushed to the screen.
    static DG_ScreenBuffer: *const u32;

    /// Initialize Doom.
    fn doomgeneric_Create(argc: c_int, argv: *mut *mut c_char) -> c_int;
    /// Advance the game state by one tick.
    /// This automatically calls all callback functions (e.g., `DG_DrawFrame()`, `DG_GetKey()`).
    fn doomgeneric_Tick();
}

/// Render resolution width.
const DOOMGENERIC_RESX: usize = 640;
/// Render resolution height.
const DOOMGENERIC_RESY: usize = 400;

/// The system time at which the application was started.
/// This is required by `DG_GetTicksMs()`
static START_TIME_MS: Once<usize> = Once::new();
/// Framebuffer instance that wraps `DG_ScreenBuffer` and flushes it to the screen.
static FRAMEBUFFER: Once<Spinlock<Framebuffer>> = Once::new();

#[repr(u8)]
#[derive(Clone, Copy)]
/// In-game keys.
pub enum DoomKey {
    RightArrow = 0xae,
    LeftArrow = 0xac,
    UpArrow = 0xad,
    DownArrow = 0xaf,
    StrafeL = 0xa0,
    StrafeR = 0xa1,
    Use = 0xa2,
    Fire = 0xa3,
    Escape = 27,
    Enter = 13,
    Tab = 9,
    F1 = 0x80 + 0x3b,
    F2 = 0x80 + 0x3c,
    F3 = 0x80 + 0x3d,
    F4 = 0x80 + 0x3e,
    F5 = 0x80 + 0x3f,
    F6 = 0x80 + 0x40,
    F7 = 0x80 + 0x41,
    F8 = 0x80 + 0x42,
    F9 = 0x80 + 0x43,
    F10 = 0x80 + 0x44,
    F11 = 0x80 + 0x57,
    F12 = 0x80 + 0x58,
    Backspace = 0x7f
}

/// Statistics for FPS calculation.
struct FpsInfo {
    /// Number of frames that have been rendered since the last FPS calculation.
    frame_count: usize,
    /// Time in milliseconds since the last FPS calculation.
    last_time_ms: usize,
    /// Number of frames that have been rendered during the last second.
    last_fps: usize,
}

impl TryFrom<Scancode> for DoomKey {
    type Error = ();

    /// Convert a keyboard scancode to an in-game doom key.
    fn try_from(value: Scancode) -> Result<Self, Self::Error> {
        match value {
            Scancode::Right => Ok(Self::RightArrow),
            Scancode::Left => Ok(Self::LeftArrow),
            Scancode::Up => Ok(Self::UpArrow),
            Scancode::Down => Ok(Self::DownArrow),
            Scancode::A => Ok(Self::StrafeL),
            Scancode::D => Ok(Self::StrafeR),
            Scancode::E => Ok(Self::Use),
            Scancode::Space => Ok(Self::Fire),
            Scancode::Escape => Ok(Self::Escape),
            Scancode::Enter => Ok(Self::Enter),
            Scancode::Tab => Ok(Self::Tab),
            Scancode::F1 => Ok(Self::F1),
            Scancode::F2 => Ok(Self::F2),
            Scancode::F3 => Ok(Self::F3),
            Scancode::F4 => Ok(Self::F4),
            Scancode::F5 => Ok(Self::F5),
            Scancode::F6 => Ok(Self::F6),
            Scancode::F7 => Ok(Self::F7),
            Scancode::F8 => Ok(Self::F8),
            Scancode::F9 => Ok(Self::F9),
            Scancode::F10 => Ok(Self::F10),
            Scancode::F11 => Ok(Self::F11),
            Scancode::F12 => Ok(Self::F12),
            Scancode::Backspace => Ok(Self::Backspace),
            _ => Err(())
        }
    }
}

#[unsafe(no_mangle)]
/// Called by Doom before the game starts.
/// This function should initialize any resources needed to run and display the game.
unsafe extern "C" fn DG_Init() {
    FRAMEBUFFER.init(|| unsafe {
        Spinlock::new(Framebuffer::new(DOOMGENERIC_RESX, DOOMGENERIC_RESY, DOOMGENERIC_RESX * 4, DG_ScreenBuffer as u64))
    });

    let time = usr_get_system_time();
    START_TIME_MS.init(|| time);
}

#[unsafe(no_mangle)]
/// Called by Doom after a frame has been rendered.
/// This function should flush the framebuffer to the screen.
unsafe extern "C" fn DG_DrawFrame() {
    todo!("DG_DrawFrame() has not been implemented yet!");
}

#[unsafe(no_mangle)]
/// Called by Doom when it needs to wait for a certain amount of time.
/// This function should block the current thread for the specified amount of time.
unsafe extern "C" fn DG_SleepMs(ms: u32) {
    todo!("DG_SleepMs() has not been implemented yet!");
}

#[unsafe(no_mangle)]
/// Called by Doom when it needs to get the time since application start.
/// This function should return the number of milliseconds since the application started.
unsafe extern "C" fn DG_GetTicksMs() -> u32 {
    todo!("DG_GetTicksMs() has not been implemented yet!");
}

#[unsafe(no_mangle)]
/// Called by Doom to get the next key event.
/// This function should check if a new key event is available.
/// If so, it should set `pressed` to 1 if the key was pressed or 0 if it was released
/// and `key` to corresponding key code.
/// If the scancode cannot be converted to `DoomKey` but has an ASCII representation,
/// `key` should be set to that ASCII character.
unsafe extern "C" fn DG_GetKey(pressed: *mut c_int, key: *mut c_char) -> c_int {
    todo!("DG_GetKey() has not been implemented yet!");
}

#[unsafe(no_mangle)]
/// Called by Doom to set the window title.
/// This function is not needed by our implementation.
unsafe extern "C" fn DG_SetWindowTitle(_title: *const c_char) {}

#[unsafe(link_section = ".main")]
#[unsafe(no_mangle)]
fn main() {
    usr_process_map_heap(USER_HEAP_VIRT_START, USER_HEAP_SIZE);
    init_allocator(USER_HEAP_VIRT_START as usize, USER_HEAP_SIZE);

    // The first argument to a program is always its name.
    // Since doomgeneric is a C program, we need to create a C-style string.
    let arg0 = CString::new("doom").unwrap();

    // The second argument is the IWAD file.
    let arg1 = CString::new("-iwad").unwrap();
    let arg2 = CString::new("doom.wad").unwrap();

    // Create argv array, consisting of pointers to C-style strings.
    let mut argv: [*mut c_char; 3] = [arg0.into_raw(), arg1.into_raw(), arg2.into_raw()];

    unsafe {
        // Call the doomgeneric initialization function.
        doomgeneric_Create(3, argv.as_mut_ptr());

        // Enter the main loop, calling doomgeneric_Tick() repeatedly.
        // This function handles all game logic and rendering and calls our DG_* functions as needed.
        loop {
            doomgeneric_Tick();
        }
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    usr_thread_exit();
}