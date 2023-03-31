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

/// Extension of the embedded-storage flash type information with block size and erase value.
pub trait Flash: NorFlash + ReadNorFlash {
    /// The erase value of the flash. Typically the default of 0xFF is used, but some flashes use a different value.
    const ERASE_VALUE: u8 = 0xFF;
}

/// Trait defining the flash handles used for active and DFU partition
pub trait FlashConfig {
    /// The block size that is used when writing to flash.
    /// The update progress is tracked in blocks of this size.
    ///
    /// The size of a block must be such that
    /// 1) BLOCK_SIZE >= ACTIVE::WRITE_SIZE && ACTIVE::WRITE_SIZE % BLOCK_SIZE == 0 && ACTIVE::capacity() % BLOCK_SIZE == 0
    /// 2) BLOCK_SIZE >= DFU::WRITE_SIZE && DFU::WRITE_SIZE % BLOCK_SIZE == 0 && DFU::capacity() % BLOCK_SIZE == 0
    /// 3) BLOCK_SIZE >= STATE::WRITE_SIZE
    const BLOCK_SIZE: usize;

    /// Flash type used for the state partition.
    type STATE: Flash;
    /// Flash type used for the active partition.
    type ACTIVE: Flash;
    /// Flash type used for the dfu partition.
    type DFU: Flash;

    /// Return flash instance used to write/read to/from active partition.
    fn active(&mut self) -> &mut Self::ACTIVE;
    /// Return flash instance used to write/read to/from dfu partition.
    fn dfu(&mut self) -> &mut Self::DFU;
    /// Return flash instance used to write/read to/from bootloader state.
    fn state(&mut self) -> &mut Self::STATE;
}

/// BootLoader works with any flash implementing embedded_storage and can also work with
/// different page sizes and flash write sizes.
pub struct BootLoader {
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
    /// |    Active |          0 |      A |      B |      C |      - |
    /// |       DFU |          0 |      1 |      2 |      3 |      - |
    /// +-----------+------------+--------+--------+--------+--------+
    ///
    /// The algorithm starts by copying 'backwards', and after the first step, the layout is
    /// as follows:
    ///
    /// +-----------+------------+--------+--------+--------+--------+
    /// | Partition | Swap Index | Page 0 | Page 1 | Page 3 | Page 4 |
    /// +-----------+------------+--------+--------+--------+--------+
    /// |    Active |          1 |      A |      B |      3 |      - |
    /// |       DFU |          1 |      1 |      2 |      3 |      C |
    /// +-----------+------------+--------+--------+--------+--------+
    ///
    /// The next iteration performs the same steps
    ///
    /// +-----------+------------+--------+--------+--------+--------+
    /// | Partition | Swap Index | Page 0 | Page 1 | Page 3 | Page 4 |
    /// +-----------+------------+--------+--------+--------+--------+
    /// |    Active |          2 |      A |      2 |      3 |      - |
    /// |       DFU |          2 |      1 |      2 |      B |      C |
    /// +-----------+------------+--------+--------+--------+--------+
    ///
    /// And again until we're done
    ///
    /// +-----------+------------+--------+--------+--------+--------+
    /// | Partition | Swap Index | Page 0 | Page 1 | Page 3 | Page 4 |
    /// +-----------+------------+--------+--------+--------+--------+
    /// |    Active |          3 |      1 |      2 |      3 |      - |
    /// |       DFU |          3 |      3 |      A |      B |      C |
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
    /// |    Active |            3 |      A |      2 |      3 |      - |
    /// |       DFU |            3 |      1 |      A |      B |      C |
    /// +-----------+--------------+--------+--------+--------+--------+
    ///
    ///
    /// +-----------+--------------+--------+--------+--------+--------+
    /// | Partition | Revert Index | Page 0 | Page 1 | Page 3 | Page 4 |
    /// +-----------+--------------+--------+--------+--------+--------+
    /// |    Active |            3 |      A |      B |      3 |      - |
    /// |       DFU |            3 |      1 |      2 |      B |      C |
    /// +-----------+--------------+--------+--------+--------+--------+
    ///
    /// +-----------+--------------+--------+--------+--------+--------+
    /// | Partition | Revert Index | Page 0 | Page 1 | Page 3 | Page 4 |
    /// +-----------+--------------+--------+--------+--------+--------+
    /// |    Active |            3 |      A |      B |      C |      - |
    /// |       DFU |            3 |      1 |      2 |      3 |      C |
    /// +-----------+--------------+--------+--------+--------+--------+
    ///
    pub fn prepare_boot<P: FlashConfig>(
        &mut self,
        p: &mut P,
        aligned_block_buffer: &mut [u8],
    ) -> Result<State, BootError> {
        assert_eq!(P::BLOCK_SIZE, aligned_block_buffer.len());
        assert!(P::BLOCK_SIZE >= P::ACTIVE::WRITE_SIZE);
        assert!(P::BLOCK_SIZE >= P::DFU::WRITE_SIZE);
        assert!(P::BLOCK_SIZE >= P::STATE::WRITE_SIZE);

        assert_eq!(0, P::BLOCK_SIZE % P::ACTIVE::WRITE_SIZE);
        assert_eq!(0, P::BLOCK_SIZE % P::DFU::WRITE_SIZE);

        assert_eq!(0, p.active().capacity() % P::BLOCK_SIZE);
        assert_eq!(0, p.dfu().capacity() % P::BLOCK_SIZE);

        // Ensure we have enough progress pages to store copy progress
        assert_partitions(self.active, self.dfu, self.state, P::BLOCK_SIZE, P::STATE::WRITE_SIZE);

        // Copy contents from partition N to active
        let state = self.read_state(p, aligned_block_buffer)?;
        if state == State::Swap {
            //
            // Check if we already swapped. If we're in the swap state, this means we should revert
            // since the app has failed to mark boot as successful
            //
            if !self.is_swapped(p, aligned_block_buffer)? {
                trace!("Swapping");
                self.swap(p, aligned_block_buffer)?;
                trace!("Swapping done");
            } else {
                trace!("Reverting");
                self.revert(p, aligned_block_buffer)?;

                // Overwrite magic and reset progress
                let state_flash = p.state();
                let magic = &mut aligned_block_buffer[..P::STATE::WRITE_SIZE];

                magic.fill(!P::STATE::ERASE_VALUE);
                self.state.write_blocking(state_flash, 0, magic)?;
                self.state.wipe_blocking(state_flash)?;

                magic.fill(BOOT_MAGIC);
                self.state.write_blocking(state_flash, 0, magic)?;
            }
        }
        Ok(state)
    }

