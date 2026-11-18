use embassy_crypto::Sha512;
use embassy_crypto::ed25519::{SigningKey, VerifyingKey};
use embassy_crypto_rand as _;
use embassy_crypto_rustcrypto as _;
use embassy_futures::block_on;
use embedded_storage::nor_flash::NorFlashErrorKind;

use super::*;
use crate::mem_flash::MemFlash;
use crate::{BlockingDirectXipState, DirectXipState, Slot};

macro_rules! updater_tests {
    ($module:ident, $updater:ident, $state:ident $(, $await:ident)?) => {
        mod $module {
            use super::*;

            #[test]
            fn writes_across_pages_and_keeps_confirmed_update() {
                block_on(async {
                    let mut state_flash = MemFlash::<128, 64, 4>::default();
                    let mut slot_a = MemFlash::<64, 32, 4>::new(0x55);
                    let mut slot_b = MemFlash::<64, 32, 4>::new(0);
                    let mut aligned = [0; 16];
                    {
                        let state = $state::new(&mut state_flash, &mut aligned).unwrap();
                        let mut updater = $updater::new_direct_xip(&mut slot_b, state, Slot::A);
                        updater.write_firmware(0, &[0xa5; 24]) $(.$await)?.unwrap();
                        updater.write_firmware(24, &[0x5a; 32]) $(.$await)?.unwrap();
                        let mut expected = [0xff; 64];
                        expected[..24].fill(0xa5);
                        expected[24..56].fill(0x5a);
                        let mut actual = [0; 64];
                        updater.read_dfu(0, &mut actual) $(.$await)?.unwrap();
                        assert_eq!(actual, expected);
                        let digest = updater.hash::<Sha512>(56, &mut [0; 16]) $(.$await)?.unwrap();
                        assert_eq!(digest.as_ref(), Sha512::digest(&expected[..56]).as_ref());
                        updater.mark_updated() $(.$await)?.unwrap();
                    }
                    {
                        let mut state = $state::new(&mut state_flash, &mut aligned).unwrap();
                        assert_eq!(state.prepare_boot([true, true]) $(.$await)?, Ok(Slot::B));
                        let mut updater = $updater::new_direct_xip(&mut slot_a, state, Slot::B);
                        assert!(matches!(updater.prepare_update() $(.$await)?, Err(FirmwareUpdaterError::BadState)));
                        assert!(!updater.is_booted() $(.$await)?.unwrap());
                        updater.mark_booted() $(.$await)?.unwrap();
                        assert!(updater.is_booted() $(.$await)?.unwrap());
                        updater.prepare_update() $(.$await)?.unwrap();
                    }
                    assert_eq!(slot_a.mem, [0xff; 64]);
                    let mut state = $state::new(&mut state_flash, &mut aligned).unwrap();
                    assert_eq!(state.prepare_boot([true, true]) $(.$await)?, Ok(Slot::B));
                    assert_eq!(state.update_slot() $(.$await)?, Ok(Slot::A));
                });
            }

            #[test]
            fn unconfirmed_update_returns_to_previous_image() {
                block_on(async {
                    let mut state_flash = MemFlash::<128, 64, 4>::default();
                    let mut slot_b = MemFlash::<64, 32, 4>::default();
                    let mut aligned = [0; 16];
                    {
                        let state = $state::new(&mut state_flash, &mut aligned).unwrap();
                        let mut updater = $updater::new_direct_xip(&mut slot_b, state, Slot::A);
                        updater.write_firmware(0, &[0xaa; 32]) $(.$await)?.unwrap();
                        updater.mark_updated() $(.$await)?.unwrap();
                    }
                    let mut state = $state::new(&mut state_flash, &mut aligned).unwrap();
                    assert_eq!(state.prepare_boot([true, true]) $(.$await)?, Ok(Slot::B));
                    assert_eq!(state.prepare_boot([true, true]) $(.$await)?, Ok(Slot::A));
                });
            }

            #[test]
            fn pending_update_or_wrong_running_slot_cannot_modify_images() {
                block_on(async {
                    for (running, pending) in [(Slot::A, true), (Slot::B, false)] {
                        let mut state_flash = MemFlash::<128, 64, 4>::default();
                        let mut image = MemFlash::<64, 32, 4>::new(0x55);
                        let mut aligned = [0; 16];
                        {
                            let mut state = $state::new(&mut state_flash, &mut aligned).unwrap();
                            if pending {
                                state.mark_updated(Slot::B) $(.$await)?.unwrap();
                            }
                            let mut updater = $updater::new_direct_xip(&mut image, state, running);
                            assert!(matches!(updater.write_firmware(0, &[0; 4]) $(.$await)?, Err(FirmwareUpdaterError::BadState)));
                            assert!(matches!(updater.prepare_update() $(.$await)?, Err(FirmwareUpdaterError::BadState)));
                            // Marking the pending slot again is fine; marking the confirmed one isn't.
                            assert_eq!(updater.mark_updated() $(.$await)?.is_ok(), pending);
                        }
                        assert_eq!(image.mem, [0x55; 64], "running={running:?}, pending={pending}");
                    }
                });
            }

            #[test]
            fn signature_must_verify_before_update_is_selected() {
                block_on(async {
                    let key = SigningKey::from_bytes(&[1; 32]);
                    let public_key = key.verifying_key().unwrap().to_bytes();
                    let signature = key.sign(&Sha512::digest(&[0xaa; 32])).unwrap().to_bytes();
                    let mut bad_signature = signature;
                    bad_signature[0] ^= 1;
                    let mut state_flash = MemFlash::<128, 64, 4>::default();
                    let mut slot_b = MemFlash::<64, 32, 4>::default();
                    let mut aligned = [0; 16];
                    {
                        let state = $state::new(&mut state_flash, &mut aligned).unwrap();
                        let mut updater = $updater::new_direct_xip(&mut slot_b, state, Slot::A);
                        updater.write_firmware(0, &[0xaa; 32]) $(.$await)?.unwrap();
                        assert!(matches!(
                            updater.verify_and_mark_updated::<Sha512, VerifyingKey>(
                                &public_key, &bad_signature, &mut [0; 16], 32
                            ) $(.$await)?,
                            Err(FirmwareUpdaterError::Signature(_))
                        ));
                    }
                    let mut state = $state::new(&mut state_flash, &mut aligned).unwrap();
                    assert_eq!(state.update_slot() $(.$await)?, Ok(Slot::B));
                    {
                        let mut updater = $updater::new_direct_xip(&mut slot_b, state, Slot::A);
                        updater.verify_and_mark_updated::<Sha512, VerifyingKey>(
                            &public_key, &signature, &mut [0; 16], 32
                        ) $(.$await)?.unwrap();
                    }
                    let mut state = $state::new(&mut state_flash, &mut aligned).unwrap();
                    assert_eq!(state.prepare_boot([true, true]) $(.$await)?, Ok(Slot::B));
                });
            }

            #[test]
            fn state_read_failure_leaves_image_untouched() {
                block_on(async {
                    let mut state_flash = MemFlash::<128, 64, 4>::default();
                    state_flash.pending_read_successes = Some(0);
                    let mut image = MemFlash::<64, 32, 4>::new(0x55);
                    let mut aligned = [0; 16];
                    {
                        let state = $state::new(&mut state_flash, &mut aligned).unwrap();
                        let mut updater = $updater::new_direct_xip(&mut image, state, Slot::A);
                        assert!(matches!(
                            updater.write_firmware(0, &[0; 4]) $(.$await)?,
                            Err(FirmwareUpdaterError::Flash(NorFlashErrorKind::Other))
                        ));
                    }
                    assert_eq!(image.mem, [0x55; 64]);
                });
            }
        }
    };
}

updater_tests!(asynch, FirmwareUpdater, DirectXipState, await);
updater_tests!(blocking, BlockingFirmwareUpdater, BlockingDirectXipState);
