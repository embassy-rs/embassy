//! Quadrature decoder (QDEC) driver.

#![macro_use]

use core::future::poll_fn;
use core::marker::PhantomData;
use core::task::Poll;

use embassy_hal_internal::{into_ref, PeripheralRef};

use crate::gpio::sealed::Pin as _;
use crate::gpio::{AnyPin, Pin as GpioPin};
use crate::interrupt::typelevel::Interrupt;
use crate::{interrupt, Peripheral};

/// Quadrature decoder driver.
pub struct Qdec<'d, T: Instance> {
    _p: PeripheralRef<'d, T>,
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
        T::regs().intenclr.write(|w| w.reportrdy().clear());
        T::state().waker.wake();
    }
}

impl<'d, T: Instance> Qdec<'d, T> {
    /// Create a new QDEC.
    pub fn new(
        qdec: impl Peripheral<P = T> + 'd,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        a: impl Peripheral<P = impl GpioPin> + 'd,
        b: impl Peripheral<P = impl GpioPin> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(qdec, a, b);
        Self::new_inner(qdec, a.map_into(), b.map_into(), None, config)
    }

    /// Create a new QDEC, with a pin for LED output.
    pub fn new_with_led(
        qdec: impl Peripheral<P = T> + 'd,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        a: impl Peripheral<P = impl GpioPin> + 'd,
        b: impl Peripheral<P = impl GpioPin> + 'd,
        led: impl Peripheral<P = impl GpioPin> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(qdec, a, b, led);
        Self::new_inner(qdec, a.map_into(), b.map_into(), Some(led.map_into()), config)
    }

    fn new_inner(
        p: PeripheralRef<'d, T>,
        a: PeripheralRef<'d, AnyPin>,
        b: PeripheralRef<'d, AnyPin>,
        led: Option<PeripheralRef<'d, AnyPin>>,
        config: Config,
    ) -> Self {
        let r = T::regs();

        // Select pins.
        a.conf().write(|w| w.input().connect().pull().pullup());
        b.conf().write(|w| w.input().connect().pull().pullup());
        r.psel.a.write(|w| unsafe { w.bits(a.psel_bits()) });
        r.psel.b.write(|w| unsafe { w.bits(b.psel_bits()) });
        if let Some(led_pin) = &led {
            led_pin.conf().write(|w| w.dir().output());
            r.psel.led.write(|w| unsafe { w.bits(led_pin.psel_bits()) });
        }

        // Enables/disable input debounce filters
        r.dbfen.write(|w| match config.debounce {
            true => w.dbfen().enabled(),
            false => w.dbfen().disabled(),
        });

        // Set LED output pin polarity
        r.ledpol.write(|w| match config.led_polarity {
            LedPolarity::ActiveHigh => w.ledpol().active_high(),
            LedPolarity::ActiveLow => w.ledpol().active_low(),
        });

        // Set time period the LED is switched ON prior to sampling (0..511 us).
        r.ledpre
            .write(|w| unsafe { w.ledpre().bits(config.led_pre_usecs.min(511)) });

        // Set sample period
        r.sampleper.write(|w| match config.period {
            SamplePeriod::_128us => w.sampleper()._128us(),
            SamplePeriod::_256us => w.sampleper()._256us(),
            SamplePeriod::_512us => w.sampleper()._512us(),
            SamplePeriod::_1024us => w.sampleper()._1024us(),
            SamplePeriod::_2048us => w.sampleper()._2048us(),
            SamplePeriod::_4096us => w.sampleper()._4096us(),
            SamplePeriod::_8192us => w.sampleper()._8192us(),
            SamplePeriod::_16384us => w.sampleper()._16384us(),
            SamplePeriod::_32ms => w.sampleper()._32ms(),
            SamplePeriod::_65ms => w.sampleper()._65ms(),
            SamplePeriod::_131ms => w.sampleper()._131ms(),
        });

        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        // Enable peripheral
        r.enable.write(|w| w.enable().set_bit());

        // Start sampling
        unsafe { r.tasks_start.write(|w| w.bits(1)) };

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
        t.intenset.write(|w| w.reportrdy().set());
        unsafe { t.tasks_readclracc.write(|w| w.bits(1)) };

        let value = poll_fn(|cx| {
            T::state().waker.register(cx.waker());
            if t.events_reportrdy.read().bits() == 0 {
                return Poll::Pending;
            } else {
                t.events_reportrdy.reset();
                let acc = t.accread.read().bits();
                Poll::Ready(acc as i16)
            }
        })
        .await;
        value
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
        fn regs() -> &'static crate::pac::qdec::RegisterBlock;
        fn state() -> &'static State;
    }
}

/// qdec peripheral instance.
pub trait Instance: Peripheral<P = Self> + sealed::Instance + 'static + Send {
    /// Interrupt for this peripheral.
    type Interrupt: interrupt::typelevel::Interrupt;
}

macro_rules! impl_qdec {
    ($type:ident, $pac_type:ident, $irq:ident) => {
        impl crate::qdec::sealed::Instance for peripherals::$type {
            fn regs() -> &'static crate::pac::qdec::RegisterBlock {
                unsafe { &*pac::$pac_type::ptr() }
            }
            fn state() -> &'static crate::qdec::sealed::State {
                static STATE: crate::qdec::sealed::State = crate::qdec::sealed::State::new();
                &STATE
            }
        }
        impl crate::qdec::Instance for peripherals::$type {
            type Interrupt = crate::interrupt::typelevel::$irq;
        }
    };
}
