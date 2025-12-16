//! Sv39 3-level page tables.
use super::{align_down, align_up, PAGE_SIZE};
use crate::mm::pte;

const PTES_PER_PAGE: usize = 512;

#[inline(always)]
fn vpn_indexes(va: usize) -> [usize; 3] {
    [
        (va >> 12) & 0x1ff, // vpn0
        (va >> 21) & 0x1ff, // vpn1
        (va >> 30) & 0x1ff, // vpn2
    ]
}

#[inline(always)]
fn pte_ptr(table_pa: usize, idx: usize) -> *mut u64 {
    (table_pa as *mut u64).wrapping_add(idx)
}

pub fn zero_table(table_pa: usize) {
    unsafe { core::ptr::write_bytes(table_pa as *mut u8, 0, PAGE_SIZE); }
}

/// Map one 4KiB page: va -> pa with flags.
pub fn map_one(root_pa: usize, va: usize, pa: usize, flags: u64) {
    let vpn = vpn_indexes(va);
    let mut table_pa = root_pa;

    // Walk level 2 -> level 1, creating intermediate tables.
    for level in (1..=2).rev() {
        let idx = vpn[level];
        let pte_addr = pte_ptr(table_pa, idx);
        let cur = unsafe { pte_addr.read_volatile() };
        if !pte::pte_is_valid(cur) {
            let new_table = crate::mm::frame::alloc().expect("out of memory: page table");
            let new_pte = pte::make_pte(new_table, pte::PTE_V);
            unsafe { pte_addr.write_volatile(new_pte); }
            table_pa = new_table;
        } else {
            let next_table_pa = pte::pte_ppn(cur) << 12;
            table_pa = next_table_pa;
        }
    }

    // Leaf at level 0.
    let idx0 = vpn[0];
    let leaf_addr = pte_ptr(table_pa, idx0);
    let leaf = pte::make_pte(pa, flags);
    unsafe { leaf_addr.write_volatile(leaf); }
}

/// Map a range [va, va+len) to [pa, pa+len) with 4KiB pages.
pub fn map_range(root_pa: usize, va_start: usize, pa_start: usize, len: usize, flags: u64) {
    let va0 = align_down(va_start, PAGE_SIZE);
    let pa0 = align_down(pa_start, PAGE_SIZE);
    let end = align_up(va_start + len, PAGE_SIZE);
    let mut va = va0;
    let mut pa = pa0;
    while va < end {
        map_one(root_pa, va, pa, flags);
        va += PAGE_SIZE;
        pa += PAGE_SIZE;
    }
}

/// Translate a virtual address to physical address using Sv39 page table.
/// Returns (pa, pte_flags).
pub fn translate(root_pa: usize, va: usize) -> Option<(usize, u64)> {
    let vpn = vpn_indexes(va);
    let mut table_pa = root_pa;

    for level in (0..=2).rev() {
        let idx = vpn[level];
        let pte_val = unsafe { pte_ptr(table_pa, idx).read_volatile() };
        if !pte::pte_is_valid(pte_val) {
            return None;
        }
        let flags = pte_val & 0x3ff;
        let ppn = pte::pte_ppn(pte_val);
        if (flags & (pte::PTE_R | pte::PTE_W | pte::PTE_X)) != 0 {
            // Leaf.
            let page_pa = ppn << 12;
            let page_off = va & 0xfff;
            return Some((page_pa + page_off, flags));
        } else {
            // Next level.
            table_pa = ppn << 12;
        }
    }
    None
}

/// satp for Sv39: MODE=8, ASID=0, PPN=root_ppn.
pub fn make_satp(root_pa: usize) -> usize {
    let root_ppn = root_pa >> 12;
    (8usize << 60) | (root_ppn & ((1usize << 44) - 1))
}
