use core::arch::asm;

/// Non-returning handoff to a SysV x86_64 kernel entry:
/// - RSP ← stack_top (aligned down to 16 bytes)
/// - RDI ← boot_info (SysV first argument)
/// - JMP entry (no return address pushed)
///
/// # Safety
/// - `entry` must be a valid code address with signature `extern "sysv64" fn(*const T) -> !`.
/// - `stack_top` must point to a valid, writable kernel stack (top of stack).
/// - Interrupts state is your responsibility (usually disable before calling).
#[inline(always)]
pub unsafe fn sysv(entry: usize, stack_top: usize, boot_info: usize) -> ! {
    asm!(
    "mov     rax, {entry}",
    "mov     rsp, {stack}",
    "and     rsp, -16",
    "mov     rdi, {boot}",
    "xor     rbp, rbp",
    "jmp     rax",
    entry = in(reg) entry,
    stack = in(reg) stack_top,
    boot  = in(reg) boot_info,
    options(noreturn),
    )
}

/// Windows x64 variant (RCX = first arg).
#[inline(always)]
pub unsafe fn win64(entry: usize, stack_top: usize, boot_info: usize) -> ! {
    asm!(
    "mov     rax, {entry}",
    "mov     rsp, {stack}",
    "and     rsp, -16",
    "mov     rcx, {boot}",
    "xor     rbp, rbp",
    "jmp     rax",
    entry = in(reg) entry,
    stack = in(reg) stack_top,
    boot  = in(reg) boot_info,
    options(noreturn),
    )
}
