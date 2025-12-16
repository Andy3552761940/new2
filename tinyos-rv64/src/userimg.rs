//! Embedded user program images (as raw bytes), copied into user address space.
use core::slice;

extern "C" {
    static _user_a_start: u8;
    static _user_a_end: u8;
    static _user_b_start: u8;
    static _user_b_end: u8;
}

#[inline(always)]
pub fn user_a() -> &'static [u8] {
    unsafe {
        let start = &_user_a_start as *const u8 as usize;
        let end = &_user_a_end as *const u8 as usize;
        slice::from_raw_parts(start as *const u8, end - start)
    }
}

#[inline(always)]
pub fn user_b() -> &'static [u8] {
    unsafe {
        let start = &_user_b_start as *const u8 as usize;
        let end = &_user_b_end as *const u8 as usize;
        slice::from_raw_parts(start as *const u8, end - start)
    }
}
