//! Hash generator (HASH)
use core::cmp::min;
#[cfg(any(hash_v2, hash_v3, hash_v4))]
use core::future::poll_fn;
use core::marker::PhantomData;
#[cfg(any(hash_v2, hash_v3, hash_v4))]
use core::ptr;
#[cfg(any(hash_v2, hash_v3, hash_v4))]
use core::task::Poll;

use embassy_hal_internal::PeripheralType;
use embassy_sync::waitqueue::AtomicWaker;
use stm32_metapac::hash::regs::*;

#[cfg(any(hash_v2, hash_v3, hash_v4))]
use crate::dma::ChannelAndRequest;
#[cfg(any(hash_v2, hash_v3, hash_v4))]
use crate::interrupt::typelevel::Interrupt;
#[cfg(any(hash_v2, hash_v3, hash_v4))]
use crate::mode::Async;
use crate::mode::{Blocking, Mode};
use crate::peripherals::HASH;
use crate::suspend::SealedSuspendablePeripheral;
use crate::{Peri, interrupt, pac, peripherals, rcc};

static HASH_WAKER: AtomicWaker = AtomicWaker::new();

/// HASH interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _marker: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let bits = T::regs().sr().read();
        if bits.dinis() {
            T::regs().imr().modify(|reg| reg.set_dinie(false));
            HASH_WAKER.wake();
        }
        if bits.dcis() {
            T::regs().imr().modify(|reg| reg.set_dcie(false));
            HASH_WAKER.wake();
        }
    }
}

///Hash algorithm selection
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Algorithm {
    /// SHA-1 Algorithm
    SHA1 = 0,

    #[cfg(any(hash_v1, hash_v2, hash_v4))]
    /// MD5 Algorithm
    MD5 = 1,

    /// SHA-224 Algorithm
    SHA224 = 2,

    /// SHA-256 Algorithm
    SHA256 = 3,

    #[cfg(hash_v3)]
    /// SHA-384 Algorithm
    SHA384 = 12,

    #[cfg(hash_v3)]
    /// SHA-512/224 Algorithm
    SHA512_224 = 13,

    #[cfg(hash_v3)]
    /// SHA-512/256 Algorithm
    SHA512_256 = 14,

    #[cfg(hash_v3)]
    /// SHA-256 Algorithm
    SHA512 = 15,
}

#[allow(missing_docs)]
pub trait AlgorithmSpec {
    const ALGORITHM: Algorithm;
    const BLOCK_SIZE: usize;
    const DIGEST_WORDS: usize;
    type BlockingBuffer: BufferStorage;
    type AsyncBuffer: BufferStorage;
    type KeyBuffer: BufferStorage;
    type NonHmacCsr: CsrStorage;
    type HmacCsr: CsrStorage;
}

#[allow(missing_docs)]
#[derive(Clone, Copy)]
pub struct Sha1;
#[allow(missing_docs)]
#[derive(Clone, Copy)]
pub struct Sha224;
#[allow(missing_docs)]
#[derive(Clone, Copy)]
pub struct Sha256;
#[cfg(any(hash_v1, hash_v2, hash_v4))]
#[allow(missing_docs)]
#[derive(Clone, Copy)]
pub struct Md5;
#[cfg(hash_v3)]
#[allow(missing_docs)]
#[derive(Clone, Copy)]
pub struct Sha384;
#[cfg(hash_v3)]
#[allow(missing_docs)]
#[derive(Clone, Copy)]
pub struct Sha512_224;
#[cfg(hash_v3)]
#[allow(missing_docs)]
#[derive(Clone, Copy)]
pub struct Sha512_256;
#[cfg(hash_v3)]
#[allow(missing_docs)]
#[derive(Clone, Copy)]
pub struct Sha512;

#[derive(Debug, Clone)]
#[allow(missing_docs)]
pub struct HashBuffer<const N: usize>([u8; N]);

#[allow(missing_docs)]
pub trait BufferStorage: Clone + core::fmt::Debug {
    fn new() -> Self;
    fn as_slice(&self) -> &[u8];
    fn as_mut_slice(&mut self) -> &mut [u8];
}

#[allow(missing_docs)]
pub trait CsrStorage: Clone + core::fmt::Debug {
    fn new() -> Self;
    fn get(&self, index: usize) -> u32;
    fn set(&mut self, index: usize, value: u32);
}

impl<const N: usize> CsrStorage for [u32; N] {
    fn new() -> Self {
        [0; N]
    }
    fn get(&self, index: usize) -> u32 {
        self[index]
    }
    fn set(&mut self, index: usize, value: u32) {
        self[index] = value;
    }
}

