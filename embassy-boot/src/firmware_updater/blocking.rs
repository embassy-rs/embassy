use digest::Digest;
#[cfg(target_os = "none")]
use embassy_embedded_hal::flash::partition::BlockingPartition;
#[cfg(target_os = "none")]
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embedded_storage::nor_flash::NorFlash;

use super::FirmwareUpdaterConfig;
use crate::{BOOT_MAGIC, DFU_DETACH_MAGIC, FirmwareUpdaterError, STATE_ERASE_VALUE, SWAP_MAGIC, State};

/// Blocking FirmwareUpdater is an application API for interacting with the BootLoader without the ability to
/// 'mess up' the internal bootloader state
pub struct BlockingFirmwareUpdater<'d, DFU: NorFlash, STATE: NorFlash> {
    dfu: DFU,
    state: BlockingFirmwareState<'d, STATE>,
    last_erased_dfu_sector_index: Option<usize>,
}

#[cfg(target_os = "none")]
impl<'a, DFU: NorFlash, STATE: NorFlash>
    FirmwareUpdaterConfig<BlockingPartition<'a, NoopRawMutex, DFU>, BlockingPartition<'a, NoopRawMutex, STATE>>
{
    /// Constructs a `FirmwareUpdaterConfig` instance from flash memory and address symbols defined in the linker file.
    ///
    /// This method initializes `BlockingPartition` instances for the DFU (Device Firmware Update), and state
    /// partitions, leveraging start and end addresses specified by the linker. These partitions are critical
    /// for managing firmware updates, application state, and boot operations within the bootloader.
    ///
    /// # Parameters
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
    /// A `FirmwareUpdaterConfig` instance with `BlockingPartition` instances for the DFU, and state partitions.
    ///
    /// # Example
    /// ```ignore
    /// // Assume `dfu_flash`, and `state_flash` share the same flash memory interface.
    /// let layout = Flash::new_blocking(p.FLASH).into_blocking_regions();
    /// let flash = Mutex::new(RefCell::new(layout.bank1_region));
    ///
    /// let config = FirmwareUpdaterConfig::from_linkerfile_blocking(&flash, &flash);
    /// // `config` can now be used to create a `FirmwareUpdater` instance for managing boot operations.
    /// ```
    /// Working examples can be found in the bootloader examples folder.
    pub fn from_linkerfile_blocking(
        dfu_flash: &'a embassy_sync::blocking_mutex::Mutex<NoopRawMutex, core::cell::RefCell<DFU>>,
        state_flash: &'a embassy_sync::blocking_mutex::Mutex<NoopRawMutex, core::cell::RefCell<STATE>>,
    ) -> Self {
        unsafe extern "C" {
            static __bootloader_state_start: u32;
            static __bootloader_state_end: u32;
            static __bootloader_dfu_start: u32;
            static __bootloader_dfu_end: u32;
        }

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

        Self { dfu, state }
    }
}

impl<'d, DFU: NorFlash, STATE: NorFlash> BlockingFirmwareUpdater<'d, DFU, STATE> {
    /// Create a firmware updater instance with partition ranges for the update and state partitions.
    ///
    /// # Safety
    ///
    /// The `aligned` buffer must have a size of STATE::WRITE_SIZE, and follow the alignment rules for the flash being read from
    /// and written to.
    pub fn new(config: FirmwareUpdaterConfig<DFU, STATE>, aligned: &'d mut [u8]) -> Self {
        Self {
            dfu: config.dfu,
            state: BlockingFirmwareState::new(config.state, aligned),
            last_erased_dfu_sector_index: None,
        }
    }

    /// Obtain the current state.
    ///
    /// This is useful to check if the bootloader has just done a swap, in order
    /// to do verifications and self-tests of the new image before calling
    /// `mark_booted`.
    pub fn get_state(&mut self) -> Result<State, FirmwareUpdaterError> {
        self.state.get_state()
    }

