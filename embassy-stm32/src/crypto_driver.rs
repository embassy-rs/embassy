//! `embassy-crypto` AES drivers, one per `embassy-crypto-aes*` feature, served
//! by the AES peripheral or, with the `embassy-crypto-saes` feature, by the
//! SAES one, or by the CRYP one.
//!
//! All three peripherals expose the same blocking cipher flow through
//! [`crate::crypto::BlockingCipherOps`], so the drivers are written once
//! against whichever of them is selected. Nothing is registered for what the
//! selected peripheral cannot do: the 256-bit key size on `aes_v1`, the
//! authenticated modes on `aes_v1` and `cryp_v1`, CTR and the authenticated
//! modes on `saes_v1b`.

use embassy_crypto::Error as CryptoError;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::{Mutex, MutexGuard};

#[cfg(any(
    feature = "embassy-crypto-aes128-cbc",
    all(feature = "embassy-crypto-aes256-cbc", not(aes_v1))
))]
use crate::crypto::AesCbc;
#[cfg(all(
    not(all(feature = "embassy-crypto-saes", saes_v1b)),
    any(
        feature = "embassy-crypto-aes128-ctr",
        all(feature = "embassy-crypto-aes256-ctr", not(aes_v1))
    )
))]
use crate::crypto::AesCtr;
#[cfg(any(
    feature = "embassy-crypto-aes128-ecb",
    all(feature = "embassy-crypto-aes256-ecb", not(aes_v1))
))]
use crate::crypto::AesEcb;
#[cfg(all(
    not(any(cryp_v1, aes_v1, all(feature = "embassy-crypto-saes", saes_v1b))),
    any(feature = "embassy-crypto-aes128-gcm", feature = "embassy-crypto-aes256-gcm")
))]
use crate::crypto::AesGcm;
#[cfg(all(
    not(any(cryp_v1, aes_v1, all(feature = "embassy-crypto-saes", saes_v1b))),
    any(
        feature = "embassy-crypto-aes128-gcm",
        feature = "embassy-crypto-aes256-gcm",
        feature = "embassy-crypto-aes128-ccm",
        feature = "embassy-crypto-aes256-ccm"
    )
))]
use crate::crypto::CcmOp;
#[cfg(all(
    not(any(cryp_v1, aes_v1, all(feature = "embassy-crypto-saes", saes_v1b))),
    any(
        feature = "embassy-crypto-aes128-gcm",
        feature = "embassy-crypto-aes256-gcm",
        feature = "embassy-crypto-aes128-ccm",
        feature = "embassy-crypto-aes256-ccm"
    )
))]
use crate::crypto::CipherAuthenticated;
use crate::crypto::{BlockingCipherOps, Cipher, CipherSized, Direction, IVSized};
#[cfg(any(cryp, aes_v3a, aes_v3b, feature = "embassy-crypto-saes"))]
use crate::mode::Blocking;
use crate::suspend::ResumablePeripheral;

#[cfg(all(feature = "embassy-crypto-saes", not(saes)))]
compile_error!("the `embassy-crypto-saes` feature needs a chip with a SAES peripheral");

// Exactly one backend is compiled per chip: CRYP where the chip has it, else
// the AES peripheral, or the SAES one when `embassy-crypto-saes` is selected.
#[cfg(cryp)]
foreach_peripheral!(
    (cryp, $inst:ident) => {
        type Hw = crate::cryp::Cryp<'static, crate::peripherals::$inst, Blocking>;

        static DRIVER: Mutex<CriticalSectionRawMutex, ResumablePeripheral<Hw>> =
            Mutex::new(ResumablePeripheral::new_suspended(unsafe { crate::peripherals::$inst::steal() }));
    };
);

#[cfg(all(not(cryp), not(feature = "embassy-crypto-saes")))]
foreach_peripheral!(
    (aes, $inst:ident) => {
        #[cfg(any(aes_v1, aes_v2, aes_f7))]
        type Hw = crate::aes::Aes<'static, crate::peripherals::$inst>;
        #[cfg(any(aes_v3a, aes_v3b))]
        type Hw = crate::aes::Aes<'static, crate::peripherals::$inst, Blocking>;

        static DRIVER: Mutex<CriticalSectionRawMutex, ResumablePeripheral<Hw>> =
            Mutex::new(ResumablePeripheral::new_suspended(unsafe { crate::peripherals::$inst::steal() }));
    };
);

#[cfg(all(not(cryp), feature = "embassy-crypto-saes"))]
foreach_peripheral!(
    (saes, $inst:ident) => {
        type Hw = crate::saes::Saes<'static, crate::peripherals::$inst, Blocking>;

        static DRIVER: Mutex<CriticalSectionRawMutex, ResumablePeripheral<Hw>> =
            Mutex::new(ResumablePeripheral::new_suspended(unsafe { crate::peripherals::$inst::steal() }));
    };
);

