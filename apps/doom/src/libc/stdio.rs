/*
 * Contains functions from 'stdio.h'.
 *
 * Author: Fabian Ruhland, Heinrich Heine University Duesseldorf, 2026-06-18
 * License: GPLv3
 */
use crate::libc::errno::{set_errno, Errno};
use crate::libc::str_from_c_ptr;
use alloc::string::String;
use core::cmp::min;
use core::ffi::{c_char, c_int, c_long, c_size_t, c_void, VaList};
use core::slice;
use usrlib::user_api::{usr_file_close, usr_file_open, usr_file_read, usr_file_seek, usr_print, SeekMode};
use usrlib::{print, println};

/// The FILE type is used for file operations.
/// It must be defined by C standard library implementation.
/// In our case, the type can be anything.
/// We only need a `usize` for file operations and just cast the `FILE` pointers to `usize` and vice versa.
/// This is somewhat hacky but works for our simple implementation.
type FILE = c_size_t;

#[unsafe(no_mangle)]
static stdout: FILE = 0x00;
#[unsafe(no_mangle)]
static stderr: FILE = 0x01;

/// Convert a FILE pointer to a `usize` that can be used as a handle for filesystem operations.
fn file_to_handle(file: *mut FILE) -> usize {
    file as usize
}

/// Convert a `usize` that represents a handle to a FILE pointer.
fn handle_to_file(handle: usize) -> *mut FILE {
    handle as *mut FILE
}

#[unsafe(no_mangle)]
/// Open a file. We only support read access to files (mode must be 'r' or 'rb').
/// The returned `FILE` pointer is actually just the `usize` file handle, casted to a pointer.
pub(crate) unsafe extern "C" fn fopen(filename: *const c_char, mode: *const c_char) -> *mut FILE {
    let mode = str_from_c_ptr(mode);
    if mode != "r" && mode != "rb" {
        set_errno(Errno::EACCES);
        return core::ptr::null_mut();
    }

    let path = str_from_c_ptr(filename);

    // TODO: Open the file and return a FILE pointer.
    //       You can convert file handles to FILE pointers by calling `handle_to_file()`.

    todo!("fopen() has not been implemented yet!");
}

#[unsafe(no_mangle)]
/// Close a file.
pub(crate) unsafe extern "C" fn fclose(stream: *mut FILE) -> c_int {
    // TODO: Close the file represented by the given FILE pointer.
    //       You can convert file FILE pointers to handles by calling `file_to_handle()`.

    todo!("fclose() has not been implemented yet!");
}

#[unsafe(no_mangle)]
/// Read data from a file into a buffer.
/// The number of bytes to read is calculated by multiplying `size` and `count`.
/// Return the actual number of objects read (number of bytes / `size`).
pub(crate) unsafe extern "C" fn fread(buffer: *mut c_void, size: c_size_t, count: c_size_t, stream: *mut FILE) -> c_size_t {
    todo!("fread() has not been implemented yet!");
}

#[unsafe(no_mangle)]
/// Write data from a buffer to a file.
/// This is not supported by our implementation.
pub(crate) unsafe extern "C" fn fwrite(_buffer: *const c_void, _size: c_size_t, _count: c_size_t, _stream: *mut FILE) -> c_size_t {
    set_errno(Errno::EACCES);
    0
}

#[unsafe(no_mangle)]
/// Flush the output buffer of a file.
/// Since we do not have any output buffers, this function does effectively nothing.
pub(crate) unsafe extern "C" fn fflush(stream: *mut FILE) -> c_int {
    let handle = file_to_handle(stream);
    if handle != stdout as usize && handle != stderr as usize {
        // Only STDOUT and STDERR can be flushed.
        set_errno(Errno::EACCES);
        return -1;
    }

    0
}

