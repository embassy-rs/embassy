//! Hash generator (HASH)
use core::cmp::min;
#[cfg(hash_v2)]
use core::future::poll_fn;
use core::marker::PhantomData;
#[cfg(hash_v2)]
use core::ptr;
#[cfg(hash_v2)]
use core::task::Poll;

use embassy_hal_internal::PeripheralType;
use embassy_sync::waitqueue::AtomicWaker;
use stm32_metapac::hash::regs::*;

#[cfg(hash_v2)]
use crate::dma::ChannelAndRequest;
use crate::interrupt::typelevel::Interrupt;
#[cfg(hash_v2)]
use crate::mode::Async;
use crate::mode::{Blocking, Mode};
use crate::peripherals::HASH;
use crate::{interrupt, pac, peripherals, rcc, Peri};

#[cfg(hash_v1)]
const NUM_CONTEXT_REGS: usize = 51;
#[cfg(hash_v3)]
const NUM_CONTEXT_REGS: usize = 103;
#[cfg(any(hash_v2, hash_v4))]
const NUM_CONTEXT_REGS: usize = 54;

const HASH_BUFFER_LEN: usize = 132;
const DIGEST_BLOCK_SIZE: usize = 128;

static HASH_WAKER: AtomicWaker = AtomicWaker::new();

/// HASH interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
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
#[derive(Clone, Copy, PartialEq)]
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

/// Input data width selection
#[repr(u8)]
#[derive(Clone, Copy)]
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
#[derive(Clone)]
pub struct Context<'c> {
    first_word_sent: bool,
    key_sent: bool,
    buffer: [u8; HASH_BUFFER_LEN],
    buflen: usize,
    algo: Algorithm,
    format: DataType,
    imr: u32,
    str: u32,
    cr: u32,
    csr: [u32; NUM_CONTEXT_REGS],
    key: HmacKey<'c>,
}

type HmacKey<'k> = Option<&'k [u8]>;

/// HASH driver.
pub struct Hash<'d, T: Instance, M: Mode> {
    _peripheral: Peri<'d, T>,
    _phantom: PhantomData<M>,
    #[cfg(hash_v2)]
    dma: Option<ChannelAndRequest<'d>>,
}

impl<'d, T: Instance> Hash<'d, T, Blocking> {
    /// Instantiates, resets, and enables the HASH peripheral.
    pub fn new_blocking(
        peripheral: Peri<'d, T>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
    ) -> Self {
        rcc::enable_and_reset::<HASH>();
        let instance = Self {
            _peripheral: peripheral,
            _phantom: PhantomData,
            #[cfg(hash_v2)]
            dma: None,
        };

        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        instance
    }
}

