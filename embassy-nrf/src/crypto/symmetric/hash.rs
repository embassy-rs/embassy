//! Hash and HMAC.

use super::{MAX_CHUNK, Symmetric};
use crate::mode::Mode;
use crate::util::for_each_ram_chunk;

#[cfg_attr(feature = "_cryptocell", path = "hash_cryptocell.rs")]
#[cfg_attr(feature = "_cracen", path = "hash_cracen.rs")]
mod hw;

/// Fixed-size byte buffer. Implemented for `[u8; N]`.
///
/// Used for the digest type of a [`HashAlgorithm`].
pub trait Buffer: Copy + AsRef<[u8]> + AsMut<[u8]> {
    /// The all-zero buffer.
    const ZERO: Self;
}

impl<const N: usize> Buffer for [u8; N] {
    const ZERO: Self = [0; N];
}

/// Hash engine mode.
#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Sha1,
    Sha256,
    Sha384,
    Sha512,
}

trait SealedAlgorithm {}

/// Hash algorithm. Implemented by [`Sha1`], [`Sha224`], [`Sha256`] and, on CRACEN, the
/// SHA-512 family.
///
/// This trait is sealed.
#[allow(private_bounds)]
pub trait HashAlgorithm: SealedAlgorithm + Copy + 'static {
    /// Block length in bytes.
    const BLOCK_LEN: usize;
    /// Digest length in bytes.
    const DIGEST_LEN: usize;
    /// Digest type: `[u8; DIGEST_LEN]`.
    type Digest: Buffer;
    #[doc(hidden)]
    type State: Buffer;
    #[doc(hidden)]
    type Block: Buffer;
    #[doc(hidden)]
    const IV: Self::State;
    #[doc(hidden)]
    const KIND: Kind;
}

const fn be32<const W: usize, const N: usize>(words: [u32; W]) -> [u8; N] {
    let mut out = [0; N];
    let mut i = 0;
    while i < W {
        let b = words[i].to_be_bytes();
        out[4 * i] = b[0];
        out[4 * i + 1] = b[1];
        out[4 * i + 2] = b[2];
        out[4 * i + 3] = b[3];
        i += 1;
    }
    out
}

#[cfg(feature = "_cracen")]
const fn be64(words: [u64; 8]) -> [u8; 64] {
    let mut out = [0; 64];
    let mut i = 0;
    while i < 8 {
        let b = words[i].to_be_bytes();
        let mut j = 0;
        while j < 8 {
            out[8 * i + j] = b[j];
            j += 1;
        }
        i += 1;
    }
    out
}

