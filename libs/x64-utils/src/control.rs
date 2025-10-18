use core::arch::asm;

#[inline(always)] pub fn read_cr0() -> u64 { let v; unsafe{ asm!("mov {}, cr0", out(reg) v) } v }
#[inline(always)] pub fn write_cr0(v: u64) { unsafe { asm!("mov cr0, {}", in(reg) v) } }

#[inline(always)] pub fn read_cr3() -> u64 { let v; unsafe{ asm!("mov {}, cr3", out(reg) v) } v }
#[inline(always)] pub fn write_cr3(v: u64) { unsafe { asm!("mov cr3, {}", in(reg) v) } }

#[inline(always)] pub fn read_cr4() -> u64 { let v; unsafe{ asm!("mov {}, cr4", out(reg) v) } v }
#[inline(always)] pub fn write_cr4(v: u64) { unsafe { asm!("mov cr4, {}", in(reg) v) } }
