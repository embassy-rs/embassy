use embassy_crypto_driver::CryptoError;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;

use super::{Aes, AesCbc, AesCcm, AesCtr, AesEcb, AesGcm, Direction};
#[cfg(aes_v3b)]
use crate::mode::Blocking;
use crate::suspend::ResumablePeripheral;

foreach_peripheral!(
    (aes, $inst:ident) => {
        #[cfg(aes_v2)]
        type BlockingAes = Aes<'static, crate::peripherals::$inst>;
        #[cfg(aes_v3b)]
        type BlockingAes = Aes<'static, crate::peripherals::$inst, Blocking>;

        static DRIVER: Mutex<CriticalSectionRawMutex, ResumablePeripheral<BlockingAes>> =
            Mutex::new(ResumablePeripheral::new_suspended(unsafe { crate::peripherals::$inst::steal() }));
    };
);

fn map_error(error: super::Error) -> CryptoError {
    match error {
        super::Error::KeyError => CryptoError::InvalidKey,
        super::Error::ConfigError => CryptoError::InvalidInput,
        super::Error::ReadError | super::Error::WriteError => CryptoError::HardwareError,
    }
}

fn run_in_place<'c, C>(
    aes: &mut BlockingAes,
    cipher: &'c C,
    direction: Direction,
    buffer: &mut [u8],
) -> Result<(), CryptoError>
where
    C: super::Cipher<'c> + super::CipherSized + super::IVSized,
{
    let mut context = aes.start(cipher, direction);
    for chunk in buffer.chunks_exact_mut(16) {
        let mut block = [0u8; 16];
        block.copy_from_slice(chunk);
        aes.payload_blocking(&mut context, &block, chunk, true)
            .map_err(map_error)?;
    }
    aes.finish_blocking(context).map(|_| ()).map_err(map_error)
}

fn run_authenticated<'c, C, const TAG_SIZE: usize>(
    aes: &mut BlockingAes,
    cipher: &'c C,
    direction: Direction,
    aad: &[u8],
    input: &[u8],
    output: &mut [u8],
    tag: Option<&[u8; TAG_SIZE]>,
    tag_output: Option<&mut [u8; TAG_SIZE]>,
) -> Result<(), CryptoError>
where
    C: super::Cipher<'c> + super::CipherSized + super::IVSized + super::CipherAuthenticated<TAG_SIZE>,
{
    let mut context = aes.start(cipher, direction);
    aes.aad_blocking(&mut context, aad, true).map_err(map_error)?;
    aes.payload_blocking(&mut context, input, output, true)
        .map_err(map_error)?;
    let result = aes
        .finish_blocking(context)
        .map_err(map_error)?
        .ok_or(CryptoError::HardwareError)?;
    if let Some(tag) = tag {
        let mut difference = 0u8;
        for (actual, expected) in result[..TAG_SIZE].iter().zip(tag.iter()) {
            difference |= actual ^ expected;
        }
        if difference != 0 {
            return Err(CryptoError::InvalidSignature);
        }
    } else if let Some(tag_output) = tag_output {
        tag_output.copy_from_slice(&result[..TAG_SIZE]);
    }
    Ok(())
}

macro_rules! define_gcm_runner {
    ($name:ident, $key_size:expr) => {
        fn $name(
            aes: &mut BlockingAes,
            key: &[u8; $key_size],
            nonce: &[u8],
            aad: &[u8],
            input: &[u8],
            output: &mut [u8],
            tag: Option<&[u8; 16]>,
            tag_output: Option<&mut [u8; 16]>,
            direction: Direction,
        ) -> Result<(), CryptoError> {
            let nonce: &[u8; 12] = nonce.try_into().map_err(|_| CryptoError::InvalidInput)?;
            let cipher = AesGcm::<$key_size>::new(key, nonce);
            run_authenticated(aes, &cipher, direction, aad, input, output, tag, tag_output)
        }
    };
}

define_gcm_runner!(run_gcm128, 16);
define_gcm_runner!(run_gcm256, 32);

