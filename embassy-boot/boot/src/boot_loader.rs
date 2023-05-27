use embedded_storage::nor_flash::{ErrorType, NorFlash, NorFlashError, NorFlashErrorKind, ReadNorFlash};

use crate::{State, BOOT_MAGIC, SWAP_MAGIC};

const STATE_ERASE_VALUE: u8 = 0xFF;

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

/// BootLoader works with any flash implementing embedded_storage.
pub struct BootLoader<ACTIVE: NorFlash, DFU: NorFlash, STATE: NorFlash> {
    /// Flash type used for the active partition - the partition which will be booted from.
    active: ACTIVE,
    /// Flash type used for the dfu partition - he partition which will be swapped in when requested.
    dfu: DFU,
    /// Flash type used for the state partition.
    ///
    /// The state partition has the following format:
    /// All ranges are in multiples of WRITE_SIZE bytes.
    /// | Range    | Description                                                                      |
    /// | 0..1     | Magic indicating bootloader state. BOOT_MAGIC means boot, SWAP_MAGIC means swap. |
    /// | 1..2     | Progress validity. ERASE_VALUE means valid, !ERASE_VALUE means invalid.          |
    /// | 2..2 + N | Progress index used while swapping or reverting      
    state: STATE,
}

impl<ACTIVE: NorFlash, DFU: NorFlash, STATE: NorFlash> BootLoader<ACTIVE, DFU, STATE> {
    /// Get the page size which is the "unit of operation" within the bootloader.
    const PAGE_SIZE: u32 = if ACTIVE::ERASE_SIZE > DFU::ERASE_SIZE {
        ACTIVE::ERASE_SIZE as u32
    } else {
        DFU::ERASE_SIZE as u32
    };

