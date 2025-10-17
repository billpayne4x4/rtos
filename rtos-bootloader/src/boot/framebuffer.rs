use uefi::boot::{self, SearchType};
use uefi::proto::console::gop::{GraphicsOutput, Mode, PixelFormat};
use uefi::{Error, Identify, Status};

use crate::boot::console::{write_hex, write_line};
use crate::boot::aspectratio::AspectRatio;

#[repr(C)]
pub struct FramebufferInfo {
    pub base: u64,
    pub size: usize,
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub format: FramebufferFormat,
}

#[repr(u32)]
#[derive(Copy, Clone)]
pub enum FramebufferFormat {
    Bgr = 0,
    Rgb = 1,
    BltOnly = 2,
}

// ---- helpers ----

fn ratio_diff(w: u32, h: u32, target: (u32, u32)) -> u64 {
    let (tn, td) = target;
    (w as u64)
        .saturating_mul(td as u64)
        .abs_diff((h as u64).saturating_mul(tn as u64))
}

fn pick_highest_for_ratio(gop: &mut GraphicsOutput, target: (u32, u32)) -> Option<(Mode, u64)> {
    let mut best: Option<(Mode, u64, u64)> = None; // (mode, diff, area)

    for mode in gop.modes() {
        let info = mode.info();
        if info.pixel_format() == PixelFormat::BltOnly {
            continue;
        }

        let (wu, hu) = info.resolution();      // usize, usize
        let w = wu as u32;
        let h = hu as u32;

        let diff = ratio_diff(w, h, target);
        let area = (wu as u64) * (hu as u64);  // keep max precision for area

        match best {
            None => best = Some((mode, diff, area)),
            Some((_, best_diff, best_area)) => {
                if diff < best_diff || (diff == best_diff && area > best_area) {
                    best = Some((mode, diff, area));
                }
            }
        }
    }

    best.map(|(m, d, _)| (m, d))
}

// ---- main ----

pub fn get_gop_framebuffer(aspect_ratio: AspectRatio) -> Result<FramebufferInfo, Status> {
    // Locate a GOP handle
    let handles = boot::locate_handle_buffer(SearchType::ByProtocol(&GraphicsOutput::GUID))
        .map_err(|e: Error| e.status())?;
    let handle = *handles.first().ok_or(Status::NOT_FOUND)?;

    // Open GOP (ScopedProtocol<GraphicsOutput>)
    let mut gop_handle = boot::open_protocol_exclusive::<GraphicsOutput>(handle)
        .map_err(|e: Error| e.status())?;
    let gop: &mut GraphicsOutput = &mut *gop_handle;

    // Log current
    let cur = gop.current_mode_info();
    let (cw, ch) = cur.resolution();
    write_line("BL: GOP current mode");
    write_hex("  width",  cw as u64);
    write_hex("  height", ch as u64);

    // Try requested ratio (or skip if Unspecified)
    if let Some(target) = aspect_ratio.as_tuple() {
        if let Some((mode, diff)) = pick_highest_for_ratio(gop, target) {
            let info = mode.info();
            let (w, h) = info.resolution();
            write_line("BL: choosing aspect-preferred mode");
            write_hex("  target_w", target.0 as u64);
            write_hex("  target_h", target.1 as u64);
            write_hex("  chosen_w", w as u64);
            write_hex("  chosen_h", h as u64);
            write_hex("  diff(0=exact)", diff);
            gop.set_mode(&mode).map_err(|e: Error| e.status())?;
        } else {
            write_line("BL: no GOP modes matched/near target; keeping current mode");
        }
    } else {
        write_line("BL: aspect ratio Unspecified; keeping current mode");
    }

    // Final framebuffer info
    let info = gop.current_mode_info();
    let (wusize, husize) = info.resolution();
    let w = wusize as u32;
    let h = husize as u32;

    let mut fb = gop.frame_buffer(); // mutable for as_mut_ptr()
    let fmt = match info.pixel_format() {
        PixelFormat::Bgr => FramebufferFormat::Bgr,
        PixelFormat::Rgb => FramebufferFormat::Rgb,
        _ => FramebufferFormat::BltOnly,
    };

    write_line("BL: GOP final mode");
    write_hex("  width",  w as u64);
    write_hex("  height", h as u64);
    write_hex("  stride", info.stride() as u64);

    Ok(FramebufferInfo {
        base: fb.as_mut_ptr() as u64,
        size: fb.size(),
        width: w,
        height: h,
        stride: info.stride() as u32,
        format: fmt,
    })
}
