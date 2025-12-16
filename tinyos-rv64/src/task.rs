//! Processes + preemptive scheduler with Sv39 address spaces.
//!
//! - task0: idle (S-mode), uses kernel page table
//! - task1: user A (U-mode), own page table (kernel mapping shared, user mapping private)
//! - task2: user B (U-mode), own page table, will deliberately fault by touching kernel page
use core::arch::asm;
use core::sync::atomic::{AtomicUsize, Ordering};

use crate::{arch, mm, sbi, trap::TrapFrame, userimg};

const NUM_USER_TASKS: usize = 2;
const NUM_TASKS: usize = 1 + NUM_USER_TASKS;

const KSTACK_SIZE: usize = 16 * 1024;

#[repr(align(16))]
pub struct AlignedStack<const N: usize>(pub [u8; N]);

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    Ready,
    Running,
    Sleeping(u64), // wakeup tick
    Exited(i32),
}

pub struct Task {
    pub id: usize,
    pub is_user: bool,
    pub state: TaskState,
    pub trapframe: TrapFrame,
    pub kstack: AlignedStack<KSTACK_SIZE>,
    // Sv39 address space
    pub root_pa: usize, // root page table physical address
    pub satp: usize,    // satp value (Sv39 mode)
}

impl Task {
    pub const fn new(id: usize, is_user: bool) -> Self {
        Self {
            id,
            is_user,
            state: TaskState::Ready,
            trapframe: TrapFrame::zero(),
            kstack: AlignedStack([0; KSTACK_SIZE]),
            root_pa: 0,
            satp: 0,
        }
    }

    #[inline(always)]
    pub fn kstack_top(&self) -> usize {
        self.kstack.0.as_ptr() as usize + KSTACK_SIZE
    }
}

static mut TASKS: [Task; NUM_TASKS] = [
    Task::new(0, false),
    Task::new(1, true),
    Task::new(2, true),
];

static CURRENT: AtomicUsize = AtomicUsize::new(0);

extern "C" {
    fn __restore() -> !;
}

#[inline(always)]
pub fn current_id() -> usize {
    CURRENT.load(Ordering::Relaxed)
}

#[inline(always)]
pub fn current_root_pa() -> usize {
    let id = current_id();
    unsafe { TASKS[id].root_pa }
}

pub fn init() {
    let gp = arch::read_gp();
    unsafe {
        init_idle(&mut TASKS[0], idle_task as usize, gp);
        init_user(&mut TASKS[1], userimg::user_a(), gp);
        init_user(&mut TASKS[2], userimg::user_b(), gp);
    }
}

unsafe fn init_idle(task: &mut Task, entry: usize, gp: usize) {
    task.trapframe = TrapFrame::zero();
    task.trapframe.sepc = entry;
    task.trapframe.sp = task.kstack_top();
    task.trapframe.gp = gp;
    task.trapframe.tp = task.id;
    task.trapframe.sstatus = arch::SSTATUS_SPP | arch::SSTATUS_SPIE;
    task.trapframe.kernel_sp = 0;
    task.is_user = false;
    task.state = TaskState::Ready;
    task.root_pa = mm::kernel_root_pa();
    task.satp = mm::kernel_satp();
}