    /// Create a new instance of a bootloader with the flash partitions.
    ///
    /// - All partitions must be aligned with the PAGE_SIZE const generic parameter.
    /// - The dfu partition must be at least PAGE_SIZE bigger than the active partition.
    pub fn new(active: ACTIVE, dfu: DFU, state: STATE) -> Self {
        Self { active, dfu, state }
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
    pub fn prepare_boot(&mut self, aligned_buf: &mut [u8]) -> Result<State, BootError> {
        // Ensure we have enough progress pages to store copy progress
        assert_eq!(0, Self::PAGE_SIZE % aligned_buf.len() as u32);
        assert_eq!(0, Self::PAGE_SIZE % ACTIVE::WRITE_SIZE as u32);
        assert_eq!(0, Self::PAGE_SIZE % ACTIVE::ERASE_SIZE as u32);
        assert_eq!(0, Self::PAGE_SIZE % DFU::WRITE_SIZE as u32);
        assert_eq!(0, Self::PAGE_SIZE % DFU::ERASE_SIZE as u32);
        assert!(aligned_buf.len() >= STATE::WRITE_SIZE);
        assert_eq!(0, aligned_buf.len() % ACTIVE::WRITE_SIZE);
        assert_eq!(0, aligned_buf.len() % DFU::WRITE_SIZE);

        assert_partitions(&self.active, &self.dfu, &self.state, Self::PAGE_SIZE);

        // Copy contents from partition N to active
        let state = self.read_state(aligned_buf)?;
        if state == State::Swap {
            //
            // Check if we already swapped. If we're in the swap state, this means we should revert
            // since the app has failed to mark boot as successful
            //
            if !self.is_swapped(aligned_buf)? {
                trace!("Swapping");
                self.swap(aligned_buf)?;
                trace!("Swapping done");
            } else {
                trace!("Reverting");
                self.revert(aligned_buf)?;

                let state_word = &mut aligned_buf[..STATE::WRITE_SIZE];

                // Invalidate progress
                state_word.fill(!STATE_ERASE_VALUE);
                self.state.write(STATE::WRITE_SIZE as u32, state_word)?;

                // Clear magic and progress
                self.state.erase(0, self.state.capacity() as u32)?;

                // Set magic
                state_word.fill(BOOT_MAGIC);
                self.state.write(0, state_word)?;
            }
        }
        Ok(state)
    }

    fn is_swapped(&mut self, aligned_buf: &mut [u8]) -> Result<bool, BootError> {
        let page_count = self.active.capacity() / Self::PAGE_SIZE as usize;
        let progress = self.current_progress(aligned_buf)?;

        Ok(progress >= page_count * 2)
    }

    fn current_progress(&mut self, aligned_buf: &mut [u8]) -> Result<usize, BootError> {
        let write_size = STATE::WRITE_SIZE as u32;
        let max_index = ((self.state.capacity() - STATE::WRITE_SIZE) / STATE::WRITE_SIZE) - 2;
        let state_word = &mut aligned_buf[..write_size as usize];

        self.state.read(write_size, state_word)?;
        if state_word.iter().any(|&b| b != STATE_ERASE_VALUE) {
            // Progress is invalid
            return Ok(max_index);
        }

        for index in 0..max_index {
            self.state.read((2 + index) as u32 * write_size, state_word)?;

            if state_word.iter().any(|&b| b == STATE_ERASE_VALUE) {
                return Ok(index);
            }
        }
        Ok(max_index)
    }

    fn update_progress(&mut self, progress_index: usize, aligned_buf: &mut [u8]) -> Result<(), BootError> {
        let state_word = &mut aligned_buf[..STATE::WRITE_SIZE];
        state_word.fill(!STATE_ERASE_VALUE);
        self.state
            .write((2 + progress_index) as u32 * STATE::WRITE_SIZE as u32, state_word)?;
        Ok(())
    }

    fn copy_page_once_to_active(
        &mut self,
        progress_index: usize,
        from_offset: u32,
        to_offset: u32,
        aligned_buf: &mut [u8],
    ) -> Result<(), BootError> {
        if self.current_progress(aligned_buf)? <= progress_index {
            let page_size = Self::PAGE_SIZE as u32;

            self.active.erase(to_offset, to_offset + page_size)?;

            for offset_in_page in (0..page_size).step_by(aligned_buf.len()) {
                self.dfu.read(from_offset + offset_in_page as u32, aligned_buf)?;
                self.active.write(to_offset + offset_in_page as u32, aligned_buf)?;
            }

            self.update_progress(progress_index, aligned_buf)?;
        }
        Ok(())
    }

    fn copy_page_once_to_dfu(
        &mut self,
        progress_index: usize,
        from_offset: u32,
        to_offset: u32,
        aligned_buf: &mut [u8],
    ) -> Result<(), BootError> {
        if self.current_progress(aligned_buf)? <= progress_index {
            let page_size = Self::PAGE_SIZE as u32;

            self.dfu.erase(to_offset as u32, to_offset + page_size)?;

            for offset_in_page in (0..page_size).step_by(aligned_buf.len()) {
                self.active.read(from_offset + offset_in_page as u32, aligned_buf)?;
                self.dfu.write(to_offset + offset_in_page as u32, aligned_buf)?;
            }

            self.update_progress(progress_index, aligned_buf)?;
        }
        Ok(())
    }

    fn swap(&mut self, aligned_buf: &mut [u8]) -> Result<(), BootError> {
        let page_count = self.active.capacity() as u32 / Self::PAGE_SIZE;
        for page_num in 0..page_count {
            let progress_index = (page_num * 2) as usize;

            // Copy active page to the 'next' DFU page.
            let active_from_offset = (page_count - 1 - page_num) * Self::PAGE_SIZE;
            let dfu_to_offset = (page_count - page_num) * Self::PAGE_SIZE;
            //trace!("Copy active {} to dfu {}", active_from_offset, dfu_to_offset);
            self.copy_page_once_to_dfu(progress_index, active_from_offset, dfu_to_offset, aligned_buf)?;

            // Copy DFU page to the active page
            let active_to_offset = (page_count - 1 - page_num) * Self::PAGE_SIZE;
            let dfu_from_offset = (page_count - 1 - page_num) * Self::PAGE_SIZE;
            //trace!("Copy dfy {} to active {}", dfu_from_offset, active_to_offset);
            self.copy_page_once_to_active(progress_index + 1, dfu_from_offset, active_to_offset, aligned_buf)?;
        }

        Ok(())
    }

    fn revert(&mut self, aligned_buf: &mut [u8]) -> Result<(), BootError> {
        let page_count = self.active.capacity() as u32 / Self::PAGE_SIZE;
        for page_num in 0..page_count {
            let progress_index = (page_count * 2 + page_num * 2) as usize;

            // Copy the bad active page to the DFU page
            let active_from_offset = page_num * Self::PAGE_SIZE;
            let dfu_to_offset = page_num * Self::PAGE_SIZE;
            self.copy_page_once_to_dfu(progress_index, active_from_offset, dfu_to_offset, aligned_buf)?;

            // Copy the DFU page back to the active page
            let active_to_offset = page_num * Self::PAGE_SIZE;
            let dfu_from_offset = (page_num + 1) * Self::PAGE_SIZE;
            self.copy_page_once_to_active(progress_index + 1, dfu_from_offset, active_to_offset, aligned_buf)?;
        }

        Ok(())
    }

    fn read_state(&mut self, aligned_buf: &mut [u8]) -> Result<State, BootError> {
        let state_word = &mut aligned_buf[..STATE::WRITE_SIZE];
        self.state.read(0, state_word)?;

        if !state_word.iter().any(|&b| b != SWAP_MAGIC) {
            Ok(State::Swap)
        } else {
            Ok(State::Boot)
        }
    }
}

fn assert_partitions<ACTIVE: NorFlash, DFU: NorFlash, STATE: NorFlash>(
    active: &ACTIVE,
    dfu: &DFU,
    state: &STATE,
    page_size: u32,
) {
    assert_eq!(active.capacity() as u32 % page_size, 0);
    assert_eq!(dfu.capacity() as u32 % page_size, 0);
    assert!(dfu.capacity() as u32 - active.capacity() as u32 >= page_size);
    assert!(2 + 2 * (active.capacity() as u32 / page_size) <= state.capacity() as u32 / STATE::WRITE_SIZE as u32);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mem_flash::MemFlash;

    #[test]
    #[should_panic]
    fn test_range_asserts() {
        const ACTIVE_SIZE: usize = 4194304 - 4096;
        const DFU_SIZE: usize = 4194304;
        const STATE_SIZE: usize = 4096;
        static ACTIVE: MemFlash<ACTIVE_SIZE, 4, 4> = MemFlash::new(0xFF);
        static DFU: MemFlash<DFU_SIZE, 4, 4> = MemFlash::new(0xFF);
        static STATE: MemFlash<STATE_SIZE, 4, 4> = MemFlash::new(0xFF);
        assert_partitions(&ACTIVE, &DFU, &STATE, 4096);
    }
}
