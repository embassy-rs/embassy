//! `embassy-crypto` AES drivers, one per `embassy-crypto-aes*` feature, served by the AES
//! peripheral or, with the `embassy-crypto-saes` feature, by the SAES one.
//!
//! Both peripherals expose the same blocking cipher flow, so the drivers are written once
//! against whichever of the two is selected. Nothing is registered for what the selected
//! peripheral cannot do: the 256-bit key size and the authenticated modes on `aes_v1`, CTR
//! and the authenticated modes on `saes_v1b`.

use embassy_crypto::Error as CryptoError;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::{Mutex, MutexGuard};

#[cfg(not(all(feature = "embassy-crypto-saes", saes_v1b)))]
use super::AesCtr;
use super::{AesCbc, AesEcb, Direction};
#[cfg(not(any(aes_v1, all(feature = "embassy-crypto-saes", saes_v1b))))]
use super::{AesCcm, AesGcm};
#[cfg(any(aes_v3a, aes_v3b))]
use crate::mode::Blocking;
use crate::suspend::ResumablePeripheral;

#[cfg(all(feature = "embassy-crypto-saes", not(saes)))]
compile_error!("the `embassy-crypto-saes` feature needs a chip with a SAES peripheral");

#[cfg(not(feature = "embassy-crypto-saes"))]
foreach_peripheral!(
    (aes, $inst:ident) => {
        #[cfg(any(aes_v1, aes_v2, aes_f7))]
        type BlockingAes = super::Aes<'static, crate::peripherals::$inst>;
        #[cfg(any(aes_v3a, aes_v3b))]
        type BlockingAes = super::Aes<'static, crate::peripherals::$inst, Blocking>;

        static DRIVER: Mutex<CriticalSectionRawMutex, ResumablePeripheral<BlockingAes>> =
            Mutex::new(ResumablePeripheral::new_suspended(unsafe { crate::peripherals::$inst::steal() }));
    };
);

#[cfg(feature = "embassy-crypto-saes")]
foreach_peripheral!(
    (saes, $inst:ident) => {
        type BlockingAes = crate::saes::Saes<'static, crate::peripherals::$inst, Blocking>;

        static DRIVER: Mutex<CriticalSectionRawMutex, ResumablePeripheral<BlockingAes>> =
            Mutex::new(ResumablePeripheral::new_suspended(unsafe { crate::peripherals::$inst::steal() }));
    };
);

/// Takes the peripheral, which is clocked for as long as the guard's borrow lives.
fn lock() -> MutexGuard<'static, CriticalSectionRawMutex, ResumablePeripheral<BlockingAes>> {
    // The SAES fetches random numbers from the RNG whenever it is reset.
    #[cfg(all(feature = "embassy-crypto-saes", feature = "embassy-crypto-rng"))]
    crate::rng::driver::ensure_running();
    DRIVER.try_lock().expect("the AES is in use")
}

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
        let ptr = chunk.as_mut_ptr();
        let len = chunk.len();
        let input = unsafe { core::slice::from_raw_parts(ptr, len) };
        let output = unsafe { core::slice::from_raw_parts_mut(ptr, len) };
        aes.payload_blocking(&mut context, input, output, true)
            .map_err(map_error)?;
    }
    aes.finish_blocking(context).map(|_| ()).map_err(map_error)
}

fn run_separate<'c, C>(
    aes: &mut BlockingAes,
    cipher: &'c C,
    direction: Direction,
    input: &[u8],
    output: &mut [u8],
) -> Result<(), CryptoError>
where
    C: super::Cipher<'c> + super::CipherSized + super::IVSized,
{
    let mut context = aes.start(cipher, direction);
    for (in_chunk, out_chunk) in input.chunks_exact(16).zip(output.chunks_exact_mut(16)) {
        aes.payload_blocking(&mut context, in_chunk, out_chunk, true)
            .map_err(map_error)?;
    }
    aes.finish_blocking(context).map(|_| ()).map_err(map_error)
}

#[cfg(not(any(aes_v1, all(feature = "embassy-crypto-saes", saes_v1b))))]
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