impl<const N: usize> BufferStorage for HashBuffer<N> {
    fn new() -> Self {
        Self([0; N])
    }
    fn as_slice(&self) -> &[u8] {
        &self.0
    }
    fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

macro_rules! impl_algorithm {
    ($type:ident, $algorithm:expr, $block:expr, $digest:expr) => {
        impl AlgorithmSpec for $type {
            const ALGORITHM: Algorithm = $algorithm;
            const BLOCK_SIZE: usize = $block;
            const DIGEST_WORDS: usize = $digest;
            type BlockingBuffer = HashBuffer<{ $block + 4 }>;
            type AsyncBuffer = HashBuffer<{ 2 * $block }>;
            type KeyBuffer = HashBuffer<$block>;
            type NonHmacCsr = [u32; csr_count($algorithm, false)];
            type HmacCsr = [u32; csr_count($algorithm, true)];
        }
    };
}

impl_algorithm!(Sha1, Algorithm::SHA1, 64, 5);
impl_algorithm!(Sha224, Algorithm::SHA224, 64, 7);
impl_algorithm!(Sha256, Algorithm::SHA256, 64, 8);
#[cfg(any(hash_v1, hash_v2, hash_v4))]
impl_algorithm!(Md5, Algorithm::MD5, 64, 4);
#[cfg(hash_v3)]
impl_algorithm!(Sha384, Algorithm::SHA384, 128, 12);
#[cfg(hash_v3)]
impl_algorithm!(Sha512_224, Algorithm::SHA512_224, 128, 7);
#[cfg(hash_v3)]
impl_algorithm!(Sha512_256, Algorithm::SHA512_256, 128, 8);
#[cfg(hash_v3)]
impl_algorithm!(Sha512, Algorithm::SHA512, 128, 16);

#[allow(missing_docs)]
pub trait HmacMode<A: AlgorithmSpec> {
    const HMAC: bool;
    type Key;
    type Csr: CsrStorage;
    const CSR_COUNT: usize;
    fn key(key: A::KeyBuffer) -> Self::Key;
    fn key_ref(key: &Self::Key) -> Option<&[u8]>;
}

#[allow(missing_docs)]
#[derive(Clone, Copy)]
pub struct NonHmac;
#[allow(missing_docs)]
#[derive(Clone, Copy)]
pub struct Hmac;

impl<A: AlgorithmSpec> HmacMode<A> for NonHmac {
    const HMAC: bool = false;
    type Key = ();
    type Csr = A::NonHmacCsr;
    const CSR_COUNT: usize = csr_count(A::ALGORITHM, false);
    fn key(_key: A::KeyBuffer) -> Self::Key {}
    fn key_ref(_key: &Self::Key) -> Option<&[u8]> {
        None
    }
}

impl<A: AlgorithmSpec> HmacMode<A> for Hmac {
    const HMAC: bool = true;
    type Key = A::KeyBuffer;
    type Csr = A::HmacCsr;
    const CSR_COUNT: usize = csr_count(A::ALGORITHM, true);
    fn key(key: A::KeyBuffer) -> Self::Key {
        key
    }
    fn key_ref(key: &Self::Key) -> Option<&[u8]> {
        Some(key.as_slice())
    }
}

/// Input data width selection
#[repr(u8)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, Copy)]
pub enum DataType {
    ///32-bit data, no data is swapped.
    Width32 = 0,
    ///16-bit data, each half-word is swapped.
    Width16 = 1,
    ///8-bit data, all bytes are swapped.
    Width8 = 2,
    ///1-bit data, all bits are swapped.
    Width1 = 3,
}

/// Stores the state of the HASH peripheral for suspending/resuming
/// digest calculation.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone)]
pub struct Context<A: AlgorithmSpec, M: Mode, H: HmacMode<A>>
where
    A: ContextBufferType<M>,
{
    id: u32,
    peripheral_initialized: bool,
    hmac_key_processed: bool,
    /// True when the IN buffer holds the staged trigger word (the saveable
    /// state: NBWP = 1, DINIS = 1). False when the buffer is empty (NBWP = 0,
    /// e.g. right after INIT or after any DCAL-drained phase such as HMAC key
    /// processing). The next feed unit must be block+1 words (17 for
    /// SHA-256) from empty, and exactly one block (16 words) from the
    /// saveable state; see update_blocking. Recorded from hardware at
    /// store_context on hash_v3/v4 (where DINIS = 1 makes NBWP unambiguous);
    /// tracked in software on hash_v1/v2, which have no readable equivalent.
    staged: bool,
    buffer: ContextBuffer<A, M>,
    buflen: usize,
    imr: u32,
    str: u32,
    cr: u32,
    csr: H::Csr,
    key: H::Key,
}

#[allow(missing_docs)]
pub trait ContextBufferType<M: Mode> {
    type Buffer: BufferStorage;
}

impl<A: AlgorithmSpec> ContextBufferType<Blocking> for A {
    type Buffer = A::BlockingBuffer;
}

#[cfg(any(hash_v2, hash_v3, hash_v4))]
impl<A: AlgorithmSpec> ContextBufferType<Async> for A {
    type Buffer = A::AsyncBuffer;
}

type ContextBuffer<A, M> = <A as ContextBufferType<M>>::Buffer;

impl<A: AlgorithmSpec, M: Mode, H: HmacMode<A>> Context<A, M, H>
where
    A: ContextBufferType<M>,
{
    fn buffer(&self) -> &[u8] {
        self.buffer.as_slice()
    }
    fn buffer_mut(&mut self) -> &mut [u8] {
        self.buffer.as_mut_slice()
    }
}

/// HASH driver.
pub struct Hash<'d, T: Instance, M: Mode> {
    _peripheral: Peri<'d, T>,
    _marker: PhantomData<M>,
    current_id: Option<u32>,
    #[cfg(any(hash_v2, hash_v3, hash_v4))]
    dma: Option<ChannelAndRequest<'d>>,
    next_id: u32,
}

