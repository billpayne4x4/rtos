use uefi::cstr16;
use uefi::proto::media::file::{Directory, File, FileAttribute, FileInfo, FileMode, FileType, RegularFile};

pub fn open_kernel(root: &mut Directory) -> uefi::Result<RegularFile> {
    if let Ok(h) = root.open(cstr16!(r"\EFI\BOOT\KERNEL.RTOSK"), FileMode::Read, FileAttribute::empty()) {
        match h.into_type()? {
            FileType::Regular(f) => return Ok(f),
            _ => {}
        }
    }
    let h = root.open(cstr16!("KERNEL.RTOSK"), FileMode::Read, FileAttribute::empty())?;
    match h.into_type()? {
        FileType::Regular(f) => Ok(f),
        _ => Err(uefi::Status::NOT_FOUND.into()),
    }
}

pub fn file_size(file: &mut RegularFile) -> Option<usize> {
    let mut info_buf = [0u8; 256];
    file.get_info::<FileInfo>(&mut info_buf).ok().map(|i| i.file_size() as usize)
}

pub fn read_exact(file: &mut RegularFile, dst: &mut [u8]) -> uefi::Result {
    let mut off = 0usize;
    while off < dst.len() {
        let got = file.read(&mut dst[off..])?;
        if got == 0 { break; }
        off += got;
    }
    if off == dst.len() { Ok(()) } else { Err(uefi::Status::END_OF_FILE.into()) }
}