#[unsafe(no_mangle)]
/// Seek to a given position in a file.
/// Return 0 on success.
pub(crate) unsafe extern "C" fn fseek(stream: *mut FILE, offset: c_long, origin: c_int) -> c_int {
    let mode = match origin {
        0 => SeekMode::Start,
        1 => SeekMode::Current,
        2 => SeekMode::End,
        _ => {
            set_errno(Errno::EDOM);
            return -1;
        }
    };

    todo!("fseek() has not been implemented yet!");
}

#[unsafe(no_mangle)]
/// Get the current position in a file.
/// NOTE: In HeineOS, the seek operation returns the new position.
///       Thus, `ftell()` can be implemented by seeking to the current position.
pub(crate) unsafe extern "C" fn ftell(stream: *mut FILE) -> c_long {
    todo!("ftell() has not been implemented yet!");
}

#[unsafe(no_mangle)]
/// Remove a file.
/// This is not supported by our implementation.
pub(crate) unsafe extern "C" fn remove(_pathname: *const c_char) -> c_int {
    set_errno(Errno::EACCES);
    -1
}

#[unsafe(no_mangle)]
/// Rename a file.
/// This is not supported by our implementation.
pub(crate) unsafe extern "C" fn rename(_old_filename: *const c_char, _new_filename: *const c_char) -> c_int {
    set_errno(Errno::EACCES);
    -1
}

#[unsafe(no_mangle)]
/// Write a character to stdout (i.e., the terminal).
/// Return the written character on success.
pub(crate) unsafe extern "C" fn putchar(ch: c_int) -> c_int {
    todo!("putchar() has not been implemented yet!");
}

#[unsafe(no_mangle)]
/// Write a string to stdout (i.e., the terminal), followed by a newline.
/// Return 0 on success.
pub(crate) unsafe extern "C" fn puts(str: *const c_char) -> c_int {
    todo!("puts() has not been implemented yet!");
}

#[unsafe(no_mangle)]
/// Write a formatted string to a file. We only support STDOUT and STDERR (i.e., the terminal).
/// We use the `printf_compat` crate for this, which provides a printf implementation, that is (almost fully) adherent to the C standard.
pub(crate) unsafe extern "C" fn vfprintf(stream: *mut FILE, format: *const c_char, vlist: VaList) -> c_int {
    let stream = stream as usize;
    if stream != stdout as usize && stream != stderr as usize {
        set_errno(Errno::EBADF);
        return -1;
    }


    unsafe {
        let mut string = String::new();
        let result = printf_compat::format(format, vlist, printf_compat::output::fmt_write(&mut string));

        usr_print(string.as_str());

        result
    }
}

#[unsafe(no_mangle)]
/// Write a formatted string to a given buffer.
/// We use the `printf_compat` crate for this, which provides a printf implementation, that is (almost fully) adherent to the C standard.
pub(crate) unsafe extern "C" fn vsnprintf(buffer: *mut c_char, bufsz: c_size_t, format: *const c_char, vlist: VaList) -> c_int {
    unsafe {
        let mut string = String::new();
        let result = printf_compat::format(format, vlist, printf_compat::output::fmt_write(&mut string));

        // Workaround for missing integer precision support in printf_compat.
        // Example: printf("STCFN%.3d", 33) should print "STCFN033", but it prints "STCFN33".
        // This hacky solution works with the DOOM shareware WAD but may not work with other WAD files.
        if string.starts_with("STCFN") {
            string.insert(5, '0');
        } else if string.starts_with("WIA") {
            string.insert(4, '0');
            string.insert(6, '0');
        }

        let bytes = string.as_bytes();
        let copy_len = min(bufsz as usize - 1, bytes.len());
        bytes.as_ptr().copy_to_nonoverlapping(buffer as *mut u8, copy_len);
        buffer.add(copy_len).write(0); // Null-terminate

        result
    }
}

#[unsafe(no_mangle)]
/// Scan a string for a given format.
/// This is not supported by our implementation.
pub(crate) unsafe extern "C" fn sscanf(_s: *const c_char, _format: *const c_char, _args: ...) -> c_int {
    0
}