/// Takes the peripheral, which is clocked for as long as the guard's borrow lives.
fn lock() -> MutexGuard<'static, CriticalSectionRawMutex, ResumablePeripheral<Hw>> {
    // The SAES fetches random numbers from the RNG whenever it is reset.
    #[cfg(all(feature = "embassy-crypto-saes", feature = "embassy-crypto-rng"))]
    crate::rng::driver::ensure_running();
    DRIVER.try_lock().expect("the crypto peripheral is in use")
}

fn map_error(error: crate::crypto::Error) -> CryptoError {
    match error {
        crate::crypto::Error::KeyError => CryptoError::InvalidKey,
        crate::crypto::Error::ConfigError => CryptoError::InvalidInput,
        crate::crypto::Error::ReadError | crate::crypto::Error::WriteError => CryptoError::HardwareError,
    }
}

fn run_in_place<'c, H, C>(hw: &mut H, cipher: &'c C, direction: Direction, buffer: &mut [u8]) -> Result<(), CryptoError>
where
    H: BlockingCipherOps<'c, C>,
    C: Cipher<'c> + CipherSized + IVSized,
{
    if buffer.len() % 16 != 0 {
        return Err(CryptoError::InvalidInput);
    }
    let mut context = hw.start(cipher, direction).map_err(map_error)?;
    let total = buffer.len();
    let mut processed = 0;
    for chunk in buffer.chunks_exact_mut(16) {
        processed += 16;
        let ptr = chunk.as_mut_ptr();
        let len = chunk.len();
        let input = unsafe { core::slice::from_raw_parts(ptr, len) };
        let output = unsafe { core::slice::from_raw_parts_mut(ptr, len) };
        hw.payload(&mut context, input, output, processed == total)
            .map_err(map_error)?;
    }
    hw.finish(context).map(|_| ()).map_err(map_error)
}

fn run_separate<'c, H, C>(
    hw: &mut H,
    cipher: &'c C,
    direction: Direction,
    input: &[u8],
    output: &mut [u8],
) -> Result<(), CryptoError>
where
    H: BlockingCipherOps<'c, C>,
    C: Cipher<'c> + CipherSized + IVSized,
{
    if output.len() < input.len() {
        return Err(CryptoError::BufferTooSmall);
    }
    if input.len() % 16 != 0 {
        return Err(CryptoError::InvalidInput);
    }
    let mut context = hw.start(cipher, direction).map_err(map_error)?;
    let total = input.len();
    let mut processed = 0;
    for (in_chunk, out_chunk) in input.chunks_exact(16).zip(output.chunks_exact_mut(16)) {
        processed += 16;
        hw.payload(&mut context, in_chunk, out_chunk, processed == total)
            .map_err(map_error)?;
    }
    hw.finish(context).map(|_| ()).map_err(map_error)
}

#[cfg(not(any(cryp_v1, aes_v1, all(feature = "embassy-crypto-saes", saes_v1b))))]
fn run_authenticated<'c, H, C>(
    hw: &mut H,
    cipher: &'c C,
    direction: Direction,
    aad: &[u8],
    input: &[u8],
    output: &mut [u8],
    tag: Option<&[u8]>,
    tag_output: Option<&mut [u8]>,
) -> Result<(), CryptoError>
where
    H: BlockingCipherOps<'c, C>,
    C: Cipher<'c> + CipherSized + IVSized + CipherAuthenticated<16>,
{
    if output.len() < input.len() {
        return Err(CryptoError::BufferTooSmall);
    }
    let mut context = hw.start(cipher, direction).map_err(map_error)?;
    // With no associated data there is no header phase at all: entering it
    // would feed the core an all-zero header block, which CCM authenticates.
    if !aad.is_empty() {
        hw.aad(&mut context, aad, true).map_err(map_error)?;
    }
    hw.payload(&mut context, input, output, true).map_err(map_error)?;
    let result = hw
        .finish(context)
        .map_err(map_error)?
        .ok_or(CryptoError::HardwareError)?;
    if let Some(tag) = tag {
        if tag.len() > result.len() {
            return Err(CryptoError::InvalidInput);
        }
        let mut difference = 0u8;
        for i in 0..tag.len() {
            difference |= result[i] ^ tag[i];
        }
        if difference != 0 {
            return Err(CryptoError::InvalidSignature);
        }
    } else if let Some(tag_output) = tag_output {
        if tag_output.len() > result.len() {
            return Err(CryptoError::InvalidInput);
        }
        tag_output.copy_from_slice(&result[..tag_output.len()]);
    }
    Ok(())
}

#[cfg(any(feature = "embassy-crypto-aes128-gcm", feature = "embassy-crypto-aes256-gcm"))]
#[cfg(not(any(cryp_v1, aes_v1, all(feature = "embassy-crypto-saes", saes_v1b))))]
macro_rules! define_gcm_runner {
    ($name:ident, $key_size:expr) => {
        fn $name<H>(
            hw: &mut H,
            key: &[u8; $key_size],
            nonce: &[u8; 12],
            aad: &[u8],
            input: &[u8],
            output: &mut [u8],
            tag: Option<&[u8]>,
            tag_output: Option<&mut [u8]>,
            direction: Direction,
        ) -> Result<(), CryptoError>
        where
            H: for<'c> BlockingCipherOps<'c, AesGcm<'c, $key_size>>,
        {
            let cipher = AesGcm::<$key_size>::new(key, nonce);
            run_authenticated(hw, &cipher, direction, aad, input, output, tag, tag_output)
        }
    };
}

