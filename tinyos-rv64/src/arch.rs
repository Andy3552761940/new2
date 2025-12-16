//! CSR helpers (S-mode).
use core::arch::asm;

pub const SIE_SSIE: usize = 1 << 1; // supervisor software interrupt enable
pub const SIE_STIE: usize = 1 << 5; // supervisor timer interrupt enable

pub const SIP_SSIP: usize = 1 << 1; // supervisor software interrupt pending

pub const SSTATUS_SIE: usize = 1 << 1;
pub const SSTATUS_SPIE: usize = 1 << 5;
pub const SSTATUS_SPP: usize = 1 << 8; // 1 = Supervisor, 0 = User

#[inline(always)]
pub fn read_time() -> u64 {
    let x: u64;
    unsafe { asm!("rdtime {}", out(reg) x, options(nomem, nostack)); }
    x
}

#[inline(always)]
pub fn read_gp() -> usize {
    let x: usize;
    unsafe { asm!("mv {}, gp", out(reg) x, options(nomem, nostack)); }
    x
}

#[inline(always)]
pub unsafe fn write_stvec(addr: usize) {
    asm!("csrw stvec, {}", in(reg) addr, options(nomem, nostack));
}

#[inline(always)]
pub unsafe fn write_sscratch(val: usize) {
    asm!("csrw sscratch, {}", in(reg) val, options(nomem, nostack));
}

#[inline(always)]
pub unsafe fn set_sie(mask: usize) {
    asm!("csrs sie, {}", in(reg) mask, options(nomem, nostack));
}

#[inline(always)]
pub unsafe fn set_sip(mask: usize) {
    asm!("csrs sip, {}", in(reg) mask, options(nomem, nostack));
}

#[inline(always)]
pub unsafe fn clear_sip(mask: usize) {
    asm!("csrc sip, {}", in(reg) mask, options(nomem, nostack));
}

#[inline(always)]
pub unsafe fn write_satp(val: usize) {
    asm!("csrw satp, {}", in(reg) val, options(nomem, nostack));
}

#[inline(always)]
pub fn sfence_vma_all() {
    unsafe { asm!("sfence.vma x0, x0", options(nomem, nostack)); }
}

#[inline(always)]
pub fn wfi() {
    unsafe { asm!("wfi", options(nomem, nostack)); }
}