macro_rules! algorithm {
    ($(#[$meta:meta])* $name:ident, $kind:expr, $block:expr, $digest:expr, $state:expr, $iv:expr) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy)]
        #[cfg_attr(feature = "defmt", derive(defmt::Format))]
        pub struct $name;

        impl SealedAlgorithm for $name {}
        impl HashAlgorithm for $name {
            const BLOCK_LEN: usize = $block;
            const DIGEST_LEN: usize = $digest;
            type Digest = [u8; $digest];
            type State = [u8; $state];
            type Block = [u8; $block];
            const IV: Self::State = $iv;
            const KIND: Kind = $kind;
        }
    };
}

algorithm!(
    /// SHA-1.
    ///
    /// SHA-1 is not collision resistant. Use it only where a protocol requires it.
    Sha1,
    Kind::Sha1,
    64,
    20,
    20,
    be32([0x67452301, 0xefcdab89, 0x98badcfe, 0x10325476, 0xc3d2e1f0])
);
algorithm!(
    /// SHA-224.
    Sha224,
    Kind::Sha256,
    64,
    28,
    32,
    be32([
        0xc1059ed8, 0x367cd507, 0x3070dd17, 0xf70e5939, 0xffc00b31, 0x68581511, 0x64f98fa7, 0xbefa4fa4,
    ])
);
algorithm!(
    /// SHA-256.
    Sha256,
    Kind::Sha256,
    64,
    32,
    32,
    be32([
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
    ])
);
#[cfg(feature = "_cracen")]
algorithm!(
    /// SHA-384.
    Sha384,
    Kind::Sha384,
    128,
    48,
    64,
    be64([
        0xcbbb9d5dc1059ed8,
        0x629a292a367cd507,
        0x9159015a3070dd17,
        0x152fecd8f70e5939,
        0x67332667ffc00b31,
        0x8eb44a8768581511,
        0xdb0c2e0d64f98fa7,
        0x47b5481dbefa4fa4,
    ])
);
#[cfg(feature = "_cracen")]
algorithm!(
    /// SHA-512.
    Sha512,
    Kind::Sha512,
    128,
    64,
    64,
    be64([
        0x6a09e667f3bcc908,
        0xbb67ae8584caa73b,
        0x3c6ef372fe94f82b,
        0xa54ff53a5f1d36f1,
        0x510e527fade682d1,
        0x9b05688c2b3e6c1f,
        0x1f83d9abfb41bd6b,
        0x5be0cd19137e2179,
    ])
);
#[cfg(feature = "_cracen")]
algorithm!(
    /// SHA-512/224.
    Sha512_224,
    Kind::Sha512,
    128,
    28,
    64,
    be64([
        0x8c3d37c819544da2,
        0x73e1996689dcd4d6,
        0x1dfab7ae32ff9c82,
        0x679dd514582f9fcf,
        0x0f6d2b697bd44da8,
        0x77e36f7304c48942,
        0x3f9d85a86a1d36c8,
        0x1112e6ad91d692a1,
    ])
);
#[cfg(feature = "_cracen")]
algorithm!(
    /// SHA-512/256.
    Sha512_256,
    Kind::Sha512,
    128,
    32,
    64,
    be64([
        0x22312194fc2bf72c,
        0x9f555fa3c84c64c2,
        0x2393b86b6f53b151,
        0x963877195940eabd,
        0x96283ee2a88effe3,
        0xbe5e1e2553863992,
        0x2b0199fc2c85b8aa,
        0x0eb72ddc81c52ca2,
    ])
);

/// State of a hash computation in progress. Created by [`Symmetric::hash_start`].
#[derive(Clone)]
pub struct HashContext<A: HashAlgorithm> {
    state: A::State,
    buf: A::Block,
    /// Number of message bytes fed so far.
    len: u64,
    buf_len: u8,
}

impl<A: HashAlgorithm> HashContext<A> {
    fn new(state: A::State, len: u64) -> Self {
        Self {
            state,
            buf: A::Block::ZERO,
            len,
            buf_len: 0,
        }
    }

    fn compress(state: &mut A::State, blocks: &[u8]) {
        hw::compress(A::KIND, state.as_mut(), blocks);
    }

    fn update(&mut self, data: &[u8]) {
        let block = A::BLOCK_LEN;
        self.len += data.len() as u64;
        let mut data = data;

        if self.buf_len != 0 {
            let n = (block - self.buf_len as usize).min(data.len());
            self.buf.as_mut()[self.buf_len as usize..][..n].copy_from_slice(&data[..n]);
            self.buf_len += n as u8;
            data = &data[n..];
            if self.buf_len as usize == block {
                let b = self.buf;
                Self::compress(&mut self.state, b.as_ref());
                self.buf_len = 0;
            } else {
                // All of `data` fit in the buffer.
                return;
            }
        }

        let full = data.len() / block * block;
        let state = &mut self.state;
        for_each_ram_chunk(&data[..full], MAX_CHUNK, |chunk| Self::compress(state, chunk));

        self.buf.as_mut()[..data.len() - full].copy_from_slice(&data[full..]);
        self.buf_len = (data.len() - full) as u8;
    }

    fn finish(mut self) -> A::Digest {
        let block = A::BLOCK_LEN;
        // SHA-384/512 have a 128-bit length field, the others 64-bit.
        let len_field = block / 8;
        let bits = (self.len as u128) * 8;

        let buf = self.buf.as_mut();
        let mut n = self.buf_len as usize;
        buf[n] = 0x80;
        n += 1;
        if n + len_field > block {
            buf[n..block].fill(0);
            Self::compress(&mut self.state, &buf[..block]);
            n = 0;
        }
        buf[n..block - len_field].fill(0);
        buf[block - len_field..block].copy_from_slice(&bits.to_be_bytes()[16 - len_field..]);
        Self::compress(&mut self.state, &buf[..block]);

        let mut digest = A::Digest::ZERO;
        digest.as_mut().copy_from_slice(&self.state.as_ref()[..A::DIGEST_LEN]);
        digest
    }
}

/// State of an HMAC computation in progress. Created by [`Symmetric::hmac_start`].
#[derive(Clone)]
pub struct HmacContext<A: HashAlgorithm> {
    inner: HashContext<A>,
    /// Hash state after absorbing `K0 ^ ipad`.
    ipad: A::State,
    /// Hash state after absorbing `K0 ^ opad`.
    opad: A::State,
}

impl<A: HashAlgorithm> HmacContext<A> {
    fn new(key: &[u8]) -> Self {
        // K0: the key, hashed if longer than a block, zero-padded to a block.
        let mut k0 = A::Block::ZERO;
        if key.len() > A::BLOCK_LEN {
            let mut ctx = HashContext::<A>::new(A::IV, 0);
            ctx.update(key);
            let digest = ctx.finish();
            k0.as_mut()[..A::DIGEST_LEN].copy_from_slice(digest.as_ref());
        } else {
            k0.as_mut()[..key.len()].copy_from_slice(key);
        }

        let mut ipad = A::IV;
        let mut opad = A::IV;
        let mut block = k0;
        for b in block.as_mut() {
            *b ^= 0x36;
        }
        HashContext::<A>::compress(&mut ipad, block.as_ref());
        let mut block = k0;
        for b in block.as_mut() {
            *b ^= 0x5c;
        }
        HashContext::<A>::compress(&mut opad, block.as_ref());

        Self {
            inner: HashContext::new(ipad, A::BLOCK_LEN as u64),
            ipad,
            opad,
        }
    }

    /// Resets the context, keeping the key.
    ///
    /// Afterwards the context is as if [`Symmetric::hmac_start`] had just returned it. This
    /// is cheaper than starting a new one, since the key does not have to be processed again.
    pub fn reset(&mut self) {
        self.inner = HashContext::new(self.ipad, A::BLOCK_LEN as u64);
    }

    fn finish(self) -> A::Digest {
        let inner = self.inner.finish();
        let mut outer = HashContext::<A>::new(self.opad, A::BLOCK_LEN as u64);
        outer.update(inner.as_ref());
        outer.finish()
    }
}

trait SealedDigestContext {}

/// A hash or HMAC context. Implemented by [`HashContext`] and [`HmacContext`].
///
/// This trait is sealed.
#[allow(private_bounds)]
pub trait DigestContext: SealedDigestContext + Sized {
    /// The hash algorithm of this context.
    type Algorithm: HashAlgorithm;
    #[doc(hidden)]
    fn update(&mut self, data: &[u8]);
    #[doc(hidden)]
    fn finish(self) -> <Self::Algorithm as HashAlgorithm>::Digest;
}

impl<A: HashAlgorithm> SealedDigestContext for HashContext<A> {}
impl<A: HashAlgorithm> DigestContext for HashContext<A> {
    type Algorithm = A;
    fn update(&mut self, data: &[u8]) {
        HashContext::update(self, data)
    }
    fn finish(self) -> A::Digest {
        HashContext::finish(self)
    }
}

impl<A: HashAlgorithm> SealedDigestContext for HmacContext<A> {}
impl<A: HashAlgorithm> DigestContext for HmacContext<A> {
    type Algorithm = A;
    fn update(&mut self, data: &[u8]) {
        self.inner.update(data)
    }
    fn finish(self) -> A::Digest {
        HmacContext::finish(self)
    }
}

impl<'d, M: Mode> Symmetric<'d, M> {
    /// Starts a hash computation with algorithm `A`.
    pub fn hash_start<A: HashAlgorithm>(&mut self) -> HashContext<A> {
        HashContext::new(A::IV, 0)
    }

    /// Starts an HMAC computation with algorithm `A` and the given key.
    ///
    /// The key may have any length.
    pub fn hmac_start<A: HashAlgorithm>(&mut self, key: &[u8]) -> HmacContext<A> {
        HmacContext::new(key)
    }

    /// Feeds message data into a hash or HMAC context.
    ///
    /// Data may be fed in chunks of any size.
    pub fn hash_blocking_update<H: DigestContext>(&mut self, ctx: &mut H, data: &[u8]) {
        ctx.update(data)
    }

    /// Finishes a hash or HMAC context and returns the digest.
    pub fn hash_blocking_finish<H: DigestContext>(&mut self, ctx: H) -> <H::Algorithm as HashAlgorithm>::Digest {
        ctx.finish()
    }
}
