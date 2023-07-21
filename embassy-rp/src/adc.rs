use core::future::poll_fn;
use core::marker::PhantomData;
use core::sync::atomic::{compiler_fence, Ordering};
use core::task::Poll;

use embassy_hal_internal::{into_ref, PeripheralRef};
use embassy_sync::waitqueue::AtomicWaker;

use crate::gpio::sealed::Pin as GpioPin;
use crate::gpio::{self, AnyPin, Pull};
use crate::interrupt::typelevel::Binding;
use crate::interrupt::InterruptExt;
use crate::peripherals::{ADC, ADC_TEMP_SENSOR};
use crate::{interrupt, pac, peripherals, Peripheral, RegExt};

static WAKER: AtomicWaker = AtomicWaker::new();

#[non_exhaustive]
pub struct Config {}

impl Default for Config {
    fn default() -> Self {
        Self {}
    }
}

enum Source<'p> {
    Pin(PeripheralRef<'p, AnyPin>),
    TempSensor(PeripheralRef<'p, ADC_TEMP_SENSOR>),
}

pub struct Channel<'p>(Source<'p>);

impl<'p> Channel<'p> {
    pub fn new_pin(pin: impl Peripheral<P = impl AdcPin + 'p> + 'p, pull: Pull) -> Self {
        into_ref!(pin);
        pin.pad_ctrl().modify(|w| {
            // manual says:
            //
            // > When using an ADC input shared with a GPIO pin, the pin’s
            // > digital functions must be disabled by setting IE low and OD
            // > high in the pin’s pad control register
            w.set_ie(false);
            w.set_od(true);
            w.set_pue(pull == Pull::Up);
            w.set_pde(pull == Pull::Down);
        });
        Self(Source::Pin(pin.map_into()))
    }

    pub fn new_sensor(s: impl Peripheral<P = ADC_TEMP_SENSOR> + 'p) -> Self {
        let r = pac::ADC;
        r.cs().write_set(|w| w.set_ts_en(true));
        Self(Source::TempSensor(s.into_ref()))
    }

    fn channel(&self) -> u8 {
        match &self.0 {
            // this requires adc pins to be sequential and matching the adc channels,
            // which is the case for rp2040
            Source::Pin(p) => p._pin() - 26,
            Source::TempSensor(_) => 4,
        }
    }
}

impl<'p> Drop for Source<'p> {
    fn drop(&mut self) {
        match self {
            Source::Pin(p) => {
                p.pad_ctrl().modify(|w| {
                    w.set_ie(true);
                    w.set_od(false);
                    w.set_pue(false);
                    w.set_pde(true);
                });
            }
            Source::TempSensor(_) => {
                pac::ADC.cs().write_clear(|w| w.set_ts_en(true));
            }
        }
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    ConversionFailed,
}

pub trait Mode {}

pub struct Async;
impl Mode for Async {}

pub struct Blocking;
impl Mode for Blocking {}

pub struct Adc<'d, M: Mode> {
    phantom: PhantomData<(&'d ADC, M)>,
}

impl<'d, M: Mode> Drop for Adc<'d, M> {
    fn drop(&mut self) {
        let r = Self::regs();
        // disable ADC. leaving it enabled comes with a ~150µA static
        // current draw. the temperature sensor has already been disabled
        // by the temperature-reading methods, so we don't need to touch that.
        r.cs().write(|w| w.set_en(false));
    }
}

impl<'d, M: Mode> Adc<'d, M> {
    #[inline]
    fn regs() -> pac::adc::Adc {
        pac::ADC
    }

    #[inline]
    fn reset() -> pac::resets::regs::Peripherals {
        let mut ret = pac::resets::regs::Peripherals::default();
        ret.set_adc(true);
        ret
    }

    fn setup() {
        let reset = Self::reset();
        crate::reset::reset(reset);
        crate::reset::unreset_wait(reset);
        let r = Self::regs();
        // Enable ADC
        r.cs().write(|w| w.set_en(true));
        // Wait for ADC ready
        while !r.cs().read().ready() {}
    }

    pub fn blocking_read(&mut self, ch: &mut Channel) -> Result<u16, Error> {
        let r = Self::regs();
        r.cs().modify(|w| {
            w.set_ainsel(ch.channel());
            w.set_start_once(true);
            w.set_err(true);
        });
        while !r.cs().read().ready() {}
        match r.cs().read().err() {
            true => Err(Error::ConversionFailed),
            false => Ok(r.result().read().result().into()),
        }
    }
}

impl<'d> Adc<'d, Async> {
    pub fn new(
        _inner: impl Peripheral<P = ADC> + 'd,
        _irq: impl Binding<interrupt::typelevel::ADC_IRQ_FIFO, InterruptHandler>,
        _config: Config,
    ) -> Self {
        Self::setup();

        // Setup IRQ
        interrupt::ADC_IRQ_FIFO.unpend();
        unsafe { interrupt::ADC_IRQ_FIFO.enable() };

        Self { phantom: PhantomData }
    }

    async fn wait_for_ready() {
        let r = Self::regs();
        r.inte().write(|w| w.set_fifo(true));
        compiler_fence(Ordering::SeqCst);
        poll_fn(|cx| {
            WAKER.register(cx.waker());
            if r.cs().read().ready() {
                return Poll::Ready(());
            }
            Poll::Pending
        })
        .await;
    }

    pub async fn read(&mut self, ch: &mut Channel<'_>) -> Result<u16, Error> {
        let r = Self::regs();
        r.cs().modify(|w| {
            w.set_ainsel(ch.channel());
            w.set_start_once(true);
            w.set_err(true);
        });
        Self::wait_for_ready().await;
        match r.cs().read().err() {
            true => Err(Error::ConversionFailed),
            false => Ok(r.result().read().result().into()),
        }
    }
}

impl<'d> Adc<'d, Blocking> {
    pub fn new_blocking(_inner: impl Peripheral<P = ADC> + 'd, _config: Config) -> Self {
        Self::setup();

        Self { phantom: PhantomData }
    }
}

pub struct InterruptHandler {
    _empty: (),
}

impl interrupt::typelevel::Handler<interrupt::typelevel::ADC_IRQ_FIFO> for InterruptHandler {
    unsafe fn on_interrupt() {
        let r = Adc::<Async>::regs();
        r.inte().write(|w| w.set_fifo(false));
        WAKER.wake();
    }
}

mod sealed {
    pub trait AdcChannel {}
}

pub trait AdcChannel: sealed::AdcChannel {}
pub trait AdcPin: AdcChannel + gpio::Pin {}

macro_rules! impl_pin {
    ($pin:ident, $channel:expr) => {
        impl sealed::AdcChannel for peripherals::$pin {}
        impl AdcChannel for peripherals::$pin {}
        impl AdcPin for peripherals::$pin {}
    };
}

impl_pin!(PIN_26, 0);
impl_pin!(PIN_27, 1);
impl_pin!(PIN_28, 2);
impl_pin!(PIN_29, 3);

impl sealed::AdcChannel for peripherals::ADC_TEMP_SENSOR {}
impl AdcChannel for peripherals::ADC_TEMP_SENSOR {}
