use core::cell::RefCell;

use embassy_embedded_hal::flash::partition::BlockingPartition;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::blocking_mutex::Mutex;
use embedded_storage::nor_flash::{NorFlash, NorFlashError, NorFlashErrorKind};

use crate::{State, REVERT_MAGIC, STATE_ERASE_VALUE};

/// Describes the type of partition, either Active or Dfu.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum PartitionType {
    /// The active partition.
    Active,
    /// The DFU partition.
    Dfu,
}

/// Offset of the retry counter in the state partition.
#[cfg(feature = "recovery")]
const RETRY_COUNTER_OFFSET: usize = 1;
/// Offset of the progress validity marker in the state partition.
const PROGRESS_VALIDITY_OFFSET: usize = 1
    + if cfg!(feature = "recovery") {
        1
    } else {
        0
    };
/// Offset of the progress start in the state partition.
const PROGRESS_START_OFFSET: usize = PROGRESS_VALIDITY_OFFSET + 1;

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

/// Bootloader flash configuration holding the three flashes used by the bootloader
///
/// If only a single flash is actually used, then that flash should be partitioned into three partitions before use.
/// The easiest way to do this is to use [`BootLoaderConfig::from_linkerfile_blocking`] which will partition
/// the provided flash according to symbols defined in the linkerfile.
pub struct BootLoaderConfig<ACTIVE, DFU, STATE> {
    /// Flash type used for the active partition - the partition which will be booted from.
    pub active: ACTIVE,
    /// Flash type used for the dfu partition - the partition which will be swapped in when requested.
    pub dfu: DFU,
    /// Flash type used for the state partition.
    pub state: STATE,
}

impl<'a, ACTIVE: NorFlash, DFU: NorFlash, STATE: NorFlash>
    BootLoaderConfig<
        BlockingPartition<'a, NoopRawMutex, ACTIVE>,
        BlockingPartition<'a, NoopRawMutex, DFU>,
        BlockingPartition<'a, NoopRawMutex, STATE>,
    >
{
    /// Constructs a `BootLoaderConfig` instance from flash memory and address symbols defined in the linker file.
    ///
    /// This method initializes `BlockingPartition` instances for the active, DFU (Device Firmware Update),
    /// and state partitions, leveraging start and end addresses specified by the linker. These partitions
    /// are critical for managing firmware updates, application state, and boot operations within the bootloader.
    ///
    /// # Parameters
    /// - `active_flash`: A reference to a mutex-protected `RefCell` for the active partition's flash interface.
    /// - `dfu_flash`: A reference to a mutex-protected `RefCell` for the DFU partition's flash interface.
    /// - `state_flash`: A reference to a mutex-protected `RefCell` for the state partition's flash interface.
    ///
    /// # Safety
    /// The method contains `unsafe` blocks for dereferencing raw pointers that represent the start and end addresses
    /// of the bootloader's partitions in flash memory. It is crucial that these addresses are accurately defined
    /// in the memory.x file to prevent undefined behavior.
    ///
    /// The caller must ensure that the memory regions defined by these symbols are valid and that the flash memory
    /// interfaces provided are compatible with these regions.
    ///
    /// # Returns
    /// A `BootLoaderConfig` instance with `BlockingPartition` instances for the active, DFU, and state partitions.
    ///
    /// # Example
    /// ```ignore
    /// // Assume `active_flash`, `dfu_flash`, and `state_flash` all share the same flash memory interface.
    /// let layout = Flash::new_blocking(p.FLASH).into_blocking_regions();
    /// let flash = Mutex::new(RefCell::new(layout.bank1_region));
    ///
    /// let config = BootLoaderConfig::from_linkerfile_blocking(&flash, &flash, &flash);
    /// // `config` can now be used to create a `BootLoader` instance for managing boot operations.
    /// ```
    /// Working examples can be found in the bootloader examples folder.
    // #[cfg(target_os = "none")]
    pub fn from_linkerfile_blocking(
        active_flash: &'a Mutex<NoopRawMutex, RefCell<ACTIVE>>,
        dfu_flash: &'a Mutex<NoopRawMutex, RefCell<DFU>>,
        state_flash: &'a Mutex<NoopRawMutex, RefCell<STATE>>,
    ) -> Self {
        extern "C" {
            static __bootloader_state_start: u32;
            static __bootloader_state_end: u32;
            static __bootloader_active_start: u32;
            static __bootloader_active_end: u32;
            static __bootloader_dfu_start: u32;
            static __bootloader_dfu_end: u32;
        }

        let active = unsafe {
            let start = &__bootloader_active_start as *const u32 as u32;
            let end = &__bootloader_active_end as *const u32 as u32;
            trace!("ACTIVE: 0x{:x} - 0x{:x}", start, end);

            BlockingPartition::new(active_flash, start, end - start)
        };
        let dfu = unsafe {
            let start = &__bootloader_dfu_start as *const u32 as u32;
            let end = &__bootloader_dfu_end as *const u32 as u32;
            trace!("DFU: 0x{:x} - 0x{:x}", start, end);

            BlockingPartition::new(dfu_flash, start, end - start)
        };
        let state = unsafe {
            let start = &__bootloader_state_start as *const u32 as u32;
            let end = &__bootloader_state_end as *const u32 as u32;
            trace!("STATE: 0x{:x} - 0x{:x}", start, end);

            BlockingPartition::new(state_flash, start, end - start)
        };

        Self { active, dfu, state }
    }
}

