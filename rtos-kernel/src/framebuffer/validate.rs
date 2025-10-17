use x86_64::VirtAddr;
use crate::framebuffer::Framebuffer;
use crate::serial_log;

#[derive(Clone, Copy, Debug)]
pub enum FbSoftCheckError {
    NonCanonicalVa,
    ZeroDims,
    MisalignedPtr,
    StrideDeltaMismatch { delta_bytes: usize, expect_bytes: usize },
    ProbeStartMismatch { wrote: u32, read: u32 },
    ProbeEndMismatch { wrote: u32, read: u32 },
}

#[inline]
fn fb_total_bytes(fb: &Framebuffer) -> usize {
    (fb.stride as usize) * (fb.height as usize) * 4
}

pub fn validate_framebuffer_soft(fb: &mut Framebuffer) -> Result<(), FbSoftCheckError> {
    let va = VirtAddr::new(fb.ptr as u64);
    if !is_canonical_u64(va.as_u64())  { return Err(FbSoftCheckError::NonCanonicalVa); }
    if fb.width == 0 || fb.height == 0 { return Err(FbSoftCheckError::ZeroDims); }
    if (fb.ptr as usize) & 0x3 != 0 { return Err(FbSoftCheckError::MisalignedPtr); }

    let row0 = fb.row_ptr(0) as usize;
    let row1 = if fb.height > 1 { fb.row_ptr(1) as usize } else { row0 };
    let delta = row1.wrapping_sub(row0);
    let expect = (fb.stride as usize) * 4;
    if delta != expect { return Err(FbSoftCheckError::StrideDeltaMismatch { delta_bytes: delta, expect_bytes: expect }); }

    let width = fb.width as usize;
    let stride = fb.stride as usize;
    let draw_w = if width < stride { width } else { stride };
    if draw_w == 0 { return Ok(()); }

    unsafe {
        let p0 = fb.row_ptr(0);
        let pE = fb.row_ptr(0).add(draw_w - 1);

        let old0 = core::ptr::read_volatile(p0);
        let oldE = core::ptr::read_volatile(pE);

        let t0 = old0 ^ 0x0055AA55;
        let tE = oldE ^ 0x00AA55AA;

        core::ptr::write_volatile(p0, t0);
        core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
        let r0 = core::ptr::read_volatile(p0);
        core::ptr::write_volatile(p0, old0);
        if r0 != t0 { return Err(FbSoftCheckError::ProbeStartMismatch { wrote: t0, read: r0 }); }

        core::ptr::write_volatile(pE, tE);
        core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
        let rE = core::ptr::read_volatile(pE);
        core::ptr::write_volatile(pE, oldE);
        if rE != tE { return Err(FbSoftCheckError::ProbeEndMismatch { wrote: tE, read: rE }); }
    }

    let _ = fb_total_bytes(fb); // keeps the intent obvious for future checks
    Ok(())
}

pub fn is_canonical_u64(va: u64) -> bool {
    // 4-level paging: canonical iff bits 63..48 are sign-ext of bit 47
    let sign = (va >> 47) & 1;
    let hi   = va >> 48;
    (sign == 0 && hi == 0) || (sign == 1 && hi == 0xFFFF)
}