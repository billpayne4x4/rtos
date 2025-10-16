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
