#![feature(async_fn_in_trait)]
#![allow(incomplete_features)]
#![no_std]
#![warn(missing_docs)]
#![doc = include_str!("../README.md")]
mod fmt;

mod boot_loader;
mod firmware_updater;
mod firmware_writer;
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
    use core::convert::Infallible;

    use embedded_storage::nor_flash::{ErrorType, NorFlash, ReadNorFlash};
    use embedded_storage_async::nor_flash::{NorFlash as AsyncNorFlash, ReadNorFlash as AsyncReadNorFlash};
    use futures::executor::block_on;

    use super::*;

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

        let mut flash = MemFlash::<131072, 4096, 4>([0xff; 131072]);
        flash.0[0..4].copy_from_slice(&[BOOT_MAGIC; 4]);
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
        let mut flash = MemFlash::<131072, 4096, 4>([0xff; 131072]);

        let original: [u8; ACTIVE.len()] = [rand::random::<u8>(); ACTIVE.len()];
        let update: [u8; DFU.len()] = [rand::random::<u8>(); DFU.len()];
        let mut aligned = [0; 4];

        for i in ACTIVE.from..ACTIVE.to {
            flash.0[i] = original[i - ACTIVE.from];
        }

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
            assert_eq!(flash.0[i], update[i - ACTIVE.from], "Index {}", i);
        }

        // First DFU page is untouched
        for i in DFU.from + 4096..DFU.to {
            assert_eq!(flash.0[i], original[i - DFU.from - 4096], "Index {}", i);
        }

        // Running again should cause a revert
        assert_eq!(
            State::Swap,
            bootloader
                .prepare_boot(&mut SingleFlashConfig::<_, 4096>::new(&mut flash), &mut block_buffer)
                .unwrap()
        );

        for i in ACTIVE.from..ACTIVE.to {
            assert_eq!(flash.0[i], original[i - ACTIVE.from], "Index {}", i);
        }

        // Last page is untouched
        for i in DFU.from..DFU.to - 4096 {
            assert_eq!(flash.0[i], update[i - DFU.from], "Index {}", i);
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
    fn test_separate_flash_active_page_biggest() {
        const STATE: Partition = Partition::new(2048, 4096);
        const ACTIVE: Partition = Partition::new(4096, 16384);
        const DFU: Partition = Partition::new(0, 16384);

        let mut active = MemFlash::<16384, 4096, 8>([0xff; 16384]);
        let mut dfu = MemFlash::<16384, 2048, 8>([0xff; 16384]);
        let mut state = MemFlash::<4096, 128, 4>([0xff; 4096]);
        let mut aligned = [0; 4];

        let original: [u8; ACTIVE.len()] = [rand::random::<u8>(); ACTIVE.len()];
        let update: [u8; DFU.len()] = [rand::random::<u8>(); DFU.len()];

        for i in ACTIVE.from..ACTIVE.to {
            active.0[i] = original[i - ACTIVE.from];
        }

        let mut updater = FirmwareUpdater::new(DFU, STATE);

        let mut offset = 0;
        for chunk in update.chunks(2048) {
            block_on(updater.write_firmware(offset, chunk, &mut dfu, chunk.len())).unwrap();
            offset += chunk.len();
        }
        block_on(updater.mark_updated(&mut state, &mut aligned)).unwrap();

        let mut bootloader: BootLoader = BootLoader::new(ACTIVE, DFU, STATE);
        let mut block_buffer = [0; 4096];

        assert_eq!(
            State::Swap,
            bootloader
                .prepare_boot(
                    &mut MultiFlashConfig::<_, _, _, 4096>::new(&mut active, &mut state, &mut dfu),
                    &mut block_buffer,
                )
                .unwrap()
        );

        for i in ACTIVE.from..ACTIVE.to {
            assert_eq!(active.0[i], update[i - ACTIVE.from], "Index {}", i);
        }

        // First DFU page is untouched
        for i in DFU.from + 4096..DFU.to {
            assert_eq!(dfu.0[i], original[i - DFU.from - 4096], "Index {}", i);
        }
    }

    #[test]
    #[cfg(not(feature = "_verify"))]
    fn test_separate_flash_dfu_page_biggest() {
        const STATE: Partition = Partition::new(2048, 4096);
        const ACTIVE: Partition = Partition::new(4096, 16384);
        const DFU: Partition = Partition::new(0, 16384);

        let mut aligned = [0; 4];
        let mut active = MemFlash::<16384, 2048, 4>([0xff; 16384]);
        let mut dfu = MemFlash::<16384, 4096, 8>([0xff; 16384]);
        let mut state = MemFlash::<4096, 128, 4>([0xff; 4096]);

        let original: [u8; ACTIVE.len()] = [rand::random::<u8>(); ACTIVE.len()];
        let update: [u8; DFU.len()] = [rand::random::<u8>(); DFU.len()];

        for i in ACTIVE.from..ACTIVE.to {
            active.0[i] = original[i - ACTIVE.from];
        }

        let mut updater = FirmwareUpdater::new(DFU, STATE);

        let mut offset = 0;
        for chunk in update.chunks(4096) {
            block_on(updater.write_firmware(offset, chunk, &mut dfu, chunk.len())).unwrap();
            offset += chunk.len();
        }
        block_on(updater.mark_updated(&mut state, &mut aligned)).unwrap();

        let mut bootloader: BootLoader = BootLoader::new(ACTIVE, DFU, STATE);
        let mut block_buffer = [0; 4096];
        assert_eq!(
            State::Swap,
            bootloader
                .prepare_boot(
                    &mut MultiFlashConfig::<_, _, _, 4096>::new(&mut active, &mut state, &mut dfu,),
                    &mut block_buffer,
                )
                .unwrap()
        );

        for i in ACTIVE.from..ACTIVE.to {
            assert_eq!(active.0[i], update[i - ACTIVE.from], "Index {}", i);
        }

        // First DFU page is untouched
        for i in DFU.from + 4096..DFU.to {
            assert_eq!(dfu.0[i], original[i - DFU.from - 4096], "Index {}", i);
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
        let mut flash = MemFlash::<8192, 4096, 4>([0xff; 8192]);

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

    pub struct MemFlash<const SIZE: usize, const ERASE_SIZE: usize, const WRITE_SIZE: usize>(pub [u8; SIZE]);

    impl<const SIZE: usize, const ERASE_SIZE: usize, const WRITE_SIZE: usize> NorFlash
        for MemFlash<SIZE, ERASE_SIZE, WRITE_SIZE>
    {
        const WRITE_SIZE: usize = WRITE_SIZE;
        const ERASE_SIZE: usize = ERASE_SIZE;
        fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
            let from = from as usize;
            let to = to as usize;
            assert!(from % ERASE_SIZE == 0);
            assert!(to % ERASE_SIZE == 0, "To: {}, erase size: {}", to, ERASE_SIZE);
            for i in from..to {
                self.0[i] = 0xFF;
            }
            Ok(())
        }

        fn write(&mut self, offset: u32, data: &[u8]) -> Result<(), Self::Error> {
            assert!(data.len() % WRITE_SIZE == 0);
            assert!(offset as usize % WRITE_SIZE == 0);
            assert!(offset as usize + data.len() <= SIZE);

            self.0[offset as usize..offset as usize + data.len()].copy_from_slice(data);

            Ok(())
        }
    }

    impl<const SIZE: usize, const ERASE_SIZE: usize, const WRITE_SIZE: usize> ErrorType
        for MemFlash<SIZE, ERASE_SIZE, WRITE_SIZE>
    {
        type Error = Infallible;
    }

    impl<const SIZE: usize, const ERASE_SIZE: usize, const WRITE_SIZE: usize> ReadNorFlash
        for MemFlash<SIZE, ERASE_SIZE, WRITE_SIZE>
    {
        const READ_SIZE: usize = 1;

        fn read(&mut self, offset: u32, buf: &mut [u8]) -> Result<(), Self::Error> {
            let len = buf.len();
            buf[..].copy_from_slice(&self.0[offset as usize..offset as usize + len]);
            Ok(())
        }

        fn capacity(&self) -> usize {
            SIZE
        }
    }

    impl<const SIZE: usize, const ERASE_SIZE: usize, const WRITE_SIZE: usize> AsyncReadNorFlash
        for MemFlash<SIZE, ERASE_SIZE, WRITE_SIZE>
    {
        const READ_SIZE: usize = 1;

        async fn read(&mut self, offset: u32, buf: &mut [u8]) -> Result<(), Self::Error> {
            let len = buf.len();
            buf[..].copy_from_slice(&self.0[offset as usize..offset as usize + len]);
            Ok(())
        }

        fn capacity(&self) -> usize {
            SIZE
        }
    }

    impl<const SIZE: usize, const ERASE_SIZE: usize, const WRITE_SIZE: usize> AsyncNorFlash
        for MemFlash<SIZE, ERASE_SIZE, WRITE_SIZE>
    {
        const WRITE_SIZE: usize = WRITE_SIZE;
        const ERASE_SIZE: usize = ERASE_SIZE;

        async fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
            let from = from as usize;
            let to = to as usize;
            assert!(from % ERASE_SIZE == 0);
            assert!(to % ERASE_SIZE == 0);
            for i in from..to {
                self.0[i] = 0xFF;
            }
            Ok(())
        }

        async fn write(&mut self, offset: u32, data: &[u8]) -> Result<(), Self::Error> {
            info!("Writing {} bytes to 0x{:x}", data.len(), offset);
            assert!(data.len() % WRITE_SIZE == 0);
            assert!(offset as usize % WRITE_SIZE == 0);
            assert!(
                offset as usize + data.len() <= SIZE,
                "OFFSET: {}, LEN: {}, FLASH SIZE: {}",
                offset,
                data.len(),
                SIZE
            );

            self.0[offset as usize..offset as usize + data.len()].copy_from_slice(data);

            Ok(())
        }
    }
}
