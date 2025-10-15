use uefi::prelude::*;
use uefi::boot;
use crate::boot::console::write_line;

// NOTE: ScopedProtocol lives in `uefi::boot` (not `uefi::scoped_protocol`)
pub fn open_loaded_image(
    image: Handle,
) -> Result<uefi::boot::ScopedProtocol<uefi::proto::loaded_image::LoadedImage>, Status> {
    match boot::open_protocol_exclusive::<uefi::proto::loaded_image::LoadedImage>(image) {
        Ok(p) => Ok(p),
        Err(e) => { write_line("BL: ERROR LoadedImage"); Err(e.status()) }
    }
}

pub fn open_simple_fs(
    loaded: &uefi::proto::loaded_image::LoadedImage,
) -> Result<uefi::boot::ScopedProtocol<uefi::proto::media::fs::SimpleFileSystem>, Status> {
    let device = match loaded.device() {
        Some(h) => h,
        None => { write_line("BL: ERROR LoadedImage.device() is None"); return Err(Status::LOAD_ERROR); }
    };
    match boot::open_protocol_exclusive::<uefi::proto::media::fs::SimpleFileSystem>(device) {
        Ok(p) => Ok(p),
        Err(e) => { write_line("BL: ERROR SimpleFileSystem"); Err(e.status()) }
    }
}

pub fn open_root_dir(
    sfs: &mut uefi::proto::media::fs::SimpleFileSystem,
) -> Result<uefi::proto::media::file::Directory, Status> {
    sfs.open_volume().map_err(|e| e.status())
}
