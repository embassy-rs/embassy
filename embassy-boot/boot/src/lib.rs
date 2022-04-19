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

use embedded_storage::nor_flash::{NorFlash, NorFlashError, NorFlashErrorKind, ReadNorFlash};
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
pub enum BootError {
    Flash(NorFlashErrorKind),
    BadMagic,
}

impl<E> From<E> for BootError
where
    E: NorFlashError,
{
    fn from(error: E) -> Self {
        BootError::Flash(error.kind())
    }
}

pub trait FlashConfig {
    const BLOCK_SIZE: usize;
    type FLASH: NorFlash + ReadNorFlash;

    fn flash(&mut self) -> &mut Self::FLASH;
}

/// Trait defining the flash handles used for active and DFU partition
pub trait FlashProvider {
    type STATE: FlashConfig;
    type ACTIVE: FlashConfig;
    type DFU: FlashConfig;

    /// Return flash instance used to write/read to/from active partition.
    fn active(&mut self) -> &mut Self::ACTIVE;
    /// Return flash instance used to write/read to/from dfu partition.
    fn dfu(&mut self) -> &mut Self::DFU;
    /// Return flash instance used to write/read to/from bootloader state.
    fn state(&mut self) -> &mut Self::STATE;
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
    pub fn prepare_boot<P: FlashProvider>(&mut self, p: &mut P) -> Result<State, BootError> {
        // Copy contents from partition N to active
        let state = self.read_state(p.state())?;
        match state {
            State::Swap => {
                //
                // Check if we already swapped. If we're in the swap state, this means we should revert
                // since the app has failed to mark boot as successful
                //
                if !self.is_swapped(p.state())? {
                    trace!("Swapping");
                    self.swap(p)?;
                } else {
                    trace!("Reverting");
                    self.revert(p)?;

                    // Overwrite magic and reset progress
                    let fstate = p.state().flash();
                    fstate.write(self.state.from as u32, &[0, 0, 0, 0])?;
                    fstate.erase(self.state.from as u32, self.state.to as u32)?;
                    fstate.write(self.state.from as u32, &BOOT_MAGIC.to_le_bytes())?;
                }
            }
            _ => {}
        }
        Ok(state)
    }

    fn is_swapped<P: FlashConfig>(&mut self, p: &mut P) -> Result<bool, BootError> {
        let page_count = self.active.len() / PAGE_SIZE;
        let progress = self.current_progress(p)?;

        Ok(progress >= page_count * 2)
    }

    fn current_progress<P: FlashConfig>(&mut self, p: &mut P) -> Result<usize, BootError> {
        let max_index = ((self.state.len() - 4) / 4) - 1;
        let flash = p.flash();
        for i in 0..max_index {
            let mut buf: [u8; 4] = [0; 4];
            flash.read((self.state.from + 4 + i * 4) as u32, &mut buf)?;
            if buf == [0xFF, 0xFF, 0xFF, 0xFF] {
                return Ok(i);
            }
        }
        Ok(max_index)
    }

