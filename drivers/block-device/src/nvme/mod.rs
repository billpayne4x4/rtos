#![allow(dead_code)]

use crate::{BlockDevice, BlockError};

pub struct NvmeController;

impl NvmeController {
    pub fn new_uninitialized() -> Self { Self }
}

impl BlockDevice for NvmeController {
    fn read(&mut self, _lba: u64, _sector_count: usize, _buffer: &mut [u8]) -> Result<(), BlockError> {
        Err(BlockError::Unsupported)
    }
}
