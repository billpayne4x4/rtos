#![no_std]
#![no_main]

use core::fmt::Write;
use panic_abort as _;

use uefi::boot::{
    allocate_pool, free_pool, get_image_file_system, image_handle, load_image, start_image,
    LoadImageSource, MemoryType,
};
use uefi::prelude::*;
use uefi::proto::media::file::{
    Directory, File as FileTrait, FileInfo, FileMode, FileType, RegularFile,
};
use uefi::{cstr16, Status};

#[entry]
fn efi_main() -> Status {
    if uefi::helpers::init().is_err() {
        return Status::ABORTED;
    }

    uefi::system::with_stdout(|stdout| {
        let _ = stdout.reset(false);
        let _ = writeln!(stdout, "rtos-bootloader: starting");
    });

    let mut sfs = match get_image_file_system(image_handle()) {
        Ok(p) => p,
        Err(e) => return e.status(),
    };

    let mut root: Directory = match sfs.open_volume() {
        Ok(d) => d,
        Err(e) => return e.status(),
    };

    // Matches what the script stages
    let kernel_path = cstr16!("\\EFI\\BOOT\\KERNELX64.EFI");
    let handle = match FileTrait::open(&mut root, kernel_path, FileMode::Read, Default::default()) {
        Ok(h) => h,
        Err(e) => return e.status(),
    };

    let mut kernel: RegularFile = match handle.into_type() {
        Ok(FileType::Regular(f)) => f,
        _ => return Status::NOT_FOUND,
    };

    // First call with 0-len buffer to discover required FileInfo size
    let mut tmp = [0u8; 0];
    let info_size = match FileTrait::get_info::<FileInfo>(&mut kernel, &mut tmp) {
        Ok(_) => return Status::DEVICE_ERROR, // should not succeed with zero buffer
        Err(err) => match err.data() {
            Some(sz) => *sz,
            None => return Status::DEVICE_ERROR,
        },
    };

    // Allocate pool for FileInfo and fetch it
    let info_ptr = match allocate_pool(MemoryType::LOADER_DATA, info_size) {
        Ok(p) => p,
        Err(e) => return e.status(),
    };
    let info_buf = unsafe { core::slice::from_raw_parts_mut(info_ptr.as_ptr(), info_size) };
    let info = match FileTrait::get_info::<FileInfo>(&mut kernel, info_buf) {
        Ok(i) => i,
        Err(e) => {
            unsafe { let _ = free_pool(info_ptr); }
            return e.status();
        }
    };
    let kernel_size = info.file_size() as usize;
    unsafe { let _ = free_pool(info_ptr); } // done with FileInfo buffer

    // Allocate pool for the kernel image and read it fully
    let img_ptr = match allocate_pool(MemoryType::LOADER_DATA, kernel_size) {
        Ok(p) => p,
        Err(e) => return e.status(),
    };
    let img_buf = unsafe { core::slice::from_raw_parts_mut(img_ptr.as_ptr(), kernel_size) };

    let mut read_total = 0usize;
    while read_total < kernel_size {
        match kernel.read(&mut img_buf[read_total..]) {
            Ok(0) => break,        // EOF
            Ok(n) => read_total += n,
            Err(_) => {
                unsafe { let _ = free_pool(img_ptr); }
                return Status::DEVICE_ERROR;
            }
        }
    }
    if read_total != kernel_size {
        unsafe { let _ = free_pool(img_ptr); }
        return Status::END_OF_FILE;
    }

    // Load and start the kernel from the in-memory buffer
    let src = LoadImageSource::FromBuffer {
        buffer: &img_buf[..read_total],
        file_path: None,
    };
    let child = match load_image(image_handle(), src) {
        Ok(h) => h,
        Err(e) => {
            unsafe { let _ = free_pool(img_ptr); }
            return e.status();
        }
    };

    // Firmware parsed the image; free our buffer before transferring control
    unsafe { let _ = free_pool(img_ptr); }

    match start_image(child) {
        Ok(()) => Status::SUCCESS,
        Err(e) => e.status(),
    }
}