impl<'d, T: Instance, M: Mode> Hash<'d, T, M> {
    /// Starts computation of a new hash and returns the saved peripheral state.
    pub fn start<'c>(&mut self, algorithm: Algorithm, format: DataType, key: HmacKey<'c>) -> Context<'c> {
        // Define a context for this new computation.
        let mut ctx = Context {
            first_word_sent: false,
            key_sent: false,
            buffer: [0; HASH_BUFFER_LEN],
            buflen: 0,
            algo: algorithm,
            format: format,
            imr: 0,
            str: 0,
            cr: 0,
            csr: [0; NUM_CONTEXT_REGS],
            key,
        };

        // Set the data type in the peripheral.
        T::regs().cr().modify(|w| w.set_datatype(ctx.format as u8));

        // Select the algorithm.
        #[cfg(hash_v1)]
        if ctx.algo == Algorithm::MD5 {
            T::regs().cr().modify(|w| w.set_algo(true));
        }

        #[cfg(hash_v2)]
        {
            // Select the algorithm.
            let mut algo0 = false;
            let mut algo1 = false;
            if ctx.algo == Algorithm::MD5 || ctx.algo == Algorithm::SHA256 {
                algo0 = true;
            }
            if ctx.algo == Algorithm::SHA224 || ctx.algo == Algorithm::SHA256 {
                algo1 = true;
            }
            T::regs().cr().modify(|w| w.set_algo0(algo0));
            T::regs().cr().modify(|w| w.set_algo1(algo1));
        }

        #[cfg(any(hash_v3, hash_v4))]
        T::regs().cr().modify(|w| w.set_algo(ctx.algo as u8));

        // Configure HMAC mode if a key is provided.
        if let Some(key) = ctx.key {
            T::regs().cr().modify(|w| w.set_mode(true));
            if key.len() > 64 {
                T::regs().cr().modify(|w| w.set_lkey(true));
            }
        }

        T::regs().cr().modify(|w| w.set_init(true));

        // Store and return the state of the peripheral.
        self.store_context(&mut ctx);
        ctx
    }

    /// Restores the peripheral state using the given context,
    /// then updates the state with the provided data.
    /// Peripheral state is saved upon return.
    pub fn update_blocking<'c>(&mut self, ctx: &mut Context<'c>, input: &[u8]) {
        // Restore the peripheral state.
        self.load_context(&ctx);

        // Load the HMAC key if provided.
        if !ctx.key_sent {
            if let Some(key) = ctx.key {
                self.accumulate_blocking(key);
                T::regs().str().write(|w| w.set_dcal(true));
                // Block waiting for digest.
                while !T::regs().sr().read().dinis() {}
            }
            ctx.key_sent = true;
        }

        let mut data_waiting = input.len() + ctx.buflen;
        if data_waiting < DIGEST_BLOCK_SIZE || (data_waiting < ctx.buffer.len() && !ctx.first_word_sent) {
            // There isn't enough data to digest a block, so append it to the buffer.
            ctx.buffer[ctx.buflen..ctx.buflen + input.len()].copy_from_slice(input);
            ctx.buflen += input.len();
            self.store_context(ctx);
            return;
        }

        let mut ilen_remaining = input.len();
        let mut input_start = 0;

        // Handle first block.
        if !ctx.first_word_sent {
            let empty_len = ctx.buffer.len() - ctx.buflen;
            let copy_len = min(empty_len, ilen_remaining);
            // Fill the buffer.
            if copy_len > 0 {
                ctx.buffer[ctx.buflen..ctx.buflen + copy_len].copy_from_slice(&input[0..copy_len]);
                ctx.buflen += copy_len;
                ilen_remaining -= copy_len;
                input_start += copy_len;
            }
            self.accumulate_blocking(ctx.buffer.as_slice());
            data_waiting -= ctx.buflen;
            ctx.buflen = 0;
            ctx.first_word_sent = true;
        }

        if data_waiting < DIGEST_BLOCK_SIZE {
            // There isn't enough data remaining to process another block, so store it.
            ctx.buffer[0..ilen_remaining].copy_from_slice(&input[input_start..input_start + ilen_remaining]);
            ctx.buflen += ilen_remaining;
        } else {
            // First ingest the data in the buffer.
            let empty_len = DIGEST_BLOCK_SIZE - ctx.buflen;
            if empty_len > 0 {
                let copy_len = min(empty_len, ilen_remaining);
                ctx.buffer[ctx.buflen..ctx.buflen + copy_len]
                    .copy_from_slice(&input[input_start..input_start + copy_len]);
                ctx.buflen += copy_len;
                ilen_remaining -= copy_len;
                input_start += copy_len;
            }
            self.accumulate_blocking(&ctx.buffer[0..DIGEST_BLOCK_SIZE]);
            ctx.buflen = 0;

            // Move any extra data to the now-empty buffer.
            let leftovers = ilen_remaining % 64;
            if leftovers > 0 {
                ctx.buffer[0..leftovers].copy_from_slice(&input[input.len() - leftovers..input.len()]);
                ctx.buflen += leftovers;
                ilen_remaining -= leftovers;
            }

            // Hash the remaining data.
            self.accumulate_blocking(&input[input_start..input_start + ilen_remaining]);
        }

        // Save the peripheral context.
        self.store_context(ctx);
    }

    /// Computes a digest for the given context.
    /// The digest buffer must be large enough to accomodate a digest for the selected algorithm.
    /// The largest returned digest size is 128 bytes for SHA-512.
    /// Panics if the supplied digest buffer is too short.
    pub fn finish_blocking<'c>(&mut self, mut ctx: Context<'c>, digest: &mut [u8]) -> usize {
        // Restore the peripheral state.
        self.load_context(&ctx);

        // Hash the leftover bytes, if any.
        self.accumulate_blocking(&ctx.buffer[0..ctx.buflen]);
        ctx.buflen = 0;

        //Start the digest calculation.
        T::regs().str().write(|w| w.set_dcal(true));

        // Load the HMAC key if provided.
        if let Some(key) = ctx.key {
            while !T::regs().sr().read().dinis() {}
            self.accumulate_blocking(key);
            T::regs().str().write(|w| w.set_dcal(true));
        }

        // Block until digest computation is complete.
        while !T::regs().sr().read().dcis() {}

        // Return the digest.
        let digest_words = match ctx.algo {
            Algorithm::SHA1 => 5,
            #[cfg(any(hash_v1, hash_v2, hash_v4))]
            Algorithm::MD5 => 4,
            Algorithm::SHA224 => 7,
            Algorithm::SHA256 => 8,
            #[cfg(hash_v3)]
            Algorithm::SHA384 => 12,
            #[cfg(hash_v3)]
            Algorithm::SHA512_224 => 7,
            #[cfg(hash_v3)]
            Algorithm::SHA512_256 => 8,
            #[cfg(hash_v3)]
            Algorithm::SHA512 => 16,
        };

        let digest_len_bytes = digest_words * 4;
        // Panics if the supplied digest buffer is too short.
        if digest.len() < digest_len_bytes {
            panic!("Digest buffer must be at least {} bytes long.", digest_words * 4);
        }

        let mut i = 0;
        while i < digest_words {
            let word = T::regs().hr(i).read();
            digest[(i * 4)..((i * 4) + 4)].copy_from_slice(word.to_be_bytes().as_slice());
            i += 1;
        }
        digest_len_bytes
    }

    /// Push data into the hash core.
    fn accumulate_blocking(&mut self, input: &[u8]) {
        // Set the number of valid bits.
        let num_valid_bits: u8 = (8 * (input.len() % 4)) as u8;
        T::regs().str().modify(|w| w.set_nblw(num_valid_bits));

        let mut i = 0;
        while i < input.len() {
            let mut word: [u8; 4] = [0; 4];
            let copy_idx = min(i + 4, input.len());
            word[0..copy_idx - i].copy_from_slice(&input[i..copy_idx]);
            T::regs().din().write_value(u32::from_ne_bytes(word));
            i += 4;
        }
    }

    /// Save the peripheral state to a context.
    fn store_context<'c>(&mut self, ctx: &mut Context<'c>) {
        // Block waiting for data in ready.
        while !T::regs().sr().read().dinis() {}

        // Store peripheral context.
        ctx.imr = T::regs().imr().read().0;
        ctx.str = T::regs().str().read().0;
        ctx.cr = T::regs().cr().read().0;
        let mut i = 0;
        while i < NUM_CONTEXT_REGS {
            ctx.csr[i] = T::regs().csr(i).read();
            i += 1;
        }
    }

    /// Restore the peripheral state from a context.
    fn load_context(&mut self, ctx: &Context) {
        // Restore the peripheral state from the context.
        T::regs().imr().write_value(Imr { 0: ctx.imr });
        T::regs().str().write_value(Str { 0: ctx.str });
        T::regs().cr().write_value(Cr { 0: ctx.cr });
        T::regs().cr().modify(|w| w.set_init(true));
        let mut i = 0;
        while i < NUM_CONTEXT_REGS {
            T::regs().csr(i).write_value(ctx.csr[i]);
            i += 1;
        }
    }
}