    fn update_progress<P: FlashConfig>(&mut self, idx: usize, p: &mut P) -> Result<(), BootError> {
        let flash = p.flash();
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

    fn copy_page_once_to_active<P: FlashProvider>(
        &mut self,
        idx: usize,
        from_page: usize,
        to_page: usize,
        p: &mut P,
    ) -> Result<(), BootError> {
        let mut buf: [u8; PAGE_SIZE] = [0; PAGE_SIZE];
        if self.current_progress(p.state())? <= idx {
            let mut offset = from_page;
            for chunk in buf.chunks_mut(P::DFU::BLOCK_SIZE) {
                p.dfu().flash().read(offset as u32, chunk)?;
                offset += chunk.len();
            }

            p.active()
                .flash()
                .erase(to_page as u32, (to_page + PAGE_SIZE) as u32)?;

            let mut offset = to_page;
            for chunk in buf.chunks(P::ACTIVE::BLOCK_SIZE) {
                p.active().flash().write(offset as u32, &chunk)?;
                offset += chunk.len();
            }
            self.update_progress(idx, p.state())?;
        }
        Ok(())
    }

    fn copy_page_once_to_dfu<P: FlashProvider>(
        &mut self,
        idx: usize,
        from_page: usize,
        to_page: usize,
        p: &mut P,
    ) -> Result<(), BootError> {
        let mut buf: [u8; PAGE_SIZE] = [0; PAGE_SIZE];
        if self.current_progress(p.state())? <= idx {
            let mut offset = from_page;
            for chunk in buf.chunks_mut(P::ACTIVE::BLOCK_SIZE) {
                p.active().flash().read(offset as u32, chunk)?;
                offset += chunk.len();
            }

            p.dfu()
                .flash()
                .erase(to_page as u32, (to_page + PAGE_SIZE) as u32)?;

            let mut offset = to_page;
            for chunk in buf.chunks(P::DFU::BLOCK_SIZE) {
                p.dfu().flash().write(offset as u32, chunk)?;
                offset += chunk.len();
            }
            self.update_progress(idx, p.state())?;
        }
        Ok(())
    }

    fn swap<P: FlashProvider>(&mut self, p: &mut P) -> Result<(), BootError> {
        let page_count = self.active.len() / PAGE_SIZE;
        // trace!("Page count: {}", page_count);
        for page in 0..page_count {
            // Copy active page to the 'next' DFU page.
            let active_page = self.active_addr(page_count - 1 - page);
            let dfu_page = self.dfu_addr(page_count - page);
            info!("Copy active {} to dfu {}", active_page, dfu_page);
            self.copy_page_once_to_dfu(page * 2, active_page, dfu_page, p)?;

            // Copy DFU page to the active page
            let active_page = self.active_addr(page_count - 1 - page);
            let dfu_page = self.dfu_addr(page_count - 1 - page);
            info!("Copy dfy {} to active {}", dfu_page, active_page);
            self.copy_page_once_to_active(page * 2 + 1, dfu_page, active_page, p)?;
        }

        Ok(())
    }

    fn revert<P: FlashProvider>(&mut self, p: &mut P) -> Result<(), BootError> {
        let page_count = self.active.len() / PAGE_SIZE;
        for page in 0..page_count {
            // Copy the bad active page to the DFU page
            let active_page = self.active_addr(page);
            let dfu_page = self.dfu_addr(page);
            self.copy_page_once_to_dfu(page_count * 2 + page * 2, active_page, dfu_page, p)?;

            // Copy the DFU page back to the active page
            let active_page = self.active_addr(page);
            let dfu_page = self.dfu_addr(page + 1);
            self.copy_page_once_to_active(page_count * 2 + page * 2 + 1, dfu_page, active_page, p)?;
        }

        Ok(())
    }

    fn read_state<P: FlashConfig>(&mut self, p: &mut P) -> Result<State, BootError> {
        let mut magic: [u8; 4] = [0; 4];
        let flash = p.flash();
        flash.read(self.state.from as u32, &mut magic)?;

        match u32::from_le_bytes(magic) {
            SWAP_MAGIC => Ok(State::Swap),
            _ => Ok(State::Boot),
        }
    }
}

/// Convenience provider that uses a single flash for everything
pub struct SingleFlashProvider<'a, F>
where
    F: NorFlash + ReadNorFlash,
{
    config: SingleFlashConfig<'a, F>,
}

impl<'a, F> SingleFlashProvider<'a, F>
where
    F: NorFlash + ReadNorFlash,
{
    pub fn new(flash: &'a mut F) -> Self {
        Self {
            config: SingleFlashConfig { flash },
        }
    }
}

pub struct SingleFlashConfig<'a, F>
where
    F: NorFlash + ReadNorFlash,
{
    flash: &'a mut F,
}

impl<'a, F> FlashProvider for SingleFlashProvider<'a, F>
where
    F: NorFlash + ReadNorFlash,
{
    type STATE = SingleFlashConfig<'a, F>;
    type ACTIVE = SingleFlashConfig<'a, F>;
    type DFU = SingleFlashConfig<'a, F>;

    fn active(&mut self) -> &mut Self::STATE {
        &mut self.config
    }
    fn dfu(&mut self) -> &mut Self::ACTIVE {
        &mut self.config
    }
    fn state(&mut self) -> &mut Self::DFU {
        &mut self.config
    }
}

impl<'a, F> FlashConfig for SingleFlashConfig<'a, F>
where
    F: NorFlash + ReadNorFlash,
{
    const BLOCK_SIZE: usize = F::ERASE_SIZE;
    type FLASH = F;
    fn flash(&mut self) -> &mut F {
        self.flash
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
        #[repr(align(4))]
        struct Aligned([u8; 4]);

        let mut magic = Aligned([0; 4]);
        flash.read(self.state.from as u32, &mut magic.0).await?;
        let magic = u32::from_le_bytes(magic.0);

        if magic != SWAP_MAGIC {
            flash
                .write(self.state.from as u32, &Aligned([0; 4]).0)
                .await?;
            flash
                .erase(self.state.from as u32, self.state.to as u32)
                .await?;
            trace!(
                "Setting swap magic at {} to 0x{:x}, LE: 0x{:x}",
                self.state.from,
                &SWAP_MAGIC,
                &SWAP_MAGIC.to_le_bytes()
            );
            flash
                .write(self.state.from as u32, &SWAP_MAGIC.to_le_bytes())
                .await?;
        }
        Ok(())
    }

    /// Mark firmware boot successfully
    pub async fn mark_booted<F: AsyncNorFlash>(&mut self, flash: &mut F) -> Result<(), F::Error> {
        #[repr(align(4))]
        struct Aligned([u8; 4]);

        let mut magic = Aligned([0; 4]);
        flash.read(self.state.from as u32, &mut magic.0).await?;
        let magic = u32::from_le_bytes(magic.0);

        if magic != BOOT_MAGIC {
            flash
                .write(self.state.from as u32, &Aligned([0; 4]).0)
                .await?;
            flash
                .erase(self.state.from as u32, self.state.to as u32)
                .await?;
            flash
                .write(self.state.from as u32, &BOOT_MAGIC.to_le_bytes())
                .await?;
        }
        Ok(())
    }

    // Write to a region of the DFU page
    pub async fn write_firmware<F: AsyncNorFlash>(
        &mut self,
        offset: usize,
        data: &[u8],
        flash: &mut F,
        block_size: usize,
    ) -> Result<(), F::Error> {
        assert!(data.len() >= F::ERASE_SIZE);

        trace!(
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

        trace!(
            "Erased from {} to {}",
            self.dfu.from + offset,
            self.dfu.from + offset + data.len()
        );

        let mut write_offset = self.dfu.from + offset;
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
