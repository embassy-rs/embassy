#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockDeviceError {
    // TODO
}

pub trait BlockDevice {
    /// The number of bytes per block. This determines the size of the buffer passed
    /// to read/write functions
    fn block_size(&self) -> usize;

    /// Number of blocks in device (max LBA index)
    fn num_blocks(&self) -> u32;

    /// Read the block indicated by `lba` into the provided buffer
    fn read_block(&self, lba: u32, block: &mut [u8]) -> Result<(), BlockDeviceError>;

    /// Write the `block` buffer to the block indicated by `lba`
    fn write_block(&mut self, lba: u32, block: &[u8]) -> Result<(), BlockDeviceError>;
}
