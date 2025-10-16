BITS 64
default rel

section .rodata
text_crlf          db 0x0D,0x0A,0
text_entered       db 'TRAMPOLINE: Entered',13,10,0
text_entry         db 'TRAMPOLINE: Entry ',0
text_stack         db 'TRAMPOLINE: Stack Top ',0
text_bootinfo      db 'TRAMPOLINE: Boot Info ',0
text_memdump       db 'TRAMPOLINE: Memory at entry point:',13,10,0
text_jumping       db 'TRAMPOLINE: Jumping to Kernel Entry Point...',13,10,0
hex_alphabet       db '0123456789ABCDEF'

section .text
global jump_to_kernel

; RCX = entry, RDX = stack_top, R8 = BootInfo*
jump_to_kernel:
    ; Save the arguments we need BEFORE any operations
    mov     r15, rcx            ; R15 = entry (preserve it)
    mov     r14, rdx            ; R14 = stack_top
    mov     r13, r8             ; R13 = boot_info

    lea     rdi, [rel text_entered]
    call    write_status

    ; Print Entry
    lea     rdi, [rel text_entry]
    call    write_status
    mov     rax, r15
    call    write_hex64
    call    print_crlf

    ; Print Stack Top
    lea     rdi, [rel text_stack]
    call    write_status
    mov     rax, r14
    call    write_hex64
    call    print_crlf

    ; Print Boot Info
    lea     rdi, [rel text_bootinfo]
    call    write_status
    mov     rax, r13
    call    write_hex64
    call    print_crlf

    ; Dump memory at kernel entry
    lea     rdi, [rel text_memdump]
    call    write_status
    mov     rsi, r15            ; Entry address
    mov     ecx, 64             ; Dump 64 bytes
    call    dump_memory

    lea     rdi, [rel text_jumping]
    call    write_status

    ; Final setup for the jump - use preserved registers
    mov     rsp, r14            ; R14 = stack top
    and     rsp, -16            ; Align stack to 16 bytes
    mov     rdi, r13            ; R13 = boot_info (first argument)
    xor     rbp, rbp            ; Clear frame pointer

    ; Disable interrupts
    cli

    ; Jump to kernel entry (in R15)
    jmp     r15

; --- helper routines below ---

write_char:
    push    rdx
    push    rbx
    mov     bl, al
    mov     dx, 0x03F8
    out     dx, al
    mov     dx, 0x00E9
    out     dx, al
    pop     rbx
    pop     rdx
    ret

write_status:
    push    rax
    push    rdx
.ws_loop:
    mov     al, [rdi]
    test    al, al
    je      .ws_done
    call    write_char
    inc     rdi
    jmp     .ws_loop
.ws_done:
    pop     rdx
    pop     rax
    ret

print_crlf:
    push    rdi
    lea     rdi, [rel text_crlf]
    call    write_status
    pop     rdi
    ret

write_hex64:
    ; RAX holds the 64-bit value to print
    push    rdi
    push    rbx
    push    r10
    push    r11
    push    rax                 ; Save original RAX

    lea     rdi, [rel hex_alphabet]
    mov     ebx, 16
.wh_loop:
    rol     qword [rsp], 4      ; Rotate the saved value on stack
    mov     r10, [rsp]
    and     r10d, 0x0F
    mov     r11b, [rdi+r10]
    push    rax
    mov     al, r11b
    call    write_char
    pop     rax
    dec     ebx
    jnz     .wh_loop

    pop     rax                 ; Clean up stack
    pop     r11
    pop     r10
    pop     rbx
    pop     rdi
    ret

write_hex8:
    ; AL holds the byte to print
    push    rdi
    push    rbx
    push    rax

    lea     rdi, [rel hex_alphabet]

    ; High nibble
    mov     bl, al
    shr     bl, 4
    and     ebx, 0x0F
    mov     al, [rdi+rbx]
    call    write_char

    ; Low nibble
    mov     bl, [rsp]
    and     ebx, 0x0F
    mov     al, [rdi+rbx]
    call    write_char

    pop     rax
    pop     rbx
    pop     rdi
    ret

dump_memory:
    ; RSI = address to dump
    ; ECX = number of bytes
    push    rsi
    push    rcx
    push    rax

.dm_loop:
    test    ecx, ecx
    jz      .dm_done

    ; Print byte
    mov     al, [rsi]
    call    write_hex8

    ; Print space every byte
    mov     al, ' '
    call    write_char

    ; Print newline every 16 bytes
    mov     eax, ecx
    and     eax, 0x0F
    cmp     eax, 0x01
    jne     .dm_skip_newline
    call    print_crlf
.dm_skip_newline:

    inc     rsi
    dec     ecx
    jmp     .dm_loop

.dm_done:
    call    print_crlf
    pop     rax
    pop     rcx
    pop     rsi
    ret