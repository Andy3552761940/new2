use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    crate::println!("\n\n!!! KERNEL PANIC !!!");
    if let Some(loc) = info.location() {
        crate::println!("at {}:{}:{}", loc.file(), loc.line(), loc.column());
    }
    if let Some(msg) = info.message() {
        crate::println!("message: {}", msg);
    }
    crate::println!("system shutdown.");
    crate::sbi::shutdown();
}
