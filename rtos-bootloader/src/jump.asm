BITS 64
default rel
section .rodata
color_rgb:  dd 0x00612D00
color_bgr:  dd 0x00002D61
logo_data:
    incbin "../images/rtos-logo-transparent.raw"
logo_width:  dd 1024
logo_height: dd 1024
section .text
global jump_to_kernel
jump_to_kernel:
    mov     r15, rcx
    mov     r14, rdx
    mov     r13, r8
    mov     rbx, [r13 + 0x00]
    mov     r12d, [r13 + 0x10]
    mov     r11d, [r13 + 0x14]
    mov     r10d, [r13 + 0x18]
    mov     eax,  [r13 + 0x1C]
    cmp     eax, 0
    je      .use_bgr
    cmp     eax, 1
    je      .use_rgb
    jmp     .skip_fill
.use_bgr:
    mov     eax, dword [rel color_bgr]
    mov     r8d, 0
    jmp     .fill_screen
.use_rgb:
    mov     eax, dword [rel color_rgb]
    mov     r8d, 1
.fill_screen:
    xor     ecx, ecx
.fill_row:
    cmp     ecx, r11d
    jae     .draw_logo
    mov     edx, ecx
    imul    rdx, r10
    lea     rsi, [rbx + rdx*4]
    mov     edi, r12d
.fill_px:
    mov     [rsi], eax
    add     rsi, 4
    dec     edi
    jnz     .fill_px
    inc     ecx
    jmp     .fill_row
.draw_logo:
    mov     eax, [rel logo_width]
    mov     edx, [rel logo_height]
    mov     rsi, r12
    shl     rsi, 16
    xor     rdx, rdx
    mov     rax, rsi
    div     dword [rel logo_width]
    mov     r9, rax
    mov     eax, r11d
    shl     rax, 16
    xor     rdx, rdx
    div     dword [rel logo_height]
    cmp     rax, r9
    cmovb   r9, rax
    mov     rax, 0x10000
    cmp     r9, rax
    cmova   r9, rax
    mov     eax, [rel logo_width]
    imul    rax, r9
    shr     rax, 16
    mov     rdi, rax
    mov     eax, [rel logo_height]
    imul    rax, r9
    shr     rax, 16
    mov     rsi, rax
    mov     eax, r12d
    sub     eax, edi
    shr     eax, 1
    push    rax
    mov     eax, r11d
    sub     eax, esi
    shr     eax, 1
    push    rax
    push    r9
    push    rdi
    push    rsi
    xor     ebp, ebp
.logo_row_loop:
    cmp     ebp, r11d
    jge     .done_logo
    mov     eax, ebp
    sub     eax, [rsp + 24]
    test    eax, eax
    js      .next_logo_row
    cmp     eax, [rsp]
    jge     .next_logo_row
    shl     rax, 16
    xor     rdx, rdx
    div     qword [rsp + 16]
    mov     eax, eax
    mov     edx, [rel logo_width]
    imul    eax, edx
    lea     rsi, [rel logo_data]
    lea     rsi, [rsi + rax*4]
    mov     eax, ebp
    imul    rax, r10
    lea     rdi, [rbx + rax*4]
    xor     ecx, ecx
.logo_pixel_loop:
    cmp     ecx, r12d
    jge     .next_logo_row
    mov     eax, ecx
    sub     eax, [rsp + 32]
    test    eax, eax
    js      .advance_px
    cmp     eax, [rsp + 8]
    jge     .advance_px
    shl     rax, 16
    xor     rdx, rdx
    div     qword [rsp + 16]
    push    rbx
    push    rbp
    mov     edx, eax
    shl     rdx, 2
    mov     edx, [rsi + rdx]
    mov     eax, edx
    shr     eax, 24
    test    eax, eax
    jz      .skip_store
    mov     ebx, edx
    and     ebx, 0xFF
    mov     eax, edx
    shr     eax, 8
    and     eax, 0xFF
    mov     ebp, edx
    shr     ebp, 16
    and     ebp, 0xFF
    cmp     r8d, 0
    jne     .store_rgb
    shl     ebx, 16
    shl     eax, 8
    or      ebx, eax
    or      ebx, ebp
    mov     [rdi + rcx*4], ebx
    jmp     .skip_store
.store_rgb:
    shl     ebp, 16
    shl     eax, 8
    or      ebp, eax
    or      ebp, ebx
    mov     [rdi + rcx*4], ebp
.skip_store:
    pop     rbp
    pop     rbx
.advance_px:
    inc     ecx
    jmp     .logo_pixel_loop
.next_logo_row:
    inc     ebp
    jmp     .logo_row_loop
.done_logo:
    add     rsp, 40
.skip_fill:
    mov     rsp, r14
    and     rsp, -16
    mov     rdi, r13
    xor     rbp, rbp
    cli
    jmp     r15