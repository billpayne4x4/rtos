#![no_std]

#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum FramebufferFormat {
    Bgr = 0,
    Rgb = 1,
    BltOnly = 2,
}

// --------- POD structs passed across the ABI ---------

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

// --------- Compile-time ABI guards (x86_64) ---------

const _: () = {
    use core::mem::{align_of, size_of};

    // Expect 32-byte FramebufferInfo, 8-byte alignment on x86_64
    let _ = [(); 32 - size_of::<FramebufferInfo>()]; // error if not 32
    let _ = [(); 8  - align_of::<FramebufferInfo>()]; // error if not 8

    // BootInfo is just the wrapper with one field; should also be 32/8
    let _ = [(); 32 - size_of::<BootInfo>()];
    let _ = [(); 8  - align_of::<BootInfo>()];
};