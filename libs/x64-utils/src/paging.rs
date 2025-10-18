use core::arch::asm;
#[inline(always)] pub fn invlpg(addr: usize) { unsafe { asm!("invlpg [{}]", in(reg) addr, options(nostack)) } }
