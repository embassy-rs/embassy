//! Poly1305 in software, for the CryptoCell, which has no Poly1305 engine.
//!
//! The accumulator and `r` are 32-bit limbs, the arrangement Monocypher and OpenSSL's ARMv4
//! code use. It suits Cortex-M: a limb product is one `UMLAL`, and the reduction is a chain
//! of add-with-carry instead of 64-bit shifts. Clamping clears the low two bits of the upper
//! three words of `r`, so the words that pass 2^130 fold back as `(r >> 2) * 5` exactly.
//! Nothing here branches or indexes on secret data.

/// Length of a Poly1305 block, and of the tag.
pub(crate) const BLOCK_LEN: usize = 16;

/// A Poly1305 computation in progress.
#[derive(Clone)]
pub(crate) struct Poly1305 {
    /// Clamped `r`: every word below 2^28, words 1 to 3 with the low two bits clear.
    r: [u32; 4],
    /// `(r[i] >> 2) * 5`, the multiplier of the words that wrap past 2^130.
    rr: [u32; 4],
    /// Accumulator: four words plus a small top word, at most 4 between blocks.
    h: [u32; 5],
    /// `s`, little-endian words.
    pad: [u32; 4],
}

#[inline(always)]
fn le32(b: &[u8]) -> u32 {
    u32::from_le_bytes(b.try_into().unwrap())
}

impl Poly1305 {
    /// Starts a computation with a one-time key.
    pub(crate) fn new(key: &[u8; 32]) -> Self {
        let r = [
            le32(&key[0..4]) & 0x0fff_ffff,
            le32(&key[4..8]) & 0x0fff_fffc,
            le32(&key[8..12]) & 0x0fff_fffc,
            le32(&key[12..16]) & 0x0fff_fffc,
        ];
        let rr = [
            (r[0] >> 2) * 5,
            (r[1] >> 2) + r[1],
            (r[2] >> 2) + r[2],
            (r[3] >> 2) + r[3],
        ];
        Self {
            r,
            rr,
            h: [0; 5],
            pad: [
                le32(&key[16..20]),
                le32(&key[20..24]),
                le32(&key[24..28]),
                le32(&key[28..32]),
            ],
        }
    }

    /// Absorbs whole blocks. The caller pads the last block of a message with zeros.
    pub(crate) fn update(&mut self, data: &[u8]) {
        debug_assert!(data.len().is_multiple_of(BLOCK_LEN));
        let [r0, r1, r2, r3] = self.r;
        let [rr0, rr1, rr2, rr3] = self.rr;
        let [mut h0, mut h1, mut h2, mut h3, mut h4] = self.h;

        for block in data.chunks_exact(BLOCK_LEN) {
            // s = h + m + 2^128, carries propagated so every limb is a u32. The extra bit
            // above 128 is what makes the encoding of a block injective.
            let (s0, c) = h0.overflowing_add(le32(&block[0..4]));
            let (s1, c) = h1.carrying_add(le32(&block[4..8]), c);
            let (s2, c) = h2.carrying_add(le32(&block[8..12]), c);
            let (s3, c) = h3.carrying_add(le32(&block[12..16]), c);
            let s4 = h4 + 1 + c as u32; // at most 6

            let (s0, s1, s2, s3, s4) = (s0 as u64, s1 as u64, s2 as u64, s3 as u64, s4 as u64);
            let (r0w, r1w, r2w, r3w) = (r0 as u64, r1 as u64, r2 as u64, r3 as u64);
            let (rr0, rr1, rr2, rr3) = (rr0 as u64, rr1 as u64, rr2 as u64, rr3 as u64);

            // s * r, the words past 2^130 already folded back. Each column is at most
            // 4 * 2^60 plus a small term, so it fits a u64 without carries.
            let x0 = s0 * r0w + s1 * rr3 + s2 * rr2 + s3 * rr1 + s4 * rr0;
            let x1 = s0 * r1w + s1 * r0w + s2 * rr3 + s3 * rr2 + s4 * rr1;
            let x2 = s0 * r2w + s1 * r1w + s2 * r0w + s3 * rr3 + s4 * rr2;
            let x3 = s0 * r3w + s1 * r2w + s2 * r1w + s3 * r0w + s4 * rr3;
            let x4 = (s4 as u32) * (r0 & 3);

            // Partial reduction: the bits above 2^130 come back times 5.
            let u5 = x4 + (x3 >> 32) as u32;
            let u0 = ((u5 >> 2) * 5) as u64 + (x0 as u32) as u64;
            let u1 = (u0 >> 32) + (x1 as u32) as u64 + (x0 >> 32);
            let u2 = (u1 >> 32) + (x2 as u32) as u64 + (x1 >> 32);
            let u3 = (u2 >> 32) + (x3 as u32) as u64 + (x2 >> 32);
            h4 = (u3 >> 32) as u32 + (u5 & 3);
            h0 = u0 as u32;
            h1 = u1 as u32;
            h2 = u2 as u32;
            h3 = u3 as u32;
        }

        self.h = [h0, h1, h2, h3, h4];
    }

    /// Returns the tag.
    pub(crate) fn finish(self) -> [u8; BLOCK_LEN] {
        let [h0, h1, h2, h3, h4] = self.h;

        // h < 2p, so one conditional subtraction: g = h + 5 - 2^130. If it does not borrow,
        // h was at least p and g is the reduced value.
        let (g0, c) = h0.overflowing_add(5);
        let (g1, c) = h1.carrying_add(0, c);
        let (g2, c) = h2.carrying_add(0, c);
        let (g3, c) = h3.carrying_add(0, c);
        let g4 = h4.wrapping_add(c as u32).wrapping_sub(4);

        // Select g if it did not borrow (top bit clear), else h. A mask, not a branch; the
        // black_box keeps the compiler from turning it back into one.
        let select = core::hint::black_box((g4 >> 31).wrapping_sub(1));
        let keep = !select;
        let w0 = (h0 & keep) | (g0 & select);
        let w1 = (h1 & keep) | (g1 & select);
        let w2 = (h2 & keep) | (g2 & select);
        let w3 = (h3 & keep) | (g3 & select);

        // Plus s modulo 2^128.
        let (o0, c) = w0.overflowing_add(self.pad[0]);
        let (o1, c) = w1.carrying_add(self.pad[1], c);
        let (o2, c) = w2.carrying_add(self.pad[2], c);
        let (o3, _) = w3.carrying_add(self.pad[3], c);

        let mut tag = [0u8; BLOCK_LEN];
        tag[0..4].copy_from_slice(&o0.to_le_bytes());
        tag[4..8].copy_from_slice(&o1.to_le_bytes());
        tag[8..12].copy_from_slice(&o2.to_le_bytes());
        tag[12..16].copy_from_slice(&o3.to_le_bytes());
        tag
    }
}