#[cfg(feature = "embassy-crypto-aes128-gcm")]
#[cfg(not(any(cryp_v1, aes_v1, all(feature = "embassy-crypto-saes", saes_v1b))))]
define_gcm_runner!(run_gcm128, 16);
#[cfg(feature = "embassy-crypto-aes256-gcm")]
#[cfg(not(any(cryp_v1, aes_v1, all(feature = "embassy-crypto-saes", saes_v1b))))]
define_gcm_runner!(run_gcm256, 32);

/// Run one AES-CCM operation via a type-erased `CcmOp`.
#[cfg(not(any(cryp_v1, aes_v1, all(feature = "embassy-crypto-saes", saes_v1b))))]
fn run_ccm<H>(
    hw: &mut H,
    key: &[u8],
    nonce: &[u8],
    tag_size: usize,
    direction: Direction,
    aad: &[u8],
    input: &[u8],
    output: &mut [u8],
    tag: Option<&[u8]>,
    tag_output: Option<&mut [u8]>,
) -> Result<(), CryptoError>
where
    H: for<'c> BlockingCipherOps<'c, CcmOp<'c>>,
{
    if !(7..=13).contains(&nonce.len()) {
        return Err(CryptoError::InvalidInput);
    }
    let cipher = CcmOp::new(key, nonce, tag_size, aad.len(), input.len());
    run_authenticated(hw, &cipher, direction, aad, input, output, tag, tag_output)
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
        let hw = &mut *driver.borrow();
        let cipher = AesEcb::new(ctx);
        let len = blocks.len();
        let (in_ptr, out_ptr) = blocks.into_raw();
        if in_ptr == out_ptr {
            let flat = unsafe { core::slice::from_raw_parts_mut(out_ptr, len) };
            run_in_place(hw, &cipher, Direction::Encrypt, flat).unwrap();
        } else {
            let input = unsafe { core::slice::from_raw_parts(in_ptr, len) };
            let output = unsafe { core::slice::from_raw_parts_mut(out_ptr, len) };
            run_separate(hw, &cipher, Direction::Encrypt, input, output).unwrap();
        }
    }

    fn decrypt_blocks(ctx: &Self::Context, blocks: embassy_crypto::driver::InOutBuf<'_, '_, u8>) {
        if blocks.is_empty() {
            return;
        }
        let mut driver = lock();
        let hw = &mut *driver.borrow();
        let cipher = AesEcb::new(ctx);
        let len = blocks.len();
        let (in_ptr, out_ptr) = blocks.into_raw();
        if in_ptr == out_ptr {
            let flat = unsafe { core::slice::from_raw_parts_mut(out_ptr, len) };
            run_in_place(hw, &cipher, Direction::Decrypt, flat).unwrap();
        } else {
            let input = unsafe { core::slice::from_raw_parts(in_ptr, len) };
            let output = unsafe { core::slice::from_raw_parts_mut(out_ptr, len) };
            run_separate(hw, &cipher, Direction::Decrypt, input, output).unwrap();
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
        let hw = &mut *driver.borrow();
        let cipher = AesEcb::new(ctx);
        let len = blocks.len();
        let (in_ptr, out_ptr) = blocks.into_raw();
        if in_ptr == out_ptr {
            let flat = unsafe { core::slice::from_raw_parts_mut(out_ptr, len) };
            run_in_place(hw, &cipher, Direction::Encrypt, flat).unwrap();
        } else {
            let input = unsafe { core::slice::from_raw_parts(in_ptr, len) };
            let output = unsafe { core::slice::from_raw_parts_mut(out_ptr, len) };
            run_separate(hw, &cipher, Direction::Encrypt, input, output).unwrap();
        }
    }

    fn decrypt_blocks(ctx: &Self::Context, blocks: embassy_crypto::driver::InOutBuf<'_, '_, u8>) {
        if blocks.is_empty() {
            return;
        }
        let mut driver = lock();
        let hw = &mut *driver.borrow();
        let cipher = AesEcb::new(ctx);
        let len = blocks.len();
        let (in_ptr, out_ptr) = blocks.into_raw();
        if in_ptr == out_ptr {
            let flat = unsafe { core::slice::from_raw_parts_mut(out_ptr, len) };
            run_in_place(hw, &cipher, Direction::Decrypt, flat).unwrap();
        } else {
            let input = unsafe { core::slice::from_raw_parts(in_ptr, len) };
            let output = unsafe { core::slice::from_raw_parts_mut(out_ptr, len) };
            run_separate(hw, &cipher, Direction::Decrypt, input, output).unwrap();
        }
    }
}

