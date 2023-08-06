#![cfg_attr(feature = "nightly", feature(async_fn_in_trait))]
#![no_std]
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
pub(crate) const STATE_ERASE_VALUE: u8 = 0xFF;
pub use boot_loader::{BootError, BootLoader, BootLoaderConfig};
pub use firmware_updater::{
    BlockingFirmwareState, BlockingFirmwareUpdater, FirmwareUpdaterConfig, FirmwareUpdaterError,
};
#[cfg(feature = "nightly")]
pub use firmware_updater::{FirmwareState, FirmwareUpdater};

pub(crate) const BOOT_MAGIC: u8 = 0xD0;
pub(crate) const SWAP_MAGIC: u8 = 0xF0;

/// The state of the bootloader after running prepare.
#[derive(PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum State {
    /// Bootloader is ready to boot the active partition.
    Boot,
    /// Bootloader has swapped the active partition with the dfu partition and will attempt boot.
    Swap,
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

#[cfg(test)]
mod tests {
    #![allow(unused_imports)]

    use embedded_storage::nor_flash::{NorFlash, ReadNorFlash};
    #[cfg(feature = "nightly")]
    use embedded_storage_async::nor_flash::NorFlash as AsyncNorFlash;
    use futures::executor::block_on;

    use super::*;
    use crate::boot_loader::BootLoaderConfig;
    use crate::firmware_updater::FirmwareUpdaterConfig;
    use crate::mem_flash::MemFlash;
    #[cfg(feature = "nightly")]
    use crate::test_flash::AsyncTestFlash;
    use crate::test_flash::BlockingTestFlash;

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
        let flash = BlockingTestFlash::new(BootLoaderConfig {
            active: MemFlash::<57344, 4096, 4>::default(),
            dfu: MemFlash::<61440, 4096, 4>::default(),
            state: MemFlash::<4096, 4096, 4>::default(),
        });

        flash.state().write(0, &[BOOT_MAGIC; 4]).unwrap();

        let mut bootloader = BootLoader::new(BootLoaderConfig {
            active: flash.active(),
            dfu: flash.dfu(),
            state: flash.state(),
        });

        let mut page = [0; 4096];
        assert_eq!(State::Boot, bootloader.prepare_boot(&mut page).unwrap());
    }

    #[test]
    #[cfg(all(feature = "nightly", not(feature = "_verify")))]
    fn test_swap_state() {
        const FIRMWARE_SIZE: usize = 57344;
        let flash = AsyncTestFlash::new(BootLoaderConfig {
            active: MemFlash::<FIRMWARE_SIZE, 4096, 4>::default(),
            dfu: MemFlash::<61440, 4096, 4>::default(),
            state: MemFlash::<4096, 4096, 4>::default(),
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
        });
        assert_eq!(State::Boot, bootloader.prepare_boot(&mut page).unwrap());
    }

    #[test]
    #[cfg(all(feature = "nightly", not(feature = "_verify")))]
    fn test_swap_state_active_page_biggest() {
        const FIRMWARE_SIZE: usize = 12288;
        let flash = AsyncTestFlash::new(BootLoaderConfig {
            active: MemFlash::<12288, 4096, 8>::random(),
            dfu: MemFlash::<16384, 2048, 8>::random(),
            state: MemFlash::<2048, 128, 4>::random(),
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
    #[cfg(all(feature = "nightly", not(feature = "_verify")))]
    fn test_swap_state_dfu_page_biggest() {
        const FIRMWARE_SIZE: usize = 12288;
        let flash = AsyncTestFlash::new(BootLoaderConfig {
            active: MemFlash::<FIRMWARE_SIZE, 2048, 4>::random(),
            dfu: MemFlash::<16384, 4096, 8>::random(),
            state: MemFlash::<2048, 128, 4>::random(),
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
    #[cfg(all(feature = "nightly", feature = "_verify"))]
    fn test_verify() {
        // The following key setup is based on:
        // https://docs.rs/ed25519-dalek/latest/ed25519_dalek/#example

        use ed25519_dalek::Keypair;
        use rand::rngs::OsRng;

        let mut csprng = OsRng {};
        let keypair: Keypair = Keypair::generate(&mut csprng);

        use ed25519_dalek::{Digest, Sha512, Signature, Signer};
        let firmware: &[u8] = b"This are bytes that would otherwise be firmware bytes for DFU.";
        let mut digest = Sha512::new();
        digest.update(&firmware);
        let message = digest.finalize();
        let signature: Signature = keypair.sign(&message);

        use ed25519_dalek::PublicKey;
        let public_key: PublicKey = keypair.public;

        // Setup flash
        let flash = BlockingTestFlash::new(BootLoaderConfig {
            active: MemFlash::<0, 0, 0>::default(),
            dfu: MemFlash::<4096, 4096, 4>::default(),
            state: MemFlash::<4096, 4096, 4>::default(),
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
}
