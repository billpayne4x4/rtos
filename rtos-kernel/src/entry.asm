; Minimal ELF64 entry that calls into Rust kmain while keeping your bootloader’s
; RDI=BootInfo*, RSP already 16-byte aligned by the trampoline.

BITS 64
default rel

global rtos_entry
extern kmain

section .text.rtos_entry align=16
rtos_entry:
    ; System V ABI requires 16-byte alignment at function entry.
    ; A CALL pushes 8 bytes, so pre-bias by 8 before the call.
    sub     rsp, 8

    ; RDI already holds BootInfo* from the trampoline.
    call    kmain

.hang:
    hlt
    jmp .hang
