#![no_std]
#![no_main]

mod arch;
mod console;
mod mm;
mod panic;
mod sbi;
mod syscall;
mod task;
mod timer;
mod trap;
mod userimg;

use core::arch::global_asm;

// Assembly entry & trap vectors & user images.
global_asm!(include_str!("entry.S"));
global_asm!(include_str!("trap.S"));
global_asm!(include_str!("user_img.S"));

extern "C" {
    fn sbss();
    fn ebss();
}

#[inline(always)]
unsafe fn clear_bss() {
    let mut cur = sbss as usize;
    let end = ebss as usize;
    while cur < end {
        (cur as *mut u8).write_volatile(0);
        cur += 1;
    }
}

/// OpenSBI will jump to `_start` with:
/// - a0 = hart id
/// - a1 = device tree blob (DTB) physical address
#[no_mangle]
pub extern "C" fn rust_main(hart_id: usize, dtb_pa: usize) -> ! {
    unsafe { clear_bss(); }

    println!("\n=== TinyOS-RV64 boot ===");
    println!("hart_id = {}, dtb_pa = 0x{:x}", hart_id, dtb_pa);

    mm::init();          // enable Sv39 paging (kernel identity map)
    trap::init();        // stvec
    timer::init();       // timer interrupt
    task::init();        // create processes + per-process page tables

    println!("init done. enter scheduler...");
    task::run_first()
}
