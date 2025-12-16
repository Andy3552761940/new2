.section .text
.align 4
.globl __trap_entry
.globl __restore

# TrapFrame layout (must match src/trap.rs):
#  0   ra
#  8   sp
# 16   gp
# 24   tp
# 32   t0
# 40   t1
# 48   t2
# 56   s0
# 64   s1
# 72   a0
# 80   a1
# 88   a2
# 96   a3
# 104  a4
# 112  a5
# 120  a6
# 128  a7
# 136  s2
# 144  s3
# 152  s4
# 160  s5
# 168  s6
# 176  s7
# 184  s8
# 192  s9
# 200  s10
# 208  s11
# 216  t3
# 224  t4
# 232  t5
# 240  t6
# 248  sepc
# 256  sstatus
# 264  scause
# 272  stval
# 280  kernel_sp

__trap_entry:
    # t0 <-> sscratch:
    #   t0 becomes TrapFrame pointer, old t0 stored in sscratch temporarily.
    csrrw t0, sscratch, t0
    sd    sscratch, 32(t0)     # save original t0 into tf.t0
    csrw  sscratch, t0         # restore sscratch = TrapFrame pointer

    sd    ra, 0(t0)
    sd    sp, 8(t0)
    sd    gp, 16(t0)
    sd    tp, 24(t0)

    sd    t1, 40(t0)
    sd    t2, 48(t0)
    sd    s0, 56(t0)
    sd    s1, 64(t0)

    sd    a0, 72(t0)
    sd    a1, 80(t0)
    sd    a2, 88(t0)
    sd    a3, 96(t0)
    sd    a4, 104(t0)
    sd    a5, 112(t0)
    sd    a6, 120(t0)
    sd    a7, 128(t0)

    sd    s2, 136(t0)
    sd    s3, 144(t0)
    sd    s4, 152(t0)
    sd    s5, 160(t0)
    sd    s6, 168(t0)
    sd    s7, 176(t0)
    sd    s8, 184(t0)
    sd    s9, 192(t0)
    sd    s10, 200(t0)
    sd    s11, 208(t0)

    sd    t3, 216(t0)
    sd    t4, 224(t0)
    sd    t5, 232(t0)
    sd    t6, 240(t0)

    csrr  t1, sepc
    sd    t1, 248(t0)
    csrr  t1, sstatus
    sd    t1, 256(t0)
    csrr  t1, scause
    sd    t1, 264(t0)
    csrr  t1, stval
    sd    t1, 272(t0)

    # If trap comes from U-mode (SPP==0), switch to kernel stack for Rust handler.
    # sstatus.SPP bit = 8 (1 => previous privilege is S, 0 => U).
    ld    t1, 256(t0)
    li    t2, (1 << 8)
    and   t2, t2, t1
    bnez  t2, 1f
    ld    sp, 280(t0)
1:

    mv    a0, t0               # arg0 = &TrapFrame
    call  trap_handler

    j __restore

__restore:
    csrr  t0, sscratch         # t0 = TrapFrame pointer

    # Restore CSRs first.
    ld    t1, 248(t0)
    csrw  sepc, t1
    ld    t1, 256(t0)
    csrw  sstatus, t1

    # Restore integer registers.
    ld    ra, 0(t0)
    ld    sp, 8(t0)
    ld    gp, 16(t0)
    ld    tp, 24(t0)

    ld    t1, 40(t0)
    ld    t2, 48(t0)
    ld    s0, 56(t0)
    ld    s1, 64(t0)

    ld    a0, 72(t0)
    ld    a1, 80(t0)
    ld    a2, 88(t0)
    ld    a3, 96(t0)
    ld    a4, 104(t0)
    ld    a5, 112(t0)
    ld    a6, 120(t0)
    ld    a7, 128(t0)

    ld    s2, 136(t0)
    ld    s3, 144(t0)
    ld    s4, 152(t0)
    ld    s5, 160(t0)
    ld    s6, 168(t0)
    ld    s7, 176(t0)
    ld    s8, 184(t0)
    ld    s9, 192(t0)
    ld    s10, 200(t0)
    ld    s11, 208(t0)

    ld    t3, 216(t0)
    ld    t4, 224(t0)
    ld    t5, 232(t0)
    ld    t6, 240(t0)

    ld    t0, 32(t0)           # restore original t0

    sret
