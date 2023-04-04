use embedded_storage::nor_flash::{ErrorType, NorFlash, NorFlashError, NorFlashErrorKind, ReadNorFlash};

use crate::{Partition, State, BOOT_MAGIC, SWAP_MAGIC};

/// Errors returned by bootloader
#[derive(PartialEq, Eq, Debug)]
pub enum BootError {
    /// Error from flash.
    Flash(NorFlashErrorKind),
    /// Invalid bootloader magic
    BadMagic,
}

#[cfg(feature = "defmt")]
impl defmt::Format for BootError {
    fn format(&self, fmt: defmt::Formatter) {
        match self {
            BootError::Flash(_) => defmt::write!(fmt, "BootError::Flash(_)"),
            BootError::BadMagic => defmt::write!(fmt, "BootError::BadMagic"),
        }
    }
}

impl<E> From<E> for BootError
where
    E: NorFlashError,
{
    fn from(error: E) -> Self {
        BootError::Flash(error.kind())
    }
}

/// Trait defining the flash handles used for active and DFU partition.
pub trait FlashConfig {
    /// The erase value of the state flash. Typically the default of 0xFF is used, but some flashes use a different value.
    const STATE_ERASE_VALUE: u8 = 0xFF;
    /// Flash type used for the state partition.
    type STATE: NorFlash;
    /// Flash type used for the active partition.
    type ACTIVE: NorFlash;
    /// Flash type used for the dfu partition.
    type DFU: NorFlash;

    /// Return flash instance used to write/read to/from active partition.
    fn active(&mut self) -> &mut Self::ACTIVE;
    /// Return flash instance used to write/read to/from dfu partition.
    fn dfu(&mut self) -> &mut Self::DFU;
    /// Return flash instance used to write/read to/from bootloader state.
    fn state(&mut self) -> &mut Self::STATE;
}

trait FlashConfigEx {
    fn page_size() -> usize;
}

impl<T: FlashConfig> FlashConfigEx for T {
    /// Get the page size which is the "unit of operation" within the bootloader.
    fn page_size() -> usize {
        core::cmp::max(T::ACTIVE::ERASE_SIZE, T::DFU::ERASE_SIZE)
    }
}

/// BootLoader works with any flash implementing embedded_storage.
pub struct BootLoader {
    // Page with current state of bootloader. The state partition has the following format:
    // All ranges are in multiples of WRITE_SIZE bytes.
    // | Range    | Description                                                                      |
    // | 0..1     | Magic indicating bootloader state. BOOT_MAGIC means boot, SWAP_MAGIC means swap. |
    // | 1..2     | Progress validity. ERASE_VALUE means valid, !ERASE_VALUE means invalid.          |
    // | 2..2 + N | Progress index used while swapping or reverting                                  |
    state: Partition,
    // Location of the partition which will be booted from
    active: Partition,
    // Location of the partition which will be swapped in when requested
    dfu: Partition,
}

impl BootLoader {
    /// Create a new instance of a bootloader with the given partitions.
    ///
    /// - All partitions must be aligned with the PAGE_SIZE const generic parameter.
    /// - The dfu partition must be at least PAGE_SIZE bigger than the active partition.
    pub fn new(active: Partition, dfu: Partition, state: Partition) -> Self {
        Self { active, dfu, state }
    }

    /// Return the offset of the active partition into the active flash.
    pub fn boot_address(&self) -> usize {
        self.active.from
    }

