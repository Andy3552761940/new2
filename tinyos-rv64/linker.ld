OUTPUT_ARCH(riscv)
ENTRY(_start)

BASE_ADDRESS = 0x80200000;

SECTIONS
{
  . = BASE_ADDRESS;

  stext = .;
  .text : ALIGN(4) {
    KEEP(*(.text.entry))
    *(.text .text.*)
  }
  etext = .;

  srodata = .;
  .rodata : ALIGN(8) {
    *(.rodata .rodata.*)
  }
  erodata = .;

  sdata = .;
  .data : ALIGN(8) {
    PROVIDE(__global_pointer$ = . + 0x800);
    *(.sdata .sdata.*)
    *(.data .data.*)
  }
  edata = .;

  sbss = .;
  .bss : ALIGN(8) {
    *(.sbss .sbss.*)
    *(.bss .bss.*)
    *(COMMON)
  }
  ebss = .;

  . = ALIGN(16);
  ekernel = .;
}
