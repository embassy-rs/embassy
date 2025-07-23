#![no_std]
#![allow(async_fn_in_trait)]
#![warn(missing_docs)]
#![doc = include_str!("../README.md")]
mod fmt;

mod boot_loader;
mod digest_adapters;
mod firmware_updater;
#[cfg(test)]
mod mem_flash;
#[cfg(test)]
mod test_flash;

// The expected value of the flash after an erase
// TODO: Use the value provided by NorFlash when available
#[cfg(not(feature = "flash-erase-zero"))]
pub(crate) const STATE_ERASE_VALUE: u8 = 0xFF;
#[cfg(feature = "flash-erase-zero")]
pub(crate) const STATE_ERASE_VALUE: u8 = 0x00;

pub use boot_loader::{BootError, BootLoader, BootLoaderConfig};
pub use firmware_updater::{
    BlockingFirmwareState, BlockingFirmwareUpdater, FirmwareState, FirmwareUpdater, FirmwareUpdaterConfig,
    FirmwareUpdaterError,
};

pub(crate) const REVERT_MAGIC: u8 = 0xC0;
pub(crate) const BOOT_MAGIC: u8 = 0xD0;
pub(crate) const SWAP_MAGIC: u8 = 0xF0;
pub(crate) const DFU_DETACH_MAGIC: u8 = 0xE0;
#[cfg(feature = "restore")]
pub(crate) const BACKUP_MAGIC: u8 = 0xA1;
#[cfg(feature = "restore")]
pub(crate) const RESTORE_MAGIC: u8 = 0xB0;
#[cfg(feature = "safe")]
pub(crate) const SAFE_MAGIC: u8 = 0xA0;

/// The state of the bootloader after running prepare.
#[derive(PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum State {
    /// Bootloader is ready to boot the active partition.
    Boot,
    /// Bootloader has swapped the active partition with the dfu partition and will attempt boot.
    Swap,
    /// Bootloader has reverted the active partition with the dfu partition and will attempt boot.
    Revert,
    /// Bootloader is requested to boot the safe image.
    #[cfg(feature = "safe")]
    Safe,
    /// Bootloader will copy the active partition to the dfu partition as a backup.
    #[cfg(feature = "restore")]
    /// Bootloader will copy the active partition to the dfu partition as a backup.
    Backup,
    /// Bootloader will copy the dfu partition to the active partition to restore.
    #[cfg(feature = "restore")]
    Restore,
    /// Application has received a request to reboot into DFU mode to apply an update.
    DfuDetach,
}

impl<T> From<T> for State
where
    T: AsRef<[u8]>,
{
    fn from(magic: T) -> State {
        let magic = magic.as_ref();
        if magic.iter().all(|&b| b == SWAP_MAGIC) {
            return State::Swap;
        }
        if magic.iter().all(|&b| b == REVERT_MAGIC) {
            return State::Revert;
        }
        if magic.iter().all(|&b| b == DFU_DETACH_MAGIC) {
            return State::DfuDetach;
        }
        #[cfg(feature = "safe")]
        if magic.iter().all(|&b| b == SAFE_MAGIC) {
            return State::Safe;
        }
        #[cfg(feature = "restore")]
        if magic.iter().all(|&b| b == BACKUP_MAGIC) {
            return State::Backup;
        }
        #[cfg(feature = "restore")]
        if magic.iter().all(|&b| b == RESTORE_MAGIC) {
            return State::Restore;
        }
        return State::Boot;
    }
}

/// Buffer aligned to 32 byte boundary, largest known alignment requirement for embassy-boot.
#[repr(align(32))]
pub struct AlignedBuffer<const N: usize>(pub [u8; N]);

