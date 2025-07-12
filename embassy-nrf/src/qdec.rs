//! Quadrature decoder (QDEC) driver.

#![macro_use]

use core::future::poll_fn;
use core::marker::PhantomData;
use core::task::Poll;

use embassy_hal_internal::{Peri, PeripheralType};
use embassy_sync::waitqueue::AtomicWaker;

use crate::gpio::{AnyPin, Pin as GpioPin, SealedPin as _};
use crate::interrupt::typelevel::Interrupt;
use crate::pac::gpio::vals as gpiovals;
use crate::pac::qdec::vals;
use crate::{interrupt, pac};

/// Quadrature decoder driver.
pub struct Qdec<'d, T: Instance> {
    _p: Peri<'d, T>,
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

impl<'d, T: Instance> Qdec<'d, T> {
    /// Create a new QDEC.
    pub fn new(
        qdec: Peri<'d, T>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        a: Peri<'d, impl GpioPin>,
        b: Peri<'d, impl GpioPin>,
        config: Config,
    ) -> Self {
        Self::new_inner(qdec, a.into(), b.into(), None, config)
    }

    /// Create a new QDEC, with a pin for LED output.
    pub fn new_with_led(
        qdec: Peri<'d, T>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        a: Peri<'d, impl GpioPin>,
        b: Peri<'d, impl GpioPin>,
        led: Peri<'d, impl GpioPin>,
        config: Config,
    ) -> Self {
        Self::new_inner(qdec, a.into(), b.into(), Some(led.into()), config)
    }

    fn new_inner(
        p: Peri<'d, T>,
        a: Peri<'d, AnyPin>,
        b: Peri<'d, AnyPin>,
        led: Option<Peri<'d, AnyPin>>,
        config: Config,
    ) -> Self {
        let r = T::regs();

        // Select pins.
        a.conf().write(|w| {
            w.set_input(gpiovals::Input::CONNECT);
            w.set_pull(gpiovals::Pull::PULLUP);
        });
        b.conf().write(|w| {
            w.set_input(gpiovals::Input::CONNECT);
            w.set_pull(gpiovals::Pull::PULLUP);
        });
        r.psel().a().write_value(a.psel_bits());
        r.psel().b().write_value(b.psel_bits());
        if let Some(led_pin) = &led {
            led_pin.conf().write(|w| w.set_dir(gpiovals::Dir::OUTPUT));
            r.psel().led().write_value(led_pin.psel_bits());
        }

        // Enables/disable input debounce filters
        r.dbfen().write(|w| match config.debounce {
            true => w.set_dbfen(true),
            false => w.set_dbfen(false),
        });

        // Set LED output pin polarity
        r.ledpol().write(|w| match config.led_polarity {
            LedPolarity::ActiveHigh => w.set_ledpol(vals::Ledpol::ACTIVE_HIGH),
            LedPolarity::ActiveLow => w.set_ledpol(vals::Ledpol::ACTIVE_LOW),
        });

        // Set time period the LED is switched ON prior to sampling (0..511 us).
        r.ledpre().write(|w| w.set_ledpre(config.led_pre_usecs.min(511)));

        // Set sample period
        r.sampleper().write(|w| match config.period {
            SamplePeriod::_128us => w.set_sampleper(vals::Sampleper::_128US),
            SamplePeriod::_256us => w.set_sampleper(vals::Sampleper::_256US),
            SamplePeriod::_512us => w.set_sampleper(vals::Sampleper::_512US),
            SamplePeriod::_1024us => w.set_sampleper(vals::Sampleper::_1024US),
            SamplePeriod::_2048us => w.set_sampleper(vals::Sampleper::_2048US),
            SamplePeriod::_4096us => w.set_sampleper(vals::Sampleper::_4096US),
            SamplePeriod::_8192us => w.set_sampleper(vals::Sampleper::_8192US),
            SamplePeriod::_16384us => w.set_sampleper(vals::Sampleper::_16384US),
            SamplePeriod::_32ms => w.set_sampleper(vals::Sampleper::_32MS),
            SamplePeriod::_65ms => w.set_sampleper(vals::Sampleper::_65MS),
            SamplePeriod::_131ms => w.set_sampleper(vals::Sampleper::_131MS),
        });

        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        // Enable peripheral
        r.enable().write(|w| w.set_enable(true));

        // Start sampling
        r.tasks_start().write_value(1);

        Self { _p: p }
    }

    /// Perform an asynchronous read of the decoder.
    /// The returned future can be awaited to obtain the number of steps.
    ///
    /// If the future is dropped, the read is cancelled.
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
    /// let mut q = Qdec::new(p.QDEC, Irqs, p.P0_31, p.P0_30, config);
    /// let delta = q.read().await;
    /// # };
    /// ```
    pub async fn read(&mut self) -> i16 {
        let t = T::regs();
        t.intenset().write(|w| w.set_reportrdy(true));
        t.tasks_readclracc().write_value(1);

        poll_fn(|cx| {
            T::state().waker.register(cx.waker());
            if t.events_reportrdy().read() == 0 {
                Poll::Pending
            } else {
                t.events_reportrdy().write_value(0);
                let acc = t.accread().read();
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