/// Returns the number of CSR registers that must be saved/restored
/// for the given algorithm and HMAC mode.
const fn csr_count(algo: Algorithm, hmac: bool) -> usize {
    #[cfg(hash_v1)]
    {
        let _ = algo;
        if hmac { 51 } else { 38 }
    }
    #[cfg(any(hash_v2, hash_v4))]
    {
        let _ = algo;
        if hmac { 54 } else { 38 }
    }
    #[cfg(hash_v3)]
    {
        match algo {
            Algorithm::SHA384 | Algorithm::SHA512_224 | Algorithm::SHA512_256 | Algorithm::SHA512 => {
                if hmac {
                    103
                } else {
                    91
                }
            }
            _ => {
                if hmac {
                    54
                } else {
                    38
                }
            }
        }
    }
}

impl<'d, T: Instance> Hash<'d, T, Blocking> {
    /// Instantiates, resets, and enables the HASH peripheral.
    pub fn new_blocking(
        peripheral: Peri<'d, T>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
    ) -> Self {
        rcc::enable_and_reset::<HASH>();

        Self {
            _peripheral: peripheral,
            _marker: PhantomData,
            current_id: None,
            #[cfg(any(hash_v2, hash_v3, hash_v4))]
            dma: None,
            next_id: 1,
        }
    }
}

impl<'d, T: Instance, M: Mode> Hash<'d, T, M> {
    /// Starts computation of a new hash and returns the saved peripheral state.
    pub fn start<A: AlgorithmSpec, H: HmacMode<A>>(&mut self, format: DataType, key: Option<&[u8]>) -> Context<A, M, H>
    where
        A: ContextBufferType<M>,
    {
        // Batch configure the control register.
        let mut cr = Cr(0);
        cr.set_datatype(format as u8);

        // Select the algorithm.
        #[cfg(hash_v1)]
        if A::ALGORITHM == Algorithm::MD5 {
            cr.set_algo(true);
        }

        #[cfg(hash_v2)]
        {
            let mut algo0 = false;
            let mut algo1 = false;
            if A::ALGORITHM == Algorithm::MD5 || A::ALGORITHM == Algorithm::SHA256 {
                algo0 = true;
            }
            if A::ALGORITHM == Algorithm::SHA224 || A::ALGORITHM == Algorithm::SHA256 {
                algo1 = true;
            }
            cr.set_algo0(algo0);
            cr.set_algo1(algo1);
        }

        #[cfg(any(hash_v3, hash_v4))]
        cr.set_algo(A::ALGORITHM as u8);

        let mut hmac_key = <A::KeyBuffer>::new();
        let mut long_hmac_key = false;
        if H::HMAC {
            let key = key.unwrap_or(&[]);
            if key.len() <= A::BLOCK_SIZE {
                hmac_key.as_mut_slice()[..key.len()].copy_from_slice(key);
            } else {
                long_hmac_key = true;
                cr.set_init(true);
                T::regs().cr().write_value(cr);
                self.accumulate_blocking::<A>(key);
                T::regs().str().write(|w| w.set_dcal(true));
                while !T::regs().sr().read().dcis() {}

                for i in 0..A::DIGEST_WORDS {
                    let word = T::regs().hr(i).read();
                    hmac_key.as_mut_slice()[(i * 4)..((i * 4) + 4)].copy_from_slice(word.to_be_bytes().as_slice());
                }
            }
        }

        // Define a context for this new computation.
        let mut ctx = Context::<A, M, H> {
            id: 0,
            peripheral_initialized: long_hmac_key,
            hmac_key_processed: false,
            staged: false,
            buffer: <ContextBuffer<A, M>>::new(),
            buflen: 0,
            imr: 0,
            str: 0,
            cr: 0,
            csr: H::Csr::new(),
            key: H::key(hmac_key),
        };

        // Configure HMAC mode with the normalized key.
        if H::HMAC {
            cr.set_mode(true);
        }

        cr.set_init(true);

        // Process the normalized HMAC key if requested.
        if long_hmac_key {
            let key = H::key_ref(&ctx.key).unwrap();
            T::regs().cr().write_value(cr);
            self.accumulate_blocking::<A>(key);
            T::regs().str().write(|w| w.set_dcal(true));
            while !T::regs().sr().read().dinis() {}
            ctx.hmac_key_processed = true;
        }

        trace!("start: algo={:?}, format={:?}, key={}", A::ALGORITHM, format, H::HMAC);
        if long_hmac_key {
            self.store_context(&mut ctx);
        } else {
            ctx.cr = cr.0;
            ctx.id = self.next_id;
            self.next_id = self.next_id.wrapping_add(1);
            trace!("start: assigned lazy initial id={}", ctx.id);
        }
        ctx
    }

