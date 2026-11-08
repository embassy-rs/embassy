//! Per-LUN SCSI commands and block I/O.

#[cfg(feature = "block-device-driver")]
use core::marker::PhantomData;

use embassy_sync::blocking_mutex::raw::{NoopRawMutex, RawMutex};
use embassy_usb_driver::host::UsbHostAllocator;

use super::super::scsi::*;
use super::{
    BlockCapacity, CommandOutcome, DataDir, InquiryData, MscDevice, MscError, PeripheralType, SenseData, SenseKey,
};

/// Handle for a single Logical Unit.
///
/// Borrows the [`MscDevice`] for transport access. Convenience methods
/// wrap the SCSI commands a storage host typically needs; power users
/// can issue arbitrary CDBs via [`MscDevice::command`].
///
/// Block-I/O methods use the LUN's cached capacity. The first call to
/// [`MscLun::capacity`] fills it; [`MscLun::invalidate_capacity`]
/// clears it (useful after a `UnitAttention` indicating media change).
pub struct MscLun<'dev, 'd, A, M = NoopRawMutex>
where
    A: UsbHostAllocator<'d>,
    M: RawMutex,
{
    pub(super) device: &'dev MscDevice<'d, A, M>,
    pub(super) lun: u8,
    pub(super) capacity: Option<BlockCapacity>,
}

