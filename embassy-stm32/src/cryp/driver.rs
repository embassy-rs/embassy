use embassy_crypto_driver::CryptoError;
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

    fn aes128ecb_init(key: &[u8; 16]) -> Self::Context {
        *key
    }

    fn aes128ecb_clone(ctx: &Self::Context) -> Self::Context {
        *ctx
    }

    fn aes128ecb_encrypt_block(ctx: &Self::Context, blocks: &mut [[u8; 16]]) {
        if blocks.is_empty() {
            return;
        }
        let mut driver = DRIVER.try_lock().unwrap();
        let cryp = driver.borrow();
        let flat = unsafe { core::slice::from_raw_parts_mut(blocks.as_mut_ptr() as *mut u8, blocks.len() * 16) };
        run_in_place(&cryp, &AesEcb::new(ctx), Direction::Encrypt, flat).unwrap();
    }

    fn aes128ecb_decrypt_block(ctx: &Self::Context, blocks: &mut [[u8; 16]]) {
        if blocks.is_empty() {
            return;
        }
        let mut driver = DRIVER.try_lock().unwrap();
        let cryp = driver.borrow();
        let flat = unsafe { core::slice::from_raw_parts_mut(blocks.as_mut_ptr() as *mut u8, blocks.len() * 16) };
        run_in_place(&cryp, &AesEcb::new(ctx), Direction::Decrypt, flat).unwrap();
    }
}

impl embassy_crypto_driver::Aes256Ecb for AesDriver {
    type Context = [u8; 32];

    fn aes256ecb_init(key: &[u8; 32]) -> Self::Context {
        *key
    }

    fn aes256ecb_clone(ctx: &Self::Context) -> Self::Context {
        *ctx
    }

    fn aes256ecb_encrypt_block(ctx: &Self::Context, blocks: &mut [[u8; 16]]) {
        if blocks.is_empty() {
            return;
        }
        let mut driver = DRIVER.try_lock().unwrap();
        let cryp = driver.borrow();
        let flat = unsafe { core::slice::from_raw_parts_mut(blocks.as_mut_ptr() as *mut u8, blocks.len() * 16) };
        run_in_place(&cryp, &AesEcb::new(ctx), Direction::Encrypt, flat).unwrap();
    }

    fn aes256ecb_decrypt_block(ctx: &Self::Context, blocks: &mut [[u8; 16]]) {
        if blocks.is_empty() {
            return;
        }
        let mut driver = DRIVER.try_lock().unwrap();
        let cryp = driver.borrow();
        let flat = unsafe { core::slice::from_raw_parts_mut(blocks.as_mut_ptr() as *mut u8, blocks.len() * 16) };
        run_in_place(&cryp, &AesEcb::new(ctx), Direction::Decrypt, flat).unwrap();
    }
}

#[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
impl embassy_crypto_driver::Aes128Gcm for AesDriver {
    type Context = [u8; 16];

    fn aes128gcm_init(key: &[u8; 16]) -> Self::Context {
        *key
    }

    fn aes128gcm_clone(ctx: &Self::Context) -> Self::Context {
        *ctx
    }

    fn aes128gcm_encrypt(
        ctx: &Self::Context,
        nonce: &[u8],
        aad: &[u8],
        buffer: &mut [u8],
        tag: &mut [u8; 16],
    ) -> Result<(), CryptoError> {
        let mut driver = DRIVER.try_lock().unwrap();
        let cryp = driver.borrow();
        let input = unsafe { core::slice::from_raw_parts(buffer.as_ptr(), buffer.len()) };
        run_gcm128(
            &cryp,
            ctx,
            nonce,
            aad,
            input,
            buffer,
            None,
            Some(tag),
            Direction::Encrypt,
        )
    }

    fn aes128gcm_decrypt(
        ctx: &Self::Context,
        nonce: &[u8],
        aad: &[u8],
        buffer: &mut [u8],
        tag: &[u8; 16],
    ) -> Result<(), CryptoError> {
        let mut driver = DRIVER.try_lock().unwrap();
        let cryp = driver.borrow();
        let input = unsafe { core::slice::from_raw_parts(buffer.as_ptr(), buffer.len()) };
        run_gcm128(
            &cryp,
            ctx,
            nonce,
            aad,
            input,
            buffer,
            Some(tag),
            None,
            Direction::Decrypt,
        )
    }
}

#[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
impl embassy_crypto_driver::Aes256Gcm for AesDriver {
    type Context = [u8; 32];

