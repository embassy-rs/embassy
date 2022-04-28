#![feature(type_alias_impl_trait)]
#![feature(generic_associated_types)]
#![feature(generic_const_exprs)]
#![allow(incomplete_features)]
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

const BOOT_MAGIC: u8 = 0xD0;
const SWAP_MAGIC: u8 = 0xF0;

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
    const ERASE_VALUE: u8;
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
/// different page sizes and flash write sizes.
///
/// The PAGE_SIZE const parameter must be a multiple of the ACTIVE and DFU page sizes.
pub struct BootLoader<const PAGE_SIZE: usize> {
    // Page with current state of bootloader. The state partition has the following format:
    // | Range          | Description                                                                      |
    // | 0 - WRITE_SIZE | Magic indicating bootloader state. BOOT_MAGIC means boot, SWAP_MAGIC means swap. |
    // | WRITE_SIZE - N | Progress index used while swapping or reverting                                  |
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
    pub fn prepare_boot<P: FlashProvider>(&mut self, p: &mut P) -> Result<State, BootError>
    where
        [(); <<P as FlashProvider>::STATE as FlashConfig>::FLASH::WRITE_SIZE]:,
        [(); <<P as FlashProvider>::ACTIVE as FlashConfig>::FLASH::ERASE_SIZE]:,
    {
        // Ensure we have enough progress pages to store copy progress
        assert!(
            self.active.len() / PAGE_SIZE
                <= (self.state.len()
                    - <<P as FlashProvider>::STATE as FlashConfig>::FLASH::WRITE_SIZE)
                    / <<P as FlashProvider>::STATE as FlashConfig>::FLASH::WRITE_SIZE
        );

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
                    trace!("Swapping done");
                } else {
                    trace!("Reverting");
                    self.revert(p)?;

                    // Overwrite magic and reset progress
                    let fstate = p.state().flash();
                    let aligned = Aligned(
                        [!P::STATE::ERASE_VALUE;
                            <<P as FlashProvider>::STATE as FlashConfig>::FLASH::WRITE_SIZE],
                    );
                    fstate.write(self.state.from as u32, &aligned.0)?;
                    fstate.erase(self.state.from as u32, self.state.to as u32)?;
                    let aligned = Aligned(
                        [BOOT_MAGIC;
                            <<P as FlashProvider>::STATE as FlashConfig>::FLASH::WRITE_SIZE],
                    );
                    fstate.write(self.state.from as u32, &aligned.0)?;
                }
            }
            _ => {}
        }
        Ok(state)
    }

    fn is_swapped<P: FlashConfig>(&mut self, p: &mut P) -> Result<bool, BootError>
    where
        [(); P::FLASH::WRITE_SIZE]:,
    {
        let page_count = self.active.len() / P::FLASH::ERASE_SIZE;
        let progress = self.current_progress(p)?;

        Ok(progress >= page_count * 2)
    }

    fn current_progress<P: FlashConfig>(&mut self, p: &mut P) -> Result<usize, BootError>
    where
        [(); P::FLASH::WRITE_SIZE]:,
    {
        let write_size = P::FLASH::WRITE_SIZE;
        let max_index = ((self.state.len() - write_size) / write_size) - 1;
        let flash = p.flash();
        let mut aligned = Aligned([!P::ERASE_VALUE; P::FLASH::WRITE_SIZE]);
        for i in 0..max_index {
            flash.read(
                (self.state.from + write_size + i * write_size) as u32,
                &mut aligned.0,
            )?;
            if aligned.0 == [P::ERASE_VALUE; P::FLASH::WRITE_SIZE] {
                return Ok(i);
            }
        }
        Ok(max_index)
    }

    fn update_progress<P: FlashConfig>(&mut self, idx: usize, p: &mut P) -> Result<(), BootError>
    where
        [(); P::FLASH::WRITE_SIZE]:,
    {
        let flash = p.flash();
        let write_size = P::FLASH::WRITE_SIZE;
        let w = self.state.from + write_size + idx * write_size;
        let aligned = Aligned([!P::ERASE_VALUE; P::FLASH::WRITE_SIZE]);
        flash.write(w as u32, &aligned.0)?;
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
    ) -> Result<(), BootError>
    where
        [(); <<P as FlashProvider>::STATE as FlashConfig>::FLASH::WRITE_SIZE]:,
    {
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
    ) -> Result<(), BootError>
    where
        [(); <<P as FlashProvider>::STATE as FlashConfig>::FLASH::WRITE_SIZE]:,
    {
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

    fn swap<P: FlashProvider>(&mut self, p: &mut P) -> Result<(), BootError>
    where
        [(); <<P as FlashProvider>::STATE as FlashConfig>::FLASH::WRITE_SIZE]:,
    {
        let page_count = self.active.len() / PAGE_SIZE;
        trace!("Page count: {}", page_count);
        for page in 0..page_count {
            trace!("COPY PAGE {}", page);
            // Copy active page to the 'next' DFU page.
            let active_page = self.active_addr(page_count - 1 - page);
            let dfu_page = self.dfu_addr(page_count - page);
            //trace!("Copy active {} to dfu {}", active_page, dfu_page);
            self.copy_page_once_to_dfu(page * 2, active_page, dfu_page, p)?;

            // Copy DFU page to the active page
            let active_page = self.active_addr(page_count - 1 - page);
            let dfu_page = self.dfu_addr(page_count - 1 - page);
            //trace!("Copy dfy {} to active {}", dfu_page, active_page);
            self.copy_page_once_to_active(page * 2 + 1, dfu_page, active_page, p)?;
        }

        Ok(())
    }

    fn revert<P: FlashProvider>(&mut self, p: &mut P) -> Result<(), BootError>
    where
        [(); <<P as FlashProvider>::STATE as FlashConfig>::FLASH::WRITE_SIZE]:,
    {
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

    fn read_state<P: FlashConfig>(&mut self, p: &mut P) -> Result<State, BootError>
    where
        [(); P::FLASH::WRITE_SIZE]:,
    {
        let mut magic: [u8; P::FLASH::WRITE_SIZE] = [0; P::FLASH::WRITE_SIZE];
        let flash = p.flash();
        flash.read(self.state.from as u32, &mut magic)?;

        if magic == [SWAP_MAGIC; P::FLASH::WRITE_SIZE] {
            Ok(State::Swap)
        } else {
            Ok(State::Boot)
        }
    }
}

/// Convenience provider that uses a single flash for everything
pub struct SingleFlashProvider<'a, F, const ERASE_VALUE: u8 = 0xFF>
where
    F: NorFlash + ReadNorFlash,
{
    config: SingleFlashConfig<'a, F, ERASE_VALUE>,
}

impl<'a, F, const ERASE_VALUE: u8> SingleFlashProvider<'a, F, ERASE_VALUE>
where
    F: NorFlash + ReadNorFlash,
{
    pub fn new(flash: &'a mut F) -> Self {
        Self {
            config: SingleFlashConfig { flash },
        }
    }
}

pub struct SingleFlashConfig<'a, F, const ERASE_VALUE: u8 = 0xFF>
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

impl<'a, F, const ERASE_VALUE: u8> FlashConfig for SingleFlashConfig<'a, F, ERASE_VALUE>
where
    F: NorFlash + ReadNorFlash,
{
    const BLOCK_SIZE: usize = F::ERASE_SIZE;
    const ERASE_VALUE: u8 = ERASE_VALUE;
    type FLASH = F;
    fn flash(&mut self) -> &mut F {
        self.flash
    }
}

/// Convenience provider that uses a single flash for everything
pub struct MultiFlashProvider<'a, ACTIVE, STATE, DFU>
where
    ACTIVE: NorFlash + ReadNorFlash,
    STATE: NorFlash + ReadNorFlash,
    DFU: NorFlash + ReadNorFlash,
{
    active: SingleFlashConfig<'a, ACTIVE>,
    state: SingleFlashConfig<'a, STATE>,
    dfu: SingleFlashConfig<'a, DFU>,
}

impl<'a, ACTIVE, STATE, DFU> MultiFlashProvider<'a, ACTIVE, STATE, DFU>
where
    ACTIVE: NorFlash + ReadNorFlash,
    STATE: NorFlash + ReadNorFlash,
    DFU: NorFlash + ReadNorFlash,
{
    pub fn new(active: &'a mut ACTIVE, state: &'a mut STATE, dfu: &'a mut DFU) -> Self {
        Self {
            active: SingleFlashConfig { flash: active },
            state: SingleFlashConfig { flash: state },
            dfu: SingleFlashConfig { flash: dfu },
        }
    }
}

impl<'a, ACTIVE, STATE, DFU> FlashProvider for MultiFlashProvider<'a, ACTIVE, STATE, DFU>
where
    ACTIVE: NorFlash + ReadNorFlash,
    STATE: NorFlash + ReadNorFlash,
    DFU: NorFlash + ReadNorFlash,
{
    type STATE = SingleFlashConfig<'a, STATE>;
    type ACTIVE = SingleFlashConfig<'a, ACTIVE>;
    type DFU = SingleFlashConfig<'a, DFU>;

    fn active(&mut self) -> &mut Self::ACTIVE {
        &mut self.active
    }
    fn dfu(&mut self) -> &mut Self::DFU {
        &mut self.dfu
    }
    fn state(&mut self) -> &mut Self::STATE {
        &mut self.state
    }
}

/// FirmwareUpdater is an application API for interacting with the BootLoader without the ability to
/// 'mess up' the internal bootloader state
pub struct FirmwareUpdater {
    state: Partition,
    dfu: Partition,
}

// NOTE: Aligned to the largest write size supported by flash
#[repr(align(32))]
pub struct Aligned<const N: usize>([u8; N]);

impl Default for FirmwareUpdater {
    fn default() -> Self {
        extern "C" {
            static __bootloader_state_start: u32;
            static __bootloader_state_end: u32;
            static __bootloader_dfu_start: u32;
            static __bootloader_dfu_end: u32;
        }

        let dfu = unsafe {
            Partition::new(
                &__bootloader_dfu_start as *const u32 as usize,
                &__bootloader_dfu_end as *const u32 as usize,
            )
        };
        let state = unsafe {
            Partition::new(
                &__bootloader_state_start as *const u32 as usize,
                &__bootloader_state_end as *const u32 as usize,
            )
        };

        trace!("DFU: 0x{:x} - 0x{:x}", dfu.from, dfu.to);
        trace!("STATE: 0x{:x} - 0x{:x}", state.from, state.to);
        FirmwareUpdater::new(dfu, state)
    }
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
    /// Must be provided with an aligned buffer to use for reading and writing magic;
    pub async fn update<F: AsyncNorFlash>(&mut self, flash: &mut F) -> Result<(), F::Error>
    where
        [(); F::WRITE_SIZE]:,
    {
        let mut aligned = Aligned([0; { F::WRITE_SIZE }]);
        self.set_magic(&mut aligned.0, SWAP_MAGIC, flash).await
    }

    /// Mark firmware boot successfully
    pub async fn mark_booted<F: AsyncNorFlash>(&mut self, flash: &mut F) -> Result<(), F::Error>
    where
        [(); F::WRITE_SIZE]:,
    {
        let mut aligned = Aligned([0; { F::WRITE_SIZE }]);
        self.set_magic(&mut aligned.0, BOOT_MAGIC, flash).await
    }

    async fn set_magic<F: AsyncNorFlash>(
        &mut self,
        aligned: &mut [u8],
        magic: u8,
        flash: &mut F,
    ) -> Result<(), F::Error> {
        flash.read(self.state.from as u32, aligned).await?;

        let mut is_set = true;
        for b in 0..aligned.len() {
            if aligned[b] != magic {
                is_set = false;
            }
        }
        if !is_set {
            for i in 0..aligned.len() {
                aligned[i] = 0;
            }
            flash.write(self.state.from as u32, aligned).await?;
            flash
                .erase(self.state.from as u32, self.state.to as u32)
                .await?;

            for i in 0..aligned.len() {
                aligned[i] = magic;
            }
            flash.write(self.state.from as u32, aligned).await?;
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
    use embedded_storage::nor_flash::ErrorType;
    use embedded_storage_async::nor_flash::AsyncReadNorFlash;
    use futures::executor::block_on;

    /*
    #[test]
    fn test_bad_magic() {
        let mut flash = MemFlash([0xff; 131072]);
        let mut flash = SingleFlashProvider::new(&mut flash);

        let mut bootloader = BootLoader::<4096>::new(ACTIVE, DFU, STATE);

        assert_eq!(
            bootloader.prepare_boot(&mut flash),
            Err(BootError::BadMagic)
        );
    }
    */

    #[test]
    fn test_boot_state() {
        const STATE: Partition = Partition::new(0, 4096);
        const ACTIVE: Partition = Partition::new(4096, 61440);
        const DFU: Partition = Partition::new(61440, 122880);

        let mut flash = MemFlash::<131072, 4096, 4>([0xff; 131072]);
        flash.0[0..4].copy_from_slice(&[BOOT_MAGIC; 4]);
        let mut flash = SingleFlashProvider::new(&mut flash);

        let mut bootloader: BootLoader<4096> = BootLoader::new(ACTIVE, DFU, STATE);

        assert_eq!(State::Boot, bootloader.prepare_boot(&mut flash).unwrap());
    }

    #[test]
    fn test_swap_state() {
        const STATE: Partition = Partition::new(0, 4096);
        const ACTIVE: Partition = Partition::new(4096, 61440);
        const DFU: Partition = Partition::new(61440, 122880);
        let mut flash = MemFlash::<131072, 4096, 4>([0xff; 131072]);

        let original: [u8; ACTIVE.len()] = [rand::random::<u8>(); ACTIVE.len()];
        let update: [u8; DFU.len()] = [rand::random::<u8>(); DFU.len()];

        for i in ACTIVE.from..ACTIVE.to {
            flash.0[i] = original[i - ACTIVE.from];
        }

        let mut bootloader: BootLoader<4096> = BootLoader::new(ACTIVE, DFU, STATE);
        let mut updater = FirmwareUpdater::new(DFU, STATE);
        let mut offset = 0;
        for chunk in update.chunks(4096) {
            block_on(updater.write_firmware(offset, &chunk, &mut flash, 4096)).unwrap();
            offset += chunk.len();
        }
        block_on(updater.update(&mut flash)).unwrap();

        assert_eq!(
            State::Swap,
            bootloader
                .prepare_boot(&mut SingleFlashProvider::new(&mut flash))
                .unwrap()
        );

        for i in ACTIVE.from..ACTIVE.to {
            assert_eq!(flash.0[i], update[i - ACTIVE.from], "Index {}", i);
        }

        // First DFU page is untouched
        for i in DFU.from + 4096..DFU.to {
            assert_eq!(flash.0[i], original[i - DFU.from - 4096], "Index {}", i);
        }

        // Running again should cause a revert
        assert_eq!(
            State::Swap,
            bootloader
                .prepare_boot(&mut SingleFlashProvider::new(&mut flash))
                .unwrap()
        );

        for i in ACTIVE.from..ACTIVE.to {
            assert_eq!(flash.0[i], original[i - ACTIVE.from], "Index {}", i);
        }

        // Last page is untouched
        for i in DFU.from..DFU.to - 4096 {
            assert_eq!(flash.0[i], update[i - DFU.from], "Index {}", i);
        }

        // Mark as booted
        block_on(updater.mark_booted(&mut flash)).unwrap();
        assert_eq!(
            State::Boot,
            bootloader
                .prepare_boot(&mut SingleFlashProvider::new(&mut flash))
                .unwrap()
        );
    }

    #[test]
    fn test_separate_flash_active_page_biggest() {
        const STATE: Partition = Partition::new(2048, 4096);
        const ACTIVE: Partition = Partition::new(4096, 16384);
        const DFU: Partition = Partition::new(0, 16384);

        let mut active = MemFlash::<16384, 4096, 8>([0xff; 16384]);
        let mut dfu = MemFlash::<16384, 2048, 8>([0xff; 16384]);
        let mut state = MemFlash::<4096, 128, 4>([0xff; 4096]);

        let original: [u8; ACTIVE.len()] = [rand::random::<u8>(); ACTIVE.len()];
        let update: [u8; DFU.len()] = [rand::random::<u8>(); DFU.len()];

        for i in ACTIVE.from..ACTIVE.to {
            active.0[i] = original[i - ACTIVE.from];
        }

        let mut updater = FirmwareUpdater::new(DFU, STATE);

        let mut offset = 0;
        for chunk in update.chunks(2048) {
            block_on(updater.write_firmware(offset, &chunk, &mut dfu, chunk.len())).unwrap();
            offset += chunk.len();
        }
        block_on(updater.update(&mut state)).unwrap();

        let mut bootloader: BootLoader<4096> = BootLoader::new(ACTIVE, DFU, STATE);
        assert_eq!(
            State::Swap,
            bootloader
                .prepare_boot(&mut MultiFlashProvider::new(
                    &mut active,
                    &mut state,
                    &mut dfu,
                ))
                .unwrap()
        );

        for i in ACTIVE.from..ACTIVE.to {
            assert_eq!(active.0[i], update[i - ACTIVE.from], "Index {}", i);
        }

        // First DFU page is untouched
        for i in DFU.from + 4096..DFU.to {
            assert_eq!(dfu.0[i], original[i - DFU.from - 4096], "Index {}", i);
        }
    }

    #[test]
    fn test_separate_flash_dfu_page_biggest() {
        const STATE: Partition = Partition::new(2048, 4096);
        const ACTIVE: Partition = Partition::new(4096, 16384);
        const DFU: Partition = Partition::new(0, 16384);

        let mut active = MemFlash::<16384, 2048, 4>([0xff; 16384]);
        let mut dfu = MemFlash::<16384, 4096, 8>([0xff; 16384]);
        let mut state = MemFlash::<4096, 128, 4>([0xff; 4096]);

        let original: [u8; ACTIVE.len()] = [rand::random::<u8>(); ACTIVE.len()];
        let update: [u8; DFU.len()] = [rand::random::<u8>(); DFU.len()];

        for i in ACTIVE.from..ACTIVE.to {
            active.0[i] = original[i - ACTIVE.from];
        }

        let mut updater = FirmwareUpdater::new(DFU, STATE);

        let mut offset = 0;
        for chunk in update.chunks(4096) {
            block_on(updater.write_firmware(offset, &chunk, &mut dfu, chunk.len())).unwrap();
            offset += chunk.len();
        }
        block_on(updater.update(&mut state)).unwrap();

        let mut bootloader: BootLoader<4096> = BootLoader::new(ACTIVE, DFU, STATE);
        assert_eq!(
            State::Swap,
            bootloader
                .prepare_boot(&mut MultiFlashProvider::new(
                    &mut active,
                    &mut state,
                    &mut dfu,
                ))
                .unwrap()
        );

        for i in ACTIVE.from..ACTIVE.to {
            assert_eq!(active.0[i], update[i - ACTIVE.from], "Index {}", i);
        }

        // First DFU page is untouched
        for i in DFU.from + 4096..DFU.to {
            assert_eq!(dfu.0[i], original[i - DFU.from - 4096], "Index {}", i);
        }
    }

    struct MemFlash<const SIZE: usize, const ERASE_SIZE: usize, const WRITE_SIZE: usize>(
        [u8; SIZE],
    );

    impl<const SIZE: usize, const ERASE_SIZE: usize, const WRITE_SIZE: usize> NorFlash
        for MemFlash<SIZE, ERASE_SIZE, WRITE_SIZE>
    {
        const WRITE_SIZE: usize = WRITE_SIZE;
        const ERASE_SIZE: usize = ERASE_SIZE;
        fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
            let from = from as usize;
            let to = to as usize;
            assert!(from % ERASE_SIZE == 0);
            assert!(
                to % ERASE_SIZE == 0,
                "To: {}, erase size: {}",
                to,
                ERASE_SIZE
            );
            for i in from..to {
                self.0[i] = 0xFF;
            }
            Ok(())
        }

        fn write(&mut self, offset: u32, data: &[u8]) -> Result<(), Self::Error> {
            assert!(data.len() % WRITE_SIZE == 0);
            assert!(offset as usize % WRITE_SIZE == 0);
            assert!(offset as usize + data.len() <= SIZE);

            self.0[offset as usize..offset as usize + data.len()].copy_from_slice(data);

            Ok(())
        }
    }

    impl<const SIZE: usize, const ERASE_SIZE: usize, const WRITE_SIZE: usize> ErrorType
        for MemFlash<SIZE, ERASE_SIZE, WRITE_SIZE>
    {
        type Error = Infallible;
    }

    impl<const SIZE: usize, const ERASE_SIZE: usize, const WRITE_SIZE: usize> ReadNorFlash
        for MemFlash<SIZE, ERASE_SIZE, WRITE_SIZE>
    {
        const READ_SIZE: usize = 4;

        fn read(&mut self, offset: u32, buf: &mut [u8]) -> Result<(), Self::Error> {
            let len = buf.len();
            buf[..].copy_from_slice(&self.0[offset as usize..offset as usize + len]);
            Ok(())
        }

        fn capacity(&self) -> usize {
            SIZE
        }
    }

    impl<const SIZE: usize, const ERASE_SIZE: usize, const WRITE_SIZE: usize> AsyncReadNorFlash
        for MemFlash<SIZE, ERASE_SIZE, WRITE_SIZE>
    {
        const READ_SIZE: usize = 4;

        type ReadFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a;
        fn read<'a>(&'a mut self, offset: u32, buf: &'a mut [u8]) -> Self::ReadFuture<'a> {
            async move {
                let len = buf.len();
                buf[..].copy_from_slice(&self.0[offset as usize..offset as usize + len]);
                Ok(())
            }
        }

        fn capacity(&self) -> usize {
            SIZE
        }
    }

    impl<const SIZE: usize, const ERASE_SIZE: usize, const WRITE_SIZE: usize> AsyncNorFlash
        for MemFlash<SIZE, ERASE_SIZE, WRITE_SIZE>
    {
        const WRITE_SIZE: usize = WRITE_SIZE;
        const ERASE_SIZE: usize = ERASE_SIZE;

        type EraseFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a;
        fn erase<'a>(&'a mut self, from: u32, to: u32) -> Self::EraseFuture<'a> {
            async move {
                let from = from as usize;
                let to = to as usize;
                assert!(from % ERASE_SIZE == 0);
                assert!(to % ERASE_SIZE == 0);
                for i in from..to {
                    self.0[i] = 0xFF;
                }
                Ok(())
            }
        }

        type WriteFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a;
        fn write<'a>(&'a mut self, offset: u32, data: &'a [u8]) -> Self::WriteFuture<'a> {
            info!("Writing {} bytes to 0x{:x}", data.len(), offset);
            async move {
                assert!(data.len() % WRITE_SIZE == 0);
                assert!(offset as usize % WRITE_SIZE == 0);
                assert!(
                    offset as usize + data.len() <= SIZE,
                    "OFFSET: {}, LEN: {}, FLASH SIZE: {}",
                    offset,
                    data.len(),
                    SIZE
                );

                self.0[offset as usize..offset as usize + data.len()].copy_from_slice(data);

                Ok(())
            }
        }
    }
}
