//! Hash generator (HASH)
use core::cmp::min;
use core::future::poll_fn;
use core::marker::PhantomData;
use core::ptr;
use core::task::Poll;

use embassy_hal_internal::{into_ref, PeripheralRef};
use embassy_sync::waitqueue::AtomicWaker;

use crate::dma::Transfer;
use crate::peripherals::HASH;
use stm32_metapac::hash::regs::*;

use crate::interrupt::typelevel::Interrupt;
use crate::rcc::sealed::RccPeripheral;
use crate::{interrupt, pac, peripherals, Peripheral};

#[cfg(hash_v1)]
const NUM_CONTEXT_REGS: usize = 51;
#[cfg(hash_v2)]
const NUM_CONTEXT_REGS: usize = 54;
const DIGEST_BLOCK_SIZE: usize = 64;

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
#[derive(PartialEq)]
pub enum Algorithm {
    /// SHA-1 Algorithm
    SHA1 = 0,
    /// MD5 Algorithm
    MD5 = 1,
    #[cfg(hash_v2)]
    /// SHA-224 Algorithm
    SHA224 = 2,
    #[cfg(hash_v2)]
    /// SHA-256 Algorithm
    SHA256 = 3,
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
pub struct Context {
    buffer: [u8; DIGEST_BLOCK_SIZE],
    buflen: usize,
    algo: Algorithm,
    format: DataType,
    imr: u32,
    str: u32,
    cr: u32,
    csr: [u32; NUM_CONTEXT_REGS],
}

/// HASH driver.
pub struct Hash<'d, T: Instance, D: Dma<T>> {
    _peripheral: PeripheralRef<'d, T>,
    dma: PeripheralRef<'d, D>,
}

impl<'d, T: Instance, D: Dma<T>> Hash<'d, T, D> {
    /// Instantiates, resets, and enables the HASH peripheral.
    pub fn new(peripheral: impl Peripheral<P = T> + 'd, dma: impl Peripheral<P = D> + 'd) -> Self {
        HASH::enable_and_reset();
        into_ref!(peripheral, dma);
        let instance = Self {
            _peripheral: peripheral,
            dma: dma,
        };

        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        instance
    }

    /// Starts computation of a new hash and returns the saved peripheral state.
    pub async fn start(&mut self, algorithm: Algorithm, format: DataType) -> Context {
        // Define a context for this new computation.
        let mut ctx = Context {
            buffer: [0; DIGEST_BLOCK_SIZE],
            buflen: 0,
            algo: algorithm,
            format: format,
            imr: 0,
            str: 0,
            cr: 0,
            csr: [0; NUM_CONTEXT_REGS],
        };

        // Set the data type in the peripheral.
        T::regs().cr().modify(|w| w.set_datatype(ctx.format as u8));

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

        // Enable multiple DMA transfers.
        T::regs().cr().modify(|w| w.set_mdmat(true));

        // Set init to load the context registers. Necessary before storing context.
        T::regs().cr().modify(|w| w.set_init(true));

        // Store and return the state of the peripheral.
        self.store_context(&mut ctx).await;
        ctx
    }

    /// Restores the peripheral state using the given context,
    /// then updates the state with the provided data.
    /// Peripheral state is saved upon return.
    pub async fn update(&mut self, ctx: &mut Context, input: &[u8]) {
        let data_waiting = input.len() + ctx.buflen;
        if data_waiting < DIGEST_BLOCK_SIZE {
            // There isn't enough data to digest a block, so append it to the buffer.
            ctx.buffer[ctx.buflen..ctx.buflen + input.len()].copy_from_slice(input);
            ctx.buflen += input.len();
            return;
        }

        // Restore the peripheral state.
        self.load_context(&ctx);

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
        self.accumulate(&ctx.buffer).await;
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
        self.store_context(ctx).await;
    }

    /// Computes a digest for the given context. A slice of the provided digest buffer is returned.
    /// The length of the returned slice is dependent on the digest length of the selected algorithm.
    pub async fn finish<'a>(&mut self, mut ctx: Context, digest: &'a mut [u8; 32]) -> &'a [u8] {
        // Restore the peripheral state.
        self.load_context(&ctx);

        // Must be cleared prior to the last DMA transfer.
        T::regs().cr().modify(|w| w.set_mdmat(false));

        // Hash the leftover bytes, if any.
        self.accumulate(&ctx.buffer[0..ctx.buflen]).await;
        ctx.buflen = 0;

        // Wait for completion.
        poll_fn(|cx| {
            // Check if already done.
            let bits = T::regs().sr().read();
            if bits.dcis() {
                return Poll::Ready(());
            }
            // Register waker, then enable interrupts.
            HASH_WAKER.register(cx.waker());
            T::regs().imr().modify(|reg| reg.set_dinie(true));
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
            Algorithm::MD5 => 4,
            Algorithm::SHA224 => 7,
            Algorithm::SHA256 => 8,
        };
        let mut i = 0;
        while i < digest_words {
            let word = T::regs().hr(i).read();
            digest[(i * 4)..((i * 4) + 4)].copy_from_slice(word.to_be_bytes().as_slice());
            i += 1;
        }
        &digest[0..digest_words * 4]
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
        let dma_request = self.dma.request();
        let dst_ptr = T::regs().din().as_ptr();
        let mut num_words = input.len() / 4;
        if input.len() % 4 > 0 {
            num_words += 1;
        }
        let src_ptr = ptr::slice_from_raw_parts(input.as_ptr().cast(), num_words);
        let dma_transfer =
            unsafe { Transfer::new_write_raw(&mut self.dma, dma_request, src_ptr, dst_ptr, Default::default()) };
        T::regs().cr().modify(|w| w.set_dmae(true));

        // Wait for the transfer to complete.
        dma_transfer.await;
    }

    /// Save the peripheral state to a context.
    async fn store_context(&mut self, ctx: &mut Context) {
        // Wait for interrupt.
        poll_fn(|cx| {
            // Check if already done.
            let bits = T::regs().sr().read();
            if bits.dinis() {
                return Poll::Ready(());
            }
            // Register waker, then enable interrupts.
            HASH_WAKER.register(cx.waker());
            T::regs().imr().modify(|reg| reg.set_dinie(true));
            // Check for completion.
            let bits = T::regs().sr().read();
            if bits.dinis() {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await;

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

pub(crate) mod sealed {
    use super::*;

    pub trait Instance {
        fn regs() -> pac::hash::Hash;
    }
}

/// HASH instance trait.
pub trait Instance: sealed::Instance + Peripheral<P = Self> + crate::rcc::RccPeripheral + 'static + Send {
    /// Interrupt for this HASH instance.
    type Interrupt: interrupt::typelevel::Interrupt;
}

foreach_interrupt!(
    ($inst:ident, hash, HASH, GLOBAL, $irq:ident) => {
        impl Instance for peripherals::$inst {
            type Interrupt = crate::interrupt::typelevel::$irq;
        }

        impl sealed::Instance for peripherals::$inst {
            fn regs() -> crate::pac::hash::Hash {
                crate::pac::$inst
            }
        }
    };
);

dma_trait!(Dma, Instance);
