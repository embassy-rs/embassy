//! Driver shell for the `aes_v3b` hardware revision (STM32H5, WBA).
//!
//! The cipher types and the GCM/CCM state machine live in [`super::common`];
//! this module provides the [`Aes`] driver, its interrupt handler, instance
//! wiring, and constructors. `aes_v3b` parts have a dedicated AES interrupt and
//! DMA channels, so both a blocking constructor and a DMA-capable async
//! constructor are offered.
//!
//! # Key sizes
//!
//! - 128-bit (16 bytes) and 256-bit (32 bytes); 192-bit keys are not supported.
//!
//! # Example (blocking GCM)
//!
//! ```no_run
//! use embassy_stm32::aes::{Aes, AesGcm, Direction};
//! use embassy_stm32::{bind_interrupts, peripherals};
//!
//! bind_interrupts!(struct Irqs {
//!     AES => embassy_stm32::aes::InterruptHandler<peripherals::AES>;
//! });
//!
//! let key = [0u8; 16];
//! let iv = [0u8; 12];
//! let cipher = AesGcm::new(&key, &iv);
//!
//! let mut aes = Aes::new_blocking(p.AES, Irqs);
//! let mut ctx = aes.start(&cipher, Direction::Encrypt);
//! aes.aad_blocking(&mut ctx, b"metadata", true).unwrap();
//! let mut ciphertext = [0u8; 16];
//! aes.payload_blocking(&mut ctx, &[0u8; 16], &mut ciphertext, true).unwrap();
//! let _tag = aes.finish_blocking(ctx).unwrap();
//! ```

use core::marker::PhantomData;

use embassy_hal_internal::{Peri, PeripheralType};
use embassy_sync::waitqueue::AtomicWaker;

use super::common::{
    self, Cipher, CipherAuthenticated, CipherSized, Context, Direction, Error, IVSized, SealedInstance,
};
use crate::dma::ChannelAndRequest;
use crate::interrupt::typelevel::Interrupt;
use crate::mode::{Async, Blocking, Mode};
use crate::{interrupt, peripherals, rcc};

static AES_WAKER: AtomicWaker = AtomicWaker::new();

/// AES interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _marker: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let sr = T::regs().sr().read();

        // Wake on completion flag.
        if sr.ccf() {
            T::regs().icr().write(|w| w.0 = 0xFFFF_FFFF);
            AES_WAKER.wake();
        }

        // Clear error flags if any.
        if sr.rderr() || sr.wrerr() {
            T::regs().icr().write(|w| w.0 = 0xFFFF_FFFF);
        }
    }
}

/// AES driver.
pub struct Aes<'d, T: Instance, M: Mode> {
    _peripheral: Peri<'d, T>,
    _marker: PhantomData<M>,
    #[allow(dead_code)] // Reserved for future async/DMA implementation
    dma_in: Option<ChannelAndRequest<'d>>,
    #[allow(dead_code)] // Reserved for future async/DMA implementation
    dma_out: Option<ChannelAndRequest<'d>>,
}

impl<'d, T: Instance> Aes<'d, T, Blocking> {
    /// Instantiates, resets, and enables the AES peripheral.
    pub fn new_blocking(
        peripheral: Peri<'d, T>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
    ) -> Self {
        rcc::enable_and_reset::<T>();
        let instance = Self {
            _peripheral: peripheral,
            _marker: PhantomData,
            dma_in: None,
            dma_out: None,
        };

        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        instance
    }
}

impl<'d, T: Instance> Aes<'d, T, Async> {
    /// Instantiates, resets, and enables the AES peripheral with DMA support.
    pub fn new<D1: DmaIn<T>, D2: DmaOut<T>>(
        peripheral: Peri<'d, T>,
        dma_in: Peri<'d, D1>,
        dma_out: Peri<'d, D2>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>>
        + interrupt::typelevel::Binding<D1::Interrupt, crate::dma::InterruptHandler<D1>>
        + interrupt::typelevel::Binding<D2::Interrupt, crate::dma::InterruptHandler<D2>>
        + 'd,
    ) -> Self {
        rcc::enable_and_reset::<T>();

        let instance = Self {
            _peripheral: peripheral,
            _marker: PhantomData,
            dma_in: new_dma!(dma_in, _irq),
            dma_out: new_dma!(dma_out, _irq),
        };

        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        instance
    }
}

impl<'d, T: Instance, M: Mode> Aes<'d, T, M> {
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

/// AES instance trait.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + crate::rcc::RccPeripheral + 'static + Send {
    /// Interrupt for this AES instance.
    type Interrupt: interrupt::typelevel::Interrupt;
}

foreach_interrupt!(
    ($inst:ident, aes, AES, GLOBAL, $irq:ident) => {
        impl Instance for peripherals::$inst {
            type Interrupt = crate::interrupt::typelevel::$irq;
        }
    };
);

dma_trait!(DmaIn, Instance);
dma_trait!(DmaOut, Instance);
