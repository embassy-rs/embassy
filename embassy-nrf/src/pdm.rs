//! Pulse Density Modulation (PDM) microphone driver

#![macro_use]

use core::future::poll_fn;
use core::marker::PhantomData;
use core::sync::atomic::{compiler_fence, Ordering};
use core::task::Poll;

use embassy_hal_internal::drop::OnDrop;
use embassy_hal_internal::{Peri, PeripheralType};
use embassy_sync::waitqueue::AtomicWaker;
use fixed::types::I7F1;

use crate::chip::EASY_DMA_SIZE;
use crate::gpio::{AnyPin, Pin as GpioPin, SealedPin, DISCONNECTED};
use crate::interrupt::typelevel::Interrupt;
use crate::pac::gpio::vals as gpiovals;
use crate::pac::pdm::vals;
pub use crate::pac::pdm::vals::Freq as Frequency;
#[cfg(any(
    feature = "nrf52840",
    feature = "nrf52833",
    feature = "_nrf5340-app",
    feature = "_nrf91",
))]
pub use crate::pac::pdm::vals::Ratio;
use crate::{interrupt, pac};

/// Interrupt handler
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let r = T::regs();

        if r.events_end().read() != 0 {
            r.intenclr().write(|w| w.set_end(true));
        }

        if r.events_started().read() != 0 {
            r.intenclr().write(|w| w.set_started(true));
        }

        if r.events_stopped().read() != 0 {
            r.intenclr().write(|w| w.set_stopped(true));
        }

        T::state().waker.wake();
    }
}

/// PDM microphone interface
pub struct Pdm<'d, T: Instance> {
    _peri: Peri<'d, T>,
}

/// PDM error
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    /// Buffer is too long
    BufferTooLong,
    /// Buffer is empty
    BufferZeroLength,
    /// PDM is not running
    NotRunning,
    /// PDM is already running
    AlreadyRunning,
}

static DUMMY_BUFFER: [i16; 1] = [0; 1];

/// The state of a continuously running sampler. While it reflects
/// the progress of a sampler, it also signals what should be done
/// next. For example, if the sampler has stopped then the PDM implementation
/// can then tear down its infrastructure
#[derive(PartialEq)]
pub enum SamplerState {
    /// The sampler processed the samples and is ready for more
    Sampled,
    /// The sampler is done processing samples
    Stopped,
}

impl<'d, T: Instance> Pdm<'d, T> {
    /// Create PDM driver
    pub fn new(
        pdm: Peri<'d, T>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        clk: Peri<'d, impl GpioPin>,
        din: Peri<'d, impl GpioPin>,
        config: Config,
    ) -> Self {
        Self::new_inner(pdm, clk.into(), din.into(), config)
    }

