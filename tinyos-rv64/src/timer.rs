//! Timer tick via SBI set_timer + Supervisor Timer Interrupt.
use core::sync::atomic::{AtomicUsize, Ordering};

use crate::{arch, sbi};

// QEMU virt timebase is commonly 10MHz; this value is "good enough" for demo.
// You can later parse the DTB to get the exact timebase-frequency.
const TICK_INTERVAL: u64 = 100_000; // ~10ms if timebase is 10MHz

static TICKS: AtomicUsize = AtomicUsize::new(0);

pub fn init() {
    set_next_trigger();
    unsafe {
        arch::set_sie(arch::SIE_STIE);
    }
}

#[inline(always)]
pub fn ticks() -> usize {
    TICKS.load(Ordering::Relaxed)
}

#[inline(always)]
fn set_next_trigger() {
    let now = arch::read_time();
    sbi::set_timer(now + TICK_INTERVAL);
}

pub fn on_timer_interrupt() {
    TICKS.fetch_add(1, Ordering::Relaxed);
    set_next_trigger();
}
