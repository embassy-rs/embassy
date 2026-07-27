//! Driver shell for the `aes_v2` hardware revision (STM32G0, G4, L5, U0, WL).
//!
//! The cipher types and the GCM/CCM state machine live in [`super::common`];
//! this module provides the [`Aes`] driver, instance wiring, and constructors.
//!
//! A blocking API is always available. An interrupt-driven async API is also
//! offered on parts whose AES engine has a dedicated interrupt vector (L5, WL),
//! gated on [`InterruptableInstance`]; on parts where the engine shares its
//! vector or has none (U0, G0, G4) only the blocking API compiles in.

use core::future::poll_fn;
use core::marker::PhantomData;
use core::task::Poll;

use embassy_hal_internal::{Peri, PeripheralType};
use embassy_sync::waitqueue::AtomicWaker;

use super::common::{
    self, Cipher, CipherAuthenticated, CipherSized, Context, Direction, Error, IVSized, SealedInstance,
};
use crate::interrupt::typelevel::Interrupt;
use crate::{interrupt, pac, peripherals, rcc};

// A single AES instance exists per chip, so one waker suffices.
static AES_WAKER: AtomicWaker = AtomicWaker::new();

/// Await the computation-complete flag (CCF), driven by the AES interrupt.
///
/// The interrupt is enabled through `CCFIE` only after the waker is registered.
/// The ISR masks `CCFIE` when it fires (it does not clear `CCF`), so the caller
/// still reads `DOUTR` and clears `CCF` itself. `CCFIE` is also masked on the
/// already-complete fast path to avoid a spurious interrupt before the flag is
/// cleared.
async fn wait_ccf(p: pac::aes::Aes) {
    poll_fn(|cx| {
        if p.sr().read().ccf() {
            p.cr().modify(|w| w.set_ccfie(false));
            return Poll::Ready(());
        }
        AES_WAKER.register(cx.waker());
        p.cr().modify(|w| w.set_ccfie(true));
        // Re-check after arming to close the register/enable race.
        if p.sr().read().ccf() {
            p.cr().modify(|w| w.set_ccfie(false));
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    })
    .await
}

/// AES driver.
pub struct Aes<'d, T: Instance> {
    _peripheral: Peri<'d, T>,
}

impl<'d, T: Instance> Aes<'d, T> {
    /// Instantiates, resets, and enables the AES peripheral.
    ///
    /// Unlike the `aes_v3b` driver, no interrupt binding is required: on
    /// `aes_v2` parts the engine has no dedicated interrupt, so the driver
    /// polls the completion flag.
    pub fn new_blocking(peripheral: Peri<'d, T>) -> Self {
        rcc::enable_and_reset::<T>();
        Self {
            _peripheral: peripheral,
        }
    }

    /// Starts a new cipher operation and returns the context.
    pub fn start<'c, C>(&mut self, cipher: &'c C, dir: Direction) -> Context<'c, C>
    where
        C: Cipher<'c> + CipherSized + IVSized,
    {
        common::op_start(T::regs(), cipher, dir)
    }

    /// Process authenticated additional data (AAD) for GCM/CCM modes.
    /// Must be called after `start` and before `payload_blocking`.
    /// Set `last` to true for the final AAD block.
    pub fn aad_blocking<'c, C>(&mut self, ctx: &mut Context<'c, C>, aad: &[u8], last: bool) -> Result<(), Error>
    where
        C: Cipher<'c> + CipherAuthenticated<16>,
    {
        common::op_aad(T::regs(), ctx, aad, last)
    }

    /// Process payload data in blocking mode.
    ///
    /// Set `last` to true for the final block. Intermediate chunks (`last=false`)
    /// must be block-aligned (16 bytes); only the final chunk can be partial.
    pub fn payload_blocking<'c, C>(
        &mut self,
        ctx: &mut Context<'c, C>,
        input: &[u8],
        output: &mut [u8],
        last: bool,
    ) -> Result<(), Error>
    where
        C: Cipher<'c>,
    {
        common::op_payload(T::regs(), ctx, input, output, last)
    }

    /// Finishes the cipher operation and returns the authentication tag (for GCM/CCM).
    pub fn finish_blocking<'c, C>(&mut self, ctx: Context<'c, C>) -> Result<Option<[u8; 16]>, Error>
    where
        C: Cipher<'c>,
    {
        common::op_finish(T::regs(), ctx)
    }
}