impl<const N: usize> AsRef<[u8]> for AlignedBuffer<N> {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl<const N: usize> AsMut<[u8]> for AlignedBuffer<N> {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

/// Dummy error struct for `DummySafe` struct
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Error {
    OutOfBounds,
}

impl embedded_storage::nor_flash::NorFlashError for Error {
    fn kind(&self) -> embedded_storage::nor_flash::NorFlashErrorKind {
        match self {
            Self::OutOfBounds => embedded_storage::nor_flash::NorFlashErrorKind::OutOfBounds,
        }
    }
}

/// Dummy struct for the `BootLoader`'s `safe_flash` parameter, used when the `safe` feature is disabled.
pub struct DummySafe;

impl embedded_storage::nor_flash::NorFlash for DummySafe {
    const WRITE_SIZE: usize = 0;
    const ERASE_SIZE: usize = 0;

    fn erase(&mut self, _from: u32, _to: u32) -> Result<(), Self::Error> {
        return Err(Error::OutOfBounds);
    }
    fn write(&mut self, _offset: u32, _bytes: &[u8]) -> Result<(), Self::Error> {
        return Err(Error::OutOfBounds);
    }
}

impl embedded_storage::nor_flash::ReadNorFlash for DummySafe {
    const READ_SIZE: usize = 0;
    fn read(&mut self, _offset: u32, _bytes: &mut [u8]) -> Result<(), Self::Error> {
        return Err(Error::OutOfBounds);
    }

    fn capacity(&self) -> usize {
        0
    }
}

impl embedded_storage::nor_flash::ErrorType for DummySafe {
    type Error = Error;
}

/// Dummy struct for the `BootLoader`'s `safe_flash` parameter, used when the `safe` feature is disabled.
pub struct DummySafeAsync;

impl embedded_storage_async::nor_flash::NorFlash for DummySafeAsync {
    const WRITE_SIZE: usize = 0;
    const ERASE_SIZE: usize = 0;

    async fn erase(&mut self, _from: u32, _to: u32) -> Result<(), Self::Error> {
        return Err(Error::OutOfBounds);
    }
    async fn write(&mut self, _offset: u32, _bytes: &[u8]) -> Result<(), Self::Error> {
        return Err(Error::OutOfBounds);
    }
}

impl embedded_storage_async::nor_flash::ReadNorFlash for DummySafeAsync {
    const READ_SIZE: usize = 0;
    async fn read(&mut self, _offset: u32, _bytes: &mut [u8]) -> Result<(), Self::Error> {
        return Err(Error::OutOfBounds);
    }

    fn capacity(&self) -> usize {
        0
    }
}

impl embedded_storage_async::nor_flash::ErrorType for DummySafeAsync {
    type Error = Error;
}

#[cfg(test)]
mod tests {
    #![allow(unused_imports)]

    use embedded_storage::nor_flash::{NorFlash, ReadNorFlash};
    use embedded_storage_async::nor_flash::NorFlash as AsyncNorFlash;
    use futures::executor::block_on;

    use super::*;
    use crate::boot_loader::BootLoaderConfig;
    use crate::firmware_updater::FirmwareUpdaterConfig;
    use crate::mem_flash::MemFlash;
    use crate::test_flash::{AsyncTestFlash, BlockingTestFlash};

    /*
    #[test]
    fn test_bad_magic() {
        let mut flash = MemFlash([0xff; 131072]);
        let mut flash = SingleFlashConfig::new(&mut flash);

        let mut bootloader = BootLoader::<4096>::new(ACTIVE, DFU, STATE);

        assert_eq!(
            bootloader.prepare_boot(&mut flash),
            Err(BootError::BadMagic)
        );
    }
    */

    #[test]
    fn test_boot_state() {
        const FIRMWARE_SIZE: usize = 12288;
        let flash = BlockingTestFlash::new(BootLoaderConfig {
            active: MemFlash::<FIRMWARE_SIZE, 4096, 4>::default(),
            dfu: MemFlash::<{ FIRMWARE_SIZE + 4096 }, 4096, 4>::default(),
            state: MemFlash::<4096, 4096, 4>::default(),
            safe: MemFlash::<FIRMWARE_SIZE, 4096, 4>::default(),
        });

        flash.state().write(0, &[BOOT_MAGIC; 4]).unwrap();

        let mut bootloader = BootLoader::new(BootLoaderConfig {
            active: flash.active(),
            dfu: flash.dfu(),
            state: flash.state(),
            safe: flash.safe(),
        });

        let mut page = [0; 4096];
        assert_eq!(State::Boot, bootloader.prepare_boot(&mut page).unwrap());
    }

