use core::ffi::c_char;

mod string;
mod ctype;
mod errno;
mod math;
mod stdlib;
mod stdio;
mod sys_stat;

fn str_from_c_ptr<'a>(c_str: *const c_char) -> &'a str {
    use core::ffi::CStr;

    if c_str.is_null() {
        return "";
    }

    unsafe { CStr::from_ptr(c_str).to_str().expect("libc: Invalid UTF-8 string") }
}