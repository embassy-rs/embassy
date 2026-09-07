//! Known-answer test suites for [`embassy-crypto`](embassy_crypto) drivers.
//!
//! Every suite here drives the public `embassy_crypto` API, so whichever driver
//! an application links, software or hardware, is what gets tested: a host
//! test and a hardware-in-the-loop binary call the same functions.
//!
//! The vectors come from [Wycheproof](https://github.com/C2SP/wycheproof)
//! where it has a file for the algorithm, and are generated at build time with
//! the RustCrypto crates otherwise (plain digests, AES-ECB, AES-CTR, curve
//! arithmetic, X25519 key generation). See `build.rs`.
//!
//! Each suite returns an [`Outcome`]: the number of cases that passed and were
//! skipped, or the first failure with the Wycheproof `tcId` (or the index into
//! the generated table) so it can be looked up upstream. A case is skipped when
//! the API cannot express its parameters, such as a GCM nonce that is not 12
//! bytes.
//!
//! ```ignore
//! use embassy_crypto_rustcrypto as _; // or a HAL that registers drivers
//!
//! embassy_crypto_tests::sha256().unwrap();
//! embassy_crypto_tests::aes128_gcm().unwrap();
//! ```

#![no_std]
#![warn(missing_docs)]

use embassy_crypto::{Error, Rng};

pub mod vectors;
use vectors::*;

// =============================================================================
// Results
// =============================================================================

/// Counts of a suite that ran to completion.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Stats {
    /// Cases whose outcome matched the expectation.
    pub passed: u32,
    /// Cases the API could not express.
    pub skipped: u32,
}

/// The first case of a suite whose outcome did not match the expectation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Failure {
    /// The suite, named after its Wycheproof file or generated table.
    pub suite: &'static str,
    /// The Wycheproof `tcId`, or the index for a generated table.
    pub tc_id: u32,
    /// What went wrong.
    pub what: &'static str,
}

/// The result of running a suite.
pub type Outcome = Result<Stats, Failure>;

/// What a single case did.
enum Verdict {
    Pass,
    Skip,
}

type CaseResult = Result<Verdict, &'static str>;

fn run<T>(suite: &Suite<T>, tc_id: impl Fn(usize, &T) -> u32, mut f: impl FnMut(&T) -> CaseResult) -> Outcome {
    let mut stats = Stats::default();
    for (i, case) in suite.cases.iter().enumerate() {
        match f(case) {
            Ok(Verdict::Pass) => stats.passed += 1,
            Ok(Verdict::Skip) => stats.skipped += 1,
            Err(what) => {
                return Err(Failure {
                    suite: suite.name,
                    tc_id: tc_id(i, case),
                    what,
                });
            }
        }
    }
    Ok(stats)
}

/// Whether an operation that was accepted (with the right output) or rejected
/// matches the expectation.
fn judge(expected: Expected, accepted: bool) -> CaseResult {
    match (expected, accepted) {
        (Expected::Valid, false) => Err("valid case rejected"),
        (Expected::Invalid, true) => Err("invalid case accepted"),
        _ => Ok(Verdict::Pass),
    }
}

/// Deterministic xorshift generator: the suites need arbitrary values, not secure ones.
struct TestRng(u64);

impl TestRng {
    fn new() -> Self {
        Self(0x9E37_79B9_7F4A_7C15)
    }
}

impl Rng for TestRng {
    fn fill_bytes(&mut self, buf: &mut [u8]) -> Result<(), Error> {
        for b in buf {
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 7;
            self.0 ^= self.0 << 17;
            *b = self.0 as u8;
        }
        Ok(())
    }
}

/// The message all generated vectors are computed over.
const MAX_MSG: usize = 2048;

// =============================================================================
// Digests
// =============================================================================

/// A digest type of `embassy_crypto`.
pub trait Digest: Clone {
    /// Size of the output.
    const OUTPUT_SIZE: usize;
    /// See `embassy_crypto::Sha256::new`.
    fn new() -> Self;
    /// See `embassy_crypto::Sha256::update`.
    fn update(&mut self, data: &[u8]);
    /// `finalize`, written to `out` (of `OUTPUT_SIZE` bytes).
    fn finalize_into(self, out: &mut [u8]);
}

macro_rules! impl_digest {
    ($($t:ident),* $(,)?) => {$(
        impl Digest for embassy_crypto::$t {
            const OUTPUT_SIZE: usize = embassy_crypto::$t::OUTPUT_SIZE;
            fn new() -> Self {
                embassy_crypto::$t::new()
            }
            fn update(&mut self, data: &[u8]) {
                embassy_crypto::$t::update(self, data)
            }
            fn finalize_into(self, out: &mut [u8]) {
                out.copy_from_slice(&embassy_crypto::$t::finalize(self))
            }
        }
    )*};
}
impl_digest!(Md5, Sha1, Sha224, Sha256, Sha384, Sha512, Sha512_224, Sha512_256);

/// Run a digest suite: one-shot, in odd-sized chunks with a second hasher of
/// the same type interleaved, and with the state cloned mid-way.
pub fn digest<D: Digest>(suite: &Suite<vectors::Digest>) -> Outcome {
    run(
        suite,
        |i, _| i as u32,
        |case| {
            let msg = &MESSAGE[..case.len];
            let mut out = [0u8; 64];
            let out = &mut out[..D::OUTPUT_SIZE];

            let mut h = D::new();
            h.update(msg);
            h.finalize_into(out);
            if out != case.digest {
                return Err("one-shot digest mismatch");
            }

            let mut a = D::new();
            let mut b = D::new();
            for (i, chunk) in msg.chunks(37).enumerate() {
                if i % 2 == 0 {
                    a.update(chunk);
                    b.update(chunk);
                } else {
                    b.update(chunk);
                    a.update(chunk);
                }
            }
            let fork = a.clone();
            a.update(b"more");
            fork.finalize_into(out);
            if out != case.digest {
                return Err("cloned state digest mismatch");
            }
            b.finalize_into(out);
            if out != case.digest {
                return Err("interleaved digest mismatch");
            }
            a.finalize_into(out);
            if out == case.digest {
                return Err("digest unchanged by extra data");
            }
            Ok(Verdict::Pass)
        },
    )
}

// =============================================================================
// MACs
// =============================================================================

/// An HMAC or CMAC type of `embassy_crypto`.
pub trait Mac: Clone {
    /// Size of the tag.
    const OUTPUT_SIZE: usize;
    /// See `embassy_crypto::HmacSha256::new`. `None` if the key size is not accepted.
    fn new(key: &[u8]) -> Option<Self>;
    /// See `embassy_crypto::HmacSha256::update`.
    fn update(&mut self, data: &[u8]);
    /// `finalize`, written to `out` (of `OUTPUT_SIZE` bytes).
    fn finalize_into(self, out: &mut [u8]);
    /// See `embassy_crypto::HmacSha256::verify`.
    fn verify(self, tag: &[u8]) -> Result<(), Error>;
}

