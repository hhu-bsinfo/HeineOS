/*
 * Contains functions from 'string.h'.
 *
 * Author: Fabian Ruhland, Heinrich Heine University Duesseldorf, 2026-06-18
 * License: GPLv3
 */
use core::cmp::min;
use core::ffi::{c_char, c_int, c_size_t, c_void};
use crate::libc::ctype;
use crate::libc::stdlib::malloc;

// These functions are compiler builtins, so we do not need to implement them ourselves.
unsafe extern "C" {
    pub(crate) fn memcmp(s1: *const c_void, s2: *const c_void, n: c_size_t) -> c_int;
    pub(crate) fn strlen(s: *const c_char) -> c_size_t;
}

#[unsafe(no_mangle)]
/// Compare two strings.
pub(crate) unsafe extern "C" fn strcmp(mut lhs: *const c_char, mut rhs: *const c_char) -> c_int {
    unsafe {
        loop {
            let lch = *lhs;
            let rch = *rhs;

            if lch == 0 && rch == 0 {
                return 0;
            }

            if lch < rch {
                return -1;
            } else if lch > rch {
                return 1;
            }

            lhs = lhs.add(1);
            rhs = rhs.add(1);
        }
    }
}

#[unsafe(no_mangle)]
/// Compare the first `count` characters of two strings.
pub(crate) unsafe extern "C" fn strncmp(mut lhs: *const c_char, mut rhs: *const c_char, count: c_size_t) -> c_int {
    unsafe {
        for _ in 0..count {
            let lch = *lhs;
            let rch = *rhs;

            if lch == 0 && rch == 0 {
                return 0;
            }

            if lch < rch {
                return -1;
            } else if lch > rch {
                return 1;
            }

            lhs = lhs.add(1);
            rhs = rhs.add(1);
        }

        0
    }
}

#[unsafe(no_mangle)]
/// Compare two strings ignoring case.
pub(crate) unsafe extern "C" fn strcasecmp(mut lhs: *const c_char, mut rhs: *const c_char) -> c_int {
    unsafe {
        loop {
            let lch = ctype::toupper(*lhs as c_int) as c_char;
            let rch = ctype::toupper(*rhs as c_int) as c_char;

            if lch == 0 && rch == 0 {
                return 0;
            }

            if lch < rch {
                return -1;
            } else if lch > rch {
                return 1;
            }

            lhs = lhs.add(1);
            rhs = rhs.add(1);
        }
    }
}

#[unsafe(no_mangle)]
/// Compare the first `count` characters of two strings ignoring case.
pub(crate) unsafe extern "C" fn strncasecmp(mut lhs: *const c_char, mut rhs: *const c_char, count: c_size_t) -> c_int {
    unsafe {
        for _ in 0..count {
            let lch = ctype::toupper(*lhs as c_int) as c_char;
            let rch = ctype::toupper(*rhs as c_int) as c_char;

            if lch == 0 && rch == 0 {
                return 0;
            }

            if lch < rch {
                return -1;
            } else if lch > rch {
                return 1;
            }

            lhs = lhs.add(1);
            rhs = rhs.add(1);
        }

        0
    }
}

#[unsafe(no_mangle)]
/// Copy a string (maximum `count` characters are copied).
pub(crate) unsafe extern "C" fn strncpy(dest: *mut c_char, src: *const c_char, count: c_size_t) -> *mut c_char {
    unsafe {
        let src_len = strlen(src);
        let bytes_to_copy = min(src_len, count);

        dest.copy_from(src, bytes_to_copy as usize);
        if count > src_len {
            let padding_size = count - src_len;
            dest.add(src_len as usize).write_bytes(0, padding_size as usize);
        }
    }

    dest
}

#[unsafe(no_mangle)]
/// Duplicate a string.
/// Memory for the new string is allocated on the heap and the caller is responsible for freeing it.
pub(crate) unsafe extern "C" fn strdup(str1: *const c_char) -> *mut c_char {
    unsafe {
        let len = strlen(str1);
        let dup = malloc(len + 1) as *mut u8;

        if !dup.is_null() {
            dup.copy_from_nonoverlapping(str1 as *const u8, len + 1);
        }

        dup as *mut c_char
    }
}

#[unsafe(no_mangle)]
/// Search for a character in a string.
/// Return a pointer to the character.
pub(crate) unsafe extern "C" fn strchr(str: *const c_char, ch: c_int) -> *mut c_char {
    let mut ptr = str;

    unsafe {
        loop {
            let current_char = *ptr;
            if current_char == (ch as c_char) {
                return ptr as *mut c_char;
            }
            if current_char == 0 {
                return core::ptr::null_mut();
            }

            ptr = ptr.add(1);
        }
    }
}

#[unsafe(no_mangle)]
/// Search for the last occurrence of a character in a string.
/// Return a pointer to the character.
pub(crate) unsafe extern "C" fn strrchr(str: *const c_char, ch: c_int) -> *mut c_char {
    let mut result: *mut c_char = core::ptr::null_mut();
    let mut ptr = str;

    unsafe {
        loop {
            let current_char = *ptr;
            if current_char == (ch as c_char) {
                result = ptr as *mut c_char;
            }
            if current_char == 0 {
                break;
            }

            ptr = ptr.add(1);
        }
    }

    result
}

#[unsafe(no_mangle)]
/// Search for a substring in a string.
/// Return a pointer to the beginning of the substring.
pub(crate) unsafe extern "C" fn strstr(str: *const c_char, substr: *const c_char) -> *mut c_char {
    let substr_len = unsafe { strlen(substr) };
    if substr_len == 0 {
        return str as *mut c_char;
    }

    let mut ptr = str;

    unsafe {
        while *ptr != 0 {
            if memcmp(ptr as *const c_void, substr as *const c_void, substr_len) == 0 {
                return ptr as *mut c_char;
            }
            
            ptr = ptr.add(1);
        }
    }

    core::ptr::null_mut()
}