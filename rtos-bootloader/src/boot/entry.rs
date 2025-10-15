use crate::boot::{bootfs, map, open, prepare};
use crate::boot::console::{write_hex, write_line};
use crate::rtk::{parse_header, parse_segments};
use rtkfmt::constants::RTOSK_MAGIC;

pub fn boot_entry() -> uefi::Status {
    write_line("BL: boot_entry start");

    let image = uefi::boot::image_handle();

    let loaded = match open::open_loaded_image(image) {
        Ok(v) => { write_line("BL: opened loaded_image"); v }
        Err(e) => { write_line("BL: ERROR open_loaded_image"); return e; }
    };

    let mut sfs = match open::open_simple_fs(&loaded) {
        Ok(v) => { write_line("BL: opened SimpleFileSystem"); v }
        Err(e) => { write_line("BL: ERROR open_simple_fs"); return e; }
    };

    let mut root = match open::open_root_dir(&mut *sfs) {
        Ok(v) => { write_line("BL: opened root dir"); v }
        Err(e) => { write_line("BL: ERROR open_root_dir"); return e; }
    };

    let mut kf = match bootfs::open_kernel(&mut root) {
        Ok(v) => { write_line("BL: opened KERNEL.RTOSK"); v }
        Err(e) => { write_line("BL: ERROR open KERNEL.RTOSK"); return e.status(); }
    };

    let ksize = match bootfs::file_size(&mut kf) {
        Some(sz) if sz > 0 => sz,
        _ => { write_line("BL: ERROR kernel size"); return uefi::Status::LOAD_ERROR; }
    };
    write_hex("BL: kernel_size", ksize as u64);

    use uefi::boot::{allocate_pages, AllocateType};
    use uefi::mem::memory_map::MemoryType;

    let pages = (ksize + 4095) / 4096;
    let buf_ptr = match allocate_pages(AllocateType::AnyPages, MemoryType::LOADER_DATA, pages) {
        Ok(p) => p,
        Err(e) => { write_line("BL: ERROR allocate kernel buffer"); return e.status(); }
    };
    let buf_base = buf_ptr.as_ptr() as usize;
    let blob_slice = unsafe { core::slice::from_raw_parts_mut(buf_base as *mut u8, ksize) };

    if let Err(e) = bootfs::read_exact(&mut kf, blob_slice) {
        write_line("BL: ERROR read kernel blob");
        return e.status();
    }
    write_line("BL: kernel read");
    write_line("BL: kernel blob loaded");

    let magic_off = find_rtosk_offset(blob_slice).unwrap_or(usize::MAX);
    if magic_off == usize::MAX {
        write_line("BL: ERROR RTOSK magic not found");
        return uefi::Status::LOAD_ERROR;
    }
    write_hex("BL: RTOSK off", magic_off as u64);

    let image_bytes = &blob_slice[magic_off..ksize];

    let (header, consumed) = match parse_header(image_bytes) {
        Ok(v) => v,
        Err(_) => { write_line("BL: parse_header failed"); return uefi::Status::LOAD_ERROR; }
    };

    write_hex("BL: entry64", header.entry64 as u64);
    write_hex("BL: seg_count", header.seg_count as u64);
    write_hex("BL: page_size", header.page_size as u64);
    write_hex("BL: hdr.len", header.header_len as u64);

    if consumed > image_bytes.len() {
        write_line("BL: ERROR header size beyond image");
        return uefi::Status::LOAD_ERROR;
    }
    let (segments, seg_bytes) = match parse_segments(&image_bytes[consumed..], header.seg_count) {
        Ok(v) => v,
        Err(_) => { write_line("BL: parse_segments failed"); return uefi::Status::LOAD_ERROR; }
    };
    write_hex("BL: segments_bytes", seg_bytes as u64);

    for (i, seg) in segments.iter().enumerate() {
        write_hex("BL: seg[i]", i as u64);
        write_hex("  file_offset", seg.file_offset as u64);
        write_hex("  file_size", seg.file_size as u64);
        write_hex("  memory_addr", seg.memory_addr as u64);
        write_hex("  memory_size", seg.memory_size as u64);
        write_hex("  flags", seg.flags as u64);
    }

    if header.entry64 == 0 {
        write_line("BL: FATAL: header.entry64 is 0 — refusing to jump");
        write_line("BL: Hint: packer must write the 64-bit VA of the kernel entry into RTOSK header.entry64");
        return uefi::Status::LOAD_ERROR;
    }

    let page_size = map::effective_page_size(&header);
    let (stack_top, boot_info) = match prepare::prepare_stack_and_info(page_size) {
        Ok(v) => v,
        Err(e) => { write_line("BL: ERROR prepare_stack_and_info"); return e; }
    };

    if let Err(status) = map::map_segments(segments, image_bytes) {
        write_line("BL: ERROR map_segments");
        return status;
    }

    let entry_ptr = header.entry64 as usize;
    write_hex("BL: entry (header.entry64)", entry_ptr as u64);

    extern "C" {
        fn jump_to_kernel(_a: usize, _b: usize, stack_top: usize, entry: usize, boot_info: usize) -> !;
    }

    unsafe {
        write_line("BL: calling trampoline (jump_to_kernel)");
        jump_to_kernel(0, 0, stack_top, entry_ptr, boot_info);
    }
}

fn find_rtosk_offset(haystack: &[u8]) -> Option<usize> {
    if haystack.len() < RTOSK_MAGIC.len() { return None; }
    for i in 0..=haystack.len() - RTOSK_MAGIC.len() {
        if &haystack[i..i + RTOSK_MAGIC.len()] == RTOSK_MAGIC {
            return Some(i);
        }
    }
    None
}
