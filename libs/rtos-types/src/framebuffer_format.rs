#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum FramebufferFormat {
    Bgr = 0,
    Rgb = 1,
    BltOnly = 2,
}

impl FramebufferFormat {
    /// Returns true if the format supports direct pixel access.
    #[inline]
    pub const fn is_memory_accessible(&self) -> bool {
        match self {
            FramebufferFormat::Bgr | FramebufferFormat::Rgb => true,
            FramebufferFormat::BltOnly => false,
        }
    }

    /// Creates a format from a raw integer (e.g. from firmware tables).
    #[inline]
    pub const fn from_u32(value: u32) -> Self {
        match value {
            0 => FramebufferFormat::Bgr,
            1 => FramebufferFormat::Rgb,
            _ => FramebufferFormat::BltOnly,
        }
    }

    /// Returns the numeric value for this format.
    #[inline]
    pub const fn as_u32(&self) -> u32 {
        *self as u32
    }

    /// Returns a human-readable name for logging.
    #[inline]
    pub const fn as_str(&self) -> &'static str {
        match self {
            FramebufferFormat::Bgr => "BGR",
            FramebufferFormat::Rgb => "RGB",
            FramebufferFormat::BltOnly => "BLT-only",
        }
    }
}