    fn new_inner(pdm: Peri<'d, T>, clk: Peri<'d, AnyPin>, din: Peri<'d, AnyPin>, config: Config) -> Self {
        let r = T::regs();

        // setup gpio pins
        din.conf().write(|w| w.set_input(gpiovals::Input::CONNECT));
        r.psel().din().write_value(din.psel_bits());
        clk.set_low();
        clk.conf().write(|w| w.set_dir(gpiovals::Dir::OUTPUT));
        r.psel().clk().write_value(clk.psel_bits());

        // configure
        r.pdmclkctrl().write(|w| w.set_freq(config.frequency));
        #[cfg(any(
            feature = "nrf52840",
            feature = "nrf52833",
            feature = "_nrf5340-app",
            feature = "_nrf91",
        ))]
        r.ratio().write(|w| w.set_ratio(config.ratio));
        r.mode().write(|w| {
            w.set_operation(config.operation_mode.into());
            w.set_edge(config.edge.into());
        });

        Self::_set_gain(r, config.gain_left, config.gain_right);

        // Disable all events interrupts
        r.intenclr().write(|w| w.0 = 0x003F_FFFF);

        // IRQ
        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        r.enable().write(|w| w.set_enable(true));

        Self { _peri: pdm }
    }

    fn _set_gain(r: pac::pdm::Pdm, gain_left: I7F1, gain_right: I7F1) {
        let gain_to_bits = |gain: I7F1| -> vals::Gain {
            let gain: i8 = gain.saturating_add(I7F1::from_bits(0x28)).to_bits().clamp(0, 0x50);
            vals::Gain::from_bits(gain as u8)
        };
        r.gainl().write(|w| w.set_gainl(gain_to_bits(gain_left)));
        r.gainr().write(|w| w.set_gainr(gain_to_bits(gain_right)));
    }

    /// Adjust the gain of the PDM microphone on the fly
    pub fn set_gain(&mut self, gain_left: I7F1, gain_right: I7F1) {
        Self::_set_gain(T::regs(), gain_left, gain_right)
    }

    /// Start sampling microphone data into a dummy buffer.
    /// Useful to start the microphone and keep it active between recording samples.
    pub async fn start(&mut self) {
        let r = T::regs();

        // start dummy sampling because microphone needs some setup time
        r.sample().ptr().write_value(DUMMY_BUFFER.as_ptr() as u32);
        r.sample().maxcnt().write(|w| w.set_buffsize(DUMMY_BUFFER.len() as _));

        r.tasks_start().write_value(1);
    }

    /// Stop sampling microphone data inta a dummy buffer
    pub async fn stop(&mut self) {
        let r = T::regs();
        r.tasks_stop().write_value(1);
        r.events_started().write_value(0);
    }

    /// Sample data into the given buffer
    pub async fn sample(&mut self, buffer: &mut [i16]) -> Result<(), Error> {
        if buffer.is_empty() {
            return Err(Error::BufferZeroLength);
        }
        if buffer.len() > EASY_DMA_SIZE {
            return Err(Error::BufferTooLong);
        }

        let r = T::regs();

        if r.events_started().read() == 0 {
            return Err(Error::NotRunning);
        }

        let drop = OnDrop::new(move || {
            r.intenclr().write(|w| w.set_end(true));
            r.events_stopped().write_value(0);

            // reset to dummy buffer
            r.sample().ptr().write_value(DUMMY_BUFFER.as_ptr() as u32);
            r.sample().maxcnt().write(|w| w.set_buffsize(DUMMY_BUFFER.len() as _));

            while r.events_stopped().read() == 0 {}
        });

        // setup user buffer
        let ptr = buffer.as_ptr();
        let len = buffer.len();
        r.sample().ptr().write_value(ptr as u32);
        r.sample().maxcnt().write(|w| w.set_buffsize(len as _));

        // wait till the current sample is finished and the user buffer sample is started
        Self::wait_for_sample().await;

        // reset the buffer back to the dummy buffer
        r.sample().ptr().write_value(DUMMY_BUFFER.as_ptr() as u32);
        r.sample().maxcnt().write(|w| w.set_buffsize(DUMMY_BUFFER.len() as _));

        // wait till the user buffer is sampled
        Self::wait_for_sample().await;

        drop.defuse();

        Ok(())
    }

    async fn wait_for_sample() {
        let r = T::regs();

        r.events_end().write_value(0);
        r.intenset().write(|w| w.set_end(true));

        compiler_fence(Ordering::SeqCst);

        poll_fn(|cx| {
            T::state().waker.register(cx.waker());
            if r.events_end().read() != 0 {
                return Poll::Ready(());
            }
            Poll::Pending
        })
        .await;

        compiler_fence(Ordering::SeqCst);
    }

    /// Continuous sampling with double buffers.
    ///
    /// A sampler closure is provided that receives the buffer of samples, noting
    /// that the size of this buffer can be less than the original buffer's size.
    /// A command is return from the closure that indicates whether the sampling
    /// should continue or stop.
    ///
    /// NOTE: The time spent within the callback supplied should not exceed the time
    /// taken to acquire the samples into a single buffer. You should measure the
    /// time taken by the callback and set the sample buffer size accordingly.
    /// Exceeding this time can lead to samples becoming dropped.
    pub async fn run_task_sampler<S, const N: usize>(
        &mut self,
        bufs: &mut [[i16; N]; 2],
        mut sampler: S,
    ) -> Result<(), Error>
    where
        S: FnMut(&[i16; N]) -> SamplerState,
    {
        let r = T::regs();

        if r.events_started().read() != 0 {
            return Err(Error::AlreadyRunning);
        }

        r.sample().ptr().write_value(bufs[0].as_mut_ptr() as u32);
        r.sample().maxcnt().write(|w| w.set_buffsize(N as _));

        // Reset and enable the events
        r.events_end().write_value(0);
        r.events_started().write_value(0);
        r.events_stopped().write_value(0);
        r.intenset().write(|w| {
            w.set_end(true);
            w.set_started(true);
            w.set_stopped(true);
        });

        // Don't reorder the start event before the previous writes. Hopefully self
        // wouldn't happen anyway
        compiler_fence(Ordering::SeqCst);

        r.tasks_start().write_value(1);

        let mut current_buffer = 0;

        let mut done = false;

        let drop = OnDrop::new(|| {
            r.tasks_stop().write_value(1);
            // N.B. It would be better if this were async, but Drop only support sync code
            while r.events_stopped().read() != 0 {}
        });

        // Wait for events and complete when the sampler indicates it has had enough
        poll_fn(|cx| {
            let r = T::regs();

            T::state().waker.register(cx.waker());

            if r.events_end().read() != 0 {
                compiler_fence(Ordering::SeqCst);

                r.events_end().write_value(0);
                r.intenset().write(|w| w.set_end(true));

                if !done {
                    // Discard the last buffer after the user requested a stop
                    if sampler(&bufs[current_buffer]) == SamplerState::Sampled {
                        let next_buffer = 1 - current_buffer;
                        current_buffer = next_buffer;
                    } else {
                        r.tasks_stop().write_value(1);
                        done = true;
                    };
                };
            }

            if r.events_started().read() != 0 {
                r.events_started().write_value(0);
                r.intenset().write(|w| w.set_started(true));

                let next_buffer = 1 - current_buffer;
                r.sample().ptr().write_value(bufs[next_buffer].as_mut_ptr() as u32);
            }

            if r.events_stopped().read() != 0 {
                return Poll::Ready(());
            }

            Poll::Pending
        })
        .await;
        drop.defuse();
        Ok(())
    }
}

