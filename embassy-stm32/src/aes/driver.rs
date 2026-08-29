use embassy_crypto_driver::CryptoError;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;

use super::{Aes, AesCbc, AesCcm, AesEcb, AesGcm, Direction};
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
    C: super::Cipher<'c> + super::CipherSized + super::IVSized + super::CipherAuthenticated<16>,
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
        // SAFETY: AES-GCM processes each output byte based only on the corresponding input byte,
        // so in-place operation is safe even with overlapping input/output views.
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
        if tag.len() != 16 {
            return Err(CryptoError::Unsupported);
        }
        let mut driver = DRIVER.try_lock().unwrap();
        let aes = &mut driver.borrow();
        let input = unsafe { core::slice::from_raw_parts(buffer.as_ptr(), buffer.len()) };
        let tag_out: &mut [u8; 16] = tag.try_into().map_err(|_| CryptoError::InvalidInput)?;
        run_ccm!(
            16,
            16,
            aes,
            ctx,
            nonce,
            aad,
            input,
            buffer,
            None,
            Some(tag_out),
            Direction::Encrypt
        )
    }

    fn aes128ccm_decrypt(
        ctx: &Self::Context,
        nonce: &[u8],
        aad: &[u8],
        buffer: &mut [u8],
        tag: &[u8],
    ) -> Result<(), CryptoError> {
        if tag.len() != 16 {
            return Err(CryptoError::Unsupported);
        }
        let mut driver = DRIVER.try_lock().unwrap();
        let aes = &mut driver.borrow();
        let input = unsafe { core::slice::from_raw_parts(buffer.as_ptr(), buffer.len()) };
        let tag_ref: &[u8; 16] = tag.try_into().map_err(|_| CryptoError::InvalidInput)?;
        run_ccm!(
            16,
            16,
            aes,
            ctx,
            nonce,
            aad,
            input,
            buffer,
            Some(tag_ref),
            None,
            Direction::Decrypt
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
        if tag.len() != 16 {
            return Err(CryptoError::Unsupported);
        }
        let mut driver = DRIVER.try_lock().unwrap();
        let aes = &mut driver.borrow();
        let input = unsafe { core::slice::from_raw_parts(buffer.as_ptr(), buffer.len()) };
        let tag_out: &mut [u8; 16] = tag.try_into().map_err(|_| CryptoError::InvalidInput)?;
        run_ccm!(
            32,
            16,
            aes,
            ctx,
            nonce,
            aad,
            input,
            buffer,
            None,
            Some(tag_out),
            Direction::Encrypt
        )
    }

    fn aes256ccm_decrypt(
        ctx: &Self::Context,
        nonce: &[u8],
        aad: &[u8],
        buffer: &mut [u8],
        tag: &[u8],
    ) -> Result<(), CryptoError> {
        if tag.len() != 16 {
            return Err(CryptoError::Unsupported);
        }
        let mut driver = DRIVER.try_lock().unwrap();
        let aes = &mut driver.borrow();
        let input = unsafe { core::slice::from_raw_parts(buffer.as_ptr(), buffer.len()) };
        let tag_ref: &[u8; 16] = tag.try_into().map_err(|_| CryptoError::InvalidInput)?;
        run_ccm!(
            32,
            16,
            aes,
            ctx,
            nonce,
            aad,
            input,
            buffer,
            Some(tag_ref),
            None,
            Direction::Decrypt
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

embassy_crypto_driver::embassy_crypto_aes128ecb_impl!(AesDriver);
embassy_crypto_driver::embassy_crypto_aes256ecb_impl!(AesDriver);
embassy_crypto_driver::embassy_crypto_aes128cbc_impl!(AesDriver);
embassy_crypto_driver::embassy_crypto_aes256cbc_impl!(AesDriver);
embassy_crypto_driver::embassy_crypto_aes128gcm_impl!(AesDriver);
embassy_crypto_driver::embassy_crypto_aes256gcm_impl!(AesDriver);
embassy_crypto_driver::embassy_crypto_aes128ccm_impl!(AesDriver);
embassy_crypto_driver::embassy_crypto_aes256ccm_impl!(AesDriver);
