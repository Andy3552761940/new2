//! Sv39 memory management (minimal but competition-friendly).
pub mod copy;
pub mod frame;
pub mod pagetable;
pub mod pte;

use crate::arch;

pub const PAGE_SIZE: usize = 4096;

// QEMU virt: DRAM at 0x8000_0000, default size often 128MB.
pub const MEMORY_START: usize = 0x8000_0000;
pub const MEMORY_END: usize = 0x8800_0000; // 128MB

pub const USER_BASE: usize = 0x0001_0000;
pub const USER_STACK_TOP: usize = 0x0008_0000;
pub const USER_STACK_PAGES: usize = 4; // 16KB

static mut KERNEL_ROOT_PA: usize = 0;
static mut KERNEL_SATP: usize = 0;

extern "C" {
    fn ekernel();
}

#[inline(always)]
pub const fn align_down(addr: usize, align: usize) -> usize {
    addr & !(align - 1)
}

#[inline(always)]
pub const fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}

pub fn kernel_root_pa() -> usize {
    unsafe { KERNEL_ROOT_PA }
}

pub fn kernel_satp() -> usize {
    unsafe { KERNEL_SATP }
}

/// Initialize frame allocator, build a kernel page table (identity map DRAM),
/// enable Sv39 paging.
pub fn init() {
    let free_start = align_up(ekernel as usize, PAGE_SIZE);
    frame::init(free_start, MEMORY_END);

    let root_pa = frame::alloc().expect("no memory for kernel root page table");
    pagetable::zero_table(root_pa);

    // Identity-map DRAM as supervisor-only (U=0).
    pagetable::map_range(
        root_pa,
        MEMORY_START,
        MEMORY_START,
        MEMORY_END - MEMORY_START,
        pte::PTE_V | pte::PTE_R | pte::PTE_W | pte::PTE_X | pte::PTE_A | pte::PTE_D,
    );

    let satp = pagetable::make_satp(root_pa);
    unsafe { KERNEL_ROOT_PA = root_pa; }
    unsafe { KERNEL_SATP = satp; }

    unsafe { arch::write_satp(satp); }
    arch::sfence_vma_all();

    crate::println!("[mm] Sv39 enabled. kernel_satp=0x{:x}", satp);
}

#[inline(always)]
pub fn activate(satp: usize) {
    unsafe { arch::write_satp(satp); }
    arch::sfence_vma_all();
}
