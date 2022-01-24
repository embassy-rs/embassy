#![feature(type_alias_impl_trait)]
#![feature(generic_associated_types)]
#![no_std]
///! embassy-boot is a bootloader and firmware updater for embedded devices with flash
///! storage implemented using embedded-storage
///!
///! The bootloader works in conjunction with the firmware application, and only has the
///! ability to manage two flash banks with an active and a updatable part. It implements
///! a swap algorithm that is power-failure safe, and allows reverting to the previous
///! version of the firmware, should the application crash and fail to mark itself as booted.
///!
///! This library is intended to be used by platform-specific bootloaders, such as embassy-boot-nrf,
///! which defines the limits and flash type for that particular platform.
///!
mod fmt;

use embedded_storage::nor_flash::{NorFlash, ReadNorFlash};
use embedded_storage_async::nor_flash::AsyncNorFlash;

pub const BOOT_MAGIC: u32 = 0xD00DF00D;
pub const SWAP_MAGIC: u32 = 0xF00FDAAD;

#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Partition {
    pub from: usize,
    pub to: usize,
}

impl Partition {
    pub const fn new(from: usize, to: usize) -> Self {
        Self { from, to }
    }
    pub const fn len(&self) -> usize {
        self.to - self.from
    }
}

#[derive(PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum State {
    Boot,
    Swap,
}

#[derive(PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum BootError<E> {
    Flash(E),
    BadMagic,
}

impl<E> From<E> for BootError<E> {
    fn from(error: E) -> Self {
        BootError::Flash(error)
    }
}

/// BootLoader works with any flash implementing embedded_storage and can also work with
/// different page sizes.
pub struct BootLoader<const PAGE_SIZE: usize> {
    // Page with current state of bootloader. The state partition has the following format:
    // | Range    | Description                                                                                        |
    // | 0 - 4    | Magic indicating bootloader state. BOOT_MAGIC means boot, SWAP_MAGIC means swap. |
    // | 4 - N    | Progress index used while swapping or reverting                                                    |
    state: Partition,
    // Location of the partition which will be booted from
    active: Partition,
    // Location of the partition which will be swapped in when requested
    dfu: Partition,
}

impl<const PAGE_SIZE: usize> BootLoader<PAGE_SIZE> {
    pub fn new(active: Partition, dfu: Partition, state: Partition) -> Self {
        assert_eq!(active.len() % PAGE_SIZE, 0);
        assert_eq!(dfu.len() % PAGE_SIZE, 0);
        // DFU partition must have an extra page
        assert!(dfu.len() - active.len() >= PAGE_SIZE);
        // Ensure we have enough progress pages to store copy progress
        assert!(active.len() / PAGE_SIZE >= (state.len() - 4) / PAGE_SIZE);
        Self { active, dfu, state }
    }

    pub fn boot_address(&self) -> usize {
        self.active.from
    }

