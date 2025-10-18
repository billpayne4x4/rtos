use crate::constants::RTOSK_EXEC_FLAG;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct RtoskSegment {
    pub file_offset: u64,
    pub memory_addr: u64,
    pub memory_size: u64,
    pub file_size: u64,
    pub flags: u32,
}

impl RtoskSegment {
    /// Creates an empty (zeroed) segment descriptor.
    #[inline]
    pub const fn empty() -> Self {
        RtoskSegment {
            file_offset: 0,
            memory_addr: 0,
            memory_size: 0,
            file_size: 0,
            flags: 0,
        }
    }

    /// Constructs a new segment descriptor.
    #[inline]
    pub const fn new(file_offset: u64, memory_addr: u64, memory_size: u64, file_size: u64, flags: u32) -> Self {
        RtoskSegment {
            file_offset,
            memory_addr,
            memory_size,
            file_size,
            flags,
        }
    }

    /// Returns true if the segment has nonzero memory and file size.
    #[inline]
    pub const fn is_loadable(&self) -> bool {
        self.memory_size > 0 && self.file_size > 0
    }

    /// Returns true if the segment is marked executable.
    #[inline]
    pub const fn is_executable(&self) -> bool {
        (self.flags & RTOSK_EXEC_FLAG) != 0
    }
}

impl Default for RtoskSegment {
    fn default() -> Self {
        Self::empty()
    }
}