impl<'dev, 'd, A, M> MscLun<'dev, 'd, A, M>
where
    A: UsbHostAllocator<'d>,
    M: RawMutex,
{
    /// LUN index.
    pub fn lun(&self) -> u8 {
        self.lun
    }

    /// Cached [`BlockCapacity`], if [`capacity`](Self::capacity) has
    /// been called.
    pub fn cached_capacity(&self) -> Option<BlockCapacity> {
        self.capacity
    }

    /// Clear the cached capacity so the next I/O re-fetches it.
    pub fn invalidate_capacity(&mut self) {
        self.capacity = None;
    }

    /// Run a single command with auto-sense on failure. Takes and
    /// releases the transport lock.
    async fn run(&mut self, cdb: &[u8], data: DataDir<'_>) -> Result<u32, MscError> {
        let mut t = self.device.transport.lock().await;
        t.run_with_sense(self.lun, cdb, data).await
    }

    /// Run `INQUIRY` (standard data, 36 bytes).
    pub async fn inquiry<'a>(&mut self, buf: &'a mut [u8; INQUIRY_DATA_LEN]) -> Result<InquiryData<'a>, MscError> {
        let cdb = [SCSI_INQUIRY, 0, 0, 0, INQUIRY_DATA_LEN as u8, 0];
        self.run(&cdb, DataDir::In(&mut buf[..])).await?;
        Ok(InquiryData {
            peripheral: PeripheralType::from_bits(buf[0]),
            removable: buf[1] & INQUIRY_RMB != 0,
            vendor: &buf[8..16],
            product: &buf[16..32],
            revision: &buf[32..36],
        })
    }

    /// Read SCSI sense data via `REQUEST SENSE`.
    pub async fn request_sense(&mut self) -> Result<SenseData, MscError> {
        self.device.transport.lock().await.request_sense(self.lun).await
    }

    /// Probe with `TEST UNIT READY`.
    ///
    /// Returns `Ok(true)` when the unit is ready, `Ok(false)` on a
    /// transient `NotReady` or `UnitAttention` sense (e.g. medium
    /// not yet spun up or just inserted). Other failures surface as
    /// [`MscError::Scsi`].
    pub async fn test_unit_ready(&mut self) -> Result<bool, MscError> {
        let cdb = [SCSI_TEST_UNIT_READY, 0, 0, 0, 0, 0];
        // Hold the lock across the TUR → REQUEST SENSE pair so the
        // sense data reports our TUR rather than any interleaved
        // command.
        let sense = {
            let mut t = self.device.transport.lock().await;
            match t.command(self.lun, &cdb, DataDir::None).await? {
                CommandOutcome::Ok { .. } => return Ok(true),
                CommandOutcome::Failed { .. } => t.request_sense(self.lun).await?,
            }
        };
        match sense.key {
            SenseKey::NotReady | SenseKey::UnitAttention => {
                self.invalidate_capacity();
                Ok(false)
            }
            _ => Err(MscError::Scsi(sense)),
        }
    }

    /// Enable or disable medium removal by the user.
    pub async fn prevent_medium_removal(&mut self, prevent: bool) -> Result<(), MscError> {
        let cdb = [SCSI_PREVENT_ALLOW_MEDIUM_REMOVAL, 0, 0, 0, prevent as u8, 0];
        self.run(&cdb, DataDir::None).await?;
        Ok(())
    }

    /// Fetch and cache the LUN's block capacity.
    ///
    /// Uses `READ CAPACITY(10)`; falls back to `READ CAPACITY(16)` when
    /// the device reports the sentinel `0xFFFFFFFF` (i.e. the LUN is
    /// larger than 2 TiB at 512-byte blocks). The two probes run
    /// under a single lock hold.
    pub async fn capacity(&mut self) -> Result<BlockCapacity, MscError> {
        if let Some(c) = self.capacity {
            return Ok(c);
        }

        let cap = {
            let mut t = self.device.transport.lock().await;

            let cdb = [SCSI_READ_CAPACITY_10, 0, 0, 0, 0, 0, 0, 0, 0, 0];
            let mut buf = [0u8; Capacity::LEN_10];
            t.run_with_sense(self.lun, &cdb, DataDir::In(&mut buf)).await?;
            let mut cap = Capacity::parse_10(&buf);

            if cap.last_lba == Capacity::LBA_10_OVERFLOW as u64 {
                let mut cdb16 = [0u8; 16];
                cdb16[0] = SCSI_SERVICE_ACTION_IN_16;
                cdb16[1] = SCSI_SA_READ_CAPACITY_16;
                cdb16[13] = Capacity::LEN_16 as u8;
                let mut buf16 = [0u8; Capacity::LEN_16];
                t.run_with_sense(self.lun, &cdb16, DataDir::In(&mut buf16)).await?;
                cap = Capacity::parse_16(&buf16);
            }

            BlockCapacity {
                block_count: cap.last_lba.saturating_add(1),
                block_size: cap.block_size,
            }
        };

        if cap.block_size == 0 {
            return Err(MscError::InvalidResponse);
        }

        self.capacity = Some(cap);
        Ok(cap)
    }

    /// Read `buf.len() / block_size` blocks starting at `lba`.
    ///
    /// `buf.len()` must be a non-zero multiple of `block_size`.
    ///
    /// Uses `READ(10)` when `lba` fits in `u32` and the chunk count
    /// fits in `u16`, else `READ(16)`. Large reads are transparently
    /// split into chunks of up to 65 535 blocks per command, all
    /// issued under a single transport lock hold.
    pub async fn read_blocks(&mut self, lba: u64, buf: &mut [u8]) -> Result<(), MscError> {
        self.transfer_blocks(lba, DataDir::In(buf)).await
    }

    /// Write `buf.len() / block_size` blocks starting at `lba`.
    ///
    /// `buf.len()` must be a non-zero multiple of `block_size`.
    ///
    /// Uses `WRITE(10)` when `lba` fits in `u32` and the chunk count
    /// fits in `u16`, else `WRITE(16)`. Large writes are transparently
    /// split into chunks of up to 65 535 blocks per command, all
    /// issued under a single transport lock hold.
    pub async fn write_blocks(&mut self, lba: u64, buf: &[u8]) -> Result<(), MscError> {
        self.transfer_blocks(lba, DataDir::Out(buf)).await
    }

    /// Move `data` to or from the blocks starting at `lba`, in as many
    /// READ or WRITE commands as needed, under a single lock hold.
    async fn transfer_blocks(&mut self, lba: u64, mut data: DataDir<'_>) -> Result<(), MscError> {
        let cap = self.capacity().await?;
        let (block_size, total_blocks) = check_block_args(lba, data.len(), &cap)?;
        let write = matches!(data, DataDir::Out(_));

        let mut t = self.device.transport.lock().await;

        let mut cdb_buf = [0u8; 16];
        let mut cur_lba = lba;
        let mut offset = 0usize;
        let mut remaining = total_blocks;

        while remaining > 0 {
            let blocks = chunk_blocks(cur_lba, remaining);
            let bytes = blocks as usize * block_size;
            let cdb = ReadWrite {
                write,
                lba: cur_lba,
                blocks,
            }
            .encode(&mut cdb_buf);

            let residue = t
                .run_with_sense(self.lun, cdb, data.slice(offset..offset + bytes))
                .await?;
            if residue != 0 {
                return Err(MscError::InvalidResponse);
            }

            offset += bytes;
            cur_lba += blocks as u64;
            remaining -= blocks as u64;
        }
        Ok(())
    }

    /// Flush the device's write cache (`SYNCHRONIZE CACHE(10)` over
    /// the entire LUN).
    pub async fn flush(&mut self) -> Result<(), MscError> {
        let cdb = [SCSI_SYNCHRONIZE_CACHE_10, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        self.run(&cdb, DataDir::None).await?;
        Ok(())
    }

    /// Wrap this LUN as a [`block_device_driver::BlockDevice`].
    ///
    /// `ALIGN` selects the alignment of the caller's block buffers
    /// (must divide `SIZE`). The `SIZE` const generic on the trait
    /// impl is the block size in bytes; it is checked against the
    /// LUN's reported block size on every call and
    /// [`MscError::BlockSizeMismatch`] is returned on mismatch.
    #[cfg(feature = "block-device-driver")]
    pub fn as_block_device<ALIGN>(&mut self) -> MscBlockDevice<'_, 'dev, 'd, A, M, ALIGN>
    where
        ALIGN: aligned::Alignment,
    {
        MscBlockDevice {
            lun: self,
            _align: PhantomData,
        }
    }
}