macro_rules! impl_hmac {
    ($($t:ident),* $(,)?) => {$(
        impl Mac for embassy_crypto::$t {
            const OUTPUT_SIZE: usize = embassy_crypto::$t::OUTPUT_SIZE;
            fn new(key: &[u8]) -> Option<Self> {
                Some(embassy_crypto::$t::new(key))
            }
            fn update(&mut self, data: &[u8]) {
                embassy_crypto::$t::update(self, data)
            }
            fn finalize_into(self, out: &mut [u8]) {
                out.copy_from_slice(&embassy_crypto::$t::finalize(self))
            }
            fn verify(self, tag: &[u8]) -> Result<(), Error> {
                embassy_crypto::$t::verify(self, tag)
            }
        }
    )*};
}
impl_hmac!(
    HmacSha1,
    HmacSha224,
    HmacSha256,
    HmacSha384,
    HmacSha512,
    HmacSha512_224,
    HmacSha512_256
);

macro_rules! impl_cmac {
    ($($t:ident),* $(,)?) => {$(
        impl Mac for embassy_crypto::$t {
            const OUTPUT_SIZE: usize = embassy_crypto::$t::OUTPUT_SIZE;
            fn new(key: &[u8]) -> Option<Self> {
                Some(embassy_crypto::$t::new(key.try_into().ok()?))
            }
            fn update(&mut self, data: &[u8]) {
                embassy_crypto::$t::update(self, data)
            }
            fn finalize_into(self, out: &mut [u8]) {
                out.copy_from_slice(&embassy_crypto::$t::finalize(self))
            }
            fn verify(self, tag: &[u8]) -> Result<(), Error> {
                embassy_crypto::$t::verify(self, tag)
            }
        }
    )*};
}
impl_cmac!(Aes128Cmac, Aes256Cmac);

/// Run a MAC suite. Tags in the vectors may be truncated; they are compared
/// against the prefix of the computed tag and through `verify`.
pub fn mac<M: Mac>(suite: &Suite<vectors::Mac>) -> Outcome {
    run(
        suite,
        |_, c| c.tc_id,
        |case| {
            let Some(m) = M::new(case.key) else {
                return judge(case.result, false);
            };
            let mut out = [0u8; 64];
            let out = &mut out[..M::OUTPUT_SIZE];

            let mut one_shot = m.clone();
            one_shot.update(case.msg);
            one_shot.finalize_into(out);
            let tag_matches = case.tag.len() <= out.len() && out[..case.tag.len()] == *case.tag;

            let mut chunked = m.clone();
            for chunk in case.msg.chunks(13) {
                chunked.update(chunk);
            }
            let verified = chunked.verify(case.tag).is_ok();

            if tag_matches != verified {
                return Err("finalize and verify disagree");
            }
            judge(case.result, verified)
        },
    )
}

// =============================================================================
// AES
// =============================================================================

/// A raw block cipher type of `embassy_crypto`.
pub trait BlockCipher {
    /// Size of the key.
    const KEY_SIZE: usize;
    /// See `embassy_crypto::Aes128::new`. `None` if the key size is not accepted.
    fn new(key: &[u8]) -> Option<Self>
    where
        Self: Sized;
    /// See `embassy_crypto::Aes128::encrypt_blocks`.
    fn encrypt_blocks(&self, blocks: &mut [u8]) -> Result<(), Error>;
    /// See `embassy_crypto::Aes128::decrypt_blocks`.
    fn decrypt_blocks(&self, blocks: &mut [u8]) -> Result<(), Error>;
    /// See `embassy_crypto::Aes128::encrypt_blocks_to`.
    fn encrypt_blocks_to(&self, input: &[u8], output: &mut [u8]) -> Result<(), Error>;
    /// See `embassy_crypto::Aes128::decrypt_blocks_to`.
    fn decrypt_blocks_to(&self, input: &[u8], output: &mut [u8]) -> Result<(), Error>;
}

macro_rules! impl_block_cipher {
    ($($t:ident),* $(,)?) => {$(
        impl BlockCipher for embassy_crypto::$t {
            const KEY_SIZE: usize = embassy_crypto::$t::KEY_SIZE;
            fn new(key: &[u8]) -> Option<Self> {
                Some(embassy_crypto::$t::new(key.try_into().ok()?))
            }
            fn encrypt_blocks(&self, blocks: &mut [u8]) -> Result<(), Error> {
                embassy_crypto::$t::encrypt_blocks(self, blocks)
            }
            fn decrypt_blocks(&self, blocks: &mut [u8]) -> Result<(), Error> {
                embassy_crypto::$t::decrypt_blocks(self, blocks)
            }
            fn encrypt_blocks_to(&self, input: &[u8], output: &mut [u8]) -> Result<(), Error> {
                embassy_crypto::$t::encrypt_blocks_to(self, input, output)
            }
            fn decrypt_blocks_to(&self, input: &[u8], output: &mut [u8]) -> Result<(), Error> {
                embassy_crypto::$t::decrypt_blocks_to(self, input, output)
            }
        }
    )*};
}
impl_block_cipher!(Aes128, Aes256);

/// Run an ECB suite: in place and to a separate buffer, both directions, plus
/// rejection of a partial block.
pub fn aes_ecb<C: BlockCipher>(suite: &Suite<Ecb>) -> Outcome {
    run(
        suite,
        |i, _| i as u32,
        |case| {
            let aes = C::new(case.key).ok_or("key rejected")?;
            let pt = &MESSAGE[..case.pt_len];
            let mut buf = [0u8; MAX_MSG];
            let buf = &mut buf[..case.pt_len];
            let mut out = [0u8; MAX_MSG];
            let out = &mut out[..case.pt_len];

            buf.copy_from_slice(pt);
            aes.encrypt_blocks(buf).map_err(|_| "encrypt failed")?;
            if buf != case.ct {
                return Err("ciphertext mismatch");
            }
            aes.decrypt_blocks(buf).map_err(|_| "decrypt failed")?;
            if buf != pt {
                return Err("decrypted plaintext mismatch");
            }

            aes.encrypt_blocks_to(pt, out).map_err(|_| "encrypt_to failed")?;
            if out != case.ct {
                return Err("ciphertext mismatch (separate buffers)");
            }
            aes.decrypt_blocks_to(case.ct, out).map_err(|_| "decrypt_to failed")?;
            if out != pt {
                return Err("decrypted plaintext mismatch (separate buffers)");
            }

            if aes.encrypt_blocks(&mut buf[..15]) != Err(Error::InvalidInput) {
                return Err("partial block accepted");
            }
            if aes.encrypt_blocks_to(&buf[..16], out) != Err(Error::InvalidInput) {
                return Err("mismatched buffer lengths accepted");
            }
            Ok(Verdict::Pass)
        },
    )
}

