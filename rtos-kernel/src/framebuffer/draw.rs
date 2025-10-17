use core::ptr;
use super::Framebuffer;
use super::utils::pack_rgb_fmt;
use crate::serial_log;
use crate::utils::SerialWriter;

impl Framebuffer {
    #[inline]
    fn bytes_per_pixel(&self) -> usize { 4 }

    #[inline]
    fn row_ptr(&self, y: u32) -> *mut u32 {
        // Advance by (y * stride * bpp) bytes from the base pointer.
        let bpp = self.bytes_per_pixel();
        let stride_bytes = (self.stride as usize) * bpp;
        unsafe { (self.ptr as *mut u8).add((y as usize) * stride_bytes) as *mut u32 }
    }

    /// Solid clear (no alpha). Uses volatile writes and respects stride.
    pub fn clear(&mut self, r: u8, g: u8, b: u8) {
        let fill   = pack_rgb_fmt(self.format, r, g, b);
        let width  = self.width  as usize;
        let height = self.height as usize;
        let stride = self.stride as usize;
        serial_log!("Width: ", width);
        serial_log!("Height: ", height);
        serial_log!("Stride: ", stride);

        SerialWriter::write("K: Testing direct write with asm...\n");
        unsafe {
            core::arch::asm!(
            "mov [rdi], eax",
            in("rdi") self.ptr as *mut u32,
            in("eax") fill,
            options(nostack)
            );
        }
        SerialWriter::write("K: Direct asm write succeeded!\n");

        let draw_w = if width < stride { width } else { stride };
        serial_log!("DrawW: ", draw_w);

        let mut y = 0;
        while y < height {
            if (y & 0x3F) == 0 { serial_log!("Row: ", y); }
            let mut px = self.row_ptr(y as u32);

            let mut x = 0;
            while x < width {
                unsafe {
                    core::arch::asm!(
                    "mov [rdi], eax",
                    in("rdi") px,
                    in("eax") fill,
                    options(nostack)
                    );
                    px = px.add(1);
                }
                x += 1;
            }
            y += 1;
        }
        serial_log!("Clear done.");
    }

    /// Put a single pixel; volatile write.
    #[inline]
    pub fn put_pixel(&mut self, x: u32, y: u32, color: u32) {
        if x >= self.width || y >= self.height { return; }
        let p = unsafe { self.row_ptr(y).add(x as usize) };
        unsafe { ptr::write_volatile(p, color) };
    }

    /// Fill a rectangle (clipped to framebuffer); volatile writes; respects stride.
    pub fn fill_rect(&mut self, x: u32, y: u32, w: u32, h: u32, color: u32) {
        if w == 0 || h == 0 { return; }
        let x1 = x.min(self.width);
        let y1 = y.min(self.height);
        let x2 = (x.saturating_add(w)).min(self.width);
        let y2 = (y.saturating_add(h)).min(self.height);
        if x1 >= x2 || y1 >= y2 { return; }

        let span = (x2 - x1) as usize;

        for yy in y1..y2 {
            let mut px = unsafe { self.row_ptr(yy).add(x1 as usize) };
            for _ in 0..span {
                unsafe { ptr::write_volatile(px, color) };
                unsafe { px = px.add(1) };
            }
        }
    }

    /// Blit RGBA centered, no scaling. Alpha path delegated to blend.rs.
    pub fn blit_rgba_centered_noscale(&mut self, rgba: &[u8], w: u32, h: u32, use_alpha: bool) {
        if use_alpha {
            // Implemented in blend.rs
            self.blit_rgba_centered_alpha(rgba, w, h);
            return;
        }

        if w == 0 || h == 0 { return; }

        let fb_w = self.width;
        let fb_h = self.height;

        let draw_w = core::cmp::min(fb_w, w);
        let draw_h = core::cmp::min(fb_h, h);
        let off_x  = (fb_w - draw_w) / 2;
        let off_y  = (fb_h - draw_h) / 2;

        let fmt = self.format;

        for y in 0..draw_h {
            // Source row (RGBA: 4 bytes per pixel)
            let src_row = (y * w) as usize;

            // Destination pointer at start of this row, offset by off_x
            let mut dst = unsafe { self.row_ptr(off_y + y).add(off_x as usize) };

            // Walk visible pixels only (draw_w)
            for x in 0..draw_w {
                let s = (src_row + x as usize) * 4;
                if s + 3 >= rgba.len() { break; } // guard against short buffers

                // RGBA (bytes R,G,B,A in memory)
                let r = rgba[s + 0];
                let g = rgba[s + 1];
                let b = rgba[s + 2];
                // let a = rgba[s + 3]; // ignored in non-alpha path

                let packed = pack_rgb_fmt(fmt, r, g, b);
                unsafe {
                    ptr::write_volatile(dst, packed);
                    dst = dst.add(1);
                }
            }
        }
    }
}
