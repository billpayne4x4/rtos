#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum FramebufferFormat {
    /// 3 channels, usually stored in 4 bytes (BGRX)
    Bgr     = 0,
    /// 3 channels, usually stored in 4 bytes (RGBX)
    Rgb     = 1,
    /// Unsupported for direct CPU writes
    BltOnly = 2,
    /// 4 channels, BGRA order
    Bgra    = 3,
    /// 4 channels, RGBA order
    Rgba    = 4,
}

impl FramebufferFormat {
    /// Returns true if this format supports direct pixel access.
    pub const fn is_memory_accessible(self) -> bool {
        matches!(self, Self::Bgr | Self::Rgb | Self::Bgra | Self::Rgba)
    }

    /// Creates a format from a raw integer (e.g. from firmware tables).
    pub const fn from_u32(value: u32) -> Self {
        match value {
            0 => Self::Bgr,
            1 => Self::Rgb,
            2 => Self::BltOnly,
            3 => Self::Bgra,
            4 => Self::Rgba,
            _ => Self::BltOnly,
        }
    }

    /// Returns the numeric value for this format.
    pub const fn as_u32(self) -> u32 {
        self as u32
    }

    /// Returns a human-readable name for logging or debugging.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Bgr     => "BGR",
            Self::Rgb     => "RGB",
            Self::BltOnly => "BLT-only",
            Self::Bgra    => "BGRA",
            Self::Rgba    => "RGBA",
        }
    }

    /// Returns bytes per pixel for this format.
    pub const fn bytes_per_pixel(self) -> usize {
        match self {
            Self::Bgr | Self::Rgb => 4, // typically 32-bit aligned (BGRX/RGBX)
            Self::Bgra | Self::Rgba => 4,
            Self::BltOnly => 0,
        }
    }

    /// Returns (R, G, B, A) byte offsets for this format.
    ///
    /// Example: `Bgr → (2,1,0,None)`, `Rgba → (0,1,2,Some(3))`
    pub const fn component_offsets(self) -> (usize, usize, usize, Option<usize>) {
        match self {
            Self::Rgb  => (0, 1, 2, None),
            Self::Bgr  => (2, 1, 0, None),
            Self::Rgba => (0, 1, 2, Some(3)),
            Self::Bgra => (2, 1, 0, Some(3)),
            Self::BltOnly => (0, 0, 0, None),
        }
    }

    /// Returns true if this format has an alpha channel.
    pub const fn has_alpha(self) -> bool {
        matches!(self, Self::Bgra | Self::Rgba)
    }

    /// Returns the bit depth per channel (usually 8).
    pub const fn bits_per_channel(self) -> usize {
        8
    }
}
