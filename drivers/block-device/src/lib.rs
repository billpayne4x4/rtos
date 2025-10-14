#![no_std]

pub const SECTOR_SIZE: usize = 512;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum BlockError { Io, BadParam, Unsupported }

pub trait BlockDevice {
    fn read(&mut self, lba: u64, sector_count: usize, buffer: &mut [u8]) -> Result<(), BlockError>;
    fn write(&mut self, _lba: u64, _sector_count: usize, _buffer: &[u8]) -> Result<(), BlockError> { Err(BlockError::Unsupported) }
}

#[cfg(feature = "ahci")]
pub mod ahci;

#[cfg(feature = "nvme")]
pub mod nvme;

#[cfg(feature = "virtio-blk")]
pub mod virtio_blk;

#[cfg(feature = "fat32")]
pub mod file_systems {
    pub mod fat32;
}
