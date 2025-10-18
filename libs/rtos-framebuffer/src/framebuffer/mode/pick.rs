use uefi::proto::console::gop::{GraphicsOutput, Mode, PixelFormat};
use crate::framebuffer::mode::aspect::AspectRatio;

/// Collects all available GOP modes into a static slice.
/// (The Vec is intentionally leaked since the mode list lives for the program lifetime.)
pub fn list_modes<'a>(gop: &'a mut GraphicsOutput) -> impl Iterator<Item = Mode> + 'a {
    gop.modes().map(|m| m)
}

/// Compute how far off a mode's aspect ratio is from the target.
/// Smaller = closer match.
fn ratio_diff(w: u32, h: u32, target: (u32, u32)) -> u64 {
    let (tn, td) = target;
    (w as u64)
        .saturating_mul(td as u64)
        .abs_diff((h as u64).saturating_mul(tn as u64))
}

/// Picks the highest-resolution GOP mode that matches or approximates
/// the given `AspectRatio`, with fallback to similar ratios if no mode matches.
/// Returns the chosen mode and its aspect difference value.
pub fn pick_highest_for_ratio(
    gop: &mut GraphicsOutput,
    mut aspect: AspectRatio,
) -> Option<(Mode, u64)> {
    while let Some(target) = aspect.as_tuple() {
        let mut best: Option<(Mode, u64, u64)> = None;

        // <— FIXED: create fresh iterator each time
        for mode in list_modes(gop) {
            let info = mode.info();
            if info.pixel_format() == PixelFormat::BltOnly {
                continue;
            }

            let (wu, hu) = info.resolution();
            let diff = ratio_diff(wu as u32, hu as u32, target);
            let area = (wu as u64) * (hu as u64);

            match best {
                None => best = Some((mode, diff, area)),
                Some((_, best_diff, best_area)) => {
                    if diff < best_diff || (diff == best_diff && area > best_area) {
                        best = Some((mode, diff, area));
                    }
                }
            }
        }

        if let Some((m, d, _)) = best {
            return Some((m, d));
        }

        if aspect.get_fallback().is_none() {
            break;
        }
    }

    None
}


/// Picks a specific GOP mode by index (safe lookup).
/// Returns the mode if valid, or None if out of range or unsupported.
pub fn pick_mode(gop: &mut GraphicsOutput, index: usize) -> Option<Mode> {
    list_modes(gop).nth(index)
}
