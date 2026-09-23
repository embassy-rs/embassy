use core::cell::RefCell;

use embassy_embedded_hal::flash::partition::BlockingPartition;
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embedded_storage::nor_flash::{NorFlash, NorFlashError, NorFlashErrorKind};

#[cfg(feature = "_verify")]
use crate::{
    AlignedBuffer,
    firmware_updater::{VerificationError, verify},
};
use crate::{DFU_DETACH_MAGIC, REVERT_MAGIC, STATE_ERASE_VALUE, SWAP_MAGIC, State};

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
        unsafe extern "C" {
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
    /// All ranges are in multiples of WRITE_SIZE bytes.
    /// N = Active partition size divided by WRITE_SIZE.
    /// | Range              | Description                                                                      |
    /// | 0..1               | Magic indicating bootloader state. BOOT_MAGIC means boot, SWAP_MAGIC means swap. |
    /// | 1..2               | Progress validity. ERASE_VALUE means valid, !ERASE_VALUE means invalid.          |
    /// | 2..(2 + 2N)        | Progress index used while swapping                                               |
    /// | (2 + 2N)..(2 + 4N) | Progress index used while reverting
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
    pub fn new(config: BootLoaderConfig<ACTIVE, DFU, STATE>) -> Self {
        Self {
            active: config.active,
            dfu: config.dfu,
            state: config.state,
        }
    }

    /// Verify the update in the DFU partition without copying either image.
    ///
    /// On `Swap` with zero copy progress, verify the signature at `signature_offset`
    /// over the SHA-512 digest of DFU bytes `0..update_len`.
    ///
    /// An invalid signature records and returns `Revert`. Other states and swaps
    /// with recorded progress are left unchanged. Flash errors are returned.
    /// Call this before [`Self::prepare_boot`] on every boot; successful
    /// verification is not recorded separately from swap progress.
    ///
    /// `aligned_buf` must satisfy [`Self::read_state`]'s buffer requirements and
    /// the [hashing requirements](crate::BlockingFirmwareUpdater::hash).
    /// The 64-byte signature at `signature_offset` must be readable from DFU.
    #[cfg(feature = "_verify")]
    pub fn verify_update(
        &mut self,
        aligned_buf: &mut [u8],
        public_key: &[u8; 32],
        update_len: u32,
        signature_offset: u32,
    ) -> Result<State, BootError> {
        let state = self.read_state(aligned_buf)?;
        if state != State::Swap || self.current_progress(aligned_buf)? != 0 {
            return Ok(state);
        }
        let mut signature = AlignedBuffer([0; 64]);
        self.dfu.read(signature_offset, signature.as_mut())?;
        match verify(&mut self.dfu, public_key, &signature.0, update_len, aligned_buf) {
            Ok(()) => Ok(State::Swap),
            Err(VerificationError::Flash(error)) => Err(BootError::Flash(error)),
            Err(VerificationError::Signature(_)) => {
                // Invalidating progress here would make an interrupted rejection
                // look like a completed swap requiring rollback.
                self.state.erase(0, self.state.capacity() as u32)?;
                let state_word = &mut aligned_buf[..STATE::WRITE_SIZE];
                state_word.fill(REVERT_MAGIC);
                self.state.write(0, state_word)?;
                Ok(State::Revert)
            }
        }
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
        const {
            core::assert!(Self::PAGE_SIZE % ACTIVE::WRITE_SIZE as u32 == 0);
            core::assert!(Self::PAGE_SIZE % ACTIVE::ERASE_SIZE as u32 == 0);
            core::assert!(Self::PAGE_SIZE % DFU::WRITE_SIZE as u32 == 0);
            core::assert!(Self::PAGE_SIZE % DFU::ERASE_SIZE as u32 == 0);
        }

        // Ensure we have enough progress pages to store copy progress
        assert_eq!(0, Self::PAGE_SIZE % aligned_buf.len() as u32);
        assert!(aligned_buf.len() >= STATE::WRITE_SIZE);
        assert_eq!(0, aligned_buf.len() % ACTIVE::WRITE_SIZE);
        assert_eq!(0, aligned_buf.len() % DFU::WRITE_SIZE);

        // Ensure our partitions are able to handle boot operations
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
                state_word.fill(REVERT_MAGIC);
                self.state.write(0, state_word)?;
            }
        }
        Ok(state)
    }

    /// Read the magic state from flash.
    ///
    /// The buffer must hold at least `STATE::WRITE_SIZE` bytes and satisfy the
    /// state flash's buffer alignment requirements.
    pub fn read_state(&mut self, aligned_buf: &mut [u8]) -> Result<State, BootError> {
        let state_word = &mut aligned_buf[..STATE::WRITE_SIZE];
        self.state.read(0, state_word)?;

        if !state_word.iter().any(|&b| b != SWAP_MAGIC) {
            Ok(State::Swap)
        } else if !state_word.iter().any(|&b| b != DFU_DETACH_MAGIC) {
            Ok(State::DfuDetach)
        } else if !state_word.iter().any(|&b| b != REVERT_MAGIC) {
            Ok(State::Revert)
        } else {
            Ok(State::Boot)
        }
    }

    fn is_swapped(&mut self, aligned_buf: &mut [u8]) -> Result<bool, BootError> {
        let page_count = self.active.capacity() / Self::PAGE_SIZE as usize;
        let progress = self.current_progress(aligned_buf)?;

        Ok(progress >= page_count * 2)
    }

    fn current_progress(&mut self, aligned_buf: &mut [u8]) -> Result<usize, BootError> {
        let write_size = STATE::WRITE_SIZE as u32;
        let state_words = self.state.capacity() / STATE::WRITE_SIZE;
        // Magic, progress validity, progress records, and a trailing reserved word.
        assert!(state_words > 3);
        let max_index = state_words - 3;
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
    assert!(2 + 4 * (active.capacity() as u32 / page_size) <= state.capacity() as u32 / STATE::WRITE_SIZE as u32);
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

#[cfg(all(test, feature = "_verify"))]
mod verification_tests {
    use ed25519_dalek::{Digest, Sha512, Signer, SigningKey};

    use super::*;
    use crate::mem_flash::MemFlash;

    type Loader = BootLoader<MemFlash<96, 16, 4>, MemFlash<112, 16, 4, 4>, MemFlash<128, 128, 4>>;

    fn pending() -> (Loader, [u8; 32]) {
        let key = SigningKey::from_bytes(&[1; 32]);
        let mut dfu = MemFlash::default();
        dfu.mem[..32].fill(0xaa);
        let signature = key.sign(&Sha512::digest(&dfu.mem[..32]));
        dfu.mem[32..96].copy_from_slice(&signature.to_bytes());
        let mut state = MemFlash::default();
        state.mem[..4].fill(SWAP_MAGIC);
        (
            BootLoader::new(BootLoaderConfig {
                active: MemFlash::new(0x55),
                dfu,
                state,
            }),
            key.verifying_key().to_bytes(),
        )
    }

    #[test]
    fn invalid_signature_preserves_images_and_reports_revert() {
        let (mut boot, key) = pending();
        boot.dfu.mem[32..96].fill(0);
        let staged = boot.dfu.mem;
        let mut buf = [0; 4];
        assert_eq!(boot.verify_update(&mut buf, &key, 32, 32), Ok(State::Revert));
        assert_eq!(boot.active.mem, [0x55; 96]);
        assert_eq!(boot.dfu.mem, staged);
        assert_eq!(boot.read_state(&mut buf), Ok(State::Revert));
        boot.dfu.pending_read_successes = Some(0);
        assert_eq!(boot.verify_update(&mut buf, &key, 32, 32), Ok(State::Revert));
        assert_eq!(boot.prepare_boot(&mut buf), Ok(State::Revert));
        assert_eq!(boot.active.mem, [0x55; 96]);
    }

    #[test]
    fn read_errors_preserve_pending_swap_and_rejection_write_errors_preserve_active() {
        // Fail both the signature read and a read while hashing the image.
        for reads in [0, 1] {
            let (mut boot, key) = pending();
            let staged = boot.dfu.mem;
            let mut buf = [0; 4];
            boot.dfu.pending_read_successes = Some(reads);
            assert_eq!(
                boot.verify_update(&mut buf, &key, 32, 32),
                Err(BootError::Flash(NorFlashErrorKind::Other))
            );
            assert_eq!(boot.read_state(&mut buf), Ok(State::Swap));
            assert_eq!(boot.active.mem, [0x55; 96]);
            assert_eq!(boot.dfu.mem, staged);
        }
        let (mut boot, key) = pending();
        boot.dfu.mem[32..96].fill(0);
        let mut buf = [0; 4];
        // Reset after erasing the state but before recording Revert.
        boot.state.pending_write_successes = Some(0);
        assert!(boot.verify_update(&mut buf, &key, 32, 32).is_err());
        boot.state.pending_write_successes = None;
        boot.dfu.pending_read_successes = Some(0);
        assert_eq!(boot.verify_update(&mut buf, &key, 32, 32), Ok(State::Boot));
        assert_eq!(boot.active.mem, [0x55; 96]);
    }

    #[test]
    fn interrupted_swap_verifies_only_before_progress_and_can_revert() {
        let wrong_key = SigningKey::from_bytes(&[2; 32]).verifying_key().to_bytes();
        for partition in 0..3 {
            for writes in 0..=24 {
                let (mut boot, key) = pending();
                let candidate: [u8; 96] = boot.dfu.mem[..96].try_into().unwrap();
                match partition {
                    0 => boot.active.pending_write_successes = Some(writes),
                    1 => boot.dfu.pending_write_successes = Some(writes),
                    _ => boot.state.pending_write_successes = Some(writes),
                }
                let mut buf = [0; 4];
                assert_eq!(boot.verify_update(&mut buf, &key, 32, 32), Ok(State::Swap));
                assert_eq!(boot.active.mem, [0x55; 96]);
                assert_eq!(&boot.dfu.mem[..96], &candidate);
                let result = boot.prepare_boot(&mut buf);
                boot.active.pending_write_successes = None;
                boot.dfu.pending_write_successes = None;
                boot.state.pending_write_successes = None;
                if result.is_err() {
                    let unstarted = boot.current_progress(&mut buf).unwrap() == 0;
                    if unstarted {
                        assert_eq!(&boot.dfu.mem[..96], &candidate);
                        // The candidate must be verified again at zero progress.
                        assert_eq!(boot.verify_update(&mut buf, &wrong_key, 32, 32), Ok(State::Revert));
                        assert_eq!(boot.prepare_boot(&mut buf), Ok(State::Revert));
                        assert_eq!(boot.active.mem, [0x55; 96]);
                        continue;
                    }
                    // A different key must not recheck already authenticated copying.
                    assert_eq!(boot.verify_update(&mut buf, &wrong_key, 32, 32), Ok(State::Swap));
                    assert_eq!(boot.prepare_boot(&mut buf), Ok(State::Swap));
                }
                assert_eq!(boot.active.mem, candidate);
                // Rollback must not try to authenticate the mixed DFU contents.
                assert_eq!(boot.verify_update(&mut buf, &wrong_key, 32, 32), Ok(State::Swap));
                assert_eq!(boot.active.mem, candidate);
                boot.prepare_boot(&mut buf).unwrap();
                assert_eq!(boot.read_state(&mut buf), Ok(State::Revert));
                assert_eq!(boot.active.mem, [0x55; 96]);
            }
        }
    }
}
