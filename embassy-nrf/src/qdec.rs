//! Quadrature decoder (QDEC) driver.

#![macro_use]

use core::future::poll_fn;
use core::marker::PhantomData;
use core::task::Poll;

use embassy_hal_internal::{Peri, PeripheralType};
use embassy_sync::waitqueue::AtomicWaker;

use crate::gpio::{Flex, OutputDrive, Pull};
use crate::interrupt::typelevel::Interrupt;
use crate::pac::qdec::vals;
use crate::{interrupt, pac};

/// Quadrature decoder driver.
pub struct Qdec<'d> {
    r: pac::qdec::Qdec,
    state: &'static State,
    _phantom: PhantomData<&'d ()>,
}

/// QDEC config
#[non_exhaustive]
pub struct Config {
    /// Number of samples
    pub num_samples: NumSamples,
    /// Sample period
    pub period: SamplePeriod,
    /// Set LED output pin polarity
    pub led_polarity: LedPolarity,
    /// Enable/disable input debounce filters
    pub debounce: bool,
    /// Time period the LED is switched ON prior to sampling (0..511 us).
    pub led_pre_usecs: u16,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            num_samples: NumSamples::_1smpl,
            period: SamplePeriod::_256us,
            led_polarity: LedPolarity::ActiveHigh,
            debounce: true,
            led_pre_usecs: 0,
        }
    }
}

/// Interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        T::regs().intenclr().write(|w| w.set_reportrdy(true));
        T::state().waker.wake();
    }
}

impl<'d> Qdec<'d> {
    /// Create a new QDEC.
    pub fn new<T: Instance>(
        qdec: Peri<'d, T>,
        a: Peri<'d, impl APin<T>>,
        b: Peri<'d, impl BPin<T>>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        config: Config,
    ) -> Self {
        Self::new_inner(qdec, Flex::new(a), Flex::new(b), None, config)
    }

    /// Create a new QDEC, with a pin for LED output.
    pub fn new_with_led<T: Instance>(
        qdec: Peri<'d, T>,
        a: Peri<'d, impl APin<T>>,
        b: Peri<'d, impl BPin<T>>,
        led: Peri<'d, impl LedPin<T>>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        config: Config,
    ) -> Self {
        Self::new_inner(qdec, Flex::new(a), Flex::new(b), Some(Flex::new(led)), config)
    }

    fn new_inner<T: Instance>(
        _p: Peri<'d, T>,
        mut a: Flex<'d>,
        mut b: Flex<'d>,
        led: Option<Flex<'d>>,
        config: Config,
    ) -> Self {
        let r = T::regs();

        // Select pins. They're left configured on drop.
        a.set_as_input(Pull::Up);
        b.set_as_input(Pull::Up);
        r.psel().a().write_value(a.psel_bits());
        r.psel().b().write_value(b.psel_bits());
        a.persist();
        b.persist();
        if let Some(mut led_pin) = led {
            led_pin.set_as_input_output(Pull::None, OutputDrive::Standard);
            r.psel().led().write_value(led_pin.psel_bits());
            led_pin.persist();
        }

        // Enables/disable input debounce filters
        r.dbfen().write(|w| match config.debounce {
            true => w.set_dbfen(true),
            false => w.set_dbfen(false),
        });

        // Set LED output pin polarity
        r.ledpol().write(|w| match config.led_polarity {
            LedPolarity::ActiveHigh => w.set_ledpol(vals::Ledpol::ActiveHigh),
            LedPolarity::ActiveLow => w.set_ledpol(vals::Ledpol::ActiveLow),
        });

        // Set time period the LED is switched ON prior to sampling (0..511 us).
        r.ledpre().write(|w| w.set_ledpre(config.led_pre_usecs.min(511)));

        // Set sample period
        r.sampleper().write(|w| match config.period {
            SamplePeriod::_128us => w.set_sampleper(vals::Sampleper::_128us),
            SamplePeriod::_256us => w.set_sampleper(vals::Sampleper::_256us),
            SamplePeriod::_512us => w.set_sampleper(vals::Sampleper::_512us),
            SamplePeriod::_1024us => w.set_sampleper(vals::Sampleper::_1024us),
            SamplePeriod::_2048us => w.set_sampleper(vals::Sampleper::_2048us),
            SamplePeriod::_4096us => w.set_sampleper(vals::Sampleper::_4096us),
            SamplePeriod::_8192us => w.set_sampleper(vals::Sampleper::_8192us),
            SamplePeriod::_16384us => w.set_sampleper(vals::Sampleper::_16384us),
            SamplePeriod::_32ms => w.set_sampleper(vals::Sampleper::_32ms),
            SamplePeriod::_65ms => w.set_sampleper(vals::Sampleper::_65ms),
            SamplePeriod::_131ms => w.set_sampleper(vals::Sampleper::_131ms),
        });

        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        // Enable peripheral
        r.enable().write(|w| w.set_enable(true));

        // Start sampling
        r.tasks_start().write_value(1);

        Self {
            r: T::regs(),
            state: T::state(),
            _phantom: PhantomData,
        }
    }