    /// Verify the DFU given a public key. If there is an error then DO NOT
    /// proceed with updating the firmware as it must be signed with a
    /// corresponding private key (otherwise it could be malicious firmware).
    ///
    /// Mark to trigger firmware swap on next boot if verify succeeds.
    ///
    /// If the "ed25519-salty" feature is set (or another similar feature) then the signature is expected to have
    /// been generated from a SHA-512 digest of the firmware bytes.
    ///
    /// If no signature feature is set then this method will always return a
    /// signature error.
    #[cfg(feature = "_verify")]
    pub fn verify_and_mark_updated(
        &mut self,
        _public_key: &[u8; 32],
        _signature: &[u8; 64],
        _update_len: u32,
    ) -> Result<(), FirmwareUpdaterError> {
        assert!(_update_len <= self.dfu.capacity() as u32);

        self.state.verify_booted()?;

        #[cfg(feature = "ed25519-dalek")]
        {
            use ed25519_dalek::{Signature, SignatureError, Verifier, VerifyingKey};

            use crate::digest_adapters::ed25519_dalek::Sha512;

            let into_signature_error = |e: SignatureError| FirmwareUpdaterError::Signature(e.into());

            let public_key = VerifyingKey::from_bytes(_public_key).map_err(into_signature_error)?;
            let signature = Signature::from_bytes(_signature);

            let mut message = [0; 64];
            let mut chunk_buf = [0; 2];
            self.hash::<Sha512>(_update_len, &mut chunk_buf, &mut message)?;

            public_key.verify(&message, &signature).map_err(into_signature_error)?;
            return self.state.mark_updated();
        }
        #[cfg(feature = "ed25519-salty")]
        {
            use salty::{PublicKey, Signature};

            use crate::digest_adapters::salty::Sha512;

            fn into_signature_error<E>(_: E) -> FirmwareUpdaterError {
                FirmwareUpdaterError::Signature(signature::Error::default())
            }

            let public_key = PublicKey::try_from(_public_key).map_err(into_signature_error)?;
            let signature = Signature::try_from(_signature).map_err(into_signature_error)?;

            let mut message = [0; 64];
            let mut chunk_buf = [0; 2];
            self.hash::<Sha512>(_update_len, &mut chunk_buf, &mut message)?;

            let r = public_key.verify(&message, &signature);
            trace!(
                "Verifying with public key {}, signature {} and message {} yields ok: {}",
                public_key.to_bytes(),
                signature.to_bytes(),
                message,
                r.is_ok()
            );
            r.map_err(into_signature_error)?;
            return self.state.mark_updated();
        }
        #[cfg(not(any(feature = "ed25519-dalek", feature = "ed25519-salty")))]
        {
            Err(FirmwareUpdaterError::Signature(signature::Error::new()))
        }
    }

    /// Verify the update in DFU with any digest.
    pub fn hash<D: Digest>(
        &mut self,
        update_len: u32,
        chunk_buf: &mut [u8],
        output: &mut [u8],
    ) -> Result<(), FirmwareUpdaterError> {
        let mut digest = D::new();
        for offset in (0..update_len).step_by(chunk_buf.len()) {
            self.dfu.read(offset, chunk_buf)?;
            let len = core::cmp::min((update_len - offset) as usize, chunk_buf.len());
            digest.update(&chunk_buf[..len]);
        }
        output.copy_from_slice(digest.finalize().as_slice());
        Ok(())
    }

    /// Read a slice of data from the DFU storage peripheral, starting the read
    /// operation at the given address offset, and reading `buf.len()` bytes.
    ///
    /// # Errors
    ///
    /// Returns an error if the arguments are not aligned or out of bounds.
    pub fn read_dfu(&mut self, offset: u32, buf: &mut [u8]) -> Result<(), FirmwareUpdaterError> {
        self.dfu.read(offset, buf)?;
        Ok(())
    }