    /// Restores the peripheral state using the given context,
    /// then updates the state with the provided data.
    /// Peripheral state is saved upon return.
    pub fn update_blocking<A: AlgorithmSpec, H: HmacMode<A>>(&mut self, ctx: &mut Context<A, Blocking, H>, input: &[u8])
    where
        A: ContextBufferType<Blocking>,
    {
        trace!(
            "update_blocking: input_len={}, ctx.buflen={}, ctx.id={}",
            input.len(),
            ctx.buflen,
            ctx.id
        );

        // Feed discipline (RM software feeding + suspend procedure):
        //
        // A block's digest is triggered only by the FIRST WORD OF THE NEXT
        // BLOCK. The saveable state at store_context is therefore "total
        // words fed == 1 (mod 16)": one staged word in DIN, FIFO empty,
        // DINIS = 1. Measured on STM32H563 (v3 dump): 17 words from empty
        // -> NBWP=1 (saveable); 17 words from the saveable state -> NBWP=2
        // (DINIS never returns, store spins forever).
        //
        // Hence the feed unit is 16 words (one block) once the peripheral
        // is in the saveable state, and 17 words (block + trigger) only
        // from the empty state. ctx.buffer holds exactly one 17-word unit
        // (A::BLOCK_SIZE + 4 bytes). DINIS is polled before each unit by
        // accumulate_blocking, so a unit is never written while the
        // previous block's digest is still draining the FIFO.
        let total = input.len() + ctx.buflen;
        let min_feed = if ctx.staged { A::BLOCK_SIZE } else { ctx.buffer().len() };

        if total < min_feed {
            let buflen = ctx.buflen;
            ctx.buffer_mut()[buflen..total].copy_from_slice(input);
            ctx.buflen = total;
            return;
        }

        self.load_context(ctx);

        // The HMAC key phase ends with DCAL, which drains the DIN buffer to
        // empty -- the message that follows restarts at the block+trigger unit.
        let mut unit = if ctx.staged { A::BLOCK_SIZE } else { ctx.buffer().len() };
        if !ctx.hmac_key_processed
            && let Some(key) = H::key_ref(&ctx.key)
        {
            self.accumulate_blocking::<A>(key);
            T::regs().str().write(|w| w.set_dcal(true));
            while !T::regs().sr().read().dinis() {}
            ctx.hmac_key_processed = true;
            ctx.staged = false;
            unit = ctx.buffer().len();
        }

        // Append input to the buffer, feeding one unit at a time.
        let mut remaining = input;
        while ctx.buflen + remaining.len() >= unit {
            let fill = min(unit - ctx.buflen, remaining.len());
            let buflen = ctx.buflen;
            ctx.buffer_mut()[buflen..buflen + fill].copy_from_slice(&remaining[..fill]);
            ctx.buflen += fill;
            remaining = &remaining[fill..];

            if ctx.buflen == unit {
                self.accumulate_blocking::<A>(&ctx.buffer()[..unit]);
                ctx.buflen = 0;
                // A full unit always ends with the trigger word of the next
                // block sitting in the IN buffer (FIFO empty, DIN holding one
                // word): the saveable "staged" state. Tracked in software
                // here; on hash_v3/v4 store_context re-reads it from NBWP/
                // DINNE for hardware truth.
                ctx.staged = true;
                // Subsequent units are whole blocks (16 words).
                unit = A::BLOCK_SIZE;
            }
        }

        // Buffer the tail (< one block). The sub-block remainder is legal
        // here only because finish_blocking ends the message with DCAL.
        let buflen = ctx.buflen;
        ctx.buffer_mut()[buflen..buflen + remaining.len()].copy_from_slice(remaining);
        ctx.buflen += remaining.len();

        self.store_context(ctx);
    }

    /// Computes a digest for the given context.
    /// The digest buffer must be large enough to accomodate a digest for the selected algorithm.
    /// The largest returned digest size is 128 bytes for SHA-512.
    /// Panics if the supplied digest buffer is too short.
    pub fn finish_blocking<A: AlgorithmSpec, H: HmacMode<A>>(
        &mut self,
        mut ctx: Context<A, Blocking, H>,
        digest: &mut [u8],
    ) -> usize
    where
        A: ContextBufferType<Blocking>,
    {
        // Restore the peripheral state.
        self.load_context(&ctx);

        if !ctx.hmac_key_processed
            && let Some(key) = H::key_ref(&ctx.key)
        {
            self.accumulate_blocking::<A>(key);
            T::regs().str().write(|w| w.set_dcal(true));
            while !T::regs().sr().read().dinis() {}
            ctx.hmac_key_processed = true;
        }

        // Hash the leftover bytes, if any.
        self.accumulate_blocking::<A>(&ctx.buffer()[0..ctx.buflen]);
        ctx.buflen = 0;

        // Start the digest calculation.
        T::regs().str().write(|w| w.set_dcal(true));

        // For HMAC, after message digest the peripheral waits for the outer key.
        if let Some(key) = H::key_ref(&ctx.key) {
            while !T::regs().sr().read().dinis() {}
            self.accumulate_blocking::<A>(key);
            T::regs().str().write(|w| w.set_dcal(true));
        }
        // Block until digest computation is complete.
        while !T::regs().sr().read().dcis() {}

        // Return the digest.
        let digest_words = A::DIGEST_WORDS;
        let digest_len_bytes = digest_words * 4;
        // Panics if the supplied digest buffer is too short.
        if digest.len() < digest_len_bytes {
            panic!("Digest buffer must be at least {} bytes long.", digest_len_bytes);
        }

        let mut hr = [0u32; 16];
        for i in 0..digest_words {
            hr[i] = T::regs().hr(i).read();
        }
        for i in 0..digest_words {
            let word = hr[i];
            digest[(i * 4)..((i * 4) + 4)].copy_from_slice(word.to_be_bytes().as_slice());
        }

        digest_len_bytes
    }

