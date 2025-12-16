use core::fmt::{self, Write};

struct Stdout;

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for ch in s.bytes() {
            crate::sbi::console_putchar(ch);
        }
        Ok(())
    }
}

pub fn _print(args: fmt::Arguments) {
    Stdout.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::console::_print(core::format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! println {
    () => { $crate::print!("\n") };
    ($fmt:expr) => { $crate::print!(core::concat!($fmt, "\n")) };
    ($fmt:expr, $($arg:tt)*) => { $crate::print!(core::concat!($fmt, "\n"), $($arg)*) };
}
