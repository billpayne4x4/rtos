//! Interrupt controller setup. Minimal placeholders; expand with an IDT later.

#[inline]
pub unsafe fn init() {
    // Placeholder for IDT/GDT/timer PIC/APIC setup.
    // Keep empty for now to avoid extra dependencies.
}

#[inline]
pub unsafe fn enable() {
    core::arch::asm!("sti", options(nomem, nostack, preserves_flags));
}

#[inline]
pub unsafe fn disable() {
    core::arch::asm!("cli", options(nomem, nostack, preserves_flags));
}