    fn aes256gcm_init(key: &[u8; 32]) -> Self::Context {
        *key
    }

    fn aes256gcm_clone(ctx: &Self::Context) -> Self::Context {
        *ctx
    }

    fn aes256gcm_encrypt(
        ctx: &Self::Context,
        nonce: &[u8],
        aad: &[u8],
        buffer: &mut [u8],
        tag: &mut [u8; 16],
    ) -> Result<(), CryptoError> {
        let mut driver = DRIVER.try_lock().unwrap();
        let cryp = driver.borrow();
        let input = unsafe { core::slice::from_raw_parts(buffer.as_ptr(), buffer.len()) };
        run_gcm256(
            &cryp,
            ctx,
            nonce,
            aad,
            input,
            buffer,
            None,
            Some(tag),
            Direction::Encrypt,
        )
    }

    fn aes256gcm_decrypt(
        ctx: &Self::Context,
        nonce: &[u8],
        aad: &[u8],
        buffer: &mut [u8],
        tag: &[u8; 16],
    ) -> Result<(), CryptoError> {
        let mut driver = DRIVER.try_lock().unwrap();
        let cryp = driver.borrow();
        let input = unsafe { core::slice::from_raw_parts(buffer.as_ptr(), buffer.len()) };
        run_gcm256(
            &cryp,
            ctx,
            nonce,
            aad,
            input,
            buffer,
            Some(tag),
            None,
            Direction::Decrypt,
        )
    }
}

#[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
impl embassy_crypto_driver::Aes128Ccm for AesDriver {
    type Context = [u8; 16];

    fn aes128ccm_init(key: &[u8; 16]) -> Self::Context {
        *key
    }

    fn aes128ccm_clone(ctx: &Self::Context) -> Self::Context {
        *ctx
    }

    fn aes128ccm_encrypt(
        ctx: &Self::Context,
        nonce: &[u8],
        aad: &[u8],
        buffer: &mut [u8],
        tag: &mut [u8],
    ) -> Result<(), CryptoError> {
        let mut driver = DRIVER.try_lock().unwrap();
        let cryp = driver.borrow();
        let input = unsafe { core::slice::from_raw_parts(buffer.as_ptr(), buffer.len()) };
        cryp_ccm_dispatch!(
            16,
            &cryp,
            ctx,
            nonce,
            aad,
            input,
            buffer,
            tag,
            Direction::Encrypt,
            encrypt
        )
    }

    fn aes128ccm_decrypt(
        ctx: &Self::Context,
        nonce: &[u8],
        aad: &[u8],
        buffer: &mut [u8],
        tag: &[u8],
    ) -> Result<(), CryptoError> {
        let mut driver = DRIVER.try_lock().unwrap();
        let cryp = driver.borrow();
        let input = unsafe { core::slice::from_raw_parts(buffer.as_ptr(), buffer.len()) };
        cryp_ccm_dispatch!(
            16,
            &cryp,
            ctx,
            nonce,
            aad,
            input,
            buffer,
            tag,
            Direction::Decrypt,
            decrypt
        )
    }
}

#[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
impl embassy_crypto_driver::Aes256Ccm for AesDriver {
    type Context = [u8; 32];

    fn aes256ccm_init(key: &[u8; 32]) -> Self::Context {
        *key
    }

    fn aes256ccm_clone(ctx: &Self::Context) -> Self::Context {
        *ctx
    }

    fn aes256ccm_encrypt(
        ctx: &Self::Context,
        nonce: &[u8],
        aad: &[u8],
        buffer: &mut [u8],
        tag: &mut [u8],
    ) -> Result<(), CryptoError> {
        let mut driver = DRIVER.try_lock().unwrap();
        let cryp = driver.borrow();
        let input = unsafe { core::slice::from_raw_parts(buffer.as_ptr(), buffer.len()) };
        cryp_ccm_dispatch!(
            32,
            &cryp,
            ctx,
            nonce,
            aad,
            input,
            buffer,
            tag,
            Direction::Encrypt,
            encrypt
        )
    }

    fn aes256ccm_decrypt(
        ctx: &Self::Context,
        nonce: &[u8],
        aad: &[u8],
        buffer: &mut [u8],
        tag: &[u8],
    ) -> Result<(), CryptoError> {
        let mut driver = DRIVER.try_lock().unwrap();
        let cryp = driver.borrow();
        let input = unsafe { core::slice::from_raw_parts(buffer.as_ptr(), buffer.len()) };
        cryp_ccm_dispatch!(
            32,
            &cryp,
            ctx,
            nonce,
            aad,
            input,
            buffer,
            tag,
            Direction::Decrypt,
            decrypt
        )
    }
}

