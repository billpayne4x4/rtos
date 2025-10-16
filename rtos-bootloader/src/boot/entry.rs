use core::{cmp::max, slice};

use crate::boot::{bootfs, map, open, prepare};
use crate::boot::console::{write_hex, write_line};
use crate::rtosk::parse_header_and_segments;

use rtoskfmt::constants::RTOSK_MAGIC;

pub fn boot_entry() -> uefi::Status {
    write_line("BL: boot_entry start");

    let image = uefi::boot::image_handle();

    let loaded = match open::open_loaded_image(image) {
        Ok(x) => { write_line("BL: opened loaded_image"); x }
        Err(e) => { write_line("BL: ERROR open_loaded_image"); return e; }
    };

    let mut sfs = match open::open_simple_fs(&loaded) {
        Ok(x) => { write_line("BL: opened SimpleFileSystem"); x }
        Err(e) => { write_line("BL: ERROR open_simple_fs"); return e; }
    };

    let mut root = match open::open_root_dir(&mut *sfs) {
        Ok(x) => { write_line("BL: opened root dir"); x }
        Err(e) => { write_line("BL: ERROR open_root_dir"); return e; }
    };

    // Open KERNEL.RTOSK using bootfs
    let mut kfile = match bootfs::open_kernel(&mut root) {
        Ok(f) => { write_line("BL: opened KERNEL.RTOSK"); f }
        Err(e) => { write_line("BL: ERROR open_kernel"); return e.status(); }
    };

    // Size + buffer
    let kernel_size = match bootfs::file_size(&mut kfile) {
        Some(sz) => { write_hex("BL: kernel_size", sz as u64); sz }
        None => { write_line("BL: ERROR get kernel size"); return uefi::Status::LOAD_ERROR; }
    };

    // Allocate a temporary buffer in LOADER_DATA and read the file into it.
    let pages = (kernel_size + 4095) / 4096;
    let buf_ptr = match uefi::boot::allocate_pages(
        uefi::boot::AllocateType::AnyPages,
        uefi::mem::memory_map::MemoryType::LOADER_DATA,
        pages,
    ) {
        Ok(p) => p,
        Err(e) => { write_line("BL: ERROR allocate buffer"); return e.status(); }
    };
    let blob_base = buf_ptr.as_ptr() as usize;
    let blob_slice = unsafe { slice::from_raw_parts_mut(blob_base as *mut u8, pages * 4096) };
    if let Err(e) = bootfs::read_exact(&mut kfile, &mut blob_slice[..kernel_size]) {
        write_line("BL: ERROR kernel read");
        return e.status();
    }
    write_line("BL: kernel blob loaded");

    // Find magic
    let magic_off = find_magic(&blob_slice[..kernel_size], &RTOSK_MAGIC).unwrap_or(usize::MAX);
    if magic_off == usize::MAX {
        write_line("BL: ERROR RTOSK magic not found");
        return uefi::Status::LOAD_ERROR;
    }
    write_hex("BL: RTOSK off", magic_off as u64);

    // Parse header + segments
    let image_bytes = &blob_slice[magic_off..kernel_size];
    let (header, segments, header_len, seg_bytes) = match parse_header_and_segments(image_bytes) {
        Ok(t) => t,
        Err(_) => { write_line("BL: ERROR parse RTOSK"); return uefi::Status::LOAD_ERROR; }
    };

    write_hex("BL: entry64", header.entry64 as u64);
    write_hex("BL: seg_count", header.seg_count as u64);
    write_hex("BL: page_size", header.page_size as u64);
    write_hex("BL: hdr.len", header_len as u64);
    write_hex("BL: segments_bytes", seg_bytes as u64);

    for (i, seg) in segments.iter().enumerate() {
        write_hex("BL: seg[i]", i as u64);
        write_hex("  file_offset", seg.file_offset as u64);
        write_hex("  file_size", seg.file_size as u64);
        write_hex("  memory_addr", seg.memory_addr as u64);
        write_hex("  memory_size", seg.memory_size as u64);
        write_hex("  flags", seg.flags as u64);
    }

    // Prepare stack + boot info
    let page_size = max(header.page_size as usize, 4096usize);
    let (stack_top, boot_info) = match prepare::prepare_stack_and_info(page_size) {
        Ok(t) => t,
        Err(e) => { write_line("BL: ERROR prepare_stack_and_info"); return e; }
    };

    // Map the segments to their requested addresses
    if let Err(e) = map::map_segments(segments, image_bytes) {
        write_line("BL: ERROR map_segments");
        return e;
    }

    // Entry must be non-zero now
    let entry_ptr = header.entry64 as usize;
    if entry_ptr == 0 {
        write_line("BL: FATAL: header.entry64 is 0 — refusing to jump");
        return uefi::Status::LOAD_ERROR;
    }

    // Trampoline uses Win64 ABI: RCX=entry, RDX=stack_top, R8=boot_info
    extern "win64" {
        fn jump_to_kernel(entry: usize, stack_top: usize, boot_info: usize) -> !;
    }

    write_hex("BL: entry (header.entry64)", entry_ptr as u64);
    write_line("BL: calling trampoline (jump_to_kernel)");

    unsafe { jump_to_kernel(entry_ptr, stack_top, boot_info) }
}

// Small local helper so we don’t depend on a separate blob module.
fn find_magic(haystack: &[u8], magic: &[u8]) -> Option<usize> {
    if haystack.len() < magic.len() { return None; }
    for i in 0..=haystack.len() - magic.len() {
        if &haystack[i..i + magic.len()] == magic {
            return Some(i);
        }
    }
    None
}
