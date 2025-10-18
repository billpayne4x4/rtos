use core::arch::asm;

#[inline(always)] pub fn cli() { unsafe { asm!("cli", options(nomem, nostack, preserves_flags)) } }
#[inline(always)] pub fn sti() { unsafe { asm!("sti", options(nomem, nostack, preserves_flags)) } }
#[inline(always)] pub fn hlt() { unsafe { asm!("hlt", options(nomem, nostack, preserves_flags)) } }
#[inline(always)] pub fn sti_hlt() { unsafe { asm!("sti; hlt", options(nomem, nostack, preserves_flags)) } }