unsafe fn init_user(task: &mut Task, img: &[u8], gp: usize) {
    // Build a fresh address space for this user process.
    let root_pa = crate::mm::frame::alloc().expect("no memory: user root page table");
    crate::mm::pagetable::zero_table(root_pa);

    // Share kernel mapping: identity map DRAM as supervisor-only.
    crate::mm::pagetable::map_range(
        root_pa,
        mm::MEMORY_START,
        mm::MEMORY_START,
        mm::MEMORY_END - mm::MEMORY_START,
        mm::pte::PTE_V | mm::pte::PTE_R | mm::pte::PTE_W | mm::pte::PTE_X | mm::pte::PTE_A | mm::pte::PTE_D,
    );

    // Map user program at USER_BASE (RXU).
    let prog_pages = crate::mm::align_up(img.len(), mm::PAGE_SIZE) / mm::PAGE_SIZE;
    for i in 0..prog_pages {
        let pa = crate::mm::frame::alloc().expect("no memory: user program page");
        let dst = pa as *mut u8;
        let begin = i * mm::PAGE_SIZE;
        let end = core::cmp::min(begin + mm::PAGE_SIZE, img.len());
        unsafe {
            core::ptr::copy_nonoverlapping(img.as_ptr().add(begin), dst, end - begin);
        }
        let va = mm::USER_BASE + i * mm::PAGE_SIZE;
        crate::mm::pagetable::map_one(
            root_pa,
            va,
            pa,
            mm::pte::PTE_V | mm::pte::PTE_R | mm::pte::PTE_X | mm::pte::PTE_U | mm::pte::PTE_A,
        );
    }

    // Map user stack at [USER_STACK_TOP - N*PAGE, USER_STACK_TOP) (RWU).
    let stack_bottom = mm::USER_STACK_TOP - mm::USER_STACK_PAGES * mm::PAGE_SIZE;
    for i in 0..mm::USER_STACK_PAGES {
        let pa = crate::mm::frame::alloc().expect("no memory: user stack page");
        let va = stack_bottom + i * mm::PAGE_SIZE;
        crate::mm::pagetable::map_one(
            root_pa,
            va,
            pa,
            mm::pte::PTE_V | mm::pte::PTE_R | mm::pte::PTE_W | mm::pte::PTE_U | mm::pte::PTE_A | mm::pte::PTE_D,
        );
    }

    let satp = crate::mm::pagetable::make_satp(root_pa);

    task.root_pa = root_pa;
    task.satp = satp;
    task.is_user = true;
    task.state = TaskState::Ready;

    task.trapframe = TrapFrame::zero();
    task.trapframe.sepc = mm::USER_BASE;           // entry at base of program image
    task.trapframe.sp = mm::USER_STACK_TOP;        // user stack top (in user VA)
    task.trapframe.gp = gp;
    task.trapframe.tp = task.id;
    // Return to U-mode: SPP=0, enable interrupts after sret (SPIE -> SIE).
    task.trapframe.sstatus = arch::SSTATUS_SPIE;
    // Kernel stack used during traps from U-mode.
    task.trapframe.kernel_sp = task.kstack_top();
}

pub fn run_first() -> ! {
    let first = pick_next_runnable(0).unwrap_or(0);
    CURRENT.store(first, Ordering::Relaxed);
    unsafe {
        TASKS[first].state = TaskState::Running;
        mm::activate(TASKS[first].satp);
        arch::write_sscratch(&TASKS[first].trapframe as *const _ as usize);
        __restore();
    }
}

pub fn on_timer_tick(now: u64) {
    unsafe {
        for t in TASKS.iter_mut() {
            if let TaskState::Sleeping(deadline) = t.state {
                if now >= deadline {
                    t.state = TaskState::Ready;
                }
            }
        }
    }
}

pub fn yield_current() {
    unsafe {
        let id = current_id();
        if let TaskState::Running = TASKS[id].state {
            TASKS[id].state = TaskState::Ready;
        }
    }
}

pub fn sleep_current(wakeup_tick: u64) {
    unsafe {
        let id = current_id();
        TASKS[id].state = TaskState::Sleeping(wakeup_tick);
    }
}

pub fn exit_current(code: i32) {
    unsafe {
        let id = current_id();
        TASKS[id].state = TaskState::Exited(code);
    }
}

fn all_user_exited() -> bool {
    unsafe {
        for t in TASKS.iter() {
            if t.is_user && !matches!(t.state, TaskState::Exited(_)) {
                return false;
            }
        }
    }
    true
}

fn pick_next_runnable(from: usize) -> Option<usize> {
    for step in 1..=NUM_TASKS {
        let id = (from + step) % NUM_TASKS;
        unsafe {
            if matches!(TASKS[id].state, TaskState::Ready) {
                return Some(id);
            }
        }
    }
    None
}

pub fn schedule() {
    let cur = CURRENT.load(Ordering::Relaxed);

    if all_user_exited() {
        crate::println!("[kernel] all user tasks exited. shutdown.");
        sbi::shutdown();
    }

    unsafe {
        if matches!(TASKS[cur].state, TaskState::Running) {
            TASKS[cur].state = TaskState::Ready;
        }
    }

    let next = pick_next_runnable(cur).unwrap_or(0);
    CURRENT.store(next, Ordering::Relaxed);

    unsafe {
        TASKS[next].state = TaskState::Running;
        // Switch address space.
        mm::activate(TASKS[next].satp);
        // Switch context pointer.
        arch::write_sscratch(&TASKS[next].trapframe as *const _ as usize);
    }
}

fn idle_task() -> ! {
    loop {
        unsafe { asm!("wfi", options(nomem, nostack)); }
    }
}
