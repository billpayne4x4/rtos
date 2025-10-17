use rtos_types::FramebufferFormat;
use super::{Framebuffer};
use super::utils::{pack_rgb_fmt, unpack_rgb_fmt};

#[inline]
pub fn blend_over(dst: u32, fmt: FramebufferFormat, r: u8, g: u8, b: u8, a: u8) -> u32 {
    if a == 0 { return dst; }
    if a == 255 { return pack_rgb_fmt(fmt, r, g, b); }

    let (dr, dg, db) = unpack_rgb_fmt(fmt, dst);
    let (sr, sg, sb) = (r as u32, g as u32, b as u32);
    let (dr, dg, db) = (dr as u32, dg as u32, db as u32);
    let sa = a as u32;
    let inv = 255 - sa;

    // integer round-nearest via *257 >> 16 trick
    let or = (((sr * sa + dr * inv + 127) * 257) >> 16) as u8;
    let og = (((sg * sa + dg * inv + 127) * 257) >> 16) as u8;
    let ob = (((sb * sa + db * inv + 127) * 257) >> 16) as u8;

    pack_rgb_fmt(fmt, or, og, ob)
}

impl Framebuffer {
    /// Blit RGBA image centered with alpha blending.
    pub fn blit_rgba_centered_alpha(&mut self, rgba: &[u8], w: u32, h: u32) {
        if w == 0 || h == 0 { return; }

        let fmt  = self.format;
        let fb_w = self.width;
        let fb_h = self.height;
        let stride = self.stride as usize;

        let draw_w = core::cmp::min(fb_w, w);
        let draw_h = core::cmp::min(fb_h, h);
        let off_x = (fb_w - draw_w) / 2;
        let off_y = (fb_h - draw_h) / 2;

        for y in 0..draw_h {
            let src_row = (y * w) as usize;
            let dst_y = off_y + y;

            // Take the row mutably; avoid calling &self methods while held.
            let row = unsafe {
                let off = (dst_y as usize) * stride;
                core::slice::from_raw_parts_mut(self.ptr.add(off), stride)
            };

            for x in 0..draw_w {
                let s = (src_row + x as usize) * 4;
                let r = rgba[s + 0];
                let g = rgba[s + 1];
                let b = rgba[s + 2];
                let a = rgba[s + 3];

                let idx = (off_x + x) as usize;
                let dst = row[idx];
                row[idx] = blend_over(dst, fmt, r, g, b, a);
            }
        }
    }
}