    /// Push data into the hash core.
    ///
    /// Per the RM software-feeding procedure, software may write a new
    /// quantum only once DINIS = 1 ("16 IN-buffer locations are free"). One
    /// quantum is NBWE words (the block plus the word that triggers the
    /// block's digest); after each quantum the buffer holds at most the one
    /// staged word in DIN, which is the only state in which the context may
    /// be saved. A partial final quantum is legal only when the caller ends
    /// the message with DCAL (finish and HMAC key paths).
    fn accumulate_blocking<A: AlgorithmSpec>(&mut self, input: &[u8]) {
        if input.is_empty() {
            return;
        }
        // Set the number of valid bits for the final partial word.
        let num_valid_bits: u8 = (8 * (input.len() % 4)) as u8;
        T::regs().str().modify(|w| w.set_nblw(num_valid_bits));

        let quantum = A::BLOCK_SIZE / 4 + 1;
        let total_words = input.len() / 4 + usize::from(!input.len().is_multiple_of(4));
        let mut word = 0;
        while word < total_words {
            if word % quantum == 0 {
                // DINIS = 1: one full quantum can be accepted.
                while !T::regs().sr().read().dinis() {}
            }
            let byte = word * 4;
            let n = min(4, input.len() - byte);
            let mut data: [u8; 4] = [0; 4];
            data[..n].copy_from_slice(&input[byte..byte + n]);
            T::regs().din().write_value(u32::from_ne_bytes(data));
            word += 1;
        }
    }

    /// Save the peripheral state to a context.
    fn store_context<A: AlgorithmSpec, CM: Mode, H: HmacMode<A>>(&mut self, ctx: &mut Context<A, CM, H>)
    where
        A: ContextBufferType<CM>,
    {
        trace!(
            "store_context: old_ctx_id={} -> new_id={}, next={}",
            ctx.id,
            self.next_id,
            self.next_id.wrapping_add(1)
        );

        // RM suspend (software-fed): "wait for BUSY = 0 then poll DINIS".
        // DINIS = 1 guarantees <= 1 word remains in the IN buffer, which is
        // the only state whose context is saveable.
        while T::regs().sr().read().busy() {}
        while !T::regs().sr().read().dinis() {}

        // DINIS = 1 means NBWP <= 1: the buffer either holds exactly the
        // staged trigger word (saveable; next feed unit = one block) or is
        // empty (next unit = block + trigger word). Record which.
        //
        // NBWP/DINNE only exist on hash_v3/hash_v4 status registers. On
        // hash_v1/hash_v2 the identical staged state cannot be read back, so
        // it is tracked in software (see update_blocking); the RM feed
        // discipline is the same on all versions ("16 words, plus one if it
        // is the first block" == NBWE words), so the tracked value matches
        // what the hardware read would return.
        #[cfg(any(hash_v3, hash_v4))]
        {
            let sr = T::regs().sr().read();
            ctx.staged = sr.nbwp() != 0 || sr.dinne();
        }

        // Store peripheral context.
        ctx.imr = T::regs().imr().read().0;
        ctx.str = T::regs().str().read().0;
        ctx.cr = T::regs().cr().read().0;
        let count = H::CSR_COUNT;
        trace!(
            "store_context: saving {} of {} CSR regs for {:?} hmac={}",
            count,
            H::CSR_COUNT,
            A::ALGORITHM,
            H::HMAC
        );
        let mut i = 0;
        while i < count {
            ctx.csr.set(i, T::regs().csr(i).read());
            i += 1;
        }
        ctx.peripheral_initialized = true;
        trace!("store_context: csr[0..{}] saved", count);

        ctx.id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        self.current_id = Some(ctx.id);
        trace!("store_context: saved, current_id set to {}", ctx.id);
    }

    /// Restore the peripheral state from a context.
    fn load_context<A: AlgorithmSpec, CM: Mode, H: HmacMode<A>>(&mut self, ctx: &Context<A, CM, H>)
    where
        A: ContextBufferType<CM>,
    {
        trace!("load_context: current={:?}, ctx.id={}", self.current_id, ctx.id);
        if self.current_id == Some(ctx.id) {
            trace!("load_context: ids match, skipping CSR restore");
            return;
        }
        let count = H::CSR_COUNT;
        trace!(
            "load_context: restoring {} CSR regs for {:?} hmac={}",
            count,
            A::ALGORITHM,
            H::HMAC
        );
        // Restore the peripheral state from the context.
        T::regs().imr().write_value(Imr(ctx.imr));
        T::regs().str().write_value(Str(ctx.str));
        T::regs().cr().write_value(Cr(ctx.cr));
        T::regs().cr().modify(|w| w.set_init(true));
        if ctx.peripheral_initialized {
            for i in 0..count {
                T::regs().csr(i).write_value(ctx.csr.get(i));
            }
        }
        // A full restore rewrites CR+INIT, so the hardware now holds THIS
        // context's state regardless of what current_id said before. Claim
        // ownership: without this, a subsequent ids-match skip on the
        // previously-current context would use clobbered hardware (found by
        // test_boundary_sizes n=68 m=1: finish of a fresh context between
        // store and finish of another silently corrupts the digest).
        self.current_id = Some(ctx.id);
        trace!("load_context: csr[0..{}] restored", count);
    }
}

