use core::arch::asm;

/// Read the current stack pointer (RSP).
#[inline(always)]
pub fn get_pointer() -> usize {
    let mut sp: usize;
    unsafe {
        asm!(
        "mov {sp}, rsp",
        sp = out(reg) sp,
        options(nomem, nostack, preserves_flags),
        );
    }
    sp
}

/// Set the stack pointer (RSP) to `sp` **exactly as given**.
///
/// # Safety
/// - `sp` must point to valid, writable stack memory owned by your code.
/// - After this call, *all* pushes/calls/interrupt frames will use that region.
/// - No alignment is applied; if you’re about to enter a SysV/Win64 function,
///   you are responsible for ensuring 16-byte alignment.
///
/// This function does **not** return a value; it merely updates RSP.
#[inline(always)]
pub unsafe fn set_pointer(sp: usize) {
    asm!(
    "mov rsp, {sp}",
    sp = in(reg) sp,
    options(nostack, preserves_flags),
    );
}

/// Set the stack pointer (RSP) to `sp` **rounded down to a 16-byte boundary**.
/// Returns the *aligned* value actually loaded into RSP (useful for logging).
///
/// # Safety
/// - Same preconditions as [`set_stack_pointer`]—the aligned address must be valid.
/// - 16-byte alignment satisfies the SysV/Win64 ABI requirement at function entry.
#[inline(always)]
pub unsafe fn set_pointer_aligned(sp: usize) -> usize {
    let mut aligned = sp & !0xFu64 as usize;
    asm!(
    "mov rsp, {sp}",
    sp = in(reg) aligned,
    options(nostack, preserves_flags),
    );
    aligned
}