    /// Mark to trigger firmware swap on next boot.
    #[cfg(not(feature = "_verify"))]
    pub fn mark_updated(&mut self) -> Result<(), FirmwareUpdaterError> {
        self.state.mark_updated()
    }

    /// Mark to trigger USB DFU device on next boot.
    pub fn mark_dfu(&mut self) -> Result<(), FirmwareUpdaterError> {
        self.state.verify_booted()?;
        self.state.mark_dfu()
    }

    /// Mark firmware boot successful and stop rollback on reset.
    pub fn mark_booted(&mut self) -> Result<(), FirmwareUpdaterError> {
        self.state.mark_booted()
    }

    /// Writes firmware data to the device.
    ///
    /// This function writes the given data to the firmware area starting at the specified offset.
    /// It handles sector erasures and data writes while verifying the device is in a proper state
    /// for firmware updates. The function ensures that only unerased sectors are erased before
    /// writing and efficiently handles the writing process across sector boundaries and in
    /// various configurations (data size, sector size, etc.).
    ///
    /// # Arguments
    ///
    /// * `offset` - The starting offset within the firmware area where data writing should begin.
    /// * `data` - A slice of bytes representing the firmware data to be written. It must be a
    /// multiple of NorFlash WRITE_SIZE.
    ///
    /// # Returns
    ///
    /// A `Result<(), FirmwareUpdaterError>` indicating the success or failure of the write operation.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    ///
    /// - The device is not in a proper state to receive firmware updates (e.g., not booted).
    /// - There is a failure erasing a sector before writing.
    /// - There is a failure writing data to the device.
    pub fn write_firmware(&mut self, offset: usize, data: &[u8]) -> Result<(), FirmwareUpdaterError> {
        // Make sure we are running a booted firmware to avoid reverting to a bad state.
        self.state.verify_booted()?;

        // Initialize variables to keep track of the remaining data and the current offset.
        let mut remaining_data = data;
        let mut offset = offset;

        // Continue writing as long as there is data left to write.
        while !remaining_data.is_empty() {
            // Compute the current sector and its boundaries.
            let current_sector = offset / DFU::ERASE_SIZE;
            let sector_start = current_sector * DFU::ERASE_SIZE;
            let sector_end = sector_start + DFU::ERASE_SIZE;
            // Determine if the current sector needs to be erased before writing.
            let need_erase = self
                .last_erased_dfu_sector_index
                .map_or(true, |last_erased_sector| current_sector != last_erased_sector);

            // If the sector needs to be erased, erase it and update the last erased sector index.
            if need_erase {
                self.dfu.erase(sector_start as u32, sector_end as u32)?;
                self.last_erased_dfu_sector_index = Some(current_sector);
            }

            // Calculate the size of the data chunk that can be written in the current iteration.
            let write_size = core::cmp::min(remaining_data.len(), sector_end - offset);
            // Split the data to get the current chunk to be written and the remaining data.
            let (data_chunk, rest) = remaining_data.split_at(write_size);

            // Write the current data chunk.
            self.dfu.write(offset as u32, data_chunk)?;

            // Update the offset and remaining data for the next iteration.
            remaining_data = rest;
            offset += write_size;
        }

        Ok(())
    }

    /// Prepare for an incoming DFU update by erasing the entire DFU area and
    /// returning its `Partition`.
    ///
    /// Using this instead of `write_firmware` allows for an optimized API in
    /// exchange for added complexity.
    pub fn prepare_update(&mut self) -> Result<&mut DFU, FirmwareUpdaterError> {
        self.state.verify_booted()?;
        self.dfu.erase(0, self.dfu.capacity() as u32)?;

        Ok(&mut self.dfu)
    }
}

/// Manages the state partition of the firmware update.
///
/// Can be used standalone for more fine grained control, or as part of the updater.
pub struct BlockingFirmwareState<'d, STATE> {
    state: STATE,
    aligned: &'d mut [u8],
}