#[cfg(feature = "embassy-crypto-aes128-gcm")]
#[cfg(not(any(cryp_v1, aes_v1, all(feature = "embassy-crypto-saes", saes_v1b))))]
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
        let hw = &mut *driver.borrow();
        let len = buffer.len();
        let (in_ptr, out_ptr) = buffer.into_raw();
        let input = unsafe { core::slice::from_raw_parts(in_ptr, len) };
        let output = unsafe { core::slice::from_raw_parts_mut(out_ptr, len) };
        run_gcm128(hw, ctx, nonce, aad, input, output, None, Some(tag), Direction::Encrypt)
    }

    fn decrypt(
        ctx: &Self::Context,
        nonce: &[u8; 12],
        aad: &[u8],
        buffer: embassy_crypto::driver::InOutBuf<'_, '_, u8>,
        tag: &[u8; 16],
    ) -> Result<(), CryptoError> {
        let mut driver = lock();
        let hw = &mut *driver.borrow();
        let len = buffer.len();
        let (in_ptr, out_ptr) = buffer.into_raw();
        let input = unsafe { core::slice::from_raw_parts(in_ptr, len) };
        let output = unsafe { core::slice::from_raw_parts_mut(out_ptr, len) };
        run_gcm128(hw, ctx, nonce, aad, input, output, Some(tag), None, Direction::Decrypt)
    }
}

#[cfg(feature = "embassy-crypto-aes256-gcm")]
#[cfg(not(any(cryp_v1, aes_v1, all(feature = "embassy-crypto-saes", saes_v1b))))]
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
        let hw = &mut *driver.borrow();
        let len = buffer.len();
        let (in_ptr, out_ptr) = buffer.into_raw();
        let input = unsafe { core::slice::from_raw_parts(in_ptr, len) };
        let output = unsafe { core::slice::from_raw_parts_mut(out_ptr, len) };
        run_gcm256(hw, ctx, nonce, aad, input, output, None, Some(tag), Direction::Encrypt)
    }

    fn decrypt(
        ctx: &Self::Context,
        nonce: &[u8; 12],
        aad: &[u8],
        buffer: embassy_crypto::driver::InOutBuf<'_, '_, u8>,
        tag: &[u8; 16],
    ) -> Result<(), CryptoError> {
        let mut driver = lock();
        let hw = &mut *driver.borrow();
        let len = buffer.len();
        let (in_ptr, out_ptr) = buffer.into_raw();
        let input = unsafe { core::slice::from_raw_parts(in_ptr, len) };
        let output = unsafe { core::slice::from_raw_parts_mut(out_ptr, len) };
        run_gcm256(hw, ctx, nonce, aad, input, output, Some(tag), None, Direction::Decrypt)
    }
}

#[cfg(feature = "embassy-crypto-aes128-ccm")]
#[cfg(not(any(cryp_v1, aes_v1, all(feature = "embassy-crypto-saes", saes_v1b))))]
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
        let hw = &mut *driver.borrow();
        let len = buffer.len();
        let (in_ptr, out_ptr) = buffer.into_raw();
        let input = unsafe { core::slice::from_raw_parts(in_ptr, len) };
        let output = unsafe { core::slice::from_raw_parts_mut(out_ptr, len) };
        match tag.len() {
            4 | 6 | 8 | 10 | 12 | 14 | 16 => run_ccm(
                hw,
                ctx,
                nonce,
                tag.len(),
                Direction::Encrypt,
                aad,
                input,
                output,
                None,
                Some(tag),
            ),
            _ => Err(CryptoError::Unsupported),
        }
    }

    fn decrypt(
        ctx: &Self::Context,
        nonce: &[u8],
        aad: &[u8],
        buffer: embassy_crypto::driver::InOutBuf<'_, '_, u8>,
        tag: &[u8],
    ) -> Result<(), CryptoError> {
        let mut driver = lock();
        let hw = &mut *driver.borrow();
        let len = buffer.len();
        let (in_ptr, out_ptr) = buffer.into_raw();
        let input = unsafe { core::slice::from_raw_parts(in_ptr, len) };
        let output = unsafe { core::slice::from_raw_parts_mut(out_ptr, len) };
        match tag.len() {
            4 | 6 | 8 | 10 | 12 | 14 | 16 => run_ccm(
                hw,
                ctx,
                nonce,
                tag.len(),
                Direction::Decrypt,
                aad,
                input,
                output,
                Some(tag),
                None,
            ),
            _ => Err(CryptoError::Unsupported),
        }
    }
}

#[cfg(feature = "embassy-crypto-aes256-ccm")]
#[cfg(not(any(cryp_v1, aes_v1, all(feature = "embassy-crypto-saes", saes_v1b))))]
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
        let hw = &mut *driver.borrow();
        let len = buffer.len();
        let (in_ptr, out_ptr) = buffer.into_raw();
        let input = unsafe { core::slice::from_raw_parts(in_ptr, len) };
        let output = unsafe { core::slice::from_raw_parts_mut(out_ptr, len) };
        match tag.len() {
            4 | 6 | 8 | 10 | 12 | 14 | 16 => run_ccm(
                hw,
                ctx,
                nonce,
                tag.len(),
                Direction::Encrypt,
                aad,
                input,
                output,
                None,
                Some(tag),
            ),
            _ => Err(CryptoError::Unsupported),
        }
    }

    fn decrypt(
        ctx: &Self::Context,
        nonce: &[u8],
        aad: &[u8],
        buffer: embassy_crypto::driver::InOutBuf<'_, '_, u8>,
        tag: &[u8],
    ) -> Result<(), CryptoError> {
        let mut driver = lock();
        let hw = &mut *driver.borrow();
        let len = buffer.len();
        let (in_ptr, out_ptr) = buffer.into_raw();
        let input = unsafe { core::slice::from_raw_parts(in_ptr, len) };
        let output = unsafe { core::slice::from_raw_parts_mut(out_ptr, len) };
        match tag.len() {
            4 | 6 | 8 | 10 | 12 | 14 | 16 => run_ccm(
                hw,
                ctx,
                nonce,
                tag.len(),
                Direction::Decrypt,
                aad,
                input,
                output,
                Some(tag),
                None,
            ),
            _ => Err(CryptoError::Unsupported),
        }
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
        let hw = &mut *driver.borrow();
        let (key, iv) = ctx;
        let cipher = AesCbc::new(key, iv);
        let len = blocks.len();
        let (in_ptr, out_ptr) = blocks.into_raw();
        if in_ptr == out_ptr {
            let flat = unsafe { core::slice::from_raw_parts_mut(out_ptr, len) };
            run_in_place(hw, &cipher, Direction::Encrypt, flat).unwrap();
            let last_block: [u8; 16] = flat[flat.len() - 16..].try_into().unwrap();
            iv.copy_from_slice(&last_block);
        } else {
            let input = unsafe { core::slice::from_raw_parts(in_ptr, len) };
            let output = unsafe { core::slice::from_raw_parts_mut(out_ptr, len) };
            run_separate(hw, &cipher, Direction::Encrypt, input, output).unwrap();
            let last_block: [u8; 16] = output[output.len() - 16..].try_into().unwrap();
            iv.copy_from_slice(&last_block);
        }
    }

    fn decrypt_blocks(ctx: &mut Self::DecryptContext, blocks: embassy_crypto::driver::InOutBuf<'_, '_, u8>) {
        if blocks.is_empty() {
            return;
        }
        let mut driver = lock();
        let hw = &mut *driver.borrow();
        let (key, iv) = ctx;
        let cipher = AesCbc::new(key, iv);
        let len = blocks.len();
        let (in_ptr, out_ptr) = blocks.into_raw();
        if in_ptr == out_ptr {
            let flat = unsafe { core::slice::from_raw_parts_mut(out_ptr, len) };
            let last_ciphertext: [u8; 16] = flat[flat.len() - 16..].try_into().unwrap();
            run_in_place(hw, &cipher, Direction::Decrypt, flat).unwrap();
            iv.copy_from_slice(&last_ciphertext);
        } else {
            let input = unsafe { core::slice::from_raw_parts(in_ptr, len) };
            let output = unsafe { core::slice::from_raw_parts_mut(out_ptr, len) };
            let last_ciphertext: [u8; 16] = input[input.len() - 16..].try_into().unwrap();
            run_separate(hw, &cipher, Direction::Decrypt, input, output).unwrap();
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
        let hw = &mut *driver.borrow();
        let (key, iv) = ctx;
        let cipher = AesCbc::new(key, iv);
        let len = blocks.len();
        let (in_ptr, out_ptr) = blocks.into_raw();
        if in_ptr == out_ptr {
            let flat = unsafe { core::slice::from_raw_parts_mut(out_ptr, len) };
            run_in_place(hw, &cipher, Direction::Encrypt, flat).unwrap();
            let last_block: [u8; 16] = flat[flat.len() - 16..].try_into().unwrap();
            iv.copy_from_slice(&last_block);
        } else {
            let input = unsafe { core::slice::from_raw_parts(in_ptr, len) };
            let output = unsafe { core::slice::from_raw_parts_mut(out_ptr, len) };
            run_separate(hw, &cipher, Direction::Encrypt, input, output).unwrap();
            let last_block: [u8; 16] = output[output.len() - 16..].try_into().unwrap();
            iv.copy_from_slice(&last_block);
        }
    }

    fn decrypt_blocks(ctx: &mut Self::DecryptContext, blocks: embassy_crypto::driver::InOutBuf<'_, '_, u8>) {
        if blocks.is_empty() {
            return;
        }
        let mut driver = lock();
        let hw = &mut *driver.borrow();
        let (key, iv) = ctx;
        let cipher = AesCbc::new(key, iv);
        let len = blocks.len();
        let (in_ptr, out_ptr) = blocks.into_raw();
        if in_ptr == out_ptr {
            let flat = unsafe { core::slice::from_raw_parts_mut(out_ptr, len) };
            let last_ciphertext: [u8; 16] = flat[flat.len() - 16..].try_into().unwrap();
            run_in_place(hw, &cipher, Direction::Decrypt, flat).unwrap();
            iv.copy_from_slice(&last_ciphertext);
        } else {
            let input = unsafe { core::slice::from_raw_parts(in_ptr, len) };
            let output = unsafe { core::slice::from_raw_parts_mut(out_ptr, len) };
            let last_ciphertext: [u8; 16] = input[input.len() - 16..].try_into().unwrap();
            run_separate(hw, &cipher, Direction::Decrypt, input, output).unwrap();
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

/// XOR the keystream into full 16-byte blocks of a buffer, defined once per
/// key size so the cipher type (and its `CipherSized` bound) stays concrete.
#[cfg(any(feature = "embassy-crypto-aes128-ctr", feature = "embassy-crypto-aes256-ctr"))]
#[cfg(not(all(feature = "embassy-crypto-saes", saes_v1b)))]
macro_rules! define_ctr_blocks {
    ($in_place:ident, $separate:ident, $key_size:literal) => {
        fn $in_place<H>(hw: &mut H, key: &[u8; $key_size], iv: &mut [u8; 16], buffer: &mut [u8])
        where
            H: for<'c> BlockingCipherOps<'c, AesCtr<'c, $key_size>>,
        {
            let mut buffer = buffer;
            while buffer.len() >= 16 {
                let n = ctr_run_blocks(iv, buffer.len() / 16);
                let (run, rest) = buffer.split_at_mut(n * 16);
                let start = *iv;
                let cipher = AesCtr::<$key_size>::new(key, &start);
                let mut context = hw.start(&cipher, Direction::Encrypt).map_err(map_error).unwrap();
                let run_len = run.len();
                let mut processed = 0;
                for chunk in run.chunks_exact_mut(16) {
                    processed += 16;
                    let ptr = chunk.as_mut_ptr();
                    let len = chunk.len();
                    let input = unsafe { core::slice::from_raw_parts(ptr, len) };
                    let output = unsafe { core::slice::from_raw_parts_mut(ptr, len) };
                    hw.payload(&mut context, input, output, processed == run_len)
                        .map_err(map_error)
                        .unwrap();
                }
                ctr_advance(iv, n);
                buffer = rest;
            }
        }

        fn $separate<H>(hw: &mut H, key: &[u8; $key_size], iv: &mut [u8; 16], input: &[u8], output: &mut [u8])
        where
            H: for<'c> BlockingCipherOps<'c, AesCtr<'c, $key_size>>,
        {
            assert_eq!(input.len(), output.len());
            let mut input = input;
            let mut output = output;
            while input.len() >= 16 {
                let n = ctr_run_blocks(iv, input.len() / 16);
                let (in_run, in_rest) = input.split_at(n * 16);
                let (out_run, out_rest) = output.split_at_mut(n * 16);
                let start = *iv;
                let cipher = AesCtr::<$key_size>::new(key, &start);
                let mut context = hw.start(&cipher, Direction::Encrypt).map_err(map_error).unwrap();
                let run_len = in_run.len();
                let mut processed = 0;
                for (in_chunk, out_chunk) in in_run.chunks_exact(16).zip(out_run.chunks_exact_mut(16)) {
                    processed += 16;
                    hw.payload(&mut context, in_chunk, out_chunk, processed == run_len)
                        .map_err(map_error)
                        .unwrap();
                }
                ctr_advance(iv, n);
                input = in_rest;
                output = out_rest;
            }
        }
    };
}

#[cfg(feature = "embassy-crypto-aes128-ctr")]
#[cfg(not(all(feature = "embassy-crypto-saes", saes_v1b)))]
define_ctr_blocks!(ctr_blocks_in_place_16, ctr_blocks_separate_16, 16);
#[cfg(feature = "embassy-crypto-aes256-ctr")]
#[cfg(not(all(feature = "embassy-crypto-saes", saes_v1b)))]
#[cfg(not(aes_v1))]
define_ctr_blocks!(ctr_blocks_in_place_32, ctr_blocks_separate_32, 32);

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
        let mut driver = lock();
        let hw = &mut *driver.borrow();
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
                ctr_blocks_in_place_16(hw, key, iv, &mut buf[..full_len]);
            }

            // 3. Generate one extra keystream block for any trailing partial data.
            let tail = &mut buf[full_len..];
            if !tail.is_empty() {
                let mut keystream = [0u8; 16];
                // Advances the counter, unlike a bare hardware run would.
                ctr_blocks_in_place_16(hw, key, iv, &mut keystream);
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
                ctr_blocks_separate_16(hw, key, iv, &input[..full_len], &mut output[..full_len]);
            }

            // 3. Generate one extra keystream block for trailing partial data.
            let tail_in = &input[full_len..];
            let tail_out = &mut output[full_len..];
            if !tail_in.is_empty() {
                let mut keystream = [0u8; 16];
                // Advances the counter, unlike a bare hardware run would.
                ctr_blocks_in_place_16(hw, key, iv, &mut keystream);
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
        let mut driver = lock();
        let hw = &mut *driver.borrow();
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
                ctr_blocks_in_place_32(hw, key, iv, &mut buf[..full_len]);
            }

            let tail = &mut buf[full_len..];
            if !tail.is_empty() {
                let mut keystream = [0u8; 16];
                // Advances the counter, unlike a bare hardware run would.
                ctr_blocks_in_place_32(hw, key, iv, &mut keystream);
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
                ctr_blocks_separate_32(hw, key, iv, &input[..full_len], &mut output[..full_len]);
            }

            let tail_in = &input[full_len..];
            let tail_out = &mut output[full_len..];
            if !tail_in.is_empty() {
                let mut keystream = [0u8; 16];
                // Advances the counter, unlike a bare hardware run would.
                ctr_blocks_in_place_32(hw, key, iv, &mut keystream);
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

// ===========================================================================
// AES-128/256 CMAC (NIST SP 800-38B)
// ===========================================================================
//
// No AES/SAES/CRYP revision exposes a CMAC mode, so the MAC is computed in
// software per NIST SP 800-38B with the raw block cipher runs served by the
// peripheral — the same layering the CTR driver uses. ECB is universal, so
// CMAC works on every backend; only the 256-bit key size is unavailable on
// `aes_v1`.

/// State of one CMAC computation.
#[cfg(any(feature = "embassy-crypto-aes128-cmac", feature = "embassy-crypto-aes256-cmac"))]
#[derive(Clone)]
struct CmacContext<const KEY_SIZE: usize> {
    key: [u8; KEY_SIZE],
    /// First subkey, `2L` for `L = AES_k(0)`.
    k1: [u8; 16],
    /// Second subkey, `2K1`.
    k2: [u8; 16],
    /// Chaining state X.
    state: [u8; 16],
    /// Pending message bytes, holding back the last (possibly partial) block.
    buf: [u8; 16],
    buf_len: usize,
}

/// The low term of the reduction polynomial `x^128 + x^7 + x^2 + x + 1`
/// (NIST SP 800-38B §5.3).
#[cfg(any(feature = "embassy-crypto-aes128-cmac", feature = "embassy-crypto-aes256-cmac"))]
const CMAC_RB: u8 = 0x87;

/// One multiplication by `x` (doubling) in GF(2^128).
#[cfg(any(feature = "embassy-crypto-aes128-cmac", feature = "embassy-crypto-aes256-cmac"))]
fn cmac_double(block: &[u8; 16]) -> [u8; 16] {
    let mut out = [0u8; 16];
    let mut carry = 0u8;
    for i in (0..16).rev() {
        out[i] = (block[i] << 1) | carry;
        carry = block[i] >> 7;
    }
    if carry != 0 {
        out[15] ^= CMAC_RB;
    }
    out
}

/// Fold one full message block into the chaining state: `X = AES_k(X ^ block)`.
/// One concrete function per key size, so the cipher type (and its
/// `CipherSized` bound) stays concrete.
#[cfg(any(feature = "embassy-crypto-aes128-cmac", feature = "embassy-crypto-aes256-cmac"))]
macro_rules! define_cmac_process {
    ($name:ident, $key_size:literal) => {
        fn $name(hw: &mut Hw, key: &[u8; $key_size], state: &mut [u8; 16], block: &[u8; 16]) {
            let mut block = *block;
            for i in 0..16 {
                block[i] ^= state[i];
            }
            let cipher = AesEcb::new(key);
            run_in_place(hw, &cipher, Direction::Encrypt, &mut block).unwrap();
            *state = block;
        }
    };
}

#[cfg(feature = "embassy-crypto-aes128-cmac")]
define_cmac_process!(cmac_process_16, 16);
#[cfg(feature = "embassy-crypto-aes256-cmac")]
#[cfg(not(aes_v1))]
define_cmac_process!(cmac_process_32, 32);

#[cfg(any(feature = "embassy-crypto-aes128-cmac", feature = "embassy-crypto-aes256-cmac"))]
macro_rules! define_cmac_impl {
    ($trait:ident, $key_size:literal, $process:ident) => {
        impl embassy_crypto::driver::$trait for AesDriver {
            type Context = CmacContext<$key_size>;

            fn init(key: &[u8; $key_size]) -> Self::Context {
                let mut driver = lock();
                let hw = &mut *driver.borrow();
                // Subkeys K1 = 2L and K2 = 2K1, with L = AES_k(0) (§6.1).
                let mut l = [0u8; 16];
                let cipher = AesEcb::new(key);
                run_in_place(hw, &cipher, Direction::Encrypt, &mut l).unwrap();
                let k1 = cmac_double(&l);
                let k2 = cmac_double(&k1);
                CmacContext {
                    key: *key,
                    k1,
                    k2,
                    state: [0; 16],
                    buf: [0; 16],
                    buf_len: 0,
                }
            }

            fn update(ctx: &mut Self::Context, mut data: &[u8]) {
                if data.is_empty() {
                    return;
                }
                let mut driver = lock();
                let hw = &mut *driver.borrow();
                // A buffered full block was only possibly the last one; with
                // more data arriving it no longer is, so fold it in.
                if ctx.buf_len == 16 {
                    $process(hw, &ctx.key, &mut ctx.state, &ctx.buf);
                    ctx.buf_len = 0;
                }
                // Top up a partial block; if it fills and more data follows,
                // it is not the last block either.
                if ctx.buf_len > 0 {
                    let n = core::cmp::min(16 - ctx.buf_len, data.len());
                    ctx.buf[ctx.buf_len..ctx.buf_len + n].copy_from_slice(&data[..n]);
                    ctx.buf_len += n;
                    data = &data[n..];
                    if data.is_empty() {
                        return;
                    }
                    $process(hw, &ctx.key, &mut ctx.state, &ctx.buf);
                    ctx.buf_len = 0;
                }
                // Fold in all but the last full block, which stays buffered:
                // only finalize knows whether it is XORed with K1 or padded
                // and XORed with K2.
                while data.len() > 16 {
                    let block: [u8; 16] = data[..16].try_into().unwrap();
                    $process(hw, &ctx.key, &mut ctx.state, &block);
                    data = &data[16..];
                }
                ctx.buf[..data.len()].copy_from_slice(data);
                ctx.buf_len = data.len();
            }

            fn finalize(ctx: Self::Context, out: &mut [u8; 16]) {
                let mut driver = lock();
                let hw = &mut *driver.borrow();
                let mut last = [0u8; 16];
                last[..ctx.buf_len].copy_from_slice(&ctx.buf[..ctx.buf_len]);
                if ctx.buf_len == 16 {
                    // Complete final block: no padding, XOR K1.
                    for i in 0..16 {
                        last[i] ^= ctx.k1[i];
                    }
                } else {
                    // Partial final block: pad with 10*, then XOR K2.
                    last[ctx.buf_len] ^= 0x80;
                    for i in 0..16 {
                        last[i] ^= ctx.k2[i];
                    }
                }
                let mut block = last;
                for i in 0..16 {
                    block[i] ^= ctx.state[i];
                }
                let cipher = AesEcb::new(&ctx.key);
                run_in_place(hw, &cipher, Direction::Encrypt, &mut block).unwrap();
                *out = block;
            }

            fn reset(ctx: &mut Self::Context) {
                ctx.state = [0; 16];
                ctx.buf = [0; 16];
                ctx.buf_len = 0;
            }
        }
    };
}

#[cfg(feature = "embassy-crypto-aes128-cmac")]
define_cmac_impl!(Aes128Cmac, 16, cmac_process_16);
#[cfg(feature = "embassy-crypto-aes256-cmac")]
#[cfg(not(aes_v1))]
define_cmac_impl!(Aes256Cmac, 32, cmac_process_32);

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
#[cfg(not(any(cryp_v1, aes_v1, all(feature = "embassy-crypto-saes", saes_v1b))))]
embassy_crypto::aes128_gcm_impl!(AesDriver);
#[cfg(feature = "embassy-crypto-aes256-gcm")]
#[cfg(not(any(cryp_v1, aes_v1, all(feature = "embassy-crypto-saes", saes_v1b))))]
embassy_crypto::aes256_gcm_impl!(AesDriver);
#[cfg(feature = "embassy-crypto-aes128-ccm")]
#[cfg(not(any(cryp_v1, aes_v1, all(feature = "embassy-crypto-saes", saes_v1b))))]
embassy_crypto::aes128_ccm_impl!(AesDriver);
#[cfg(feature = "embassy-crypto-aes256-ccm")]
#[cfg(not(any(cryp_v1, aes_v1, all(feature = "embassy-crypto-saes", saes_v1b))))]
embassy_crypto::aes256_ccm_impl!(AesDriver);
#[cfg(feature = "embassy-crypto-aes128-ctr")]
#[cfg(not(all(feature = "embassy-crypto-saes", saes_v1b)))]
embassy_crypto::aes128_ctr_impl!(AesDriver);
#[cfg(feature = "embassy-crypto-aes256-ctr")]
#[cfg(not(all(feature = "embassy-crypto-saes", saes_v1b)))]
#[cfg(not(aes_v1))]
embassy_crypto::aes256_ctr_impl!(AesDriver);
#[cfg(feature = "embassy-crypto-aes128-cmac")]
embassy_crypto::aes128_cmac_impl!(AesDriver);
#[cfg(feature = "embassy-crypto-aes256-cmac")]
#[cfg(not(aes_v1))]
embassy_crypto::aes256_cmac_impl!(AesDriver);
