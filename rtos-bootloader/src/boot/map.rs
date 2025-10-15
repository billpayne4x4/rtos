use rtkfmt::{RtkHeader, RtkSegment};
use crate::boot::console::{write_line, write_hex};

pub fn effective_page_size(header: &RtkHeader) -> usize {
    let ps = header.page_size as usize;
    if ps.is_power_of_two() && ps >= 4096 { ps } else { 4096 }
}

/// Map & copy RTOSK segments to their requested memory addresses.
/// Allocates per 4KiB page and skips pages we've already allocated this run,
/// so overlapping segments (e.g. 0x200000 and 0x200048) work cleanly.
pub fn map_segments(segments: &[RtkSegment], image_bytes: &[u8]) -> Result<(), uefi::Status> {
    use core::ptr;
    use uefi::boot::{allocate_pages, AllocateType};
    use uefi::mem::memory_map::MemoryType;

    const PAGE: usize = 4096;
    const MAX_TRACKED: usize = 512;

    let mut tracked_pages: [usize; MAX_TRACKED] = [0; MAX_TRACKED];
    let mut tracked_count: usize = 0;

    for (seg_index, seg) in segments.iter().enumerate() {
        let target_addr = seg.memory_addr as usize;
        let memory_len  = seg.memory_size as usize;
        let file_offset = seg.file_offset as usize;
        let file_len    = seg.file_size as usize;

        if memory_len == 0 {
            write_hex("BL: map skip (zero mem) seg", seg_index as u64);
            continue;
        }

        // Page range covering this segment
        let start = target_addr;
        let end = match target_addr.checked_add(memory_len) {
            Some(v) => v,
            None => return Err(uefi::Status::LOAD_ERROR),
        };
        let start_page = start & !(PAGE - 1);
        let end_page   = (end + (PAGE - 1)) & !(PAGE - 1); // exclusive

        write_hex("BL: map seg", seg_index as u64);
        write_hex("  start_page", start_page as u64);
        write_hex("  end_page", end_page as u64);

        // NOTE: allocate pages as LOADER_CODE so the region is executable
        let mem_type = MemoryType::LOADER_CODE;

        // Allocate each page once
        let mut page_base = start_page;
        while page_base < end_page {
            let mut already_allocated = false;
            for i in 0..tracked_count {
                if tracked_pages[i] == page_base {
                    already_allocated = true;
                    break;
                }
            }

            if already_allocated {
                write_hex("BL: map page already alloc", page_base as u64);
            } else {
                match allocate_pages(AllocateType::Address(page_base as u64), mem_type, 1) {
                    Ok(ptr_nonnull) => {
                        let got = ptr_nonnull.as_ptr() as usize;
                        if got != page_base {
                            write_line("BL: map alloc addr mismatch");
                            write_hex("  got", got as u64);
                            write_hex("  want", page_base as u64);
                            return Err(uefi::Status::LOAD_ERROR);
                        }
                        write_hex("BL: map page allocated", page_base as u64);
                        if tracked_count < MAX_TRACKED {
                            tracked_pages[tracked_count] = page_base;
                            tracked_count += 1;
                        } else {
                            write_line("BL: map warn: page tracker full; may re-alloc");
                        }
                    }
                    Err(err) => {
                        write_line("BL: ERROR allocate_pages");
                        write_hex("  page_base", page_base as u64);
                        return Err(err.status());
                    }
                }
            }

            page_base += PAGE;
        }

        // Copy file bytes (if any)
        if file_len > 0 {
            if let Some(end_idx) = file_offset.checked_add(file_len).filter(|e| *e <= image_bytes.len()) {
                unsafe {
                    let src = &image_bytes[file_offset..end_idx];
                    let dst = target_addr as *mut u8;
                    ptr::copy_nonoverlapping(src.as_ptr(), dst, src.len());
                }
                write_hex("BL: map copied", file_len as u64);
            } else {
                write_line("BL: ERROR map file range OOB");
                return Err(uefi::Status::LOAD_ERROR);
            }
        }

        // Zero the rest
        if memory_len > file_len {
            let zero_start = target_addr + file_len;
            let zero_len   = memory_len - file_len;
            unsafe { ptr::write_bytes(zero_start as *mut u8, 0, zero_len); }
            write_hex("BL: map zeroed", zero_len as u64);
        }
    }

    Ok(())
}
