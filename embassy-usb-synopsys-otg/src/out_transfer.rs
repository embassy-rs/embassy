//! Bookkeeping for multi-packet OUT transfers.
//!
//! This module is hardware independent so that it can be unit tested on the host.
//!
//! In slave mode, a bulk OUT endpoint can be armed for more than one packet: `DOEPTSIZ.PKTCNT` is
//! the number of packets and `DOEPTSIZ.XFRSIZ` the number of bytes the core may accept before it
//! stops. While the endpoint is armed the core ACKs packets from the host without any software
//! involvement, and pushes them into the shared RX FIFO. The interrupt handler appends each packet
//! to the endpoint's buffer ([`OutTransfer::packet`]). The core ends the transfer, with
//! `PKTSTS = OUT_DATA_DONE`, when `PKTCNT` reaches zero *or* when a packet shorter than the
//! maximum packet size (a short packet, or a zero-length packet) arrives. The interrupt handler
//! then publishes the transfer ([`OutTransfer::done`]) as a [`Chunk`] and the endpoint stays
//! disarmed (the host is NAKed) until the reader has taken the data and re-armed it.

/// Largest value of `DOEPTSIZ.PKTCNT` (a 10 bit field).
const MAX_PKTCNT: u16 = 1023;

/// A packet did not fit the transfer buffer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Overflow;

/// A completed transfer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Chunk {
    /// Number of bytes received.
    pub len: u16,
    /// The transfer ended because a short packet (or a zero-length packet) arrived, rather than
    /// because the transfer size was reached.
    ///
    /// This cannot be derived from `len`: a transfer of exactly one full packet followed by a
    /// zero-length packet has the same length as a transfer that filled one packet and is
    /// continued by the next one.
    pub short: bool,
}

impl Chunk {
    /// Number of packets the host sent for this chunk. A zero-length packet that terminates a
    /// transfer whose length is a multiple of the packet size is counted.
    pub(crate) fn packets(&self, mps: u16) -> u16 {
        let full_and_partial = self.len.div_ceil(mps);
        if self.short && self.len.is_multiple_of(mps) {
            full_and_partial + 1
        } else {
            full_and_partial
        }
    }

    /// The `index`th packet of the chunk, as the byte range of the chunk that it covers.
    pub(crate) fn packet_range(&self, mps: u16, index: u16) -> core::ops::Range<usize> {
        let start = (u32::from(index) * u32::from(mps)).min(u32::from(self.len));
        let end = (start + u32::from(mps)).min(u32::from(self.len));
        start as usize..end as usize
    }
}

/// The OUT transfer in progress on one endpoint. Only the interrupt handler uses it, and the code
/// that resets the endpoint while the interrupt cannot run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct OutTransfer {
    mps: u16,
    size: u16,
    fill: u16,
    last_short: bool,
    overflowed: bool,
}

impl OutTransfer {
    /// A transfer of at most `size` bytes on an endpoint with maximum packet size `mps`.
    ///
    /// `size` is rounded down to a whole number of packets (the core requires it) and limited to
    /// what `PKTCNT` can count. It is at least one packet.
    pub(crate) const fn new(mps: u16, size: u16) -> Self {
        // In u32: 1023 packets of 512 bytes do not fit in 16 bits.
        let max = MAX_PKTCNT as u32 * mps as u32;
        let size = if size as u32 > max { max as u16 } else { size };
        let size = if size < mps { mps } else { size - size % mps };
        Self {
            mps,
            size,
            fill: 0,
            last_short: false,
            overflowed: false,
        }
    }

    /// Number of bytes the endpoint's buffer must hold.
    pub(crate) const fn size(&self) -> u16 {
        self.size
    }

    /// The values for `DOEPTSIZ`: `(XFRSIZ, PKTCNT)`.
    pub(crate) const fn arm(&self) -> (u32, u16) {
        (self.size as u32, self.size / self.mps)
    }

    /// Forget a transfer in progress.
    pub(crate) fn reset(&mut self) {
        self.fill = 0;
        self.last_short = false;
        self.overflowed = false;
    }

    /// The core pushed a packet of `len` bytes into the RX FIFO. Returns the offset in the buffer
    /// the packet belongs at.
    ///
    /// On `Err` the packet must still be drained from the FIFO and discarded, and the transfer is
    /// dropped when it ends.
    pub(crate) fn packet(&mut self, len: u16) -> Result<usize, Overflow> {
        if self.overflowed || len > self.mps || u32::from(self.fill) + u32::from(len) > u32::from(self.size) {
            self.overflowed = true;
            return Err(Overflow);
        }
        let at = usize::from(self.fill);
        self.fill += len;
        self.last_short = len < self.mps;
        Ok(at)
    }