    fn is_swapped<P: FlashConfig>(&mut self, p: &mut P, aligned_block_buffer: &mut [u8]) -> Result<bool, BootError> {
        let block_count = self.active.len() / P::BLOCK_SIZE;
        let progress = self.current_progress(p, aligned_block_buffer)?;

        Ok(progress >= block_count * 2)
    }

    fn current_progress<P: FlashConfig>(
        &mut self,
        config: &mut P,
        aligned_block_buffer: &mut [u8],
    ) -> Result<usize, BootError> {
        let write_size = P::STATE::WRITE_SIZE;
        let max_index = ((self.state.len() - write_size) / write_size) - 1;

        let read_buffer = &mut aligned_block_buffer[..write_size];
        read_buffer.fill(!P::STATE::ERASE_VALUE);

        let state_flash = config.state();
        for i in 0..max_index {
            self.state
                .read_blocking(state_flash, (write_size + i * write_size) as u32, read_buffer)?;

            if read_buffer.iter().any(|&b| b == P::STATE::ERASE_VALUE) {
                return Ok(i);
            }
        }
        Ok(max_index)
    }

    fn update_progress<P: FlashConfig>(
        &mut self,
        progress_index: usize,
        p: &mut P,
        aligned_block_buffer: &mut [u8],
    ) -> Result<(), BootError> {
        let write_size = P::STATE::WRITE_SIZE;
        let write_buffer = &mut aligned_block_buffer[..write_size];
        write_buffer.fill(!P::STATE::ERASE_VALUE);
        self.state.write_blocking(
            p.state(),
            (write_size + progress_index * write_size) as u32,
            write_buffer,
        )?;
        Ok(())
    }

    fn copy_block_once_to_active<P: FlashConfig>(
        &mut self,
        progress_index: usize,
        from_dfu_offset: u32,
        to_active_offset: u32,
        p: &mut P,
        aligned_block_buffer: &mut [u8],
    ) -> Result<(), BootError> {
        if self.current_progress(p, aligned_block_buffer)? <= progress_index {
            let mut offset = from_dfu_offset;
            for chunk in aligned_block_buffer.chunks_mut(P::BLOCK_SIZE) {
                self.dfu.read_blocking(p.dfu(), offset, chunk)?;
                offset += chunk.len() as u32;
            }

            let mut offset = to_active_offset;
            for chunk in aligned_block_buffer.chunks(P::BLOCK_SIZE) {
                self.active.write_blocking(p.active(), offset, chunk)?;
                offset += chunk.len() as u32;
            }
            self.update_progress(progress_index, p, aligned_block_buffer)?;
        }
        Ok(())
    }

