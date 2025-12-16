.section .text.entry
.globl _start
_start:
    # Set up boot stack (only used before first task starts).
    la  sp, boot_stack_top

    # Set up global pointer for RISC-V small data.
    la  gp, __global_pointer$

    # OpenSBI provides:
    #   a0 = hartid
    #   a1 = dtb physical address
    call rust_main

1:
    j 1b

.section .bss.stack
.align 12
boot_stack:
    .space 4096 * 4
boot_stack_top:
