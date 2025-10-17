//! Early memory and paging hooks. These are safe no-ops for now.

#[inline]
pub fn init() {
    // Add page table setup, heap bring-up, frame allocator, etc.
    // Left intentionally empty so the kernel compiles/run with no extra deps.
}

/// Compiler fence for ordering when poking MMIO/FB Maybe?? I dunno :)
#[inline]
pub fn fence() {
    core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
}