impl<'d, STATE: NorFlash> BlockingFirmwareState<'d, STATE> {
    /// Creates a firmware state instance from a FirmwareUpdaterConfig, with a buffer for magic content and state partition.
    ///
    /// # Safety
    ///
    /// The `aligned` buffer must have a size of STATE::WRITE_SIZE, and follow the alignment rules for the flash being read from
    /// and written to.
    pub fn from_config<DFU: NorFlash>(config: FirmwareUpdaterConfig<DFU, STATE>, aligned: &'d mut [u8]) -> Self {
        Self::new(config.state, aligned)
    }

    /// Create a firmware state instance with a buffer for magic content and state partition.
    ///
    /// # Safety
    ///
    /// The `aligned` buffer must have a size of STATE::WRITE_SIZE, and follow the alignment rules for the flash being read from
    /// and written to.
    pub fn new(state: STATE, aligned: &'d mut [u8]) -> Self {
        assert_eq!(aligned.len(), STATE::WRITE_SIZE);
        Self { state, aligned }
    }

    // Make sure we are running a booted firmware to avoid reverting to a bad state.
    fn verify_booted(&mut self) -> Result<(), FirmwareUpdaterError> {
        let state = self.get_state()?;
        if state == State::Boot || state == State::DfuDetach || state == State::Revert {
            Ok(())
        } else {
            Err(FirmwareUpdaterError::BadState)
        }
    }

    /// Obtain the current state.
    ///
    /// This is useful to check if the bootloader has just done a swap, in order
    /// to do verifications and self-tests of the new image before calling
    /// `mark_booted`.
    pub fn get_state(&mut self) -> Result<State, FirmwareUpdaterError> {
        self.state.read(0, &mut self.aligned)?;
        Ok(State::from(&self.aligned))
    }

    /// Mark to trigger firmware swap on next boot.
    pub fn mark_updated(&mut self) -> Result<(), FirmwareUpdaterError> {
        self.set_magic(SWAP_MAGIC)
    }

    /// Mark to trigger USB DFU on next boot.
    pub fn mark_dfu(&mut self) -> Result<(), FirmwareUpdaterError> {
        self.set_magic(DFU_DETACH_MAGIC)
    }

    /// Mark firmware boot successful and stop rollback on reset.
    pub fn mark_booted(&mut self) -> Result<(), FirmwareUpdaterError> {
        self.set_magic(BOOT_MAGIC)
    }

