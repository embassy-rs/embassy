//! CryptoMaster DMA: descriptor chains and transfers.
//!
//! Every symmetric operation on CRACEN is one scatter-gather DMA transaction:
//!
//! - An input ("fetch") chain carries engine configuration and data.
//! - An output ("push") chain receives the results.
//!
//! The tag of each input descriptor names the target engine (BA411E AES, BA413 hash, BA417
//! ChaCha20-Poly1305) and the role of the data.
//!
//! This follows the `sxsymcrypt` layer of Nordic's `nrf_security`.
//!
//! There is one DMA, so one transaction runs at a time. The owner of the `CRYPTO_SYMMETRIC`
//! peripheral, or the lock of the `embassy-crypto` drivers, guarantees that.

use core::ptr;
use core::sync::atomic::{Ordering, compiler_fence};

use crate::pac;

/// BA411E AES engine.
pub(crate) const ENGINE_AES: u32 = 1;
/// BA413 hash engine.
pub(crate) const ENGINE_HASH: u32 = 3;
/// BA417 ChaCha20-Poly1305 engine.
pub(crate) const ENGINE_CHACHA: u32 = 4;

/// Tag for a configuration-register write at `offset` in `engine`.
pub(crate) const fn tag_config(engine: u32, offset: u32) -> u32 {
    engine | (1 << 4) | (offset << 8)
}

/// Tag for data of type `datatype` for `engine`.
pub(crate) const fn tag_data(engine: u32, datatype: u32) -> u32 {
    engine | (datatype << 6)
}

/// Marks the last descriptor of a transaction.
pub(crate) const TAG_LAST: u32 = 1 << 5;

/// Number of trailing bytes of a padded descriptor that the engine must ignore.
pub(crate) const fn tag_ignore(n: usize) -> u32 {
    (n as u32) << 8
}

/// Realigns the engine's data path at the end of the descriptor.
pub(crate) const REALIGN: u32 = 1 << 29;
/// Discards the output instead of writing it to memory.
const DISCARD: u32 = 1 << 30;
/// `next` pointer value ending a chain.
const LAST_DESC: u32 = 1;

#[repr(C)]
#[derive(Clone, Copy)]
struct Desc {
    addr: u32,
    next: u32,
    sz: u32,
    tag: u32,
}

const EMPTY: Desc = Desc {
    addr: 0,
    next: 0,
    sz: 0,
    tag: 0,
};

/// A chain of at most `N` descriptors.
pub(crate) struct Chain<const N: usize> {
    descs: [Desc; N],
    n: usize,
}

impl<const N: usize> Chain<N> {
    pub(crate) const fn new() -> Self {
        Self {
            descs: [EMPTY; N],
            n: 0,
        }
    }

    /// Adds a descriptor for `len` bytes at `addr`.
    ///
    /// `flags` are OR'd into the size word (e.g. [`REALIGN`]).
    pub(crate) fn push(&mut self, addr: *const u8, len: usize, flags: u32, tag: u32) {
        assert!(self.n < N, "descriptor chain full");
        self.descs[self.n] = Desc {
            addr: addr as u32,
            next: 0,
            sz: (len as u32) | flags,
            tag,
        };
        self.n += 1;
    }

    /// Adds an input descriptor of `len` bytes, rounded up to a multiple of `align`.
    ///
    /// The padding bytes are tagged as ignored, but the engine still reads them. So the
    /// rounded-up length must be readable at `addr`.
    pub(crate) fn push_padded(&mut self, addr: *const u8, len: usize, align: usize, tag: u32) {
        let padded = (len + align - 1) / align * align;
        self.push(addr, padded, REALIGN, tag | tag_ignore(padded - len));
    }

    /// Adds an output descriptor that discards `len` bytes.
    pub(crate) fn push_discard(&mut self, len: usize) {
        if len != 0 {
            self.push(ptr::null(), len, DISCARD, 0);
        }
    }

    /// Adds an output descriptor for `len` bytes, discarding the engine's padding up to a
    /// multiple of `align`.
    pub(crate) fn push_out_padded(&mut self, addr: *mut u8, len: usize, align: usize) {
        let padded = (len + align - 1) / align * align;
        if len != 0 {
            self.push(addr, len, 0, 0);
        }
        self.push_discard(padded - len);
    }

    fn link(&mut self) {
        assert!(self.n > 0, "empty descriptor chain");
        for i in 0..self.n - 1 {
            self.descs[i].next = &self.descs[i + 1] as *const Desc as u32;
        }
        let last = &mut self.descs[self.n - 1];
        last.next = LAST_DESC;
        last.tag |= TAG_LAST;
        last.sz |= REALIGN;
    }
}

/// Runs one transaction and waits for it to complete.
///
/// # Safety
///
/// Every input descriptor must point to readable RAM, and every output descriptor to
/// writable RAM, for its whole length including padding.
pub(crate) unsafe fn run<const NI: usize, const NO: usize>(input: &mut Chain<NI>, output: &mut Chain<NO>) {
    input.link();
    output.link();

    let dma = pac::CRACENCORE.cryptmstrdma();
    compiler_fence(Ordering::SeqCst);

    dma.fetchaddrlsb().write_value(input.descs.as_ptr() as u32);
    dma.pushaddrlsb().write_value(output.descs.as_ptr() as u32);
    dma.config().write(|w| {
        w.set_fetchctrlindirect(true);
        w.set_pushctrlindirect(true);
    });
    dma.start().write(|w| {
        w.set_startfetch(true);
        w.set_startpush(true);
    });

    loop {
        let status = dma.status().read();
        if !(status.fetchbusy() || status.pushbusy() || status.pushwaitingfifo()) {
            break;
        }
    }
    compiler_fence(Ordering::SeqCst);

    let status = dma.intstatraw().read();
    let error = status.fetchererror() || status.pushererror();
    dma.intstatclr().write(|w| w.0 = !0);
    if error {
        soft_reset();
        panic!("CryptoMaster DMA error");
    }
}

/// Resets the DMA after a bus error.
fn soft_reset() {
    let dma = pac::CRACENCORE.cryptmstrdma();
    dma.config().write(|w| w.set_softrst(true));
    // A too short reset pulse can leave an active transaction in a bad state.
    cortex_m::asm::delay(256);
    dma.config().write(|w| w.set_softrst(false));
    while dma.status().read().softrstbusy() {}
}
