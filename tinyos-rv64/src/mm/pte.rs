//! Sv39 page table entry flags.
pub const PTE_V: u64 = 1 << 0;
pub const PTE_R: u64 = 1 << 1;
pub const PTE_W: u64 = 1 << 2;
pub const PTE_X: u64 = 1 << 3;
pub const PTE_U: u64 = 1 << 4;
pub const PTE_G: u64 = 1 << 5;
pub const PTE_A: u64 = 1 << 6;
pub const PTE_D: u64 = 1 << 7;

#[inline(always)]
pub const fn pte_is_valid(pte: u64) -> bool {
    (pte & PTE_V) != 0
}

#[inline(always)]
pub const fn pte_ppn(pte: u64) -> usize {
    ((pte >> 10) & ((1u64 << 44) - 1)) as usize
}

#[inline(always)]
pub const fn make_pte(pa: usize, flags: u64) -> u64 {
    let ppn = (pa >> 12) as u64;
    (ppn << 10) | flags
}