#[cfg(not(any(aes_v1, all(feature = "embassy-crypto-saes", saes_v1b))))]
macro_rules! define_gcm_runner {
    ($name:ident, $key_size:expr) => {
        fn $name(
            aes: &mut BlockingAes,
            key: &[u8; $key_size],
            nonce: &[u8; 12],
            aad: &[u8],
            input: &[u8],
            output: &mut [u8],
            tag: Option<&[u8; 16]>,
            tag_output: Option<&mut [u8; 16]>,
            direction: Direction,
        ) -> Result<(), CryptoError> {
            let cipher = AesGcm::<$key_size>::new(key, nonce);
            run_authenticated(aes, &cipher, direction, aad, input, output, tag, tag_output)
        }
    };
}

#[cfg(not(any(aes_v1, all(feature = "embassy-crypto-saes", saes_v1b))))]
define_gcm_runner!(run_gcm128, 16);
#[cfg(not(any(aes_v1, all(feature = "embassy-crypto-saes", saes_v1b))))]
define_gcm_runner!(run_gcm256, 32);

#[cfg(not(any(aes_v1, all(feature = "embassy-crypto-saes", saes_v1b))))]
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

#[cfg(not(any(aes_v1, all(feature = "embassy-crypto-saes", saes_v1b))))]
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

#[cfg(feature = "embassy-crypto-aes128-ecb")]
impl embassy_crypto::driver::Aes128Ecb for AesDriver {
    type Context = [u8; 16];

    fn init(key: &[u8; 16]) -> Self::Context {
        *key
    }

    fn encrypt_blocks(ctx: &Self::Context, blocks: embassy_crypto::driver::InOutBuf<'_, '_, u8>) {
        if blocks.is_empty() {
            return;
        }
        let mut driver = lock();
        let aes = &mut driver.borrow();
        let cipher = AesEcb::new(ctx);
        let len = blocks.len();
        let (in_ptr, out_ptr) = blocks.into_raw();
        if in_ptr == out_ptr {
            let flat = unsafe { core::slice::from_raw_parts_mut(out_ptr, len) };
            run_in_place(aes, &cipher, Direction::Encrypt, flat).unwrap();
        } else {
            let input = unsafe { core::slice::from_raw_parts(in_ptr, len) };
            let output = unsafe { core::slice::from_raw_parts_mut(out_ptr, len) };
            run_separate(aes, &cipher, Direction::Encrypt, input, output).unwrap();
        }
    }

    fn decrypt_blocks(ctx: &Self::Context, blocks: embassy_crypto::driver::InOutBuf<'_, '_, u8>) {
        if blocks.is_empty() {
            return;
        }
        let mut driver = lock();
        let aes = &mut driver.borrow();
        let cipher = AesEcb::new(ctx);
        let len = blocks.len();
        let (in_ptr, out_ptr) = blocks.into_raw();
        if in_ptr == out_ptr {
            let flat = unsafe { core::slice::from_raw_parts_mut(out_ptr, len) };
            run_in_place(aes, &cipher, Direction::Decrypt, flat).unwrap();
        } else {
            let input = unsafe { core::slice::from_raw_parts(in_ptr, len) };
            let output = unsafe { core::slice::from_raw_parts_mut(out_ptr, len) };
            run_separate(aes, &cipher, Direction::Decrypt, input, output).unwrap();
        }
    }
}

#[cfg(feature = "embassy-crypto-aes256-ecb")]
#[cfg(not(aes_v1))]
impl embassy_crypto::driver::Aes256Ecb for AesDriver {
    type Context = [u8; 32];

    fn init(key: &[u8; 32]) -> Self::Context {
        *key
    }

    fn encrypt_blocks(ctx: &Self::Context, blocks: embassy_crypto::driver::InOutBuf<'_, '_, u8>) {
        if blocks.is_empty() {
            return;
        }
        let mut driver = lock();
        let aes = &mut driver.borrow();
        let cipher = AesEcb::new(ctx);
        let len = blocks.len();
        let (in_ptr, out_ptr) = blocks.into_raw();
        if in_ptr == out_ptr {
            let flat = unsafe { core::slice::from_raw_parts_mut(out_ptr, len) };
            run_in_place(aes, &cipher, Direction::Encrypt, flat).unwrap();
        } else {
            let input = unsafe { core::slice::from_raw_parts(in_ptr, len) };
            let output = unsafe { core::slice::from_raw_parts_mut(out_ptr, len) };
            run_separate(aes, &cipher, Direction::Encrypt, input, output).unwrap();
        }
    }

