use embassy_crypto_driver::{AesOperation, CryptoError};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;

use super::{AesCbc, AesEcb, Blocking, Cipher, CipherSized, Cryp, Direction, IVSized};
use crate::suspend::ResumablePeripheral;

foreach_peripheral!(
    (cryp, $inst:ident) => {
        type BlockingCryp = Cryp<'static, crate::peripherals::$inst, Blocking>;

        static DRIVER: Mutex<CriticalSectionRawMutex, ResumablePeripheral<BlockingCryp>> =
            Mutex::new(ResumablePeripheral::new_suspended(unsafe { crate::peripherals::$inst::steal() }));
    };
);

fn validate_payload(input: &[u8], output: &[u8]) -> Result<(), CryptoError> {
    if output.len() < input.len() {
        return Err(CryptoError::BufferTooSmall);
    }
    if input.len() % 16 != 0 {
        return Err(CryptoError::InvalidInput);
    }
    Ok(())
}

fn run_in_place<'c, C>(
    cryp: &BlockingCryp,
    cipher: &'c C,
    direction: Direction,
    buffer: &mut [u8],
) -> Result<(), CryptoError>
where
    C: Cipher<'c> + CipherSized + IVSized,
{
    validate_payload(buffer, buffer)?;
    let mut context = cryp.start_blocking(cipher, direction);
    for chunk in buffer.chunks_exact_mut(16) {
        let mut block = [0u8; 16];
        block.copy_from_slice(chunk);
        cryp.payload_blocking(&mut context, &block, chunk, false);
    }
    Ok(())
}

#[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
fn run_authenticated<'c, C, const TAG_SIZE: usize>(
    cryp: &BlockingCryp,
    cipher: &'c C,
    direction: Direction,
    aad: &[u8],
    input: &[u8],
    output: &mut [u8],
    expected_tag: Option<&[u8; TAG_SIZE]>,
    tag_output: Option<&mut [u8; TAG_SIZE]>,
) -> Result<(), CryptoError>
where
    C: Cipher<'c> + CipherSized + IVSized + super::CipherAuthenticated<TAG_SIZE>,
{
    if output.len() < input.len() {
        return Err(CryptoError::BufferTooSmall);
    }
    let mut context = cryp.start_blocking(cipher, direction);
    cryp.aad_blocking(&mut context, aad, true);
    cryp.payload_blocking(&mut context, input, output, true);
    let actual = cryp.finish_blocking(context);

    if let Some(expected_tag) = expected_tag {
        let mut difference = 0u8;
        for (actual, expected) in actual.iter().zip(expected_tag.iter()) {
            difference |= actual ^ expected;
        }
        if difference != 0 {
            return Err(CryptoError::InvalidSignature);
        }
    } else if let Some(tag_output) = tag_output {
        tag_output.copy_from_slice(&actual);
    }
    Ok(())
}

#[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
macro_rules! define_gcm_runner {
    ($name:ident, $key_size:expr) => {
        fn $name(
            cryp: &BlockingCryp,
            key: &[u8; $key_size],
            nonce: &[u8],
            aad: &[u8],
            input: &[u8],
            output: &mut [u8],
            expected_tag: Option<&[u8; 16]>,
            tag_output: Option<&mut [u8; 16]>,
            direction: Direction,
        ) -> Result<(), CryptoError> {
            let nonce: &[u8; 12] = nonce.try_into().map_err(|_| CryptoError::InvalidInput)?;
            let cipher = super::AesGcm::<$key_size>::new(key, nonce);
            run_authenticated(
                cryp,
                &cipher,
                direction,
                aad,
                input,
                output,
                expected_tag,
                tag_output,
            )
        }
    };
}

#[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
define_gcm_runner!(run_gcm128, 16);
#[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
define_gcm_runner!(run_gcm256, 32);