/// BootLoader works with any flash implementing embedded_storage.
pub struct BootLoader<ACTIVE: NorFlash, DFU: NorFlash, STATE: NorFlash> {
    active: ACTIVE,
    dfu: DFU,
    /// The state partition has the following format:
    /// All ranges are in multiples of `STATE::WRITE_SIZE` bytes.
    ///
    /// | Offset                        | Description                                                                      |
    /// |-------------------------------|----------------------------------------------------------------------------------|
    /// | `0`                           | Magic indicating bootloader state. See `crate::State` and `crate::*_MAGIC`.      |
    /// | `RETRY_COUNTER_OFFSET`        | (Optional, if `recovery` feature is enabled) Retry counter for recovery mode.    |
    /// | `PROGRESS_VALIDITY_OFFSET`    | Progress validity. `STATE_ERASE_VALUE` means valid, `!STATE_ERASE_VALUE` means invalid. |
    /// | `PROGRESS_START_OFFSET..+N`   | Progress index used while swapping or reverting.                                 |
    state: STATE,
}

#[cfg(feature = "recovery")]
impl<ACTIVE: NorFlash, DFU: NorFlash, STATE: NorFlash> BootLoader<ACTIVE, DFU, STATE> {
    fn _read_retry_counter_internal(&mut self, aligned_buf: &mut [u8]) -> Result<u8, BootError> {
        self.state.read(RETRY_COUNTER_OFFSET as u32, aligned_buf)?;
        Ok(aligned_buf[0])
    }

    fn _write_retry_counter_internal(&mut self, count: u8, aligned_buf: &mut [u8]) -> Result<(), BootError> {
        aligned_buf[0] = count;
        self.state.write(RETRY_COUNTER_OFFSET as u32, &aligned_buf[..1])?;
        Ok(())
    }

    fn increment_retry_counter(&mut self, aligned_buf: &mut [u8]) -> Result<(), BootError> {
        let count = self._read_retry_counter_internal(aligned_buf)?;
        self._write_retry_counter_internal(count.saturating_add(1), aligned_buf)
    }

    fn reset_retry_counter(&mut self, aligned_buf: &mut [u8]) -> Result<(), BootError> {
        self._write_retry_counter_internal(0, aligned_buf)
    }

    /// Read the retry counter.
    pub fn read_retry_counter(&mut self, aligned_buf: &mut [u8]) -> Result<u8, BootError> {
        self._read_retry_counter_internal(aligned_buf)
    }
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
    pub fn new(config: BootLoaderConfig<ACTIVE, DFU, STATE>) -> Self {
        Self {
            active: config.active,
            dfu: config.dfu,
            state: config.state,
        }
    }

    fn set_magic_internal(&mut self, magic: u8, aligned_buf: &mut [u8]) -> Result<(), BootError> {
        let state_word = &mut aligned_buf[..STATE::WRITE_SIZE];

        // Invalidate progress
        state_word.fill(!STATE_ERASE_VALUE);
        self.state.write(PROGRESS_VALIDITY_OFFSET as u32, state_word)?;

        // Clear magic and progress
        self.state.erase(0, self.state.capacity() as u32)?;

        // Set magic
        state_word.fill(magic);
        self.state.write(0, state_word)?;

        #[cfg(feature = "recovery")]
        if magic == crate::BOOT_MAGIC {
            self.reset_retry_counter(aligned_buf)?;
        }
        Ok(())
    }

    fn copy_partition_internal<F1: NorFlash, F2: NorFlash>(
        from_flash: &mut F1,
        to_flash: &mut F2,
        size: u32,
        page_buf: &mut [u8],
    ) -> Result<(), BootError> {
        assert_eq!(size % Self::PAGE_SIZE, 0);
        assert_eq!(page_buf.len() as u32 % F1::READ_SIZE as u32, 0);
        assert_eq!(page_buf.len() as u32 % F2::WRITE_SIZE as u32, 0);

        for page_offset in (0..size).step_by(Self::PAGE_SIZE as usize) {
            to_flash.erase(page_offset, page_offset + Self::PAGE_SIZE)?;
            for chunk_offset in (0..Self::PAGE_SIZE).step_by(page_buf.len()) {
                from_flash.read(page_offset + chunk_offset, page_buf)?;
                to_flash.write(page_offset + chunk_offset, page_buf)?;
            }
        }
        Ok(())
    }