    fn decrypt_blocks(ctx: &Self::Context, blocks: embassy_crypto::driver::InOutBuf<'_, '_, u8>) {
        if blocks.is_empty() {
            return;
        }
        let mut driver = lock();
        let aes = &mut driver.borrow();
        let cipher = AesEcb::new(ctx);
        let len = blocks.len();
        let (in_ptr, out_ptr) = blocks.into_raw();
        if in_ptr == out_ptr {
            let flat = unsafe { core::slice::from_raw_parts_mut(out_ptr, len) };
            run_in_place(aes, &cipher, Direction::Decrypt, flat).unwrap();
        } else {
            let input = unsafe { core::slice::from_raw_parts(in_ptr, len) };
            let output = unsafe { core::slice::from_raw_parts_mut(out_ptr, len) };
            run_separate(aes, &cipher, Direction::Decrypt, input, output).unwrap();
        }
    }
}

#[cfg(feature = "embassy-crypto-aes128-gcm")]
#[cfg(not(any(aes_v1, all(feature = "embassy-crypto-saes", saes_v1b))))]
impl embassy_crypto::driver::Aes128Gcm for AesDriver {
    type Context = [u8; 16];

    fn init(key: &[u8; 16]) -> Self::Context {
        *key
    }

    fn encrypt(
        ctx: &Self::Context,
        nonce: &[u8; 12],
        aad: &[u8],
        buffer: embassy_crypto::driver::InOutBuf<'_, '_, u8>,
        tag: &mut [u8; 16],
    ) -> Result<(), CryptoError> {
        let mut driver = lock();
        let aes = &mut driver.borrow();
        let len = buffer.len();
        let (in_ptr, out_ptr) = buffer.into_raw();
        let input = unsafe { core::slice::from_raw_parts(in_ptr, len) };
        let output = unsafe { core::slice::from_raw_parts_mut(out_ptr, len) };
        run_gcm128(aes, ctx, nonce, aad, input, output, None, Some(tag), Direction::Encrypt)
    }

    fn decrypt(
        ctx: &Self::Context,
        nonce: &[u8; 12],
        aad: &[u8],
        buffer: embassy_crypto::driver::InOutBuf<'_, '_, u8>,
        tag: &[u8; 16],
    ) -> Result<(), CryptoError> {
        let mut driver = lock();
        let aes = &mut driver.borrow();
        let len = buffer.len();
        let (in_ptr, out_ptr) = buffer.into_raw();
        let input = unsafe { core::slice::from_raw_parts(in_ptr, len) };
        let output = unsafe { core::slice::from_raw_parts_mut(out_ptr, len) };
        run_gcm128(aes, ctx, nonce, aad, input, output, Some(tag), None, Direction::Decrypt)
    }
}

#[cfg(feature = "embassy-crypto-aes256-gcm")]
#[cfg(not(any(aes_v1, all(feature = "embassy-crypto-saes", saes_v1b))))]
impl embassy_crypto::driver::Aes256Gcm for AesDriver {
    type Context = [u8; 32];

    fn init(key: &[u8; 32]) -> Self::Context {
        *key
    }

    fn encrypt(
        ctx: &Self::Context,
        nonce: &[u8; 12],
        aad: &[u8],
        buffer: embassy_crypto::driver::InOutBuf<'_, '_, u8>,
        tag: &mut [u8; 16],
    ) -> Result<(), CryptoError> {
        let mut driver = lock();
        let aes = &mut driver.borrow();
        let len = buffer.len();
        let (in_ptr, out_ptr) = buffer.into_raw();
        let input = unsafe { core::slice::from_raw_parts(in_ptr, len) };
        let output = unsafe { core::slice::from_raw_parts_mut(out_ptr, len) };
        run_gcm256(aes, ctx, nonce, aad, input, output, None, Some(tag), Direction::Encrypt)
    }

    fn decrypt(
        ctx: &Self::Context,
        nonce: &[u8; 12],
        aad: &[u8],
        buffer: embassy_crypto::driver::InOutBuf<'_, '_, u8>,
        tag: &[u8; 16],
    ) -> Result<(), CryptoError> {
        let mut driver = lock();
        let aes = &mut driver.borrow();
        let len = buffer.len();
        let (in_ptr, out_ptr) = buffer.into_raw();
        let input = unsafe { core::slice::from_raw_parts(in_ptr, len) };
        let output = unsafe { core::slice::from_raw_parts_mut(out_ptr, len) };
        run_gcm256(aes, ctx, nonce, aad, input, output, Some(tag), None, Direction::Decrypt)
    }
}

