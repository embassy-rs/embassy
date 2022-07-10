//! PDM mirophone interface

use core::marker::PhantomData;
use core::sync::atomic::{compiler_fence, Ordering};
use core::task::Poll;

use embassy_hal_common::drop::OnDrop;
use embassy_hal_common::{into_ref, PeripheralRef};
use embassy_sync::waitqueue::AtomicWaker;
use futures::future::poll_fn;

use crate::chip::EASY_DMA_SIZE;
use crate::gpio::sealed::Pin;
use crate::gpio::{AnyPin, Pin as GpioPin};
use crate::interrupt::{self, InterruptExt};
use crate::peripherals::PDM;
use crate::{pac, Peripheral};

/// PDM microphone interface
pub struct Pdm<'d> {
    irq: PeripheralRef<'d, interrupt::PDM>,
    phantom: PhantomData<&'d PDM>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    BufferTooLong,
    BufferZeroLength,
    NotRunning,
}

static WAKER: AtomicWaker = AtomicWaker::new();
static DUMMY_BUFFER: [i16; 1] = [0; 1];

impl<'d> Pdm<'d> {
    /// Create PDM driver
    pub fn new(
        pdm: impl Peripheral<P = PDM> + 'd,
        irq: impl Peripheral<P = interrupt::PDM> + 'd,
        clk: impl Peripheral<P = impl GpioPin> + 'd,
        din: impl Peripheral<P = impl GpioPin> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(clk, din);
        Self::new_inner(pdm, irq, clk.map_into(), din.map_into(), config)
    }

    fn new_inner(
        _pdm: impl Peripheral<P = PDM> + 'd,
        irq: impl Peripheral<P = interrupt::PDM> + 'd,
        clk: PeripheralRef<'d, AnyPin>,
        din: PeripheralRef<'d, AnyPin>,
        config: Config,
    ) -> Self {
        into_ref!(irq);

        let r = Self::regs();

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
        irq.disable();
        irq.set_handler(|_| {
            let r = Self::regs();
            r.intenclr.write(|w| w.end().clear());
            WAKER.wake();
        });
        irq.enable();

        r.enable.write(|w| w.enable().set_bit());

        Self {
            phantom: PhantomData,
            irq,
        }
    }

    /// Start sampling microphon data into a dummy buffer
    /// Usefull to start the microphon and keep it active between recording samples
    pub async fn start(&mut self) {
        let r = Self::regs();

        // start dummy sampling because microphon needs some setup time
        r.sample
            .ptr
            .write(|w| unsafe { w.sampleptr().bits(DUMMY_BUFFER.as_ptr() as u32) });
        r.sample
            .maxcnt
            .write(|w| unsafe { w.buffsize().bits(DUMMY_BUFFER.len() as _) });

        r.tasks_start.write(|w| w.tasks_start().set_bit());
    }

    /// Stop sampling microphon data inta a dummy buffer
    pub async fn stop(&mut self) {
        let r = Self::regs();
        r.tasks_stop.write(|w| w.tasks_stop().set_bit());
        r.events_started.reset();
    }

    pub async fn sample(&mut self, buffer: &mut [i16]) -> Result<(), Error> {
        if buffer.len() == 0 {
            return Err(Error::BufferZeroLength);
        }
        if buffer.len() > EASY_DMA_SIZE {
            return Err(Error::BufferTooLong);
        }

        let r = Self::regs();

        if r.events_started.read().events_started().bit_is_clear() {
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
        let r = Self::regs();

        r.events_end.reset();
        r.intenset.write(|w| w.end().set());

        compiler_fence(Ordering::SeqCst);

        poll_fn(|cx| {
            WAKER.register(cx.waker());
            if r.events_end.read().events_end().bit_is_set() {
                return Poll::Ready(());
            }
            Poll::Pending
        })
        .await;

        compiler_fence(Ordering::SeqCst);
    }

    fn regs() -> &'static pac::pdm::RegisterBlock {
        unsafe { &*pac::PDM::ptr() }
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

#[derive(PartialEq)]
pub enum OperationMode {
    Mono,
    Stereo,
}
#[derive(PartialEq)]
pub enum Edge {
    LeftRising,
    LeftFalling,
}

impl<'d> Drop for Pdm<'d> {
    fn drop(&mut self) {
        let r = Self::regs();

        r.tasks_stop.write(|w| w.tasks_stop().set_bit());

        self.irq.disable();

        r.enable.write(|w| w.enable().disabled());

        r.psel.din.reset();
        r.psel.clk.reset();
    }
}
