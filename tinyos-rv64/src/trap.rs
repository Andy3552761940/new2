//! Trap handling (S-mode) with Sv39 enabled.
//!
//! - Timer interrupt: preemptive scheduling
//! - Software interrupt: optional internal yield
//! - ecall from U: syscall
//! - Page faults from U: kill current process (keep kernel alive)
use crate::{arch, syscall, task, timer};

extern "C" {
    fn __trap_entry();
}

/// Full register context saved/restored by `trap.S`.
///
/// NOTE: The layout must match `src/trap.S` offsets.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct TrapFrame {
    // x1..x31
    pub ra: usize,
    pub sp: usize,
    pub gp: usize,
    pub tp: usize,
    pub t0: usize,
    pub t1: usize,
    pub t2: usize,
    pub s0: usize,
    pub s1: usize,
    pub a0: usize,
    pub a1: usize,
    pub a2: usize,
    pub a3: usize,
    pub a4: usize,
    pub a5: usize,
    pub a6: usize,
    pub a7: usize,
    pub s2: usize,
    pub s3: usize,
    pub s4: usize,
    pub s5: usize,
    pub s6: usize,
    pub s7: usize,
    pub s8: usize,
    pub s9: usize,
    pub s10: usize,
    pub s11: usize,
    pub t3: usize,
    pub t4: usize,
    pub t5: usize,
    pub t6: usize,
    // CSRs we care about
    pub sepc: usize,
    pub sstatus: usize,
    pub scause: usize,
    pub stval: usize,
    // Kernel-only field (used when trapping from U-mode)
    pub kernel_sp: usize,
}

impl TrapFrame {
    pub const fn zero() -> Self {
        Self {
            ra: 0,
            sp: 0,
            gp: 0,
            tp: 0,
            t0: 0,
            t1: 0,
            t2: 0,
            s0: 0,
            s1: 0,
            a0: 0,
            a1: 0,
            a2: 0,
            a3: 0,
            a4: 0,
            a5: 0,
            a6: 0,
            a7: 0,
            s2: 0,
            s3: 0,
            s4: 0,
            s5: 0,
            s6: 0,
            s7: 0,
            s8: 0,
            s9: 0,
            s10: 0,
            s11: 0,
            t3: 0,
            t4: 0,
            t5: 0,
            t6: 0,
            sepc: 0,
            sstatus: 0,
            scause: 0,
            stval: 0,
            kernel_sp: 0,
        }
    }

    #[inline(always)]
    pub fn from_user(&self) -> bool {
        (self.sstatus & arch::SSTATUS_SPP) == 0
    }
}

#[inline(always)]
fn scause_is_interrupt(scause: usize) -> bool {
    (scause >> (usize::BITS - 1)) != 0
}

#[inline(always)]
fn scause_code(scause: usize) -> usize {
    scause & (!(1usize << (usize::BITS - 1)))
}

pub fn init() {
    unsafe {
        arch::write_stvec(__trap_entry as usize);
        // Enable supervisor software interrupt (optional yield).
        arch::set_sie(arch::SIE_SSIE);
    }
}

fn page_fault_kind(code: usize) -> Option<&'static str> {
    match code {
        12 => Some("instruction page fault"),
        13 => Some("load page fault"),
        15 => Some("store/amo page fault"),
        _ => None,
    }
}

/// Called from `trap.S` with a pointer to current task's trapframe.
#[no_mangle]
pub extern "C" fn trap_handler(tf: &mut TrapFrame) {
    let scause = tf.scause;
    if scause_is_interrupt(scause) {
        match scause_code(scause) {
            5 => {
                // Supervisor timer interrupt
                timer::on_timer_interrupt();
                task::on_timer_tick(timer::ticks() as u64);
                task::schedule();
            }
            1 => {
                // Supervisor software interrupt (optional internal yield)
                unsafe { arch::clear_sip(arch::SIP_SSIP); }
                task::schedule();
            }
            _ => {
                panic!(
                    "Unhandled interrupt: scause=0x{:x} sepc=0x{:x}",
                    tf.scause, tf.sepc
                );
            }
        }
    } else {
        let code = scause_code(scause);
        match code {
            8 => {
                // Environment call from U-mode -> syscall
                syscall::handle_syscall(tf);
            }
            _ => {
                if tf.from_user() {
                    if let Some(kind) = page_fault_kind(code) {
                        crate::println!(
                            "[kernel] kill user task {}: {} at va=0x{:x} sepc=0x{:x}",
                            task::current_id(),
                            kind,
                            tf.stval,
                            tf.sepc
                        );
                    } else {
                        crate::println!(
                            "[kernel] kill user task {}: exception scause=0x{:x} stval=0x{:x} sepc=0x{:x}",
                            task::current_id(),
                            tf.scause,
                            tf.stval,
                            tf.sepc
                        );
                    }
                    task::exit_current(-1);
                    task::schedule();
                } else {
                    panic!(
                        "Unhandled kernel exception: scause=0x{:x} stval=0x{:x} sepc=0x{:x}",
                        tf.scause, tf.stval, tf.sepc
                    );
                }
            }
        }
    }
}
