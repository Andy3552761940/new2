//! copy_from_user / copy_to_user via page table translation.
//!
//! Security rule for isolation demo:
//! - only allow copying to/from pages with PTE.U=1
//! - read requires PTE.R, write requires PTE.W
use super::{pagetable, pte, PAGE_SIZE};

fn min(a: usize, b: usize) -> usize {
    if a < b { a } else { b }
}

/// Copy user [src_va, src_va+dst.len()) into dst.
pub fn copy_from_user(root_pa: usize, dst: &mut [u8], src_va: usize) -> Result<(), ()> {
    let mut off = 0usize;
    while off < dst.len() {
        let va = src_va.wrapping_add(off);
        let (pa, flags) = pagetable::translate(root_pa, va).ok_or(())?;
        if (flags & pte::PTE_U) == 0 || (flags & pte::PTE_R) == 0 {
            return Err(());
        }
        let page_off = va & (PAGE_SIZE - 1);
        let n = min(PAGE_SIZE - page_off, dst.len() - off);
        unsafe {
            core::ptr::copy_nonoverlapping(
                pa as *const u8,
                dst.as_mut_ptr().add(off),
                n,
            );
        }
        off += n;
    }
    Ok(())
}

/// Copy src into user [dst_va, dst_va+src.len()).
pub fn copy_to_user(root_pa: usize, dst_va: usize, src: &[u8]) -> Result<(), ()> {
    let mut off = 0usize;
    while off < src.len() {
        let va = dst_va.wrapping_add(off);
        let (pa, flags) = pagetable::translate(root_pa, va).ok_or(())?;
        if (flags & pte::PTE_U) == 0 || (flags & pte::PTE_W) == 0 {
            return Err(());
        }
        let page_off = va & (PAGE_SIZE - 1);
        let n = min(PAGE_SIZE - page_off, src.len() - off);
        unsafe {
            core::ptr::copy_nonoverlapping(
                src.as_ptr().add(off),
                pa as *mut u8,
                n,
            );
        }
        off += n;
    }
    Ok(())
}