    fn copy_block_once_to_dfu<P: FlashConfig>(
        &mut self,
        progress_index: usize,
        from_active_offset: u32,
        to_dfu_offset: u32,
        p: &mut P,
        aligned_block_buffer: &mut [u8],
    ) -> Result<(), BootError> {
        if self.current_progress(p, aligned_block_buffer)? <= progress_index {
            let mut offset = from_active_offset;
            for chunk in aligned_block_buffer.chunks_mut(P::BLOCK_SIZE) {
                self.active.read_blocking(p.active(), offset, chunk)?;
                offset += chunk.len() as u32;
            }

            let mut offset = to_dfu_offset;
            for chunk in aligned_block_buffer.chunks(P::BLOCK_SIZE) {
                self.dfu.write_blocking(p.dfu(), offset, chunk)?;
                offset += chunk.len() as u32;
            }
            self.update_progress(progress_index, p, aligned_block_buffer)?;
        }
        Ok(())
    }

    fn swap<P: FlashConfig>(&mut self, p: &mut P, aligned_block_buffer: &mut [u8]) -> Result<(), BootError> {
        let block_size = P::BLOCK_SIZE;
        let block_count = self.active.len() / block_size;
        trace!("Block count: {}", block_count);
        for block_index in 0..block_count {
            let progress_index = block_index * 2;

            // Copy active block to the 'next' DFU block.
            let active_from_offset = ((block_count - 1 - block_index) * block_size) as u32;
            let dfu_to_offset = ((block_count - block_index) * block_size) as u32;

            if block_index % P::DFU::ERASE_SIZE == 0 {
                self.dfu
                    .erase_blocking(p.dfu(), dfu_to_offset, dfu_to_offset + P::DFU::ERASE_SIZE as u32)?;
            }

            //trace!("Copy active {} to dfu {}", active_from_offset, dfu_to_offset);
            self.copy_block_once_to_dfu(
                progress_index,
                active_from_offset,
                dfu_to_offset,
                p,
                aligned_block_buffer,
            )?;

            // Copy DFU block to the active block
            let active_to_offset = ((block_count - 1 - block_index) * block_size) as u32;
            let dfu_from_offset = ((block_count - 1 - block_index) * block_size) as u32;

            if block_index % P::ACTIVE::ERASE_SIZE == 0 {
                self.active.erase_blocking(
                    p.active(),
                    active_to_offset,
                    active_to_offset + P::ACTIVE::ERASE_SIZE as u32,
                )?;
            }

            //trace!("Copy dfy {} to active {}", dfu_from_offset, active_to_offset);
            self.copy_block_once_to_active(
                progress_index + 1,
                dfu_from_offset,
                active_to_offset,
                p,
                aligned_block_buffer,
            )?;
        }

        Ok(())
    }

    fn revert<P: FlashConfig>(&mut self, p: &mut P, aligned_block_buffer: &mut [u8]) -> Result<(), BootError> {
        let block_size = P::BLOCK_SIZE;
        let block_count = self.active.len() / block_size;
        trace!("Block count: {}", block_count);
        for block_index in 0..block_count {
            let progress_index = block_count * 2 + block_index * 2;

            // Copy the bad active block to the DFU block
            let active_from_offset = (block_index * block_size) as u32;
            let dfu_to_offset = (block_index * block_size) as u32;

            if block_index % P::DFU::ERASE_SIZE == 0 {
                self.dfu
                    .erase_blocking(p.dfu(), dfu_to_offset, dfu_to_offset + P::DFU::ERASE_SIZE as u32)?;
            }

            self.copy_block_once_to_dfu(
                progress_index,
                active_from_offset,
                dfu_to_offset,
                p,
                aligned_block_buffer,
            )?;

            // Copy the DFU block back to the active block
            let active_to_offset = (block_index * block_size) as u32;
            let dfu_from_offset = ((block_index + 1) * block_size) as u32;

            if block_index % P::ACTIVE::ERASE_SIZE == 0 {
                self.active.erase_blocking(
                    p.active(),
                    active_to_offset,
                    active_to_offset + P::ACTIVE::ERASE_SIZE as u32,
                )?;
            }

            self.copy_block_once_to_active(
                progress_index + 1,
                dfu_from_offset,
                active_to_offset,
                p,
                aligned_block_buffer,
            )?;
        }

        Ok(())
    }

