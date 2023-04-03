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
        let mut offset = offset as u32;
        for chunk in data.chunks(block_size) {
            self.0.write(flash, offset, chunk).await?;
            offset += chunk.len() as u32;
        }

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
        let mut offset = offset as u32;
        for chunk in data.chunks(block_size) {
            self.0.write_blocking(flash, offset, chunk)?;
            offset += chunk.len() as u32;
        }

        Ok(())
    }
}
