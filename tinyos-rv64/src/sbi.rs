//! Minimal SBI legacy calls (works with OpenSBI used by QEMU virt).
use core::arch::asm;

#[inline(always)]
fn sbi_call(which: usize, arg0: usize, arg1: usize, arg2: usize) -> usize {
    let ret: usize;
    unsafe {
        asm!(
            "ecall",
            inlateout("a0") arg0 => ret,
            in("a1") arg1,
            in("a2") arg2,
            in("a7") which,
            options(nostack)
        );
    }
    ret
}

/// Legacy SBI: console_putchar (a7=1, a0=ch)
#[inline(always)]
pub fn console_putchar(ch: u8) {
    let _ = sbi_call(1, ch as usize, 0, 0);
}

/// Legacy SBI: set_timer (a7=0, a0=stime_value)
#[inline(always)]
pub fn set_timer(stime_value: u64) {
    let _ = sbi_call(0, stime_value as usize, 0, 0);
}

/// Legacy SBI: shutdown (a7=8)
pub fn shutdown() -> ! {
    let _ = sbi_call(8, 0, 0, 0);
    loop {
        unsafe { asm!("wfi", options(nomem, nostack)); }
    }
}
