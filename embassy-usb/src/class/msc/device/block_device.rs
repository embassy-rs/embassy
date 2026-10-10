//! Block device adapters for [`MscClass`](super::MscClass).

use super::AsyncBlockDevice;

/// A cacheing wrapper for `AsyncBlockDevice` that aggregates writes to a device
/// with large blocks, eliminating multiple flash read-erase-write cycles.
///
/// Reads and caches one device block in `buf`, written back when required or
/// on [`flush`](AsyncBlockDevice::flush).
pub struct SectorCache<'a, D> {
    device: D,
    buf: &'a mut [u8],
    block_size: u32,
    lba: Option<u32>,
    dirty: bool,
}

impl<'a, D: AsyncBlockDevice> SectorCache<'a, D> {
    /// Cache one block of `device` in `buf`
    pub fn new(device: D, buf: &'a mut [u8], block_size: u32) -> Self {
        assert_eq!(
            buf.len(),
            device.block_size() as usize,
            "cache must hold one device block"
        );
        assert!(
            block_size > 0 && buf.len() % block_size as usize == 0,
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

    /// Host blocks per device block.
    fn per_block(&self) -> u32 {
        self.buf.len() as u32 / self.block_size
    }

    async fn write_back(&mut self) -> Result<(), D::Error> {
        if let (true, Some(lba)) = (self.dirty, self.lba) {
            self.device.write_blocks(lba, self.buf).await?;
            self.dirty = false;
        }
        Ok(())
    }

    /// Load block `lba`, returning its byte range in `buf`.
    async fn load(&mut self, lba: u32) -> Result<core::ops::Range<usize>, D::Error> {
        let (device_lba, index) = (lba / self.per_block(), lba % self.per_block());
        if self.lba != Some(device_lba) {
            self.write_back().await?;
            self.lba = None;
            self.device.read_blocks(device_lba, self.buf).await?;
            self.lba = Some(device_lba);
        }
        let start = (index * self.block_size) as usize;
        Ok(start..start + self.block_size as usize)
    }
}

impl<'a, D: AsyncBlockDevice> AsyncBlockDevice for SectorCache<'a, D> {
    type Error = D::Error;

    fn block_size(&self) -> u32 {
        self.block_size
    }

    fn block_count(&self) -> u32 {
        self.device.block_count().saturating_mul(self.per_block())
    }

    async fn read_blocks(&mut self, lba: u32, buf: &mut [u8]) -> Result<(), Self::Error> {
        for (i, block) in buf.chunks_exact_mut(self.block_size as usize).enumerate() {
            let range = self.load(lba + i as u32).await?;
            block.copy_from_slice(&self.buf[range]);
        }
        Ok(())
    }

    async fn write_blocks(&mut self, lba: u32, data: &[u8]) -> Result<(), Self::Error> {
        for (i, block) in data.chunks_exact(self.block_size as usize).enumerate() {
            let range = self.load(lba + i as u32).await?;
            self.buf[range].copy_from_slice(block);
            self.dirty = true;
        }
        Ok(())
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        self.write_back().await?;
        self.device.flush().await
    }

    fn is_write_protected(&self) -> bool {
        self.device.is_write_protected()
    }
}

/// Adapts a [`block_device_driver::BlockDevice`] with `SIZE`-byte blocks into an [`AsyncBlockDevice`].
///
/// Buffers, such as the `block_buf` given to [`MscClass::run`](super::MscClass::run), must be
/// aligned to `B::Align` or the call panics.
#[cfg(feature = "block-device-driver")]
pub struct BlockDeviceAdapter<B, const SIZE: usize> {
    device: B,
    block_count: u32,
}

#[cfg(feature = "block-device-driver")]
impl<B: block_device_driver::BlockDevice<SIZE>, const SIZE: usize> BlockDeviceAdapter<B, SIZE> {
    /// Wrap `device`, reading its size once.
    pub async fn new(mut device: B) -> Result<Self, B::Error> {
        let block_count = (device.size().await? / SIZE as u64).min(u32::MAX as u64) as u32;
        Ok(Self { device, block_count })
    }

    /// Present `block_size`-byte blocks through a [`SectorCache`] with an aligned device-block buffer.
    pub fn with_cache<'a>(
        self,
        buf: &'a mut aligned::Aligned<B::Align, [u8; SIZE]>,
        block_size: u32,
    ) -> SectorCache<'a, Self> {
        SectorCache::new(self, &mut buf[..], block_size)
    }
}

#[cfg(feature = "block-device-driver")]
impl<B: block_device_driver::BlockDevice<SIZE>, const SIZE: usize> AsyncBlockDevice for BlockDeviceAdapter<B, SIZE> {
    type Error = B::Error;

    fn block_size(&self) -> u32 {
        SIZE as u32
    }

    fn block_count(&self) -> u32 {
        self.block_count
    }

    async fn read_blocks(&mut self, lba: u32, buf: &mut [u8]) -> Result<(), Self::Error> {
        let blocks = block_device_driver::slice_to_blocks_mut(buf);
        self.device.read(lba, blocks).await
    }

