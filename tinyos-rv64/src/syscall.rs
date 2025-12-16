//! System call handling (ecall from U-mode).
//!
//! ABI:
//! - a7: syscall number
//! - a0..a2: args
//! - a0: return value
//!
//! Numbers are *ours*.
use crate::{mm, sbi, task, timer, trap::TrapFrame};

pub const SYS_WRITE: usize = 1;
pub const SYS_YIELD: usize = 2;
pub const SYS_EXIT: usize = 3;
pub const SYS_GET_TICKS: usize = 4;
pub const SYS_SLEEP: usize = 5;

fn write_user_bytes(root_pa: usize, user_ptr: usize, len: usize) -> Result<usize, ()> {
    let mut i = 0usize;
    while i < len {
        let va = user_ptr.wrapping_add(i);
        let (pa, flags) = mm::pagetable::translate(root_pa, va).ok_or(())?;
        // Enforce isolation: kernel will only read from U pages.
        if (flags & mm::pte::PTE_U) == 0 || (flags & mm::pte::PTE_R) == 0 {
            return Err(());
        }
        let ch = unsafe { (pa as *const u8).read_volatile() };
        sbi::console_putchar(ch);
        i += 1;
    }
    Ok(len)
}

pub fn handle_syscall(tf: &mut TrapFrame) {
    let num = tf.a7;
    let a0 = tf.a0;
    let a1 = tf.a1;
    let a2 = tf.a2;

    // skip `ecall`
    tf.sepc = tf.sepc.wrapping_add(4);

    match num {
        SYS_WRITE => {
            // write(fd, buf, len) -> written
            let _fd = a0;
            let root = task::current_root_pa();
            match write_user_bytes(root, a1, a2) {
                Ok(n) => tf.a0 = n,
                Err(_) => {
                    crate::println!(
                        "[kernel] kill user task {}: invalid user buffer buf=0x{:x} len={}",
                        task::current_id(),
                        a1,
                        a2
                    );
                    task::exit_current(-1);
                    task::schedule();
                }
            }
        }
        SYS_GET_TICKS => {
            tf.a0 = timer::ticks();
        }
        SYS_YIELD => {
            task::yield_current();
            task::schedule();
        }
        SYS_SLEEP => {
            let dur = a0 as u64;
            let now = timer::ticks() as u64;
            task::sleep_current(now.saturating_add(dur));
            task::schedule();
        }
        SYS_EXIT => {
            let code = a0 as i32;
            task::exit_current(code);
            task::schedule();
        }
        _ => {
            crate::println!(
                "[kernel] unknown syscall {} from task {}",
                num,
                task::current_id()
            );
            tf.a0 = usize::MAX;
        }
    }
}
