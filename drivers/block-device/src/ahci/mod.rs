#![allow(dead_code)]

use crate::{BlockDevice, BlockError, SECTOR_SIZE};

pub struct AhciController;

impl AhciController {
    pub fn new_uninitialized() -> Self { Self }
}

impl BlockDevice for AhciController {
    fn read(&mut self, _lba: u64, _sector_count: usize, _buffer: &mut [u8]) -> Result<(), BlockError> {
        Err(BlockError::Unsupported)
    }
    fn write(&mut self, _lba: u64, _sector_count: usize, _buffer: &[u8]) -> Result<(), BlockError> {
        Err(BlockError::Unsupported)
    }
}