    /// The core reported the transfer as done. Starts a new transfer.
    ///
    /// Returns `Err` if a packet of this transfer was discarded.
    pub(crate) fn done(&mut self) -> Result<Chunk, Overflow> {
        let chunk = Chunk {
            len: self.fill,
            short: self.last_short,
        };
        let overflowed = self.overflowed;
        self.reset();
        if overflowed { Err(Overflow) } else { Ok(chunk) }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;
    use std::vec::Vec;

    use super::*;

    const MPS: u16 = 64;
    const SIZE: u16 = 2048;

    #[test]
    fn arm_covers_whole_packets() {
        assert_eq!(OutTransfer::new(64, 2048).arm(), (2048, 32));
        assert_eq!(OutTransfer::new(64, 2049).arm(), (2048, 32));
        assert_eq!(OutTransfer::new(64, 100).arm(), (64, 1));
        assert_eq!(OutTransfer::new(64, 10).arm(), (64, 1));
        assert_eq!(OutTransfer::new(64, 0).arm(), (64, 1));
        assert_eq!(OutTransfer::new(512, 4096).arm(), (4096, 8));
        // PKTCNT is 10 bits wide.
        assert_eq!(OutTransfer::new(8, u16::MAX).arm(), (8184, 1023));
        assert_eq!(OutTransfer::new(64, u16::MAX).arm(), (65472, 1023));
        assert_eq!(OutTransfer::new(512, u16::MAX).arm(), (65024, 127));
        assert_eq!(OutTransfer::new(1024, u16::MAX).arm(), (64512, 63));
    }

    #[test]
    fn size_is_what_the_buffer_must_hold() {
        assert_eq!(OutTransfer::new(64, 2049).size(), 2048);
        assert_eq!(OutTransfer::new(64, 0).size(), 64);
    }

    #[test]
    fn short_flag_is_not_derivable_from_length() {
        // A full packet then a ZLP.
        let mut t = OutTransfer::new(64, 2048);
        t.packet(64).unwrap();
        t.packet(0).unwrap();
        assert_eq!(t.done(), Ok(Chunk { len: 64, short: true }));

        // A full packet and nothing more: the next packet continues the transfer.
        let mut t = OutTransfer::new(64, 64);
        t.packet(64).unwrap();
        assert_eq!(t.done(), Ok(Chunk { len: 64, short: false }));
    }

    #[test]
    fn lone_zlp() {
        let mut t = OutTransfer::new(64, 2048);
        assert_eq!(t.packet(0), Ok(0));
        assert_eq!(t.done(), Ok(Chunk { len: 0, short: true }));
    }

    #[test]
    fn full_transfer_is_not_short() {
        let mut t = OutTransfer::new(64, 128);
        assert_eq!(t.packet(64), Ok(0));
        assert_eq!(t.packet(64), Ok(64));
        assert_eq!(t.done(), Ok(Chunk { len: 128, short: false }));
    }

    #[test]
    fn overflow_drops_the_transfer_and_recovers() {
        let mut t = OutTransfer::new(64, 128);
        assert_eq!(t.packet(64), Ok(0));
        assert_eq!(t.packet(64), Ok(64));
        assert_eq!(t.packet(1), Err(Overflow));
        // Once a packet was refused, later packets of the transfer are refused too.
        assert_eq!(t.packet(0), Err(Overflow));
        assert_eq!(t.done(), Err(Overflow));
        // The next transfer starts clean.
        assert_eq!(t.packet(10), Ok(0));
        assert_eq!(t.done(), Ok(Chunk { len: 10, short: true }));
        // A packet larger than the maximum packet size is refused.
        assert_eq!(OutTransfer::new(64, 128).packet(65), Err(Overflow));
    }

    #[test]
    fn reset_forgets_partial_data() {
        let mut t = OutTransfer::new(64, 128);
        t.packet(64).unwrap();
        t.reset();
        assert_eq!(t.packet(64), Ok(0));
    }

    #[test]
    fn chunk_packets() {
        let c = |len, short| Chunk { len, short };
        assert_eq!(c(0, true).packets(64), 1); // lone ZLP
        assert_eq!(c(1, true).packets(64), 1);
        assert_eq!(c(64, true).packets(64), 2); // full packet + ZLP
        assert_eq!(c(64, false).packets(64), 1);
        assert_eq!(c(65, true).packets(64), 2);
        assert_eq!(c(2048, false).packets(64), 32);
        assert_eq!(c(2048, true).packets(64), 33);
        assert_eq!(c(100, true).packet_range(64, 0), 0..64);
        assert_eq!(c(100, true).packet_range(64, 1), 64..100);
        assert_eq!(c(128, true).packet_range(64, 2), 128..128); // the ZLP
        assert_eq!(c(0, true).packet_range(64, 0), 0..0);
    }

    #[test]
    fn packets_of_a_chunk_cover_it_exactly() {
        for len in [0u16, 1, 63, 64, 65, 128, 130, 2048] {
            for short in [false, true] {
                // A chunk that is not short is a whole number of packets.
                if !short && (len == 0 || len % 64 != 0) {
                    continue;
                }
                let c = Chunk { len, short };
                let mut covered = 0;
                for i in 0..c.packets(64) {
                    let r = c.packet_range(64, i);
                    assert_eq!(r.start, covered);
                    assert!(r.len() <= 64);
                    // Only the last packet of a transfer is short.
                    assert!(r.len() == 64 || i + 1 == c.packets(64));
                    covered = r.end;
                }
                assert_eq!(covered, usize::from(len));
            }
        }
    }

    /// The host: transfers cut into packets, with a short packet or a ZLP after each.
    fn host_packets(transfers: &[Vec<u8>]) -> VecDeque<Vec<u8>> {
        let mut out = VecDeque::new();
        for t in transfers {
            for c in t.chunks(MPS as usize) {
                out.push_back(c.to_vec());
            }
            if t.len() % MPS as usize == 0 {
                out.push_back(Vec::new());
            }
        }
        out
    }

    /// Small deterministic generator, so that the model test needs no dependency.
    struct Rng(u64);
    impl Rng {
        fn next(&mut self) -> u32 {
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 7;
            self.0 ^= self.0 << 17;
            (self.0 >> 32) as u32
        }
    }

    /// Runs `transfers` through a model of the core, the interrupt handler, and a reader that
    /// implements the `read_transfer` rule (read chunks until a short one, or until the caller's
    /// buffer is full), with a random interleaving of the three and random pauses of the reader.
    fn run(transfers: &[Vec<u8>], seed: u64) -> Vec<Vec<u8>> {
        let mut rng = Rng(seed | 1);
        let mut host = host_packets(transfers);
        let mut xfer = OutTransfer::new(MPS, SIZE);
        let buf_size = usize::from(SIZE);

        // The core.
        let mut armed = true;
        let mut pktcnt = xfer.arm().1;
        let mut fifo: VecDeque<Vec<u8>> = VecDeque::new();
        let fifo_words = 48usize;
        let mut done_pending = false;
        // The endpoint's buffer and what the interrupt published.
        let mut buffer = std::vec![0u8; buf_size];
        let mut chunk: Option<Chunk> = None;
        // The reader.
        let mut acc: Vec<u8> = Vec::new();
        let mut paused = 0u32;
        let mut out = Vec::new();

        for _ in 0..2_000_000 {
            match rng.next() % 3 {
                0 => {
                    // The host sends a packet, if the core takes it.
                    if let Some(p) = host.front() {
                        let used: usize = fifo.iter().map(|q| q.len().div_ceil(4) + 1).sum();
                        if armed && !done_pending && used + p.len().div_ceil(4) + 2 <= fifo_words {
                            let p = host.pop_front().unwrap();
                            let short = p.len() < usize::from(MPS);
                            fifo.push_back(p);
                            pktcnt -= 1;
                            if short || pktcnt == 0 {
                                armed = false;
                                done_pending = true;
                            }
                        }
                    }
                }
                1 => {
                    // The interrupt handler.
                    while let Some(p) = fifo.pop_front() {
                        let at = xfer.packet(p.len() as u16).expect("fits");
                        buffer[at..at + p.len()].copy_from_slice(&p);
                    }
                    if done_pending && chunk.is_none() {
                        chunk = Some(xfer.done().expect("no overflow"));
                        done_pending = false;
                    }
                }
                _ => {
                    // The reader.
                    if paused > 0 {
                        paused -= 1;
                        continue;
                    }
                    if let Some(c) = chunk.take() {
                        assert!(acc.len() + usize::from(c.len) <= buf_size, "chunk does not fit");
                        acc.extend_from_slice(&buffer[..usize::from(c.len)]);
                        // Re-arm.
                        armed = true;
                        pktcnt = xfer.arm().1;
                        paused = rng.next() % 5;
                        if c.short {
                            out.push(core::mem::take(&mut acc));
                        }
                    }
                }
            }
            if out.len() == transfers.len() {
                break;
            }
        }
        out
    }

    #[test]
    fn model_every_transfer_arrives_whole_and_in_order() {
        // Sizes that straddle the boundaries, including exact multiples of the packet size (which
        // are ended by a ZLP) and sizes just below the buffer size. A transfer of exactly the
        // buffer size is excluded: read_transfer returns when the caller's buffer is full, and
        // the ZLP is then the next (empty) transfer.
        let sizes = [0usize, 1, 63, 64, 65, 127, 128, 129, 1542, 1984, 1985, 2047];
        let mut seed = 1u64;
        for round in 0..300usize {
            let n = 1 + round % 7;
            let transfers: Vec<Vec<u8>> = (0..n)
                .map(|i| {
                    let len = sizes[(round * 5 + i * 3) % sizes.len()];
                    (0..len).map(|b| (b + i + round) as u8).collect()
                })
                .collect();
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let got = run(&transfers, seed);
            assert_eq!(got, transfers, "round {round}");
        }
    }
}