    /// Perform necessary boot preparations like swapping images.
    ///
    /// The DFU partition is assumed to be 1 page bigger than the active partition for the swap
    /// algorithm to work correctly.
    ///
    /// SWAPPING
    ///
    /// Assume a flash size of 3 pages for the active partition, and 4 pages for the DFU partition.
    /// The swap index contains the copy progress, as to allow continuation of the copy process on
    /// power failure. The index counter is represented within 1 or more pages (depending on total
    /// flash size), where a page X is considered swapped if index at location (X + WRITE_SIZE)
    /// contains a zero value. This ensures that index updates can be performed atomically and
    /// avoid a situation where the wrong index value is set (page write size is "atomic").
    ///
    /// +-----------+------------+--------+--------+--------+--------+
    /// | Partition | Swap Index | Page 0 | Page 1 | Page 3 | Page 4 |
    /// +-----------+------------+--------+--------+--------+--------+
    /// |    Active |          0 |      1 |      2 |      3 |      - |
    /// |       DFU |          0 |      3 |      2 |      1 |      X |
    /// +-----------+-------+--------+--------+--------+--------+
    ///
    /// The algorithm starts by copying 'backwards', and after the first step, the layout is
    /// as follows:
    ///
    /// +-----------+------------+--------+--------+--------+--------+
    /// | Partition | Swap Index | Page 0 | Page 1 | Page 3 | Page 4 |
    /// +-----------+------------+--------+--------+--------+--------+
    /// |    Active |          1 |      1 |      2 |      1 |      - |
    /// |       DFU |          1 |      3 |      2 |      1 |      3 |
    /// +-----------+------------+--------+--------+--------+--------+
    ///
    /// The next iteration performs the same steps
    ///
    /// +-----------+------------+--------+--------+--------+--------+
    /// | Partition | Swap Index | Page 0 | Page 1 | Page 3 | Page 4 |
    /// +-----------+------------+--------+--------+--------+--------+
    /// |    Active |          2 |      1 |      2 |      1 |      - |
    /// |       DFU |          2 |      3 |      2 |      2 |      3 |
    /// +-----------+------------+--------+--------+--------+--------+
    ///
    /// And again until we're done
    ///
    /// +-----------+------------+--------+--------+--------+--------+
    /// | Partition | Swap Index | Page 0 | Page 1 | Page 3 | Page 4 |
    /// +-----------+------------+--------+--------+--------+--------+
    /// |    Active |          3 |      3 |      2 |      1 |      - |
    /// |       DFU |          3 |      3 |      1 |      2 |      3 |
    /// +-----------+------------+--------+--------+--------+--------+
    ///
    /// REVERTING
    ///
    /// The reverting algorithm uses the swap index to discover that images were swapped, but that
    /// the application failed to mark the boot successful. In this case, the revert algorithm will
    /// run.
    ///
    /// The revert index is located separately from the swap index, to ensure that revert can continue
    /// on power failure.
    ///
    /// The revert algorithm works forwards, by starting copying into the 'unused' DFU page at the start.
    ///
    /// +-----------+--------------+--------+--------+--------+--------+
    /// | Partition | Revert Index | Page 0 | Page 1 | Page 3 | Page 4 |
    //*/
    /// +-----------+--------------+--------+--------+--------+--------+
    /// |    Active |            3 |      1 |      2 |      1 |      - |
    /// |       DFU |            3 |      3 |      1 |      2 |      3 |
    /// +-----------+--------------+--------+--------+--------+--------+
    ///
    ///
    /// +-----------+--------------+--------+--------+--------+--------+
    /// | Partition | Revert Index | Page 0 | Page 1 | Page 3 | Page 4 |
    /// +-----------+--------------+--------+--------+--------+--------+
    /// |    Active |            3 |      1 |      2 |      1 |      - |
    /// |       DFU |            3 |      3 |      2 |      2 |      3 |
    /// +-----------+--------------+--------+--------+--------+--------+
    ///
    /// +-----------+--------------+--------+--------+--------+--------+
    /// | Partition | Revert Index | Page 0 | Page 1 | Page 3 | Page 4 |
    /// +-----------+--------------+--------+--------+--------+--------+
    /// |    Active |            3 |      1 |      2 |      3 |      - |
    /// |       DFU |            3 |      3 |      2 |      1 |      3 |
    /// +-----------+--------------+--------+--------+--------+--------+
    ///
    pub fn prepare_boot<F: NorFlash + ReadNorFlash>(
        &mut self,
        flash: &mut F,
    ) -> Result<State, BootError<F::Error>> {
        // Copy contents from partition N to active
        let state = self.read_state(flash)?;
        match state {
            State::Swap => {
                //
                // Check if we already swapped. If we're in the swap state, this means we should revert
                // since the app has failed to mark boot as successful
                //
                if !self.is_swapped(flash)? {
                    trace!("Swapping");
                    self.swap(flash)?;
                } else {
                    trace!("Reverting");
                    self.revert(flash)?;

                    // Overwrite magic and reset progress
                    flash.write(self.state.from as u32, &[0, 0, 0, 0])?;
                    flash.erase(self.state.from as u32, self.state.to as u32)?;
                    flash.write(self.state.from as u32, &BOOT_MAGIC.to_le_bytes())?;
                }
            }
            _ => {}
        }
        Ok(state)
    }

