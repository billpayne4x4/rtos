pub mod mode;
pub mod format;
pub mod info;

use crate::framebuffer::mode::{aspect::AspectRatio, pick::pick_highest_for_ratio};
use crate::framebuffer::info::FramebufferInfo;
use crate::framebuffer::format::FramebufferFormat;
use uefi::{Error, Identify, Status};
use uefi::boot::{self, SearchType, ScopedProtocol};
use uefi::proto::console::gop::{GraphicsOutput, Mode, PixelFormat};

/// Represents an initialized UEFI framebuffer with metadata and active status.
pub struct Framebuffer {
    /// Static framebuffer metadata: dimensions, stride, format, base address, etc.
    pub info: FramebufferInfo,
    /// Status result of the framebuffer initialization (e.g., `Status::SUCCESS`).
    pub status: Status,
}

impl Framebuffer {
    /// Initialize and set a framebuffer mode matching the given [`AspectRatio`].
    ///
    /// Attempts to pick the **highest resolution** mode for the given ratio,
    /// falling back using [`AspectRatio::get_fallback()`] if no exact match exists.
    ///
    /// Returns a fully initialized [`Framebuffer`] that can be used
    /// for direct pixel operations (e.g., clear, blit, render).
    pub fn new_from_aspect(aspect_ratio: AspectRatio) -> Result<Framebuffer, Status> {
        Self::init_gop_with_aspect(aspect_ratio)
    }

    /// Initialize the framebuffer by directly setting a specific GOP [`Mode`].
    ///
    /// This is used when you already have a `Mode` handle (e.g., from a previous enumeration).
    /// Returns a ready-to-use [`Framebuffer`] configured with that mode.
    pub fn new_from_mode(mode: Mode) -> Result<Framebuffer, Status> {
        Self::init_gop_with_mode(mode)
    }

    // ---- internals ---------------------------------------------------------

    /// Opens the first available UEFI Graphics Output Protocol (GOP).
    ///
    /// Returns a [`ScopedProtocol<GraphicsOutput>`] handle for direct framebuffer access.
    fn open_gop() -> Result<ScopedProtocol<GraphicsOutput>, Status> {
        let handles = boot::locate_handle_buffer(SearchType::ByProtocol(&GraphicsOutput::GUID))
            .map_err(|e: Error| e.status())?;
        let handle = *handles.first().ok_or(Status::NOT_FOUND)?;
        boot::open_protocol_exclusive::<GraphicsOutput>(handle)
            .map_err(|e: Error| e.status())
    }

    /// Internal path for GOP initialization based on an aspect ratio.
    ///
    /// Picks and applies the best available mode matching the target ratio.
    /// Returns an initialized [`Framebuffer`] configured for that mode.
    fn init_gop_with_aspect(aspect_ratio: AspectRatio) -> Result<Framebuffer, Status> {
        let mut gop_handle = Self::open_gop()?;
        let gop: &mut GraphicsOutput = &mut *gop_handle;

        if let Some((mode, _diff)) = pick_highest_for_ratio(gop, aspect_ratio) {
            gop.set_mode(&mode).map_err(|e: Error| e.status())?;
        }

        Self::make_framebuffer(gop)
    }

    /// Internal path for GOP initialization using a specific mode.
    ///
    /// The provided `Mode` must belong to the currently opened GOP.
    fn init_gop_with_mode(mode: Mode) -> Result<Framebuffer, Status> {
        let mut gop_handle = Self::open_gop()?;
        let gop: &mut GraphicsOutput = &mut *gop_handle;

        gop.set_mode(&mode).map_err(|e: Error| e.status())?;
        Self::make_framebuffer(gop)
    }

    /// Constructs a [`Framebuffer`] and [`FramebufferInfo`] from the current GOP mode.
    ///
    /// This reads GOP-provided metadata such as pixel format, stride, and resolution,
    /// and sets up an identity-mapped base pointer for direct pixel access.
    fn make_framebuffer(gop: &mut GraphicsOutput) -> Result<Framebuffer, Status> {
        let info = gop.current_mode_info();
        let (w, h) = info.resolution();
        let mut fb = gop.frame_buffer();

        let fmt = match info.pixel_format() {
            PixelFormat::Bgr => FramebufferFormat::Bgr,
            PixelFormat::Rgb => FramebufferFormat::Rgb,
            _ => FramebufferFormat::BltOnly,
        };

        let fb_info = FramebufferInfo {
            base:   fb.as_mut_ptr() as u64,
            size:   fb.size(),
            width:  w as u32,
            height: h as u32,
            stride: info.stride() as u32, // pixels per scanline
            format: fmt,
        };

        Ok(Framebuffer {
            info: fb_info,
            status: Status::SUCCESS,
        })
    }
}
