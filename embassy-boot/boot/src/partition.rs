use embedded_storage::nor_flash::{NorFlash, ReadNorFlash};
use embedded_storage_async::nor_flash::{NorFlash as AsyncNorFlash, ReadNorFlash as AsyncReadNorFlash};

/// A region in flash used by the bootloader.
#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Partition {
    /// The offset into the flash where the partition starts.
    pub from: u32,
    /// The offset into the flash where the partition ends.
    pub to: u32,
}

impl Partition {
    /// Create a new partition with the provided range
    pub const fn new(from: u32, to: u32) -> Self {
        Self { from, to }
    }

    /// Return the size of the partition
    pub const fn size(&self) -> u32 {
        self.to - self.from
    }

    /// Read from the partition on the provided flash
    pub async fn read<F: AsyncReadNorFlash>(
        &self,
        flash: &mut F,
        offset: u32,
        bytes: &mut [u8],
    ) -> Result<(), F::Error> {
        let offset = self.from as u32 + offset;
        flash.read(offset, bytes).await
    }

    /// Write to the partition on the provided flash
    pub async fn write<F: AsyncNorFlash>(&self, flash: &mut F, offset: u32, bytes: &[u8]) -> Result<(), F::Error> {
        let offset = self.from as u32 + offset;
        flash.write(offset, bytes).await?;
        trace!("Wrote from 0x{:x} len {}", offset, bytes.len());
        Ok(())
    }

    /// Erase part of the partition on the provided flash
    pub async fn erase<F: AsyncNorFlash>(&self, flash: &mut F, from: u32, to: u32) -> Result<(), F::Error> {
        let from = self.from as u32 + from;
        let to = self.from as u32 + to;
        flash.erase(from, to).await?;
        trace!("Erased from 0x{:x} to 0x{:x}", from, to);
        Ok(())
    }

    /// Erase the entire partition
    pub(crate) async fn wipe<F: AsyncNorFlash>(&self, flash: &mut F) -> Result<(), F::Error> {
        let from = self.from as u32;
        let to = self.to as u32;
        flash.erase(from, to).await?;
        trace!("Wiped from 0x{:x} to 0x{:x}", from, to);
        Ok(())
    }

    /// Read from the partition on the provided flash
    pub fn read_blocking<F: ReadNorFlash>(&self, flash: &mut F, offset: u32, bytes: &mut [u8]) -> Result<(), F::Error> {
        let offset = self.from as u32 + offset;
        flash.read(offset, bytes)
    }

    /// Write to the partition on the provided flash
    pub fn write_blocking<F: NorFlash>(&self, flash: &mut F, offset: u32, bytes: &[u8]) -> Result<(), F::Error> {
        let offset = self.from as u32 + offset;
        flash.write(offset, bytes)?;
        trace!("Wrote from 0x{:x} len {}", offset, bytes.len());
        Ok(())
    }

    /// Erase part of the partition on the provided flash
    pub fn erase_blocking<F: NorFlash>(&self, flash: &mut F, from: u32, to: u32) -> Result<(), F::Error> {
        let from = self.from as u32 + from;
        let to = self.from as u32 + to;
        flash.erase(from, to)?;
        trace!("Erased from 0x{:x} to 0x{:x}", from, to);
        Ok(())
    }

    /// Erase the entire partition
    pub(crate) fn wipe_blocking<F: NorFlash>(&self, flash: &mut F) -> Result<(), F::Error> {
        let from = self.from as u32;
        let to = self.to as u32;
        flash.erase(from, to)?;
        trace!("Wiped from 0x{:x} to 0x{:x}", from, to);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::mem_flash::MemFlash;
    use crate::Partition;

    #[test]
    fn can_erase() {
        let mut flash = MemFlash::<1024, 64, 4>::new(0x00);
        let partition = Partition::new(256, 512);

        partition.erase_blocking(&mut flash, 64, 192).unwrap();

        for (index, byte) in flash.mem.iter().copied().enumerate().take(256 + 64) {
            assert_eq!(0x00, byte, "Index {}", index);
        }

        for (index, byte) in flash.mem.iter().copied().enumerate().skip(256 + 64).take(128) {
            assert_eq!(0xFF, byte, "Index {}", index);
        }

        for (index, byte) in flash.mem.iter().copied().enumerate().skip(256 + 64 + 128) {
            assert_eq!(0x00, byte, "Index {}", index);
        }
    }

    #[test]
    fn can_wipe() {
        let mut flash = MemFlash::<1024, 64, 4>::new(0x00);
        let partition = Partition::new(256, 512);

        partition.wipe_blocking(&mut flash).unwrap();

        for (index, byte) in flash.mem.iter().copied().enumerate().take(256) {
            assert_eq!(0x00, byte, "Index {}", index);
        }

        for (index, byte) in flash.mem.iter().copied().enumerate().skip(256).take(256) {
            assert_eq!(0xFF, byte, "Index {}", index);
        }

        for (index, byte) in flash.mem.iter().copied().enumerate().skip(512) {
            assert_eq!(0x00, byte, "Index {}", index);
        }
    }
}