    #[test]
    #[cfg(not(feature = "_verify"))]
    fn test_swap_state() {
        const FIRMWARE_SIZE: usize = 12288;
        let flash = AsyncTestFlash::new(BootLoaderConfig {
            active: MemFlash::<FIRMWARE_SIZE, 4096, 4>::default(),
            dfu: MemFlash::<{ FIRMWARE_SIZE + 4096 }, 4096, 4>::default(),
            state: MemFlash::<4096, 4096, 4>::default(),
            safe: MemFlash::<FIRMWARE_SIZE, 4096, 4>::default(),
        });

        const ORIGINAL: [u8; FIRMWARE_SIZE] = [0x55; FIRMWARE_SIZE];
        const UPDATE: [u8; FIRMWARE_SIZE] = [0xAA; FIRMWARE_SIZE];
        let mut aligned = [0; 4];

        block_on(flash.active().erase(0, ORIGINAL.len() as u32)).unwrap();
        block_on(flash.active().write(0, &ORIGINAL)).unwrap();

        let mut updater = FirmwareUpdater::new(
            FirmwareUpdaterConfig {
                dfu: flash.dfu(),
                state: flash.state(),
            },
            &mut aligned,
        );
        block_on(updater.write_firmware(0, &UPDATE)).unwrap();
        block_on(updater.mark_updated()).unwrap();

        // Writing after marking updated is not allowed until marked as booted.
        let res: Result<(), FirmwareUpdaterError> = block_on(updater.write_firmware(0, &UPDATE));
        assert!(matches!(res, Err::<(), _>(FirmwareUpdaterError::BadState)));

        let flash = flash.into_blocking();
        let mut bootloader = BootLoader::new(BootLoaderConfig {
            active: flash.active(),
            dfu: flash.dfu(),
            state: flash.state(),
            safe: flash.safe(),
        });

        let mut page = [0; 1024];
        assert_eq!(State::Swap, bootloader.prepare_boot(&mut page).unwrap());

        let mut read_buf = [0; FIRMWARE_SIZE];
        flash.active().read(0, &mut read_buf).unwrap();
        assert_eq!(UPDATE, read_buf);
        // First DFU page is untouched
        flash.dfu().read(4096, &mut read_buf).unwrap();
        assert_eq!(ORIGINAL, read_buf);

        // Running again should cause a revert
        assert_eq!(State::Swap, bootloader.prepare_boot(&mut page).unwrap());

        // Next time we know it was reverted
        assert_eq!(State::Revert, bootloader.prepare_boot(&mut page).unwrap());

        let mut read_buf = [0; FIRMWARE_SIZE];
        flash.active().read(0, &mut read_buf).unwrap();
        assert_eq!(ORIGINAL, read_buf);
        // Last DFU page is untouched
        flash.dfu().read(0, &mut read_buf).unwrap();
        assert_eq!(UPDATE, read_buf);

        // Mark as booted
        let flash = flash.into_async();
        let mut updater = FirmwareUpdater::new(
            FirmwareUpdaterConfig {
                dfu: flash.dfu(),
                state: flash.state(),
            },
            &mut aligned,
        );
        block_on(updater.mark_booted()).unwrap();

        let flash = flash.into_blocking();
        let mut bootloader = BootLoader::new(BootLoaderConfig {
            active: flash.active(),
            dfu: flash.dfu(),
            state: flash.state(),
            safe: flash.safe(),
        });
        assert_eq!(State::Boot, bootloader.prepare_boot(&mut page).unwrap());
    }

