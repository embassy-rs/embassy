//! Pulse Density Modulation (PDM) mirophone driver.

#![macro_use]

use core::marker::PhantomData;
use core::sync::atomic::{compiler_fence, Ordering};
use core::task::Poll;

use embassy_hal_common::drop::OnDrop;
use embassy_hal_common::{into_ref, PeripheralRef};
use futures::future::poll_fn;

use crate::chip::EASY_DMA_SIZE;
use crate::gpio::sealed::Pin;
use crate::gpio::{AnyPin, Pin as GpioPin};
use crate::interrupt::typelevel::Interrupt;
use crate::{interrupt, Peripheral};

/// Interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        T::regs().intenclr.write(|w| w.end().clear());
        T::state().waker.wake();
    }
}

/// PDM microphone interface
pub struct Pdm<'d, T: Instance> {
    _peri: PeripheralRef<'d, T>,
}

/// PDM error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    /// Buffer is too long.
    BufferTooLong,
    /// Buffer is empty
    BufferZeroLength,
    /// PDM is not running
    NotRunning,
}

static DUMMY_BUFFER: [i16; 1] = [0; 1];

impl<'d, T: Instance> Pdm<'d, T> {
    /// Create PDM driver
    pub fn new(
        pdm: impl Peripheral<P = T> + 'd,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        clk: impl Peripheral<P = impl GpioPin> + 'd,
        din: impl Peripheral<P = impl GpioPin> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(pdm, clk, din);
        Self::new_inner(pdm, clk.map_into(), din.map_into(), config)
    }

    fn new_inner(
        pdm: PeripheralRef<'d, T>,
        clk: PeripheralRef<'d, AnyPin>,
        din: PeripheralRef<'d, AnyPin>,
        config: Config,
    ) -> Self {
        into_ref!(pdm);

        let r = T::regs();

        // setup gpio pins
        din.conf().write(|w| w.input().set_bit());
        r.psel.din.write(|w| unsafe { w.bits(din.psel_bits()) });
        clk.set_low();
        clk.conf().write(|w| w.dir().output());
        r.psel.clk.write(|w| unsafe { w.bits(clk.psel_bits()) });

        // configure
        // use default for
        // - gain right
        // - gain left
        // - clk
        // - ratio
        r.mode.write(|w| {
            w.edge().bit(config.edge == Edge::LeftRising);
            w.operation().bit(config.operation_mode == OperationMode::Mono);
            w
        });
        r.gainl.write(|w| w.gainl().default_gain());
        r.gainr.write(|w| w.gainr().default_gain());

        // IRQ
        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        r.enable.write(|w| w.enable().set_bit());

        Self { _peri: pdm }
    }

    /// Start sampling microphon data into a dummy buffer
    /// Usefull to start the microphon and keep it active between recording samples
    pub async fn start(&mut self) {
        let r = T::regs();

        // start dummy sampling because microphon needs some setup time
        r.sample
            .ptr
            .write(|w| unsafe { w.sampleptr().bits(DUMMY_BUFFER.as_ptr() as u32) });
        r.sample
            .maxcnt
            .write(|w| unsafe { w.buffsize().bits(DUMMY_BUFFER.len() as _) });

        r.tasks_start.write(|w| unsafe { w.bits(1) });
    }

    /// Stop sampling microphon data inta a dummy buffer
    pub async fn stop(&mut self) {
        let r = T::regs();
        r.tasks_stop.write(|w| unsafe { w.bits(1) });
        r.events_started.reset();
    }

