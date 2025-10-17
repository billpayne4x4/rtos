BITS 64
default rel

section .rodata
; Background color = #002D61 (R=0x00, G=0x2D, B=0x61)
color_rgb:  dd 0x00612D00
color_bgr:  dd 0x00002D61

; Logo bitmap data (RGBA 1024x1024, 4 bytes per pixel)
; Generated with:
;   convert images/rtos-logo-transparent.png -alpha on -depth 8 -resize 1024x1024 -gravity center -extent 1024x1024 RGBA:images/rtos-logo-transparent.raw
logo_data:
    incbin "../images/rtos-logo-transparent.raw"
logo_width:  dd 1024
logo_height: dd 1024

section .text
global jump_to_kernel

; RCX = entry, RDX = stack_top, R8 = BootInfo*
; BootInfo.framebuffer (packed):
;   +0x00 base   u64
;   +0x08 size   u64
;   +0x10 width  u32
;   +0x14 height u32
;   +0x18 stride u32   ; pixels per scanline
;   +0x1C format u32   ; 0=BGR, 1=RGB, 2=BltOnly

jump_to_kernel:
    ; Preserve arguments
    mov     r15, rcx            ; kernel entry
    mov     r14, rdx            ; stack top
    mov     r13, r8             ; &BootInfo

    ; Framebuffer info
    mov     rbx, [r13 + 0x00]   ; base
    mov     r12d, [r13 + 0x10]  ; width
    mov     r11d, [r13 + 0x14]  ; height
    mov     r10d, [r13 + 0x18]  ; stride (pixels)
    mov     eax,  [r13 + 0x1C]  ; format

    ; Choose background color and remember framebuffer order in r8d
    cmp     eax, 0
    je      jump_to_kernel.use_bgr
    cmp     eax, 1
    je      jump_to_kernel.use_rgb
    jmp     jump_to_kernel.skip_fill

jump_to_kernel.use_bgr:
    mov     eax, dword [rel color_bgr]
    mov     r8d, 0              ; BGR framebuffer
    jmp     jump_to_kernel.fill_screen

jump_to_kernel.use_rgb:
    mov     eax, dword [rel color_rgb]
    mov     r8d, 1              ; RGB framebuffer

jump_to_kernel.fill_screen:
    ; Clear framebuffer to background color
    xor     ecx, ecx
.fill_row:
    cmp     ecx, r11d
    jae     jump_to_kernel.draw_logo

    mov     edx, ecx
    imul    rdx, r10            ; y * stride (pixels)
    lea     rsi, [rbx + rdx*4]  ; 4 bytes/pixel

    mov     edi, r12d
.fill_px:
    mov     [rsi], eax
    add     rsi, 4
    dec     edi
    jnz     .fill_px

    inc     ecx
    jmp     .fill_row

jump_to_kernel.draw_logo:
    ; Render centered (no scaling), using alpha as a mask to paint white
    call    jump_to_kernel.render_logo_mask_white
    jmp     jump_to_kernel.skip_fill

; -----------------------------------------------------------------------------
; void render_logo_mask_white(void)
; Centered blit of logo_data as alpha mask; paints solid white where A != 0.
; Uses: rbx(base), r12d(width), r11d(height), r10d(stride in pixels), r8d(format)
; Clobbers: rax, rcx, rdx, rsi, rdi, r9d, r10d, r11d, r12d, ebp
; -----------------------------------------------------------------------------
jump_to_kernel.render_logo_mask_white:
    ; draw_w = min(screen_w, logo_w) -> r9d
    mov     r9d, dword [rel logo_width]
    mov     eax, r12d
    cmp     eax, r9d
    cmova   eax, r9d
    mov     r9d, eax

    ; draw_h = min(screen_h, logo_h) -> eax
    mov     eax, r11d
    cmp     eax, dword [rel logo_height]
    cmova   eax, dword [rel logo_height]

    ; Reserve stack for: dst_off_x [0], dst_off_y [4], draw_w [8], draw_h [12]
    sub     rsp, 16

    ; dst_off_x = (screen_w - draw_w)/2
    mov     edx, r12d
    sub     edx, r9d
    shr     edx, 1
    mov     dword [rsp + 0], edx

    ; dst_off_y = (screen_h - draw_h)/2
    mov     edx, r11d
    sub     edx, eax
    shr     edx, 1
    mov     dword [rsp + 4], edx

    mov     dword [rsp + 8], r9d  ; draw_w
    mov     dword [rsp +12], eax  ; draw_h

    xor     ecx, ecx              ; y = 0
.rl_row:
    cmp     ecx, dword [rsp +12]
    jae     .rl_done

    ; src_row = logo_data + (y * logo_width) * 4
    mov     edx, ecx
    imul    edx, dword [rel logo_width]
    lea     rsi, [rel logo_data + rdx*4]

    ; dst_row = base + ((dst_off_y + y) * stride + dst_off_x) * 4
    mov     edx, dword [rsp + 4]
    add     edx, ecx
    imul    edx, r10d
    add     edx, dword [rsp + 0]
    lea     rdi, [rbx + rdx*4]

    xor     edx, edx              ; x = 0
.rl_px:
    cmp     edx, dword [rsp + 8]
    jae     .next_row

    ; Load alpha (RGBA → A in high byte)
    mov     eax, [rsi + rdx*4]
    shr     eax, 24
    test    eax, eax
    jz      .advance               ; skip fully transparent

    ; Paint solid white (ignore src RGB), pack to fb order
    cmp     r8d, 0
    jne     .store_rgb

    ; BGR framebuffer: (R<<16)|(G<<8)|B, white = 0x00FFFFFF
    mov     dword [rdi + rdx*4], 0x00FFFFFF
    jmp     .advance

.store_rgb:
    ; RGB framebuffer: R|(G<<8)|(B<<16), white = 0x00FFFFFF
    mov     dword [rdi + rdx*4], 0x00FFFFFF

.advance:
    inc     edx
    jmp     .rl_px

.next_row:
    inc     ecx
    jmp     .rl_row

.rl_done:
    add     rsp, 16
    ret

; -----------------------------------------------------------------------------
; Final jump to kernel
; -----------------------------------------------------------------------------
jump_to_kernel.skip_fill:
    mov     rsp, r14
    and     rsp, -16
    mov     rdi, r13
    xor     rbp, rbp
    cli
    jmp     r15
