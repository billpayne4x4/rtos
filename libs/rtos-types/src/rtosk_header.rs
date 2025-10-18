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

impl RtoskHeader {
    /// Creates a zeroed / invalid header.
    #[inline]
    pub const fn empty() -> Self {
        RtoskHeader {
            magic: [0; 5],
            ver_major: 0,
            ver_minor: 0,
            header_len: 0,
            entry64: 0,
            page_size: 0,
            seg_count: 0,
            image_crc32: 0,
            flags: 0,
        }
    }

    /// Constructs a new header with basic version info and entry point.
    #[inline]
    pub const fn new(entry64: u64, page_size: u32, seg_count: u32, image_crc32: u32, flags: u32) -> Self {
        RtoskHeader {
            magic: *b"RTOSK",
            ver_major: 1,
            ver_minor: 0,
            header_len: core::mem::size_of::<RtoskHeader>() as u32,
            entry64,
            page_size,
            seg_count,
            image_crc32,
            flags,
        }
    }

    /// Checks whether the magic field matches "RTOSK".
    #[inline]
    pub const fn is_valid(&self) -> bool {
        let m = self.magic;
        m[0] == b'R' && m[1] == b'T' && m[2] == b'O' && m[3] == b'S' && m[4] == b'K'
    }

    /// Returns the full version as a (major, minor) tuple.
    #[inline]
    pub const fn version(&self) -> (u16, u16) {
        (self.ver_major, self.ver_minor)
    }
}

impl Default for RtoskHeader {
    fn default() -> Self {
        Self::empty()
    }
}