#[cfg(any(hash_v2, hash_v3, hash_v4))]
impl<'d, T: Instance> Hash<'d, T, Async> {
    /// Instantiates, resets, and enables the HASH peripheral.
    pub fn new<D: Dma<T>>(
        peripheral: Peri<'d, T>,
        dma: Peri<'d, D>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>>
        + interrupt::typelevel::Binding<D::Interrupt, crate::dma::InterruptHandler<D>>
        + 'd,
    ) -> Self {
        rcc::enable_and_reset::<HASH>();
        let instance = Self {
            _peripheral: peripheral,
            _marker: PhantomData,
            current_id: None,
            dma: new_dma!(dma, _irq),
            next_id: 1,
        };

        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        instance
    }

    /// Restores the peripheral state using the given context,
    /// then updates the state with the provided data.
    /// Peripheral state is saved upon return.
    pub async fn update<A: AlgorithmSpec, H: HmacMode<A>>(&mut self, ctx: &mut Context<A, Async, H>, input: &[u8])
    where
        A: ContextBufferType<Async>,
    {
        trace!(
            "update: input_len={}, ctx.buflen={}, ctx.id={}",
            input.len(),
            ctx.buflen,
            ctx.id
        );

        let bs = A::BLOCK_SIZE;
        let total = ctx.buflen + input.len();
        // Buffer data if there isn't enough to both DMA and leave a block for the context-save release.
        let buffer_len = ctx.buffer().len();
        if total < buffer_len {
            let buflen = ctx.buflen;
            ctx.buffer_mut()[buflen..buflen + input.len()].copy_from_slice(input);
            ctx.buflen += input.len();
            return;
        }

        // Restore the peripheral state.
        self.load_context(ctx);

        if !ctx.hmac_key_processed
            && let Some(key) = H::key_ref(&ctx.key)
        {
            self.accumulate(key).await;
            T::regs().str().write(|w| w.set_dcal(true));
            while !T::regs().sr().read().dinis() {}
            ctx.hmac_key_processed = true;
        }

        // Enable multiple DMA transfers.
        T::regs().cr().modify(|w| w.set_mdmat(true));

        // Reserve the last full block and any tail bytes.
        let tail = total % bs;
        let reserve = bs + tail;
        let feed = total - reserve;

        // Extract the reserve before DMA starts to ensure a contiguous buffer for manual writing.
        let mut scratch = [0u8; 2 * 128];
        for (i, p) in (feed..total).enumerate() {
            scratch[i] = if p < ctx.buflen {
                ctx.buffer()[p]
            } else {
                input[p - ctx.buflen]
            };
        }

        // DMA the data in block-aligned chunks.
        if feed <= ctx.buflen {
            self.accumulate(&ctx.buffer()[..feed]).await;
        } else {
            let buf_blocks = ctx.buflen / bs * bs;
            if buf_blocks > 0 {
                self.accumulate(&ctx.buffer()[..buf_blocks]).await;
            }
            let buf_rem = ctx.buflen - buf_blocks;
            let mut in_idx = 0;
            if buf_rem > 0 {
                let buflen = ctx.buflen;
                ctx.buffer_mut().copy_within(buf_blocks..buflen, 0);
                let need = bs - buf_rem;
                ctx.buffer_mut()[buf_rem..bs].copy_from_slice(&input[..need]);
                self.accumulate(&ctx.buffer()[..bs]).await;
                in_idx = need;
            }
            let in_blocks = feed - buf_blocks - if buf_rem > 0 { bs } else { 0 };
            if in_blocks > 0 {
                self.accumulate(&input[in_idx..in_idx + in_blocks]).await;
            }
        }

        // The peripheral holds the last DMA'd block in DIN. Push reserved words
        // manually to force the core to drain, making the context saveable.
        let mut rp = 0;
        while rp + 4 <= reserve && !T::regs().sr().read().dinis() {
            T::regs()
                .din()
                .write_value(u32::from_ne_bytes(scratch[rp..rp + 4].try_into().unwrap()));
            while T::regs().sr().read().busy() {}
            rp += 4;
        }
        ctx.buffer_mut()[..reserve - rp].copy_from_slice(&scratch[rp..reserve]);
        ctx.buflen = reserve - rp;

        // Save the peripheral context.
        self.store_context(ctx);
    }

