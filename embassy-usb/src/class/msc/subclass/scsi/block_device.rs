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
    /// Device-side buffer is misaligned or does not contain whole blocks.
    Unaligned,
    /// Unknown error
    Unknown,
}

/// Block storage backing a [`Scsi`](super::Scsi) logical unit.
pub trait BlockDevice {
    /// Bytes per block, reported to the host by `READ CAPACITY`.
    fn block_size(&self) -> usize;

    /// Number of blocks on the medium.
    ///
    /// Also answers `TEST UNIT READY`: return [`BlockDeviceError::MediumNotPresent`] when there is no medium.
    async fn num_blocks(&mut self) -> Result<u32, BlockDeviceError>;

    /// Read consecutive blocks starting at `lba`; `buf.len()` is a multiple of [`Self::block_size`].
    async fn read(&mut self, lba: u32, buf: &mut [u8]) -> Result<(), BlockDeviceError>;

    /// Write consecutive blocks starting at `lba`; `buf.len()` is a multiple of [`Self::block_size`].
    async fn write(&mut self, lba: u32, buf: &[u8]) -> Result<(), BlockDeviceError>;

    /// Commit any cached writes to the medium; called at the end of every `WRITE` and on `SYNCHRONIZE CACHE`.
    async fn flush(&mut self) -> Result<(), BlockDeviceError> {
        Ok(())
    }
}

/// Presents a device with large blocks, such as flash erase sectors, as smaller host blocks.
///
/// Holds one device block in `buf`: partial blocks are read-modify-written there, and written back
/// on [`flush`](BlockDevice::flush) or when another device block is needed.
pub struct SectorCache<'a, D> {
    device: D,
    buf: &'a mut [u8],
    block_size: usize,
    /// Device block held in `buf`
    lba: Option<u32>,
    dirty: bool,
}

impl<'a, D: BlockDevice> SectorCache<'a, D> {
    /// Cache one block of `device` in `buf`, presenting `block_size`-byte blocks.
    pub fn new(device: D, buf: &'a mut [u8], block_size: usize) -> Self {
        assert_eq!(buf.len(), device.block_size(), "cache must hold one device block");
        assert!(
            block_size > 0 && buf.len() % block_size == 0,
            "block size must divide the device block size"
        );
        Self {
            device,
            buf,
            block_size,
            lba: None,
            dirty: false,
        }
    }

    async fn write_back(&mut self) -> Result<(), BlockDeviceError> {
        if let (true, Some(lba)) = (self.dirty, self.lba) {
            self.device.write(lba, self.buf).await?;
            self.dirty = false;
        }
        Ok(())
    }

    /// Make `lba` the cached device block, reading it in unless it is about to be overwritten.
    async fn load(&mut self, lba: u32, fill: bool) -> Result<(), BlockDeviceError> {
        if self.lba != Some(lba) {
            self.write_back().await?;
            self.lba = None;
            if fill {
                self.device.read(lba, self.buf).await?;
            }
            self.lba = Some(lba);
        }
        Ok(())
    }

    /// Device block, offset within it, and bytes available there for a transfer at byte `pos`.
    fn locate(&self, pos: u64, remaining: usize) -> (u32, usize, usize) {
        let size = self.buf.len();
        let offset = (pos % size as u64) as usize;
        ((pos / size as u64) as u32, offset, remaining.min(size - offset))
    }
}

impl<'a, D: BlockDevice> BlockDevice for SectorCache<'a, D> {
    fn block_size(&self) -> usize {
        self.block_size
    }

    async fn num_blocks(&mut self) -> Result<u32, BlockDeviceError> {
        let per_block = (self.buf.len() / self.block_size) as u64;
        Ok((self.device.num_blocks().await? as u64 * per_block).min(u32::MAX as u64) as u32)
    }

    async fn read(&mut self, lba: u32, mut buf: &mut [u8]) -> Result<(), BlockDeviceError> {
        let mut pos = lba as u64 * self.block_size as u64;
        while !buf.is_empty() {
            let (lba, offset, len) = self.locate(pos, buf.len());
            self.load(lba, true).await?;
            let (head, rest) = core::mem::take(&mut buf).split_at_mut(len);
            head.copy_from_slice(&self.buf[offset..offset + len]);
            buf = rest;
            pos += len as u64;
        }
        Ok(())
    }

    async fn write(&mut self, lba: u32, mut buf: &[u8]) -> Result<(), BlockDeviceError> {
        let mut pos = lba as u64 * self.block_size as u64;
        while !buf.is_empty() {
            let (lba, offset, len) = self.locate(pos, buf.len());
            self.load(lba, len < self.buf.len()).await?;
            self.buf[offset..offset + len].copy_from_slice(&buf[..len]);
            self.dirty = true;
            buf = &buf[len..];
            pos += len as u64;
        }
        Ok(())
    }

