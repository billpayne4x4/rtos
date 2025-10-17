use x86_64::VirtAddr;
use x86_64::structures::paging::mapper::Translate;
use crate::serial_log;
use crate::utils::SerialWriter;
use crate::framebuffer::Framebuffer;

#[derive(Debug, Clone, Copy)]
pub enum FbCheckError {
    NonCanonicalVa,
    UnmappedStart,
    UnmappedEnd,
    StrideDeltaMismatch { delta_bytes: usize, expect_bytes: usize },
    ProbeWriteMismatchStart { wrote: u32, read_back: u32 },
    ProbeWriteMismatchEnd { wrote: u32, read_back: u32 },
}

#[inline]
fn is_canonical_u64(va: u64) -> bool {
    let sign = (va >> 47) & 1;
    let hi = va >> 48;
    (sign == 0 && hi == 0) || (sign == 1 && hi == 0xFFFF)
}

#[inline]
fn fb_byte_len(fb: &Framebuffer) -> usize {
    (fb.stride as usize) * (fb.height as usize) * 4
}

pub unsafe fn check_framebuffer_mapped<T: Translate>(
    mapper: &T,
    fb: &Framebuffer,
) -> Result<(), FbCheckError> {
    let va = VirtAddr::new(fb.ptr as u64);
    if !is_canonical_u64(va.as_u64()) {
        return Err(FbCheckError::NonCanonicalVa);
    }

    let start = VirtAddr::new(fb.ptr as u64);
    let end = VirtAddr::new((fb.ptr as u64).saturating_add(fb_byte_len(fb).saturating_sub(1) as u64));

    if mapper.translate_addr(start).is_none() { return Err(FbCheckError::UnmappedStart); }
    if mapper.translate_addr(end).is_none()   { return Err(FbCheckError::UnmappedEnd); }

    let row0 = fb.row_ptr(0) as usize;
    let row1 = if fb.height > 1 { fb.row_ptr(1) as usize } else { row0 };
    let delta = row1.wrapping_sub(row0);
    let expect = (fb.stride as usize) * 4;
    if delta != expect {
        return Err(FbCheckError::StrideDeltaMismatch { delta_bytes: delta, expect_bytes: expect });
    }

    Ok(())
}

pub unsafe fn probe_framebuffer_rw(fb: &mut Framebuffer) -> Result<(), FbCheckError> {
    let width = fb.width as usize;
    let height = fb.height as usize;
    let stride = fb.stride as usize;
    let draw_w = core::cmp::min(width, stride);
    if draw_w == 0 || height == 0 { return Ok(()); }

    let p0 = fb.row_ptr(0);
    let pE = fb.row_ptr(0).add(draw_w - 1);

    let old0 = core::ptr::read_volatile(p0);
    let oldE = core::ptr::read_volatile(pE);

    let test0 = old0 ^ 0x0055AA55;
    let testE = oldE ^ 0x00AA55AA;

    core::ptr::write_volatile(p0, test0);
    core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
    let back0 = core::ptr::read_volatile(p0);
    core::ptr::write_volatile(p0, old0);
    if back0 != test0 {
        return Err(FbCheckError::ProbeWriteMismatchStart { wrote: test0, read_back: back0 });
    }

    core::ptr::write_volatile(pE, testE);
    core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
    let backE = core::ptr::read_volatile(pE);
    core::ptr::write_volatile(pE, oldE);
    if backE != testE {
        return Err(FbCheckError::ProbeWriteMismatchEnd { wrote: testE, read_back: backE });
    }

    Ok(())
}

pub unsafe fn validate_framebuffer<T: Translate>(
    mapper: Option<&T>,
    fb: &mut Framebuffer,
) -> bool {
    SerialWriter::write("K: FB base: "); SerialWriter::write_hex(fb.ptr as usize); SerialWriter::write("\n");

    if let Some(m) = mapper {
        match check_framebuffer_mapped(m, fb) {
            Ok(()) => serial_log!("FB mapping looks present."),
            Err(e) => { serial_log!("FB mapping check failed: "); pretty_err(e); return false; }
        }
    } else {
        serial_log!("Skipping page-table translation check (no mapper provided).");
    }

    let row0 = fb.row_ptr(0) as usize;
    let row1 = if fb.height > 1 { fb.row_ptr(1) as usize } else { row0 };
    SerialWriter::write("K: row0: "); SerialWriter::write_hex(row0); SerialWriter::write("\n");
    SerialWriter::write("K: row1: "); SerialWriter::write_hex(row1); SerialWriter::write("\n");
    SerialWriter::write("K: delta: "); SerialWriter::write_usize(row1.wrapping_sub(row0)); SerialWriter::write("\n");
    SerialWriter::write("K: expect: "); SerialWriter::write_usize((fb.stride as usize) * 4); SerialWriter::write("\n");

    match probe_framebuffer_rw(fb) {
        Ok(()) => { serial_log!("FB probe writes OK"); true }
        Err(e) => { serial_log!("FB probe writes FAILED: "); pretty_err(e); false }
    }
}

fn pretty_err(e: FbCheckError) {
    use FbCheckError::*;
    match e {
        NonCanonicalVa => SerialWriter::write("NonCanonicalVa\n"),
        UnmappedStart  => SerialWriter::write("UnmappedStart\n"),
        UnmappedEnd    => SerialWriter::write("UnmappedEnd\n"),
        StrideDeltaMismatch{delta_bytes, expect_bytes} => {
            SerialWriter::write("StrideDeltaMismatch delta="); SerialWriter::write_usize(delta_bytes);
            SerialWriter::write(" expect="); SerialWriter::write_usize(expect_bytes); SerialWriter::write("\n");
        }
        ProbeWriteMismatchStart{wrote, read_back} => {
            SerialWriter::write("ProbeStart wrote="); SerialWriter::write_hex(wrote as usize);
            SerialWriter::write(" read="); SerialWriter::write_hex(read_back as usize); SerialWriter::write("\n");
        }
        ProbeWriteMismatchEnd{wrote, read_back} => {
            SerialWriter::write("ProbeEnd wrote="); SerialWriter::write_hex(wrote as usize);
            SerialWriter::write(" read="); SerialWriter::write_hex(read_back as usize); SerialWriter::write("\n");
        }
    }
}