    /// Computes a digest for the given context.
    /// The digest buffer must be large enough to accomodate a digest for the selected algorithm.
    /// The largest returned digest size is 128 bytes for SHA-512.
    /// Panics if the supplied digest buffer is too short.
    pub async fn finish<A: AlgorithmSpec, H: HmacMode<A>>(
        &mut self,
        mut ctx: Context<A, Async, H>,
        digest: &mut [u8],
    ) -> usize
    where
        A: ContextBufferType<Async>,
    {
        // Restore the peripheral state.
        self.load_context(&ctx);

        if !ctx.hmac_key_processed
            && let Some(key) = H::key_ref(&ctx.key)
        {
            self.accumulate(key).await;
            T::regs().str().write(|w| w.set_dcal(true));
            while !T::regs().sr().read().dinis() {}
            ctx.hmac_key_processed = true;
        }

        // Must be cleared prior to the last DMA transfer.
        T::regs().cr().modify(|w| w.set_mdmat(false));

        // Finalize the hash. Carried bytes automatically trigger the digest;
        // otherwise, we must trigger it manually.
        if ctx.buflen > 0 {
            self.accumulate(&ctx.buffer()[0..ctx.buflen]).await;
        }
        T::regs().str().write(|w| w.set_dcal(true));
        ctx.buflen = 0;

        // For HMAC, after message digest the peripheral waits for the outer key.
        if let Some(key) = H::key_ref(&ctx.key) {
            while !T::regs().sr().read().dinis() {}
            self.accumulate(key).await;
            T::regs().str().write(|w| w.set_dcal(true));
        }
        // Wait for completion.
        poll_fn(|cx| {
            // Check if already done.
            let bits = T::regs().sr().read();
            if bits.dcis() {
                return Poll::Ready(());
            }
            // Register waker, then enable interrupts.
            HASH_WAKER.register(cx.waker());
            T::regs().imr().modify(|reg| reg.set_dcie(true));
            // Check for completion.
            let bits = T::regs().sr().read();
            if bits.dcis() { Poll::Ready(()) } else { Poll::Pending }
        })
        .await;

        // Return the digest.
        let digest_words = A::DIGEST_WORDS;
        let digest_len_bytes = digest_words * 4;
        // Panics if the supplied digest buffer is too short.
        if digest.len() < digest_len_bytes {
            panic!("Digest buffer must be at least {} bytes long.", digest_len_bytes);
        }

        let mut hr = [0u32; 16];
        for i in 0..digest_words {
            hr[i] = T::regs().hr(i).read();
        }
        for i in 0..digest_words {
            let word = hr[i];
            digest[(i * 4)..((i * 4) + 4)].copy_from_slice(word.to_be_bytes().as_slice());
        }
        digest_len_bytes
    }

    /// Push data into the hash core.
    async fn accumulate(&mut self, input: &[u8]) {
        // Ignore an input length of 0.
        if input.is_empty() {
            return;
        }

        // Set the number of valid bits.
        let num_valid_bits: u8 = (8 * (input.len() % 4)) as u8;
        T::regs().str().modify(|w| w.set_nblw(num_valid_bits));

        // Configure DMA to transfer input to hash core.
        let dst_ptr: *mut u32 = T::regs().din().as_ptr();
        let mut num_words = input.len() / 4;
        if !input.len().is_multiple_of(4) {
            num_words += 1;
        }
        let src_ptr: *const [u8] = ptr::slice_from_raw_parts(input.as_ptr().cast(), num_words * 4);

        let dma = self.dma.as_mut().unwrap();
        let dma_transfer = unsafe { dma.write_raw(src_ptr, dst_ptr as *mut u32, Default::default()) };
        T::regs().cr().modify(|w| w.set_dmae(true));

        // Wait for the transfer to complete.
        dma_transfer.await;
    }
}

impl<'d> SealedSuspendablePeripheral for Hash<'d, HASH, Blocking> {
    type InternalState = (Option<u32>, u32);

    fn resume(state: Self::InternalState) -> Self {
        critical_section::with(rcc::enable_and_reset_with_cs_no_refcount::<HASH>);

        Self {
            _peripheral: unsafe { core::mem::transmute(()) },
            _marker: PhantomData,
            current_id: state.0,
            #[cfg(any(hash_v2, hash_v3, hash_v4))]
            dma: None,
            next_id: state.1,
        }
    }

    fn suspend(self) -> Self::InternalState {
        (self.current_id, self.next_id)
    }
}

mod driver {
    use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
    use embassy_sync::mutex::Mutex;

    #[cfg(any(hash_v1, hash_v2, hash_v4))]
    use crate::hash::Md5;
    use crate::hash::{Context, DataType, Hash, Hmac, NonHmac, Sha1, Sha224, Sha256};
    #[cfg(hash_v3)]
    use crate::hash::{Sha384, Sha512, Sha512_224, Sha512_256};
    use crate::mode::Blocking;
    use crate::peripherals::HASH;
    use crate::suspend::ResumablePeripheral;

    static DRIVER: Mutex<CriticalSectionRawMutex, ResumablePeripheral<Hash<'static, HASH, Blocking>>> =
        Mutex::new(ResumablePeripheral::new_suspended((None, 0)));

    // =====================================================================
    // Digest driver macro
    // =====================================================================

