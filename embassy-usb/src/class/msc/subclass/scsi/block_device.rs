#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum BlockDeviceError {
    /// Block device is not present and cannot be accessed.
    ///
    /// SCSI NOT READY 3Ah/00h MEDIUM NOT PRESENT
    MediumNotPresent,
    /// Logical Block Address is out of range
    ///
    /// SCSI ILLEGAL REQUEST 21h/00h LOGICAL BLOCK ADDRESS OUT OF RANGE
    LbaOutOfRange,
    /// Unrecoverable hardware error
    ///
    /// SCSI HARDWARE ERROR 00h/00h NO ADDITIONAL SENSE INFORMATION
    HardwareError,
    /// SCSI MEDIUM ERROR 11h/00h UNRECOVERED READ ERROR
    ReadError,
    /// SCSI MEDIUM ERROR 0Ch/00h WRITE ERROR
    WriteError,
    /// SCSI MEDIUM ERROR 51h/00h ERASE FAILURE
    EraseError,
}

pub trait BlockDevice {
    /// Called for periodic `TEST UNIT READY` SCSI requests.
    ///
    /// Should return error if device is not ready (i.e. [BlockDeviceError::MediumRemoved] if SD card is not present).
    fn status(&self) -> Result<(), BlockDeviceError>;

    /// The number of bytes per block. This determines the size of the buffer passed
    /// to read/write functions.
    fn block_size(&self) -> Result<usize, BlockDeviceError>;

    /// Number of blocks in device (max LBA index)
    fn max_lba(&self) -> Result<u32, BlockDeviceError>;

    /// Read the block indicated by `lba` into the provided buffer
    async fn read_block(&self, lba: u32, block: &mut [u8]) -> Result<(), BlockDeviceError>;

    /// Write the `block` buffer to the block indicated by `lba`
    async fn write_block(&mut self, lba: u32, block: &[u8]) -> Result<(), BlockDeviceError>;
}