#[cfg(hash_v2)]
impl<'d, T: Instance> Hash<'d, T, Async> {
    /// Instantiates, resets, and enables the HASH peripheral.
    pub fn new(
        peripheral: Peri<'d, T>,
        dma: Peri<'d, impl Dma<T>>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
    ) -> Self {
        rcc::enable_and_reset::<HASH>();
        let instance = Self {
            _peripheral: peripheral,
            _phantom: PhantomData,
            dma: new_dma!(dma),
        };

        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        instance
    }

    /// Restores the peripheral state using the given context,
    /// then updates the state with the provided data.
    /// Peripheral state is saved upon return.
    pub async fn update(&mut self, ctx: &mut Context<'_>, input: &[u8]) {
        // Restore the peripheral state.
        self.load_context(&ctx);

        // Load the HMAC key if provided.
        if !ctx.key_sent {
            if let Some(key) = ctx.key {
                self.accumulate(key).await;
            }
            ctx.key_sent = true;
        }

        let data_waiting = input.len() + ctx.buflen;
        if data_waiting < DIGEST_BLOCK_SIZE {
            // There isn't enough data to digest a block, so append it to the buffer.
            ctx.buffer[ctx.buflen..ctx.buflen + input.len()].copy_from_slice(input);
            ctx.buflen += input.len();
            self.store_context(ctx);
            return;
        }

        // Enable multiple DMA transfers.
        T::regs().cr().modify(|w| w.set_mdmat(true));

        let mut ilen_remaining = input.len();
        let mut input_start = 0;

        // First ingest the data in the buffer.
        let empty_len = DIGEST_BLOCK_SIZE - ctx.buflen;
        if empty_len > 0 {
            let copy_len = min(empty_len, ilen_remaining);
            ctx.buffer[ctx.buflen..ctx.buflen + copy_len].copy_from_slice(&input[input_start..input_start + copy_len]);
            ctx.buflen += copy_len;
            ilen_remaining -= copy_len;
            input_start += copy_len;
        }
        self.accumulate(&ctx.buffer[..DIGEST_BLOCK_SIZE]).await;
        ctx.buflen = 0;

        // Move any extra data to the now-empty buffer.
        let leftovers = ilen_remaining % DIGEST_BLOCK_SIZE;
        if leftovers > 0 {
            assert!(ilen_remaining >= leftovers);
            ctx.buffer[0..leftovers].copy_from_slice(&input[input.len() - leftovers..input.len()]);
            ctx.buflen += leftovers;
            ilen_remaining -= leftovers;
        } else {
            ctx.buffer
                .copy_from_slice(&input[input.len() - DIGEST_BLOCK_SIZE..input.len()]);
            ctx.buflen += DIGEST_BLOCK_SIZE;
            ilen_remaining -= DIGEST_BLOCK_SIZE;
        }

        // Hash the remaining data.
        self.accumulate(&input[input_start..input_start + ilen_remaining]).await;

        // Save the peripheral context.
        self.store_context(ctx);
    }