/// A CBC encryptor type of `embassy_crypto`.
pub trait CbcEncrypt {
    /// See `embassy_crypto::Aes128CbcEncrypt::new`. `None` if the key size is not accepted.
    fn new(key: &[u8], iv: &[u8; 16]) -> Option<Self>
    where
        Self: Sized;
    /// See `embassy_crypto::Aes128CbcEncrypt::encrypt`.
    fn encrypt(&mut self, blocks: &mut [u8]) -> Result<(), Error>;
    /// See `embassy_crypto::Aes128CbcEncrypt::encrypt_to`.
    fn encrypt_to(&mut self, input: &[u8], output: &mut [u8]) -> Result<(), Error>;
}

/// A CBC decryptor type of `embassy_crypto`.
pub trait CbcDecrypt {
    /// See `embassy_crypto::Aes128CbcDecrypt::new`. `None` if the key size is not accepted.
    fn new(key: &[u8], iv: &[u8; 16]) -> Option<Self>
    where
        Self: Sized;
    /// See `embassy_crypto::Aes128CbcDecrypt::decrypt`.
    fn decrypt(&mut self, blocks: &mut [u8]) -> Result<(), Error>;
    /// See `embassy_crypto::Aes128CbcDecrypt::decrypt_to`.
    fn decrypt_to(&mut self, input: &[u8], output: &mut [u8]) -> Result<(), Error>;
}

macro_rules! impl_cbc {
    ($($enc:ident / $dec:ident),* $(,)?) => {$(
        impl CbcEncrypt for embassy_crypto::$enc {
            fn new(key: &[u8], iv: &[u8; 16]) -> Option<Self> {
                Some(embassy_crypto::$enc::new(key.try_into().ok()?, iv))
            }
            fn encrypt(&mut self, blocks: &mut [u8]) -> Result<(), Error> {
                embassy_crypto::$enc::encrypt(self, blocks)
            }
            fn encrypt_to(&mut self, input: &[u8], output: &mut [u8]) -> Result<(), Error> {
                embassy_crypto::$enc::encrypt_to(self, input, output)
            }
        }
        impl CbcDecrypt for embassy_crypto::$dec {
            fn new(key: &[u8], iv: &[u8; 16]) -> Option<Self> {
                Some(embassy_crypto::$dec::new(key.try_into().ok()?, iv))
            }
            fn decrypt(&mut self, blocks: &mut [u8]) -> Result<(), Error> {
                embassy_crypto::$dec::decrypt(self, blocks)
            }
            fn decrypt_to(&mut self, input: &[u8], output: &mut [u8]) -> Result<(), Error> {
                embassy_crypto::$dec::decrypt_to(self, input, output)
            }
        }
    )*};
}
impl_cbc!(Aes128CbcEncrypt / Aes128CbcDecrypt, Aes256CbcEncrypt / Aes256CbcDecrypt);

/// Length of the PKCS#7 payload of `buf`, or `None` if the padding is malformed.
fn unpad(buf: &[u8]) -> Option<usize> {
    let &last = buf.last()?;
    let n = last as usize;
    if n == 0 || n > 16 || n > buf.len() || buf[buf.len() - n..].iter().any(|&b| b != last) {
        return None;
    }
    Some(buf.len() - n)
}

/// Run a CBC suite. The vectors are PKCS#7 padded, and `embassy_crypto` does
/// not pad, so the padding is applied and checked here: invalid cases are
/// ciphertexts whose padding must not check out.
pub fn aes_cbc<E: CbcEncrypt, D: CbcDecrypt>(suite: &Suite<Cbc>) -> Outcome {
    const MAX: usize = 256;
    run(
        suite,
        |_, c| c.tc_id,
        |case| {
            let Ok(iv) = <&[u8; 16]>::try_from(case.iv) else {
                return judge(case.result, false);
            };
            let (Some(mut enc), Some(mut dec)) = (E::new(case.key, iv), D::new(case.key, iv)) else {
                return judge(case.result, false);
            };
            if case.msg.len() + 16 > MAX || case.ct.len() > MAX {
                return Ok(Verdict::Skip);
            }

            // Decrypt the ciphertext and check the padding: every case has a ciphertext.
            let mut buf = [0u8; MAX];
            let buf = &mut buf[..case.ct.len()];
            buf.copy_from_slice(case.ct);
            let accepted = match dec.decrypt(buf) {
                Ok(()) => unpad(buf).is_some_and(|n| buf[..n] == *case.msg),
                Err(_) => false,
            };
            if case.result != Expected::Valid {
                return judge(case.result, accepted);
            }
            if !accepted {
                return Err("valid ciphertext rejected");
            }

            // Encrypt the padded message, chaining across calls.
            let pad = 16 - case.msg.len() % 16;
            let len = case.msg.len() + pad;
            let mut buf = [0u8; MAX];
            let buf = &mut buf[..len];
            buf[..case.msg.len()].copy_from_slice(case.msg);
            buf[case.msg.len()..].fill(pad as u8);
            let padded = {
                let mut p = [0u8; MAX];
                p[..len].copy_from_slice(buf);
                p
            };
            let padded = &padded[..len];
            enc.encrypt(&mut buf[..16]).map_err(|_| "encrypt failed")?;
            enc.encrypt(&mut buf[16..]).map_err(|_| "encrypt failed")?;
            if buf != case.ct {
                return Err("ciphertext mismatch");
            }

            // Separate buffers, both directions, chaining across calls.
            let mut out = [0u8; MAX];
            let out = &mut out[..len];
            let mut enc = E::new(case.key, iv).unwrap();
            let split = if len > 16 { 16 } else { 0 };
            enc.encrypt_to(&padded[..split], &mut out[..split])
                .map_err(|_| "encrypt_to failed")?;
            enc.encrypt_to(&padded[split..], &mut out[split..])
                .map_err(|_| "encrypt_to failed")?;
            if out != case.ct {
                return Err("ciphertext mismatch (separate buffers)");
            }
            let mut dec = D::new(case.key, iv).unwrap();
            dec.decrypt_to(&case.ct[..split], &mut out[..split])
                .map_err(|_| "decrypt_to failed")?;
            dec.decrypt_to(&case.ct[split..], &mut out[split..])
                .map_err(|_| "decrypt_to failed")?;
            if out != &padded[..len] {
                return Err("plaintext mismatch (separate buffers)");
            }
            Ok(Verdict::Pass)
        },
    )
}