    async fn flush(&mut self) -> Result<(), BlockDeviceError> {
        self.write_back().await?;
        self.device.flush().await
    }
}

/// Adapts a [`block_device_driver::BlockDevice`] with `SIZE`-byte blocks into a [`BlockDevice`].
///
/// Direct reads and writes require buffers aligned to `B::Align` and sized in whole device blocks.
/// [`with_cache`](Self::with_cache) supplies a typed, aligned buffer for smaller blocks.
#[cfg(feature = "block-device-driver")]
pub struct BlockDeviceAdapter<B, const SIZE: usize> {
    device: B,
}

#[cfg(feature = "block-device-driver")]
impl<B: block_device_driver::BlockDevice<SIZE>, const SIZE: usize> BlockDeviceAdapter<B, SIZE> {
    /// Wrap `device`.
    pub fn new(device: B) -> Self {
        Self { device }
    }

    /// Present `block_size`-byte blocks through a [`SectorCache`] with an aligned device-block buffer.
    pub fn with_cache<'a>(
        self,
        buf: &'a mut aligned::Aligned<B::Align, [u8; SIZE]>,
        block_size: usize,
    ) -> SectorCache<'a, Self> {
        SectorCache::new(self, &mut buf[..], block_size)
    }
}

#[cfg(feature = "block-device-driver")]
impl<B: block_device_driver::BlockDevice<SIZE>, const SIZE: usize> BlockDevice for BlockDeviceAdapter<B, SIZE> {
    fn block_size(&self) -> usize {
        SIZE
    }

    async fn num_blocks(&mut self) -> Result<u32, BlockDeviceError> {
        let size = self.device.size().await.map_err(|_| {
            error!("block device size failed");
            BlockDeviceError::MediumNotPresent
        })?;
        Ok((size / SIZE as u64).min(u32::MAX as u64) as u32)
    }

    async fn read(&mut self, lba: u32, buf: &mut [u8]) -> Result<(), BlockDeviceError> {
        check_blocks::<B::Align, SIZE>(buf)?;
        let blocks = block_device_driver::slice_to_blocks_mut::<B::Align, SIZE>(buf);
        self.device.read(lba, blocks).await.map_err(|_| {
            error!("block device read failed");
            BlockDeviceError::ReadError
        })
    }

    async fn write(&mut self, lba: u32, buf: &[u8]) -> Result<(), BlockDeviceError> {
        check_blocks::<B::Align, SIZE>(buf)?;
        let blocks = block_device_driver::slice_to_blocks::<B::Align, SIZE>(buf);
        self.device.write(lba, blocks).await.map_err(|_| {
            error!("block device write failed");
            BlockDeviceError::WriteError
        })
    }
}