    /// Perform necessary boot preparations like swapping images.
    ///
    /// The DFU partition is assumed to be 1 page bigger than the active partition for the swap
    /// algorithm to work correctly.
    ///
    /// The provided aligned_buf argument must satisfy any alignment requirements
    /// given by the partition flashes. All flash operations will use this buffer.
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
    /// +-----------+------------+--------+--------+--------+--------+
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
    pub fn prepare_boot<P: FlashConfig>(&mut self, p: &mut P, aligned_buf: &mut [u8]) -> Result<State, BootError> {
        // Ensure we have enough progress pages to store copy progress
        assert_eq!(0, P::page_size() % aligned_buf.len());
        assert_eq!(0, P::page_size() % P::ACTIVE::WRITE_SIZE);
        assert_eq!(0, P::page_size() % P::DFU::WRITE_SIZE);
        assert_eq!(0, P::page_size() % P::ACTIVE::ERASE_SIZE);
        assert_eq!(0, P::page_size() % P::DFU::ERASE_SIZE);
        assert!(aligned_buf.len() >= P::STATE::WRITE_SIZE);
        assert_partitions(self.active, self.dfu, self.state, P::page_size(), P::STATE::WRITE_SIZE);

        // Copy contents from partition N to active
        let state = self.read_state(p, aligned_buf)?;
        if state == State::Swap {
            //
            // Check if we already swapped. If we're in the swap state, this means we should revert
            // since the app has failed to mark boot as successful
            //
            if !self.is_swapped(p, aligned_buf)? {
                trace!("Swapping");
                self.swap(p, aligned_buf)?;
                trace!("Swapping done");
            } else {
                trace!("Reverting");
                self.revert(p, aligned_buf)?;

                let state_flash = p.state();
                let state_word = &mut aligned_buf[..P::STATE::WRITE_SIZE];

                // Invalidate progress
                state_word.fill(!P::STATE_ERASE_VALUE);
                self.state
                    .write_blocking(state_flash, P::STATE::WRITE_SIZE as u32, state_word)?;

                // Clear magic and progress
                self.state.wipe_blocking(state_flash)?;

                // Set magic
                state_word.fill(BOOT_MAGIC);
                self.state.write_blocking(state_flash, 0, state_word)?;
            }
        }
        Ok(state)
    }

    fn is_swapped<P: FlashConfig>(&mut self, p: &mut P, aligned_buf: &mut [u8]) -> Result<bool, BootError> {
        let page_count = self.active.len() / P::page_size();
        let progress = self.current_progress(p, aligned_buf)?;

        Ok(progress >= page_count * 2)
    }

    fn current_progress<P: FlashConfig>(&mut self, config: &mut P, aligned_buf: &mut [u8]) -> Result<usize, BootError> {
        let max_index = ((self.state.len() - P::STATE::WRITE_SIZE) / P::STATE::WRITE_SIZE) - 2;
        let state_flash = config.state();
        let state_word = &mut aligned_buf[..P::STATE::WRITE_SIZE];

        self.state
            .read_blocking(state_flash, P::STATE::WRITE_SIZE as u32, state_word)?;
        if state_word.iter().any(|&b| b != P::STATE_ERASE_VALUE) {
            // Progress is invalid
            return Ok(max_index);
        }

        for index in 0..max_index {
            self.state.read_blocking(
                state_flash,
                (2 + index) as u32 * P::STATE::WRITE_SIZE as u32,
                state_word,
            )?;

            if state_word.iter().any(|&b| b == P::STATE_ERASE_VALUE) {
                return Ok(index);
            }
        }
        Ok(max_index)
    }

    fn update_progress<P: FlashConfig>(
        &mut self,
        index: usize,
        p: &mut P,
        aligned_buf: &mut [u8],
    ) -> Result<(), BootError> {
        let state_word = &mut aligned_buf[..P::STATE::WRITE_SIZE];
        state_word.fill(!P::STATE_ERASE_VALUE);
        self.state
            .write_blocking(p.state(), (2 + index) as u32 * P::STATE::WRITE_SIZE as u32, state_word)?;
        Ok(())
    }

    fn copy_page_once_to_active<P: FlashConfig>(
        &mut self,
        idx: usize,
        from_offset: u32,
        to_offset: u32,
        p: &mut P,
        aligned_buf: &mut [u8],
    ) -> Result<(), BootError> {
        if self.current_progress(p, aligned_buf)? <= idx {
            let page_size = P::page_size() as u32;

            self.active
                .erase_blocking(p.active(), to_offset, to_offset + page_size)?;

            for offset_in_page in (0..page_size).step_by(aligned_buf.len()) {
                self.dfu
                    .read_blocking(p.dfu(), from_offset + offset_in_page as u32, aligned_buf)?;
                self.active
                    .write_blocking(p.active(), to_offset + offset_in_page as u32, aligned_buf)?;
            }

            self.update_progress(idx, p, aligned_buf)?;
        }
        Ok(())
    }

