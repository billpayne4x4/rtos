pub mod mode;

use crate::framebuffer::mode::aspect::AspectRatio;
use crate::framebuffer::mode::pick::pick_highest_for_ratio;

use uefi::{Error, Identify, Status};
use uefi::boot::{self, SearchType, ScopedProtocol};
use uefi::proto::console::gop::{GraphicsOutput, Mode, PixelFormat};
use rtos_types::{framebuffer_info::FramebufferInfo, framebuffer_format::FramebufferFormat};


pub struct Framebuffer {
    info: FramebufferInfo,
    status: Status
}

impl Framebuffer {
    /// Initialize a framebuffer preferring the given aspect ratio, falling back
    /// using `AspectRatio::get_fallback()` if no exact/close match is available.
    pub fn new_from_aspect(aspect_ratio: AspectRatio) -> Result<FramebufferInfo, Status> {
        Self::init_gop_with_aspect(aspect_ratio)
    }

    /// Initialize a framebuffer by setting a specific GOP `Mode`.
    /// (Note: the `Mode` must belong to the GOP we open internally.)
    pub fn new_from_mode(mode: Mode) -> Result<FramebufferInfo, Status> {
        Self::init_gop_with_mode(mode)
    }

    // ---- internals ---------------------------------------------------------

    /// Open the first GraphicsOutput protocol we can find.
    fn open_gop() -> Result<ScopedProtocol<GraphicsOutput>, Status> {
        let handles = boot::locate_handle_buffer(SearchType::ByProtocol(&GraphicsOutput::GUID))
            .map_err(|e: Error| e.status())?;
        let handle = *handles.first().ok_or(Status::NOT_FOUND)?;
        boot::open_protocol_exclusive::<GraphicsOutput>(handle)
            .map_err(|e: Error| e.status())
    }


    /// GOP init path that picks the highest mode for (or near) the desired aspect.
    fn init_gop_with_aspect(mut aspect: AspectRatio) -> Result<FramebufferInfo, Status> {
        let mut gop_handle = Self::open_gop()?;
        let gop: &mut GraphicsOutput = &mut *gop_handle;

        if let Some((mode, _diff)) = pick_highest_for_ratio(gop, aspect) {
            // Best match for the requested ratio; set it.
            gop.set_mode(&mode).map_err(|e: Error| e.status())?;
        } else {
            // No usable mode for that ratio (and fallbacks); keep current mode.
        }

        Self::make_info(gop)
    }

    /// GOP init path that directly sets a given `Mode`.
    fn init_gop_with_mode(mode: Mode) -> Result<FramebufferInfo, Status> {
        let mut gop_handle = Self::open_gop()?;
        let gop: &mut GraphicsOutput = &mut *gop_handle;

        gop.set_mode(&mode).map_err(|e: Error| e.status())?;
        Self::make_info(gop)
    }

    /// Build a `FramebufferInfo` from the current GOP mode.
    fn make_info(gop: &mut GraphicsOutput) -> Result<FramebufferInfo, Status> {
        let info = gop.current_mode_info();
        let (w, h) = info.resolution();
        let mut fb = gop.frame_buffer();

        let fmt = match info.pixel_format() {
            PixelFormat::Bgr => FramebufferFormat::Bgr,
            PixelFormat::Rgb => FramebufferFormat::Rgb,
            _ => FramebufferFormat::BltOnly,
        };

        Ok(FramebufferInfo {
            base:   fb.as_mut_ptr() as u64,
            size:   fb.size(),
            width:  w as u32,
            height: h as u32,
            stride: info.stride() as u32, // pixels per scanline
            format: fmt,
        })
    }
}