#[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
macro_rules! run_ccm {
    ($tag_size:expr, $cryp:expr, $key:expr, $nonce:expr, $aad:expr, $input:expr, $output:expr, $expected:expr, $tag_output:expr, $direction:expr) => {{
        match $nonce.len() {
            7 => {
                let nonce: &[u8; 7] = $nonce.try_into().map_err(|_| CryptoError::InvalidInput)?;
                let cipher = super::AesCcm::<16, $tag_size, 7>::new($key, nonce, $aad.len(), $input.len());
                run_authenticated(
                    $cryp,
                    &cipher,
                    $direction,
                    $aad,
                    $input,
                    $output,
                    $expected,
                    $tag_output,
                )
            }
            8 => {
                let nonce: &[u8; 8] = $nonce.try_into().map_err(|_| CryptoError::InvalidInput)?;
                let cipher = super::AesCcm::<16, $tag_size, 8>::new($key, nonce, $aad.len(), $input.len());
                run_authenticated(
                    $cryp,
                    &cipher,
                    $direction,
                    $aad,
                    $input,
                    $output,
                    $expected,
                    $tag_output,
                )
            }
            9 => {
                let nonce: &[u8; 9] = $nonce.try_into().map_err(|_| CryptoError::InvalidInput)?;
                let cipher = super::AesCcm::<16, $tag_size, 9>::new($key, nonce, $aad.len(), $input.len());
                run_authenticated(
                    $cryp,
                    &cipher,
                    $direction,
                    $aad,
                    $input,
                    $output,
                    $expected,
                    $tag_output,
                )
            }
            10 => {
                let nonce: &[u8; 10] = $nonce.try_into().map_err(|_| CryptoError::InvalidInput)?;
                let cipher = super::AesCcm::<16, $tag_size, 10>::new($key, nonce, $aad.len(), $input.len());
                run_authenticated(
                    $cryp,
                    &cipher,
                    $direction,
                    $aad,
                    $input,
                    $output,
                    $expected,
                    $tag_output,
                )
            }
            11 => {
                let nonce: &[u8; 11] = $nonce.try_into().map_err(|_| CryptoError::InvalidInput)?;
                let cipher = super::AesCcm::<16, $tag_size, 11>::new($key, nonce, $aad.len(), $input.len());
                run_authenticated(
                    $cryp,
                    &cipher,
                    $direction,
                    $aad,
                    $input,
                    $output,
                    $expected,
                    $tag_output,
                )
            }
            12 => {
                let nonce: &[u8; 12] = $nonce.try_into().map_err(|_| CryptoError::InvalidInput)?;
                let cipher = super::AesCcm::<16, $tag_size, 12>::new($key, nonce, $aad.len(), $input.len());
                run_authenticated(
                    $cryp,
                    &cipher,
                    $direction,
                    $aad,
                    $input,
                    $output,
                    $expected,
                    $tag_output,
                )
            }
            13 => {
                let nonce: &[u8; 13] = $nonce.try_into().map_err(|_| CryptoError::InvalidInput)?;
                let cipher = super::AesCcm::<16, $tag_size, 13>::new($key, nonce, $aad.len(), $input.len());
                run_authenticated(
                    $cryp,
                    &cipher,
                    $direction,
                    $aad,
                    $input,
                    $output,
                    $expected,
                    $tag_output,
                )
            }
            _ => Err(CryptoError::InvalidInput),
        }
    }};
}

struct AesDriver;