// --------------------------------------------------------------------------
// block-device-driver integration.
// --------------------------------------------------------------------------

/// [`block_device_driver::BlockDevice`] adapter for an [`MscLun`].
///
/// Constructed by [`MscLun::as_block_device`]. The `ALIGN` type
/// parameter picks the buffer alignment required by the caller (for
/// example the DMA alignment of the downstream consumer). The
/// `BlockDevice` trait is implemented for every `SIZE` — the LUN's
/// reported block size is validated on every call.
#[cfg(feature = "block-device-driver")]
pub struct MscBlockDevice<'lun, 'dev, 'd, A, M, ALIGN>
where
    A: UsbHostAllocator<'d>,
    M: RawMutex,
    ALIGN: aligned::Alignment,
{
    lun: &'lun mut MscLun<'dev, 'd, A, M>,
    _align: PhantomData<fn() -> ALIGN>,
}

#[cfg(feature = "block-device-driver")]
impl<'lun, 'dev, 'd, A, M, ALIGN, const SIZE: usize> block_device_driver::BlockDevice<SIZE>
    for MscBlockDevice<'lun, 'dev, 'd, A, M, ALIGN>
where
    A: UsbHostAllocator<'d>,
    M: RawMutex,
    ALIGN: aligned::Alignment,
{
    type Error = MscError;
    type Align = ALIGN;

    async fn read(
        &mut self,
        block_address: u32,
        data: &mut [aligned::Aligned<ALIGN, [u8; SIZE]>],
    ) -> Result<(), MscError> {
        let cap = self.lun.capacity().await?;
        if cap.block_size as usize != SIZE {
            return Err(MscError::BlockSizeMismatch);
        }
        let bytes = block_device_driver::blocks_to_slice_mut(data);
        self.lun.read_blocks(block_address as u64, bytes).await
    }

    async fn write(
        &mut self,
        block_address: u32,
        data: &[aligned::Aligned<ALIGN, [u8; SIZE]>],
    ) -> Result<(), MscError> {
        let cap = self.lun.capacity().await?;
        if cap.block_size as usize != SIZE {
            return Err(MscError::BlockSizeMismatch);
        }
        let bytes = block_device_driver::blocks_to_slice(data);
        self.lun.write_blocks(block_address as u64, bytes).await
    }

    async fn size(&mut self) -> Result<u64, MscError> {
        let cap = self.lun.capacity().await?;
        Ok(cap.block_count.saturating_mul(cap.block_size as u64))
    }
}