    fn read_state<P: FlashConfig>(
        &mut self,
        config: &mut P,
        aligned_block_buffer: &mut [u8],
    ) -> Result<State, BootError> {
        let magic = &mut aligned_block_buffer[..P::STATE::WRITE_SIZE];
        self.state.read_blocking(config.state(), 0, magic)?;

        if !magic.iter().any(|&b| b != SWAP_MAGIC) {
            Ok(State::Swap)
        } else {
            Ok(State::Boot)
        }
    }
}

fn assert_partitions(active: Partition, dfu: Partition, state: Partition, block_size: usize, state_write_size: usize) {
    assert_eq!(active.len() % block_size, 0);
    assert_eq!(dfu.len() % block_size, 0);
    assert!(dfu.len() - active.len() >= block_size);
    assert!(2 * (active.len() / block_size) <= (state.len() - state_write_size) / state_write_size);
}

/// A flash wrapper implementing the Flash and embedded_storage traits.
pub struct BootFlash<F, const ERASE_VALUE: u8 = 0xFF>
where
    F: NorFlash + ReadNorFlash,
{
    flash: F,
}

impl<F, const ERASE_VALUE: u8> BootFlash<F, ERASE_VALUE>
where
    F: NorFlash + ReadNorFlash,
{
    /// Create a new instance of a bootable flash
    pub fn new(flash: F) -> Self {
        Self { flash }
    }
}

impl<F, const ERASE_VALUE: u8> Flash for BootFlash<F, ERASE_VALUE>
where
    F: NorFlash + ReadNorFlash,
{
    const ERASE_VALUE: u8 = ERASE_VALUE;
}

impl<F, const ERASE_VALUE: u8> ErrorType for BootFlash<F, ERASE_VALUE>
where
    F: ReadNorFlash + NorFlash,
{
    type Error = F::Error;
}

impl<F, const ERASE_VALUE: u8> NorFlash for BootFlash<F, ERASE_VALUE>
where
    F: ReadNorFlash + NorFlash,
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

impl<F, const ERASE_VALUE: u8> ReadNorFlash for BootFlash<F, ERASE_VALUE>
where
    F: ReadNorFlash + NorFlash,
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
pub struct SingleFlashConfig<'a, F, const BLOCK_SIZE: usize>
where
    F: Flash,
{
    flash: &'a mut F,
}

impl<'a, F, const BLOCK_SIZE: usize> SingleFlashConfig<'a, F, BLOCK_SIZE>
where
    F: Flash,
{
    /// Create a provider for a single flash.
    pub fn new(flash: &'a mut F) -> Self {
        Self { flash }
    }
}

impl<'a, F, const BLOCK_SIZE: usize> FlashConfig for SingleFlashConfig<'a, F, BLOCK_SIZE>
where
    F: Flash,
{
    const BLOCK_SIZE: usize = BLOCK_SIZE;
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
pub struct MultiFlashConfig<'a, ACTIVE, STATE, DFU, const BLOCK_SIZE: usize>
where
    ACTIVE: Flash,
    STATE: Flash,
    DFU: Flash,
{
    active: &'a mut ACTIVE,
    state: &'a mut STATE,
    dfu: &'a mut DFU,
}

impl<'a, ACTIVE, STATE, DFU, const BLOCK_SIZE: usize> MultiFlashConfig<'a, ACTIVE, STATE, DFU, BLOCK_SIZE>
where
    ACTIVE: Flash,
    STATE: Flash,
    DFU: Flash,
{
    /// Create a new flash provider with separate configuration for all three partitions.
    pub fn new(active: &'a mut ACTIVE, state: &'a mut STATE, dfu: &'a mut DFU) -> Self {
        Self { active, state, dfu }
    }
}

impl<'a, ACTIVE, STATE, DFU, const BLOCK_SIZE: usize> FlashConfig
    for MultiFlashConfig<'a, ACTIVE, STATE, DFU, BLOCK_SIZE>
where
    ACTIVE: Flash,
    STATE: Flash,
    DFU: Flash,
{
    const BLOCK_SIZE: usize = BLOCK_SIZE;
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
