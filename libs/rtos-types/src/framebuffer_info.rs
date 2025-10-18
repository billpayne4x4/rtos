use crate::framebuffer_format::FramebufferFormat;
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct FramebufferInfo {
    pub base: u64,      // linear framebuffer base (identity-mapped)
    pub size: usize,    // total bytes
    pub width: u32,     // pixels
    pub height: u32,    // pixels
    pub stride: u32,    // pixels per scanline (NOT bytes)
    pub format: FramebufferFormat
}

impl FramebufferFormat {
    #[inline]
    pub const fn default() -> Self { FramebufferFormat::BltOnly }
}

impl Default for FramebufferFormat {
    fn default() -> Self { FramebufferFormat::BltOnly }
}

impl FramebufferInfo {
    /// A zeroed/invalid framebuffer descriptor (safe placeholder).
    #[inline]
    pub const fn empty() -> Self {
        FramebufferInfo {
            base:   0,
            size:   0,
            width:  0,
            height: 0,
            stride: 0,
            format: FramebufferFormat::BltOnly,
        }
    }
}

impl Default for FramebufferInfo {
    fn default() -> Self { FramebufferInfo::empty() }
}