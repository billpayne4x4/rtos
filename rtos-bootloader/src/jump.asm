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

jump_to_kernel:
    ; newline + entering
    lea     rsi, [rel nl_crlf]
    call    status_write
    lea     rsi, [rel msg_enter]
    call    status_write

    ; entry value
    lea     rsi, [rel msg_entry]
    call    status_write
    mov     rax, rcx
    call    print_hex64
    call    print_crlf

    ; dump 64 bytes at entry
    lea     rsi, [rel msg_dumpa]
    call    status_write
    mov     rax, rcx
    call    print_hex64
    call    print_crlf

    mov     rsi, rcx
    mov     ecx, 4          ; 4 rows * 16 bytes = 64
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
    call    print_hex8_nibble_pair   ; prints two hex + trailing space
    inc     rdi
    dec     edx
    jnz     .row_loop

    call    print_crlf
    add     rsi, 16
    pop     rcx
    dec     ecx
    jnz     .dump_rows

    ; rsp
    lea     rsi, [rel msg_rsp]
    call    status_write
    mov     rax, rsp
    call    print_hex64
    call    print_crlf

    ; rdi (BootInfo*)
    lea     rsi, [rel msg_rdi]
    call    status_write
    mov     rax, r8         ; bootinfo passed in r8 from Rust
    call    print_hex64
    call    print_crlf

    ; final handoff banner
    lea     rsi, [rel msg_handoff]
    call    status_write

    ; set up and jump
    mov     rsp, rdx        ; stack_top
    and     rsp, -16
    mov     rdi, r8         ; BootInfo*
    xor     rbp, rbp
    jmp     rcx             ; entry

; ---------------------------------------------------------------
; Helpers: output to COM1 and 0xE9

; putch: AL -> COM1 and 0xE9
; NOTE: preserve the character across the LSR poll; do NOT use SIL.
putch:
    push    rdx
    mov     bl, al          ; save the character
.wait_thr:
    mov     dx, 0x3FD       ; LSR
    in      al, dx
    test    al, 0x20        ; THR empty?
    jz      .wait_thr
    mov     dx, 0x3F8       ; COM1
    mov     al, bl
    out     dx, al
    mov     dx, 0x00E9      ; Bochs/QEMU debug port
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
    mov     bl, al
    shr     bl, 4
    lea     rdx, [rel hexchars]
    mov     al, [rdx+rbx]
    call    putch
    mov     bl, byte [rdi-1] ; restore original byte (already in AL previously)
    and     bl, 0x0F
    mov     al, [rdx+rbx]
    call    putch
    mov     al, ' '
    call    putch
    pop     rbx
    pop     rdx
    ret