impl embassy_crypto_driver::Aes for AesDriver {
    fn aes_exec(op: AesOperation<'_>) -> Result<(), CryptoError> {
        let mut driver = DRIVER.try_lock().unwrap();
        let cryp = driver.borrow();

        match op {
            AesOperation::Aes128EcbEncrypt { block, key } => {
                run_in_place(&cryp, &AesEcb::new(key), Direction::Encrypt, block)
            }
            AesOperation::Aes128EcbDecrypt { block, key } => {
                run_in_place(&cryp, &AesEcb::new(key), Direction::Decrypt, block)
            }
            AesOperation::Aes128CbcEncrypt { iv, buffer, key } => {
                run_in_place(&cryp, &AesCbc::new(key, iv), Direction::Encrypt, buffer)
            }
            AesOperation::Aes128CbcDecrypt { iv, block, key } => {
                run_in_place(&cryp, &AesCbc::new(key, iv), Direction::Decrypt, block)
            }
            AesOperation::Aes256CbcEncrypt { iv, block, key } => {
                run_in_place(&cryp, &AesCbc::new(key, iv), Direction::Encrypt, block)
            }
            AesOperation::Aes256CbcDecrypt { iv, block, key } => {
                run_in_place(&cryp, &AesCbc::new(key, iv), Direction::Decrypt, block)
            }
            #[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
            AesOperation::AesGcm128Encrypt {
                key,
                nonce,
                aad,
                plaintext,
                ciphertext,
                tag,
            } => run_gcm128(
                &cryp,
                key,
                nonce,
                aad,
                plaintext,
                ciphertext,
                None,
                Some(tag),
                Direction::Encrypt,
            ),
            #[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
            AesOperation::AesGcm128Decrypt {
                key,
                nonce,
                aad,
                ciphertext,
                plaintext,
                tag,
            } => run_gcm128(
                &cryp,
                key,
                nonce,
                aad,
                ciphertext,
                plaintext,
                Some(tag),
                None,
                Direction::Decrypt,
            ),
            #[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
            AesOperation::AesGcm256Encrypt {
                key,
                nonce,
                aad,
                plaintext,
                ciphertext,
                tag,
            } => run_gcm256(
                &cryp,
                key,
                nonce,
                aad,
                plaintext,
                ciphertext,
                None,
                Some(tag),
                Direction::Encrypt,
            ),
            #[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
            AesOperation::AesGcm256Decrypt {
                key,
                nonce,
                aad,
                ciphertext,
                plaintext,
                tag,
            } => run_gcm256(
                &cryp,
                key,
                nonce,
                aad,
                ciphertext,
                plaintext,
                Some(tag),
                None,
                Direction::Decrypt,
            ),
            #[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
            AesOperation::AesCcm128Encrypt {
                key,
                nonce,
                aad,
                plaintext,
                ciphertext,
                tag,
            } => {
                run_ccm!(
                    16,
                    &cryp,
                    key,
                    nonce,
                    aad,
                    plaintext,
                    ciphertext,
                    None,
                    Some(tag),
                    Direction::Encrypt
                )
            }
            #[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
            AesOperation::AesCcm128Decrypt {
                key,
                nonce,
                aad,
                ciphertext,
                plaintext,
                tag,
            } => {
                run_ccm!(
                    16,
                    &cryp,
                    key,
                    nonce,
                    aad,
                    ciphertext,
                    plaintext,
                    Some(tag),
                    None,
                    Direction::Decrypt
                )
            }
            #[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
            AesOperation::AesCcm8_128Encrypt {
                key,
                nonce,
                aad,
                plaintext,
                ciphertext,
                tag,
            } => {
                run_ccm!(
                    8,
                    &cryp,
                    key,
                    nonce,
                    aad,
                    plaintext,
                    ciphertext,
                    None,
                    Some(tag),
                    Direction::Encrypt
                )
            }
            #[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
            AesOperation::AesCcm8_128Decrypt {
                key,
                nonce,
                aad,
                ciphertext,
                plaintext,
                tag,
            } => {
                run_ccm!(
                    8,
                    &cryp,
                    key,
                    nonce,
                    aad,
                    ciphertext,
                    plaintext,
                    Some(tag),
                    None,
                    Direction::Decrypt
                )
            }
            #[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
            AesOperation::AesCcm4_128Encrypt {
                key,
                nonce,
                aad,
                plaintext,
                ciphertext,
                tag,
            } => {
                run_ccm!(
                    4,
                    &cryp,
                    key,
                    nonce,
                    aad,
                    plaintext,
                    ciphertext,
                    None,
                    Some(tag),
                    Direction::Encrypt
                )
            }
            #[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
            AesOperation::AesCcm4_128Decrypt {
                key,
                nonce,
                aad,
                ciphertext,
                plaintext,
                tag,
            } => {
                run_ccm!(
                    4,
                    &cryp,
                    key,
                    nonce,
                    aad,
                    ciphertext,
                    plaintext,
                    Some(tag),
                    None,
                    Direction::Decrypt
                )
            }
            AesOperation::Aes128Cmac { .. } => Err(CryptoError::Unsupported),
            #[cfg(not(any(cryp_v2, cryp_v3, cryp_v4)))]
            AesOperation::AesGcm128Encrypt { .. }
            | AesOperation::AesGcm128Decrypt { .. }
            | AesOperation::AesGcm256Encrypt { .. }
            | AesOperation::AesGcm256Decrypt { .. }
            | AesOperation::AesCcm128Encrypt { .. }
            | AesOperation::AesCcm128Decrypt { .. }
            | AesOperation::AesCcm8_128Encrypt { .. }
            | AesOperation::AesCcm8_128Decrypt { .. }
            | AesOperation::AesCcm4_128Encrypt { .. }
            | AesOperation::AesCcm4_128Decrypt { .. } => Err(CryptoError::Unsupported),
            _ => Err(CryptoError::Unsupported),
        }
    }
}

embassy_crypto_driver::embassy_crypto_aes_impl!(AesDriver);
