use core::{mem, ptr, slice};
use rtos_types::{RtoskHeader, RtoskSegment, RTOSK_MAGIC};

pub fn parse_header(bytes: &[u8]) -> Result<(RtoskHeader, usize), ()> {
    if bytes.len() < mem::size_of::<RtoskHeader>() { return Err(()); }
    let mut header = unsafe { mem::zeroed::<RtoskHeader>() };
    unsafe {
        ptr::copy_nonoverlapping(
            bytes.as_ptr(),
            &mut header as *mut RtoskHeader as *mut u8,
            mem::size_of::<RtoskHeader>(),
        );
    }
    if header.magic != RTOSK_MAGIC { return Err(()); }
    Ok((header, mem::size_of::<RtoskHeader>()))
}

pub fn parse_segments(bytes: &[u8], segment_count: u32) -> Result<(&[RtoskSegment], usize), ()> {
    if segment_count == 0 { return Ok((&[], 0)); }
    let need = (segment_count as usize)
        .checked_mul(mem::size_of::<RtoskSegment>())
        .ok_or(())?;
    if bytes.len() < need { return Err(()); }
    let segs = unsafe {
        slice::from_raw_parts(bytes.as_ptr() as *const RtoskSegment, segment_count as usize)
    };
    Ok((segs, need))
}

pub fn parse_header_and_segments<'a>(
    image_bytes: &'a [u8],
) -> Result<(RtoskHeader, &'a [RtoskSegment], usize, usize), ()> {
    let (header, header_len) = parse_header(image_bytes)?;
    let rest = &image_bytes[header_len..];
    let (segments, seg_bytes) = parse_segments(rest, header.seg_count)?;
    Ok((header, segments, header_len, seg_bytes))
}

pub fn find_magic(haystack: &[u8], magic: &[u8]) -> Option<usize> {
    if haystack.len() < magic.len() { return None; }
    for i in 0..=haystack.len() - magic.len() {
        if &haystack[i..i + magic.len()] == magic {
            return Some(i);
        }
    }
    None
}