/// PDM microphone driver Config
pub struct Config {
    /// Use stero or mono operation
    pub operation_mode: OperationMode,
    /// On which edge the left channel should be samples
    pub edge: Edge,
    /// Clock frequency
    pub frequency: Frequency,
    /// Clock ratio
    #[cfg(any(
        feature = "nrf52840",
        feature = "nrf52833",
        feature = "_nrf5340-app",
        feature = "_nrf91",
    ))]
    pub ratio: Ratio,
    /// Gain left in dB
    pub gain_left: I7F1,
    /// Gain right in dB
    pub gain_right: I7F1,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            operation_mode: OperationMode::Mono,
            edge: Edge::LeftFalling,
            frequency: Frequency::DEFAULT,
            #[cfg(any(
                feature = "nrf52840",
                feature = "nrf52833",
                feature = "_nrf5340-app",
                feature = "_nrf91",
            ))]
            ratio: Ratio::RATIO80,
            gain_left: I7F1::ZERO,
            gain_right: I7F1::ZERO,
        }
    }
}

/// PDM operation mode
#[derive(PartialEq)]
pub enum OperationMode {
    /// Mono (1 channel)
    Mono,
    /// Stereo (2 channels)
    Stereo,
}

impl From<OperationMode> for vals::Operation {
    fn from(mode: OperationMode) -> Self {
        match mode {
            OperationMode::Mono => vals::Operation::MONO,
            OperationMode::Stereo => vals::Operation::STEREO,
        }
    }
}

/// PDM edge polarity
#[derive(PartialEq)]
pub enum Edge {
    /// Left edge is rising
    LeftRising,
    /// Left edge is falling
    LeftFalling,
}

impl From<Edge> for vals::Edge {
    fn from(edge: Edge) -> Self {
        match edge {
            Edge::LeftRising => vals::Edge::LEFT_RISING,
            Edge::LeftFalling => vals::Edge::LEFT_FALLING,
        }
    }
}

impl<'d, T: Instance> Drop for Pdm<'d, T> {
    fn drop(&mut self) {
        let r = T::regs();

        r.tasks_stop().write_value(1);

        r.enable().write(|w| w.set_enable(false));

        r.psel().din().write_value(DISCONNECTED);
        r.psel().clk().write_value(DISCONNECTED);
    }
}

/// Peripheral static state
pub(crate) struct State {
    waker: AtomicWaker,
}

impl State {
    pub(crate) const fn new() -> Self {
        Self {
            waker: AtomicWaker::new(),
        }
    }
}

pub(crate) trait SealedInstance {
    fn regs() -> crate::pac::pdm::Pdm;
    fn state() -> &'static State;
}

/// PDM peripheral instance
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + 'static + Send {
    /// Interrupt for this peripheral
    type Interrupt: interrupt::typelevel::Interrupt;
}

macro_rules! impl_pdm {
    ($type:ident, $pac_type:ident, $irq:ident) => {
        impl crate::pdm::SealedInstance for peripherals::$type {
            fn regs() -> crate::pac::pdm::Pdm {
                pac::$pac_type
            }
            fn state() -> &'static crate::pdm::State {
                static STATE: crate::pdm::State = crate::pdm::State::new();
                &STATE
            }
        }
        impl crate::pdm::Instance for peripherals::$type {
            type Interrupt = crate::interrupt::typelevel::$irq;
        }
    };
}
