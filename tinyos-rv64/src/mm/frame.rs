//! Very small physical frame allocator (bump allocator).
//!
//! - Page size: 4KiB
//! - Suitable for competition milestone (Sv39 + isolation demo).
//! - Next step: bitmap/buddy + free().
use core::sync::atomic::{AtomicUsize, Ordering};

use super::{align_up, PAGE_SIZE};

static START: AtomicUsize = AtomicUsize::new(0);
static END: AtomicUsize = AtomicUsize::new(0);
static NEXT: AtomicUsize = AtomicUsize::new(0);

pub fn init(free_start: usize, free_end: usize) {
    let start = align_up(free_start, PAGE_SIZE);
    START.store(start, Ordering::Relaxed);
    END.store(free_end, Ordering::Relaxed);
    NEXT.store(start, Ordering::Relaxed);
    crate::println!("[mm] frame allocator init: 0x{:x}..0x{:x}", start, free_end);
}

#[inline(always)]
fn alloc_inner() -> Option<usize> {
    let cur = NEXT.load(Ordering::Relaxed);
    let end = END.load(Ordering::Relaxed);
    if cur + PAGE_SIZE <= end {
        NEXT.store(cur + PAGE_SIZE, Ordering::Relaxed);
        Some(cur)
    } else {
        None
    }
}

/// Allocate one 4KiB frame, returns its physical address.
pub fn alloc() -> Option<usize> {
    let pa = alloc_inner()?;
    unsafe {
        core::ptr::write_bytes(pa as *mut u8, 0, PAGE_SIZE);
    }
    Some(pa)
}
