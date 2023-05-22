use digest::Digest;
use embedded_storage::nor_flash::{NorFlash, NorFlashError, NorFlashErrorKind};
#[cfg(feature = "nightly")]
use embedded_storage_async::nor_flash::NorFlash as AsyncNorFlash;

use crate::{Partition, State, BOOT_MAGIC, SWAP_MAGIC};

/// Errors returned by FirmwareUpdater
#[derive(Debug)]
pub enum FirmwareUpdaterError {
    /// Error from flash.
    Flash(NorFlashErrorKind),
    /// Signature errors.
    Signature(signature::Error),
}

#[cfg(feature = "defmt")]
impl defmt::Format for FirmwareUpdaterError {
    fn format(&self, fmt: defmt::Formatter) {
        match self {
            FirmwareUpdaterError::Flash(_) => defmt::write!(fmt, "FirmwareUpdaterError::Flash(_)"),
            FirmwareUpdaterError::Signature(_) => defmt::write!(fmt, "FirmwareUpdaterError::Signature(_)"),
        }
    }
}

impl<E> From<E> for FirmwareUpdaterError
where
    E: NorFlashError,
{
    fn from(error: E) -> Self {
        FirmwareUpdaterError::Flash(error.kind())
    }
}

/// FirmwareUpdater is an application API for interacting with the BootLoader without the ability to
/// 'mess up' the internal bootloader state
pub struct FirmwareUpdater {
    state: Partition,
    dfu: Partition,
}

#[cfg(target_os = "none")]
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
                &__bootloader_dfu_start as *const u32 as u32,
                &__bootloader_dfu_end as *const u32 as u32,
            )
        };
        let state = unsafe {
            Partition::new(
                &__bootloader_state_start as *const u32 as u32,
                &__bootloader_state_end as *const u32 as u32,
            )
        };

        trace!("DFU: 0x{:x} - 0x{:x}", dfu.from, dfu.to);
        trace!("STATE: 0x{:x} - 0x{:x}", state.from, state.to);
        FirmwareUpdater::new(dfu, state)
    }
}

impl FirmwareUpdater {
    /// Create a firmware updater instance with partition ranges for the update and state partitions.
    pub const fn new(dfu: Partition, state: Partition) -> Self {
        Self { dfu, state }
    }

