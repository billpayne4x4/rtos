use rtos_framebuffer::framebuffer::{Framebuffer, info::FramebufferInfo};
use uefi::Status;

#[repr(C)]
pub struct BootInfo {
    pub framebuffer: Framebuffer,
    pub phys_mem_offset: u64,
    pub usable_frame_ranges_ptr: u64,
    pub usable_frame_ranges_len: u64
}

impl BootInfo {
    /// Creates an empty BootInfo with a zeroed framebuffer (not usable for drawing).
    pub const fn empty() -> Self {
        BootInfo {
            framebuffer: Framebuffer {
                info: FramebufferInfo::empty(),
                status: Status::NOT_FOUND,
            },
            phys_mem_offset: 0,
            usable_frame_ranges_ptr: 0,
            usable_frame_ranges_len: 0
        }
    }

    /// Creates a BootInfo from an existing Framebuffer.
    pub const fn from_framebuffer(fb: Framebuffer) -> Self {
        BootInfo {
            framebuffer: fb,
            phys_mem_offset: 0,
            usable_frame_ranges_ptr: 0,
            usable_frame_ranges_len: 0
        }
    }

    /// Returns true if the BootInfo contains a valid, memory-accessible framebuffer.
    pub const fn has_framebuffer(&self) -> bool {
        let info = &self.framebuffer.info;
        info.base != 0 && info.size > 0 && info.format.is_memory_accessible()
    }

    /// Returns the framebuffer.
    pub const fn framebuffer(&self) -> &Framebuffer {
        &self.framebuffer
    }

    /// (Optional) Returns just the framebuffer info, if you still want this view.
    pub const fn framebuffer_info(&self) -> &FramebufferInfo {
        &self.framebuffer.info
    }
}

impl Default for BootInfo {
    fn default() -> Self {
        Self::empty()
    }
}
