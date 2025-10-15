use core::ptr;
use uefi::boot::{self, AllocateType};
use uefi::mem::memory_map::MemoryType;
use uefi::Status;
use crate::boot::console::write_hex;

pub fn prepare_stack_and_info(page_size: usize) -> Result<(usize, usize), Status> {
    let stack_pages = 8usize;
    let stack_bytes = stack_pages * page_size;
    let stack_ptr = match boot::allocate_pages(AllocateType::AnyPages, MemoryType::LOADER_DATA, stack_pages) {
        Ok(p) => p,
        Err(e) => return Err(e.status()),
    };
    let stack_top = stack_ptr.as_ptr() as usize + stack_bytes;

    let boot_info_pages = 1usize;
    let boot_info_ptr = match boot::allocate_pages(AllocateType::AnyPages, MemoryType::LOADER_DATA, boot_info_pages) {
        Ok(p) => p,
        Err(e) => return Err(e.status()),
    };
    let boot_info_base = boot_info_ptr.as_ptr() as usize;
    unsafe { ptr::write_bytes(boot_info_base as *mut u8, 0, boot_info_pages * page_size); }

    write_hex("BL: stack_top", stack_top as u64);
    write_hex("BL: boot_info", boot_info_base as u64);

    Ok((stack_top, boot_info_base))
}
