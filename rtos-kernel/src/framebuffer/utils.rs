#![allow(dead_code)]

use rtos_types::FramebufferFormat;

#[inline]
pub fn pack_rgb_fmt(fmt: FramebufferFormat, r: u8, g: u8, b: u8) -> u32 {
    match fmt {
        FramebufferFormat::Rgb => (r as u32) | ((g as u32) << 8) | ((b as u32) << 16),
        FramebufferFormat::Bgr => (b as u32) | ((g as u32) << 8) | ((r as u32) << 16),
        _ => 0,
    }
}

#[inline]
pub fn unpack_rgb_fmt(fmt: FramebufferFormat, px: u32) -> (u8, u8, u8) {
    match fmt {
        FramebufferFormat::Rgb => (
            (px & 0xFF) as u8,
            ((px >> 8) & 0xFF) as u8,
            ((px >> 16) & 0xFF) as u8,
        ),
        _ => (
            ((px >> 16) & 0xFF) as u8,
            ((px >> 8) & 0xFF) as u8,
            (px & 0xFF) as u8,
        ),
    }
}
