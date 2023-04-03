#![feature(async_fn_in_trait)]
#![allow(incomplete_features)]
#![no_std]
#![warn(missing_docs)]
#![doc = include_str!("../README.md")]
mod fmt;

mod boot_loader;
mod firmware_updater;
mod firmware_writer;
mod mem_flash;
mod partition;

pub use boot_loader::{BootError, BootFlash, BootLoader, FlashConfig, MultiFlashConfig, SingleFlashConfig};
pub use firmware_updater::{FirmwareUpdater, FirmwareUpdaterError};
pub use firmware_writer::FirmwareWriter;
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
    use embedded_storage::nor_flash::NorFlashErrorKind;
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
        STATE.write_blocking(&mut flash, 0, &[BOOT_MAGIC; 4]).unwrap();

        let mut flash = SingleFlashConfig::<_, 4096>::new(&mut flash);

        let mut bootloader: BootLoader = BootLoader::new(ACTIVE, DFU, STATE);

        let mut block_buffer = [0; 4096];
        assert_eq!(
            State::Boot,
            bootloader.prepare_boot(&mut flash, &mut block_buffer).unwrap()
        );
    }

    #[test]
    #[cfg(not(feature = "_verify"))]
    fn test_swap_state() {
        const STATE: Partition = Partition::new(0, 4096);
        const ACTIVE: Partition = Partition::new(4096, 61440);
        const DFU: Partition = Partition::new(61440, 122880);
        let mut flash = MemFlash::<131072, 4096, 4>::random().with_limited_erase_before_write_verification(4..);

        let original: [u8; ACTIVE.len()] = [rand::random::<u8>(); ACTIVE.len()];
        let update: [u8; DFU.len()] = [rand::random::<u8>(); DFU.len()];
        let mut aligned = [0; 4];

        ACTIVE.wipe_blocking(&mut flash).unwrap();
        ACTIVE.write_blocking(&mut flash, 0, original.as_slice()).unwrap();

        let mut bootloader: BootLoader = BootLoader::new(ACTIVE, DFU, STATE);
        let mut updater = FirmwareUpdater::new(DFU, STATE);
        let mut offset = 0;
        for chunk in update.chunks(4096) {
            block_on(updater.write_firmware(offset, chunk, &mut flash, 4096)).unwrap();
            offset += chunk.len();
        }
        block_on(updater.mark_updated(&mut flash, &mut aligned)).unwrap();

        let mut block_buffer = [0; 4096];
        assert_eq!(
            State::Swap,
            bootloader
                .prepare_boot(&mut SingleFlashConfig::<_, 4096>::new(&mut flash), &mut block_buffer)
                .unwrap()
        );

        for i in ACTIVE.from..ACTIVE.to {
            assert_eq!(flash.mem[i], update[i - ACTIVE.from], "Index {}", i);
        }

        // First DFU page is untouched
        for i in DFU.from + 4096..DFU.to {
            assert_eq!(flash.mem[i], original[i - DFU.from - 4096], "Index {}", i);
        }

        // Running again should cause a revert
        // The prepare_boot call does a write to the magic bytes in state flash without a preceeding erase
        // Allow this temporarely.
        flash = flash.with_limited_erase_before_write_verification(4..);
        assert_eq!(
            State::Swap,
            bootloader
                .prepare_boot(&mut SingleFlashConfig::<_, 4096>::new(&mut flash), &mut block_buffer)
                .unwrap()
        );
        flash = flash.with_limited_erase_before_write_verification(..);

        for i in ACTIVE.from..ACTIVE.to {
            assert_eq!(flash.mem[i], original[i - ACTIVE.from], "Index {}", i);
        }

        // Last page is untouched
        for i in DFU.from..DFU.to - 4096 {
            assert_eq!(flash.mem[i], update[i - DFU.from], "Index {}", i);
        }

        // Mark as booted
        block_on(updater.mark_booted(&mut flash, &mut aligned)).unwrap();
        assert_eq!(
            State::Boot,
            bootloader
                .prepare_boot(&mut SingleFlashConfig::<_, 4096>::new(&mut flash), &mut block_buffer)
                .unwrap()
        );
    }

    #[test]
    #[cfg(not(feature = "_verify"))]
    fn test_separate_flash() {
        // Active erase size is biggest, matches block size
        test_separate_flash_case::<4096, 4096, 8, 2048, 8>();

        // DFU erase size is biggest, matches block size
        test_separate_flash_case::<4096, 2048, 4, 4096, 8>();

        // Active erase size is biggest, block size is smaller
        test_separate_flash_case::<2048, 4096, 8, 2048, 8>();

        // DFU erase size is biggest, block size is larger
        test_separate_flash_case::<2048, 2048, 4, 4096, 8>();

        // Active erase size is biggest, block size is larger
        test_separate_flash_case::<4096, 2048, 8, 1024, 8>();

        // DFU erase size is biggest, block size is larger
        test_separate_flash_case::<4096, 1024, 4, 2048, 8>();
    }

    #[cfg(not(feature = "_verify"))]
    fn test_separate_flash_case<
        const BLOCK_SIZE: usize,
        const ACTIVE_ERASE: usize,
        const ACTIVE_WRITE: usize,
        const DFU_ERASE: usize,
        const DFU_WRITE: usize,
    >() {
        const STATE: Partition = Partition::new(2048, 4096);
        const ACTIVE: Partition = Partition::new(4096, 16384);
        const DFU: Partition = Partition::new(0, 16384);

        let mut active = MemFlash::<16384, ACTIVE_ERASE, ACTIVE_WRITE>::random();
        let mut dfu = MemFlash::<16384, DFU_ERASE, DFU_WRITE>::random();
        let mut state = MemFlash::<4096, 128, 4>::random().with_limited_erase_before_write_verification(2048 + 4..);
        let mut aligned = [0; 4];

        let original: [u8; ACTIVE.len()] = [rand::random::<u8>(); ACTIVE.len()];
        let update: [u8; DFU.len()] = [rand::random::<u8>(); DFU.len()];

        ACTIVE.wipe_blocking(&mut active).unwrap();
        ACTIVE.write_blocking(&mut active, 0, original.as_slice()).unwrap();

        let mut updater = FirmwareUpdater::new(DFU, STATE);

        let mut offset = 0;
        for chunk in update.chunks(DFU_ERASE) {
            block_on(updater.write_firmware(offset, chunk, &mut dfu, chunk.len())).unwrap();
            offset += chunk.len();
        }
        block_on(updater.mark_updated(&mut state, &mut aligned)).unwrap();

        let mut bootloader: BootLoader = BootLoader::new(ACTIVE, DFU, STATE);
        let mut block_buffer = [0; BLOCK_SIZE];

        assert_eq!(
            State::Swap,
            bootloader
                .prepare_boot(
                    &mut MultiFlashConfig::<_, _, _, BLOCK_SIZE>::new(&mut active, &mut state, &mut dfu),
                    &mut block_buffer,
                )
                .unwrap()
        );

        for i in ACTIVE.from..ACTIVE.to {
            assert_eq!(active.mem[i], update[i - ACTIVE.from], "Index {}", i);
        }

        // First DFU is untouched
        let max_erase = core::cmp::max(core::cmp::max(ACTIVE_ERASE, DFU_ERASE), BLOCK_SIZE);
        for i in DFU.from + max_erase..DFU.to {
            assert_eq!(dfu.mem[i], original[i - DFU.from - max_erase], "Index {}", i);
        }
    }

    #[test]
    #[cfg(not(feature = "_verify"))]
    fn test_separate_flash_failure_during_active_backup_should_continue() {
        // Active erase size is biggest, matches block size
        test_separate_flash_failure_during_active_backup_should_continue_case::<4096, 4096, 8, 2048, 8>();

        // DFU erase size is biggest, matches block size
        test_separate_flash_failure_during_active_backup_should_continue_case::<4096, 2048, 4, 4096, 8>();

        // Active erase size is biggest, block size is smaller
        test_separate_flash_failure_during_active_backup_should_continue_case::<2048, 4096, 8, 2048, 8>();

        // DFU erase size is biggest, block size is larger
        test_separate_flash_failure_during_active_backup_should_continue_case::<2048, 2048, 4, 4096, 8>();

        // Active erase size is biggest, block size is larger
        test_separate_flash_failure_during_active_backup_should_continue_case::<4096, 2048, 8, 1024, 8>();

        // DFU erase size is biggest, block size is larger
        test_separate_flash_failure_during_active_backup_should_continue_case::<4096, 1024, 4, 2048, 8>();
    }

    #[cfg(not(feature = "_verify"))]
    fn test_separate_flash_failure_during_active_backup_should_continue_case<
        const BLOCK_SIZE: usize,
        const ACTIVE_ERASE: usize,
        const ACTIVE_WRITE: usize,
        const DFU_ERASE: usize,
        const DFU_WRITE: usize,
    >() {
        const STATE: Partition = Partition::new(2048, 4096);
        const ACTIVE: Partition = Partition::new(4096, 16384);
        const DFU: Partition = Partition::new(0, 16384);

        let mut active = MemFlash::<16384, ACTIVE_ERASE, ACTIVE_WRITE>::random();
        let mut dfu = MemFlash::<16384, DFU_ERASE, DFU_WRITE>::random();
        let mut state = MemFlash::<4096, 128, 4>::random()
            .allow_same_write(true)
            .with_limited_erase_before_write_verification(2048 + 4..);
        let mut aligned = [0; 4];

        let original: [u8; ACTIVE.len()] = [rand::random::<u8>(); ACTIVE.len()];
        let update: [u8; DFU.len()] = [rand::random::<u8>(); DFU.len()];

        ACTIVE.wipe_blocking(&mut active).unwrap();
        ACTIVE.write_blocking(&mut active, 0, original.as_slice()).unwrap();

        let mut updater = FirmwareUpdater::new(DFU, STATE);

        let mut offset = 0;
        for chunk in update.chunks(DFU_ERASE) {
            block_on(updater.write_firmware(offset, chunk, &mut dfu, chunk.len())).unwrap();
            offset += chunk.len();
        }
        block_on(updater.mark_updated(&mut state, &mut aligned)).unwrap();

        let mut bootloader: BootLoader = BootLoader::new(ACTIVE, DFU, STATE);
        let mut block_buffer = [0; BLOCK_SIZE];

        // Let write return error after one success
        dfu.pending_write_successes = Some(1);

        assert_eq!(
            Err(BootError::Flash(NorFlashErrorKind::Other)),
            bootloader.prepare_boot(
                &mut MultiFlashConfig::<_, _, _, BLOCK_SIZE>::new(&mut active, &mut state, &mut dfu),
                &mut block_buffer,
            )
        );

        // Let write work again
        dfu.pending_write_successes = None;

        // Continue swapping
        assert_eq!(
            State::Swap,
            bootloader
                .prepare_boot(
                    &mut MultiFlashConfig::<_, _, _, BLOCK_SIZE>::new(&mut active, &mut state, &mut dfu),
                    &mut block_buffer,
                )
                .unwrap()
        );

        for i in ACTIVE.from..ACTIVE.to {
            assert_eq!(active.mem[i], update[i - ACTIVE.from], "Index {}", i);
        }

        // First DFU is untouched
        let max_erase = core::cmp::max(core::cmp::max(ACTIVE_ERASE, DFU_ERASE), BLOCK_SIZE);
        for i in DFU.from + max_erase..DFU.to {
            assert_eq!(dfu.mem[i], original[i - DFU.from - max_erase], "Index {}", i);
        }
    }

    #[test]
    #[cfg(feature = "_verify")]
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
        NorFlash::write(&mut flash, DFU.from as u32, &write_buf).unwrap();

        // On with the test

        let mut updater = FirmwareUpdater::new(DFU, STATE);

        let mut aligned = [0; 4];

        assert!(block_on(updater.verify_and_mark_updated(
            &mut flash,
            &public_key.to_bytes(),
            &signature.to_bytes(),
            firmware_len,
            &mut aligned,
        ))
        .is_ok());
    }
}