/// Pick the next chunk size.
///
/// Whenever the starting LBA fits in a `u32`, clamps the chunk to
/// `u16::MAX` blocks so [`ReadWrite::encode`] picks `READ/WRITE(10)`
/// for maximum device compatibility; only LBAs beyond the 32-bit range
/// use the 16-byte variants.
fn chunk_blocks(lba: u64, remaining: u64) -> u32 {
    let max = if lba <= u32::MAX as u64 {
        u16::MAX as u64
    } else {
        u32::MAX as u64
    };
    remaining.min(max) as u32
}

fn check_block_args(lba: u64, bytes: usize, cap: &BlockCapacity) -> Result<(usize, u64), MscError> {
    let block_size = cap.block_size as usize;
    if bytes == 0 || !bytes.is_multiple_of(block_size) {
        return Err(MscError::Unaligned);
    }
    let total_blocks = (bytes / block_size) as u64;
    if lba.checked_add(total_blocks).is_none_or(|end| end > cap.block_count) {
        return Err(MscError::OutOfRange);
    }
    Ok((block_size, total_blocks))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ----------------------------------------------------------------------
    // chunk_blocks
    // ----------------------------------------------------------------------

    #[test]
    fn chunk_blocks_prefers_read10_while_lba_fits_u32() {
        // Small chunk passes through verbatim.
        assert_eq!(chunk_blocks(0, 1), 1);
        // Clamped to u16::MAX even for huge remaining counts and high LBA.
        assert_eq!(chunk_blocks(0, u16::MAX as u64), u16::MAX as u32);
        assert_eq!(chunk_blocks(0, u64::MAX), u16::MAX as u32);
        assert_eq!(chunk_blocks(u32::MAX as u64, u64::MAX), u16::MAX as u32);
    }

    #[test]
    fn chunk_blocks_falls_back_to_read16_above_u32_max() {
        assert_eq!(chunk_blocks(u32::MAX as u64 + 1, 100), 100);
        assert_eq!(chunk_blocks(u64::MAX - 10, u64::MAX), u32::MAX);
    }

    // ----------------------------------------------------------------------
    // check_block_args
    // ----------------------------------------------------------------------

    const CAP_1K_512: BlockCapacity = BlockCapacity {
        block_count: 1000,
        block_size: 512,
    };

    #[test]
    fn check_block_args_accepts_aligned_in_range() {
        assert_eq!(check_block_args(0, 512, &CAP_1K_512).unwrap(), (512, 1));
        assert_eq!(check_block_args(0, 10 * 512, &CAP_1K_512).unwrap(), (512, 10));
        // Exact fit at end of device.
        assert_eq!(check_block_args(999, 512, &CAP_1K_512).unwrap(), (512, 1));
    }

    #[test]
    fn check_block_args_rejects_unaligned() {
        for bytes in [0, 511, 513] {
            assert!(matches!(
                check_block_args(0, bytes, &CAP_1K_512),
                Err(MscError::Unaligned)
            ));
        }
    }

    #[test]
    fn check_block_args_rejects_out_of_range() {
        for (lba, bytes) in [(1000, 512), (999, 1024), (u64::MAX, 512)] {
            assert!(matches!(
                check_block_args(lba, bytes, &CAP_1K_512),
                Err(MscError::OutOfRange)
            ));
        }
    }
}
