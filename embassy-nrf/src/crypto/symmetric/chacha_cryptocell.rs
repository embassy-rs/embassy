//! CryptoCell CHACHA engine primitives.

use super::{BLOCK_LEN, ChaChaVariant, KEY_LEN, NONCE_LEN};
use crate::crypto::cryptocell::dma::{self, Flow};
use crate::pac;
use crate::pac::cc_chacha::vals::{ChachaOrSalsa, KeyLen, NumOfRounds};

fn le(bytes: &[u8]) -> u32 {
    u32::from_le_bytes(bytes.try_into().unwrap())
}

/// Applies the keystream over whole blocks, starting at block `counter`.
pub(super) unsafe fn apply(
    variant: ChaChaVariant,
    key: &[u8; KEY_LEN],
    nonce: &[u8; NONCE_LEN],
    counter: u32,
    input: *const u8,
    output: *mut u8,
    len: usize,
) {
    debug_assert!(len % BLOCK_LEN == 0);
    pac::CC_MISC.chacha_clk().write(|w| w.set_enable(true));
    pac::CC_MISC.dma_clk().write(|w| w.set_enable(true));
    dma::prepare(Flow::ChaChaActive);

    let r = pac::CC_CHACHA;
    // With a 96-bit nonce, its first word takes the place of the block counter MSB.
    r.chacha_block_cnt_msb().write_value(le(&nonce[0..4]));
    r.chacha_iv(0).write_value(le(&nonce[4..8]));
    r.chacha_iv(1).write_value(le(&nonce[8..12]));
    r.chacha_block_cnt_lsb().write_value(counter);
    for (i, word) in key.chunks_exact(4).enumerate() {
        r.chacha_key(i).write_value(le(word));
    }
    r.chacha_control().write(|w| {
        w.set_chacha_or_salsa(ChachaOrSalsa::ChaCha);
        w.set_init(true);
        w.set_key_len(KeyLen::_256bits);
        w.set_num_of_rounds(match variant {
            ChaChaVariant::ChaCha20 => NumOfRounds::Default,
            ChaChaVariant::ChaCha12 => NumOfRounds::_12rounds,
            ChaChaVariant::ChaCha8 => NumOfRounds::_8rounds,
        });
        w.set_use_iv_96bit(true);
    });

    unsafe { dma::transfer(input, output, len) };

    pac::CC_MISC.chacha_clk().write(|w| w.set_enable(false));
    pac::CC_MISC.dma_clk().write(|w| w.set_enable(false));
}

/// ChaCha-Poly1305 authenticated encryption (RFC 8439 with ChaCha20).
///
/// The CryptoCell has no Poly1305 engine, so the authenticator runs in software.
pub(super) mod aead {
    use super::super::super::poly1305::Poly1305;
    use super::super::{BLOCK_LEN, ChaChaContext, ChaChaVariant, Direction, Error, KEY_LEN, NONCE_LEN};

    /// Poly1305 tag length, which is also the size of the blocks it absorbs.
    const TAG_LEN: usize = 16;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Phase {
        Aad,
        Payload,
    }

    /// State of an in-progress ChaCha-Poly1305 operation, created by
    /// [`Symmetric::chachapoly_start`](super::super::Symmetric::chachapoly_start).
    #[derive(Clone)]
    pub struct ChaChaPolyContext {
        stream: ChaChaContext,
        poly: Poly1305,
        dir: Direction,
        phase: Phase,
        /// Additional data that does not fill a whole block yet.
        aad_buf: [u8; TAG_LEN],
        aad_buf_len: u8,
        aad_len: usize,
        payload_len: usize,
    }

