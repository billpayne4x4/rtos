use rtos_types::{BootInfo, FramebufferFormat};
use core::slice;

// ===== Submodules =====
pub mod draw;
pub mod blend;
pub mod utils;
mod validate;

// Re-export their public items so external code can use them
pub use draw::*;
pub use blend::*;
pub use utils::*;
pub use validate::*;

// ===== Core framebuffer struct =====
pub struct Framebuffer {
    pub ptr: *mut u32,
    pub width: u32,
    pub height: u32,
    pub stride: u32,               // pixels per scanline
    pub format: FramebufferFormat, // Bgr or Rgb
}

impl Framebuffer {
    pub unsafe fn from_bootinfo(bi: &BootInfo) -> Self {
        Framebuffer {
            ptr: bi.framebuffer.base as *mut u32,
            width: bi.framebuffer.width,
            height: bi.framebuffer.height,
            stride: bi.framebuffer.stride,
            format: bi.framebuffer.format,
        }
    }

    #[inline]
    fn row_mut(&mut self, y: u32) -> &mut [u32] {
        let off = (y as usize) * (self.stride as usize);
        unsafe { slice::from_raw_parts_mut(self.ptr.add(off), self.stride as usize) }
    }

    #[inline]
    pub fn pack_rgb(&self, r: u8, g: u8, b: u8) -> u32 {
        // utils::pack_rgb_fmt already handles RGB vs BGR
        pack_rgb_fmt(self.format, r, g, b)
    }
}