/// A CTR type of `embassy_crypto`.
pub trait Ctr {
    /// See `embassy_crypto::Aes128Ctr::new`. `None` if the key size is not accepted.
    fn new(key: &[u8], iv: &[u8; 16]) -> Option<Self>
    where
        Self: Sized;
    /// See `embassy_crypto::Aes128Ctr::apply_keystream`.
    fn apply_keystream(&mut self, buf: &mut [u8]);
    /// See `embassy_crypto::Aes128Ctr::apply_keystream_to`.
    fn apply_keystream_to(&mut self, input: &[u8], output: &mut [u8]) -> Result<(), Error>;
}

macro_rules! impl_ctr {
    ($($t:ident),* $(,)?) => {$(
        impl Ctr for embassy_crypto::$t {
            fn new(key: &[u8], iv: &[u8; 16]) -> Option<Self> {
                Some(embassy_crypto::$t::new(key.try_into().ok()?, iv))
            }
            fn apply_keystream(&mut self, buf: &mut [u8]) {
                embassy_crypto::$t::apply_keystream(self, buf)
            }
            fn apply_keystream_to(&mut self, input: &[u8], output: &mut [u8]) -> Result<(), Error> {
                embassy_crypto::$t::apply_keystream_to(self, input, output)
            }
        }
    )*};
}
impl_ctr!(Aes128Ctr, Aes256Ctr);

/// Run a CTR suite: the keystream applied in odd-sized chunks and in one go,
/// in place and to a separate buffer.
pub fn aes_ctr<C: Ctr>(suite: &Suite<vectors::Ctr>) -> Outcome {
    run(
        suite,
        |i, _| i as u32,
        |case| {
            let iv: &[u8; 16] = case.iv.try_into().map_err(|_| "bad iv length")?;
            let pt = &MESSAGE[..case.pt_len];
            let mut buf = [0u8; MAX_MSG];
            let buf = &mut buf[..case.pt_len];

            buf.copy_from_slice(pt);
            let mut ctr = C::new(case.key, iv).ok_or("key rejected")?;
            let mut pos = 0;
            for len in [1usize, 7, 16, 20, 3, 17, 100, 33] {
                if pos + len > buf.len() {
                    break;
                }
                ctr.apply_keystream(&mut buf[pos..pos + len]);
                pos += len;
            }
            ctr.apply_keystream(&mut buf[pos..]);
            if buf != case.ct {
                return Err("ciphertext mismatch (chunked)");
            }
            C::new(case.key, iv).unwrap().apply_keystream(buf);
            if buf != pt {
                return Err("decrypted plaintext mismatch");
            }

            let mut out = [0u8; MAX_MSG];
            let out = &mut out[..case.pt_len];
            let mut ctr = C::new(case.key, iv).unwrap();
            ctr.apply_keystream_to(&pt[..5], &mut out[..5])
                .map_err(|_| "apply_keystream_to failed")?;
            ctr.apply_keystream_to(&pt[5..], &mut out[5..])
                .map_err(|_| "apply_keystream_to failed")?;
            if out != case.ct {
                return Err("ciphertext mismatch (separate buffers)");
            }
            if ctr.apply_keystream_to(&pt[..16], &mut out[..15]) != Err(Error::InvalidInput) {
                return Err("mismatched buffer lengths accepted");
            }
            Ok(Verdict::Pass)
        },
    )
}

/// A GCM type of `embassy_crypto`.
pub trait Gcm {
    /// See `embassy_crypto::Aes128Gcm::new`. `None` if the key size is not accepted.
    fn new(key: &[u8]) -> Option<Self>
    where
        Self: Sized;
    /// See `embassy_crypto::Aes128Gcm::encrypt`.
    fn encrypt(&self, nonce: &[u8; 12], aad: &[u8], buf: &mut [u8]) -> Result<[u8; 16], Error>;
    /// See `embassy_crypto::Aes128Gcm::encrypt_to`.
    fn encrypt_to(&self, nonce: &[u8; 12], aad: &[u8], input: &[u8], output: &mut [u8]) -> Result<[u8; 16], Error>;
    /// See `embassy_crypto::Aes128Gcm::decrypt`.
    fn decrypt(&self, nonce: &[u8; 12], aad: &[u8], buf: &mut [u8], tag: &[u8; 16]) -> Result<(), Error>;
    /// See `embassy_crypto::Aes128Gcm::decrypt_to`.
    fn decrypt_to(
        &self,
        nonce: &[u8; 12],
        aad: &[u8],
        input: &[u8],
        output: &mut [u8],
        tag: &[u8; 16],
    ) -> Result<(), Error>;
}

macro_rules! impl_gcm {
    ($($t:ident),* $(,)?) => {$(
        impl Gcm for embassy_crypto::$t {
            fn new(key: &[u8]) -> Option<Self> {
                Some(embassy_crypto::$t::new(key.try_into().ok()?))
            }
            fn encrypt(&self, nonce: &[u8; 12], aad: &[u8], buf: &mut [u8]) -> Result<[u8; 16], Error> {
                embassy_crypto::$t::encrypt(self, nonce, aad, buf)
            }
            fn encrypt_to(&self, nonce: &[u8; 12], aad: &[u8], input: &[u8], output: &mut [u8]) -> Result<[u8; 16], Error> {
                embassy_crypto::$t::encrypt_to(self, nonce, aad, input, output)
            }
            fn decrypt(&self, nonce: &[u8; 12], aad: &[u8], buf: &mut [u8], tag: &[u8; 16]) -> Result<(), Error> {
                embassy_crypto::$t::decrypt(self, nonce, aad, buf, tag)
            }
            fn decrypt_to(&self, nonce: &[u8; 12], aad: &[u8], input: &[u8], output: &mut [u8], tag: &[u8; 16]) -> Result<(), Error> {
                embassy_crypto::$t::decrypt_to(self, nonce, aad, input, output, tag)
            }
        }
    )*};
}
impl_gcm!(Aes128Gcm, Aes256Gcm);

/// Run a GCM suite. Cases with a nonce that is not 12 bytes are skipped.
pub fn aes_gcm<G: Gcm>(suite: &Suite<Aead>) -> Outcome {
    const MAX: usize = 1024;
    run(
        suite,
        |_, c| c.tc_id,
        |case| {
            let gcm = G::new(case.key).ok_or("key rejected")?;
            let Ok(nonce) = <&[u8; 12]>::try_from(case.nonce) else {
                return Ok(Verdict::Skip);
            };
            let Ok(tag) = <&[u8; 16]>::try_from(case.tag) else {
                return Ok(Verdict::Skip);
            };
            if case.msg.len() > MAX {
                return Ok(Verdict::Skip);
            }
            let mut buf = [0u8; MAX];
            let buf = &mut buf[..case.ct.len()];

            // Every case: decrypt, in place and to a separate buffer.
            buf.copy_from_slice(case.ct);
            let accepted = gcm.decrypt(nonce, case.aad, buf, tag).is_ok() && buf == case.msg;
            let mut out = [0u8; MAX];
            let out = &mut out[..case.ct.len()];
            let accepted_to = gcm.decrypt_to(nonce, case.aad, case.ct, out, tag).is_ok() && out == case.msg;
            if accepted != accepted_to {
                return Err("in-place and separate-buffer decrypt disagree");
            }
            if case.result != Expected::Valid {
                return judge(case.result, accepted);
            }
            if !accepted {
                return Err("valid ciphertext rejected");
            }

            // Valid cases: encrypt too.
            buf.copy_from_slice(case.msg);
            let t = gcm.encrypt(nonce, case.aad, buf).map_err(|_| "encrypt failed")?;
            if buf != case.ct || t != *tag {
                return Err("ciphertext or tag mismatch");
            }
            let t = gcm
                .encrypt_to(nonce, case.aad, case.msg, out)
                .map_err(|_| "encrypt_to failed")?;
            if out != case.ct || t != *tag {
                return Err("ciphertext or tag mismatch (separate buffers)");
            }
            let mut bad = *tag;
            bad[0] ^= 1;
            buf.copy_from_slice(case.ct);
            if gcm.decrypt(nonce, case.aad, buf, &bad) != Err(Error::InvalidSignature) {
                return Err("modified tag accepted");
            }
            Ok(Verdict::Pass)
        },
    )
}

/// A CCM type of `embassy_crypto`.
pub trait Ccm {
    /// See `embassy_crypto::Aes128Ccm::new`. `None` if the key size is not accepted.
    fn new(key: &[u8]) -> Option<Self>
    where
        Self: Sized;
    /// See `embassy_crypto::Aes128Ccm::encrypt`.
    fn encrypt(&self, nonce: &[u8], aad: &[u8], buf: &mut [u8], tag: &mut [u8]) -> Result<(), Error>;
    /// See `embassy_crypto::Aes128Ccm::encrypt_to`.
    fn encrypt_to(
        &self,
        nonce: &[u8],
        aad: &[u8],
        input: &[u8],
        output: &mut [u8],
        tag: &mut [u8],
    ) -> Result<(), Error>;
    /// See `embassy_crypto::Aes128Ccm::decrypt`.
    fn decrypt(&self, nonce: &[u8], aad: &[u8], buf: &mut [u8], tag: &[u8]) -> Result<(), Error>;
    /// See `embassy_crypto::Aes128Ccm::decrypt_to`.
    fn decrypt_to(&self, nonce: &[u8], aad: &[u8], input: &[u8], output: &mut [u8], tag: &[u8]) -> Result<(), Error>;
}

macro_rules! impl_ccm {
    ($($t:ident),* $(,)?) => {$(
        impl Ccm for embassy_crypto::$t {
            fn new(key: &[u8]) -> Option<Self> {
                Some(embassy_crypto::$t::new(key.try_into().ok()?))
            }
            fn encrypt(&self, nonce: &[u8], aad: &[u8], buf: &mut [u8], tag: &mut [u8]) -> Result<(), Error> {
                embassy_crypto::$t::encrypt(self, nonce, aad, buf, tag)
            }
            fn encrypt_to(&self, nonce: &[u8], aad: &[u8], input: &[u8], output: &mut [u8], tag: &mut [u8]) -> Result<(), Error> {
                embassy_crypto::$t::encrypt_to(self, nonce, aad, input, output, tag)
            }
            fn decrypt(&self, nonce: &[u8], aad: &[u8], buf: &mut [u8], tag: &[u8]) -> Result<(), Error> {
                embassy_crypto::$t::decrypt(self, nonce, aad, buf, tag)
            }
            fn decrypt_to(&self, nonce: &[u8], aad: &[u8], input: &[u8], output: &mut [u8], tag: &[u8]) -> Result<(), Error> {
                embassy_crypto::$t::decrypt_to(self, nonce, aad, input, output, tag)
            }
        }
    )*};
}
impl_ccm!(Aes128Ccm, Aes256Ccm);

/// Run a CCM suite. Invalid nonce and tag sizes are part of the vectors: the
/// driver must reject them.
pub fn aes_ccm<C: Ccm>(suite: &Suite<Aead>) -> Outcome {
    const MAX: usize = 1024;
    run(
        suite,
        |_, c| c.tc_id,
        |case| {
            let ccm = C::new(case.key).ok_or("key rejected")?;
            if case.msg.len() > MAX || case.tag.len() > 16 {
                return Ok(Verdict::Skip);
            }
            let mut buf = [0u8; MAX];
            let buf = &mut buf[..case.ct.len()];
            let mut out = [0u8; MAX];
            let out = &mut out[..case.ct.len()];

            buf.copy_from_slice(case.ct);
            let accepted = ccm.decrypt(case.nonce, case.aad, buf, case.tag).is_ok() && buf == case.msg;
            let accepted_to = ccm.decrypt_to(case.nonce, case.aad, case.ct, out, case.tag).is_ok() && out == case.msg;
            if accepted != accepted_to {
                return Err("in-place and separate-buffer decrypt disagree");
            }
            if case.result != Expected::Valid {
                return judge(case.result, accepted);
            }
            if !accepted {
                return Err("valid ciphertext rejected");
            }

            let mut tag = [0u8; 16];
            let tag = &mut tag[..case.tag.len()];
            buf.copy_from_slice(case.msg);
            ccm.encrypt(case.nonce, case.aad, buf, tag)
                .map_err(|_| "encrypt failed")?;
            if buf != case.ct || tag != case.tag {
                return Err("ciphertext or tag mismatch");
            }
            tag.fill(0);
            ccm.encrypt_to(case.nonce, case.aad, case.msg, out, tag)
                .map_err(|_| "encrypt_to failed")?;
            if out != case.ct || tag != case.tag {
                return Err("ciphertext or tag mismatch (separate buffers)");
            }
            tag[0] ^= 1;
            buf.copy_from_slice(case.ct);
            if ccm.decrypt(case.nonce, case.aad, buf, tag) != Err(Error::InvalidSignature) {
                return Err("modified tag accepted");
            }
            Ok(Verdict::Pass)
        },
    )
}

// =============================================================================
// Curves
// =============================================================================

/// Left-align a big-endian integer of any length into `N` bytes, or `None` if
/// it does not fit.
fn fixed<const N: usize>(bytes: &[u8]) -> Option<[u8; N]> {
    let start = bytes.iter().position(|&b| b != 0).unwrap_or(bytes.len());
    let bytes = &bytes[start..];
    if bytes.len() > N {
        return None;
    }
    let mut out = [0u8; N];
    out[N - bytes.len()..].copy_from_slice(bytes);
    Some(out)
}

macro_rules! curve_suites {
    ($mod:ident, $n:literal, $doc:literal) => {
        #[doc = $doc]
        pub mod $mod {
            use embassy_crypto::Error;
            use embassy_crypto::$mod::{Point, PublicKey, Scalar, SecretKey, Signature, SigningKey, VerifyingKey};

            use super::*;

            const N: usize = $n;

            fn point(bytes: &[u8]) -> Result<Option<Point>, &'static str> {
                if bytes.is_empty() {
                    return Ok(None);
                }
                let sec1: &[u8; 2 * N + 1] = bytes.try_into().map_err(|_| "bad point length in vector")?;
                Point::from_sec1(sec1)
                    .map(Some)
                    .map_err(|_| "vector point rejected")
            }

            fn scalar(bytes: &[u8]) -> Result<Scalar, &'static str> {
                let bytes: &[u8; N] = bytes.try_into().map_err(|_| "bad scalar length in vector")?;
                Scalar::from_bytes(bytes).map_err(|_| "vector scalar rejected")
            }

            /// Run the arithmetic suite: scalar field operations, point
            /// multiplication and addition including results at infinity, and
            /// the edge cases of parsing.
            pub fn arith(suite: &Suite<EcArith>) -> Outcome {
                run(
                    suite,
                    |i, _| i as u32,
                    |case| {
                        let a = scalar(case.a)?;
                        let b = scalar(case.b)?;
                        if a.add(&b) != scalar(case.a_plus_b)? {
                            return Err("scalar add mismatch");
                        }
                        if a.sub(&b) != scalar(case.a_minus_b)? {
                            return Err("scalar sub mismatch");
                        }
                        if a.mul(&b) != scalar(case.a_times_b)? {
                            return Err("scalar mul mismatch");
                        }
                        if a.invert() != Some(scalar(case.a_inv)?) {
                            return Err("scalar invert mismatch");
                        }
                        if a.to_bytes() != *case.a || a.as_bytes() != case.a {
                            return Err("scalar round trip mismatch");
                        }

                        let a_g = point(case.a_g)?;
                        let b_g = point(case.b_g)?;
                        if Point::mul_base(&a) != a_g {
                            return Err("base point multiplication mismatch");
                        }
                        if Point::GENERATOR.mul(&b) != b_g {
                            return Err("point multiplication mismatch");
                        }
                        let (Some(p), Some(q)) = (a_g, b_g) else {
                            return Err("vector point at infinity for a nonzero scalar");
                        };
                        if p.add(&q) != point(case.sum)? {
                            return Err("point addition mismatch");
                        }
                        if q.mul(&a) != point(case.a_times_b_g)? {
                            return Err("variable-base multiplication mismatch");
                        }
                        if Point::lincomb(&a, &Point::GENERATOR, &b, &q) != point(case.lincomb)? {
                            return Err("linear combination mismatch");
                        }
                        if p.to_sec1() != *case.a_g || Point::from_xy(p.x(), p.y()) != Ok(p) {
                            return Err("point round trip mismatch");
                        }
                        Ok(Verdict::Pass)
                    },
                )?;

                // Parsing and identities that need no vector.
                let suite_name = suite.name;
                let check = |ok: bool, what: &'static str| {
                    if ok {
                        Ok(())
                    } else {
                        Err(Failure {
                            suite: suite_name,
                            tc_id: u32::MAX,
                            what,
                        })
                    }
                };
                let g = Point::GENERATOR;
                check(Point::from_xy(g.x(), g.y()) == Ok(g), "generator not on curve")?;
                let mut y = *g.y();
                y[N - 1] ^= 1;
                check(
                    Point::from_xy(g.x(), &y) == Err(Error::InvalidKey),
                    "off-curve point accepted",
                )?;
                let mut sec1 = g.to_sec1();
                sec1[0] = 0x02;
                check(Point::from_sec1(&sec1).is_err(), "compressed point tag accepted")?;
                check(
                    Scalar::from_bytes(&embassy_crypto::$mod::ORDER) == Err(Error::InvalidKey),
                    "scalar equal to the order accepted",
                )?;
                check(
                    Scalar::from_bytes(&[0xff; N]).is_err(),
                    "scalar above the order accepted",
                )?;
                check(Scalar::ZERO.invert().is_none(), "zero inverted")?;
                check(Point::mul_base(&Scalar::ZERO).is_none(), "0 * G is not infinity")?;
                check(g.mul(&Scalar::ZERO).is_none(), "0 * P is not infinity")?;
                let one = Scalar::from_bytes(&{
                    let mut b = [0u8; N];
                    b[N - 1] = 1;
                    b
                })
                .unwrap();
                check(Point::mul_base(&one) == Some(g), "1 * G is not G")?;
                check(g.mul(&one) == Some(g), "1 * P is not P")?;
                let n_minus_1 = Scalar::ZERO.sub(&one);
                let minus_g = g.mul(&n_minus_1).ok_or(Failure {
                    suite: suite_name,
                    tc_id: u32::MAX,
                    what: "(n-1) * G is infinity",
                })?;
                check(g.add(&minus_g).is_none(), "G + (-G) is not infinity")?;
                check(g.add(&g) == Point::mul_base(&one.add(&one)), "G + G != 2G")?;
                check(
                    Point::lincomb(&one, &g, &n_minus_1, &g).is_none(),
                    "G - G through lincomb is not infinity",
                )?;
                Ok(Stats {
                    passed: suite.cases.len() as u32,
                    skipped: 0,
                })
            }

            /// Run the ECDH suite. Invalid cases are points off the curve, on
            /// the wrong curve or malformed, which must be rejected.
            pub fn ecdh(suite: &Suite<Dh>) -> Outcome {
                run(
                    suite,
                    |_, c| c.tc_id,
                    |case| {
                        let sk = fixed::<N>(case.private).and_then(|b| SecretKey::from_bytes(&b).ok());
                        let pk = <&[u8; 2 * N + 1]>::try_from(case.public)
                            .ok()
                            .and_then(|b| PublicKey::from_sec1(b).ok());
                        let (Some(sk), Some(pk)) = (sk, pk) else {
                            return judge(case.result, false);
                        };
                        let Ok(shared) = sk.diffie_hellman(&pk) else {
                            return judge(case.result, false);
                        };
                        if shared.as_bytes() != case.shared {
                            return Err("shared secret mismatch");
                        }
                        if case.result == Expected::Valid {
                            // The peer point validates, and so does our own public key.
                            Point::try_from(pk).map_err(|_| "valid peer point rejected")?;
                            let own = sk.public_key().map_err(|_| "public key derivation failed")?;
                            Point::try_from(own).map_err(|_| "derived public key not on curve")?;
                            if PublicKey::from_sec1(&own.to_sec1()) != Ok(own) {
                                return Err("public key round trip mismatch");
                            }
                            if SecretKey::from_bytes(&sk.to_bytes()).map(|k| k.to_bytes()) != Ok(sk.to_bytes()) {
                                return Err("secret key round trip mismatch");
                            }
                        }
                        judge(case.result, true)
                    },
                )
            }

            /// Run the ECDSA suite: verification against the vectors, plus a
            /// sign-and-verify round trip through the signing API.
            pub fn ecdsa(suite: &Suite<Ecdsa>) -> Outcome {
                let stats = run(
                    suite,
                    |_, c| c.tc_id,
                    |case| {
                        let vk = <&[u8; 2 * N + 1]>::try_from(case.public)
                            .ok()
                            .and_then(|b| VerifyingKey::from_sec1(b).ok())
                            .ok_or("vector public key rejected")?;
                        let digest: &[u8; N] = case
                            .digest
                            .try_into()
                            .map_err(|_| "bad digest length in vector")?;
                        let sig = <&[u8; 2 * N]>::try_from(case.sig)
                            .ok()
                            .and_then(|b| Signature::from_bytes(b).ok());
                        let Some(sig) = sig else {
                            return judge(case.result, false);
                        };
                        if sig.to_bytes() != *case.sig {
                            return Err("signature round trip mismatch");
                        }
                        judge(case.result, vk.verify_prehash(digest, &sig).is_ok())
                    },
                )?;

                let fail = |what| Failure {
                    suite: suite.name,
                    tc_id: u32::MAX,
                    what,
                };
                let mut rng = TestRng::new();
                let sk = SigningKey::generate(&mut rng).map_err(|_| fail("key generation failed"))?;
                let vk = sk
                    .verifying_key()
                    .map_err(|_| fail("verifying key derivation failed"))?;
                Point::try_from(vk).map_err(|_| fail("derived verifying key not on curve"))?;
                let digest: &[u8; N] = suite.cases[0].digest.try_into().unwrap();
                let sig = sk
                    .sign_prehash(digest, &mut rng)
                    .map_err(|_| fail("signing failed"))?;
                vk.verify_prehash(digest, &sig)
                    .map_err(|_| fail("own signature does not verify"))?;
                // Signatures are randomized: a second one differs and also verifies.
                let sig2 = sk
                    .sign_prehash(digest, &mut rng)
                    .map_err(|_| fail("signing failed"))?;
                if sig2 == sig {
                    return Err(fail("two signatures of the same digest are equal"));
                }
                vk.verify_prehash(digest, &sig2)
                    .map_err(|_| fail("own signature does not verify"))?;
                let mut other = *digest;
                other[0] ^= 1;
                if vk.verify_prehash(&other, &sig) != Err(Error::InvalidSignature) {
                    return Err(fail("signature verifies for another digest"));
                }
                let mut bytes = sig.to_bytes();
                bytes[N + 3] ^= 1;
                let bad = Signature::from_bytes(&bytes).map_err(|_| fail("modified signature unparseable"))?;
                if vk.verify_prehash(digest, &bad) != Err(Error::InvalidSignature) {
                    return Err(fail("modified signature verifies"));
                }
                bytes[..N].fill(0);
                if Signature::from_bytes(&bytes).is_ok() {
                    return Err(fail("zero r accepted"));
                }
                let mut sec1 = vk.to_sec1();
                sec1[2 * N] ^= 1;
                let bad_vk = VerifyingKey::from_sec1(&sec1).unwrap();
                if bad_vk.verify_prehash(digest, &sig) != Err(Error::InvalidKey) {
                    return Err(fail("off-curve verifying key accepted"));
                }
                Ok(stats)
            }
        }
    };
}

curve_suites!(p256, 32, "P-256 suites.");
curve_suites!(p384, 48, "P-384 suites.");

/// X25519 suites.
pub mod x25519 {
    use embassy_crypto::x25519::{PublicKey, SecretKey};

    use super::*;

    /// Run the Diffie-Hellman suite. Acceptable cases are low-order or
    /// non-canonical peer keys: they may be rejected, or must give the listed
    /// shared secret.
    pub fn dh(suite: &Suite<Dh>) -> Outcome {
        run(
            suite,
            |_, c| c.tc_id,
            |case| {
                let private: &[u8; 32] = case
                    .private
                    .try_into()
                    .map_err(|_| "bad private key length in vector")?;
                let public: &[u8; 32] = case.public.try_into().map_err(|_| "bad public key length in vector")?;
                let sk = SecretKey::from_bytes(private);
                let pk = PublicKey::from_bytes(public);
                let Ok(shared) = sk.diffie_hellman(&pk) else {
                    return judge(case.result, false);
                };
                if shared.as_bytes() != case.shared {
                    return Err("shared secret mismatch");
                }
                judge(case.result, true)
            },
        )
    }

    /// Run the key generation suite, plus a two-party agreement.
    pub fn keygen(suite: &Suite<KeyPair>) -> Outcome {
        let stats = run(
            suite,
            |i, _| i as u32,
            |case| {
                let private: &[u8; 32] = case
                    .private
                    .try_into()
                    .map_err(|_| "bad private key length in vector")?;
                let sk = SecretKey::from_bytes(private);
                let pk = sk.public_key().map_err(|_| "public key derivation failed")?;
                if pk.as_bytes() != case.public {
                    return Err("public key mismatch");
                }
                if sk.to_bytes() != *private {
                    return Err("secret key round trip mismatch");
                }
                Ok(Verdict::Pass)
            },
        )?;

        let fail = |what| Failure {
            suite: suite.name,
            tc_id: u32::MAX,
            what,
        };
        let mut rng = TestRng::new();
        let a = SecretKey::generate(&mut rng).map_err(|_| fail("key generation failed"))?;
        let b = SecretKey::generate(&mut rng).map_err(|_| fail("key generation failed"))?;
        let pa = a.public_key().map_err(|_| fail("public key derivation failed"))?;
        let pb = b.public_key().map_err(|_| fail("public key derivation failed"))?;
        let sab = a.diffie_hellman(&pb).map_err(|_| fail("agreement failed"))?;
        let sba = b.diffie_hellman(&pa).map_err(|_| fail("agreement failed"))?;
        if sab.as_bytes() != sba.as_bytes() {
            return Err(fail("shared secrets differ"));
        }
        if a.diffie_hellman(&PublicKey::from_bytes(&[0u8; 32])).is_ok() {
            return Err(fail("all-zero peer key accepted"));
        }
        Ok(stats)
    }
}

// =============================================================================
// Named suites
// =============================================================================

macro_rules! named {
    ($($(#[$meta:meta])* $name:ident => $f:expr;)*) => {$(
        $(#[$meta])*
        pub fn $name() -> Outcome {
            $f
        }
    )*};
}

named! {
    /// MD5 (generated).
    md5 => digest::<embassy_crypto::Md5>(&MD5);
    /// SHA-1 (generated).
    sha1 => digest::<embassy_crypto::Sha1>(&SHA1);
    /// SHA-224 (generated).
    sha224 => digest::<embassy_crypto::Sha224>(&SHA224);
    /// SHA-256 (generated).
    sha256 => digest::<embassy_crypto::Sha256>(&SHA256);
    /// SHA-384 (generated).
    sha384 => digest::<embassy_crypto::Sha384>(&SHA384);
    /// SHA-512 (generated).
    sha512 => digest::<embassy_crypto::Sha512>(&SHA512);
    /// SHA-512/224 (generated).
    sha512_224 => digest::<embassy_crypto::Sha512_224>(&SHA512_224);
    /// SHA-512/256 (generated).
    sha512_256 => digest::<embassy_crypto::Sha512_256>(&SHA512_256);

    /// HMAC-SHA-1 (Wycheproof `hmac_sha1_test`).
    hmac_sha1 => mac::<embassy_crypto::HmacSha1>(&HMAC_SHA1);
    /// HMAC-SHA-224 (Wycheproof `hmac_sha224_test`).
    hmac_sha224 => mac::<embassy_crypto::HmacSha224>(&HMAC_SHA224);
    /// HMAC-SHA-256 (Wycheproof `hmac_sha256_test`).
    hmac_sha256 => mac::<embassy_crypto::HmacSha256>(&HMAC_SHA256);
    /// HMAC-SHA-384 (Wycheproof `hmac_sha384_test`).
    hmac_sha384 => mac::<embassy_crypto::HmacSha384>(&HMAC_SHA384);
    /// HMAC-SHA-512 (Wycheproof `hmac_sha512_test`).
    hmac_sha512 => mac::<embassy_crypto::HmacSha512>(&HMAC_SHA512);
    /// HMAC-SHA-512/224 (Wycheproof `hmac_sha512_224_test`).
    hmac_sha512_224 => mac::<embassy_crypto::HmacSha512_224>(&HMAC_SHA512_224);
    /// HMAC-SHA-512/256 (Wycheproof `hmac_sha512_256_test`).
    hmac_sha512_256 => mac::<embassy_crypto::HmacSha512_256>(&HMAC_SHA512_256);

    /// AES-128 ECB (generated).
    aes128_ecb => aes_ecb::<embassy_crypto::Aes128>(&AES_ECB_128);
    /// AES-256 ECB (generated).
    aes256_ecb => aes_ecb::<embassy_crypto::Aes256>(&AES_ECB_256);
    /// AES-128 CBC (Wycheproof `aes_cbc_pkcs5_test`).
    aes128_cbc => aes_cbc::<embassy_crypto::Aes128CbcEncrypt, embassy_crypto::Aes128CbcDecrypt>(&AES_CBC_128);
    /// AES-256 CBC (Wycheproof `aes_cbc_pkcs5_test`).
    aes256_cbc => aes_cbc::<embassy_crypto::Aes256CbcEncrypt, embassy_crypto::Aes256CbcDecrypt>(&AES_CBC_256);
    /// AES-128 CTR (generated).
    aes128_ctr => aes_ctr::<embassy_crypto::Aes128Ctr>(&AES_CTR_128);
    /// AES-256 CTR (generated).
    aes256_ctr => aes_ctr::<embassy_crypto::Aes256Ctr>(&AES_CTR_256);
    /// AES-128 GCM (Wycheproof `aes_gcm_test`).
    aes128_gcm => aes_gcm::<embassy_crypto::Aes128Gcm>(&AES_GCM_128);
    /// AES-256 GCM (Wycheproof `aes_gcm_test`).
    aes256_gcm => aes_gcm::<embassy_crypto::Aes256Gcm>(&AES_GCM_256);
    /// AES-128 CCM (Wycheproof `aes_ccm_test`).
    aes128_ccm => aes_ccm::<embassy_crypto::Aes128Ccm>(&AES_CCM_128);
    /// AES-256 CCM (Wycheproof `aes_ccm_test`).
    aes256_ccm => aes_ccm::<embassy_crypto::Aes256Ccm>(&AES_CCM_256);
    /// AES-128 CMAC (Wycheproof `aes_cmac_test`).
    aes128_cmac => mac::<embassy_crypto::Aes128Cmac>(&AES_CMAC_128);
    /// AES-256 CMAC (Wycheproof `aes_cmac_test`).
    aes256_cmac => mac::<embassy_crypto::Aes256Cmac>(&AES_CMAC_256);

    /// P-256 arithmetic (generated).
    p256_arith => p256::arith(&P256_ARITH);
    /// P-256 ECDH (Wycheproof `ecdh_secp256r1_ecpoint_test`).
    p256_ecdh => p256::ecdh(&P256_ECDH);
    /// P-256 ECDSA (Wycheproof `ecdsa_secp256r1_sha256_p1363_test`).
    p256_ecdsa => p256::ecdsa(&P256_ECDSA);
    /// P-384 arithmetic (generated).
    p384_arith => p384::arith(&P384_ARITH);
    /// P-384 ECDH (Wycheproof `ecdh_secp384r1_ecpoint_test`).
    p384_ecdh => p384::ecdh(&P384_ECDH);
    /// P-384 ECDSA (Wycheproof `ecdsa_secp384r1_sha384_p1363_test`).
    p384_ecdsa => p384::ecdsa(&P384_ECDSA);

    /// X25519 (Wycheproof `x25519_test`).
    x25519_dh => x25519::dh(&X25519);
    /// X25519 key generation (generated).
    x25519_keygen => x25519::keygen(&X25519_KEYGEN);
}
