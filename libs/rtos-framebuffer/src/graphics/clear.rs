use crate::framebuffer::Framebuffer;
use crate::framebuffer::format::FramebufferFormat;

impl Framebuffer {
    /// Clears the framebuffer to a solid RGB color.
    /// Uses a small on-stack scratch buffer and bursts the fill for speed.
    /// Must be called after ExitBootServices when framebuffer memory is valid.
    pub unsafe fn clear_rgb(&self, r: u8, g: u8, b: u8) {
        let info = &self.info;
        if !info.format.is_memory_accessible() || info.format.bytes_per_pixel() != 4 {
            return;
        }

        // Build the pixel (32-bit word)
        let pixel: u32 = match info.format {
            FramebufferFormat::Rgb | FramebufferFormat::Rgba => u32::from_le_bytes([r, g, b, 0xFF]),
            FramebufferFormat::Bgr | FramebufferFormat::Bgra => u32::from_le_bytes([b, g, r, 0xFF]),
            FramebufferFormat::BltOnly => return,
        };

        let total_bytes = (info.stride as usize) * (info.height as usize) * 4;
        let fb_ptr = info.base as *mut u8;

        // Small scratch tile (8 KiB = 2048 pixels)
        const TILE_WORDS: usize = 2048;
        let mut tile: [u32; TILE_WORDS] = [0; TILE_WORDS];
        tile.fill(pixel);

        // Copy in bursts using the small tile
        let tile_bytes = TILE_WORDS * core::mem::size_of::<u32>();
        let mut remaining = total_bytes;
        let mut dst = fb_ptr;

        while remaining > 0 {
            let n = core::cmp::min(remaining, tile_bytes);
            core::ptr::copy_nonoverlapping(tile.as_ptr() as *const u8, dst, n);
            dst = dst.add(n);
            remaining -= n;
        }
    }
}
