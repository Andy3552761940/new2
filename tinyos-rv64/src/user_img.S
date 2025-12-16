# Two tiny user programs in a raw-byte "image" section.
# The kernel will copy these bytes into user address space (low VA) and run in U-mode.
#
# Syscall ABI:
#   a7 = syscall number
#   a0..a2 = args
#
# Numbers:
#   1 write(fd, buf, len)
#   2 yield()
#   5 sleep(ticks)

.section .rodata.userimg
.align 4

.globl _user_a_start
.globl _user_a_end
_user_a_start:
1:
    li a0, 1                 # fd=1
    la a1, msg_a
    li a2, msg_a_len
    li a7, 1                 # SYS_WRITE
    ecall

    li a0, 5                 # sleep 5 ticks
    li a7, 5                 # SYS_SLEEP
    ecall

    li a7, 2                 # SYS_YIELD
    ecall

    j 1b

msg_a:
    .ascii "[U-A] alive\n"
.set msg_a_len, . - msg_a

_user_a_end:

.align 4
.globl _user_b_start
.globl _user_b_end
_user_b_start:
    li a0, 1
    la a1, msg_b
    li a2, msg_b_len
    li a7, 1                 # SYS_WRITE
    ecall

    # Deliberately touch a Supervisor-only page (kernel identity mapping with U=0).
    li t0, 0x80200000
    ld t1, 0(t0)             # -> load page fault, should be killed by kernel

    j _user_b_start

msg_b:
    .ascii "[U-B] about to touch kernel -> expect page fault\n"
.set msg_b_len, . - msg_b

_user_b_end:
