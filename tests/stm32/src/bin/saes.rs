// required-features: saes
#![no_std]
#![no_main]

//! The SAES peripheral through its own API, against the shared known-answer suites.
//!
//! The suites are written for the `embassy-crypto` API, and `embassy-crypto` can only be
//! served by one peripheral per binary, so this wraps the `Saes` driver in the suites'
//! traits instead. CTR and the authenticated modes run where the SAES has them
//! (`saes-full`); the STM32U5 one only does ECB and CBC.

#[path = "../common.rs"]
mod common;

use core::cell::RefCell;

use common::*;
use critical_section::Mutex;
use defmt_rtt as _;
use embassy_crypto::Error;
use embassy_crypto_test::vectors;
use embassy_executor::Spawner;
use embassy_stm32::mode::Blocking;
use embassy_stm32::peripherals::SAES;
use embassy_stm32::rng::Rng;
use embassy_stm32::saes::{AesCbc, AesEcb, Cipher, CipherSized, Direction, IVSized, Saes};
#[cfg(feature = "saes-full")]
use embassy_stm32::saes::{AesCcm, AesCtr, AesGcm, CipherAuthenticated};
use embassy_stm32::{bind_interrupts, peripherals, rng, saes};
use panic_probe as _;

bind_interrupts!(struct Irqs {
    SAES => saes::InterruptHandler<peripherals::SAES>;
    RNG => rng::InterruptHandler<peripherals::RNG>;
});

/// The largest message the suites feed.
const MAX: usize = 1024;

static DRIVER: Mutex<RefCell<Option<Saes<'static, SAES, Blocking>>>> = Mutex::new(RefCell::new(None));

fn with_saes<R>(f: impl FnOnce(&mut Saes<'static, SAES, Blocking>) -> R) -> R {
    critical_section::with(|cs| f(DRIVER.borrow_ref_mut(cs).as_mut().unwrap()))
}

fn map(e: saes::Error) -> Error {
    match e {
        saes::Error::KeyError => Error::InvalidKey,
        saes::Error::ConfigError => Error::InvalidInput,
        saes::Error::ReadError | saes::Error::WriteError => Error::HardwareError,
    }
}

/// Runs whole blocks of `input` into `output`, which must not alias.
fn run_blocks<'c, C>(cipher: &'c C, dir: Direction, input: &[u8], output: &mut [u8]) -> Result<(), Error>
where
    C: Cipher<'c> + CipherSized + IVSized,
{
    if input.len() != output.len() || input.len() % 16 != 0 {
        return Err(Error::InvalidInput);
    }
    with_saes(|saes| {
        let mut ctx = saes.start(cipher, dir);
        saes.payload_blocking(&mut ctx, input, output, true).map_err(map)?;
        saes.finish_blocking(ctx).map_err(map)?;
        Ok(())
    })
}

/// Runs whole blocks in place, through a copy.
fn run_blocks_in_place<'c, C>(cipher: &'c C, dir: Direction, buf: &mut [u8]) -> Result<(), Error>
where
    C: Cipher<'c> + CipherSized + IVSized,
{
    if buf.len() > MAX {
        return Err(Error::InvalidInput);
    }
    let mut tmp = [0u8; MAX];
    let tmp = &mut tmp[..buf.len()];
    tmp.copy_from_slice(buf);
    run_blocks(cipher, dir, tmp, buf)
}

// ===========================================================================
// ECB
// ===========================================================================

struct Ecb<const K: usize>([u8; K]);

macro_rules! impl_ecb {
    ($k:literal) => {
        impl embassy_crypto_test::BlockCipher for Ecb<$k> {
            const KEY_SIZE: usize = $k;
            fn new(key: &[u8]) -> Option<Self> {
                Some(Self(key.try_into().ok()?))
            }
            fn encrypt_blocks(&self, blocks: &mut [u8]) -> Result<(), Error> {
                run_blocks_in_place(&AesEcb::new(&self.0), Direction::Encrypt, blocks)
            }
            fn decrypt_blocks(&self, blocks: &mut [u8]) -> Result<(), Error> {
                run_blocks_in_place(&AesEcb::new(&self.0), Direction::Decrypt, blocks)
            }
            fn encrypt_blocks_to(&self, input: &[u8], output: &mut [u8]) -> Result<(), Error> {
                run_blocks(&AesEcb::new(&self.0), Direction::Encrypt, input, output)
            }
            fn decrypt_blocks_to(&self, input: &[u8], output: &mut [u8]) -> Result<(), Error> {
                run_blocks(&AesEcb::new(&self.0), Direction::Decrypt, input, output)
            }
        }
    };
}
impl_ecb!(16);
impl_ecb!(32);

// ===========================================================================
// CBC
// ===========================================================================

/// A CBC direction: the IV chains from one call to the next.
struct Cbc<const K: usize> {
    key: [u8; K],
    iv: [u8; 16],
}

fn last_block(buf: &[u8]) -> [u8; 16] {
    buf[buf.len() - 16..].try_into().unwrap()
}

macro_rules! impl_cbc {
    ($k:literal) => {
        impl embassy_crypto_test::CbcEncrypt for Cbc<$k> {
            fn new(key: &[u8], iv: &[u8; 16]) -> Option<Self> {
                Some(Self {
                    key: key.try_into().ok()?,
                    iv: *iv,
                })
            }
            fn encrypt(&mut self, blocks: &mut [u8]) -> Result<(), Error> {
                if blocks.is_empty() {
                    return Ok(());
                }
                run_blocks_in_place(&AesCbc::new(&self.key, &self.iv), Direction::Encrypt, blocks)?;
                self.iv = last_block(blocks);
                Ok(())
            }
            fn encrypt_to(&mut self, input: &[u8], output: &mut [u8]) -> Result<(), Error> {
                if input.is_empty() && output.is_empty() {
                    return Ok(());
                }
                run_blocks(
                    &AesCbc::new(&self.key, &self.iv),
                    Direction::Encrypt,
                    input,
                    output,
                )?;
                self.iv = last_block(output);
                Ok(())
            }
        }
        impl embassy_crypto_test::CbcDecrypt for Cbc<$k> {
            fn new(key: &[u8], iv: &[u8; 16]) -> Option<Self> {
                Some(Self {
                    key: key.try_into().ok()?,
                    iv: *iv,
                })
            }
            fn decrypt(&mut self, blocks: &mut [u8]) -> Result<(), Error> {
                if blocks.is_empty() {
                    return Ok(());
                }
                let next = last_block(blocks);
                run_blocks_in_place(&AesCbc::new(&self.key, &self.iv), Direction::Decrypt, blocks)?;
                self.iv = next;
                Ok(())
            }
            fn decrypt_to(&mut self, input: &[u8], output: &mut [u8]) -> Result<(), Error> {
                if input.is_empty() && output.is_empty() {
                    return Ok(());
                }
                run_blocks(
                    &AesCbc::new(&self.key, &self.iv),
                    Direction::Decrypt,
                    input,
                    output,
                )?;
                self.iv = last_block(input);
                Ok(())
            }
        }
    };
}
impl_cbc!(16);
impl_cbc!(32);

// ===========================================================================
// CTR
// ===========================================================================

/// A CTR keystream: the counter block, and the unused end of the last block.
#[cfg(feature = "saes-full")]
struct Ctr<const K: usize> {
    key: [u8; K],
    counter: [u8; 16],
    partial: [u8; 16],
    partial_len: usize,
}

/// How many blocks one hardware run may take from `counter`: the peripheral only
/// increments its low 32 bits, so a run stops where they would wrap.
#[cfg(feature = "saes-full")]
fn ctr_run_blocks(counter: &[u8; 16], blocks: usize) -> usize {
    let low = u32::from_be_bytes(counter[12..].try_into().unwrap());
    let until_wrap = u64::from(u32::MAX - low) + 1;
    if until_wrap >= blocks as u64 {
        blocks
    } else {
        until_wrap as usize
    }
}

#[cfg(feature = "saes-full")]
macro_rules! impl_ctr {
    ($k:literal) => {
        impl Ctr<$k> {
            /// Encrypts whole blocks in place from the current counter, advancing it.
            fn blocks(&mut self, buf: &mut [u8]) {
                let mut buf = buf;
                while !buf.is_empty() {
                    let n = ctr_run_blocks(&self.counter, buf.len() / 16);
                    let (run, rest) = buf.split_at_mut(n * 16);
                    let start = self.counter;
                    let cipher = AesCtr::<$k>::new(&self.key, &start);
                    with_saes(|saes| {
                        let mut ctx = saes.start(&cipher, Direction::Encrypt);
                        for chunk in run.chunks_exact_mut(16) {
                            let input: [u8; 16] = chunk.try_into().unwrap();
                            saes.payload_blocking(&mut ctx, &input, chunk, true).unwrap();
                        }
                        saes.finish_blocking(ctx).unwrap();
                    });
                    self.counter = u128::from_be_bytes(self.counter)
                        .wrapping_add(n as u128)
                        .to_be_bytes();
                    buf = rest;
                }
            }
        }

        impl embassy_crypto_test::Ctr for Ctr<$k> {
            fn new(key: &[u8], iv: &[u8; 16]) -> Option<Self> {
                Some(Self {
                    key: key.try_into().ok()?,
                    counter: *iv,
                    partial: [0; 16],
                    partial_len: 0,
                })
            }
            fn apply_keystream(&mut self, buf: &mut [u8]) {
                let mut buf = buf;
                // The end of the previous block first.
                let n = self.partial_len.min(buf.len());
                for (b, k) in buf[..n].iter_mut().zip(&self.partial[16 - self.partial_len..]) {
                    *b ^= k;
                }
                self.partial_len -= n;
                buf = &mut buf[n..];
                // Whole blocks through the hardware.
                let full = buf.len() / 16 * 16;
                let (whole, tail) = buf.split_at_mut(full);
                self.blocks(whole);
                // A last partial block: one more keystream block, the rest of which is kept.
                if !tail.is_empty() {
                    let mut keystream = [0u8; 16];
                    self.blocks(&mut keystream);
                    for (b, k) in tail.iter_mut().zip(&keystream) {
                        *b ^= k;
                    }
                    self.partial = keystream;
                    self.partial_len = 16 - tail.len();
                }
            }
            fn apply_keystream_to(&mut self, input: &[u8], output: &mut [u8]) -> Result<(), Error> {
                if input.len() != output.len() {
                    return Err(Error::InvalidInput);
                }
                output.copy_from_slice(input);
                self.apply_keystream(output);
                Ok(())
            }
        }
    };
}
#[cfg(feature = "saes-full")]
impl_ctr!(16);
#[cfg(feature = "saes-full")]
impl_ctr!(32);

// ===========================================================================
// GCM and CCM
// ===========================================================================

/// Runs an authenticated operation: `output` is the payload, the tag is returned.
#[cfg(feature = "saes-full")]
fn run_aead<'c, C, const T: usize>(
    cipher: &'c C,
    dir: Direction,
    aad: &[u8],
    input: &[u8],
    output: &mut [u8],
) -> Result<[u8; 16], Error>
where
    C: Cipher<'c> + CipherSized + IVSized + CipherAuthenticated<T>,
{
    if input.len() != output.len() {
        return Err(Error::InvalidInput);
    }
    with_saes(|saes| {
        let mut ctx = saes.start(cipher, dir);
        saes.aad_blocking(&mut ctx, aad, true).map_err(map)?;
        saes.payload_blocking(&mut ctx, input, output, true).map_err(map)?;
        saes.finish_blocking(ctx).map_err(map)?.ok_or(Error::HardwareError)
    })
}

/// Like [`run_aead`], in place through a copy.
#[cfg(feature = "saes-full")]
fn run_aead_in_place<'c, C, const T: usize>(
    cipher: &'c C,
    dir: Direction,
    aad: &[u8],
    buf: &mut [u8],
) -> Result<[u8; 16], Error>
where
    C: Cipher<'c> + CipherSized + IVSized + CipherAuthenticated<T>,
{
    if buf.len() > MAX {
        return Err(Error::InvalidInput);
    }
    let mut tmp = [0u8; MAX];
    let tmp = &mut tmp[..buf.len()];
    tmp.copy_from_slice(buf);
    run_aead(cipher, dir, aad, tmp, buf)
}

/// Checks a tag, in constant time.
#[cfg(feature = "saes-full")]
fn check_tag(computed: &[u8], expected: &[u8]) -> Result<(), Error> {
    let mut diff = 0;
    for (a, b) in computed.iter().zip(expected) {
        diff |= a ^ b;
    }
    if diff == 0 {
        Ok(())
    } else {
        Err(Error::InvalidSignature)
    }
}

#[cfg(feature = "saes-full")]
struct Gcm<const K: usize>([u8; K]);

#[cfg(feature = "saes-full")]
macro_rules! impl_gcm {
    ($k:literal) => {
        impl embassy_crypto_test::Gcm for Gcm<$k> {
            fn new(key: &[u8]) -> Option<Self> {
                Some(Self(key.try_into().ok()?))
            }
            fn encrypt(&self, nonce: &[u8; 12], aad: &[u8], buf: &mut [u8]) -> Result<[u8; 16], Error> {
                run_aead_in_place(&AesGcm::new(&self.0, nonce), Direction::Encrypt, aad, buf)
            }
            fn encrypt_to(
                &self,
                nonce: &[u8; 12],
                aad: &[u8],
                input: &[u8],
                output: &mut [u8],
            ) -> Result<[u8; 16], Error> {
                run_aead(
                    &AesGcm::new(&self.0, nonce),
                    Direction::Encrypt,
                    aad,
                    input,
                    output,
                )
            }
            fn decrypt(&self, nonce: &[u8; 12], aad: &[u8], buf: &mut [u8], tag: &[u8; 16]) -> Result<(), Error> {
                let computed = run_aead_in_place(&AesGcm::new(&self.0, nonce), Direction::Decrypt, aad, buf)?;
                check_tag(&computed, tag)
            }
            fn decrypt_to(
                &self,
                nonce: &[u8; 12],
                aad: &[u8],
                input: &[u8],
                output: &mut [u8],
                tag: &[u8; 16],
            ) -> Result<(), Error> {
                let computed = run_aead(
                    &AesGcm::new(&self.0, nonce),
                    Direction::Decrypt,
                    aad,
                    input,
                    output,
                )?;
                check_tag(&computed, tag)
            }
        }
    };
}
#[cfg(feature = "saes-full")]
impl_gcm!(16);
#[cfg(feature = "saes-full")]
impl_gcm!(32);

#[cfg(feature = "saes-full")]
struct Ccm<const K: usize>([u8; K]);

/// `AesCcm` takes the nonce and tag sizes as const generics: pick them at run time.
#[cfg(feature = "saes-full")]
macro_rules! ccm_run {
    ($k:literal, $n:literal, $t:literal, $key:expr, $nonce:expr, $aad:expr, $input:expr, $output:expr, $dir:expr) => {{
        let nonce: &[u8; $n] = $nonce.try_into().unwrap();
        let cipher = AesCcm::<$k, $n, $t>::new($key, nonce, $aad.len(), $input.len());
        run_aead(&cipher, $dir, $aad, $input, $output)
    }};
}
#[cfg(feature = "saes-full")]
macro_rules! ccm_tag {
    ($k:literal, $n:literal, $tag_len:expr, $key:expr, $nonce:expr, $aad:expr, $input:expr, $output:expr, $dir:expr) => {
        match $tag_len {
            4 => ccm_run!($k, $n, 4, $key, $nonce, $aad, $input, $output, $dir),
            6 => ccm_run!($k, $n, 6, $key, $nonce, $aad, $input, $output, $dir),
            8 => ccm_run!($k, $n, 8, $key, $nonce, $aad, $input, $output, $dir),
            10 => ccm_run!($k, $n, 10, $key, $nonce, $aad, $input, $output, $dir),
            12 => ccm_run!($k, $n, 12, $key, $nonce, $aad, $input, $output, $dir),
            14 => ccm_run!($k, $n, 14, $key, $nonce, $aad, $input, $output, $dir),
            16 => ccm_run!($k, $n, 16, $key, $nonce, $aad, $input, $output, $dir),
            _ => Err(Error::InvalidInput),
        }
    };
}
#[cfg(feature = "saes-full")]
macro_rules! ccm {
    ($k:literal, $key:expr, $nonce:expr, $tag_len:expr, $aad:expr, $input:expr, $output:expr, $dir:expr) => {
        match $nonce.len() {
            7 => ccm_tag!($k, 7, $tag_len, $key, $nonce, $aad, $input, $output, $dir),
            8 => ccm_tag!($k, 8, $tag_len, $key, $nonce, $aad, $input, $output, $dir),
            9 => ccm_tag!($k, 9, $tag_len, $key, $nonce, $aad, $input, $output, $dir),
            10 => ccm_tag!($k, 10, $tag_len, $key, $nonce, $aad, $input, $output, $dir),
            11 => ccm_tag!($k, 11, $tag_len, $key, $nonce, $aad, $input, $output, $dir),
            12 => ccm_tag!($k, 12, $tag_len, $key, $nonce, $aad, $input, $output, $dir),
            13 => ccm_tag!($k, 13, $tag_len, $key, $nonce, $aad, $input, $output, $dir),
            _ => Err(Error::InvalidInput),
        }
    };
}

#[cfg(feature = "saes-full")]
macro_rules! impl_ccm {
    ($k:literal) => {
        impl embassy_crypto_test::Ccm for Ccm<$k> {
            fn new(key: &[u8]) -> Option<Self> {
                Some(Self(key.try_into().ok()?))
            }
            fn encrypt(&self, nonce: &[u8], aad: &[u8], buf: &mut [u8], tag: &mut [u8]) -> Result<(), Error> {
                if buf.len() > MAX {
                    return Err(Error::InvalidInput);
                }
                let mut tmp = [0u8; MAX];
                let tmp = &mut tmp[..buf.len()];
                tmp.copy_from_slice(buf);
                self.encrypt_to(nonce, aad, tmp, buf, tag)
            }
            fn encrypt_to(
                &self,
                nonce: &[u8],
                aad: &[u8],
                input: &[u8],
                output: &mut [u8],
                tag: &mut [u8],
            ) -> Result<(), Error> {
                let computed = ccm!(
                    $k,
                    &self.0,
                    nonce,
                    tag.len(),
                    aad,
                    input,
                    output,
                    Direction::Encrypt
                )?;
                tag.copy_from_slice(&computed[..tag.len()]);
                Ok(())
            }
            fn decrypt(&self, nonce: &[u8], aad: &[u8], buf: &mut [u8], tag: &[u8]) -> Result<(), Error> {
                if buf.len() > MAX {
                    return Err(Error::InvalidInput);
                }
                let mut tmp = [0u8; MAX];
                let tmp = &mut tmp[..buf.len()];
                tmp.copy_from_slice(buf);
                self.decrypt_to(nonce, aad, tmp, buf, tag)
            }
            fn decrypt_to(
                &self,
                nonce: &[u8],
                aad: &[u8],
                input: &[u8],
                output: &mut [u8],
                tag: &[u8],
            ) -> Result<(), Error> {
                let computed = ccm!(
                    $k,
                    &self.0,
                    nonce,
                    tag.len(),
                    aad,
                    input,
                    output,
                    Direction::Decrypt
                )?;
                check_tag(&computed[..tag.len()], tag)
            }
        }
    };
}
#[cfg(feature = "saes-full")]
impl_ccm!(16);
#[cfg(feature = "saes-full")]
impl_ccm!(32);

// ===========================================================================
// Suites
// ===========================================================================

/// Run every suite, logging each result, and fail at the end if any failed.
macro_rules! suites {
    ($($name:ident = $run:expr),* $(,)?) => {{
        let mut ok = true;
        $(
            match $run {
                Ok(stats) => info!("{}: {:?}", stringify!($name), stats),
                Err(e) => {
                    error!("{}: {:?}", stringify!($name), e);
                    ok = false;
                }
            }
        )*
        defmt::assert!(ok, "some suites failed");
    }};
}

#[cfg_attr(
    feature = "stop",
    embassy_executor::main(executor = "embassy_stm32::executor::Executor", entry = "cortex_m_rt::entry")
)]
#[cfg_attr(not(feature = "stop"), embassy_executor::main)]
async fn main(_spawner: Spawner) {
    let p: embassy_stm32::Peripherals = init();

    // The SAES draws random numbers from the RNG whenever it is reset.
    let _rng = Rng::new(p.RNG, Irqs);
    let saes = Saes::new_blocking(p.SAES, Irqs);
    critical_section::with(|cs| DRIVER.borrow_ref_mut(cs).replace(saes));
    suites!(
        aes128_ecb = embassy_crypto_test::aes_ecb::<Ecb<16>>(&vectors::AES_ECB_128),
        aes256_ecb = embassy_crypto_test::aes_ecb::<Ecb<32>>(&vectors::AES_ECB_256),
        aes128_cbc = embassy_crypto_test::aes_cbc::<Cbc<16>, Cbc<16>>(&vectors::AES_CBC_128),
        aes256_cbc = embassy_crypto_test::aes_cbc::<Cbc<32>, Cbc<32>>(&vectors::AES_CBC_256),
    );
    #[cfg(feature = "saes-full")]
    suites!(
        aes128_ctr = embassy_crypto_test::aes_ctr::<Ctr<16>>(&vectors::AES_CTR_128),
        aes256_ctr = embassy_crypto_test::aes_ctr::<Ctr<32>>(&vectors::AES_CTR_256),
        aes128_gcm = embassy_crypto_test::aes_gcm::<Gcm<16>>(&vectors::AES_GCM_128),
        aes256_gcm = embassy_crypto_test::aes_gcm::<Gcm<32>>(&vectors::AES_GCM_256),
        aes128_ccm = embassy_crypto_test::aes_ccm::<Ccm<16>>(&vectors::AES_CCM_128),
        aes256_ccm = embassy_crypto_test::aes_ccm::<Ccm<32>>(&vectors::AES_CCM_256),
    );

    info!("Test OK");
    cortex_m::asm::bkpt();
}
