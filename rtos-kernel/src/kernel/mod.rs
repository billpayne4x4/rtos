//! Kernel top-level module: initialization, basic services, and state.

pub mod init;
pub mod interrupts;
pub mod memory;
pub mod state;
pub mod fb_check;

// Re-exports for ergonomic access.
pub use init::*;
pub use state::*;
pub use memory::*;
pub use fb_check::*;

/// Common imports for kernel code (optional).
pub mod prelude {
    pub use super::init::kernel_init;
    pub use super::state::KernelState;
    pub use rtos_types::{BootInfo, FramebufferInfo, FramebufferFormat};
    pub use crate::framebuffer::Framebuffer;
}
