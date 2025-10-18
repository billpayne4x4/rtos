#![no_std]

#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum FramebufferFormat {
    Bgr = 0,
    Rgb = 1,
    BltOnly = 2,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct FramebufferInfo {
    pub base: u64,      // linear framebuffer base (identity-mapped)
    pub size: usize,    // total bytes
    pub width: u32,     // pixels
    pub height: u32,    // pixels
    pub stride: u32,    // pixels per scanline (NOT bytes)
    pub format: FramebufferFormat,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct BootInfo {
    pub framebuffer: FramebufferInfo,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct RtoskHeader {
    pub magic: [u8; 5],
    pub ver_major: u16,
    pub ver_minor: u16,
    pub header_len: u32,
    pub entry64: u64,
    pub page_size: u32,
    pub seg_count: u32,
    pub image_crc32: u32,
    pub flags: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct RtoskSegment {
    pub file_offset: u64,
    pub memory_addr: u64,
    pub memory_size: u64,
    pub file_size: u64,
    pub flags: u32,
}

// ======================================== Constants ========================================

pub const RTOSK_MAGIC: [u8; 5] = *b"RTOSK";
pub const RTOSK_EXEC_FLAG: u32 = 1 << 0;