    fn is_swapped<F: ReadNorFlash>(&mut self, flash: &mut F) -> Result<bool, F::Error> {
        let page_count = self.active.len() / PAGE_SIZE;
        let progress = self.current_progress(flash)?;

        Ok(progress >= page_count * 2)
    }

    fn current_progress<F: ReadNorFlash>(&mut self, flash: &mut F) -> Result<usize, F::Error> {
        let max_index = ((self.state.len() - 4) / 4) - 1;
        for i in 0..max_index {
            let mut buf: [u8; 4] = [0; 4];
            flash.read((self.state.from + 4 + i * 4) as u32, &mut buf)?;
            if buf == [0xFF, 0xFF, 0xFF, 0xFF] {
                return Ok(i);
            }
        }
        Ok(max_index)
    }

    fn update_progress<F: NorFlash>(&mut self, idx: usize, flash: &mut F) -> Result<(), F::Error> {
        let w = self.state.from + 4 + idx * 4;
        flash.write(w as u32, &[0, 0, 0, 0])?;
        Ok(())
    }

    fn active_addr(&self, n: usize) -> usize {
        self.active.from + n * PAGE_SIZE
    }

    fn dfu_addr(&self, n: usize) -> usize {
        self.dfu.from + n * PAGE_SIZE
    }

    fn copy_page_once<F: NorFlash + ReadNorFlash>(
        &mut self,
        idx: usize,
        from: usize,
        to: usize,
        flash: &mut F,
    ) -> Result<(), F::Error> {
        let mut buf: [u8; PAGE_SIZE] = [0; PAGE_SIZE];
        if self.current_progress(flash)? <= idx {
            flash.read(from as u32, &mut buf)?;
            flash.erase(to as u32, (to + PAGE_SIZE) as u32)?;
            flash.write(to as u32, &buf)?;
            self.update_progress(idx, flash)?;
        }
        Ok(())
    }

    fn swap<F: NorFlash + ReadNorFlash>(&mut self, flash: &mut F) -> Result<(), F::Error> {
        let page_count = self.active.len() / PAGE_SIZE;
        // trace!("Page count: {}", page_count);
        for page in 0..page_count {
            // Copy active page to the 'next' DFU page.
            let active_page = self.active_addr(page_count - 1 - page);
            let dfu_page = self.dfu_addr(page_count - page);
            // info!("Copy active {} to dfu {}", active_page, dfu_page);
            self.copy_page_once(page * 2, active_page, dfu_page, flash)?;

            // Copy DFU page to the active page
            let active_page = self.active_addr(page_count - 1 - page);
            let dfu_page = self.dfu_addr(page_count - 1 - page);
            //info!("Copy dfy {} to active {}", dfu_page, active_page);
            self.copy_page_once(page * 2 + 1, dfu_page, active_page, flash)?;
        }

        Ok(())
    }

    fn revert<F: NorFlash + ReadNorFlash>(&mut self, flash: &mut F) -> Result<(), F::Error> {
        let page_count = self.active.len() / PAGE_SIZE;
        for page in 0..page_count {
            // Copy the bad active page to the DFU page
            let active_page = self.active_addr(page);
            let dfu_page = self.dfu_addr(page);
            self.copy_page_once(page_count * 2 + page * 2, active_page, dfu_page, flash)?;

            // Copy the DFU page back to the active page
            let active_page = self.active_addr(page);
            let dfu_page = self.dfu_addr(page + 1);
            self.copy_page_once(page_count * 2 + page * 2 + 1, dfu_page, active_page, flash)?;
        }

        Ok(())
    }

    fn read_state<F: ReadNorFlash>(&mut self, flash: &mut F) -> Result<State, BootError<F::Error>> {
        let mut magic: [u8; 4] = [0; 4];
        flash.read(self.state.from as u32, &mut magic)?;

        match u32::from_le_bytes(magic) {
            SWAP_MAGIC => Ok(State::Swap),
            _ => Ok(State::Boot),
        }
    }
}

