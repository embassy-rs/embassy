//! Pulse Density Modulation (PDM) microphone driver

#![macro_use]

use core::cell::UnsafeCell;
use core::future::poll_fn;
use core::marker::PhantomData;
use core::mem::DropGuard;
use core::sync::atomic::{Ordering, compiler_fence};
use core::task::Poll;

use embassy_hal_internal::{Peri, PeripheralType};
use embassy_sync::waitqueue::AtomicWaker;

use crate::gpio::{DISCONNECTED, Flex, OutputDrive, Pull};
use crate::interrupt::typelevel::Interrupt;
use crate::pac::pdm::vals;
#[cfg(not(feature = "_nrf54l"))]
pub use crate::pac::pdm::vals::Freq as Frequency;
#[cfg(feature = "_nrf54lm20")]
pub use crate::pac::pdm::vals::Hppole as HighPassPole;
#[cfg(any(
    feature = "nrf52840",
    feature = "nrf52833",
    feature = "_nrf5340-app",
    feature = "_nrf54l",
    feature = "_nrf91",
))]
pub use crate::pac::pdm::vals::Ratio;
use crate::{interrupt, pac};

/// The maximum buffer size (in 16-bit samples) that the PDM EasyDMA can transfer in one operation.
#[cfg(not(feature = "_nrf54l"))]
pub const DMA_SIZE: usize = crate::util::easy_dma_max!(crate::pac::pdm::regs::Maxcnt, set_buffsize, buffsize);
/// The maximum buffer size (in 16-bit samples) that the PDM EasyDMA can transfer in one operation.
#[cfg(feature = "_nrf54l")]
pub const DMA_SIZE: usize = crate::util::easy_dma_max!(crate::pac::pdm::regs::Maxcnt, set_buffsize, buffsize) / 2;

/// The lowest and highest values of [`Config::prescaler`].
#[cfg(feature = "_nrf54l")]
const PRESCALER_RANGE: core::ops::RangeInclusive<u8> = 4..=126;

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
pub struct Pdm<'d> {
    r: pac::pdm::Pdm,
    state: &'static State,
    _phantom: PhantomData<&'d ()>,
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
    /// Buffer is not aligned to a 32-bit word
    BufferNotAligned,
    /// PDM is not running
    NotRunning,
    /// PDM is already running
    AlreadyRunning,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::BufferTooLong => f.write_str("buffer is too long"),
            Self::BufferZeroLength => f.write_str("buffer is empty"),
            Self::BufferNotAligned => f.write_str("buffer is not aligned to a 32-bit word"),
            Self::NotRunning => f.write_str("PDM is not running"),
            Self::AlreadyRunning => f.write_str("PDM is already running"),
        }
    }
}

impl core::error::Error for Error {}

/// Where samples go while nobody is collecting them. EasyDMA writes to this, so it has to be in RAM.
#[repr(align(4))]
struct DummyBuffer(UnsafeCell<[i16; 1]>);

// Only EasyDMA ever touches the contents, which nothing reads.
unsafe impl Sync for DummyBuffer {}

static DUMMY_BUFFER: DummyBuffer = DummyBuffer(UnsafeCell::new([0; 1]));

/// Points EasyDMA at `samples` 16-bit samples starting at `ptr`.
fn set_buffer(r: pac::pdm::Pdm, ptr: *const i16, samples: usize) {
    r.sample().ptr().write_value(ptr as u32);
    // `MAXCNT` counts bytes on the nRF54L, and samples everywhere else.
    #[cfg(feature = "_nrf54l")]
    let maxcnt = samples * 2;
    #[cfg(not(feature = "_nrf54l"))]
    let maxcnt = samples;
    r.sample().maxcnt().write(|w| w.set_buffsize(maxcnt as _));
}

fn set_dummy_buffer(r: pac::pdm::Pdm) {
    let dummy = DUMMY_BUFFER.0.get();
    set_buffer(r, dummy.cast(), unsafe { (*dummy).len() });
}

/// Starts sampling.
fn trigger_start(r: pac::pdm::Pdm) {
    // On the nRF54LM20A, stopping puts the prescaler back to its reset value of 4 although it goes
    // on reading as written, so without this every run after the first has the wrong sample rate.
    #[cfg(feature = "_nrf54l")]
    r.prescaler().write_value(r.prescaler().read());
    r.tasks_start().write_value(1);
}

/// Nordic requires PDM buffers to be word aligned. Enforced on the nRF54L only, because existing
/// code on the older chips may be getting away with `[i16; N]`, which is only 2-byte aligned.
#[cfg(feature = "_nrf54l")]
fn check_alignment(ptr: *const i16) -> Result<(), Error> {
    if !(ptr as usize).is_multiple_of(4) {
        return Err(Error::BufferNotAligned);
    }
    Ok(())
}