    #[test]
    #[cfg(not(feature = "_verify"))]
    fn test_swap_state_active_page_biggest() {
        const FIRMWARE_SIZE: usize = 12288;
        let flash = AsyncTestFlash::new(BootLoaderConfig {
            active: MemFlash::<FIRMWARE_SIZE, 4096, 8>::random(),
            dfu: MemFlash::<{ FIRMWARE_SIZE + 4096 }, 4096, 4>::default(),
            state: MemFlash::<2048, 128, 4>::random(),
            safe: MemFlash::<FIRMWARE_SIZE, 4096, 8>::random(),
        });

        const ORIGINAL: [u8; FIRMWARE_SIZE] = [0x55; FIRMWARE_SIZE];
        const UPDATE: [u8; FIRMWARE_SIZE] = [0xAA; FIRMWARE_SIZE];
        let mut aligned = [0; 4];

        block_on(flash.active().erase(0, ORIGINAL.len() as u32)).unwrap();
        block_on(flash.active().write(0, &ORIGINAL)).unwrap();

        let mut updater = FirmwareUpdater::new(
            FirmwareUpdaterConfig {
                dfu: flash.dfu(),
                state: flash.state(),
            },
            &mut aligned,
        );
        block_on(updater.write_firmware(0, &UPDATE)).unwrap();
        block_on(updater.mark_updated()).unwrap();

        let flash = flash.into_blocking();
        let mut bootloader = BootLoader::new(BootLoaderConfig {
            active: flash.active(),
            dfu: flash.dfu(),
            state: flash.state(),
            safe: flash.safe(),
        });

        let mut page = [0; 4096];
        assert_eq!(State::Swap, bootloader.prepare_boot(&mut page).unwrap());

        let mut read_buf = [0; FIRMWARE_SIZE];
        flash.active().read(0, &mut read_buf).unwrap();
        assert_eq!(UPDATE, read_buf);
        // First DFU page is untouched
        flash.dfu().read(4096, &mut read_buf).unwrap();
        assert_eq!(ORIGINAL, read_buf);
    }

    #[test]
    #[cfg(not(feature = "_verify"))]
    fn test_swap_state_dfu_page_biggest() {
        const FIRMWARE_SIZE: usize = 12288;
        let flash = AsyncTestFlash::new(BootLoaderConfig {
            active: MemFlash::<FIRMWARE_SIZE, 2048, 4>::random(),
            dfu: MemFlash::<{ FIRMWARE_SIZE + 4096 }, 4096, 4>::default(),
            state: MemFlash::<2048, 128, 4>::random(),
            safe: MemFlash::<FIRMWARE_SIZE, 2048, 4>::random(),
        });

        const ORIGINAL: [u8; FIRMWARE_SIZE] = [0x55; FIRMWARE_SIZE];
        const UPDATE: [u8; FIRMWARE_SIZE] = [0xAA; FIRMWARE_SIZE];
        let mut aligned = [0; 4];

        block_on(flash.active().erase(0, ORIGINAL.len() as u32)).unwrap();
        block_on(flash.active().write(0, &ORIGINAL)).unwrap();

        let mut updater = FirmwareUpdater::new(
            FirmwareUpdaterConfig {
                dfu: flash.dfu(),
                state: flash.state(),
            },
            &mut aligned,
        );
        block_on(updater.write_firmware(0, &UPDATE)).unwrap();
        block_on(updater.mark_updated()).unwrap();

        let flash = flash.into_blocking();
        let mut bootloader = BootLoader::new(BootLoaderConfig {
            active: flash.active(),
            dfu: flash.dfu(),
            state: flash.state(),
            safe: flash.safe(),
        });
        let mut page = [0; 4096];
        assert_eq!(State::Swap, bootloader.prepare_boot(&mut page).unwrap());

        let mut read_buf = [0; FIRMWARE_SIZE];
        flash.active().read(0, &mut read_buf).unwrap();
        assert_eq!(UPDATE, read_buf);
        // First DFU page is untouched
        flash.dfu().read(4096, &mut read_buf).unwrap();
        assert_eq!(ORIGINAL, read_buf);
    }

