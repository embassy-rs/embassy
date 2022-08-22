//! Quadrature decoder interface

use core::task::Poll;

use embassy_hal_common::{into_ref, PeripheralRef};
use embassy_sync::waitqueue::AtomicWaker;
use futures::future::poll_fn;

use crate::gpio::sealed::Pin as _;
use crate::gpio::{AnyPin, Pin as GpioPin};
use crate::interrupt::InterruptExt;
use crate::peripherals::QDEC;
use crate::{interrupt, pac, Peripheral};

/// Quadrature decoder
pub struct Qdec<'d> {
    _p: PeripheralRef<'d, QDEC>,
}

#[non_exhaustive]
pub struct Config {
    pub num_samples: NumSamples,
    pub period: SamplePeriod,
    pub led_polarity: LedPolarity,
    pub debounce: bool,
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

static WAKER: AtomicWaker = AtomicWaker::new();

impl<'d> Qdec<'d> {
    pub fn new(
        qdec: impl Peripheral<P = QDEC> + 'd,
        irq: impl Peripheral<P = interrupt::QDEC> + 'd,
        a: impl Peripheral<P = impl GpioPin> + 'd,
        b: impl Peripheral<P = impl GpioPin> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(a, b);
        Self::new_inner(qdec, irq, a.map_into(), b.map_into(), None, config)
    }

    pub fn new_with_led(
        qdec: impl Peripheral<P = QDEC> + 'd,
        irq: impl Peripheral<P = interrupt::QDEC> + 'd,
        a: impl Peripheral<P = impl GpioPin> + 'd,
        b: impl Peripheral<P = impl GpioPin> + 'd,
        led: impl Peripheral<P = impl GpioPin> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(a, b, led);
        Self::new_inner(qdec, irq, a.map_into(), b.map_into(), Some(led.map_into()), config)
    }

    fn new_inner(
        p: impl Peripheral<P = QDEC> + 'd,
        irq: impl Peripheral<P = interrupt::QDEC> + 'd,
        a: PeripheralRef<'d, AnyPin>,
        b: PeripheralRef<'d, AnyPin>,
        led: Option<PeripheralRef<'d, AnyPin>>,
        config: Config,
    ) -> Self {
        into_ref!(p, irq);
        let r = Self::regs();

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

        // Enable peripheral
        r.enable.write(|w| w.enable().set_bit());

        // Start sampling
        unsafe { r.tasks_start.write(|w| w.bits(1)) };

        irq.disable();
        irq.set_handler(|_| {
            let r = Self::regs();
            r.intenclr.write(|w| w.reportrdy().clear());
            WAKER.wake();
        });
        irq.enable();

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
    /// let irq = interrupt::take!(QDEC);
    /// let config = qdec::Config::default();
    /// let mut q = Qdec::new(p.QDEC, p.P0_31, p.P0_30, config);
    /// let delta = q.read().await;
    /// ```
    pub async fn read(&mut self) -> i16 {
        let t = Self::regs();
        t.intenset.write(|w| w.reportrdy().set());
        unsafe { t.tasks_readclracc.write(|w| w.bits(1)) };

        let value = poll_fn(|cx| {
            WAKER.register(cx.waker());
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

    fn regs() -> &'static pac::qdec::RegisterBlock {
        unsafe { &*pac::QDEC::ptr() }
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum SamplePeriod {
    _128us,
    _256us,
    _512us,
    _1024us,
    _2048us,
    _4096us,
    _8192us,
    _16384us,
    _32ms,
    _65ms,
    _131ms,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum NumSamples {
    _10smpl,
    _40smpl,
    _80smpl,
    _120smpl,
    _160smpl,
    _200smpl,
    _240smpl,
    _280smpl,
    _1smpl,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum LedPolarity {
    ActiveHigh,
    ActiveLow,
}