    /// Perform necessary boot preparations like swapping images.
    ///
    /// The DFU partition is assumed to be 1 page bigger than the active partition for the swap
    /// algorithm to work correctly.
    ///
    /// The provided aligned_buf argument must satisfy any alignment requirements
    /// given by the partition flashes. All flash operations will use this buffer.
    ///
    /// ## SWAPPING
    ///
    /// Assume a flash size of 3 pages for the active partition, and 4 pages for the DFU partition.
    /// The swap index contains the copy progress, as to allow continuation of the copy process on
    /// power failure. The index counter is represented within 1 or more pages (depending on total
    /// flash size), where a page X is considered swapped if index at location (`X + WRITE_SIZE`)
    /// contains a zero value. This ensures that index updates can be performed atomically and
    /// avoid a situation where the wrong index value is set (page write size is "atomic").
    ///
    ///
    /// | Partition | Swap Index | Page 0 | Page 1 | Page 3 | Page 4 |
    /// |-----------|------------|--------|--------|--------|--------|
    /// |    Active |          0 |      1 |      2 |      3 |      - |
    /// |       DFU |          0 |      4 |      5 |      6 |      X |
    ///
    /// The algorithm starts by copying 'backwards', and after the first step, the layout is
    /// as follows:
    ///
    /// | Partition | Swap Index | Page 0 | Page 1 | Page 3 | Page 4 |
    /// |-----------|------------|--------|--------|--------|--------|
    /// |    Active |          1 |      1 |      2 |      6 |      - |
    /// |       DFU |          1 |      4 |      5 |      6 |      3 |
    ///
    /// The next iteration performs the same steps
    ///
    /// | Partition | Swap Index | Page 0 | Page 1 | Page 3 | Page 4 |
    /// |-----------|------------|--------|--------|--------|--------|
    /// |    Active |          2 |      1 |      5 |      6 |      - |
    /// |       DFU |          2 |      4 |      5 |      2 |      3 |
    ///
    /// And again until we're done
    ///
    /// | Partition | Swap Index | Page 0 | Page 1 | Page 3 | Page 4 |
    /// |-----------|------------|--------|--------|--------|--------|
    /// |    Active |          3 |      4 |      5 |      6 |      - |
    /// |       DFU |          3 |      4 |      1 |      2 |      3 |
    ///
    /// ## REVERTING
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
    /// | Partition | Revert Index | Page 0 | Page 1 | Page 3 | Page 4 |
    /// |-----------|--------------|--------|--------|--------|--------|
    /// |    Active |            3 |      1 |      5 |      6 |      - |
    /// |       DFU |            3 |      4 |      1 |      2 |      3 |
    ///
    ///
    /// | Partition | Revert Index | Page 0 | Page 1 | Page 3 | Page 4 |
    /// |-----------|--------------|--------|--------|--------|--------|
    /// |    Active |            3 |      1 |      2 |      6 |      - |
    /// |       DFU |            3 |      4 |      5 |      2 |      3 |
    ///
    /// | Partition | Revert Index | Page 0 | Page 1 | Page 3 | Page 4 |
    /// |-----------|--------------|--------|--------|--------|--------|
    /// |    Active |            3 |      1 |      2 |      3 |      - |
    /// |       DFU |            3 |      4 |      5 |      6 |      3 |
    ///
    pub fn prepare_boot(&mut self, aligned_buf: &mut [u8]) -> Result<State, BootError> {
        // Runtime assertions, previously an inline const block
        assert!(Self::PAGE_SIZE % ACTIVE::WRITE_SIZE as u32 == 0);
        assert!(Self::PAGE_SIZE % ACTIVE::ERASE_SIZE as u32 == 0);
        assert!(Self::PAGE_SIZE % DFU::WRITE_SIZE as u32 == 0);
        assert!(Self::PAGE_SIZE % DFU::ERASE_SIZE as u32 == 0);

        // Ensure we have enough progress pages to store copy progress
        assert_eq!(0, Self::PAGE_SIZE % aligned_buf.len() as u32);
        assert!(aligned_buf.len() >= STATE::WRITE_SIZE);
        assert_eq!(0, aligned_buf.len() % ACTIVE::WRITE_SIZE);
        assert_eq!(0, aligned_buf.len() % DFU::WRITE_SIZE);

        // Ensure our partitions are able to handle boot operations
        assert_partitions(&self.active, &self.dfu, &self.state, Self::PAGE_SIZE);

        // Copy contents from partition N to active
        let mut current_state = self.read_state(aligned_buf)?;
        match current_state {
            State::Swap => {
                if !self.is_swapped(aligned_buf)? {
                    trace!("Swapping");
                    self.swap_partitions_internal(aligned_buf)?;
                    trace!("Swapping done");
                } else {
                    trace!("Reverting");
                    self.revert_partitions_internal(aligned_buf)?;
                    self.set_magic_internal(REVERT_MAGIC, aligned_buf)?;
                    current_state = State::Revert;
                }
            }
            #[cfg(feature = "recovery")]
            State::Backup => {
                trace!("Performing backup");
                self.perform_backup(aligned_buf)?;
                self.set_magic_internal(crate::BOOT_MAGIC, aligned_buf)?;
                current_state = State::Boot;
            }
            #[cfg(feature = "recovery")]
            State::Restore => {
                trace!("Performing restore");
                self.perform_restore(aligned_buf)?;
                self.set_magic_internal(crate::BOOT_MAGIC, aligned_buf)?;
                current_state = State::Boot;
            }
            #[cfg(feature = "recovery")]
            State::TryBoot => {
                // Nothing to do before trying to boot. Retries are handled by app.
                // If boot fails, app should call mark_booted to reset counter,
                // or eventually request DFU/restore.
            }
            State::Boot | State::Revert | State::DfuDetach => {
                // Nothing to do
            }
        }
        Ok(current_state)
    }

    fn is_swapped(&mut self, aligned_buf: &mut [u8]) -> Result<bool, BootError> {
        let page_count = self.active.capacity() / Self::PAGE_SIZE as usize;
        let progress = self.current_progress(aligned_buf)?;

        Ok(progress >= page_count * 2)
    }