    #[test]
    #[cfg(feature = "_verify")]
    fn test_verify() {
        // The following key setup is based on:
        // https://docs.rs/ed25519-dalek/latest/ed25519_dalek/#example

        use ed25519_dalek::{Digest, Sha512, Signature, Signer, SigningKey, VerifyingKey};
        use rand::rngs::OsRng;

        let mut csprng = OsRng {};
        let keypair = SigningKey::generate(&mut csprng);

        let firmware: &[u8] = b"This are bytes that would otherwise be firmware bytes for DFU.";
        let mut digest = Sha512::new();
        digest.update(&firmware);
        let message = digest.finalize();
        let signature: Signature = keypair.sign(&message);

        let public_key = keypair.verifying_key();

        // Setup flash
        let flash = BlockingTestFlash::new(BootLoaderConfig {
            active: MemFlash::<0, 0, 0>::default(),
            dfu: MemFlash::<4096, 4096, 4>::default(),
            state: MemFlash::<4096, 4096, 4>::default(),
            safe: MemFlash::<0, 1, 1>::default(),
        });

        let firmware_len = firmware.len();

        let mut write_buf = [0; 4096];
        write_buf[0..firmware_len].copy_from_slice(firmware);
        flash.dfu().write(0, &write_buf).unwrap();

        // On with the test
        let flash = flash.into_async();
        let mut aligned = [0; 4];
        let mut updater = FirmwareUpdater::new(
            FirmwareUpdaterConfig {
                dfu: flash.dfu(),
                state: flash.state(),
            },
            &mut aligned,
        );

        assert!(block_on(updater.verify_and_mark_updated(
            &public_key.to_bytes(),
            &signature.to_bytes(),
            firmware_len as u32,
        ))
        .is_ok());
    }

    #[test]
    #[cfg(feature = "restore")]
    fn test_backup() {
        const SIZE: usize = 4096 * 2;
        let flash = AsyncTestFlash::new(BootLoaderConfig {
            active: MemFlash::<SIZE, 4096, 4>::default(),
            dfu: MemFlash::<12288, 4096, 4>::default(),
            state: MemFlash::<4096, 4096, 4>::default(),
            safe: MemFlash::<SIZE, 4096, 4>::default(),
        });

        const ACTIVE_DATA: [u8; SIZE] = [0x11; SIZE];
        let mut aligned = [0; 4];

        block_on(flash.active().erase(0, SIZE as u32)).unwrap();
        block_on(flash.active().write(0, &ACTIVE_DATA)).unwrap();

        let mut updater = FirmwareUpdater::new(
            FirmwareUpdaterConfig {
                dfu: flash.dfu(),
                state: flash.state(),
            },
            &mut aligned,
        );
        block_on(updater.mark_backup()).unwrap();

        let flash = flash.into_blocking();
        let mut bootloader = BootLoader::new(BootLoaderConfig {
            active: flash.active(),
            dfu: flash.dfu(),
            state: flash.state(),
            safe: flash.safe(),
        });
        let mut page = [0; 4096];
        assert_eq!(State::Boot, bootloader.prepare_boot(&mut page).unwrap());

        let mut buf = [0; SIZE];
        flash.dfu().read(0, &mut buf).unwrap();
        assert_eq!(ACTIVE_DATA, buf);
    }

