use digest::Digest;
#[cfg(target_os = "none")]
use embassy_embedded_hal::flash::partition::Partition;
#[cfg(target_os = "none")]
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embedded_storage_async::nor_flash::NorFlash;

use super::FirmwareUpdaterConfig;
use crate::{FirmwareUpdaterError, State, BOOT_MAGIC, DFU_DETACH_MAGIC, STATE_ERASE_VALUE, SWAP_MAGIC};

/// FirmwareUpdater is an application API for interacting with the BootLoader without the ability to
/// 'mess up' the internal bootloader state
pub struct FirmwareUpdater<'d, DFU: NorFlash, STATE: NorFlash> {
    dfu: DFU,
    state: FirmwareState<'d, STATE>,
}

#[cfg(target_os = "none")]
impl<'a, DFU: NorFlash, STATE: NorFlash>
    FirmwareUpdaterConfig<Partition<'a, NoopRawMutex, DFU>, Partition<'a, NoopRawMutex, STATE>>
{
    /// Create a firmware updater config from the flash and address symbols defined in the linkerfile
    pub fn from_linkerfile(
        dfu_flash: &'a embassy_sync::mutex::Mutex<NoopRawMutex, DFU>,
        state_flash: &'a embassy_sync::mutex::Mutex<NoopRawMutex, STATE>,
    ) -> Self {
        extern "C" {
            static __bootloader_state_start: u32;
            static __bootloader_state_end: u32;
            static __bootloader_dfu_start: u32;
            static __bootloader_dfu_end: u32;
        }

        let dfu = unsafe {
            let start = &__bootloader_dfu_start as *const u32 as u32;
            let end = &__bootloader_dfu_end as *const u32 as u32;
            trace!("DFU: 0x{:x} - 0x{:x}", start, end);

            Partition::new(dfu_flash, start, end - start)
        };
        let state = unsafe {
            let start = &__bootloader_state_start as *const u32 as u32;
            let end = &__bootloader_state_end as *const u32 as u32;
            trace!("STATE: 0x{:x} - 0x{:x}", start, end);

            Partition::new(state_flash, start, end - start)
        };

        Self { dfu, state }
    }
}

impl<'d, DFU: NorFlash, STATE: NorFlash> FirmwareUpdater<'d, DFU, STATE> {
    /// Create a firmware updater instance with partition ranges for the update and state partitions.
    pub fn new(config: FirmwareUpdaterConfig<DFU, STATE>, aligned: &'d mut [u8]) -> Self {
        Self {
            dfu: config.dfu,
            state: FirmwareState::new(config.state, aligned),
        }
    }

    /// Obtain the current state.
    ///
    /// This is useful to check if the bootloader has just done a swap, in order
    /// to do verifications and self-tests of the new image before calling
    /// `mark_booted`.
    pub async fn get_state(&mut self) -> Result<State, FirmwareUpdaterError> {
        self.state.get_state().await
    }

    /// Verify the DFU given a public key. If there is an error then DO NOT
    /// proceed with updating the firmware as it must be signed with a
    /// corresponding private key (otherwise it could be malicious firmware).
    ///
    /// Mark to trigger firmware swap on next boot if verify suceeds.
    ///
    /// If the "ed25519-salty" feature is set (or another similar feature) then the signature is expected to have
    /// been generated from a SHA-512 digest of the firmware bytes.
    ///
    /// If no signature feature is set then this method will always return a
    /// signature error.
    #[cfg(feature = "_verify")]
    pub async fn verify_and_mark_updated(
        &mut self,
        _public_key: &[u8; 32],
        _signature: &[u8; 64],
        _update_len: u32,
    ) -> Result<(), FirmwareUpdaterError> {
        assert!(_update_len <= self.dfu.capacity() as u32);

        self.state.verify_booted().await?;

        #[cfg(feature = "ed25519-dalek")]
        {
            use ed25519_dalek::{Signature, SignatureError, Verifier, VerifyingKey};

            use crate::digest_adapters::ed25519_dalek::Sha512;

            let into_signature_error = |e: SignatureError| FirmwareUpdaterError::Signature(e.into());

            let public_key = VerifyingKey::from_bytes(_public_key).map_err(into_signature_error)?;
            let signature = Signature::from_bytes(_signature);

            let mut chunk_buf = [0; 2];
            let mut message = [0; 64];
            self.hash::<Sha512>(_update_len, &mut chunk_buf, &mut message).await?;

            public_key.verify(&message, &signature).map_err(into_signature_error)?
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
            self.hash::<Sha512>(_update_len, &mut chunk_buf, &mut message).await?;

            let r = public_key.verify(&message, &signature);
            trace!(
                "Verifying with public key {}, signature {} and message {} yields ok: {}",
                public_key.to_bytes(),
                signature.to_bytes(),
                message,
                r.is_ok()
            );
            r.map_err(into_signature_error)?
        }

        self.state.mark_updated().await
    }

    /// Verify the update in DFU with any digest.
    pub async fn hash<D: Digest>(
        &mut self,
        update_len: u32,
        chunk_buf: &mut [u8],
        output: &mut [u8],
    ) -> Result<(), FirmwareUpdaterError> {
        let mut digest = D::new();
        for offset in (0..update_len).step_by(chunk_buf.len()) {
            self.dfu.read(offset, chunk_buf).await?;
            let len = core::cmp::min((update_len - offset) as usize, chunk_buf.len());
            digest.update(&chunk_buf[..len]);
        }
        output.copy_from_slice(digest.finalize().as_slice());
        Ok(())
    }

    /// Mark to trigger firmware swap on next boot.
    #[cfg(not(feature = "_verify"))]
    pub async fn mark_updated(&mut self) -> Result<(), FirmwareUpdaterError> {
        self.state.mark_updated().await
    }

