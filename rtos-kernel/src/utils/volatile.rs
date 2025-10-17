#![no_std]

#[inline(always)]
pub unsafe fn mmio_store32(ptr: *mut u32, val: u32) {
    core::arch::asm!(
    "mov [rdi], eax",
    in("rdi") ptr,
    in("eax") val,
    options(nostack, preserves_flags)
    );
}

#[inline(always)]
pub unsafe fn mmio_load32(ptr: *const u32) -> u32 {
    let out: u32;
    core::arch::asm!(
    "mov eax, [rdi]",
    in("rdi") ptr,
    out("eax") out,
    options(nostack, preserves_flags)
    );
    out
}