/// FirmwareUpdater is an application API for interacting with the BootLoader without the ability to
/// 'mess up' the internal bootloader state
pub struct FirmwareUpdater {
    state: Partition,
    dfu: Partition,
}

impl FirmwareUpdater {
    pub const fn new(dfu: Partition, state: Partition) -> Self {
        Self { dfu, state }
    }

    /// Return the length of the DFU area
    pub fn firmware_len(&self) -> usize {
        self.dfu.len()
    }

    /// Instruct bootloader that DFU should commence at next boot.
    pub async fn mark_update<F: AsyncNorFlash>(&mut self, flash: &mut F) -> Result<(), F::Error> {
        flash.write(self.state.from as u32, &[0, 0, 0, 0]).await?;
        flash
            .erase(self.state.from as u32, self.state.to as u32)
            .await?;
        info!(
            "Setting swap magic at {} to 0x{:x}, LE: 0x{:x}",
            self.state.from,
            &SWAP_MAGIC,
            &SWAP_MAGIC.to_le_bytes()
        );
        flash
            .write(self.state.from as u32, &SWAP_MAGIC.to_le_bytes())
            .await?;
        Ok(())
    }

    /// Mark firmware boot successfully
    pub async fn mark_booted<F: AsyncNorFlash>(&mut self, flash: &mut F) -> Result<(), F::Error> {
        flash.write(self.state.from as u32, &[0, 0, 0, 0]).await?;
        flash
            .erase(self.state.from as u32, self.state.to as u32)
            .await?;
        flash
            .write(self.state.from as u32, &BOOT_MAGIC.to_le_bytes())
            .await?;
        Ok(())
    }

    // Write to a region of the DFU page
    pub async fn write_firmware<F: AsyncNorFlash>(
        &mut self,
        offset: usize,
        data: &[u8],
        flash: &mut F,
    ) -> Result<(), F::Error> {
        info!(
            "Writing firmware at offset 0x{:x} len {}",
            self.dfu.from + offset,
            data.len()
        );

        flash
            .erase(
                (self.dfu.from + offset) as u32,
                (self.dfu.from + offset + data.len()) as u32,
            )
            .await?;
        flash.write((self.dfu.from + offset) as u32, data).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::convert::Infallible;
    use core::future::Future;
    use embedded_storage_async::nor_flash::AsyncReadNorFlash;
    use futures::executor::block_on;

    const STATE: Partition = Partition::new(0, 4096);
    const ACTIVE: Partition = Partition::new(4096, 61440);
    const DFU: Partition = Partition::new(61440, 122880);

    #[test]
    fn test_bad_magic() {
        let mut flash = MemFlash([0xff; 131072]);

        let mut bootloader = BootLoader::<4096>::new(ACTIVE, DFU, STATE);

        assert_eq!(
            bootloader.prepare_boot(&mut flash),
            Err(BootError::BadMagic)
        );
    }

    #[test]
    fn test_boot_state() {
        let mut flash = MemFlash([0xff; 131072]);
        flash.0[0..4].copy_from_slice(&BOOT_MAGIC.to_le_bytes());

        let mut bootloader = BootLoader::<4096>::new(ACTIVE, DFU, STATE);

        assert_eq!(State::Boot, bootloader.prepare_boot(&mut flash).unwrap());
    }

    #[test]
    fn test_swap_state() {
        env_logger::init();
        let mut flash = MemFlash([0xff; 131072]);

        let original: [u8; ACTIVE.len()] = [rand::random::<u8>(); ACTIVE.len()];
        let update: [u8; DFU.len()] = [rand::random::<u8>(); DFU.len()];

        for i in ACTIVE.from..ACTIVE.to {
            flash.0[i] = original[i - ACTIVE.from];
        }

        let mut bootloader = BootLoader::<4096>::new(ACTIVE, DFU, STATE);
        let mut updater = FirmwareUpdater::new(DFU, STATE);
        for i in (DFU.from..DFU.to).step_by(4) {
            let base = i - DFU.from;
            let data: [u8; 4] = [
                update[base],
                update[base + 1],
                update[base + 2],
                update[base + 3],
            ];
            block_on(updater.write_firmware(i - DFU.from, &data, &mut flash)).unwrap();
        }
        block_on(updater.mark_update(&mut flash)).unwrap();

        assert_eq!(State::Swap, bootloader.prepare_boot(&mut flash).unwrap());

        for i in ACTIVE.from..ACTIVE.to {
            assert_eq!(flash.0[i], update[i - ACTIVE.from], "Index {}", i);
        }

        // First DFU page is untouched
        for i in DFU.from + 4096..DFU.to {
            assert_eq!(flash.0[i], original[i - DFU.from - 4096], "Index {}", i);
        }

        // Running again should cause a revert
        assert_eq!(State::Swap, bootloader.prepare_boot(&mut flash).unwrap());

        for i in ACTIVE.from..ACTIVE.to {
            assert_eq!(flash.0[i], original[i - ACTIVE.from], "Index {}", i);
        }

        // Last page is untouched
        for i in DFU.from..DFU.to - 4096 {
            assert_eq!(flash.0[i], update[i - DFU.from], "Index {}", i);
        }

        // Mark as booted
        block_on(updater.mark_booted(&mut flash)).unwrap();
        assert_eq!(State::Boot, bootloader.prepare_boot(&mut flash).unwrap());
    }

    struct MemFlash([u8; 131072]);

    impl NorFlash for MemFlash {
        const WRITE_SIZE: usize = 4;
        const ERASE_SIZE: usize = 4096;
        fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
            let from = from as usize;
            let to = to as usize;
            for i in from..to {
                self.0[i] = 0xFF;
                self.0[i] = 0xFF;
                self.0[i] = 0xFF;
                self.0[i] = 0xFF;
            }
            Ok(())
        }

        fn write(&mut self, offset: u32, data: &[u8]) -> Result<(), Self::Error> {
            assert!(data.len() % 4 == 0);
            assert!(offset % 4 == 0);
            assert!(offset as usize + data.len() < 131072);

            self.0[offset as usize..offset as usize + data.len()].copy_from_slice(data);

            Ok(())
        }
    }