    fn copy_page_once_to_dfu<P: FlashConfig>(
        &mut self,
        idx: usize,
        from_offset: u32,
        to_offset: u32,
        p: &mut P,
        aligned_buf: &mut [u8],
    ) -> Result<(), BootError> {
        if self.current_progress(p, aligned_buf)? <= idx {
            let page_size = P::page_size() as u32;

            self.dfu
                .erase_blocking(p.dfu(), to_offset as u32, to_offset + page_size)?;

            for offset_in_page in (0..page_size).step_by(aligned_buf.len()) {
                self.active
                    .read_blocking(p.active(), from_offset + offset_in_page as u32, aligned_buf)?;
                self.dfu
                    .write_blocking(p.dfu(), to_offset + offset_in_page as u32, aligned_buf)?;
            }

            self.update_progress(idx, p, aligned_buf)?;
        }
        Ok(())
    }

    fn swap<P: FlashConfig>(&mut self, p: &mut P, aligned_buf: &mut [u8]) -> Result<(), BootError> {
        let page_size = P::page_size();
        let page_count = self.active.len() / page_size;
        trace!("Page count: {}", page_count);
        for page_num in 0..page_count {
            trace!("COPY PAGE {}", page_num);

            let idx = page_num * 2;

            // Copy active page to the 'next' DFU page.
            let active_from_offset = ((page_count - 1 - page_num) * page_size) as u32;
            let dfu_to_offset = ((page_count - page_num) * page_size) as u32;
            //trace!("Copy active {} to dfu {}", active_from_offset, dfu_to_offset);
            self.copy_page_once_to_dfu(idx, active_from_offset, dfu_to_offset, p, aligned_buf)?;

            // Copy DFU page to the active page
            let active_to_offset = ((page_count - 1 - page_num) * page_size) as u32;
            let dfu_from_offset = ((page_count - 1 - page_num) * page_size) as u32;
            //trace!("Copy dfy {} to active {}", dfu_from_offset, active_to_offset);
            self.copy_page_once_to_active(idx + 1, dfu_from_offset, active_to_offset, p, aligned_buf)?;
        }

        Ok(())
    }

    fn revert<P: FlashConfig>(&mut self, p: &mut P, aligned_buf: &mut [u8]) -> Result<(), BootError> {
        let page_size = P::page_size();
        let page_count = self.active.len() / page_size;
        for page_num in 0..page_count {
            let idx = page_count * 2 + page_num * 2;

            // Copy the bad active page to the DFU page
            let active_from_offset = (page_num * page_size) as u32;
            let dfu_to_offset = (page_num * page_size) as u32;
            self.copy_page_once_to_dfu(idx, active_from_offset, dfu_to_offset, p, aligned_buf)?;

            // Copy the DFU page back to the active page
            let active_to_offset = (page_num * page_size) as u32;
            let dfu_from_offset = ((page_num + 1) * page_size) as u32;
            self.copy_page_once_to_active(idx + 1, dfu_from_offset, active_to_offset, p, aligned_buf)?;
        }

        Ok(())
    }

    fn read_state<P: FlashConfig>(&mut self, config: &mut P, aligned_buf: &mut [u8]) -> Result<State, BootError> {
        let state_word = &mut aligned_buf[..P::STATE::WRITE_SIZE];
        self.state.read_blocking(config.state(), 0, state_word)?;

        if !state_word.iter().any(|&b| b != SWAP_MAGIC) {
            Ok(State::Swap)
        } else {
            Ok(State::Boot)
        }
    }
}

