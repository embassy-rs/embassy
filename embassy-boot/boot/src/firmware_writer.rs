use embedded_storage::nor_flash::NorFlash;
use embedded_storage_async::nor_flash::NorFlash as AsyncNorFlash;

use crate::Partition;

/// FirmwareWriter allows writing blocks to an already erased flash.
pub struct FirmwareWriter(pub(crate) Partition);

impl FirmwareWriter {
    /// Write data to a flash page.
    ///
    /// The buffer must follow alignment requirements of the target flash and a multiple of page size big.
    ///
    /// # Safety
    ///
    /// Failing to meet alignment and size requirements may result in a panic.
    pub async fn write_block<F: AsyncNorFlash>(
        &mut self,
        offset: usize,
        data: &[u8],
        flash: &mut F,
        block_size: usize,
    ) -> Result<(), F::Error> {
        trace!(
            "Writing firmware at offset 0x{:x} len {}",
            self.0.from + offset,
            data.len()
        );

        let mut write_offset = self.0.from + offset;
        for chunk in data.chunks(block_size) {
            trace!("Wrote chunk at {}: {:?}", write_offset, chunk);
            flash.write(write_offset as u32, chunk).await?;
            write_offset += chunk.len();
        }
        /*
        trace!("Wrote data, reading back for verification");

        let mut buf: [u8; 4096] = [0; 4096];
        let mut data_offset = 0;
        let mut read_offset = self.dfu.from + offset;
        for chunk in buf.chunks_mut(block_size) {
            flash.read(read_offset as u32, chunk).await?;
            trace!("Read chunk at {}: {:?}", read_offset, chunk);
            assert_eq!(&data[data_offset..data_offset + block_size], chunk);
            read_offset += chunk.len();
            data_offset += chunk.len();
        }
        */

        Ok(())
    }

    /// Write data to a flash page.
    ///
    /// The buffer must follow alignment requirements of the target flash and a multiple of page size big.
    ///
    /// # Safety
    ///
    /// Failing to meet alignment and size requirements may result in a panic.
    pub fn write_block_blocking<F: NorFlash>(
        &mut self,
        offset: usize,
        data: &[u8],
        flash: &mut F,
        block_size: usize,
    ) -> Result<(), F::Error> {
        trace!(
            "Writing firmware at offset 0x{:x} len {}",
            self.0.from + offset,
            data.len()
        );

        let mut write_offset = self.0.from + offset;
        for chunk in data.chunks(block_size) {
            trace!("Wrote chunk at {}: {:?}", write_offset, chunk);
            flash.write(write_offset as u32, chunk)?;
            write_offset += chunk.len();
        }
        /*
        trace!("Wrote data, reading back for verification");

        let mut buf: [u8; 4096] = [0; 4096];
        let mut data_offset = 0;
        let mut read_offset = self.dfu.from + offset;
        for chunk in buf.chunks_mut(block_size) {
            flash.read(read_offset as u32, chunk).await?;
            trace!("Read chunk at {}: {:?}", read_offset, chunk);
            assert_eq!(&data[data_offset..data_offset + block_size], chunk);
            read_offset += chunk.len();
            data_offset += chunk.len();
        }
        */

        Ok(())
    }
}
