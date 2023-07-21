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
use crate::peripherals::ADC;
use crate::{interrupt, pac, peripherals, Peripheral};

static WAKER: AtomicWaker = AtomicWaker::new();

#[non_exhaustive]
pub struct Config {}

impl Default for Config {
    fn default() -> Self {
        Self {}
    }
}

pub struct Pin<'p> {
    pin: PeripheralRef<'p, AnyPin>,
}

impl<'p> Pin<'p> {
    pub fn new(pin: impl Peripheral<P = impl AdcPin + 'p> + 'p, pull: Pull) -> Self {
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
        Self { pin: pin.map_into() }
    }

    fn channel(&self) -> u8 {
        // this requires adc pins to be sequential and matching the adc channels,
        // which is the case for rp2040
        self.pin._pin() - 26
    }
}

impl<'d> Drop for Pin<'d> {
    fn drop(&mut self) {
        self.pin.pad_ctrl().modify(|w| {
            w.set_ie(true);
            w.set_od(false);
            w.set_pue(false);
            w.set_pde(true);
        });
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

    fn sample_blocking(channel: u8) -> Result<u16, Error> {
        let r = Self::regs();
        r.cs().modify(|w| {
            w.set_ainsel(channel);
            w.set_start_once(true);
            w.set_err(true);
        });
        while !r.cs().read().ready() {}
        match r.cs().read().err() {
            true => Err(Error::ConversionFailed),
            false => Ok(r.result().read().result().into()),
        }
    }

    pub fn blocking_read(&mut self, pin: &mut Pin) -> Result<u16, Error> {
        Self::sample_blocking(pin.channel())
    }

    pub fn blocking_read_temperature(&mut self) -> Result<u16, Error> {
        let r = Self::regs();
        r.cs().modify(|w| w.set_ts_en(true));
        while !r.cs().read().ready() {}
        let result = Self::sample_blocking(4);
        r.cs().modify(|w| w.set_ts_en(false));
        result
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

    async fn sample_async(channel: u8) -> Result<u16, Error> {
        let r = Self::regs();
        r.cs().modify(|w| {
            w.set_ainsel(channel);
            w.set_start_once(true);
            w.set_err(true);
        });
        Self::wait_for_ready().await;
        match r.cs().read().err() {
            true => Err(Error::ConversionFailed),
            false => Ok(r.result().read().result().into()),
        }
    }

    pub async fn read(&mut self, pin: &mut Pin<'_>) -> Result<u16, Error> {
        Self::sample_async(pin.channel()).await
    }

    pub async fn read_temperature(&mut self) -> Result<u16, Error> {
        let r = Self::regs();
        r.cs().modify(|w| w.set_ts_en(true));
        if !r.cs().read().ready() {
            Self::wait_for_ready().await;
        }
        let result = Self::sample_async(4).await;
        r.cs().modify(|w| w.set_ts_en(false));
        result
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
    ($pin:ident) => {
        impl sealed::AdcChannel for peripherals::$pin {}
        impl AdcChannel for peripherals::$pin {}
        impl AdcPin for peripherals::$pin {}
    };
}

impl_pin!(PIN_26);
impl_pin!(PIN_27);
impl_pin!(PIN_28);
impl_pin!(PIN_29);
