BITS 64
default rel

section .rodata
nl_crlf     db 0x0D,0x0A,0
msg_enter   db 'TRAMPOLINE: entering',13,10,0
msg_entry   db 'TRAMPOLINE: entry=',0
msg_dumpa   db 'TRAMPOLINE: dump[64] @',0
msg_rsp     db 'TRAMPOLINE: rsp=',0
msg_rdi     db 'TRAMPOLINE: rdi(BootInfo*)=',0
msg_handoff db 'TRAMPOLINE: handoff to kernel',13,10,0
hexchars    db '0123456789abcdef'

section .text align=16
global jump_to_kernel

; Win64/UEFI ABI:
;   RCX = _a
;   RDX = _b
;   R8  = stack_top
;   R9  = entry
;   [RSP+40] = boot_info   (retaddr=8 + shadow space=32)
jump_to_kernel:
    ; capture boot_info BEFORE touching RSP
    mov     r10, [rsp+40]        ; r10 = boot_info

    ; pretty logs
    lea     rsi, [rel nl_crlf]
    call    status_write
    lea     rsi, [rel msg_enter]
    call    status_write

    lea     rsi, [rel msg_entry]
    call    status_write
    mov     rax, r9              ; entry
    call    print_hex64
    call    print_crlf

    lea     rsi, [rel msg_dumpa]
    call    status_write
    mov     rax, r9
    call    print_hex64
    call    print_crlf

    ; dump 64 bytes at entry
    mov     rsi, r9
    mov     ecx, 4
.dump_rows:
    push    rcx
    mov     rax, rsi
    call    print_hex64
    mov     al, ':'
    call    putch
    mov     al, ' '
    call    putch

    mov     rdi, rsi
    mov     edx, 16
.row_loop:
    mov     al, [rdi]
    call    print_hex8_nibble_pair
    inc     rdi
    dec     edx
    jnz     .row_loop

    call    print_crlf
    add     rsi, 16
    pop     rcx
    dec     ecx
    jnz     .dump_rows

    ; rsp before switch (for logging)
    lea     rsi, [rel msg_rsp]
    call    status_write
    mov     rax, rsp
    call    print_hex64
    call    print_crlf

    ; rdi (BootInfo*)
    lea     rsi, [rel msg_rdi]
    call    status_write
    mov     rax, r10             ; boot_info captured from [rsp+40]
    call    print_hex64
    call    print_crlf

    ; final handoff banner
    lea     rsi, [rel msg_handoff]
    call    status_write

    ; set up and jump (Win64): rsp <- r8 (stack), rdi <- boot_info, jmp r9 (entry)
    mov     rsp, r8              ; stack_top
    and     rsp, -16
    mov     rdi, r10             ; BootInfo* in first arg register (rdi is ignored on Win64 ABI for the callee,
                                 ; but we're tail-jumping into kernel; use rdi as agreed kernel ABI)
    xor     rbp, rbp
    jmp     r9                   ; entry

; ---------------------------------------------------------------
; Helpers: output to COM1 and 0xE9

; putch: AL -> COM1 and 0xE9
putch:
    push    rdx
    mov     bl, al
.wait_thr:
    mov     dx, 0x3FD
    in      al, dx
    test    al, 0x20
    jz      .wait_thr
    mov     dx, 0x3F8
    mov     al, bl
    out     dx, al
    mov     dx, 0x00E9
    out     dx, al
    pop     rdx
    ret

; status_write: prints NUL-terminated string at RSI
status_write:
    push    rax
    push    rdx
    push    rsi
.sw_loop:
    mov     al, [rsi]
    test    al, al
    je      .sw_done
    call    putch
    inc     rsi
    jmp     .sw_loop
.sw_done:
    pop     rsi
    pop     rdx
    pop     rax
    ret

print_crlf:
    push    rax
    mov     al, 0x0D
    call    putch
    mov     al, 0x0A
    call    putch
    pop     rax
    ret

; print_hex64: RAX -> 16 hex chars
print_hex64:
    push    rcx
    push    rbx
    push    rdx
    mov     ecx, 16
.ph64_loop:
    rol     rax, 4
    mov     bl, al
    and     bl, 0x0F
    lea     rdx, [rel hexchars]
    mov     al, [rdx+rbx]
    call    putch
    dec     ecx
    jnz     .ph64_loop
    pop     rdx
    pop     rbx
    pop     rcx
    ret

; print_hex8_nibble_pair: AL -> two hex chars + space
print_hex8_nibble_pair:
    push    rdx
    push    rbx
    mov     dl, al
    lea     rdx, [rel hexchars]
    mov     bl, dl
    shr     bl, 4
    and     bl, 0x0F
    mov     al, [rdx+rbx]
    call    putch
    mov     bl, dl
    and     bl, 0x0F
    mov     al, [rdx+rbx]
    call    putch
    mov     al, ' '
    call    putch
    pop     rbx
    pop     rdx
    ret

