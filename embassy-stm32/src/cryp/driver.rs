use embassy_crypto_driver::CryptoError;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;

use super::{AesCbc, AesCtr, AesEcb, Blocking, Cipher, CipherSized, Cryp, Direction, IVSized};
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
        let ptr = chunk.as_mut_ptr();
        let len = chunk.len();
        let input = unsafe { core::slice::from_raw_parts(ptr, len) };
        let output = unsafe { core::slice::from_raw_parts_mut(ptr, len) };
        cryp.payload_blocking(&mut context, input, output, false);
    }
    Ok(())
}

fn run_separate<'c, C>(
    cryp: &BlockingCryp,
    cipher: &'c C,
    direction: Direction,
    input: &[u8],
    output: &mut [u8],
) -> Result<(), CryptoError>
where
    C: Cipher<'c> + CipherSized + IVSized,
{
    validate_payload(input, output)?;
    let mut context = cryp.start_blocking(cipher, direction);
    for (in_chunk, out_chunk) in input.chunks_exact(16).zip(output.chunks_exact_mut(16)) {
        cryp.payload_blocking(&mut context, in_chunk, out_chunk, false);
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
    ($key_size:expr, $tag_size:expr, $cryp:expr, $key:expr, $nonce:expr, $aad:expr, $input:expr, $output:expr, $expected:expr, $tag_output:expr, $direction:expr) => {{
        match $nonce.len() {
            7 => {
                let nonce: &[u8; 7] = $nonce.try_into().map_err(|_| CryptoError::InvalidInput)?;
                let cipher = super::AesCcm::<$key_size, $tag_size, 7>::new($key, nonce, $aad.len(), $input.len());
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
                let cipher = super::AesCcm::<$key_size, $tag_size, 8>::new($key, nonce, $aad.len(), $input.len());
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
                let cipher = super::AesCcm::<$key_size, $tag_size, 9>::new($key, nonce, $aad.len(), $input.len());
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
                let cipher = super::AesCcm::<$key_size, $tag_size, 10>::new($key, nonce, $aad.len(), $input.len());
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
                let cipher = super::AesCcm::<$key_size, $tag_size, 11>::new($key, nonce, $aad.len(), $input.len());
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
                let cipher = super::AesCcm::<$key_size, $tag_size, 12>::new($key, nonce, $aad.len(), $input.len());
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
                let cipher = super::AesCcm::<$key_size, $tag_size, 13>::new($key, nonce, $aad.len(), $input.len());
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

/// Dispatch AES-CCM over all supported tag sizes (4, 6, 8, 10, 12, 14, 16 bytes).
#[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
macro_rules! cryp_ccm_dispatch {
    ($key_size:literal, $cryp:expr, $ctx:expr, $nonce:expr, $aad:expr, $input:expr, $output:expr, $tag:expr, $direction:expr, encrypt) => {
        match $tag.len() {
            4 => {
                let tag_out: &mut [u8; 4] = $tag.try_into().map_err(|_| CryptoError::InvalidInput)?;
                run_ccm!(
                    $key_size,
                    4,
                    $cryp,
                    $ctx,
                    $nonce,
                    $aad,
                    $input,
                    $output,
                    None,
                    Some(tag_out),
                    $direction
                )
            }
            6 => {
                let tag_out: &mut [u8; 6] = $tag.try_into().map_err(|_| CryptoError::InvalidInput)?;
                run_ccm!(
                    $key_size,
                    6,
                    $cryp,
                    $ctx,
                    $nonce,
                    $aad,
                    $input,
                    $output,
                    None,
                    Some(tag_out),
                    $direction
                )
            }
            8 => {
                let tag_out: &mut [u8; 8] = $tag.try_into().map_err(|_| CryptoError::InvalidInput)?;
                run_ccm!(
                    $key_size,
                    8,
                    $cryp,
                    $ctx,
                    $nonce,
                    $aad,
                    $input,
                    $output,
                    None,
                    Some(tag_out),
                    $direction
                )
            }
            10 => {
                let tag_out: &mut [u8; 10] = $tag.try_into().map_err(|_| CryptoError::InvalidInput)?;
                run_ccm!(
                    $key_size,
                    10,
                    $cryp,
                    $ctx,
                    $nonce,
                    $aad,
                    $input,
                    $output,
                    None,
                    Some(tag_out),
                    $direction
                )
            }
            12 => {
                let tag_out: &mut [u8; 12] = $tag.try_into().map_err(|_| CryptoError::InvalidInput)?;
                run_ccm!(
                    $key_size,
                    12,
                    $cryp,
                    $ctx,
                    $nonce,
                    $aad,
                    $input,
                    $output,
                    None,
                    Some(tag_out),
                    $direction
                )
            }
            14 => {
                let tag_out: &mut [u8; 14] = $tag.try_into().map_err(|_| CryptoError::InvalidInput)?;
                run_ccm!(
                    $key_size,
                    14,
                    $cryp,
                    $ctx,
                    $nonce,
                    $aad,
                    $input,
                    $output,
                    None,
                    Some(tag_out),
                    $direction
                )
            }
            16 => {
                let tag_out: &mut [u8; 16] = $tag.try_into().map_err(|_| CryptoError::InvalidInput)?;
                run_ccm!(
                    $key_size,
                    16,
                    $cryp,
                    $ctx,
                    $nonce,
                    $aad,
                    $input,
                    $output,
                    None,
                    Some(tag_out),
                    $direction
                )
            }
            _ => Err(CryptoError::InvalidInput),
        }
    };
    ($key_size:literal, $cryp:expr, $ctx:expr, $nonce:expr, $aad:expr, $input:expr, $output:expr, $tag:expr, $direction:expr, decrypt) => {
        match $tag.len() {
            4 => {
                let tag_ref: &[u8; 4] = $tag.try_into().map_err(|_| CryptoError::InvalidInput)?;
                run_ccm!(
                    $key_size,
                    4,
                    $cryp,
                    $ctx,
                    $nonce,
                    $aad,
                    $input,
                    $output,
                    Some(tag_ref),
                    None,
                    $direction
                )
            }
            6 => {
                let tag_ref: &[u8; 6] = $tag.try_into().map_err(|_| CryptoError::InvalidInput)?;
                run_ccm!(
                    $key_size,
                    6,
                    $cryp,
                    $ctx,
                    $nonce,
                    $aad,
                    $input,
                    $output,
                    Some(tag_ref),
                    None,
                    $direction
                )
            }
            8 => {
                let tag_ref: &[u8; 8] = $tag.try_into().map_err(|_| CryptoError::InvalidInput)?;
                run_ccm!(
                    $key_size,
                    8,
                    $cryp,
                    $ctx,
                    $nonce,
                    $aad,
                    $input,
                    $output,
                    Some(tag_ref),
                    None,
                    $direction
                )
            }
            10 => {
                let tag_ref: &[u8; 10] = $tag.try_into().map_err(|_| CryptoError::InvalidInput)?;
                run_ccm!(
                    $key_size,
                    10,
                    $cryp,
                    $ctx,
                    $nonce,
                    $aad,
                    $input,
                    $output,
                    Some(tag_ref),
                    None,
                    $direction
                )
            }
            12 => {
                let tag_ref: &[u8; 12] = $tag.try_into().map_err(|_| CryptoError::InvalidInput)?;
                run_ccm!(
                    $key_size,
                    12,
                    $cryp,
                    $ctx,
                    $nonce,
                    $aad,
                    $input,
                    $output,
                    Some(tag_ref),
                    None,
                    $direction
                )
            }
            14 => {
                let tag_ref: &[u8; 14] = $tag.try_into().map_err(|_| CryptoError::InvalidInput)?;
                run_ccm!(
                    $key_size,
                    14,
                    $cryp,
                    $ctx,
                    $nonce,
                    $aad,
                    $input,
                    $output,
                    Some(tag_ref),
                    None,
                    $direction
                )
            }
            16 => {
                let tag_ref: &[u8; 16] = $tag.try_into().map_err(|_| CryptoError::InvalidInput)?;
                run_ccm!(
                    $key_size,
                    16,
                    $cryp,
                    $ctx,
                    $nonce,
                    $aad,
                    $input,
                    $output,
                    Some(tag_ref),
                    None,
                    $direction
                )
            }
            _ => Err(CryptoError::InvalidInput),
        }
    };
}

struct AesDriver;

impl embassy_crypto_driver::Aes128Ecb for AesDriver {
    type Context = [u8; 16];

    fn init(key: &[u8; 16]) -> Self::Context {
        *key
    }

    fn encrypt_blocks(ctx: &Self::Context, blocks: embassy_crypto_driver::InOutBuf<'_, '_, u8>) {
        if blocks.is_empty() {
            return;
        }
        let mut driver = DRIVER.try_lock().unwrap();
        let cryp = driver.borrow();
        let len = blocks.len();
        let (in_ptr, out_ptr) = blocks.into_raw();
        if in_ptr == out_ptr {
            let flat = unsafe { core::slice::from_raw_parts_mut(out_ptr, len) };
            run_in_place(&cryp, &AesEcb::new(ctx), Direction::Encrypt, flat).unwrap();
        } else {
            let input = unsafe { core::slice::from_raw_parts(in_ptr, len) };
            let output = unsafe { core::slice::from_raw_parts_mut(out_ptr, len) };
            run_separate(&cryp, &AesEcb::new(ctx), Direction::Encrypt, input, output).unwrap();
        }
    }

    fn decrypt_blocks(ctx: &Self::Context, blocks: embassy_crypto_driver::InOutBuf<'_, '_, u8>) {
        if blocks.is_empty() {
            return;
        }
        let mut driver = DRIVER.try_lock().unwrap();
        let cryp = driver.borrow();
        let len = blocks.len();
        let (in_ptr, out_ptr) = blocks.into_raw();
        if in_ptr == out_ptr {
            let flat = unsafe { core::slice::from_raw_parts_mut(out_ptr, len) };
            run_in_place(&cryp, &AesEcb::new(ctx), Direction::Decrypt, flat).unwrap();
        } else {
            let input = unsafe { core::slice::from_raw_parts(in_ptr, len) };
            let output = unsafe { core::slice::from_raw_parts_mut(out_ptr, len) };
            run_separate(&cryp, &AesEcb::new(ctx), Direction::Decrypt, input, output).unwrap();
        }
    }
}

impl embassy_crypto_driver::Aes256Ecb for AesDriver {
    type Context = [u8; 32];

    fn init(key: &[u8; 32]) -> Self::Context {
        *key
    }

    fn encrypt_blocks(ctx: &Self::Context, blocks: embassy_crypto_driver::InOutBuf<'_, '_, u8>) {
        if blocks.is_empty() {
            return;
        }
        let mut driver = DRIVER.try_lock().unwrap();
        let cryp = driver.borrow();
        let len = blocks.len();
        let (in_ptr, out_ptr) = blocks.into_raw();
        if in_ptr == out_ptr {
            let flat = unsafe { core::slice::from_raw_parts_mut(out_ptr, len) };
            run_in_place(&cryp, &AesEcb::new(ctx), Direction::Encrypt, flat).unwrap();
        } else {
            let input = unsafe { core::slice::from_raw_parts(in_ptr, len) };
            let output = unsafe { core::slice::from_raw_parts_mut(out_ptr, len) };
            run_separate(&cryp, &AesEcb::new(ctx), Direction::Encrypt, input, output).unwrap();
        }
    }

    fn decrypt_blocks(ctx: &Self::Context, blocks: embassy_crypto_driver::InOutBuf<'_, '_, u8>) {
        if blocks.is_empty() {
            return;
        }
        let mut driver = DRIVER.try_lock().unwrap();
        let cryp = driver.borrow();
        let len = blocks.len();
        let (in_ptr, out_ptr) = blocks.into_raw();
        if in_ptr == out_ptr {
            let flat = unsafe { core::slice::from_raw_parts_mut(out_ptr, len) };
            run_in_place(&cryp, &AesEcb::new(ctx), Direction::Decrypt, flat).unwrap();
        } else {
            let input = unsafe { core::slice::from_raw_parts(in_ptr, len) };
            let output = unsafe { core::slice::from_raw_parts_mut(out_ptr, len) };
            run_separate(&cryp, &AesEcb::new(ctx), Direction::Decrypt, input, output).unwrap();
        }
    }
}

#[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
impl embassy_crypto_driver::Aes128Gcm for AesDriver {
    type Context = [u8; 16];

    fn init(key: &[u8; 16]) -> Self::Context {
        *key
    }

    fn encrypt(
        ctx: &Self::Context,
        nonce: &[u8],
        aad: &[u8],
        buffer: embassy_crypto_driver::InOutBuf<'_, '_, u8>,
        tag: &mut [u8; 16],
    ) -> Result<(), CryptoError> {
        let mut driver = DRIVER.try_lock().unwrap();
        let cryp = driver.borrow();
        let len = buffer.len();
        let (in_ptr, out_ptr) = buffer.into_raw();
        let input = unsafe { core::slice::from_raw_parts(in_ptr, len) };
        let output = unsafe { core::slice::from_raw_parts_mut(out_ptr, len) };
        run_gcm128(
            &cryp,
            ctx,
            nonce,
            aad,
            input,
            output,
            None,
            Some(tag),
            Direction::Encrypt,
        )
    }

    fn decrypt(
        ctx: &Self::Context,
        nonce: &[u8],
        aad: &[u8],
        buffer: embassy_crypto_driver::InOutBuf<'_, '_, u8>,
        tag: &[u8; 16],
    ) -> Result<(), CryptoError> {
        let mut driver = DRIVER.try_lock().unwrap();
        let cryp = driver.borrow();
        let len = buffer.len();
        let (in_ptr, out_ptr) = buffer.into_raw();
        let input = unsafe { core::slice::from_raw_parts(in_ptr, len) };
        let output = unsafe { core::slice::from_raw_parts_mut(out_ptr, len) };
        run_gcm128(
            &cryp,
            ctx,
            nonce,
            aad,
            input,
            output,
            Some(tag),
            None,
            Direction::Decrypt,
        )
    }
}

#[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
impl embassy_crypto_driver::Aes256Gcm for AesDriver {
    type Context = [u8; 32];

    fn init(key: &[u8; 32]) -> Self::Context {
        *key
    }

    fn encrypt(
        ctx: &Self::Context,
        nonce: &[u8],
        aad: &[u8],
        buffer: embassy_crypto_driver::InOutBuf<'_, '_, u8>,
        tag: &mut [u8; 16],
    ) -> Result<(), CryptoError> {
        let mut driver = DRIVER.try_lock().unwrap();
        let cryp = driver.borrow();
        let len = buffer.len();
        let (in_ptr, out_ptr) = buffer.into_raw();
        let input = unsafe { core::slice::from_raw_parts(in_ptr, len) };
        let output = unsafe { core::slice::from_raw_parts_mut(out_ptr, len) };
        run_gcm256(
            &cryp,
            ctx,
            nonce,
            aad,
            input,
            output,
            None,
            Some(tag),
            Direction::Encrypt,
        )
    }

    fn decrypt(
        ctx: &Self::Context,
        nonce: &[u8],
        aad: &[u8],
        buffer: embassy_crypto_driver::InOutBuf<'_, '_, u8>,
        tag: &[u8; 16],
    ) -> Result<(), CryptoError> {
        let mut driver = DRIVER.try_lock().unwrap();
        let cryp = driver.borrow();
        let len = buffer.len();
        let (in_ptr, out_ptr) = buffer.into_raw();
        let input = unsafe { core::slice::from_raw_parts(in_ptr, len) };
        let output = unsafe { core::slice::from_raw_parts_mut(out_ptr, len) };
        run_gcm256(
            &cryp,
            ctx,
            nonce,
            aad,
            input,
            output,
            Some(tag),
            None,
            Direction::Decrypt,
        )
    }
}

#[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
impl embassy_crypto_driver::Aes128Ccm for AesDriver {
    type Context = [u8; 16];

    fn init(key: &[u8; 16]) -> Self::Context {
        *key
    }

    fn encrypt(
        ctx: &Self::Context,
        nonce: &[u8],
        aad: &[u8],
        buffer: embassy_crypto_driver::InOutBuf<'_, '_, u8>,
        tag: &mut [u8],
    ) -> Result<(), CryptoError> {
        let mut driver = DRIVER.try_lock().unwrap();
        let cryp = driver.borrow();
        let len = buffer.len();
        let (in_ptr, out_ptr) = buffer.into_raw();
        let input = unsafe { core::slice::from_raw_parts(in_ptr, len) };
        let output = unsafe { core::slice::from_raw_parts_mut(out_ptr, len) };
        cryp_ccm_dispatch!(
            16,
            &cryp,
            ctx,
            nonce,
            aad,
            input,
            output,
            tag,
            Direction::Encrypt,
            encrypt
        )
    }

    fn decrypt(
        ctx: &Self::Context,
        nonce: &[u8],
        aad: &[u8],
        buffer: embassy_crypto_driver::InOutBuf<'_, '_, u8>,
        tag: &[u8],
    ) -> Result<(), CryptoError> {
        let mut driver = DRIVER.try_lock().unwrap();
        let cryp = driver.borrow();
        let len = buffer.len();
        let (in_ptr, out_ptr) = buffer.into_raw();
        let input = unsafe { core::slice::from_raw_parts(in_ptr, len) };
        let output = unsafe { core::slice::from_raw_parts_mut(out_ptr, len) };
        cryp_ccm_dispatch!(
            16,
            &cryp,
            ctx,
            nonce,
            aad,
            input,
            output,
            tag,
            Direction::Decrypt,
            decrypt
        )
    }
}

#[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
impl embassy_crypto_driver::Aes256Ccm for AesDriver {
    type Context = [u8; 32];

    fn init(key: &[u8; 32]) -> Self::Context {
        *key
    }

    fn encrypt(
        ctx: &Self::Context,
        nonce: &[u8],
        aad: &[u8],
        buffer: embassy_crypto_driver::InOutBuf<'_, '_, u8>,
        tag: &mut [u8],
    ) -> Result<(), CryptoError> {
        let mut driver = DRIVER.try_lock().unwrap();
        let cryp = driver.borrow();
        let len = buffer.len();
        let (in_ptr, out_ptr) = buffer.into_raw();
        let input = unsafe { core::slice::from_raw_parts(in_ptr, len) };
        let output = unsafe { core::slice::from_raw_parts_mut(out_ptr, len) };
        cryp_ccm_dispatch!(
            32,
            &cryp,
            ctx,
            nonce,
            aad,
            input,
            output,
            tag,
            Direction::Encrypt,
            encrypt
        )
    }

    fn decrypt(
        ctx: &Self::Context,
        nonce: &[u8],
        aad: &[u8],
        buffer: embassy_crypto_driver::InOutBuf<'_, '_, u8>,
        tag: &[u8],
    ) -> Result<(), CryptoError> {
        let mut driver = DRIVER.try_lock().unwrap();
        let cryp = driver.borrow();
        let len = buffer.len();
        let (in_ptr, out_ptr) = buffer.into_raw();
        let input = unsafe { core::slice::from_raw_parts(in_ptr, len) };
        let output = unsafe { core::slice::from_raw_parts_mut(out_ptr, len) };
        cryp_ccm_dispatch!(
            32,
            &cryp,
            ctx,
            nonce,
            aad,
            input,
            output,
            tag,
            Direction::Decrypt,
            decrypt
        )
    }
}

impl embassy_crypto_driver::Aes128Cbc for AesDriver {
    type EncryptContext = ([u8; 16], [u8; 16]);
    type DecryptContext = ([u8; 16], [u8; 16]);

    fn encrypt_init(key: &[u8; 16], iv: &[u8; 16]) -> Self::EncryptContext {
        (*key, *iv)
    }

    fn decrypt_init(key: &[u8; 16], iv: &[u8; 16]) -> Self::DecryptContext {
        (*key, *iv)
    }

    fn encrypt_blocks(ctx: &mut Self::EncryptContext, blocks: embassy_crypto_driver::InOutBuf<'_, '_, u8>) {
        if blocks.is_empty() {
            return;
        }
        let mut driver = DRIVER.try_lock().unwrap();
        let cryp = driver.borrow();
        let (key, iv) = ctx;
        let cipher = AesCbc::new(key, iv);
        let len = blocks.len();
        let (in_ptr, out_ptr) = blocks.into_raw();
        if in_ptr == out_ptr {
            let flat = unsafe { core::slice::from_raw_parts_mut(out_ptr, len) };
            run_in_place(&cryp, &cipher, Direction::Encrypt, flat).unwrap();
            let last_block: [u8; 16] = flat[flat.len() - 16..].try_into().unwrap();
            iv.copy_from_slice(&last_block);
        } else {
            let input = unsafe { core::slice::from_raw_parts(in_ptr, len) };
            let output = unsafe { core::slice::from_raw_parts_mut(out_ptr, len) };
            run_separate(&cryp, &cipher, Direction::Encrypt, input, output).unwrap();
            let last_block: [u8; 16] = output[output.len() - 16..].try_into().unwrap();
            iv.copy_from_slice(&last_block);
        }
    }

    fn decrypt_blocks(ctx: &mut Self::DecryptContext, blocks: embassy_crypto_driver::InOutBuf<'_, '_, u8>) {
        if blocks.is_empty() {
            return;
        }
        let mut driver = DRIVER.try_lock().unwrap();
        let cryp = driver.borrow();
        let (key, iv) = ctx;
        let cipher = AesCbc::new(key, iv);
        let len = blocks.len();
        let (in_ptr, out_ptr) = blocks.into_raw();
        if in_ptr == out_ptr {
            let flat = unsafe { core::slice::from_raw_parts_mut(out_ptr, len) };
            let last_ciphertext: [u8; 16] = flat[flat.len() - 16..].try_into().unwrap();
            run_in_place(&cryp, &cipher, Direction::Decrypt, flat).unwrap();
            iv.copy_from_slice(&last_ciphertext);
        } else {
            let input = unsafe { core::slice::from_raw_parts(in_ptr, len) };
            let output = unsafe { core::slice::from_raw_parts_mut(out_ptr, len) };
            let last_ciphertext: [u8; 16] = input[input.len() - 16..].try_into().unwrap();
            run_separate(&cryp, &cipher, Direction::Decrypt, input, output).unwrap();
            iv.copy_from_slice(&last_ciphertext);
        }
    }
}

impl embassy_crypto_driver::Aes256Cbc for AesDriver {
    type EncryptContext = ([u8; 32], [u8; 16]);
    type DecryptContext = ([u8; 32], [u8; 16]);

    fn encrypt_init(key: &[u8; 32], iv: &[u8; 16]) -> Self::EncryptContext {
        (*key, *iv)
    }

    fn decrypt_init(key: &[u8; 32], iv: &[u8; 16]) -> Self::DecryptContext {
        (*key, *iv)
    }

    fn encrypt_blocks(ctx: &mut Self::EncryptContext, blocks: embassy_crypto_driver::InOutBuf<'_, '_, u8>) {
        if blocks.is_empty() {
            return;
        }
        let mut driver = DRIVER.try_lock().unwrap();
        let cryp = driver.borrow();
        let (key, iv) = ctx;
        let cipher = AesCbc::new(key, iv);
        let len = blocks.len();
        let (in_ptr, out_ptr) = blocks.into_raw();
        if in_ptr == out_ptr {
            let flat = unsafe { core::slice::from_raw_parts_mut(out_ptr, len) };
            run_in_place(&cryp, &cipher, Direction::Encrypt, flat).unwrap();
            let last_block: [u8; 16] = flat[flat.len() - 16..].try_into().unwrap();
            iv.copy_from_slice(&last_block);
        } else {
            let input = unsafe { core::slice::from_raw_parts(in_ptr, len) };
            let output = unsafe { core::slice::from_raw_parts_mut(out_ptr, len) };
            run_separate(&cryp, &cipher, Direction::Encrypt, input, output).unwrap();
            let last_block: [u8; 16] = output[output.len() - 16..].try_into().unwrap();
            iv.copy_from_slice(&last_block);
        }
    }

    fn decrypt_blocks(ctx: &mut Self::DecryptContext, blocks: embassy_crypto_driver::InOutBuf<'_, '_, u8>) {
        if blocks.is_empty() {
            return;
        }
        let mut driver = DRIVER.try_lock().unwrap();
        let cryp = driver.borrow();
        let (key, iv) = ctx;
        let cipher = AesCbc::new(key, iv);
        let len = blocks.len();
        let (in_ptr, out_ptr) = blocks.into_raw();
        if in_ptr == out_ptr {
            let flat = unsafe { core::slice::from_raw_parts_mut(out_ptr, len) };
            let last_ciphertext: [u8; 16] = flat[flat.len() - 16..].try_into().unwrap();
            run_in_place(&cryp, &cipher, Direction::Decrypt, flat).unwrap();
            iv.copy_from_slice(&last_ciphertext);
        } else {
            let input = unsafe { core::slice::from_raw_parts(in_ptr, len) };
            let output = unsafe { core::slice::from_raw_parts_mut(out_ptr, len) };
            let last_ciphertext: [u8; 16] = input[input.len() - 16..].try_into().unwrap();
            run_separate(&cryp, &cipher, Direction::Decrypt, input, output).unwrap();
            iv.copy_from_slice(&last_ciphertext);
        }
    }
}

// ===========================================================================
// AES-128/256 CTR (stream cipher) via CRYP peripheral
// ===========================================================================

/// Run CTR keystream generation over a buffer that is already a multiple of 16 bytes.
/// Uses a temporary 16-byte block for input/output since CRYP's payload_blocking
/// requires distinct input and output buffers.
/// The IV (counter) is updated in software (counter += blocks_processed).
fn run_ctr_cryp_16(cryp: &BlockingCryp, key: &[u8; 16], iv: &mut [u8; 16], buffer: &mut [u8]) {
    let blocks = buffer.len() / 16;
    if blocks == 0 {
        return;
    }

    let cipher = AesCtr::<16>::new(key, iv);
    let mut context = cryp.start_blocking(&cipher, Direction::Encrypt);
    for chunk in buffer.chunks_exact_mut(16) {
        let ptr = chunk.as_mut_ptr();
        let len = chunk.len();
        let input = unsafe { core::slice::from_raw_parts(ptr, len) };
        let output = unsafe { core::slice::from_raw_parts_mut(ptr, len) };
        cryp.payload_blocking(&mut context, input, output, false);
    }

    let mut counter = u128::from_be_bytes(*iv);
    counter = counter.wrapping_add(blocks as u128);
    *iv = counter.to_be_bytes();
}

fn run_ctr_cryp_16_separate(cryp: &BlockingCryp, key: &[u8; 16], iv: &mut [u8; 16], input: &[u8], output: &mut [u8]) {
    assert_eq!(input.len(), output.len());
    let blocks = input.len() / 16;
    if blocks == 0 {
        return;
    }

    let cipher = AesCtr::<16>::new(key, iv);
    let mut context = cryp.start_blocking(&cipher, Direction::Encrypt);
    for (in_chunk, out_chunk) in input.chunks_exact(16).zip(output.chunks_exact_mut(16)) {
        cryp.payload_blocking(&mut context, in_chunk, out_chunk, false);
    }

    let mut counter = u128::from_be_bytes(*iv);
    counter = counter.wrapping_add(blocks as u128);
    *iv = counter.to_be_bytes();
}

fn run_ctr_cryp_32(cryp: &BlockingCryp, key: &[u8; 32], iv: &mut [u8; 16], buffer: &mut [u8]) {
    let blocks = buffer.len() / 16;
    if blocks == 0 {
        return;
    }

    let cipher = AesCtr::<32>::new(key, iv);
    let mut context = cryp.start_blocking(&cipher, Direction::Encrypt);
    for chunk in buffer.chunks_exact_mut(16) {
        let ptr = chunk.as_mut_ptr();
        let len = chunk.len();
        let input = unsafe { core::slice::from_raw_parts(ptr, len) };
        let output = unsafe { core::slice::from_raw_parts_mut(ptr, len) };
        cryp.payload_blocking(&mut context, input, output, false);
    }

    let mut counter = u128::from_be_bytes(*iv);
    counter = counter.wrapping_add(blocks as u128);
    *iv = counter.to_be_bytes();
}

fn run_ctr_cryp_32_separate(cryp: &BlockingCryp, key: &[u8; 32], iv: &mut [u8; 16], input: &[u8], output: &mut [u8]) {
    assert_eq!(input.len(), output.len());
    let blocks = input.len() / 16;
    if blocks == 0 {
        return;
    }

    let cipher = AesCtr::<32>::new(key, iv);
    let mut context = cryp.start_blocking(&cipher, Direction::Encrypt);
    for (in_chunk, out_chunk) in input.chunks_exact(16).zip(output.chunks_exact_mut(16)) {
        cryp.payload_blocking(&mut context, in_chunk, out_chunk, false);
    }

    let mut counter = u128::from_be_bytes(*iv);
    counter = counter.wrapping_add(blocks as u128);
    *iv = counter.to_be_bytes();
}

impl embassy_crypto_driver::Aes128Ctr for AesDriver {
    type Context = ([u8; 16], [u8; 16], [u8; 16], u8);

    fn init(key: &[u8; 16], iv: &[u8; 16]) -> Self::Context {
        (*key, *iv, [0; 16], 0)
    }

    fn apply_keystream(ctx: &mut Self::Context, buf: embassy_crypto_driver::InOutBuf<'_, '_, u8>) {
        let (key, iv, partial, partial_len) = ctx;
        let len = buf.len();
        let (in_ptr, out_ptr) = buf.into_raw();
        if in_ptr == out_ptr {
            let mut buf = unsafe { core::slice::from_raw_parts_mut(out_ptr, len) };

            // 1. Consume buffered partial keystream.
            if *partial_len > 0 {
                let n = core::cmp::min(*partial_len as usize, buf.len());
                for i in 0..n {
                    buf[i] ^= partial[i];
                }
                partial.copy_within(n..*partial_len as usize, 0);
                *partial_len -= n as u8;
                buf = &mut buf[n..];
            }

            // 2. Process full 16-byte blocks.
            let full_len = (buf.len() / 16) * 16;
            if full_len > 0 {
                let mut driver = DRIVER.try_lock().unwrap();
                let cryp = driver.borrow();
                run_ctr_cryp_16(&cryp, key, iv, &mut buf[..full_len]);
            }

            // 3. Generate one extra keystream block for trailing partial data.
            let tail = &mut buf[full_len..];
            if !tail.is_empty() {
                let mut driver = DRIVER.try_lock().unwrap();
                let cryp = driver.borrow();
                let mut keystream = [0u8; 16];
                run_ctr_cryp_16(&cryp, key, iv, &mut keystream);
                for i in 0..tail.len() {
                    tail[i] ^= keystream[i];
                }
                let saved = 16 - tail.len();
                partial[..saved].copy_from_slice(&keystream[tail.len()..]);
                *partial_len = saved as u8;
            }
        } else {
            let mut input = unsafe { core::slice::from_raw_parts(in_ptr, len) };
            let mut output = unsafe { core::slice::from_raw_parts_mut(out_ptr, len) };

            // 1. Consume buffered partial keystream.
            if *partial_len > 0 {
                let n = core::cmp::min(*partial_len as usize, input.len());
                for i in 0..n {
                    output[i] = input[i] ^ partial[i];
                }
                partial.copy_within(n..*partial_len as usize, 0);
                *partial_len -= n as u8;
                input = &input[n..];
                output = &mut output[n..];
            }

            // 2. Process full 16-byte blocks.
            let full_len = (input.len() / 16) * 16;
            if full_len > 0 {
                let mut driver = DRIVER.try_lock().unwrap();
                let cryp = driver.borrow();
                run_ctr_cryp_16_separate(&cryp, key, iv, &input[..full_len], &mut output[..full_len]);
            }

            // 3. Generate one extra keystream block for trailing partial data.
            let tail_in = &input[full_len..];
            let tail_out = &mut output[full_len..];
            if !tail_in.is_empty() {
                let mut driver = DRIVER.try_lock().unwrap();
                let cryp = driver.borrow();
                let mut keystream = [0u8; 16];
                run_ctr_cryp_16(&cryp, key, iv, &mut keystream);
                for i in 0..tail_in.len() {
                    tail_out[i] = tail_in[i] ^ keystream[i];
                }
                let saved = 16 - tail_in.len();
                partial[..saved].copy_from_slice(&keystream[tail_in.len()..]);
                *partial_len = saved as u8;
            }
        }
    }
}

impl embassy_crypto_driver::Aes256Ctr for AesDriver {
    type Context = ([u8; 32], [u8; 16], [u8; 16], u8);

    fn init(key: &[u8; 32], iv: &[u8; 16]) -> Self::Context {
        (*key, *iv, [0; 16], 0)
    }

    fn apply_keystream(ctx: &mut Self::Context, buf: embassy_crypto_driver::InOutBuf<'_, '_, u8>) {
        let (key, iv, partial, partial_len) = ctx;
        let len = buf.len();
        let (in_ptr, out_ptr) = buf.into_raw();
        if in_ptr == out_ptr {
            let mut buf = unsafe { core::slice::from_raw_parts_mut(out_ptr, len) };

            if *partial_len > 0 {
                let n = core::cmp::min(*partial_len as usize, buf.len());
                for i in 0..n {
                    buf[i] ^= partial[i];
                }
                partial.copy_within(n..*partial_len as usize, 0);
                *partial_len -= n as u8;
                buf = &mut buf[n..];
            }

            let full_len = (buf.len() / 16) * 16;
            if full_len > 0 {
                let mut driver = DRIVER.try_lock().unwrap();
                let cryp = driver.borrow();
                run_ctr_cryp_32(&cryp, key, iv, &mut buf[..full_len]);
            }

            let tail = &mut buf[full_len..];
            if !tail.is_empty() {
                let mut driver = DRIVER.try_lock().unwrap();
                let cryp = driver.borrow();
                let mut keystream = [0u8; 16];
                run_ctr_cryp_32(&cryp, key, iv, &mut keystream);
                for i in 0..tail.len() {
                    tail[i] ^= keystream[i];
                }
                let saved = 16 - tail.len();
                partial[..saved].copy_from_slice(&keystream[tail.len()..]);
                *partial_len = saved as u8;
            }
        } else {
            let mut input = unsafe { core::slice::from_raw_parts(in_ptr, len) };
            let mut output = unsafe { core::slice::from_raw_parts_mut(out_ptr, len) };

            if *partial_len > 0 {
                let n = core::cmp::min(*partial_len as usize, input.len());
                for i in 0..n {
                    output[i] = input[i] ^ partial[i];
                }
                partial.copy_within(n..*partial_len as usize, 0);
                *partial_len -= n as u8;
                input = &input[n..];
                output = &mut output[n..];
            }

            let full_len = (input.len() / 16) * 16;
            if full_len > 0 {
                let mut driver = DRIVER.try_lock().unwrap();
                let cryp = driver.borrow();
                run_ctr_cryp_32_separate(&cryp, key, iv, &input[..full_len], &mut output[..full_len]);
            }

            let tail_in = &input[full_len..];
            let tail_out = &mut output[full_len..];
            if !tail_in.is_empty() {
                let mut driver = DRIVER.try_lock().unwrap();
                let cryp = driver.borrow();
                let mut keystream = [0u8; 16];
                run_ctr_cryp_32(&cryp, key, iv, &mut keystream);
                for i in 0..tail_in.len() {
                    tail_out[i] = tail_in[i] ^ keystream[i];
                }
                let saved = 16 - tail_in.len();
                partial[..saved].copy_from_slice(&keystream[tail_in.len()..]);
                *partial_len = saved as u8;
            }
        }
    }
}
embassy_crypto_driver::aes128ecb_impl!(AesDriver);
embassy_crypto_driver::aes256ecb_impl!(AesDriver);
embassy_crypto_driver::aes128cbc_impl!(AesDriver);
embassy_crypto_driver::aes256cbc_impl!(AesDriver);
#[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
embassy_crypto_driver::aes128gcm_impl!(AesDriver);
#[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
embassy_crypto_driver::aes256gcm_impl!(AesDriver);
#[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
embassy_crypto_driver::aes128ccm_impl!(AesDriver);
#[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
embassy_crypto_driver::aes256ccm_impl!(AesDriver);
embassy_crypto_driver::aes128ctr_impl!(AesDriver);
embassy_crypto_driver::aes256ctr_impl!(AesDriver);