    macro_rules! impl_digest_driver {
        (
            $(#[$meta:meta])*
            $driver:ident, $trait:path, $algo:ty,
            $impl_macro:path
        ) => {
            $(#[$meta])*
            struct $driver;
            impl $trait for $driver {
                type Context = Context<$algo, Blocking, NonHmac>;

                fn init() -> Self::Context {
                    DRIVER.try_lock().unwrap().borrow().start(DataType::Width8, None)
                }

                fn update(ctx: &mut Self::Context, data: &[u8]) {
                    DRIVER.try_lock().unwrap().borrow().update_blocking(ctx, data)
                }

                fn finalize(ctx: Self::Context, data: &mut [u8]) {
                    DRIVER.try_lock().unwrap().borrow().finish_blocking(ctx, data);
                }
            }
            $impl_macro!($driver);
        };
    }

    // =====================================================================
    // HMAC driver macro
    // =====================================================================

    macro_rules! impl_hmac_driver {
        (
            $(#[$meta:meta])*
            $driver:ident, $trait:path, $algo:ty,
            $impl_macro:path
        ) => {
            $(#[$meta])*
            struct $driver;
            impl $trait for $driver {
                type Context = Context<$algo, Blocking, Hmac>;

                fn init(key: &[u8]) -> Self::Context {
                    DRIVER.try_lock().unwrap().borrow().start(DataType::Width8, Some(key))
                }

                fn update(ctx: &mut Self::Context, data: &[u8]) {
                    DRIVER.try_lock().unwrap().borrow().update_blocking(ctx, data)
                }

                fn finalize(ctx: Self::Context, data: &mut [u8]) {
                    DRIVER.try_lock().unwrap().borrow().finish_blocking(ctx, data);
                }

            }
            $impl_macro!($driver);
        };
    }

    // =====================================================================
    // Digest drivers
    // =====================================================================
    #[cfg(any(hash_v1, hash_v2, hash_v4))]
    impl_digest_driver!(
        Md5Driver,
        embassy_crypto_driver::Md5,
        Md5,
        embassy_crypto_driver::md5_impl
    );

    impl_digest_driver!(
        Sha1Driver,
        embassy_crypto_driver::Sha1,
        Sha1,
        embassy_crypto_driver::sha1_impl
    );

    impl_digest_driver!(
        Sha224Driver,
        embassy_crypto_driver::Sha224,
        Sha224,
        embassy_crypto_driver::sha224_impl
    );

    impl_digest_driver!(
        Sha256Driver,
        embassy_crypto_driver::Sha256,
        Sha256,
        embassy_crypto_driver::sha256_impl
    );

    #[cfg(hash_v3)]
    impl_digest_driver!(
        Sha384Driver,
        embassy_crypto_driver::Sha384,
        Sha384,
        embassy_crypto_driver::sha384_impl
    );

    #[cfg(hash_v3)]
    impl_digest_driver!(
        Sha512_224Driver,
        embassy_crypto_driver::Sha512_224,
        Sha512_224,
        embassy_crypto_driver::sha512_224_impl
    );

    #[cfg(hash_v3)]
    impl_digest_driver!(
        Sha512_256Driver,
        embassy_crypto_driver::Sha512_256,
        Sha512_256,
        embassy_crypto_driver::sha512_256_impl
    );

    #[cfg(hash_v3)]
    impl_digest_driver!(
        Sha512Driver,
        embassy_crypto_driver::Sha512,
        Sha512,
        embassy_crypto_driver::sha512_impl
    );

    // =====================================================================
    // HMAC drivers
    // =====================================================================

    impl_hmac_driver!(
        HmacSha1Driver,
        embassy_crypto_driver::HmacSha1,
        Sha1,
        embassy_crypto_driver::hmac_sha1_impl
    );

    impl_hmac_driver!(
        HmacSha224Driver,
        embassy_crypto_driver::HmacSha224,
        Sha224,
        embassy_crypto_driver::hmac_sha224_impl
    );

    impl_hmac_driver!(
        HmacSha256Driver,
        embassy_crypto_driver::HmacSha256,
        Sha256,
        embassy_crypto_driver::hmac_sha256_impl
    );

    #[cfg(hash_v3)]
    impl_hmac_driver!(
        HmacSha384Driver,
        embassy_crypto_driver::HmacSha384,
        Sha384,
        embassy_crypto_driver::hmac_sha384_impl
    );

    #[cfg(hash_v3)]
    impl_hmac_driver!(
        HmacSha512_224Driver,
        embassy_crypto_driver::HmacSha512_224,
        Sha512_224,
        embassy_crypto_driver::hmac_sha512_224_impl
    );

    #[cfg(hash_v3)]
    impl_hmac_driver!(
        HmacSha512_256Driver,
        embassy_crypto_driver::HmacSha512_256,
        Sha512_256,
        embassy_crypto_driver::hmac_sha512_256_impl
    );

    #[cfg(hash_v3)]
    impl_hmac_driver!(
        HmacSha512Driver,
        embassy_crypto_driver::HmacSha512,
        Sha512,
        embassy_crypto_driver::hmac_sha512_impl
    );
}

trait SealedInstance {
    fn regs() -> pac::hash::Hash;
}

/// HASH instance trait.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + crate::rcc::RccPeripheral + 'static + Send {
    /// Interrupt for this HASH instance.
    type Interrupt: interrupt::typelevel::Interrupt;
}

foreach_interrupt!(
    ($inst:ident, hash, HASH, GLOBAL, $irq:ident) => {
        impl Instance for peripherals::$inst {
            type Interrupt = crate::interrupt::typelevel::$irq;
        }

        impl SealedInstance for peripherals::$inst {
            fn regs() -> crate::pac::hash::Hash {
                crate::pac::$inst
            }
        }
    };
);

dma_trait!(Dma, Instance);