    /// Computes a digest for the given context.
    /// The digest buffer must be large enough to accomodate a digest for the selected algorithm.
    /// The largest returned digest size is 128 bytes for SHA-512.
    /// Panics if the supplied digest buffer is too short.
    pub async fn finish<'c>(&mut self, mut ctx: Context<'c>, digest: &mut [u8]) -> usize {
        // Restore the peripheral state.
        self.load_context(&ctx);

        // Must be cleared prior to the last DMA transfer.
        T::regs().cr().modify(|w| w.set_mdmat(false));

        // Hash the leftover bytes, if any.
        self.accumulate(&ctx.buffer[0..ctx.buflen]).await;
        ctx.buflen = 0;

        // Load the HMAC key if provided.
        if let Some(key) = ctx.key {
            self.accumulate(key).await;
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
            if bits.dcis() {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await;

        // Return the digest.
        let digest_words = match ctx.algo {
            Algorithm::SHA1 => 5,
            #[cfg(any(hash_v1, hash_v2, hash_v4))]
            Algorithm::MD5 => 4,
            Algorithm::SHA224 => 7,
            Algorithm::SHA256 => 8,
            #[cfg(hash_v3)]
            Algorithm::SHA384 => 12,
            #[cfg(hash_v3)]
            Algorithm::SHA512_224 => 7,
            #[cfg(hash_v3)]
            Algorithm::SHA512_256 => 8,
            #[cfg(hash_v3)]
            Algorithm::SHA512 => 16,
        };

        let digest_len_bytes = digest_words * 4;
        // Panics if the supplied digest buffer is too short.
        if digest.len() < digest_len_bytes {
            panic!("Digest buffer must be at least {} bytes long.", digest_words * 4);
        }

        let mut i = 0;
        while i < digest_words {
            let word = T::regs().hr(i).read();
            digest[(i * 4)..((i * 4) + 4)].copy_from_slice(word.to_be_bytes().as_slice());
            i += 1;
        }
        digest_len_bytes
    }

    /// Push data into the hash core.
    async fn accumulate(&mut self, input: &[u8]) {
        // Ignore an input length of 0.
        if input.len() == 0 {
            return;
        }

        // Set the number of valid bits.
        let num_valid_bits: u8 = (8 * (input.len() % 4)) as u8;
        T::regs().str().modify(|w| w.set_nblw(num_valid_bits));

        // Configure DMA to transfer input to hash core.
        let dst_ptr: *mut u32 = T::regs().din().as_ptr();
        let mut num_words = input.len() / 4;
        if input.len() % 4 > 0 {
            num_words += 1;
        }
        let src_ptr: *const [u8] = ptr::slice_from_raw_parts(input.as_ptr().cast(), num_words);

        let dma = self.dma.as_mut().unwrap();
        let dma_transfer = unsafe { dma.write_raw(src_ptr, dst_ptr as *mut u32, Default::default()) };
        T::regs().cr().modify(|w| w.set_dmae(true));

        // Wait for the transfer to complete.
        dma_transfer.await;
    }
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
