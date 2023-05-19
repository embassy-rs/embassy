#![cfg_attr(feature = "nightly", feature(async_fn_in_trait))]
#![no_std]
#![warn(missing_docs)]
#![doc = include_str!("../README.md")]
mod fmt;

mod boot_loader;
mod digest_adapters;
mod firmware_updater;
mod mem_flash;
mod partition;

pub use boot_loader::{BootError, BootFlash, BootLoader, FlashConfig, MultiFlashConfig, SingleFlashConfig};
pub use firmware_updater::{FirmwareUpdater, FirmwareUpdaterError};
pub use partition::Partition;

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
    use futures::executor::block_on;

    use super::*;
    use crate::mem_flash::MemFlash;

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
        const STATE: Partition = Partition::new(0, 4096);
        const ACTIVE: Partition = Partition::new(4096, 61440);
        const DFU: Partition = Partition::new(61440, 122880);

        let mut flash = MemFlash::<131072, 4096, 4>::default();
        flash.mem[0..4].copy_from_slice(&[BOOT_MAGIC; 4]);
        let mut flash = SingleFlashConfig::new(&mut flash);

        let mut bootloader: BootLoader = BootLoader::new(ACTIVE, DFU, STATE);

        let mut page = [0; 4096];
        assert_eq!(State::Boot, bootloader.prepare_boot(&mut flash, &mut page).unwrap());
    }

    #[test]
    #[cfg(all(feature = "nightly", not(feature = "_verify")))]
    fn test_swap_state() {
        const STATE: Partition = Partition::new(0, 4096);
        const ACTIVE: Partition = Partition::new(4096, 61440);
        const DFU: Partition = Partition::new(61440, 122880);
        let mut flash = MemFlash::<131072, 4096, 4>::random();

        let original = [rand::random::<u8>(); ACTIVE.size() as usize];
        let update = [rand::random::<u8>(); ACTIVE.size() as usize];
        let mut aligned = [0; 4];

        flash.program(ACTIVE.from, &original).unwrap();

        let mut bootloader: BootLoader = BootLoader::new(ACTIVE, DFU, STATE);
        let mut updater = FirmwareUpdater::new(DFU, STATE);
        block_on(updater.write_firmware(0, &update, &mut flash)).unwrap();
        block_on(updater.mark_updated(&mut flash, &mut aligned)).unwrap();

        let mut page = [0; 1024];
        assert_eq!(
            State::Swap,
            bootloader
                .prepare_boot(&mut SingleFlashConfig::new(&mut flash), &mut page)
                .unwrap()
        );

        flash.assert_eq(ACTIVE.from, &update);
        // First DFU page is untouched
        flash.assert_eq(DFU.from + 4096, &original);

        // Running again should cause a revert
        assert_eq!(
            State::Swap,
            bootloader
                .prepare_boot(&mut SingleFlashConfig::new(&mut flash), &mut page)
                .unwrap()
        );

        flash.assert_eq(ACTIVE.from, &original);
        // Last page is untouched
        flash.assert_eq(DFU.from, &update);

        // Mark as booted
        block_on(updater.mark_booted(&mut flash, &mut aligned)).unwrap();
        assert_eq!(
            State::Boot,
            bootloader
                .prepare_boot(&mut SingleFlashConfig::new(&mut flash), &mut page)
                .unwrap()
        );
    }

    #[test]
    #[cfg(all(feature = "nightly", not(feature = "_verify")))]
    fn test_separate_flash_active_page_biggest() {
        const STATE: Partition = Partition::new(2048, 4096);
        const ACTIVE: Partition = Partition::new(4096, 16384);
        const DFU: Partition = Partition::new(0, 16384);

        let mut active = MemFlash::<16384, 4096, 8>::random();
        let mut dfu = MemFlash::<16384, 2048, 8>::random();
        let mut state = MemFlash::<4096, 128, 4>::random();
        let mut aligned = [0; 4];

        let original = [rand::random::<u8>(); ACTIVE.size() as usize];
        let update = [rand::random::<u8>(); ACTIVE.size() as usize];

        active.program(ACTIVE.from, &original).unwrap();

        let mut updater = FirmwareUpdater::new(DFU, STATE);

        block_on(updater.write_firmware(0, &update, &mut dfu)).unwrap();
        block_on(updater.mark_updated(&mut state, &mut aligned)).unwrap();

        let mut bootloader: BootLoader = BootLoader::new(ACTIVE, DFU, STATE);
        let mut page = [0; 4096];

        assert_eq!(
            State::Swap,
            bootloader
                .prepare_boot(&mut MultiFlashConfig::new(&mut active, &mut state, &mut dfu), &mut page)
                .unwrap()
        );

        active.assert_eq(ACTIVE.from, &update);
        // First DFU page is untouched
        dfu.assert_eq(DFU.from + 4096, &original);
    }

    #[test]
    #[cfg(all(feature = "nightly", not(feature = "_verify")))]
    fn test_separate_flash_dfu_page_biggest() {
        const STATE: Partition = Partition::new(2048, 4096);
        const ACTIVE: Partition = Partition::new(4096, 16384);
        const DFU: Partition = Partition::new(0, 16384);

        let mut aligned = [0; 4];
        let mut active = MemFlash::<16384, 2048, 4>::random();
        let mut dfu = MemFlash::<16384, 4096, 8>::random();
        let mut state = MemFlash::<4096, 128, 4>::random();

        let original = [rand::random::<u8>(); ACTIVE.size() as usize];
        let update = [rand::random::<u8>(); ACTIVE.size() as usize];

        active.program(ACTIVE.from, &original).unwrap();

        let mut updater = FirmwareUpdater::new(DFU, STATE);

        block_on(updater.write_firmware(0, &update, &mut dfu)).unwrap();
        block_on(updater.mark_updated(&mut state, &mut aligned)).unwrap();

        let mut bootloader: BootLoader = BootLoader::new(ACTIVE, DFU, STATE);
        let mut page = [0; 4096];
        assert_eq!(
            State::Swap,
            bootloader
                .prepare_boot(
                    &mut MultiFlashConfig::new(&mut active, &mut state, &mut dfu,),
                    &mut page
                )
                .unwrap()
        );

        active.assert_eq(ACTIVE.from, &update);
        // First DFU page is untouched
        dfu.assert_eq(DFU.from + 4096, &original);
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

        const STATE: Partition = Partition::new(0, 4096);
        const DFU: Partition = Partition::new(4096, 8192);
        let mut flash = MemFlash::<8192, 4096, 4>::default();

        let firmware_len = firmware.len();

        let mut write_buf = [0; 4096];
        write_buf[0..firmware_len].copy_from_slice(firmware);
        DFU.write_blocking(&mut flash, 0, &write_buf).unwrap();

        // On with the test

        let mut updater = FirmwareUpdater::new(DFU, STATE);

        let mut aligned = [0; 4];

        assert!(block_on(updater.verify_and_mark_updated(
            &mut flash,
            &public_key.to_bytes(),
            &signature.to_bytes(),
            firmware_len as u32,
            &mut aligned,
        ))
        .is_ok());
    }
}
