use crate::serial_writer::SerialWriter;
use crate::serial_logb;
use x64_utils::handoff;

/// Minimal handoff:
/// - sets RSP = `stack_top` (16-byte aligned inside),
/// - puts BootInfo* in RDI (SysV),
/// - `jmp` to `entry_ptr` (never returns).
#[inline(never)]
pub fn trampoline_jump(entry_ptr: usize, stack_top: usize, boot_info: usize) -> ! {
    serial_logb!("entering to kernel");
    unsafe { handoff::sysv(entry_ptr, stack_top, boot_info) }
}