#[cfg(feature = "block-device-driver")]
fn check_blocks<A, const SIZE: usize>(buf: &[u8]) -> Result<(), BlockDeviceError> {
    let align = core::mem::align_of::<A>();
    if buf.len() % SIZE != 0 || buf.len() % align != 0 || buf.as_ptr() as usize % align != 0 {
        error!(
            "block device buffer is not aligned to {} or a multiple of {}",
            align, SIZE
        );
        return Err(BlockDeviceError::Unaligned);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use embassy_futures::block_on;

    use super::*;

    const SIZE: usize = 64;
    const HOST: usize = 16;
    const BLOCKS: usize = 8;

    struct Ram {
        data: [u8; SIZE * BLOCKS],
        writes: usize,
        flushes: usize,
    }

    impl BlockDevice for Ram {
        fn block_size(&self) -> usize {
            SIZE
        }

        async fn num_blocks(&mut self) -> Result<u32, BlockDeviceError> {
            Ok(BLOCKS as u32)
        }

        async fn read(&mut self, lba: u32, buf: &mut [u8]) -> Result<(), BlockDeviceError> {
            let at = lba as usize * SIZE;
            buf.copy_from_slice(&self.data[at..at + buf.len()]);
            Ok(())
        }

        async fn write(&mut self, lba: u32, buf: &[u8]) -> Result<(), BlockDeviceError> {
            let at = lba as usize * SIZE;
            self.data[at..at + buf.len()].copy_from_slice(buf);
            self.writes += buf.len() / SIZE;
            Ok(())
        }

        async fn flush(&mut self) -> Result<(), BlockDeviceError> {
            self.flushes += 1;
            Ok(())
        }
    }

    fn ram() -> Ram {
        Ram {
            data: [0; SIZE * BLOCKS],
            writes: 0,
            flushes: 0,
        }
    }

    #[test]
    fn sector_cache_matches_model() {
        let mut buf = [0; SIZE];
        let mut cache = SectorCache::new(ram(), &mut buf, HOST);
        let mut model = [0u8; SIZE * BLOCKS];
        let mut io = [0u8; SIZE * BLOCKS];
        let mut seed = 0x1234_5678u32;
        let mut next = |n: usize| {
            seed ^= seed << 13;
            seed ^= seed >> 17;
            seed ^= seed << 5;
            seed as usize % n
        };

        block_on(async {
            assert_eq!(cache.num_blocks().await, Ok((SIZE * BLOCKS / HOST) as u32));
            for round in 0..2000 {
                let total = SIZE * BLOCKS / HOST;
                let lba = next(total);
                let count = 1 + next(total - lba);
                let buf = &mut io[..count * HOST];
                let range = lba * HOST..(lba + count) * HOST;
                if next(2) == 0 {
                    for byte in buf.iter_mut() {
                        *byte = next(256) as u8;
                    }
                    cache.write(lba as u32, buf).await.unwrap();
                    model[range].copy_from_slice(buf);
                } else {
                    cache.read(lba as u32, buf).await.unwrap();
                    assert_eq!(&buf[..], &model[range], "round {round}");
                }
                if next(4) == 0 {
                    cache.flush().await.unwrap();
                    assert_eq!(cache.device.data, model, "round {round}");
                }
            }
            cache.flush().await.unwrap();
            assert_eq!(cache.device.data, model);
        });
    }

    #[cfg(feature = "block-device-driver")]
    #[test]
    fn adapter_rejects_unaligned_buffers() {
        use aligned::{A4, Aligned};

        struct Blocks([u8; SIZE * BLOCKS]);

        impl block_device_driver::BlockDevice<SIZE> for Blocks {
            type Error = ();
            type Align = A4;

            async fn read(&mut self, lba: u32, blocks: &mut [Aligned<A4, [u8; SIZE]>]) -> Result<(), ()> {
                let at = lba as usize * SIZE;
                let bytes = block_device_driver::blocks_to_slice_mut(blocks);
                bytes.copy_from_slice(&self.0[at..at + bytes.len()]);
                Ok(())
            }

            async fn write(&mut self, lba: u32, blocks: &[Aligned<A4, [u8; SIZE]>]) -> Result<(), ()> {
                let at = lba as usize * SIZE;
                let bytes = block_device_driver::blocks_to_slice(blocks);
                self.0[at..at + bytes.len()].copy_from_slice(bytes);
                Ok(())
            }

            async fn size(&mut self) -> Result<u64, ()> {
                Ok((SIZE * BLOCKS) as u64)
            }
        }

        let mut adapter = BlockDeviceAdapter::new(Blocks([0; SIZE * BLOCKS]));
        let mut io: Aligned<A4, [u8; SIZE * 2 + 4]> = Aligned([7; SIZE * 2 + 4]);
        block_on(async {
            assert_eq!(adapter.num_blocks().await, Ok(BLOCKS as u32));
            adapter.write(1, &io[..SIZE * 2]).await.unwrap();
            io.fill(0);
            adapter.read(1, &mut io[..SIZE * 2]).await.unwrap();
            assert_eq!(io[..SIZE * 2], [7; SIZE * 2]);
            assert_eq!(
                adapter.write(1, &io[1..SIZE + 1]).await,
                Err(BlockDeviceError::Unaligned)
            );
            assert_eq!(
                adapter.read(1, &mut io[1..SIZE + 1]).await,
                Err(BlockDeviceError::Unaligned)
            );
            assert_eq!(
                adapter.read(1, &mut io[..SIZE + 4]).await,
                Err(BlockDeviceError::Unaligned)
            );
        });

        let mut buf = Aligned::<A4, _>([0; SIZE]);
        let mut cache = BlockDeviceAdapter::new(Blocks([0; SIZE * BLOCKS])).with_cache(&mut buf, HOST);
        block_on(async {
            assert_eq!(cache.num_blocks().await, Ok((SIZE * BLOCKS / HOST) as u32));
            cache.write(5, &io[1..HOST * 3 + 1]).await.unwrap();
            cache.flush().await.unwrap();
            let mut back = [0; HOST * 3];
            cache.read(5, &mut back).await.unwrap();
            assert_eq!(back[..], io[1..HOST * 3 + 1]);
        });
    }

    #[test]
    fn sector_cache_coalesces_partial_writes() {
        let mut buf = [0; SIZE];
        let mut cache = SectorCache::new(ram(), &mut buf, HOST);
        block_on(async {
            for lba in 4..8 {
                cache.write(lba, &[lba as u8; HOST]).await.unwrap();
            }
            assert_eq!(cache.device.writes, 0);
            cache.flush().await.unwrap();
            assert_eq!((cache.device.writes, cache.device.flushes), (1, 1));
            cache.flush().await.unwrap();
            assert_eq!((cache.device.writes, cache.device.flushes), (1, 2));
        });
    }
}