impl embassy_crypto_driver::Aes128Cbc for AesDriver {
    type Context = ([u8; 16], [u8; 16]);

    fn aes128cbc_init(key: &[u8; 16], iv: &[u8; 16]) -> Self::Context {
        (*key, *iv)
    }

    fn aes128cbc_clone(ctx: &Self::Context) -> Self::Context {
        *ctx
    }

    fn aes128cbc_encrypt_block(ctx: &mut Self::Context, blocks: &mut [[u8; 16]]) {
        if blocks.is_empty() {
            return;
        }
        let mut driver = DRIVER.try_lock().unwrap();
        let cryp = driver.borrow();
        let (key, iv) = ctx;
        let cipher = AesCbc::new(key, iv);
        let flat = unsafe { core::slice::from_raw_parts_mut(blocks.as_mut_ptr() as *mut u8, blocks.len() * 16) };
        run_in_place(&cryp, &cipher, Direction::Encrypt, flat).unwrap();
        iv.copy_from_slice(&blocks[blocks.len() - 1]);
    }

    fn aes128cbc_decrypt_block(ctx: &mut Self::Context, blocks: &mut [[u8; 16]]) {
        if blocks.is_empty() {
            return;
        }
        let last_ciphertext = *blocks.last().unwrap();
        let mut driver = DRIVER.try_lock().unwrap();
        let cryp = driver.borrow();
        let (key, iv) = ctx;
        let cipher = AesCbc::new(key, iv);
        let flat = unsafe { core::slice::from_raw_parts_mut(blocks.as_mut_ptr() as *mut u8, blocks.len() * 16) };
        run_in_place(&cryp, &cipher, Direction::Decrypt, flat).unwrap();
        iv.copy_from_slice(&last_ciphertext);
    }
}

impl embassy_crypto_driver::Aes256Cbc for AesDriver {
    type Context = ([u8; 32], [u8; 16]);

    fn aes256cbc_init(key: &[u8; 32], iv: &[u8; 16]) -> Self::Context {
        (*key, *iv)
    }

    fn aes256cbc_clone(ctx: &Self::Context) -> Self::Context {
        *ctx
    }

    fn aes256cbc_encrypt_block(ctx: &mut Self::Context, blocks: &mut [[u8; 16]]) {
        if blocks.is_empty() {
            return;
        }
        let mut driver = DRIVER.try_lock().unwrap();
        let cryp = driver.borrow();
        let (key, iv) = ctx;
        let cipher = AesCbc::new(key, iv);
        let flat = unsafe { core::slice::from_raw_parts_mut(blocks.as_mut_ptr() as *mut u8, blocks.len() * 16) };
        run_in_place(&cryp, &cipher, Direction::Encrypt, flat).unwrap();
        iv.copy_from_slice(&blocks[blocks.len() - 1]);
    }

    fn aes256cbc_decrypt_block(ctx: &mut Self::Context, blocks: &mut [[u8; 16]]) {
        if blocks.is_empty() {
            return;
        }
        let last_ciphertext = *blocks.last().unwrap();
        let mut driver = DRIVER.try_lock().unwrap();
        let cryp = driver.borrow();
        let (key, iv) = ctx;
        let cipher = AesCbc::new(key, iv);
        let flat = unsafe { core::slice::from_raw_parts_mut(blocks.as_mut_ptr() as *mut u8, blocks.len() * 16) };
        run_in_place(&cryp, &cipher, Direction::Decrypt, flat).unwrap();
        iv.copy_from_slice(&last_ciphertext);
    }
}

embassy_crypto_driver::embassy_crypto_aes128ecb_impl!(AesDriver);
embassy_crypto_driver::embassy_crypto_aes256ecb_impl!(AesDriver);
embassy_crypto_driver::embassy_crypto_aes128cbc_impl!(AesDriver);
embassy_crypto_driver::embassy_crypto_aes256cbc_impl!(AesDriver);
#[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
embassy_crypto_driver::embassy_crypto_aes128gcm_impl!(AesDriver);
#[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
embassy_crypto_driver::embassy_crypto_aes256gcm_impl!(AesDriver);
#[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
embassy_crypto_driver::embassy_crypto_aes128ccm_impl!(AesDriver);
#[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
embassy_crypto_driver::embassy_crypto_aes256ccm_impl!(AesDriver);