    /// Mark to trigger USB DFU on next boot.
    pub async fn mark_dfu(&mut self) -> Result<(), FirmwareUpdaterError> {
        self.state.verify_booted().await?;
        self.state.mark_dfu().await
    }

    /// Mark firmware boot successful and stop rollback on reset.
    pub async fn mark_booted(&mut self) -> Result<(), FirmwareUpdaterError> {
        self.state.mark_booted().await
    }

    /// Write data to a flash page.
    ///
    /// The buffer must follow alignment requirements of the target flash and a multiple of page size big.
    ///
    /// # Safety
    ///
    /// Failing to meet alignment and size requirements may result in a panic.
    pub async fn write_firmware(&mut self, offset: usize, data: &[u8]) -> Result<(), FirmwareUpdaterError> {
        assert!(data.len() >= DFU::ERASE_SIZE);

        self.state.verify_booted().await?;

        self.dfu.erase(offset as u32, (offset + data.len()) as u32).await?;

        self.dfu.write(offset as u32, data).await?;

        Ok(())
    }

    /// Prepare for an incoming DFU update by erasing the entire DFU area and
    /// returning its `Partition`.
    ///
    /// Using this instead of `write_firmware` allows for an optimized API in
    /// exchange for added complexity.
    pub async fn prepare_update(&mut self) -> Result<&mut DFU, FirmwareUpdaterError> {
        self.state.verify_booted().await?;
        self.dfu.erase(0, self.dfu.capacity() as u32).await?;

        Ok(&mut self.dfu)
    }
}

/// Manages the state partition of the firmware update.
///
/// Can be used standalone for more fine grained control, or as part of the updater.
pub struct FirmwareState<'d, STATE> {
    state: STATE,
    aligned: &'d mut [u8],
}

impl<'d, STATE: NorFlash> FirmwareState<'d, STATE> {
    /// Create a firmware state instance from a FirmwareUpdaterConfig with a buffer for magic content and state partition.
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
    /// The `aligned` buffer must have a size of maximum of STATE::WRITE_SIZE and STATE::READ_SIZE,
    /// and follow the alignment rules for the flash being read from and written to.
    pub fn new(state: STATE, aligned: &'d mut [u8]) -> Self {
        assert_eq!(aligned.len(), STATE::WRITE_SIZE.max(STATE::READ_SIZE));
        Self { state, aligned }
    }

    // Make sure we are running a booted firmware to avoid reverting to a bad state.
    async fn verify_booted(&mut self) -> Result<(), FirmwareUpdaterError> {
        if self.get_state().await? == State::Boot {
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
    pub async fn get_state(&mut self) -> Result<State, FirmwareUpdaterError> {
        self.state.read(0, &mut self.aligned).await?;

        if !self.aligned.iter().any(|&b| b != SWAP_MAGIC) {
            Ok(State::Swap)
        } else {
            Ok(State::Boot)
        }
    }

    /// Mark to trigger firmware swap on next boot.
    pub async fn mark_updated(&mut self) -> Result<(), FirmwareUpdaterError> {
        self.set_magic(SWAP_MAGIC).await
    }

    /// Mark to trigger USB DFU on next boot.
    pub async fn mark_dfu(&mut self) -> Result<(), FirmwareUpdaterError> {
        self.set_magic(DFU_DETACH_MAGIC).await
    }

    /// Mark firmware boot successful and stop rollback on reset.
    pub async fn mark_booted(&mut self) -> Result<(), FirmwareUpdaterError> {
        self.set_magic(BOOT_MAGIC).await
    }

    async fn set_magic(&mut self, magic: u8) -> Result<(), FirmwareUpdaterError> {
        self.state.read(0, &mut self.aligned).await?;

        if self.aligned.iter().any(|&b| b != magic) {
            // Read progress validity
            self.state.read(STATE::WRITE_SIZE as u32, &mut self.aligned).await?;

            if self.aligned.iter().any(|&b| b != STATE_ERASE_VALUE) {
                // The current progress validity marker is invalid
            } else {
                // Invalidate progress
                self.aligned.fill(!STATE_ERASE_VALUE);
                self.state.write(STATE::WRITE_SIZE as u32, &self.aligned).await?;
            }

            // Clear magic and progress
            self.state.erase(0, self.state.capacity() as u32).await?;

            // Set magic
            self.aligned.fill(magic);
            self.state.write(0, &self.aligned).await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use embassy_embedded_hal::flash::partition::Partition;
    use embassy_sync::blocking_mutex::raw::NoopRawMutex;
    use embassy_sync::mutex::Mutex;
    use futures::executor::block_on;
    use sha1::{Digest, Sha1};

    use super::*;
    use crate::mem_flash::MemFlash;

    #[test]
    fn can_verify_sha1() {
        let flash = Mutex::<NoopRawMutex, _>::new(MemFlash::<131072, 4096, 8>::default());
        let state = Partition::new(&flash, 0, 4096);
        let dfu = Partition::new(&flash, 65536, 65536);
        let mut aligned = [0; 8];

        let update = [0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66];
        let mut to_write = [0; 4096];
        to_write[..7].copy_from_slice(update.as_slice());

        let mut updater = FirmwareUpdater::new(FirmwareUpdaterConfig { dfu, state }, &mut aligned);
        block_on(updater.write_firmware(0, to_write.as_slice())).unwrap();
        let mut chunk_buf = [0; 2];
        let mut hash = [0; 20];
        block_on(updater.hash::<Sha1>(update.len() as u32, &mut chunk_buf, &mut hash)).unwrap();

        assert_eq!(Sha1::digest(update).as_slice(), hash);
    }
}