    /// Sample data into the given buffer.
    pub async fn sample(&mut self, buffer: &mut [i16]) -> Result<(), Error> {
        if buffer.len() == 0 {
            return Err(Error::BufferZeroLength);
        }
        if buffer.len() > EASY_DMA_SIZE {
            return Err(Error::BufferTooLong);
        }

        let r = T::regs();

        if r.events_started.read().bits() == 0 {
            return Err(Error::NotRunning);
        }

        let drop = OnDrop::new(move || {
            r.intenclr.write(|w| w.end().clear());
            r.events_stopped.reset();

            // reset to dummy buffer
            r.sample
                .ptr
                .write(|w| unsafe { w.sampleptr().bits(DUMMY_BUFFER.as_ptr() as u32) });
            r.sample
                .maxcnt
                .write(|w| unsafe { w.buffsize().bits(DUMMY_BUFFER.len() as _) });

            while r.events_stopped.read().bits() == 0 {}
        });

        // setup user buffer
        let ptr = buffer.as_ptr();
        let len = buffer.len();
        r.sample.ptr.write(|w| unsafe { w.sampleptr().bits(ptr as u32) });
        r.sample.maxcnt.write(|w| unsafe { w.buffsize().bits(len as _) });

        // wait till the current sample is finished and the user buffer sample is started
        Self::wait_for_sample().await;

        // reset the buffer back to the dummy buffer
        r.sample
            .ptr
            .write(|w| unsafe { w.sampleptr().bits(DUMMY_BUFFER.as_ptr() as u32) });
        r.sample
            .maxcnt
            .write(|w| unsafe { w.buffsize().bits(DUMMY_BUFFER.len() as _) });

        // wait till the user buffer is sampled
        Self::wait_for_sample().await;

        drop.defuse();

        Ok(())
    }

    async fn wait_for_sample() {
        let r = T::regs();

        r.events_end.reset();
        r.intenset.write(|w| w.end().set());

        compiler_fence(Ordering::SeqCst);

        poll_fn(|cx| {
            T::state().waker.register(cx.waker());
            if r.events_end.read().bits() != 0 {
                return Poll::Ready(());
            }
            Poll::Pending
        })
        .await;

        compiler_fence(Ordering::SeqCst);
    }
}

/// PDM microphone driver Config
pub struct Config {
    /// Use stero or mono operation
    pub operation_mode: OperationMode,
    /// On which edge the left channel should be samples
    pub edge: Edge,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            operation_mode: OperationMode::Mono,
            edge: Edge::LeftFalling,
        }
    }
}

/// PDM operation mode.
#[derive(PartialEq)]
pub enum OperationMode {
    /// Mono (1 channel)
    Mono,
    /// Stereo (2 channels)
    Stereo,
}

/// PDM edge polarity
#[derive(PartialEq)]
pub enum Edge {
    /// Left edge is rising
    LeftRising,
    /// Left edge is falling
    LeftFalling,
}

impl<'d, T: Instance> Drop for Pdm<'d, T> {
    fn drop(&mut self) {
        let r = T::regs();

        r.tasks_stop.write(|w| unsafe { w.bits(1) });

        r.enable.write(|w| w.enable().disabled());

        r.psel.din.reset();
        r.psel.clk.reset();
    }
}

pub(crate) mod sealed {
    use embassy_sync::waitqueue::AtomicWaker;

    /// Peripheral static state
    pub struct State {
        pub waker: AtomicWaker,
    }

    impl State {
        pub const fn new() -> Self {
            Self {
                waker: AtomicWaker::new(),
            }
        }
    }

    pub trait Instance {
        fn regs() -> &'static crate::pac::pdm::RegisterBlock;
        fn state() -> &'static State;
    }
}

/// PDM peripheral instance.
pub trait Instance: Peripheral<P = Self> + sealed::Instance + 'static + Send {
    /// Interrupt for this peripheral.
    type Interrupt: interrupt::typelevel::Interrupt;
}

macro_rules! impl_pdm {
    ($type:ident, $pac_type:ident, $irq:ident) => {
        impl crate::pdm::sealed::Instance for peripherals::$type {
            fn regs() -> &'static crate::pac::pdm::RegisterBlock {
                unsafe { &*pac::$pac_type::ptr() }
            }
            fn state() -> &'static crate::pdm::sealed::State {
                static STATE: crate::pdm::sealed::State = crate::pdm::sealed::State::new();
                &STATE
            }
        }
        impl crate::pdm::Instance for peripherals::$type {
            type Interrupt = crate::interrupt::typelevel::$irq;
        }
    };
}
