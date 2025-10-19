use crate::framebuffer::Framebuffer;
use crate::framebuffer::format::FramebufferFormat;

impl Framebuffer {
    pub unsafe fn render_image(
        &self,
        src_bytes: &[u8],
        src_w: usize,
        src_h: usize,
        src_format: FramebufferFormat,
        dst_x: usize,
        dst_y: usize,
    ) {
        let info = &self.info;
        if !info.format.is_memory_accessible()
            || info.width == 0 || info.height == 0
            || src_w == 0 || src_h == 0
        { return; }

        let dst_fmt = info.format;
        let dst_bpp = dst_fmt.bytes_per_pixel();
        let src_bpp = src_format.bytes_per_pixel();
        if dst_bpp != 4 || (src_bpp != 3 && src_bpp != 4) { return; }

        let pitch_bytes = (info.stride as usize) * dst_bpp;
        let fb = core::slice::from_raw_parts_mut(info.base as *mut u8, info.size);

        let (sr_off, sg_off, sb_off, sa_off) = src_format.component_offsets();
        let (dr_off, dg_off, db_off, _) = dst_fmt.component_offsets();

        let max_x = core::cmp::min(dst_x.saturating_add(src_w), info.width as usize);
        let max_y = core::cmp::min(dst_y.saturating_add(src_h), info.height as usize);
        let draw_w = max_x.saturating_sub(dst_x);
        let draw_h = max_y.saturating_sub(dst_y);
        if draw_w == 0 || draw_h == 0 { return; }

        // Fast identical-layout, no-alpha path
        let row_copy_ok =
            sa_off.is_none() &&
                ((src_format == FramebufferFormat::Rgb  && dst_fmt == FramebufferFormat::Rgb ) ||
                    (src_format == FramebufferFormat::Bgr  && dst_fmt == FramebufferFormat::Bgr ) ||
                    (src_format == FramebufferFormat::Rgba && dst_fmt == FramebufferFormat::Rgba) ||
                    (src_format == FramebufferFormat::Bgra && dst_fmt == FramebufferFormat::Bgra));

        if row_copy_ok {
            for row in 0..draw_h {
                let srow = row * src_w * src_bpp;
                let drow = (dst_y + row) * pitch_bytes;
                let src_line = &src_bytes[srow .. srow + draw_w * src_bpp];
                let dst_line = &mut fb[drow + dst_x * dst_bpp .. drow + (dst_x + draw_w) * dst_bpp];
                dst_line.copy_from_slice(src_line);
            }
            return;
        }

        // ---------- Preferred: allocate a temp offscreen buffer just for this call ----------
        #[cfg(feature = "alloc")]
        {
            extern crate alloc;
            use alloc::vec::Vec;

            let total_px = draw_w * draw_h;
            // Try to allocate without panicking on OOM.
            let mut scratch: Vec<u32> = Vec::new();
            if scratch.try_reserve_exact(total_px).is_ok() {
                unsafe { scratch.set_len(total_px); } // we’ll fill every entry below

                // Compose whole rect offscreen
                for row in 0..draw_h {
                    let srow = row * src_w * src_bpp;
                    let drow_fb = (dst_y + row) * pitch_bytes;
                    let out_row = &mut scratch[row * draw_w .. row * draw_w + draw_w];

                    let mut x = 0usize;
                    while x < draw_w {
                        let s = srow + x * src_bpp;
                        let d = drow_fb + (dst_x + x) * dst_bpp;

                        let sr = src_bytes[s + sr_off] as u32;
                        let sg = src_bytes[s + sg_off] as u32;
                        let sb = src_bytes[s + sb_off] as u32;
                        let sa = sa_off.map(|o| src_bytes[s + o] as u32).unwrap_or(255);

                        let (r_out, g_out, b_out) = if sa == 0 {
                            let dr0 = fb[d + dr_off] as u32;
                            let dg0 = fb[d + dg_off] as u32;
                            let db0 = fb[d + db_off] as u32;
                            (dr0, dg0, db0)
                        } else if sa == 255 {
                            (sr, sg, sb)
                        } else {
                            let inv = 255 - sa;
                            let dr0 = fb[d + dr_off] as u32;
                            let dg0 = fb[d + dg_off] as u32;
                            let db0 = fb[d + db_off] as u32;
                            (
                                (sr * sa + dr0 * inv + 127) >> 8,
                                (sg * sa + dg0 * inv + 127) >> 8,
                                (sb * sa + db0 * inv + 127) >> 8
                            )
                        };

                        out_row[x] = match dst_fmt {
                            FramebufferFormat::Rgb | FramebufferFormat::Rgba =>
                                u32::from_le_bytes([r_out as u8, g_out as u8, b_out as u8, 0xFF]),
                            FramebufferFormat::Bgr | FramebufferFormat::Bgra =>
                                u32::from_le_bytes([b_out as u8, g_out as u8, r_out as u8, 0xFF]),
                            FramebufferFormat::BltOnly => 0,
                        };

                        x += 1;
                    }
                }

                // Burst copy to LFB (single big copy when tight)
                let tight = dst_x == 0
                    && draw_w == info.width as usize
                    && pitch_bytes == (info.width as usize) * 4;

                if tight {
                    core::ptr::copy_nonoverlapping(
                        scratch.as_ptr() as *const u8,
                        fb.as_mut_ptr().add(dst_y * pitch_bytes),
                        draw_h * draw_w * 4,
                    );
                } else {
                    for row in 0..draw_h {
                        let drow = (dst_y + row) * pitch_bytes;
                        let src_line = &scratch[row * draw_w .. row * draw_w + draw_w];
                        core::ptr::copy_nonoverlapping(
                            src_line.as_ptr() as *const u8,
                            fb.as_mut_ptr().add(drow + dst_x * dst_bpp),
                            draw_w * 4,
                        );
                    }
                }

                // `scratch` drops here → memory is freed now.
                return;
            }
            // else: OOM → fall back below
        }

        // ---------- Fallback (no allocator or OOM): row tile burst path ----------
        const ROW_TILE_PIXELS: usize = 4096;
        let mut row_tile: [u32; ROW_TILE_PIXELS] = [0; ROW_TILE_PIXELS];

        for row in 0..draw_h {
            let srow = row * src_w * src_bpp;
            let drow = (dst_y + row) * pitch_bytes;

            let mut x = 0usize;
            while x < draw_w {
                let chunk = core::cmp::min(ROW_TILE_PIXELS, draw_w - x);

                let mut i = 0usize;
                while i < chunk {
                    let s = srow + (x + i) * src_bpp;
                    let d = drow + (dst_x + x + i) * dst_bpp;

                    let sr = src_bytes[s + sr_off] as u32;
                    let sg = src_bytes[s + sg_off] as u32;
                    let sb = src_bytes[s + sb_off] as u32;
                    let sa = sa_off.map(|o| src_bytes[s + o] as u32).unwrap_or(255);

                    let (r_out, g_out, b_out) = if sa == 0 {
                        let dr0 = fb[d + dr_off] as u32;
                        let dg0 = fb[d + dg_off] as u32;
                        let db0 = fb[d + db_off] as u32;
                        (dr0, dg0, db0)
                    } else if sa == 255 {
                        (sr, sg, sb)
                    } else {
                        let inv = 255 - sa;
                        let dr0 = fb[d + dr_off] as u32;
                        let dg0 = fb[d + dg_off] as u32;
                        let db0 = fb[d + db_off] as u32;
                        (
                            (sr * sa + dr0 * inv + 127) >> 8,
                            (sg * sa + dg0 * inv + 127) >> 8,
                            (sb * sa + db0 * inv + 127) >> 8
                        )
                    };

                    row_tile[i] = match dst_fmt {
                        FramebufferFormat::Rgb | FramebufferFormat::Rgba =>
                            u32::from_le_bytes([r_out as u8, g_out as u8, b_out as u8, 0xFF]),
                        FramebufferFormat::Bgr | FramebufferFormat::Bgra =>
                            u32::from_le_bytes([b_out as u8, g_out as u8, r_out as u8, 0xFF]),
                        FramebufferFormat::BltOnly => 0,
                    };
                    i += 1;
                }

                core::ptr::copy_nonoverlapping(
                    row_tile.as_ptr() as *const u8,
                    fb.as_mut_ptr().add(drow + (dst_x + x) * dst_bpp),
                    chunk * 4,
                );
                x += chunk;
            }
        }
    }