    impl ReadNorFlash for MemFlash {
        const READ_SIZE: usize = 4;
        type Error = Infallible;

        fn read(&mut self, offset: u32, buf: &mut [u8]) -> Result<(), Self::Error> {
            let len = buf.len();
            buf[..].copy_from_slice(&self.0[offset as usize..offset as usize + len]);
            Ok(())
        }

        fn capacity(&self) -> usize {
            131072
        }
    }

    impl AsyncReadNorFlash for MemFlash {
        const READ_SIZE: usize = 4;
        type Error = Infallible;

        type ReadFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a;
        fn read<'a>(&'a mut self, offset: usize, buf: &'a mut [u8]) -> Self::ReadFuture<'a> {
            async move {
                let len = buf.len();
                buf[..].copy_from_slice(&self.0[offset as usize..offset as usize + len]);
                Ok(())
            }
        }

        fn capacity(&self) -> usize {
            131072
        }
    }

    impl AsyncNorFlash for MemFlash {
        const WRITE_SIZE: usize = 4;
        const ERASE_SIZE: usize = 4096;

        type EraseFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a;
        fn erase<'a>(&'a mut self, from: u32, to: u32) -> Self::EraseFuture<'a> {
            async move {
                let from = from as usize;
                let to = to as usize;
                for i in from..to {
                    self.0[i] = 0xFF;
                    self.0[i] = 0xFF;
                    self.0[i] = 0xFF;
                    self.0[i] = 0xFF;
                }
                Ok(())
            }
        }

        type WriteFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a;
        fn write<'a>(&'a mut self, offset: u32, data: &'a [u8]) -> Self::WriteFuture<'a> {
            async move {
                assert!(data.len() % 4 == 0);
                assert!(offset % 4 == 0);
                assert!(offset as usize + data.len() < 131072);

                self.0[offset as usize..offset as usize + data.len()].copy_from_slice(data);

                Ok(())
            }
        }
    }
}