    impl ChaChaPolyContext {
        pub(crate) fn new(
            variant: ChaChaVariant,
            key: &[u8; KEY_LEN],
            nonce: &[u8; NONCE_LEN],
            dir: Direction,
        ) -> Self {
            // The one-time Poly1305 key is the first half of the keystream block zero; the
            // payload starts at block one.
            let mut block = ChaChaContext {
                variant,
                key: *key,
                nonce: *nonce,
                counter: 0,
                ks: [0; BLOCK_LEN],
                ks_len: 0,
            };
            let mut poly_key = [0u8; 32];
            unsafe { block.apply(poly_key.as_ptr(), poly_key.as_mut_ptr(), poly_key.len()) };

            Self {
                stream: ChaChaContext {
                    variant,
                    key: *key,
                    nonce: *nonce,
                    counter: 1,
                    ks: [0; BLOCK_LEN],
                    ks_len: 0,
                },
                poly: Poly1305::new(&poly_key),
                dir,
                phase: Phase::Aad,
                aad_buf: [0; TAG_LEN],
                aad_buf_len: 0,
                aad_len: 0,
                payload_len: 0,
            }
        }

        /// Absorbs the buffered additional data, zero-padded, and moves on to the payload.
        fn end_aad(&mut self) {
            if self.aad_buf_len > 0 {
                let mut block = self.aad_buf;
                block[self.aad_buf_len as usize..].fill(0);
                self.poly.update(&block);
                self.aad_buf_len = 0;
            }
            self.phase = Phase::Payload;
        }

        pub(crate) fn aad(&mut self, aad: &[u8], last: bool) -> Result<(), Error> {
            if self.phase != Phase::Aad {
                return Err(Error::AadAfterPayload);
            }
            self.aad_len += aad.len();

            let mut data = aad;
            if self.aad_buf_len > 0 {
                let n = (TAG_LEN - self.aad_buf_len as usize).min(data.len());
                self.aad_buf[self.aad_buf_len as usize..][..n].copy_from_slice(&data[..n]);
                self.aad_buf_len += n as u8;
                data = &data[n..];
                if self.aad_buf_len as usize == TAG_LEN {
                    let block = self.aad_buf;
                    self.poly.update(&block);
                    self.aad_buf_len = 0;
                }
            }
            // Anything left in the buffer here means the data was consumed by it entirely, so
            // the remainder below is empty and the buffered count carries over.
            let full = data.len() - data.len() % TAG_LEN;
            self.poly.update(&data[..full]);
            let rest = &data[full..];
            self.aad_buf[..rest.len()].copy_from_slice(rest);
            self.aad_buf_len += rest.len() as u8;

            if last {
                self.end_aad();
            }
            Ok(())
        }

        /// # Safety
        ///
        /// `input` must be readable and `output` writable for `len` bytes. If they overlap,
        /// they must be equal.
        pub(crate) unsafe fn payload(
            &mut self,
            input: *const u8,
            output: *mut u8,
            len: usize,
            last: bool,
        ) -> Result<(), Error> {
            if !last && len % BLOCK_LEN != 0 {
                return Err(Error::InvalidLength);
            }
            if self.phase == Phase::Aad {
                self.end_aad();
            }
            self.payload_len += len;

            // The tag covers the ciphertext: the input when decrypting, which must be absorbed
            // before the keystream overwrites it in place, and the output when encrypting.
            match self.dir {
                Direction::Decrypt => {
                    self.absorb(input, len);
                    unsafe { self.stream.apply(input, output, len) };
                }
                Direction::Encrypt => {
                    unsafe { self.stream.apply(input, output, len) };
                    self.absorb(output, len);
                }
            }
            Ok(())
        }

        /// Feeds `len` bytes of ciphertext at `ct` to the authenticator, zero-padded to the
        /// tag length.
        fn absorb(&mut self, ct: *const u8, len: usize) {
            let full = len - len % TAG_LEN;
            self.poly.update(unsafe { core::slice::from_raw_parts(ct, full) });
            if full < len {
                let mut block = [0u8; TAG_LEN];
                unsafe { core::ptr::copy_nonoverlapping(ct.add(full), block.as_mut_ptr(), len - full) };
                self.poly.update(&block);
            }
        }

        pub(crate) fn finish(mut self) -> Result<[u8; TAG_LEN], Error> {
            if self.phase == Phase::Aad {
                self.end_aad();
            }
            let mut lengths = [0u8; TAG_LEN];
            lengths[..8].copy_from_slice(&(self.aad_len as u64).to_le_bytes());
            lengths[8..].copy_from_slice(&(self.payload_len as u64).to_le_bytes());
            self.poly.update(&lengths);
            Ok(self.poly.finish())
        }
    }
}