#[cfg(feature = "embassy-crypto-aes128-ccm")]
#[cfg(not(any(aes_v1, all(feature = "embassy-crypto-saes", saes_v1b))))]
impl embassy_crypto::driver::Aes128Ccm for AesDriver {
    type Context = [u8; 16];

    fn init(key: &[u8; 16]) -> Self::Context {
        *key
    }

    fn encrypt(
        ctx: &Self::Context,
        nonce: &[u8],
        aad: &[u8],
        buffer: embassy_crypto::driver::InOutBuf<'_, '_, u8>,
        tag: &mut [u8],
    ) -> Result<(), CryptoError> {
        let mut driver = lock();
        let aes = &mut driver.borrow();
        let len = buffer.len();
        let (in_ptr, out_ptr) = buffer.into_raw();
        let input = unsafe { core::slice::from_raw_parts(in_ptr, len) };
        let output = unsafe { core::slice::from_raw_parts_mut(out_ptr, len) };
        aes_ccm_dispatch!(
            16,
            aes,
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
        buffer: embassy_crypto::driver::InOutBuf<'_, '_, u8>,
        tag: &[u8],
    ) -> Result<(), CryptoError> {
        let mut driver = lock();
        let aes = &mut driver.borrow();
        let len = buffer.len();
        let (in_ptr, out_ptr) = buffer.into_raw();
        let input = unsafe { core::slice::from_raw_parts(in_ptr, len) };
        let output = unsafe { core::slice::from_raw_parts_mut(out_ptr, len) };
        aes_ccm_dispatch!(
            16,
            aes,
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

#[cfg(feature = "embassy-crypto-aes256-ccm")]
#[cfg(not(any(aes_v1, all(feature = "embassy-crypto-saes", saes_v1b))))]
impl embassy_crypto::driver::Aes256Ccm for AesDriver {
    type Context = [u8; 32];

    fn init(key: &[u8; 32]) -> Self::Context {
        *key
    }

    fn encrypt(
        ctx: &Self::Context,
        nonce: &[u8],
        aad: &[u8],
        buffer: embassy_crypto::driver::InOutBuf<'_, '_, u8>,
        tag: &mut [u8],
    ) -> Result<(), CryptoError> {
        let mut driver = lock();
        let aes = &mut driver.borrow();
        let len = buffer.len();
        let (in_ptr, out_ptr) = buffer.into_raw();
        let input = unsafe { core::slice::from_raw_parts(in_ptr, len) };
        let output = unsafe { core::slice::from_raw_parts_mut(out_ptr, len) };
        aes_ccm_dispatch!(
            32,
            aes,
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
        buffer: embassy_crypto::driver::InOutBuf<'_, '_, u8>,
        tag: &[u8],
    ) -> Result<(), CryptoError> {
        let mut driver = lock();
        let aes = &mut driver.borrow();
        let len = buffer.len();
        let (in_ptr, out_ptr) = buffer.into_raw();
        let input = unsafe { core::slice::from_raw_parts(in_ptr, len) };
        let output = unsafe { core::slice::from_raw_parts_mut(out_ptr, len) };
        aes_ccm_dispatch!(
            32,
            aes,
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

#[cfg(feature = "embassy-crypto-aes128-cbc")]
impl embassy_crypto::driver::Aes128Cbc for AesDriver {
    type EncryptContext = ([u8; 16], [u8; 16]);
    type DecryptContext = ([u8; 16], [u8; 16]);

    fn encrypt_init(key: &[u8; 16], iv: &[u8; 16]) -> Self::EncryptContext {
        (*key, *iv)
    }

    fn decrypt_init(key: &[u8; 16], iv: &[u8; 16]) -> Self::DecryptContext {
        (*key, *iv)
    }

    fn encrypt_blocks(ctx: &mut Self::EncryptContext, blocks: embassy_crypto::driver::InOutBuf<'_, '_, u8>) {
        if blocks.is_empty() {
            return;
        }
        let mut driver = lock();
        let aes = &mut driver.borrow();
        let (key, iv) = ctx;
        let cipher = AesCbc::new(key, iv);
        let len = blocks.len();
        let (in_ptr, out_ptr) = blocks.into_raw();
        if in_ptr == out_ptr {
            let flat = unsafe { core::slice::from_raw_parts_mut(out_ptr, len) };
            run_in_place(aes, &cipher, Direction::Encrypt, flat).unwrap();
            let last_block: [u8; 16] = flat[flat.len() - 16..].try_into().unwrap();
            iv.copy_from_slice(&last_block);
        } else {
            let input = unsafe { core::slice::from_raw_parts(in_ptr, len) };
            let output = unsafe { core::slice::from_raw_parts_mut(out_ptr, len) };
            run_separate(aes, &cipher, Direction::Encrypt, input, output).unwrap();
            let last_block: [u8; 16] = output[output.len() - 16..].try_into().unwrap();
            iv.copy_from_slice(&last_block);
        }
    }

    fn decrypt_blocks(ctx: &mut Self::DecryptContext, blocks: embassy_crypto::driver::InOutBuf<'_, '_, u8>) {
        if blocks.is_empty() {
            return;
        }
        let mut driver = lock();
        let aes = &mut driver.borrow();
        let (key, iv) = ctx;
        let cipher = AesCbc::new(key, iv);
        let len = blocks.len();
        let (in_ptr, out_ptr) = blocks.into_raw();
        if in_ptr == out_ptr {
            let flat = unsafe { core::slice::from_raw_parts_mut(out_ptr, len) };
            let last_ciphertext: [u8; 16] = flat[flat.len() - 16..].try_into().unwrap();
            run_in_place(aes, &cipher, Direction::Decrypt, flat).unwrap();
            iv.copy_from_slice(&last_ciphertext);
        } else {
            let input = unsafe { core::slice::from_raw_parts(in_ptr, len) };
            let output = unsafe { core::slice::from_raw_parts_mut(out_ptr, len) };
            let last_ciphertext: [u8; 16] = input[input.len() - 16..].try_into().unwrap();
            run_separate(aes, &cipher, Direction::Decrypt, input, output).unwrap();
            iv.copy_from_slice(&last_ciphertext);
        }
    }
}

#[cfg(feature = "embassy-crypto-aes256-cbc")]
#[cfg(not(aes_v1))]
impl embassy_crypto::driver::Aes256Cbc for AesDriver {
    type EncryptContext = ([u8; 32], [u8; 16]);
    type DecryptContext = ([u8; 32], [u8; 16]);

    fn encrypt_init(key: &[u8; 32], iv: &[u8; 16]) -> Self::EncryptContext {
        (*key, *iv)
    }

    fn decrypt_init(key: &[u8; 32], iv: &[u8; 16]) -> Self::DecryptContext {
        (*key, *iv)
    }

    fn encrypt_blocks(ctx: &mut Self::EncryptContext, blocks: embassy_crypto::driver::InOutBuf<'_, '_, u8>) {
        if blocks.is_empty() {
            return;
        }
        let mut driver = lock();
        let aes = &mut driver.borrow();
        let (key, iv) = ctx;
        let cipher = AesCbc::new(key, iv);
        let len = blocks.len();
        let (in_ptr, out_ptr) = blocks.into_raw();
        if in_ptr == out_ptr {
            let flat = unsafe { core::slice::from_raw_parts_mut(out_ptr, len) };
            run_in_place(aes, &cipher, Direction::Encrypt, flat).unwrap();
            let last_block: [u8; 16] = flat[flat.len() - 16..].try_into().unwrap();
            iv.copy_from_slice(&last_block);
        } else {
            let input = unsafe { core::slice::from_raw_parts(in_ptr, len) };
            let output = unsafe { core::slice::from_raw_parts_mut(out_ptr, len) };
            run_separate(aes, &cipher, Direction::Encrypt, input, output).unwrap();
            let last_block: [u8; 16] = output[output.len() - 16..].try_into().unwrap();
            iv.copy_from_slice(&last_block);
        }
    }

    fn decrypt_blocks(ctx: &mut Self::DecryptContext, blocks: embassy_crypto::driver::InOutBuf<'_, '_, u8>) {
        if blocks.is_empty() {
            return;
        }
        let mut driver = lock();
        let aes = &mut driver.borrow();
        let (key, iv) = ctx;
        let cipher = AesCbc::new(key, iv);
        let len = blocks.len();
        let (in_ptr, out_ptr) = blocks.into_raw();
        if in_ptr == out_ptr {
            let flat = unsafe { core::slice::from_raw_parts_mut(out_ptr, len) };
            let last_ciphertext: [u8; 16] = flat[flat.len() - 16..].try_into().unwrap();
            run_in_place(aes, &cipher, Direction::Decrypt, flat).unwrap();
            iv.copy_from_slice(&last_ciphertext);
        } else {
            let input = unsafe { core::slice::from_raw_parts(in_ptr, len) };
            let output = unsafe { core::slice::from_raw_parts_mut(out_ptr, len) };
            let last_ciphertext: [u8; 16] = input[input.len() - 16..].try_into().unwrap();
            run_separate(aes, &cipher, Direction::Decrypt, input, output).unwrap();
            iv.copy_from_slice(&last_ciphertext);
        }
    }
}

// ===========================================================================
// AES-128/256 CTR (stream cipher)
// ===========================================================================

/// Number of blocks one hardware run may process from the counter block `iv`.
///
/// The peripheral only increments the low 32 bits of the counter block, while
/// CTR mode as exposed by `embassy_crypto` uses a 128-bit big-endian counter
/// (NIST SP 800-38A). A run therefore stops where the low word would wrap, and
/// the carry into the upper bits is applied by software before the next run.
#[cfg(any(feature = "embassy-crypto-aes128-ctr", feature = "embassy-crypto-aes256-ctr"))]
#[cfg(not(all(feature = "embassy-crypto-saes", saes_v1b)))]
fn ctr_run_blocks(iv: &[u8; 16], blocks: usize) -> usize {
    let low = u32::from_be_bytes(iv[12..].try_into().unwrap());
    let until_wrap = u64::from(u32::MAX - low) + 1;
    if until_wrap >= blocks as u64 {
        blocks
    } else {
        until_wrap as usize
    }
}

/// Advance the 128-bit big-endian counter block by `blocks`.
#[cfg(any(feature = "embassy-crypto-aes128-ctr", feature = "embassy-crypto-aes256-ctr"))]
#[cfg(not(all(feature = "embassy-crypto-saes", saes_v1b)))]
fn ctr_advance(iv: &mut [u8; 16], blocks: usize) {
    *iv = u128::from_be_bytes(*iv).wrapping_add(blocks as u128).to_be_bytes();
}

#[cfg(feature = "embassy-crypto-aes128-ctr")]
#[cfg(not(all(feature = "embassy-crypto-saes", saes_v1b)))]
fn ctr_block_in_place_16(aes: &mut BlockingAes, key: &[u8; 16], iv: &mut [u8; 16], buffer: &mut [u8]) {
    let mut buffer = buffer;
    while buffer.len() >= 16 {
        let n = ctr_run_blocks(iv, buffer.len() / 16);
        let (run, rest) = buffer.split_at_mut(n * 16);
        let start = *iv;
        let cipher = AesCtr::<16>::new(key, &start);
        let mut context = aes.start(&cipher, Direction::Encrypt);
        for chunk in run.chunks_exact_mut(16) {
            let ptr = chunk.as_mut_ptr();
            let len = chunk.len();
            let input = unsafe { core::slice::from_raw_parts(ptr, len) };
            let output = unsafe { core::slice::from_raw_parts_mut(ptr, len) };
            aes.payload_blocking(&mut context, input, output, true).unwrap();
        }
        ctr_advance(iv, n);
        buffer = rest;
    }
}

#[cfg(feature = "embassy-crypto-aes128-ctr")]
#[cfg(not(all(feature = "embassy-crypto-saes", saes_v1b)))]
fn ctr_block_separate_16(aes: &mut BlockingAes, key: &[u8; 16], iv: &mut [u8; 16], input: &[u8], output: &mut [u8]) {
    assert_eq!(input.len(), output.len());
    let mut input = input;
    let mut output = output;
    while input.len() >= 16 {
        let n = ctr_run_blocks(iv, input.len() / 16);
        let (in_run, in_rest) = input.split_at(n * 16);
        let (out_run, out_rest) = output.split_at_mut(n * 16);
        let start = *iv;
        let cipher = AesCtr::<16>::new(key, &start);
        let mut context = aes.start(&cipher, Direction::Encrypt);
        for (in_chunk, out_chunk) in in_run.chunks_exact(16).zip(out_run.chunks_exact_mut(16)) {
            aes.payload_blocking(&mut context, in_chunk, out_chunk, true).unwrap();
        }
        ctr_advance(iv, n);
        input = in_rest;
        output = out_rest;
    }
}

#[cfg(feature = "embassy-crypto-aes256-ctr")]
#[cfg(not(all(feature = "embassy-crypto-saes", saes_v1b)))]
#[cfg(not(aes_v1))]
fn ctr_block_in_place_32(aes: &mut BlockingAes, key: &[u8; 32], iv: &mut [u8; 16], buffer: &mut [u8]) {
    let mut buffer = buffer;
    while buffer.len() >= 16 {
        let n = ctr_run_blocks(iv, buffer.len() / 16);
        let (run, rest) = buffer.split_at_mut(n * 16);
        let start = *iv;
        let cipher = AesCtr::<32>::new(key, &start);
        let mut context = aes.start(&cipher, Direction::Encrypt);
        for chunk in run.chunks_exact_mut(16) {
            let ptr = chunk.as_mut_ptr();
            let len = chunk.len();
            let input = unsafe { core::slice::from_raw_parts(ptr, len) };
            let output = unsafe { core::slice::from_raw_parts_mut(ptr, len) };
            aes.payload_blocking(&mut context, input, output, true).unwrap();
        }
        ctr_advance(iv, n);
        buffer = rest;
    }
}

#[cfg(feature = "embassy-crypto-aes256-ctr")]
#[cfg(not(all(feature = "embassy-crypto-saes", saes_v1b)))]
#[cfg(not(aes_v1))]
fn ctr_block_separate_32(aes: &mut BlockingAes, key: &[u8; 32], iv: &mut [u8; 16], input: &[u8], output: &mut [u8]) {
    assert_eq!(input.len(), output.len());
    let mut input = input;
    let mut output = output;
    while input.len() >= 16 {
        let n = ctr_run_blocks(iv, input.len() / 16);
        let (in_run, in_rest) = input.split_at(n * 16);
        let (out_run, out_rest) = output.split_at_mut(n * 16);
        let start = *iv;
        let cipher = AesCtr::<32>::new(key, &start);
        let mut context = aes.start(&cipher, Direction::Encrypt);
        for (in_chunk, out_chunk) in in_run.chunks_exact(16).zip(out_run.chunks_exact_mut(16)) {
            aes.payload_blocking(&mut context, in_chunk, out_chunk, true).unwrap();
        }
        ctr_advance(iv, n);
        input = in_rest;
        output = out_rest;
    }
}

#[cfg(feature = "embassy-crypto-aes128-ctr")]
#[cfg(not(all(feature = "embassy-crypto-saes", saes_v1b)))]
impl embassy_crypto::driver::Aes128Ctr for AesDriver {
    type Context = ([u8; 16], [u8; 16], [u8; 16], u8);
    // (key, iv/counter, partial_keystream_buffer, partial_len)

    fn init(key: &[u8; 16], iv: &[u8; 16]) -> Self::Context {
        (*key, *iv, [0; 16], 0)
    }

    fn apply_keystream(ctx: &mut Self::Context, buf: embassy_crypto::driver::InOutBuf<'_, '_, u8>) {
        let (key, iv, partial, partial_len) = ctx;
        let len = buf.len();
        let (in_ptr, out_ptr) = buf.into_raw();
        if in_ptr == out_ptr {
            let mut buf = unsafe { core::slice::from_raw_parts_mut(out_ptr, len) };

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

            // 2. Process full 16-byte blocks via hardware.
            let full_len = (buf.len() / 16) * 16;
            if full_len > 0 {
                let mut driver = lock();
                let aes = &mut driver.borrow();
                ctr_block_in_place_16(aes, key, iv, &mut buf[..full_len]);
            }

            // 3. Generate one extra keystream block for any trailing partial data.
            let tail = &mut buf[full_len..];
            if !tail.is_empty() {
                let mut driver = lock();
                let aes = &mut driver.borrow();
                let mut keystream = [0u8; 16];
                // Advances the counter, unlike a bare hardware run would.
                ctr_block_in_place_16(aes, key, iv, &mut keystream);
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

            // 2. Process full 16-byte blocks via hardware.
            let full_len = (input.len() / 16) * 16;
            if full_len > 0 {
                let mut driver = lock();
                let aes = &mut driver.borrow();
                ctr_block_separate_16(aes, key, iv, &input[..full_len], &mut output[..full_len]);
            }

            // 3. Generate one extra keystream block for trailing partial data.
            let tail_in = &input[full_len..];
            let tail_out = &mut output[full_len..];
            if !tail_in.is_empty() {
                let mut driver = lock();
                let aes = &mut driver.borrow();
                let mut keystream = [0u8; 16];
                // Advances the counter, unlike a bare hardware run would.
                ctr_block_in_place_16(aes, key, iv, &mut keystream);
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

#[cfg(feature = "embassy-crypto-aes256-ctr")]
#[cfg(not(all(feature = "embassy-crypto-saes", saes_v1b)))]
#[cfg(not(aes_v1))]
impl embassy_crypto::driver::Aes256Ctr for AesDriver {
    type Context = ([u8; 32], [u8; 16], [u8; 16], u8);

    fn init(key: &[u8; 32], iv: &[u8; 16]) -> Self::Context {
        (*key, *iv, [0; 16], 0)
    }

    fn apply_keystream(ctx: &mut Self::Context, buf: embassy_crypto::driver::InOutBuf<'_, '_, u8>) {
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
                let mut driver = lock();
                let aes = &mut driver.borrow();
                ctr_block_in_place_32(aes, key, iv, &mut buf[..full_len]);
            }

            let tail = &mut buf[full_len..];
            if !tail.is_empty() {
                let mut driver = lock();
                let aes = &mut driver.borrow();
                let mut keystream = [0u8; 16];
                // Advances the counter, unlike a bare hardware run would.
                ctr_block_in_place_32(aes, key, iv, &mut keystream);
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
                let mut driver = lock();
                let aes = &mut driver.borrow();
                ctr_block_separate_32(aes, key, iv, &input[..full_len], &mut output[..full_len]);
            }

            let tail_in = &input[full_len..];
            let tail_out = &mut output[full_len..];
            if !tail_in.is_empty() {
                let mut driver = lock();
                let aes = &mut driver.borrow();
                let mut keystream = [0u8; 16];
                // Advances the counter, unlike a bare hardware run would.
                ctr_block_in_place_32(aes, key, iv, &mut keystream);
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
#[cfg(feature = "embassy-crypto-aes128-ecb")]
embassy_crypto::aes128_ecb_impl!(AesDriver);
#[cfg(feature = "embassy-crypto-aes256-ecb")]
#[cfg(not(aes_v1))]
embassy_crypto::aes256_ecb_impl!(AesDriver);
#[cfg(feature = "embassy-crypto-aes128-cbc")]
embassy_crypto::aes128_cbc_impl!(AesDriver);
#[cfg(feature = "embassy-crypto-aes256-cbc")]
#[cfg(not(aes_v1))]
embassy_crypto::aes256_cbc_impl!(AesDriver);
#[cfg(feature = "embassy-crypto-aes128-gcm")]
#[cfg(not(any(aes_v1, all(feature = "embassy-crypto-saes", saes_v1b))))]
embassy_crypto::aes128_gcm_impl!(AesDriver);
#[cfg(feature = "embassy-crypto-aes256-gcm")]
#[cfg(not(any(aes_v1, all(feature = "embassy-crypto-saes", saes_v1b))))]
embassy_crypto::aes256_gcm_impl!(AesDriver);
#[cfg(feature = "embassy-crypto-aes128-ccm")]
#[cfg(not(any(aes_v1, all(feature = "embassy-crypto-saes", saes_v1b))))]
embassy_crypto::aes128_ccm_impl!(AesDriver);
#[cfg(feature = "embassy-crypto-aes256-ccm")]
#[cfg(not(any(aes_v1, all(feature = "embassy-crypto-saes", saes_v1b))))]
embassy_crypto::aes256_ccm_impl!(AesDriver);
#[cfg(feature = "embassy-crypto-aes128-ctr")]
#[cfg(not(all(feature = "embassy-crypto-saes", saes_v1b)))]
embassy_crypto::aes128_ctr_impl!(AesDriver);
#[cfg(feature = "embassy-crypto-aes256-ctr")]
#[cfg(not(all(feature = "embassy-crypto-saes", saes_v1b)))]
#[cfg(not(aes_v1))]
embassy_crypto::aes256_ctr_impl!(AesDriver);