macro_rules! run_ccm {
    ($key_size:expr, $tag_size:expr, $aes:expr, $key:expr, $nonce:expr, $aad:expr, $input:expr, $output:expr, $tag:expr, $tag_output:expr, $direction:expr $(,)?) => {{
        match $nonce.len() {
            7 => {
                let nonce: &[u8; 7] = $nonce.try_into().map_err(|_| CryptoError::InvalidInput)?;
                let cipher = AesCcm::<$key_size, 7, $tag_size>::new($key, nonce, $aad.len(), $input.len());
                run_authenticated($aes, &cipher, $direction, $aad, $input, $output, $tag, $tag_output)
            }
            8 => {
                let nonce: &[u8; 8] = $nonce.try_into().map_err(|_| CryptoError::InvalidInput)?;
                let cipher = AesCcm::<$key_size, 8, $tag_size>::new($key, nonce, $aad.len(), $input.len());
                run_authenticated($aes, &cipher, $direction, $aad, $input, $output, $tag, $tag_output)
            }
            9 => {
                let nonce: &[u8; 9] = $nonce.try_into().map_err(|_| CryptoError::InvalidInput)?;
                let cipher = AesCcm::<$key_size, 9, $tag_size>::new($key, nonce, $aad.len(), $input.len());
                run_authenticated($aes, &cipher, $direction, $aad, $input, $output, $tag, $tag_output)
            }
            10 => {
                let nonce: &[u8; 10] = $nonce.try_into().map_err(|_| CryptoError::InvalidInput)?;
                let cipher = AesCcm::<$key_size, 10, $tag_size>::new($key, nonce, $aad.len(), $input.len());
                run_authenticated($aes, &cipher, $direction, $aad, $input, $output, $tag, $tag_output)
            }
            11 => {
                let nonce: &[u8; 11] = $nonce.try_into().map_err(|_| CryptoError::InvalidInput)?;
                let cipher = AesCcm::<$key_size, 11, $tag_size>::new($key, nonce, $aad.len(), $input.len());
                run_authenticated($aes, &cipher, $direction, $aad, $input, $output, $tag, $tag_output)
            }
            12 => {
                let nonce: &[u8; 12] = $nonce.try_into().map_err(|_| CryptoError::InvalidInput)?;
                let cipher = AesCcm::<$key_size, 12, $tag_size>::new($key, nonce, $aad.len(), $input.len());
                run_authenticated($aes, &cipher, $direction, $aad, $input, $output, $tag, $tag_output)
            }
            13 => {
                let nonce: &[u8; 13] = $nonce.try_into().map_err(|_| CryptoError::InvalidInput)?;
                let cipher = AesCcm::<$key_size, 13, $tag_size>::new($key, nonce, $aad.len(), $input.len());
                run_authenticated($aes, &cipher, $direction, $aad, $input, $output, $tag, $tag_output)
            }
            _ => Err(CryptoError::InvalidInput),
        }
    }};
}

