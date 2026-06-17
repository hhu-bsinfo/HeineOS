use core::fmt;
use core::fmt::Write;
use crate::spinlock::Spinlock;
use crate::user_api::usr_print;

pub struct Writer;

static WRITER: Spinlock<Writer> = Spinlock::new(Writer);

impl Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        usr_print(s);
        Ok(())
    }
}

pub fn print(args: fmt::Arguments) {
    WRITER.lock().write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
     ($($arg:tt)*) => ({
         $crate::print::print(format_args!($($arg)*));
     });
 }

#[macro_export]
macro_rules! println {
     ($fmt:expr) => ($crate::print!(concat!($fmt, "\n")));
     ($fmt:expr, $($arg:tt)*) => ($crate::print!(concat!($fmt, "\n"), $($arg)*));
 }