#[cfg(not(feature = "_nrf54l"))]
fn check_alignment(_ptr: *const i16) -> Result<(), Error> {
    Ok(())
}

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

impl<'d> Pdm<'d> {
    /// Create PDM driver
    pub fn new<T: Instance>(
        pdm: Peri<'d, T>,
        clk: Peri<'d, impl ClkPin<T>>,
        din: Peri<'d, impl DinPin<T>>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        config: Config,
    ) -> Self {
        Self::new_inner(pdm, Flex::new(clk), Flex::new(din), config)
    }

    fn new_inner<T: Instance>(_pdm: Peri<'d, T>, mut clk: Flex<'d>, mut din: Flex<'d>, config: Config) -> Self {
        let r = T::regs();

        // setup gpio pins
        // The pins are left configured on drop.
        din.set_as_input(Pull::None);
        r.psel().din().write_value(din.psel_bits());
        din.persist();
        clk.set_low();
        clk.set_as_input_output(Pull::None, OutputDrive::Standard);
        r.psel().clk().write_value(clk.psel_bits());
        clk.persist();

        // configure
        #[cfg(not(feature = "_nrf54l"))]
        r.pdmclkctrl().write(|w| w.set_freq(config.frequency));
        #[cfg(feature = "_nrf54l")]
        {
            assert!(
                PRESCALER_RANGE.contains(&config.prescaler),
                "PDM prescaler out of range"
            );
            r.prescaler().write(|w| w.set_divisor(config.prescaler));
        }
        #[cfg(feature = "_nrf54lm20")]
        {
            r.clkselect().write(|w| w.set_src(vals::Src::Pclk32m));
            match config.high_pass {
                Some(pole) => {
                    r.filter().hppole().write(|w| w.set_hppole(pole));
                    r.filter()
                        .hpdisable()
                        .write(|w| w.set_disable(vals::HpdisableDisable::Enable));
                }
                None => r
                    .filter()
                    .hpdisable()
                    .write(|w| w.set_disable(vals::HpdisableDisable::Disable)),
            }
        }
        #[cfg(any(
            feature = "nrf52840",
            feature = "nrf52833",
            feature = "_nrf5340-app",
            feature = "_nrf54l",
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

        Self {
            r: T::regs(),
            state: T::state(),
            _phantom: PhantomData,
        }
    }

    fn _set_gain(r: pac::pdm::Pdm, gain_left: i8, gain_right: i8) {
        let gain_to_bits = |gain: i8| -> vals::Gain {
            let gain = gain.saturating_add(0x28).clamp(0, 0x50);
            vals::Gain::from_bits(gain as u8)
        };
        r.gainl().write(|w| w.set_gainl(gain_to_bits(gain_left)));
        r.gainr().write(|w| w.set_gainr(gain_to_bits(gain_right)));
    }

    /// Adjust the gain of the PDM microphone on the fly.
    ///
    /// Gains are in units of 0.5 dB, from -40 (-20 dB) to 40 (+20 dB). Values outside that range are clamped.
    pub fn set_gain(&mut self, gain_left: i8, gain_right: i8) {
        Self::_set_gain(self.r, gain_left, gain_right)
    }

    /// Start sampling microphone data into a dummy buffer.
    /// Useful to start the microphone and keep it active between recording samples.
    pub async fn start(&mut self) {
        // start dummy sampling because microphone needs some setup time
        set_dummy_buffer(self.r);

        trigger_start(self.r);
    }

    /// Stop sampling microphone data inta a dummy buffer
    pub async fn stop(&mut self) {
        self.r.tasks_stop().write_value(1);
        self.r.events_started().write_value(0);
    }

    /// Sample data into the given buffer
    pub async fn sample(&mut self, buffer: &mut [i16]) -> Result<(), Error> {
        if buffer.is_empty() {
            return Err(Error::BufferZeroLength);
        }
        if buffer.len() > DMA_SIZE {
            return Err(Error::BufferTooLong);
        }
        check_alignment(buffer.as_ptr())?;

        if self.r.events_started().read() == 0 {
            return Err(Error::NotRunning);
        }

        let r = self.r;
        let drop = DropGuard::new((), move |()| {
            r.intenclr().write(|w| w.set_end(true));
            r.events_stopped().write_value(0);

            // reset to dummy buffer
            set_dummy_buffer(r);

            while r.events_stopped().read() == 0 {}
        });

        // setup user buffer
        set_buffer(self.r, buffer.as_mut_ptr(), buffer.len());

        // wait till the current sample is finished and the user buffer sample is started
        self.wait_for_sample().await;

        // reset the buffer back to the dummy buffer
        set_dummy_buffer(self.r);

        // wait till the user buffer is sampled
        self.wait_for_sample().await;

        DropGuard::dismiss(drop);

        Ok(())
    }