    pub unsafe fn render_scaled_image(
        &self,
        src_bytes: &[u8],
        src_w: usize,
        src_h: usize,
        src_format: FramebufferFormat,
        dst_x: usize,
        dst_y: usize,
        dst_w: usize,
        dst_h: usize,
    ) {
        if src_w == 0 || src_h == 0 || dst_w == 0 || dst_h == 0 { return; }
        let bytes_per_pixel = src_format.bytes_per_pixel();
        if bytes_per_pixel == 0 { return; }

        use crate::graphics::image::scale::scale_raw_nearest_into;

        #[cfg(feature = "alloc")]
        {
            extern crate alloc;
            use alloc::vec::Vec;

            let total_bytes = dst_w.saturating_mul(dst_h).saturating_mul(bytes_per_pixel);
            let mut scaled = Vec::<u8>::new();
            if scaled.try_reserve_exact(total_bytes).is_err() { return; }
            scaled.set_len(total_bytes);

            scale_raw_nearest_into(
                src_bytes,
                src_w,
                src_h,
                src_format,
                dst_w,
                dst_h,
                &mut scaled,
            );

            self.render_image(&scaled, dst_w, dst_h, src_format, dst_x, dst_y);
            return;
        }

        let _ = (src_bytes, src_w, src_h, src_format, dst_x, dst_y, dst_w, dst_h);
        return;
    }


}
