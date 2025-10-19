// libs/rtos-framebuffer/src/graphics/image/scale.rs

use crate::framebuffer::format::FramebufferFormat;

/// Nearest-neighbour scale from `src` into caller-provided `out`.
/// - `fmt` describes the source & destination pixel layout (same fmt in/out).
/// - `out` must be at least `dst_w * dst_h * fmt.bytes_per_pixel()` bytes.
/// - Works in `no_std` (no heap).
pub fn scale_raw_nearest_into(
    src: &[u8],
    src_w: usize,
    src_h: usize,
    fmt: FramebufferFormat,
    dst_w: usize,
    dst_h: usize,
    out: &mut [u8],
) {
    let bpp = fmt.bytes_per_pixel();
    if bpp == 0 || src_w == 0 || src_h == 0 || dst_w == 0 || dst_h == 0 {
        return;
    }
    assert!(out.len() >= dst_w * dst_h * bpp);

    let (r_off, g_off, b_off, a_off) = fmt.component_offsets();

    // Fixed-point stepping to avoid float in kernel/boot paths.
    let x_step = ((src_w as u64) << 32) / (dst_w as u64);
    let y_step = ((src_h as u64) << 32) / (dst_h as u64);

    let mut y_acc = 0u64;
    for dy in 0..dst_h {
        let sy = ((y_acc >> 32) as usize).min(src_h - 1);
        let srow = sy * src_w * bpp;
        let drow = dy * dst_w * bpp;

        let mut x_acc = 0u64;
        for dx in 0..dst_w {
            let sx = ((x_acc >> 32) as usize).min(src_w - 1);
            let s = srow + sx * bpp;
            let d = drow + dx * bpp;

            // Copy components per format
            out[d + r_off] = src[s + r_off];
            out[d + g_off] = src[s + g_off];
            out[d + b_off] = src[s + b_off];
            if let Some(a) = a_off {
                out[d + a] = src[s + a];
            } else if bpp == 4 {
                // For XRGB/XBGR-style 32bpp, zero the padding byte deterministically.
                out[d + 3] = 0x00;
            }

            x_acc = x_acc.wrapping_add(x_step);
        }
        y_acc = y_acc.wrapping_add(y_step);
    }
}

#[cfg(feature = "alloc")]
mod with_alloc {
    extern crate alloc;
    use alloc::vec::Vec;

    use crate::framebuffer::format::FramebufferFormat;
    use super::scale_raw_nearest_into;

    /// Heap-backed convenience wrapper. Enable with `features = ["alloc"]`.

    pub fn scale_raw_nearest(
        src: &[u8],
        src_w: usize,
        src_h: usize,
        fmt: FramebufferFormat,
        dst_w: usize,
        dst_h: usize,
    ) -> Vec<u8> {
        let bpp = fmt.bytes_per_pixel();
        if bpp == 0 || src_w == 0 || src_h == 0 || dst_w == 0 || dst_h == 0 {
            return Vec::new();
        }
        let mut out = vec![0u8; dst_w * dst_h * bpp];
        scale_raw_nearest_into(src, src_w, src_h, fmt, dst_w, dst_h, &mut out);
        out
    }

    pub use scale_raw_nearest as _alloc_scale_raw_nearest_export;
}

#[cfg(feature = "alloc")]
pub use with_alloc::_alloc_scale_raw_nearest_export as scale_raw_nearest;