    /// Perform an asynchronous read of the decoder.
    /// The returned future can be awaited to obtain the number of steps
    /// accumulated since the previous read.
    ///
    /// This method is cancel-safe: if the future is dropped before it
    /// completes, no counts are lost; they stay in the accumulator and are
    /// returned by the next read.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use embassy_nrf::qdec::{self, Qdec};
    /// use embassy_nrf::{bind_interrupts, peripherals};
    ///
    /// bind_interrupts!(struct Irqs {
    ///     QDEC => qdec::InterruptHandler<peripherals::QDEC>;
    /// });
    ///
    /// # async {
    /// # let p: embassy_nrf::Peripherals = todo!();
    /// let config = qdec::Config::default();
    /// let mut q = Qdec::new(p.QDEC, p.P0_31, p.P0_30, Irqs, config);
    /// let delta = q.read().await;
    /// # };
    /// ```
    pub async fn read(&mut self) -> i16 {
        self.r.intenset().write(|w| w.set_reportrdy(true));

        let state = self.state;
        let r = self.r;
        poll_fn(move |cx| {
            state.waker.register(cx.waker());
            if r.events_reportrdy().read() == 0 {
                Poll::Pending
            } else {
                r.events_reportrdy().write_value(0);
                r.tasks_readclracc().write_value(1);
                let acc = r.accread().read();
                Poll::Ready(acc as i16)
            }
        })
        .await
    }
}

/// Sample period
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum SamplePeriod {
    /// 128 us
    _128us,
    /// 256 us
    _256us,
    /// 512 us
    _512us,
    /// 1024 us
    _1024us,
    /// 2048 us
    _2048us,
    /// 4096 us
    _4096us,
    /// 8192 us
    _8192us,
    /// 16384 us
    _16384us,
    /// 32 ms
    _32ms,
    /// 65 ms
    _65ms,
    /// 131 ms
    _131ms,
}

/// Number of samples taken.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum NumSamples {
    /// 10 samples
    _10smpl,
    /// 40 samples
    _40smpl,
    /// 80 samples
    _80smpl,
    /// 120 samples
    _120smpl,
    /// 160 samples
    _160smpl,
    /// 200 samples
    _200smpl,
    /// 240 samples
    _240smpl,
    /// 280 samples
    _280smpl,
    /// 1 sample
    _1smpl,
}

/// LED polarity
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum LedPolarity {
    /// Active high (a high output turns on the LED).
    ActiveHigh,
    /// Active low (a low output turns on the LED).
    ActiveLow,
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
    fn regs() -> pac::qdec::Qdec;
    fn state() -> &'static State;
}

/// qdec peripheral instance.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + 'static + Send {
    /// Interrupt for this peripheral.
    type Interrupt: interrupt::typelevel::Interrupt;
}

pin_trait!(
    /// phase A pin trait.
    APin,
    Instance
);

pin_trait!(
    /// phase B pin trait.
    BPin,
    Instance
);

pin_trait!(
    /// LED pin trait.
    LedPin,
    Instance
);

macro_rules! impl_qdec {
    ($type:ident, $pac_type:ident, $irq:ident) => {
        impl crate::qdec::SealedInstance for peripherals::$type {
            fn regs() -> pac::qdec::Qdec {
                pac::$pac_type
            }
            fn state() -> &'static crate::qdec::State {
                static STATE: crate::qdec::State = crate::qdec::State::new();
                &STATE
            }
        }
        impl crate::qdec::Instance for peripherals::$type {
            type Interrupt = crate::interrupt::typelevel::$irq;
        }
    };
}