    async fn wait_for_sample(&mut self) {
        self.r.events_end().write_value(0);
        self.r.intenset().write(|w| w.set_end(true));

        compiler_fence(Ordering::SeqCst);

        let state = self.state;
        let r = self.r;
        poll_fn(|cx| {
            state.waker.register(cx.waker());
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
    /// A command is returned from the closure that indicates whether the sampling
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
        if N == 0 {
            return Err(Error::BufferZeroLength);
        }
        if N > DMA_SIZE {
            return Err(Error::BufferTooLong);
        }
        check_alignment(bufs[0].as_ptr())?;
        check_alignment(bufs[1].as_ptr())?;

        if self.r.events_started().read() != 0 {
            return Err(Error::AlreadyRunning);
        }

        set_buffer(self.r, bufs[0].as_mut_ptr(), N);

        // Reset and enable the events
        self.r.events_end().write_value(0);
        self.r.events_started().write_value(0);
        self.r.events_stopped().write_value(0);
        self.r.intenset().write(|w| {
            w.set_end(true);
            w.set_started(true);
            w.set_stopped(true);
        });

        // Don't reorder the start event before the previous writes. Hopefully self
        // wouldn't happen anyway
        compiler_fence(Ordering::SeqCst);

        trigger_start(self.r);

        let mut current_buffer = 0;

        let mut done = false;

        let r = self.r;
        let drop = DropGuard::new((), move |()| {
            r.tasks_stop().write_value(1);
            // N.B. It would be better if this were async, but Drop only support sync code
            while r.events_stopped().read() == 0 {}
        });

        let state = self.state;
        let r = self.r;
        // Wait for events and complete when the sampler indicates it has had enough
        poll_fn(|cx| {
            state.waker.register(cx.waker());

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
        DropGuard::dismiss(drop);
        Ok(())
    }
}

/// PDM microphone driver Config
pub struct Config {
    /// Use stereo or mono operation
    pub operation_mode: OperationMode,
    /// On which edge the left channel should be samples
    pub edge: Edge,
    /// Clock frequency
    #[cfg(not(feature = "_nrf54l"))]
    pub frequency: Frequency,
    /// Clock prescaler, from 4 to 126: `PDM_CLK` is the 32 MHz peripheral clock divided by this.
    ///
    /// The sample rate is `PDM_CLK` divided by [`ratio`](Self::ratio). The default of 25, with
    /// [`Ratio::Ratio80`], gives a 1.28 MHz `PDM_CLK` and 16 kHz exactly.
    #[cfg(feature = "_nrf54l")]
    pub prescaler: u8,
    /// Clock ratio
    #[cfg(any(
        feature = "nrf52840",
        feature = "nrf52833",
        feature = "_nrf5340-app",
        feature = "_nrf54l",
        feature = "_nrf91",
    ))]
    pub ratio: Ratio,
    /// Gain left, in units of 0.5 dB, from -40 (-20 dB) to 40 (+20 dB). Values outside that range are clamped.
    pub gain_left: i8,
    /// Gain right, in units of 0.5 dB, from -40 (-20 dB) to 40 (+20 dB). Values outside that range are clamped.
    pub gain_right: i8,
    /// The corner of the hardware high-pass filter, or `None` to leave it off.
    ///
    /// PDM microphones commonly have a DC offset, which this removes without any software. The
    /// named corners assume a 16 kHz sample rate and scale with it.
    #[cfg(feature = "_nrf54lm20")]
    pub high_pass: Option<HighPassPole>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            operation_mode: OperationMode::Mono,
            edge: Edge::LeftFalling,
            #[cfg(not(feature = "_nrf54l"))]
            frequency: Frequency::Default,
            #[cfg(feature = "_nrf54l")]
            prescaler: 25,
            #[cfg(any(
                feature = "nrf52840",
                feature = "nrf52833",
                feature = "_nrf5340-app",
                feature = "_nrf54l",
                feature = "_nrf91",
            ))]
            ratio: Ratio::Ratio80,
            gain_left: 0,
            gain_right: 0,
            #[cfg(feature = "_nrf54lm20")]
            high_pass: None,
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
            OperationMode::Mono => vals::Operation::Mono,
            OperationMode::Stereo => vals::Operation::Stereo,
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
            Edge::LeftRising => vals::Edge::LeftRising,
            Edge::LeftFalling => vals::Edge::LeftFalling,
        }
    }
}

impl<'d> Drop for Pdm<'d> {
    fn drop(&mut self) {
        self.r.tasks_stop().write_value(1);

        self.r.enable().write(|w| w.set_enable(false));

        self.r.psel().din().write_value(DISCONNECTED);
        self.r.psel().clk().write_value(DISCONNECTED);
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

pin_trait!(
    /// CLK pin trait.
    ClkPin,
    Instance
);

pin_trait!(
    /// DIN pin trait.
    DinPin,
    Instance
);

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