macro_rules! aes_ccm_dispatch {
    ($key_size:literal, $aes:expr, $ctx:expr, $nonce:expr, $aad:expr, $input:expr, $output:expr, $tag:expr, $direction:expr, encrypt) => {
        match $tag.len() {
            4 => {
                let tag_out: &mut [u8; 4] = $tag.try_into().map_err(|_| CryptoError::InvalidInput)?;
                run_ccm!(
                    $key_size,
                    4,
                    $aes,
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
                    $aes,
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
                    $aes,
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
                    $aes,
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
                    $aes,
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
                    $aes,
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
                    $aes,
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
            _ => Err(CryptoError::Unsupported),
        }
    };
    ($key_size:literal, $aes:expr, $ctx:expr, $nonce:expr, $aad:expr, $input:expr, $output:expr, $tag:expr, $direction:expr, decrypt) => {
        match $tag.len() {
            4 => {
                let tag_ref: &[u8; 4] = $tag.try_into().map_err(|_| CryptoError::InvalidInput)?;
                run_ccm!(
                    $key_size,
                    4,
                    $aes,
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
                    $aes,
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
                    $aes,
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
                    $aes,
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
                    $aes,
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
                    $aes,
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
                    $aes,
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
            _ => Err(CryptoError::Unsupported),
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
        let aes = &mut driver.borrow();
        let cipher = AesEcb::new(ctx);
        let flat = unsafe { core::slice::from_raw_parts_mut(blocks.as_mut_ptr() as *mut u8, blocks.len() * 16) };
        run_in_place(aes, &cipher, Direction::Encrypt, flat).unwrap();
    }

    fn aes128ecb_decrypt_block(ctx: &Self::Context, blocks: &mut [[u8; 16]]) {
        if blocks.is_empty() {
            return;
        }
        let mut driver = DRIVER.try_lock().unwrap();
        let aes = &mut driver.borrow();
        let cipher = AesEcb::new(ctx);
        let flat = unsafe { core::slice::from_raw_parts_mut(blocks.as_mut_ptr() as *mut u8, blocks.len() * 16) };
        run_in_place(aes, &cipher, Direction::Decrypt, flat).unwrap();
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
        let aes = &mut driver.borrow();
        let cipher = AesEcb::new(ctx);
        let flat = unsafe { core::slice::from_raw_parts_mut(blocks.as_mut_ptr() as *mut u8, blocks.len() * 16) };
        run_in_place(aes, &cipher, Direction::Encrypt, flat).unwrap();
    }

    fn aes256ecb_decrypt_block(ctx: &Self::Context, blocks: &mut [[u8; 16]]) {
        if blocks.is_empty() {
            return;
        }
        let mut driver = DRIVER.try_lock().unwrap();
        let aes = &mut driver.borrow();
        let cipher = AesEcb::new(ctx);
        let flat = unsafe { core::slice::from_raw_parts_mut(blocks.as_mut_ptr() as *mut u8, blocks.len() * 16) };
        run_in_place(aes, &cipher, Direction::Decrypt, flat).unwrap();
    }
}

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
        let aes = &mut driver.borrow();
        let input = unsafe { core::slice::from_raw_parts(buffer.as_ptr(), buffer.len()) };
        run_gcm128(aes, ctx, nonce, aad, input, buffer, None, Some(tag), Direction::Encrypt)
    }

    fn aes128gcm_decrypt(
        ctx: &Self::Context,
        nonce: &[u8],
        aad: &[u8],
        buffer: &mut [u8],
        tag: &[u8; 16],
    ) -> Result<(), CryptoError> {
        let mut driver = DRIVER.try_lock().unwrap();
        let aes = &mut driver.borrow();
        let input = unsafe { core::slice::from_raw_parts(buffer.as_ptr(), buffer.len()) };
        run_gcm128(aes, ctx, nonce, aad, input, buffer, Some(tag), None, Direction::Decrypt)
    }
}

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
        let aes = &mut driver.borrow();
        let input = unsafe { core::slice::from_raw_parts(buffer.as_ptr(), buffer.len()) };
        run_gcm256(aes, ctx, nonce, aad, input, buffer, None, Some(tag), Direction::Encrypt)
    }

    fn aes256gcm_decrypt(
        ctx: &Self::Context,
        nonce: &[u8],
        aad: &[u8],
        buffer: &mut [u8],
        tag: &[u8; 16],
    ) -> Result<(), CryptoError> {
        let mut driver = DRIVER.try_lock().unwrap();
        let aes = &mut driver.borrow();
        let input = unsafe { core::slice::from_raw_parts(buffer.as_ptr(), buffer.len()) };
        run_gcm256(aes, ctx, nonce, aad, input, buffer, Some(tag), None, Direction::Decrypt)
    }
}

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
        let aes = &mut driver.borrow();
        let input = unsafe { core::slice::from_raw_parts(buffer.as_ptr(), buffer.len()) };
        aes_ccm_dispatch!(
            16,
            aes,
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
        let aes = &mut driver.borrow();
        let input = unsafe { core::slice::from_raw_parts(buffer.as_ptr(), buffer.len()) };
        aes_ccm_dispatch!(
            16,
            aes,
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
        let aes = &mut driver.borrow();
        let input = unsafe { core::slice::from_raw_parts(buffer.as_ptr(), buffer.len()) };
        aes_ccm_dispatch!(
            32,
            aes,
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
        let aes = &mut driver.borrow();
        let input = unsafe { core::slice::from_raw_parts(buffer.as_ptr(), buffer.len()) };
        aes_ccm_dispatch!(
            32,
            aes,
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
        let aes = &mut driver.borrow();
        let (key, iv) = ctx;
        let cipher = AesCbc::new(key, iv);
        let flat = unsafe { core::slice::from_raw_parts_mut(blocks.as_mut_ptr() as *mut u8, blocks.len() * 16) };
        run_in_place(aes, &cipher, Direction::Encrypt, flat).unwrap();
        iv.copy_from_slice(&blocks[blocks.len() - 1]);
    }

    fn aes128cbc_decrypt_block(ctx: &mut Self::Context, blocks: &mut [[u8; 16]]) {
        if blocks.is_empty() {
            return;
        }
        let last_ciphertext = *blocks.last().unwrap();
        let mut driver = DRIVER.try_lock().unwrap();
        let aes = &mut driver.borrow();
        let (key, iv) = ctx;
        let cipher = AesCbc::new(key, iv);
        let flat = unsafe { core::slice::from_raw_parts_mut(blocks.as_mut_ptr() as *mut u8, blocks.len() * 16) };
        run_in_place(aes, &cipher, Direction::Decrypt, flat).unwrap();
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
        let aes = &mut driver.borrow();
        let (key, iv) = ctx;
        let cipher = AesCbc::new(key, iv);
        let flat = unsafe { core::slice::from_raw_parts_mut(blocks.as_mut_ptr() as *mut u8, blocks.len() * 16) };
        run_in_place(aes, &cipher, Direction::Encrypt, flat).unwrap();
        iv.copy_from_slice(&blocks[blocks.len() - 1]);
    }

    fn aes256cbc_decrypt_block(ctx: &mut Self::Context, blocks: &mut [[u8; 16]]) {
        if blocks.is_empty() {
            return;
        }
        let last_ciphertext = *blocks.last().unwrap();
        let mut driver = DRIVER.try_lock().unwrap();
        let aes = &mut driver.borrow();
        let (key, iv) = ctx;
        let cipher = AesCbc::new(key, iv);
        let flat = unsafe { core::slice::from_raw_parts_mut(blocks.as_mut_ptr() as *mut u8, blocks.len() * 16) };
        run_in_place(aes, &cipher, Direction::Decrypt, flat).unwrap();
        iv.copy_from_slice(&last_ciphertext);
    }
}

// ===========================================================================
// AES-128/256 CTR (stream cipher)
// ===========================================================================

fn ctr_block_in_place_16(aes: &mut BlockingAes, key: &[u8; 16], iv: &mut [u8; 16], buffer: &mut [u8]) {
    let blocks = buffer.len() / 16;
    if blocks == 0 {
        return;
    }

    let cipher = AesCtr::<16>::new(key, iv);
    let mut context = aes.start(&cipher, Direction::Encrypt);
    for chunk in buffer.chunks_exact_mut(16) {
        let mut tmp = [0u8; 16];
        tmp.copy_from_slice(chunk);
        aes.payload_blocking(&mut context, &tmp, chunk, true).unwrap();
    }
    let mut counter = u128::from_be_bytes(*iv);
    counter = counter.wrapping_add(blocks as u128);
    *iv = counter.to_be_bytes();
}

fn ctr_block_in_place_32(aes: &mut BlockingAes, key: &[u8; 32], iv: &mut [u8; 16], buffer: &mut [u8]) {
    let blocks = buffer.len() / 16;
    if blocks == 0 {
        return;
    }

    let cipher = AesCtr::<32>::new(key, iv);
    let mut context = aes.start(&cipher, Direction::Encrypt);
    for chunk in buffer.chunks_exact_mut(16) {
        let mut block = [0u8; 16];
        block.copy_from_slice(chunk);
        aes.payload_blocking(&mut context, &block, chunk, true).unwrap();
    }
    let mut counter = u128::from_be_bytes(*iv);
    counter = counter.wrapping_add(blocks as u128);
    *iv = counter.to_be_bytes();
}

impl embassy_crypto_driver::Aes128Ctr for AesDriver {
    type Context = ([u8; 16], [u8; 16], [u8; 16], u8);
    // (key, iv/counter, partial_keystream_buffer, partial_len)

    fn aes128ctr_init(key: &[u8; 16], iv: &[u8; 16]) -> Self::Context {
        (*key, *iv, [0; 16], 0)
    }

    fn aes128ctr_clone(ctx: &Self::Context) -> Self::Context {
        *ctx
    }

    fn aes128ctr_apply_keystream(ctx: &mut Self::Context, buf: &mut [u8]) {
        let (key, iv, partial, partial_len) = ctx;
        let mut buf = buf;

        // 1. Consume any buffered partial keystream from a previous call.
        if *partial_len > 0 {
            let n = core::cmp::min(*partial_len as usize, buf.len());
            for i in 0..n {
                buf[i] ^= partial[i];
            }
            partial.copy_within(n..*partial_len as usize, 0);
            *partial_len -= n as u8;
            buf = &mut buf[n..];
        }

        // 2. Process full 16-byte blocks via hardware without copying the whole
        //    buffer into a temporary allocation.
        let full_len = (buf.len() / 16) * 16;
        if full_len > 0 {
            let mut driver = DRIVER.try_lock().unwrap();
            let aes = &mut driver.borrow();
            ctr_block_in_place_16(aes, key, iv, &mut buf[..full_len]);
        }

        // 3. Generate one extra keystream block for any trailing partial data.
        let tail = &mut buf[full_len..];
        if !tail.is_empty() {
            let mut driver = DRIVER.try_lock().unwrap();
            let aes = &mut driver.borrow();
            let cipher = AesCtr::<16>::new(key, iv);
            let mut context = aes.start(&cipher, Direction::Encrypt);
            let mut keystream = [0u8; 16];
            let block = [0u8; 16];
            aes.payload_blocking(&mut context, &block, &mut keystream, true)
                .unwrap();
            for i in 0..tail.len() {
                tail[i] ^= keystream[i];
            }
            let saved = 16 - tail.len();
            partial[..saved].copy_from_slice(&keystream[tail.len()..]);
            *partial_len = saved as u8;
        }
    }

    fn aes128ctr_seek(ctx: &mut Self::Context, block_offset: u64) {
        let (_, iv, _, partial_len) = ctx;
        let mut counter = u128::from_be_bytes(*iv);
        counter = counter.wrapping_add(u128::from(block_offset));
        *iv = counter.to_be_bytes();
        *partial_len = 0;
    }
}

impl embassy_crypto_driver::Aes256Ctr for AesDriver {
    type Context = ([u8; 32], [u8; 16], [u8; 16], u8);

    fn aes256ctr_init(key: &[u8; 32], iv: &[u8; 16]) -> Self::Context {
        (*key, *iv, [0; 16], 0)
    }

    fn aes256ctr_clone(ctx: &Self::Context) -> Self::Context {
        *ctx
    }

    fn aes256ctr_apply_keystream(ctx: &mut Self::Context, buf: &mut [u8]) {
        let (key, iv, partial, partial_len) = ctx;
        let mut buf = buf;

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
            let aes = &mut driver.borrow();
            ctr_block_in_place_32(aes, key, iv, &mut buf[..full_len]);
        }

        let tail = &mut buf[full_len..];
        if !tail.is_empty() {
            let mut driver = DRIVER.try_lock().unwrap();
            let aes = &mut driver.borrow();
            let cipher = AesCtr::<32>::new(key, iv);
            let mut context = aes.start(&cipher, Direction::Encrypt);
            let mut keystream = [0u8; 16];
            let block = [0u8; 16];
            aes.payload_blocking(&mut context, &block, &mut keystream, true)
                .unwrap();
            for i in 0..tail.len() {
                tail[i] ^= keystream[i];
            }
            let saved = 16 - tail.len();
            partial[..saved].copy_from_slice(&keystream[tail.len()..]);
            *partial_len = saved as u8;
        }
    }

    fn aes256ctr_seek(ctx: &mut Self::Context, block_offset: u64) {
        let (_, iv, _, partial_len) = ctx;
        let mut counter = u128::from_be_bytes(*iv);
        counter = counter.wrapping_add(u128::from(block_offset));
        *iv = counter.to_be_bytes();
        *partial_len = 0;
    }
}
embassy_crypto_driver::embassy_crypto_aes128ecb_impl!(AesDriver);
embassy_crypto_driver::embassy_crypto_aes256ecb_impl!(AesDriver);
embassy_crypto_driver::embassy_crypto_aes128cbc_impl!(AesDriver);
embassy_crypto_driver::embassy_crypto_aes256cbc_impl!(AesDriver);
embassy_crypto_driver::embassy_crypto_aes128gcm_impl!(AesDriver);
embassy_crypto_driver::embassy_crypto_aes256gcm_impl!(AesDriver);
embassy_crypto_driver::embassy_crypto_aes128ccm_impl!(AesDriver);
embassy_crypto_driver::embassy_crypto_aes256ccm_impl!(AesDriver);
embassy_crypto_driver::embassy_crypto_aes128ctr_impl!(AesDriver);
embassy_crypto_driver::embassy_crypto_aes256ctr_impl!(AesDriver);
