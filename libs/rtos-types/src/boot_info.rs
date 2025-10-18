use crate::framebuffer_info::FramebufferInfo;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct BootInfo {
    pub framebuffer: FramebufferInfo,
}

impl BootInfo {
    /// Creates an empty BootInfo with a zeroed framebuffer.
    #[inline]
    pub const fn empty() -> Self {
        BootInfo {
            framebuffer: FramebufferInfo::empty(),
        }
    }

    /// Creates a BootInfo from an existing FramebufferInfo.
    #[inline]
    pub const fn from_framebuffer(info: FramebufferInfo) -> Self {
        BootInfo {
            framebuffer: info,
        }
    }

    /// Returns true if the BootInfo contains a valid framebuffer (nonzero base & size).
    #[inline]
    pub const fn has_framebuffer(&self) -> bool {
        self.framebuffer.base != 0 && self.framebuffer.size > 0
    }

    /// Returns the framebuffer info.
    #[inline]
    pub const fn framebuffer(&self) -> &FramebufferInfo {
        &self.framebuffer
    }
}

impl Default for BootInfo {
    fn default() -> Self {
        Self::empty()
    }
}