    #[test]
    #[cfg(feature = "restore")]
    fn test_restore() {
        const SIZE: usize = 4096 * 2;
        let flash = AsyncTestFlash::new(BootLoaderConfig {
            active: MemFlash::<SIZE, 4096, 4>::default(),
            dfu: MemFlash::<12288, 4096, 4>::default(),
            state: MemFlash::<4096, 4096, 4>::default(),
            safe: MemFlash::<SIZE, 4096, 4>::default(),
        });

        const ACTIVE_DATA: [u8; SIZE] = [0x11; SIZE];
        const BACKUP_DATA: [u8; SIZE] = [0x22; SIZE];
        let mut aligned = [0; 4];

        block_on(flash.active().erase(0, SIZE as u32)).unwrap();
        block_on(flash.active().write(0, &ACTIVE_DATA)).unwrap();
        block_on(flash.dfu().erase(0, 12288u32)).unwrap();
        block_on(flash.dfu().write(0, &BACKUP_DATA)).unwrap();

        let mut updater = FirmwareUpdater::new(
            FirmwareUpdaterConfig {
                dfu: flash.dfu(),
                state: flash.state(),
            },
            &mut aligned,
        );
        block_on(updater.mark_restore()).unwrap();

        let flash = flash.into_blocking();
        let mut bootloader = BootLoader::new(BootLoaderConfig {
            active: flash.active(),
            dfu: flash.dfu(),
            state: flash.state(),
            safe: flash.safe(),
        });
        let mut page = [0; 4096];
        assert_eq!(State::Boot, bootloader.prepare_boot(&mut page).unwrap());

        let mut buf = [0; SIZE];
        flash.active().read(0, &mut buf).unwrap();
        assert_eq!(BACKUP_DATA, buf);
    }

    #[test]
    #[cfg(feature = "safe")]
    fn test_safe_magic_trigger() {
        /* 1. Create AsyncTestFlash */
        let flash = AsyncTestFlash::new(BootLoaderConfig {
            active: MemFlash::<0x1000, 0x1000, 4>::default(),
            dfu: MemFlash::<0x2000, 0x1000, 4>::default(),
            state: MemFlash::<0x1000, 0x1000, 4>::default(),
            safe: MemFlash::<0x1000, 0x1000, 4>::default(),
        });
        let mut aligned = [0; 4];

        /* 2. Create FirmwareUpdater */
        let mut updater = FirmwareUpdater::new(
            FirmwareUpdaterConfig {
                dfu: flash.dfu(),
                state: flash.state(),
            },
            &mut aligned,
        );

        /* 3. Mark safe flag */
        block_on(updater.mark_safe()).unwrap();

        /* 4. Get current status */
        let state = block_on(updater.get_state()).unwrap();

        /* 5. Check if current state is safe */
        assert_eq!(State::Safe, state);
    }

    #[test]
    #[cfg(feature = "safe")]
    fn test_safe_copy_function() {
        /* 1. Create AsyncTestFlash */
        let flash = AsyncTestFlash::new(BootLoaderConfig {
            active: MemFlash::<0x1000, 0x1000, 4>::default(),
            dfu: MemFlash::<0x2000, 0x1000, 4>::default(),
            state: MemFlash::<0x1000, 0x1000, 4>::default(),
            safe: MemFlash::<0x1000, 0x1000, 4>::default(),
        });
        let mut aligned = [0; 4];

        /* 2. Create FirmwareUpdater */
        let mut updater = FirmwareUpdater::new(
            FirmwareUpdaterConfig {
                dfu: flash.dfu(),
                state: flash.state(),
            },
            &mut aligned,
        );

        /* 3. Reset active, safe. Write data to safe */
        const DATA_SAFE: [u8; 0x1000] = [0x10; 0x1000];
        block_on(flash.active().erase(0, 0x1000 as u32)).unwrap();
        block_on(flash.safe().erase(0, 0x1000 as u32)).unwrap();
        block_on(flash.safe().write(0, &DATA_SAFE)).unwrap();

        /* 4. Mark safe flag */
        block_on(updater.mark_safe()).unwrap();

        /* 5. Call prepare_boot and check if safe operation is done successfully */
        let flash = flash.into_blocking();
        let mut bootloader = BootLoader::new(BootLoaderConfig {
            active: flash.active(),
            dfu: flash.dfu(),
            state: flash.state(),
            safe: flash.safe(),
        });
        let mut page = [0; 4096];
        assert_eq!(State::Boot, bootloader.prepare_boot(&mut page).unwrap());

        // 6. read data from active area
        let mut read_buf = [0; 0x1000];
        flash.active().read(0, &mut read_buf).unwrap();

        // 7. Compare safe data and active data
        assert_eq!(DATA_SAFE, read_buf);
    }
}
