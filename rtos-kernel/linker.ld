ENTRY(rtos_entry)

/* Load the kernel at 0x0020_0000 (what your bootloader expects) */
SECTIONS
{
  . = 0x0000000000200000;

  .text : ALIGN(16) {
    *(.text .text.*)
  }

  .rodata : ALIGN(16) {
    *(.rodata .rodata.*)
  }

  .data : ALIGN(16) {
    *(.data .data.*)
  }

  .bss : ALIGN(16) {
    *(COMMON)
    *(.bss .bss.*)
    . = ALIGN(16);
  }
}