    fn current_progress(&mut self, aligned_buf: &mut [u8]) -> Result<usize, BootError> {
        let write_size = STATE::WRITE_SIZE;
        // Calculate max_index based on available space after PROGRESS_START_OFFSET
        let max_index = (self.state.capacity() - PROGRESS_START_OFFSET) / write_size;
        let state_word = &mut aligned_buf[..write_size];

        self.state.read(PROGRESS_VALIDITY_OFFSET as u32, state_word)?;
        if state_word.iter().any(|&b| b != STATE_ERASE_VALUE) {
            // Progress is invalid
            return Ok(max_index);
        }

        for index in 0..max_index {
            self.state
                .read((PROGRESS_START_OFFSET + index * write_size) as u32, state_word)?;

            if state_word.iter().any(|&b| b == STATE_ERASE_VALUE) {
                return Ok(index);
            }
        }
        Ok(max_index)
    }

    fn update_progress(&mut self, progress_index: usize, aligned_buf: &mut [u8]) -> Result<(), BootError> {
        let write_size = STATE::WRITE_SIZE;
        let state_word = &mut aligned_buf[..write_size];
        state_word.fill(!STATE_ERASE_VALUE);
        self.state.write(
            (PROGRESS_START_OFFSET + progress_index * write_size) as u32,
            state_word,
        )?;
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

    fn swap_partitions_internal(&mut self, aligned_buf: &mut [u8]) -> Result<(), BootError> {
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

    fn revert_partitions_internal(&mut self, aligned_buf: &mut [u8]) -> Result<(), BootError> {
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

    /// Perform a swap of the active and DFU partitions.
    pub fn perform_swap(&mut self, aligned_buf: &mut [u8]) -> Result<(), BootError> {
        self.swap_partitions_internal(aligned_buf)
    }

    /// Perform a backup of the active partition to the DFU partition.
    #[cfg(feature = "recovery")]
    pub fn perform_backup(&mut self, aligned_buf: &mut [u8]) -> Result<(), BootError> {
        let active_size = self.active.capacity() as u32;
        self.dfu.erase(0, active_size)?;
        Self::copy_partition_internal(&mut self.active, &mut self.dfu, active_size, aligned_buf)
    }

    /// Perform a restore of the DFU partition to the active partition.
    #[cfg(feature = "recovery")]
    pub fn perform_restore(&mut self, aligned_buf: &mut [u8]) -> Result<(), BootError> {
        let active_size = self.active.capacity() as u32;
        self.active.erase(0, active_size)?;
        Self::copy_partition_internal(&mut self.dfu, &mut self.active, active_size, aligned_buf)
    }

    fn read_state(&mut self, aligned_buf: &mut [u8]) -> Result<State, BootError> {
        let state_word = &mut aligned_buf[..STATE::WRITE_SIZE];
        self.state.read(0, state_word)?;
        // The State::from implementation handles all magic numbers including recovery ones.
        Ok(State::from(&state_word[..STATE::WRITE_SIZE]))
    }

    /// Verify a partition by reading its content page by page.
    /// The `page_buf` must be large enough to read a whole page, or a reasonable chunk size.
    /// Its length must be a multiple of the read alignment of the partition.
    pub fn verify_partition(
        &mut self,
        partition_type: PartitionType,
        page_buf: &mut [u8],
    ) -> Result<(), BootError> {
        match partition_type {
            PartitionType::Active => {
                assert_eq!(page_buf.len() as u32 % ACTIVE::READ_SIZE as u32, 0);
                for offset in (0..self.active.capacity() as u32).step_by(page_buf.len()) {
                    self.active.read(offset, page_buf).map_err(BootError::from)?;
                }
            }
            PartitionType::Dfu => {
                assert_eq!(page_buf.len() as u32 % DFU::READ_SIZE as u32, 0);
                for offset in (0..self.dfu.capacity() as u32).step_by(page_buf.len()) {
                    self.dfu.read(offset, page_buf).map_err(BootError::from)?;
                }
            }
        }
        Ok(())
    }

    /// Mark the current boot as successful.
    pub fn mark_booted(&mut self, aligned_buf: &mut [u8]) -> Result<(), BootError> {
        self.set_magic_internal(crate::BOOT_MAGIC, aligned_buf)
    }

    /// Mark that a restore should be performed on the next boot.
    #[cfg(feature = "recovery")]
    pub fn mark_restore(&mut self, aligned_buf: &mut [u8]) -> Result<(), BootError> {
        self.set_magic_internal(crate::RESTORE_MAGIC, aligned_buf)
    }

    /// Mark that a try boot should be performed on the next boot.
    #[cfg(feature = "recovery")]
    pub fn mark_try_boot(&mut self, aligned_buf: &mut [u8]) -> Result<(), BootError> {
        self.set_magic_internal(crate::TRY_BOOT_MAGIC, aligned_buf)
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
    // DFU partition has to be bigger than ACTIVE partition to handle swap algorithm
    assert!(dfu.capacity() as u32 - active.capacity() as u32 >= page_size);
    // Ensure there's enough space for magic, optional retry counter, progress validity, and progress markers
    let required_state_len = PROGRESS_START_OFFSET + (2 * (active.capacity() / page_size as usize) * STATE::WRITE_SIZE);
    assert!(state.capacity() >= required_state_len);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mem_flash::MemFlash;

    const PAGE_SIZE: usize = 4096;
    const ACTIVE_PARTITION_SIZE: usize = 2 * PAGE_SIZE;
    const DFU_PARTITION_SIZE: usize = 3 * PAGE_SIZE; // Must be at least PAGE_SIZE larger than active
    const STATE_PARTITION_SIZE: usize = PAGE_SIZE; // Must be large enough for magic, counter, progress

    fn new_bootloader() -> BootLoader<
        MemFlash<ACTIVE_PARTITION_SIZE, PAGE_SIZE, 4>,
        MemFlash<DFU_PARTITION_SIZE, PAGE_SIZE, 4>,
        MemFlash<STATE_PARTITION_SIZE, PAGE_SIZE, 4>,
    > {
        let active_flash = MemFlash::new(STATE_ERASE_VALUE);
        let dfu_flash = MemFlash::new(STATE_ERASE_VALUE);
        let state_flash = MemFlash::new(STATE_ERASE_VALUE);

        let config = BootLoaderConfig {
            active: active_flash,
            dfu: dfu_flash,
            state: state_flash,
        };
        BootLoader::new(config)
    }

    #[test]
    #[should_panic]
    fn test_range_asserts() {
        const ACTIVE_SIZE: usize = 4194304 - 4096;
        const DFU_SIZE: usize = 4194304;
        const STATE_SIZE: usize = 4096; // This would be too small with recovery enabled
        static ACTIVE: MemFlash<ACTIVE_SIZE, 4, 4> = MemFlash::new(0xFF);
        static DFU: MemFlash<DFU_SIZE, 4, 4> = MemFlash::new(0xFF);
        static STATE: MemFlash<STATE_SIZE, 4, 4> = MemFlash::new(0xFF);
        assert_partitions(&ACTIVE, &DFU, &STATE, PAGE_SIZE as u32);
    }

    #[cfg(feature = "recovery")]
    mod recovery_tests {
        use super::*;
        use crate::{BOOT_MAGIC, RESTORE_MAGIC, TRY_BOOT_MAGIC};

        #[test]
        fn test_retry_counter_initial_value_after_mark_booted() {
            let mut bootloader = new_bootloader();
            let mut aligned_buf = [STATE_ERASE_VALUE; PAGE_SIZE]; // Adjusted to typical page size for flash ops
            assert!(aligned_buf.len() >= STATE::WRITE_SIZE);


            // Simulate a fresh boot or successful boot
            bootloader.mark_booted(&mut aligned_buf).unwrap();
            let counter = bootloader.read_retry_counter(&mut aligned_buf).unwrap();
            assert_eq!(counter, 0, "Retry counter should be 0 after mark_booted");
        }

        #[test]
        fn test_retry_counter_increment() {
            let mut bootloader = new_bootloader();
            let mut aligned_buf = [STATE_ERASE_VALUE; PAGE_SIZE];
             assert!(aligned_buf.len() >= STATE::WRITE_SIZE);

            // Reset counter first (e.g. by mark_booted)
            bootloader.mark_booted(&mut aligned_buf).unwrap();
            assert_eq!(bootloader.read_retry_counter(&mut aligned_buf).unwrap(), 0);

            bootloader.increment_retry_counter(&mut aligned_buf).unwrap();
            assert_eq!(bootloader.read_retry_counter(&mut aligned_buf).unwrap(), 1);

            bootloader.increment_retry_counter(&mut aligned_buf).unwrap();
            assert_eq!(bootloader.read_retry_counter(&mut aligned_buf).unwrap(), 2);
        }

        #[test]
        fn test_retry_counter_reset() {
            let mut bootloader = new_bootloader();
            let mut aligned_buf = [STATE_ERASE_VALUE; PAGE_SIZE];
            assert!(aligned_buf.len() >= STATE::WRITE_SIZE);

            bootloader.increment_retry_counter(&mut aligned_buf).unwrap();
            bootloader.increment_retry_counter(&mut aligned_buf).unwrap();
            assert_ne!(bootloader.read_retry_counter(&mut aligned_buf).unwrap(), 0);

            bootloader.reset_retry_counter(&mut aligned_buf).unwrap();
            assert_eq!(bootloader.read_retry_counter(&mut aligned_buf).unwrap(), 0);
        }

        #[test]
        fn test_mark_booted_resets_retry_counter() {
            let mut bootloader = new_bootloader();
            let mut aligned_buf = [STATE_ERASE_VALUE; PAGE_SIZE];
            assert!(aligned_buf.len() >= STATE::WRITE_SIZE);

            bootloader.increment_retry_counter(&mut aligned_buf).unwrap();
            assert_ne!(bootloader.read_retry_counter(&mut aligned_buf).unwrap(), 0);

            bootloader.mark_booted(&mut aligned_buf).unwrap();
            assert_eq!(bootloader.read_retry_counter(&mut aligned_buf).unwrap(), 0);
        }

        #[test]
        fn test_mark_try_boot_sets_magic_and_does_not_modify_counter() {
            let mut bootloader = new_bootloader();
            let mut aligned_buf = [STATE_ERASE_VALUE; PAGE_SIZE];
            assert!(aligned_buf.len() >= STATE::WRITE_SIZE);

            // Set counter to a known value
            bootloader.reset_retry_counter(&mut aligned_buf).unwrap();
            bootloader.increment_retry_counter(&mut aligned_buf).unwrap(); // counter = 1
            let initial_counter = bootloader.read_retry_counter(&mut aligned_buf).unwrap();
            assert_eq!(initial_counter, 1);

            bootloader.mark_try_boot(&mut aligned_buf).unwrap();
            // mark_try_boot should now only set magic and not touch the counter.
            assert_eq!(read_magic(&mut bootloader, &mut aligned_buf), TRY_BOOT_MAGIC);
            assert_eq!(bootloader.read_retry_counter(&mut aligned_buf).unwrap(), initial_counter);
        }

        #[test]
        fn test_mark_restore_does_not_modify_retry_counter() {
            let mut bootloader = new_bootloader();
            let mut aligned_buf = [STATE_ERASE_VALUE; PAGE_SIZE];
            assert!(aligned_buf.len() >= STATE::WRITE_SIZE);

            bootloader.reset_retry_counter(&mut aligned_buf).unwrap();
            bootloader.increment_retry_counter(&mut aligned_buf).unwrap(); // counter = 1
            let initial_counter = bootloader.read_retry_counter(&mut aligned_buf).unwrap();

            bootloader.mark_restore(&mut aligned_buf).unwrap();
            // mark_restore should not change the counter itself, only set_magic_internal might if it was BOOT_MAGIC
            assert_eq!(bootloader.read_retry_counter(&mut aligned_buf).unwrap(), initial_counter);
        }

        fn read_magic(bootloader: &mut BootLoader<
            MemFlash<ACTIVE_PARTITION_SIZE, PAGE_SIZE, 4>,
            MemFlash<DFU_PARTITION_SIZE, PAGE_SIZE, 4>,
            MemFlash<STATE_PARTITION_SIZE, PAGE_SIZE, 4>,
        >, aligned_buf: &mut [u8]) -> u8 {
            bootloader.state.read(0, &mut aligned_buf[..STATE::WRITE_SIZE]).unwrap();
            aligned_buf[0]
        }

        fn check_progress_invalidated(bootloader: &mut BootLoader<
            MemFlash<ACTIVE_PARTITION_SIZE, PAGE_SIZE, 4>,
            MemFlash<DFU_PARTITION_SIZE, PAGE_SIZE, 4>,
            MemFlash<STATE_PARTITION_SIZE, PAGE_SIZE, 4>,
        >, aligned_buf: &mut [u8]){
            bootloader.state.read(PROGRESS_VALIDITY_OFFSET as u32, &mut aligned_buf[..STATE::WRITE_SIZE]).unwrap();
            assert!(aligned_buf[..STATE::WRITE_SIZE].iter().all(|&b| b == !STATE_ERASE_VALUE), "Progress should be invalidated");
        }

        #[test]
        fn test_mark_booted_sets_magic_and_invalidates_progress() {
            let mut bootloader = new_bootloader();
            let mut aligned_buf = [STATE_ERASE_VALUE; PAGE_SIZE];
            assert!(aligned_buf.len() >= STATE::WRITE_SIZE);

            // Set some dummy progress first by calling update_progress
            bootloader.update_progress(0, &mut aligned_buf).unwrap();
            // Make progress valid
            aligned_buf[..STATE::WRITE_SIZE].fill(STATE_ERASE_VALUE);
            bootloader.state.write(PROGRESS_VALIDITY_OFFSET as u32, &aligned_buf[..STATE::WRITE_SIZE]).unwrap();


            bootloader.mark_booted(&mut aligned_buf).unwrap();
            assert_eq!(read_magic(&mut bootloader, &mut aligned_buf), BOOT_MAGIC);
            check_progress_invalidated(&mut bootloader, &mut aligned_buf);
        }

        #[test]
        fn test_mark_restore_sets_magic_and_invalidates_progress() {
            let mut bootloader = new_bootloader();
            let mut aligned_buf = [STATE_ERASE_VALUE; PAGE_SIZE];
            assert!(aligned_buf.len() >= STATE::WRITE_SIZE);

            bootloader.update_progress(0, &mut aligned_buf).unwrap();
            aligned_buf[..STATE::WRITE_SIZE].fill(STATE_ERASE_VALUE);
            bootloader.state.write(PROGRESS_VALIDITY_OFFSET as u32, &aligned_buf[..STATE::WRITE_SIZE]).unwrap();

            bootloader.mark_restore(&mut aligned_buf).unwrap();
            assert_eq!(read_magic(&mut bootloader, &mut aligned_buf), RESTORE_MAGIC);
            check_progress_invalidated(&mut bootloader, &mut aligned_buf);
        }

        #[test]
        fn test_mark_try_boot_sets_magic_and_invalidates_progress() {
            let mut bootloader = new_bootloader();
            let mut aligned_buf = [STATE_ERASE_VALUE; PAGE_SIZE];
            assert!(aligned_buf.len() >= STATE::WRITE_SIZE);

            bootloader.update_progress(0, &mut aligned_buf).unwrap();
            aligned_buf[..STATE::WRITE_SIZE].fill(STATE_ERASE_VALUE);
            bootloader.state.write(PROGRESS_VALIDITY_OFFSET as u32, &aligned_buf[..STATE::WRITE_SIZE]).unwrap();

            bootloader.mark_try_boot(&mut aligned_buf).unwrap();
            assert_eq!(read_magic(&mut bootloader, &mut aligned_buf), TRY_BOOT_MAGIC);
            check_progress_invalidated(&mut bootloader, &mut aligned_buf);
        }

        #[test]
        fn test_perform_backup() {
            let mut bootloader = new_bootloader();
            let mut aligned_buf = [STATE_ERASE_VALUE; PAGE_SIZE];
            assert!(aligned_buf.len() >= ACTIVE::WRITE_SIZE && aligned_buf.len() >= DFU::WRITE_SIZE);
            assert_eq!(aligned_buf.len() % ACTIVE::READ_SIZE, 0);
            assert_eq!(aligned_buf.len() % DFU::WRITE_SIZE, 0);


            // Fill active partition with known data
            let mut active_data = [0u8; ACTIVE_PARTITION_SIZE];
            for i in 0..ACTIVE_PARTITION_SIZE {
                active_data[i] = (i % 256) as u8;
            }
            for offset in (0..ACTIVE_PARTITION_SIZE).step_by(aligned_buf.len()) {
                let chunk_len = core::cmp::min(aligned_buf.len(), ACTIVE_PARTITION_SIZE - offset);
                bootloader.active.write(offset as u32, &active_data[offset..offset + chunk_len]).unwrap();
            }

            bootloader.perform_backup(&mut aligned_buf).unwrap();

            // Verify DFU partition contains the same data
            let mut dfu_data_read = [0u8; ACTIVE_PARTITION_SIZE];
            for offset in (0..ACTIVE_PARTITION_SIZE).step_by(aligned_buf.len()) {
                 let chunk_len = core::cmp::min(aligned_buf.len(), ACTIVE_PARTITION_SIZE - offset);
                bootloader.dfu.read(offset as u32, &mut dfu_data_read[offset..offset+chunk_len]).unwrap();
            }
            assert_eq!(&active_data[..], &dfu_data_read[..]);
        }

        #[test]
        fn test_perform_restore() {
            let mut bootloader = new_bootloader();
            let mut aligned_buf = [STATE_ERASE_VALUE; PAGE_SIZE];
            assert!(aligned_buf.len() >= DFU::READ_SIZE && aligned_buf.len() >= ACTIVE::WRITE_SIZE);
            assert_eq!(aligned_buf.len() % DFU::READ_SIZE, 0);
            assert_eq!(aligned_buf.len() % ACTIVE::WRITE_SIZE, 0);


            // Fill DFU partition with known data (only up to active size, as that's what restore will copy)
            let mut dfu_data = [0u8; ACTIVE_PARTITION_SIZE];
            for i in 0..ACTIVE_PARTITION_SIZE {
                dfu_data[i] = (i % 256) as u8;
            }
             for offset in (0..ACTIVE_PARTITION_SIZE).step_by(aligned_buf.len()) {
                let chunk_len = core::cmp::min(aligned_buf.len(), ACTIVE_PARTITION_SIZE - offset);
                bootloader.dfu.write(offset as u32, &dfu_data[offset..offset + chunk_len]).unwrap();
            }


            bootloader.perform_restore(&mut aligned_buf).unwrap();

            // Verify active partition now contains DFU data
            let mut active_data_read = [0u8; ACTIVE_PARTITION_SIZE];
            for offset in (0..ACTIVE_PARTITION_SIZE).step_by(aligned_buf.len()) {
                let chunk_len = core::cmp::min(aligned_buf.len(), ACTIVE_PARTITION_SIZE - offset);
                bootloader.active.read(offset as u32, &mut active_data_read[offset..offset+chunk_len]).unwrap();
            }
            assert_eq!(&dfu_data[..], &active_data_read[..]);
        }

        #[test]
        fn test_prepare_boot_with_backup_magic() {
            let mut bootloader = new_bootloader();
            let mut aligned_buf = [STATE_ERASE_VALUE; PAGE_SIZE];

            // Setup active partition with known data
            let mut active_data = [0u8; ACTIVE_PARTITION_SIZE];
            for i in 0..ACTIVE_PARTITION_SIZE {
                active_data[i] = (i % 256) as u8;
            }
            for offset in (0..ACTIVE_PARTITION_SIZE).step_by(aligned_buf.len()) {
                let chunk_len = core::cmp::min(aligned_buf.len(), ACTIVE_PARTITION_SIZE - offset);
                bootloader.active.write(offset as u32, &active_data[offset..offset + chunk_len]).unwrap();
            }

            // Mark for backup
            aligned_buf[0] = crate::BACKUP_MAGIC;
            bootloader.state.write(0, &aligned_buf[..STATE::WRITE_SIZE]).unwrap();


            let state = bootloader.prepare_boot(&mut aligned_buf).unwrap();
            assert_eq!(state, State::Boot);
            assert_eq!(read_magic(&mut bootloader, &mut aligned_buf), BOOT_MAGIC);
            assert_eq!(bootloader.read_retry_counter(&mut aligned_buf).unwrap(), 0);

            // Verify DFU partition contains the backup
            let mut dfu_data_read = [0u8; ACTIVE_PARTITION_SIZE];
             for offset in (0..ACTIVE_PARTITION_SIZE).step_by(aligned_buf.len()) {
                 let chunk_len = core::cmp::min(aligned_buf.len(), ACTIVE_PARTITION_SIZE - offset);
                bootloader.dfu.read(offset as u32, &mut dfu_data_read[offset..offset+chunk_len]).unwrap();
            }
            assert_eq!(&active_data[..], &dfu_data_read[..]);
        }

        #[test]
        fn test_prepare_boot_with_restore_magic() {
            let mut bootloader = new_bootloader();
            let mut aligned_buf = [STATE_ERASE_VALUE; PAGE_SIZE];

            // Setup DFU partition with known data
            let mut dfu_data = [0u8; ACTIVE_PARTITION_SIZE];
            for i in 0..ACTIVE_PARTITION_SIZE {
                dfu_data[i] = (i % 256) as u8;
            }
            for offset in (0..ACTIVE_PARTITION_SIZE).step_by(aligned_buf.len()) {
                let chunk_len = core::cmp::min(aligned_buf.len(), ACTIVE_PARTITION_SIZE - offset);
                bootloader.dfu.write(offset as u32, &dfu_data[offset..offset + chunk_len]).unwrap();
            }

            // Mark for restore
            aligned_buf[0] = crate::RESTORE_MAGIC;
            bootloader.state.write(0, &aligned_buf[..STATE::WRITE_SIZE]).unwrap();


            let state = bootloader.prepare_boot(&mut aligned_buf).unwrap();
            assert_eq!(state, State::Boot);
            assert_eq!(read_magic(&mut bootloader, &mut aligned_buf), BOOT_MAGIC);
            assert_eq!(bootloader.read_retry_counter(&mut aligned_buf).unwrap(), 0);

            // Verify active partition contains the restored data
            let mut active_data_read = [0u8; ACTIVE_PARTITION_SIZE];
            for offset in (0..ACTIVE_PARTITION_SIZE).step_by(aligned_buf.len()) {
                let chunk_len = core::cmp::min(aligned_buf.len(), ACTIVE_PARTITION_SIZE - offset);
                bootloader.active.read(offset as u32, &mut active_data_read[offset..offset+chunk_len]).unwrap();
            }
            assert_eq!(&dfu_data[..], &active_data_read[..]);
        }

        #[test]
        fn test_prepare_boot_with_try_boot_magic() {
            let mut bootloader = new_bootloader();
            let mut aligned_buf = [STATE_ERASE_VALUE; PAGE_SIZE];

            // Set initial active data
            let mut initial_active_data = [7u8; ACTIVE_PARTITION_SIZE];
            bootloader.active.write(0, &initial_active_data).unwrap();
            // Set initial DFU data
            let mut initial_dfu_data = [8u8; DFU_PARTITION_SIZE];
            bootloader.dfu.write(0, &initial_dfu_data).unwrap();


            // Mark for try_boot and set a retry count
            bootloader.mark_try_boot(&mut aligned_buf).unwrap();
            let initial_retry_count = bootloader.read_retry_counter(&mut aligned_buf).unwrap();
            assert_eq!(read_magic(&mut bootloader, &mut aligned_buf), TRY_BOOT_MAGIC);


            let state = bootloader.prepare_boot(&mut aligned_buf).unwrap();
            assert_eq!(state, State::TryBoot); // State should remain TryBoot
            assert_eq!(read_magic(&mut bootloader, &mut aligned_buf), TRY_BOOT_MAGIC); // Magic should remain
            assert_eq!(bootloader.read_retry_counter(&mut aligned_buf).unwrap(), initial_retry_count); // Counter should be unchanged by prepare_boot

            // Verify no partition copy occurred
            let mut current_active_data = [0u8; ACTIVE_PARTITION_SIZE];
            bootloader.active.read(0, &mut current_active_data).unwrap();
            assert_eq!(initial_active_data, current_active_data);

            let mut current_dfu_data = [0u8; DFU_PARTITION_SIZE];
            bootloader.dfu.read(0, &mut current_dfu_data).unwrap();
            assert_eq!(initial_dfu_data, current_dfu_data);
        }

        #[test]
        fn test_verify_partition() {
            let mut bootloader = new_bootloader();
            let mut page_buf = [STATE_ERASE_VALUE; PAGE_SIZE]; // Use full page for verify buffer

            // Fill active partition with known data
            let mut active_data = [0u8; ACTIVE_PARTITION_SIZE];
            for i in 0..ACTIVE_PARTITION_SIZE {
                active_data[i] = (i % 250) as u8; // Use a different pattern
            }
            for offset in (0..ACTIVE_PARTITION_SIZE).step_by(page_buf.len()) {
                 let chunk_len = core::cmp::min(page_buf.len(), ACTIVE_PARTITION_SIZE - offset);
                bootloader.active.write(offset as u32, &active_data[offset..offset + chunk_len]).unwrap();
            }


            // Fill DFU partition with different known data
            let mut dfu_data = [0u8; DFU_PARTITION_SIZE];
            for i in 0..DFU_PARTITION_SIZE {
                dfu_data[i] = ((i + 10) % 250) as u8; // Use a different pattern
            }
            for offset in (0..DFU_PARTITION_SIZE).step_by(page_buf.len()) {
                let chunk_len = core::cmp::min(page_buf.len(), DFU_PARTITION_SIZE - offset);
                bootloader.dfu.write(offset as u32, &dfu_data[offset..offset+chunk_len]).unwrap();
            }

            assert!(bootloader.verify_partition(PartitionType::Active, &mut page_buf).is_ok());
            assert!(bootloader.verify_partition(PartitionType::Dfu, &mut page_buf).is_ok());

            // Intentionally corrupt a byte in active partition and verify it fails (optional)
            // bootloader.active.write(PAGE_SIZE as u32 / 2, &[0xDE]).unwrap();
            // assert!(bootloader.verify_partition(PartitionType::Active, &mut page_buf).is_err());
        }
    }
}