/// Interrupt handler for the async AES API.
///
/// Bind this to the instance's interrupt with `bind_interrupts!` before calling
/// [`Aes::new`].
pub struct InterruptHandler<T: InterruptableInstance> {
    _marker: PhantomData<T>,
}

impl<T: InterruptableInstance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let p = T::regs();
        if p.sr().read().ccf() {
            // Mask our own source without clearing CCF: the awaiting task reads
            // DOUTR and clears CCF itself. This prevents an interrupt storm.
            p.cr().modify(|w| w.set_ccfie(false));
            AES_WAKER.wake();
        }
    }
}

impl<'d, T: InterruptableInstance> Aes<'d, T> {
    /// Instantiates, resets, and enables the AES peripheral for interrupt-driven
    /// async operation.
    ///
    /// This is only available on parts whose AES engine has a dedicated
    /// interrupt vector (e.g. STM32L5 and WL). On parts where the engine shares
    /// its vector with another peripheral or has none (U0, G0, G4), use
    /// [`new_blocking`](Aes::new_blocking) instead.
    pub fn new(
        peripheral: Peri<'d, T>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
    ) -> Self {
        rcc::enable_and_reset::<T>();
        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };
        Self {
            _peripheral: peripheral,
        }
    }

    /// Starts a new cipher operation and returns the context.
    ///
    /// The GCM/CCM init phase (hash-key computation) completes asynchronously.
    /// Key derivation for ECB/CBC decryption is a one-shot step and is polled.
    pub async fn start_async<'c, C>(&mut self, cipher: &'c C, dir: Direction) -> Context<'c, C>
    where
        C: Cipher<'c> + CipherSized + IVSized,
    {
        let p = T::regs();
        let is_gcm_ccm = common::setup(p, cipher, dir);

        p.cr().modify(|w| w.set_en(true));
        if is_gcm_ccm {
            wait_ccf(p).await;
            common::clear_ccf(p);
        }

        common::make_context(p, cipher, dir, is_gcm_ccm)
    }

    /// Processes authenticated additional data (AAD) for GCM/CCM modes.
    /// Must be called after [`start_async`](Aes::start_async) and before
    /// [`payload_async`](Aes::payload_async). Set `last` to true for the final block.
    pub async fn aad_async<'c, C>(&mut self, ctx: &mut Context<'c, C>, aad: &[u8], last: bool) -> Result<(), Error>
    where
        C: Cipher<'c> + CipherAuthenticated<16>,
    {
        let p = T::regs();

        if ctx.header_processed && last {
            return Ok(());
        }

        p.cr().modify(|w| w.set_gcmph(pac::aes::vals::Gcmph::from_bits(1)));
        p.cr().modify(|w| w.set_en(true));

        let mut aad_remaining = aad.len();
        let mut aad_index = 0;

        if ctx.aad_buffer_len > 0 {
            let space_available = 16 - ctx.aad_buffer_len;
            let to_copy = core::cmp::min(space_available, aad_remaining);
            ctx.aad_buffer[ctx.aad_buffer_len..ctx.aad_buffer_len + to_copy].copy_from_slice(&aad[..to_copy]);
            ctx.aad_buffer_len += to_copy;
            aad_index += to_copy;
            aad_remaining -= to_copy;

            if ctx.aad_buffer_len == 16 {
                common::write_block(p, &ctx.aad_buffer)?;
                wait_ccf(p).await;
                common::clear_ccf(p);
                ctx.header_len += 16;
                ctx.aad_buffer_len = 0;
            }
        }

        while aad_remaining >= 16 {
            common::write_block(p, &aad[aad_index..aad_index + 16])?;
            wait_ccf(p).await;
            common::clear_ccf(p);
            ctx.header_len += 16;
            aad_index += 16;
            aad_remaining -= 16;
        }

        if aad_remaining > 0 {
            ctx.aad_buffer[..aad_remaining].copy_from_slice(&aad[aad_index..aad_index + aad_remaining]);
            ctx.aad_buffer_len = aad_remaining;
        }

        if last {
            if ctx.aad_buffer_len > 0 {
                for i in ctx.aad_buffer_len..16 {
                    ctx.aad_buffer[i] = 0;
                }
                common::write_block(p, &ctx.aad_buffer)?;
                wait_ccf(p).await;
                common::clear_ccf(p);
                ctx.header_len += ctx.aad_buffer_len as u64;
                ctx.aad_buffer_len = 0;
            }
            ctx.header_processed = true;
        }

        Ok(())
    }

    /// Processes payload data.
    ///
    /// Set `last` to true for the final block. Intermediate chunks (`last=false`)
    /// must be block-aligned (16 bytes); only the final chunk can be partial.
    pub async fn payload_async<'c, C>(
        &mut self,
        ctx: &mut Context<'c, C>,
        input: &[u8],
        output: &mut [u8],
        last: bool,
    ) -> Result<(), Error>
    where
        C: Cipher<'c>,
    {
        let p = T::regs();

        if output.len() < input.len() {
            return Err(Error::ConfigError);
        }

        common::begin_payload(p, ctx);

        let block_size = C::BLOCK_SIZE;
        let mut processed = 0;

        if !last && input.len() % block_size != 0 {
            return Err(Error::ConfigError);
        }

        let complete_blocks = input.len() / block_size;

        for _ in 0..complete_blocks {
            common::write_block(p, &input[processed..processed + block_size])?;
            self.read_block_async(&mut output[processed..processed + block_size])
                .await?;
            processed += block_size;
            ctx.payload_len += block_size as u64;
        }

        if last && processed < input.len() {
            if C::REQUIRES_PADDING {
                return Err(Error::ConfigError);
            }

            let remaining = input.len() - processed;
            let mut partial_block = [0u8; 16];
            partial_block[..remaining].copy_from_slice(&input[processed..]);

            common::set_final_npblb(p, ctx, remaining);

            common::write_block(p, &partial_block)?;
            self.read_block_async(&mut partial_block).await?;

            output[processed..processed + remaining].copy_from_slice(&partial_block[..remaining]);
            ctx.payload_len += remaining as u64;
        }

        if last {
            ctx.last_block_processed = true;
        }

        Ok(())
    }

    /// Finishes the cipher operation and returns the authentication tag (for GCM/CCM).
    pub async fn finish_async<'c, C>(&mut self, ctx: Context<'c, C>) -> Result<Option<[u8; 16]>, Error>
    where
        C: Cipher<'c>,
    {
        let p = T::regs();

        if ctx.is_gcm_ccm {
            common::begin_final(p, &ctx);

            wait_ccf(p).await;

            let mut tag = [0u8; 16];
            common::read_out_block(p, &mut tag);

            common::clear_ccf(p);
            p.cr().modify(|w| w.set_en(false));

            Ok(Some(tag))
        } else {
            p.cr().modify(|w| w.set_en(false));
            Ok(None)
        }
    }

    /// Reads a 16-byte block from the AES peripheral, awaiting completion.
    async fn read_block_async(&mut self, block: &mut [u8]) -> Result<(), Error> {
        let p = T::regs();

        wait_ccf(p).await;

        if p.sr().read().rderr() {
            common::clear_flags(p);
            return Err(Error::ReadError);
        }

        common::read_out_block(p, block);
        common::clear_ccf(p);

        Ok(())
    }
}

/// AES instance trait.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + crate::rcc::RccPeripheral + 'static + Send {}

/// Marks an AES instance that has its own dedicated interrupt vector.
///
/// The async API ([`Aes::new`] and the async cipher flow) is only available for
/// instances implementing this trait. It is implemented automatically for parts
/// whose metapac binds a dedicated `GLOBAL` AES interrupt (e.g. STM32L5, WL),
/// and is absent where the engine shares its vector with another peripheral
/// (U0, G0, G4) — on those, use the blocking API.
pub trait InterruptableInstance: Instance {
    /// The dedicated interrupt for this AES instance.
    type Interrupt: interrupt::typelevel::Interrupt;
}

foreach_peripheral!(
    (aes, $inst:ident) => {
        impl Instance for peripherals::$inst {}
    };
);

// Self-selecting: this only expands on parts whose metapac binds a dedicated
// GLOBAL AES interrupt, so the async API compiles in exactly where it is usable.
foreach_interrupt!(
    ($inst:ident, aes, AES, GLOBAL, $irq:ident) => {
        impl InterruptableInstance for peripherals::$inst {
            type Interrupt = crate::interrupt::typelevel::$irq;
        }
    };
);