    fn set_magic(&mut self, magic: u8) -> Result<(), FirmwareUpdaterError> {
        self.state.read(0, &mut self.aligned)?;

        if self.aligned.iter().any(|&b| b != magic) {
            // Read progress validity
            self.state.read(STATE::WRITE_SIZE as u32, &mut self.aligned)?;

            if self.aligned.iter().any(|&b| b != STATE_ERASE_VALUE) {
                // The current progress validity marker is invalid
            } else {
                // Invalidate progress
                self.aligned.fill(!STATE_ERASE_VALUE);
                self.state.write(STATE::WRITE_SIZE as u32, &self.aligned)?;
            }

            // Clear magic and progress
            self.state.erase(0, self.state.capacity() as u32)?;

            // Set magic
            self.aligned.fill(magic);
            self.state.write(0, &self.aligned)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use core::cell::RefCell;

    use embassy_embedded_hal::flash::partition::BlockingPartition;
    use embassy_sync::blocking_mutex::Mutex;
    use embassy_sync::blocking_mutex::raw::NoopRawMutex;
    use sha1::{Digest, Sha1};

    use super::*;
    use crate::mem_flash::MemFlash;

    #[test]
    fn can_verify_sha1() {
        let flash = Mutex::<NoopRawMutex, _>::new(RefCell::new(MemFlash::<131072, 4096, 8>::default()));
        let state = BlockingPartition::new(&flash, 0, 4096);
        let dfu = BlockingPartition::new(&flash, 65536, 65536);
        let mut aligned = [0; 8];

        let update = [0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66];
        let mut to_write = [0; 4096];
        to_write[..7].copy_from_slice(update.as_slice());

        let mut updater = BlockingFirmwareUpdater::new(FirmwareUpdaterConfig { dfu, state }, &mut aligned);
        updater.write_firmware(0, to_write.as_slice()).unwrap();
        let mut chunk_buf = [0; 2];
        let mut hash = [0; 20];
        updater
            .hash::<Sha1>(update.len() as u32, &mut chunk_buf, &mut hash)
            .unwrap();

        assert_eq!(Sha1::digest(update).as_slice(), hash);
    }

    #[test]
    fn can_verify_sha1_sector_bigger_than_chunk() {
        let flash = Mutex::<NoopRawMutex, _>::new(RefCell::new(MemFlash::<131072, 4096, 8>::default()));
        let state = BlockingPartition::new(&flash, 0, 4096);
        let dfu = BlockingPartition::new(&flash, 65536, 65536);
        let mut aligned = [0; 8];

        let update = [0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66];
        let mut to_write = [0; 4096];
        to_write[..7].copy_from_slice(update.as_slice());

        let mut updater = BlockingFirmwareUpdater::new(FirmwareUpdaterConfig { dfu, state }, &mut aligned);
        let mut offset = 0;
        for chunk in to_write.chunks(1024) {
            updater.write_firmware(offset, chunk).unwrap();
            offset += chunk.len();
        }
        let mut chunk_buf = [0; 2];
        let mut hash = [0; 20];
        updater
            .hash::<Sha1>(update.len() as u32, &mut chunk_buf, &mut hash)
            .unwrap();

        assert_eq!(Sha1::digest(update).as_slice(), hash);
    }

    #[test]
    fn can_verify_sha1_sector_smaller_than_chunk() {
        let flash = Mutex::<NoopRawMutex, _>::new(RefCell::new(MemFlash::<131072, 1024, 8>::default()));
        let state = BlockingPartition::new(&flash, 0, 4096);
        let dfu = BlockingPartition::new(&flash, 65536, 65536);
        let mut aligned = [0; 8];

        let update = [0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66];
        let mut to_write = [0; 4096];
        to_write[..7].copy_from_slice(update.as_slice());

        let mut updater = BlockingFirmwareUpdater::new(FirmwareUpdaterConfig { dfu, state }, &mut aligned);
        let mut offset = 0;
        for chunk in to_write.chunks(2048) {
            updater.write_firmware(offset, chunk).unwrap();
            offset += chunk.len();
        }
        let mut chunk_buf = [0; 2];
        let mut hash = [0; 20];
        updater
            .hash::<Sha1>(update.len() as u32, &mut chunk_buf, &mut hash)
            .unwrap();

        assert_eq!(Sha1::digest(update).as_slice(), hash);
    }

    #[test]
    fn can_verify_sha1_cross_sector_boundary() {
        let flash = Mutex::<NoopRawMutex, _>::new(RefCell::new(MemFlash::<131072, 1024, 8>::default()));
        let state = BlockingPartition::new(&flash, 0, 4096);
        let dfu = BlockingPartition::new(&flash, 65536, 65536);
        let mut aligned = [0; 8];

        let update = [0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66];
        let mut to_write = [0; 4096];
        to_write[..7].copy_from_slice(update.as_slice());

        let mut updater = BlockingFirmwareUpdater::new(FirmwareUpdaterConfig { dfu, state }, &mut aligned);
        let mut offset = 0;
        for chunk in to_write.chunks(896) {
            updater.write_firmware(offset, chunk).unwrap();
            offset += chunk.len();
        }
        let mut chunk_buf = [0; 2];
        let mut hash = [0; 20];
        updater
            .hash::<Sha1>(update.len() as u32, &mut chunk_buf, &mut hash)
            .unwrap();

        assert_eq!(Sha1::digest(update).as_slice(), hash);
    }
}
