#![allow(dead_code)]

use crate::{BlockDevice, BlockError, SECTOR_SIZE};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum FsError { Io, NotFound, BadFs, Unsupported }

pub struct Fat32;

impl Fat32 {
    pub fn mount(dev: &mut dyn BlockDevice) -> Result<Self, FsError> {
        let mut sector0 = [0u8; SECTOR_SIZE];
        dev.read(0, 1, &mut sector0).map_err(|_| FsError::Io)?;
        if sector0.get(510) != Some(&0x55) || sector0.get(511) != Some(&0xAA) {
            return Err(FsError::BadFs);
        }
        Ok(Fat32)
    }

    pub fn open<'a>(&'a self, _path: &str) -> Result<File<'a>, FsError> {
        Err(FsError::Unsupported)
    }
}

pub struct File<'a> {
    pub length_bytes: u64,
    _marker: core::marker::PhantomData<&'a ()>,
}