    /// Obtain the current state.
    ///
    /// This is useful to check if the bootloader has just done a swap, in order
    /// to do verifications and self-tests of the new image before calling
    /// `mark_booted`.
    #[cfg(feature = "nightly")]
    pub async fn get_state<F: AsyncNorFlash>(
        &mut self,
        state_flash: &mut F,
        aligned: &mut [u8],
    ) -> Result<State, FirmwareUpdaterError> {
        self.state.read(state_flash, 0, aligned).await?;

        if !aligned.iter().any(|&b| b != SWAP_MAGIC) {
            Ok(State::Swap)
        } else {
            Ok(State::Boot)
        }
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
    ///
    /// # Safety
    ///
    /// The `_aligned` buffer must have a size of F::WRITE_SIZE, and follow the alignment rules for the flash being read from
    /// and written to.
    #[cfg(all(feature = "_verify", feature = "nightly"))]
    pub async fn verify_and_mark_updated<F: AsyncNorFlash>(
        &mut self,
        _state_and_dfu_flash: &mut F,
        _public_key: &[u8],
        _signature: &[u8],
        _update_len: u32,
        _aligned: &mut [u8],
    ) -> Result<(), FirmwareUpdaterError> {
        assert_eq!(_aligned.len(), F::WRITE_SIZE);
        assert!(_update_len <= self.dfu.size());

        #[cfg(feature = "ed25519-dalek")]
        {
            use ed25519_dalek::{PublicKey, Signature, SignatureError, Verifier};

            use crate::digest_adapters::ed25519_dalek::Sha512;

            let into_signature_error = |e: SignatureError| FirmwareUpdaterError::Signature(e.into());

            let public_key = PublicKey::from_bytes(_public_key).map_err(into_signature_error)?;
            let signature = Signature::from_bytes(_signature).map_err(into_signature_error)?;

            let mut message = [0; 64];
            self.hash::<_, Sha512>(_state_and_dfu_flash, _update_len, _aligned, &mut message)
                .await?;

            public_key.verify(&message, &signature).map_err(into_signature_error)?
        }
        #[cfg(feature = "ed25519-salty")]
        {
            use salty::constants::{PUBLICKEY_SERIALIZED_LENGTH, SIGNATURE_SERIALIZED_LENGTH};
            use salty::{PublicKey, Signature};

            use crate::digest_adapters::salty::Sha512;

            fn into_signature_error<E>(_: E) -> FirmwareUpdaterError {
                FirmwareUpdaterError::Signature(signature::Error::default())
            }

            let public_key: [u8; PUBLICKEY_SERIALIZED_LENGTH] = _public_key.try_into().map_err(into_signature_error)?;
            let public_key = PublicKey::try_from(&public_key).map_err(into_signature_error)?;
            let signature: [u8; SIGNATURE_SERIALIZED_LENGTH] = _signature.try_into().map_err(into_signature_error)?;
            let signature = Signature::try_from(&signature).map_err(into_signature_error)?;

            let mut message = [0; 64];
            self.hash::<_, Sha512>(_state_and_dfu_flash, _update_len, _aligned, &mut message)
                .await?;

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

        self.set_magic(_aligned, SWAP_MAGIC, _state_and_dfu_flash).await
    }

    /// Verify the update in DFU with any digest.
    #[cfg(feature = "nightly")]
    pub async fn hash<F: AsyncNorFlash, D: Digest>(
        &mut self,
        dfu_flash: &mut F,
        update_len: u32,
        chunk_buf: &mut [u8],
        output: &mut [u8],
    ) -> Result<(), FirmwareUpdaterError> {
        let mut digest = D::new();
        for offset in (0..update_len).step_by(chunk_buf.len()) {
            self.dfu.read(dfu_flash, offset, chunk_buf).await?;
            let len = core::cmp::min((update_len - offset) as usize, chunk_buf.len());
            digest.update(&chunk_buf[..len]);
        }
        output.copy_from_slice(digest.finalize().as_slice());
        Ok(())
    }

    /// Mark to trigger firmware swap on next boot.
    ///
    /// # Safety
    ///
    /// The `aligned` buffer must have a size of F::WRITE_SIZE, and follow the alignment rules for the flash being written to.
    #[cfg(all(feature = "nightly", not(feature = "_verify")))]
    pub async fn mark_updated<F: AsyncNorFlash>(
        &mut self,
        state_flash: &mut F,
        aligned: &mut [u8],
    ) -> Result<(), FirmwareUpdaterError> {
        assert_eq!(aligned.len(), F::WRITE_SIZE);
        self.set_magic(aligned, SWAP_MAGIC, state_flash).await
    }

    /// Mark firmware boot successful and stop rollback on reset.
    ///
    /// # Safety
    ///
    /// The `aligned` buffer must have a size of F::WRITE_SIZE, and follow the alignment rules for the flash being written to.
    #[cfg(feature = "nightly")]
    pub async fn mark_booted<F: AsyncNorFlash>(
        &mut self,
        state_flash: &mut F,
        aligned: &mut [u8],
    ) -> Result<(), FirmwareUpdaterError> {
        assert_eq!(aligned.len(), F::WRITE_SIZE);
        self.set_magic(aligned, BOOT_MAGIC, state_flash).await
    }

    #[cfg(feature = "nightly")]
    async fn set_magic<F: AsyncNorFlash>(
        &mut self,
        aligned: &mut [u8],
        magic: u8,
        state_flash: &mut F,
    ) -> Result<(), FirmwareUpdaterError> {
        self.state.read(state_flash, 0, aligned).await?;

        if aligned.iter().any(|&b| b != magic) {
            // Read progress validity
            self.state.read(state_flash, F::WRITE_SIZE as u32, aligned).await?;

            // FIXME: Do not make this assumption.
            const STATE_ERASE_VALUE: u8 = 0xFF;

            if aligned.iter().any(|&b| b != STATE_ERASE_VALUE) {
                // The current progress validity marker is invalid
            } else {
                // Invalidate progress
                aligned.fill(!STATE_ERASE_VALUE);
                self.state.write(state_flash, F::WRITE_SIZE as u32, aligned).await?;
            }

            // Clear magic and progress
            self.state.wipe(state_flash).await?;

            // Set magic
            aligned.fill(magic);
            self.state.write(state_flash, 0, aligned).await?;
        }
        Ok(())
    }

    /// Write data to a flash page.
    ///
    /// The buffer must follow alignment requirements of the target flash and a multiple of page size big.
    ///
    /// # Safety
    ///
    /// Failing to meet alignment and size requirements may result in a panic.
    #[cfg(feature = "nightly")]
    pub async fn write_firmware<F: AsyncNorFlash>(
        &mut self,
        offset: usize,
        data: &[u8],
        dfu_flash: &mut F,
    ) -> Result<(), FirmwareUpdaterError> {
        assert!(data.len() >= F::ERASE_SIZE);

        self.dfu
            .erase(dfu_flash, offset as u32, (offset + data.len()) as u32)
            .await?;

        self.dfu.write(dfu_flash, offset as u32, data).await?;

        Ok(())
    }

    /// Prepare for an incoming DFU update by erasing the entire DFU area and
    /// returning its `Partition`.
    ///
    /// Using this instead of `write_firmware` allows for an optimized API in
    /// exchange for added complexity.
    #[cfg(feature = "nightly")]
    pub async fn prepare_update<F: AsyncNorFlash>(
        &mut self,
        dfu_flash: &mut F,
    ) -> Result<Partition, FirmwareUpdaterError> {
        self.dfu.wipe(dfu_flash).await?;

        Ok(self.dfu)
    }

    //
    // Blocking API
    //

    /// Obtain the current state.
    ///
    /// This is useful to check if the bootloader has just done a swap, in order
    /// to do verifications and self-tests of the new image before calling
    /// `mark_booted`.
    pub fn get_state_blocking<F: NorFlash>(
        &mut self,
        state_flash: &mut F,
        aligned: &mut [u8],
    ) -> Result<State, FirmwareUpdaterError> {
        self.state.read_blocking(state_flash, 0, aligned)?;

        if !aligned.iter().any(|&b| b != SWAP_MAGIC) {
            Ok(State::Swap)
        } else {
            Ok(State::Boot)
        }
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
    ///
    /// # Safety
    ///
    /// The `_aligned` buffer must have a size of F::WRITE_SIZE, and follow the alignment rules for the flash being read from
    /// and written to.
    #[cfg(feature = "_verify")]
    pub fn verify_and_mark_updated_blocking<F: NorFlash>(
        &mut self,
        _state_and_dfu_flash: &mut F,
        _public_key: &[u8],
        _signature: &[u8],
        _update_len: u32,
        _aligned: &mut [u8],
    ) -> Result<(), FirmwareUpdaterError> {
        assert_eq!(_aligned.len(), F::WRITE_SIZE);
        assert!(_update_len <= self.dfu.size());

        #[cfg(feature = "ed25519-dalek")]
        {
            use ed25519_dalek::{PublicKey, Signature, SignatureError, Verifier};

            use crate::digest_adapters::ed25519_dalek::Sha512;

            let into_signature_error = |e: SignatureError| FirmwareUpdaterError::Signature(e.into());

            let public_key = PublicKey::from_bytes(_public_key).map_err(into_signature_error)?;
            let signature = Signature::from_bytes(_signature).map_err(into_signature_error)?;

            let mut message = [0; 64];
            self.hash_blocking::<_, Sha512>(_state_and_dfu_flash, _update_len, _aligned, &mut message)?;

            public_key.verify(&message, &signature).map_err(into_signature_error)?
        }
        #[cfg(feature = "ed25519-salty")]
        {
            use salty::constants::{PUBLICKEY_SERIALIZED_LENGTH, SIGNATURE_SERIALIZED_LENGTH};
            use salty::{PublicKey, Signature};

            use crate::digest_adapters::salty::Sha512;

            fn into_signature_error<E>(_: E) -> FirmwareUpdaterError {
                FirmwareUpdaterError::Signature(signature::Error::default())
            }

            let public_key: [u8; PUBLICKEY_SERIALIZED_LENGTH] = _public_key.try_into().map_err(into_signature_error)?;
            let public_key = PublicKey::try_from(&public_key).map_err(into_signature_error)?;
            let signature: [u8; SIGNATURE_SERIALIZED_LENGTH] = _signature.try_into().map_err(into_signature_error)?;
            let signature = Signature::try_from(&signature).map_err(into_signature_error)?;

            let mut message = [0; 64];
            self.hash_blocking::<_, Sha512>(_state_and_dfu_flash, _update_len, _aligned, &mut message)?;

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

        self.set_magic_blocking(_aligned, SWAP_MAGIC, _state_and_dfu_flash)
    }

    /// Verify the update in DFU with any digest.
    pub fn hash_blocking<F: NorFlash, D: Digest>(
        &mut self,
        dfu_flash: &mut F,
        update_len: u32,
        chunk_buf: &mut [u8],
        output: &mut [u8],
    ) -> Result<(), FirmwareUpdaterError> {
        let mut digest = D::new();
        for offset in (0..update_len).step_by(chunk_buf.len()) {
            self.dfu.read_blocking(dfu_flash, offset, chunk_buf)?;
            let len = core::cmp::min((update_len - offset) as usize, chunk_buf.len());
            digest.update(&chunk_buf[..len]);
        }
        output.copy_from_slice(digest.finalize().as_slice());
        Ok(())
    }

    /// Mark to trigger firmware swap on next boot.
    ///
    /// # Safety
    ///
    /// The `aligned` buffer must have a size of F::WRITE_SIZE, and follow the alignment rules for the flash being written to.
    #[cfg(not(feature = "_verify"))]
    pub fn mark_updated_blocking<F: NorFlash>(
        &mut self,
        state_flash: &mut F,
        aligned: &mut [u8],
    ) -> Result<(), FirmwareUpdaterError> {
        assert_eq!(aligned.len(), F::WRITE_SIZE);
        self.set_magic_blocking(aligned, SWAP_MAGIC, state_flash)
    }

    /// Mark firmware boot successful and stop rollback on reset.
    ///
    /// # Safety
    ///
    /// The `aligned` buffer must have a size of F::WRITE_SIZE, and follow the alignment rules for the flash being written to.
    pub fn mark_booted_blocking<F: NorFlash>(
        &mut self,
        state_flash: &mut F,
        aligned: &mut [u8],
    ) -> Result<(), FirmwareUpdaterError> {
        assert_eq!(aligned.len(), F::WRITE_SIZE);
        self.set_magic_blocking(aligned, BOOT_MAGIC, state_flash)
    }

    fn set_magic_blocking<F: NorFlash>(
        &mut self,
        aligned: &mut [u8],
        magic: u8,
        state_flash: &mut F,
    ) -> Result<(), FirmwareUpdaterError> {
        self.state.read_blocking(state_flash, 0, aligned)?;

        if aligned.iter().any(|&b| b != magic) {
            // Read progress validity
            self.state.read_blocking(state_flash, F::WRITE_SIZE as u32, aligned)?;

            // FIXME: Do not make this assumption.
            const STATE_ERASE_VALUE: u8 = 0xFF;

            if aligned.iter().any(|&b| b != STATE_ERASE_VALUE) {
                // The current progress validity marker is invalid
            } else {
                // Invalidate progress
                aligned.fill(!STATE_ERASE_VALUE);
                self.state.write_blocking(state_flash, F::WRITE_SIZE as u32, aligned)?;
            }

            // Clear magic and progress
            self.state.wipe_blocking(state_flash)?;

            // Set magic
            aligned.fill(magic);
            self.state.write_blocking(state_flash, 0, aligned)?;
        }
        Ok(())
    }

    /// Write data to a flash page.
    ///
    /// The buffer must follow alignment requirements of the target flash and a multiple of page size big.
    ///
    /// # Safety
    ///
    /// Failing to meet alignment and size requirements may result in a panic.
    pub fn write_firmware_blocking<F: NorFlash>(
        &mut self,
        offset: usize,
        data: &[u8],
        dfu_flash: &mut F,
    ) -> Result<(), FirmwareUpdaterError> {
        assert!(data.len() >= F::ERASE_SIZE);

        self.dfu
            .erase_blocking(dfu_flash, offset as u32, (offset + data.len()) as u32)?;

        self.dfu.write_blocking(dfu_flash, offset as u32, data)?;

        Ok(())
    }

    /// Prepare for an incoming DFU update by erasing the entire DFU area and
    /// returning its `Partition`.
    ///
    /// Using this instead of `write_firmware_blocking` allows for an optimized
    /// API in exchange for added complexity.
    pub fn prepare_update_blocking<F: NorFlash>(&mut self, flash: &mut F) -> Result<Partition, FirmwareUpdaterError> {
        self.dfu.wipe_blocking(flash)?;

        Ok(self.dfu)
    }
}

#[cfg(test)]
mod tests {
    use futures::executor::block_on;
    use sha1::{Digest, Sha1};

    use super::*;
    use crate::mem_flash::MemFlash;

    #[test]
    #[cfg(feature = "nightly")]
    fn can_verify_sha1() {
        const STATE: Partition = Partition::new(0, 4096);
        const DFU: Partition = Partition::new(65536, 131072);

        let mut flash = MemFlash::<131072, 4096, 8>::default();

        let update = [0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66];
        let mut to_write = [0; 4096];
        to_write[..7].copy_from_slice(update.as_slice());

        let mut updater = FirmwareUpdater::new(DFU, STATE);
        block_on(updater.write_firmware(0, to_write.as_slice(), &mut flash)).unwrap();
        let mut chunk_buf = [0; 2];
        let mut hash = [0; 20];
        block_on(updater.hash::<_, Sha1>(&mut flash, update.len() as u32, &mut chunk_buf, &mut hash)).unwrap();

        assert_eq!(Sha1::digest(update).as_slice(), hash);
    }
}