    async fn write_blocks(&mut self, lba: u32, data: &[u8]) -> Result<(), Self::Error> {
        let blocks = block_device_driver::slice_to_blocks(data);
        self.device.write(lba, blocks).await
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
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

    impl AsyncBlockDevice for Ram {
        type Error = ();

        fn block_size(&self) -> u32 {
            SIZE as u32
        }

        fn block_count(&self) -> u32 {
            BLOCKS as u32
        }

        async fn read_blocks(&mut self, lba: u32, buf: &mut [u8]) -> Result<(), ()> {
            let at = lba as usize * SIZE;
            buf.copy_from_slice(&self.data[at..at + buf.len()]);
            Ok(())
        }

        async fn write_blocks(&mut self, lba: u32, data: &[u8]) -> Result<(), ()> {
            let at = lba as usize * SIZE;
            self.data[at..at + data.len()].copy_from_slice(data);
            self.writes += 1;
            Ok(())
        }

        async fn flush(&mut self) -> Result<(), ()> {
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
        let mut cache = SectorCache::new(ram(), &mut buf, HOST as u32);
        let mut model = [0u8; SIZE * BLOCKS];
        let mut io = [0u8; HOST];
        let mut seed = 0x1234_5678u32;
        let mut next = |n: usize| {
            seed ^= seed << 13;
            seed ^= seed >> 17;
            seed ^= seed << 5;
            seed as usize % n
        };

        block_on(async {
            let total = SIZE * BLOCKS / HOST;
            assert_eq!(cache.block_count(), total as u32);
            for round in 0..2000 {
                let lba = next(total);
                let range = lba * HOST..(lba + 1) * HOST;
                if next(2) == 0 {
                    for byte in io.iter_mut() {
                        *byte = next(256) as u8;
                    }
                    cache.write_blocks(lba as u32, &io).await.unwrap();
                    model[range].copy_from_slice(&io);
                } else {
                    cache.read_blocks(lba as u32, &mut io).await.unwrap();
                    assert_eq!(&io[..], &model[range], "round {round}");
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

    #[test]
    fn sector_cache_coalesces_partial_writes() {
        let mut buf = [0; SIZE];
        let mut cache = SectorCache::new(ram(), &mut buf, HOST as u32);
        block_on(async {
            for lba in 4..8 {
                cache.write_blocks(lba, &[lba as u8; HOST]).await.unwrap();
            }
            assert_eq!(cache.device.writes, 0);
            cache.flush().await.unwrap();
            assert_eq!((cache.device.writes, cache.device.flushes), (1, 1));
            cache.flush().await.unwrap();
            assert_eq!((cache.device.writes, cache.device.flushes), (1, 2));
        });
    }

    #[test]
    fn sector_cache_spans_device_blocks() {
        let mut buf = [0; SIZE];
        let mut cache = SectorCache::new(ram(), &mut buf, HOST as u32);
        let data: [u8; HOST * 6] = core::array::from_fn(|i| i as u8);
        let mut back = [0; HOST * 6];
        block_on(async {
            // Host blocks 2..8 cover the tail of device block 0 and all of device block 1.
            cache.write_blocks(2, &data).await.unwrap();
            cache.flush().await.unwrap();
            assert_eq!(&cache.device.data[2 * HOST..8 * HOST], &data[..]);
            assert_eq!(cache.device.writes, 2);
            cache.read_blocks(2, &mut back).await.unwrap();
            assert_eq!(back, data);
        });
    }

    #[cfg(feature = "block-device-driver")]
    mod adapter {
        use aligned::{A4, Aligned};

        use super::*;

        struct Blocks([u8; SIZE * BLOCKS], usize);

        impl block_device_driver::BlockDevice<SIZE> for Blocks {
            type Error = ();
            type Align = A4;

            async fn read(&mut self, lba: u32, blocks: &mut [Aligned<A4, [u8; SIZE]>]) -> Result<(), ()> {
                let at = lba as usize * SIZE;
                let bytes = block_device_driver::blocks_to_slice_mut(blocks);
                bytes.copy_from_slice(&self.0[at..at + bytes.len()]);
                self.1 += 1;
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

        #[test]
        fn reads_and_writes_through_cache() {
            let mut buf = Aligned::<A4, _>([0; SIZE]);
            block_on(async {
                let mut cache = BlockDeviceAdapter::new(Blocks([0; SIZE * BLOCKS], 0))
                    .await
                    .unwrap()
                    .with_cache(&mut buf, HOST as u32);
                assert_eq!(cache.block_count(), (SIZE * BLOCKS / HOST) as u32);
                cache.write_blocks(5, &[7; HOST]).await.unwrap();
                cache.flush().await.unwrap();
                let mut back = [0; HOST];
                cache.read_blocks(5, &mut back).await.unwrap();
                assert_eq!(back, [7; HOST]);
            });
        }

        #[test]
        fn passes_runs_of_blocks_through() {
            let mut io = Aligned::<A4, _>([0u8; SIZE * 3]);
            block_on(async {
                let mut adapter = BlockDeviceAdapter::new(Blocks([0; SIZE * BLOCKS], 0)).await.unwrap();
                adapter.write_blocks(2, &[9; SIZE * 3]).await.unwrap();
                adapter.read_blocks(2, &mut io[..]).await.unwrap();
                assert_eq!(adapter.device.1, 1);
                assert_eq!(io[..], [9; SIZE * 3]);
            });
        }

        #[test]
        #[should_panic]
        fn panics_on_unaligned_buffer() {
            let io = Aligned::<A4, _>([0u8; SIZE + 1]);
            block_on(async {
                let mut adapter = BlockDeviceAdapter::new(Blocks([0; SIZE * BLOCKS], 0)).await.unwrap();
                let _ = adapter.write_blocks(1, &io[1..]).await;
            });
        }
    }
}
