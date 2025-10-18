use core::ptr;
use uefi::boot::{self, AllocateType};
use uefi::mem::memory_map::MemoryType;
use rtos_types::{RtoskSegment, RTOSK_EXEC_FLAG};
use crate::boot::console::{write_hex, write_line};

pub fn map_segments(segments: &[RtoskSegment], image_bytes: &[u8]) -> Result<(), uefi::Status> {
    const MAX_TRACKED: usize = 4096;

    let mut pages: [usize; MAX_TRACKED] = [0; MAX_TRACKED];
    let mut count: usize = 0;

    let seen = |base: usize, c: usize, p: &[usize; MAX_TRACKED]| -> bool {
        for i in 0..c { if p[i] == base { return true; } }
        false
    };
    let track = |base: usize, c: &mut usize, p: &mut [usize; MAX_TRACKED]| {
        if *c < MAX_TRACKED { p[*c] = base; *c += 1; } else { write_line("BL: map warn: page tracker full; may re-alloc"); }
    };

    for (i, seg) in segments.iter().enumerate() {
        let tgt = seg.memory_addr as usize;
        let mem_len = seg.memory_size as usize;
        let file_off = seg.file_offset as usize;
        let file_len = seg.file_size as usize;

        write_hex("BL: map seg", i as u64);

        if mem_len == 0 {
            write_line("BL: map skip (zero mem)");
            continue;
        }

        let start_page = tgt & !0xfffusize;
        let end_page = (tgt + mem_len + 0xfff) & !0xfffusize;
        write_hex("  start_page", start_page as u64);
        write_hex("  end_page", end_page as u64);

        let mem_ty = if (seg.flags & RTOSK_EXEC_FLAG) != 0 { MemoryType::LOADER_CODE } else { MemoryType::LOADER_DATA };

        let mut page = start_page;
        while page < end_page {
            if seen(page, count, &pages) {
                write_hex("BL: map page already alloc", page as u64);
            } else {
                match boot::allocate_pages(AllocateType::Address(page as u64), mem_ty, 1) {
                    Ok(_) => { track(page, &mut count, &mut pages); }
                    Err(e) => { write_line("BL: ERROR allocate_pages"); return Err(e.status()); }
                }
            }
            page += 0x1000;
        }

        if file_len > 0 {
            let end = file_off.saturating_add(file_len);
            if end <= image_bytes.len() {
                unsafe { ptr::copy_nonoverlapping(image_bytes[file_off..end].as_ptr(), tgt as *mut u8, file_len); }
                write_hex("BL: map copied", file_len as u64);
            } else {
                write_line("BL: map ERROR file range OOB");
                return Err(uefi::Status::LOAD_ERROR);
            }
        }

        if mem_len > file_len {
            let zero_start = tgt + file_len;
            let zero_len = mem_len - file_len;
            unsafe { ptr::write_bytes(zero_start as *mut u8, 0, zero_len); }
            write_hex("BL: map zeroed", zero_len as u64);
        }
    }

    Ok(())
}