fn assert_partitions(active: Partition, dfu: Partition, state: Partition, page_size: usize, write_size: usize) {
    assert_eq!(active.len() % page_size, 0);
    assert_eq!(dfu.len() % page_size, 0);
    assert!(dfu.len() - active.len() >= page_size);
    assert!(2 + 2 * (active.len() / page_size) <= state.len() / write_size);
}

/// A flash wrapper implementing the Flash and embedded_storage traits.
pub struct BootFlash<F>
where
    F: NorFlash,
{
    flash: F,
}

impl<F> BootFlash<F>
where
    F: NorFlash,
{
    /// Create a new instance of a bootable flash
    pub fn new(flash: F) -> Self {
        Self { flash }
    }
}

impl<F> ErrorType for BootFlash<F>
where
    F: NorFlash,
{
    type Error = F::Error;
}

impl<F> NorFlash for BootFlash<F>
where
    F: NorFlash,
{
    const WRITE_SIZE: usize = F::WRITE_SIZE;
    const ERASE_SIZE: usize = F::ERASE_SIZE;

    fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
        F::erase(&mut self.flash, from, to)
    }

    fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
        F::write(&mut self.flash, offset, bytes)
    }
}

impl<F> ReadNorFlash for BootFlash<F>
where
    F: NorFlash,
{
    const READ_SIZE: usize = F::READ_SIZE;

    fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        F::read(&mut self.flash, offset, bytes)
    }

    fn capacity(&self) -> usize {
        F::capacity(&self.flash)
    }
}

/// Convenience provider that uses a single flash for all partitions.
pub struct SingleFlashConfig<'a, F>
where
    F: NorFlash,
{
    flash: &'a mut F,
}

impl<'a, F> SingleFlashConfig<'a, F>
where
    F: NorFlash,
{
    /// Create a provider for a single flash.
    pub fn new(flash: &'a mut F) -> Self {
        Self { flash }
    }
}

impl<'a, F> FlashConfig for SingleFlashConfig<'a, F>
where
    F: NorFlash,
{
    type STATE = F;
    type ACTIVE = F;
    type DFU = F;

    fn active(&mut self) -> &mut Self::STATE {
        self.flash
    }
    fn dfu(&mut self) -> &mut Self::ACTIVE {
        self.flash
    }
    fn state(&mut self) -> &mut Self::DFU {
        self.flash
    }
}

/// Convenience flash provider that uses separate flash instances for each partition.
pub struct MultiFlashConfig<'a, ACTIVE, STATE, DFU>
where
    ACTIVE: NorFlash,
    STATE: NorFlash,
    DFU: NorFlash,
{
    active: &'a mut ACTIVE,
    state: &'a mut STATE,
    dfu: &'a mut DFU,
}

impl<'a, ACTIVE, STATE, DFU> MultiFlashConfig<'a, ACTIVE, STATE, DFU>
where
    ACTIVE: NorFlash,
    STATE: NorFlash,
    DFU: NorFlash,
{
    /// Create a new flash provider with separate configuration for all three partitions.
    pub fn new(active: &'a mut ACTIVE, state: &'a mut STATE, dfu: &'a mut DFU) -> Self {
        Self { active, state, dfu }
    }
}

impl<'a, ACTIVE, STATE, DFU> FlashConfig for MultiFlashConfig<'a, ACTIVE, STATE, DFU>
where
    ACTIVE: NorFlash,
    STATE: NorFlash,
    DFU: NorFlash,
{
    type STATE = STATE;
    type ACTIVE = ACTIVE;
    type DFU = DFU;

    fn active(&mut self) -> &mut Self::ACTIVE {
        self.active
    }
    fn dfu(&mut self) -> &mut Self::DFU {
        self.dfu
    }
    fn state(&mut self) -> &mut Self::STATE {
        self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn test_range_asserts() {
        const ACTIVE: Partition = Partition::new(4096, 4194304);
        const DFU: Partition = Partition::new(4194304, 2 * 4194304);
        const STATE: Partition = Partition::new(0, 4096);
        assert_partitions(ACTIVE, DFU, STATE, 4096, 4);
    }
}
