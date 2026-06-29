//! `embedded-fatfs` adapter over `embassy-stm32`'s `StorageDevice`, plus
//! an MBR-aware `mount` for typical Mac/Windows-formatted SD cards.

use aligned::{A4, Aligned};
use block_device_driver::BlockDevice;
use defmt::{info, warn};
use embassy_stm32::sdmmc::sd::{Addressable, Card, DataBlock, StorageDevice};
use embedded_fatfs::{DefaultTimeProvider, FileSystem, FsOptions, LossyOemCpConverter};
use embedded_io_async::ErrorKind;

pub struct EmbassyBlockDevice<'a, 'b> {
    inner: StorageDevice<'a, 'b, Card>,
}

impl<'a, 'b> EmbassyBlockDevice<'a, 'b> {
    pub fn new(inner: StorageDevice<'a, 'b, Card>) -> Self {
        Self { inner }
    }
}

#[derive(Debug)]
pub struct SdError(pub embassy_stm32::sdmmc::Error);

impl core::fmt::Display for SdError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl core::error::Error for SdError {}

impl embedded_io_async::Error for SdError {
    fn kind(&self) -> ErrorKind {
        ErrorKind::Other
    }
}

impl defmt::Format for SdError {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "SdError({:?})", defmt::Debug2Format(&self.0));
    }
}

impl<'a, 'b> BlockDevice<512> for EmbassyBlockDevice<'a, 'b> {
    type Error = SdError;
    /// `DataBlock` is `repr(align(4)) [u32; 128]`, layout-compatible with `Aligned<A4, [u8; 512]>`.
    type Align = A4;

    async fn read(
        &mut self,
        block_address: u32,
        data: &mut [Aligned<Self::Align, [u8; 512]>],
    ) -> Result<(), Self::Error> {
        let blocks: &mut [DataBlock] =
            unsafe { core::slice::from_raw_parts_mut(data.as_mut_ptr() as *mut DataBlock, data.len()) };
        self.inner.read_blocks(block_address, blocks).await.map_err(SdError)
    }

    async fn write(&mut self, block_address: u32, data: &[Aligned<Self::Align, [u8; 512]>]) -> Result<(), Self::Error> {
        let blocks: &[DataBlock] =
            unsafe { core::slice::from_raw_parts(data.as_ptr() as *const DataBlock, data.len()) };
        self.inner.write_blocks(block_address, blocks).await.map_err(SdError)
    }

    async fn size(&mut self) -> Result<u64, Self::Error> {
        Ok(self.inner.card().size())
    }
}

pub type EmbassyStream<'a, 'b> = block_device_adapters::BufStream<EmbassyBlockDevice<'a, 'b>, 512>;
pub type EmbassyFs<'a, 'b> =
    FileSystem<block_device_adapters::StreamSlice<EmbassyStream<'a, 'b>>, DefaultTimeProvider, LossyOemCpConverter>;
pub type FsError<'a, 'b> = embedded_fatfs::Error<
    block_device_adapters::StreamSliceError<
        block_device_adapters::BufStreamError<<EmbassyBlockDevice<'a, 'b> as BlockDevice<512>>::Error>,
    >,
>;

/// Mount FAT on `storage`. Reads sector 0 to detect MBR vs superfloppy
/// layout and offsets the `StreamSlice` accordingly.
pub async fn mount<'a, 'b>(storage: StorageDevice<'a, 'b, Card>) -> Result<EmbassyFs<'a, 'b>, FsError<'a, 'b>> {
    let card_bytes = storage.card().size();
    let mut block_dev = EmbassyBlockDevice::new(storage);

    let mut sec0_buf = [Aligned::<A4, _>([0u8; 512])];
    block_dev.read(0, &mut sec0_buf).await.map_err(|e| {
        embedded_fatfs::Error::Io(block_device_adapters::StreamSliceError::Other(
            block_device_adapters::BufStreamError::Io(e),
        ))
    })?;
    let sec0: &[u8; 512] = &*sec0_buf[0];

    let bps = u16::from_le_bytes([sec0[11], sec0[12]]);
    let (start_byte, end_byte) = if matches!(bps, 512 | 1024 | 2048 | 4096) {
        info!("fatfs: superfloppy layout (no MBR)");
        (0u64, card_bytes)
    } else if sec0[510] == 0x55 && sec0[511] == 0xAA {
        let mut chosen: Option<(u32, u32)> = None;
        for i in 0..4 {
            let off = 0x1BE + i * 16;
            let part_type = sec0[off + 4];
            let lba = u32::from_le_bytes([sec0[off + 8], sec0[off + 9], sec0[off + 10], sec0[off + 11]]);
            let count = u32::from_le_bytes([sec0[off + 12], sec0[off + 13], sec0[off + 14], sec0[off + 15]]);
            // Skip empty / extended-container entries.
            if part_type == 0x00 || part_type == 0x05 || part_type == 0x0F || lba == 0 || count == 0 {
                continue;
            }
            info!(
                "fatfs: MBR partition {} type=0x{:02x} lba={} sectors={}",
                i, part_type, lba, count
            );
            chosen = Some((lba, count));
            break;
        }
        let (lba, count) = chosen.ok_or(embedded_fatfs::Error::CorruptedFileSystem)?;
        (lba as u64 * 512, (lba as u64 + count as u64) * 512)
    } else {
        warn!("fatfs: sector 0 has no 0x55AA signature and no valid BPB");
        return Err(embedded_fatfs::Error::CorruptedFileSystem);
    };

    let stream = block_device_adapters::BufStream::<_, 512>::new(block_dev);
    let slice = block_device_adapters::StreamSlice::new(stream, start_byte, end_byte)
        .await
        .map_err(embedded_fatfs::Error::Io)?;
    FileSystem::new(slice, FsOptions::new()).await
}
