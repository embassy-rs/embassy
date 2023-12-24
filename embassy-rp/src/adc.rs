//! ADC driver.
use core::future::poll_fn;
use core::marker::PhantomData;
use core::mem;
use core::sync::atomic::{compiler_fence, Ordering};
use core::task::Poll;

use embassy_hal_internal::{into_ref, PeripheralRef};
use embassy_sync::waitqueue::AtomicWaker;

use crate::gpio::sealed::Pin as GpioPin;
use crate::gpio::{self, AnyPin, Pull};
use crate::interrupt::typelevel::Binding;
use crate::interrupt::InterruptExt;
use crate::peripherals::{ADC, ADC_TEMP_SENSOR};
use crate::{dma, interrupt, pac, peripherals, Peripheral, RegExt};

static WAKER: AtomicWaker = AtomicWaker::new();

/// ADC config.
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

/// ADC channel.
pub struct Channel<'p>(Source<'p>);

impl<'p> Channel<'p> {
    /// Create a new ADC channel from pin with the provided [Pull] configuration.
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

    /// Create a new ADC channel for the internal temperature sensor.
    pub fn new_temp_sensor(s: impl Peripheral<P = ADC_TEMP_SENSOR> + 'p) -> Self {
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

/// ADC sample.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(transparent)]
pub struct Sample(u16);

impl Sample {
    /// Sample is valid.
    pub fn good(&self) -> bool {
        self.0 < 0x8000
    }

    /// Sample value.
    pub fn value(&self) -> u16 {
        self.0 & !0x8000
    }
}

/// ADC error.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// Error converting value.
    ConversionFailed,
}

/// ADC mode.
pub trait Mode {}

/// ADC async mode.
pub struct Async;
impl Mode for Async {}

/// ADC blocking mode.
pub struct Blocking;
impl Mode for Blocking {}

/// ADC driver.
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

    /// Sample a value from a channel in blocking mode.
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
    /// Create ADC driver in async mode.
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

    /// Sample a value from a channel until completed.
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

    async fn read_many_inner<W: dma::Word>(
        &mut self,
        ch: &mut Channel<'_>,
        buf: &mut [W],
        fcs_err: bool,
        div: u16,
        dma: impl Peripheral<P = impl dma::Channel>,
    ) -> Result<(), Error> {
        let r = Self::regs();
        // clear previous errors and set channel
        r.cs().modify(|w| {
            w.set_ainsel(ch.channel());
            w.set_err_sticky(true); // clear previous errors
            w.set_start_many(false);
        });
        // wait for previous conversions and drain fifo. an earlier batch read may have
        // been cancelled, leaving the adc running.
        while !r.cs().read().ready() {}
        while !r.fcs().read().empty() {
            r.fifo().read();
        }

        // set up fifo for dma
        r.fcs().write(|w| {
            w.set_thresh(1);
            w.set_dreq_en(true);
            w.set_shift(mem::size_of::<W>() == 1);
            w.set_en(true);
            w.set_err(fcs_err);
        });

        // reset dma config on drop, regardless of whether it was a future being cancelled
        // or the method returning normally.
        struct ResetDmaConfig;
        impl Drop for ResetDmaConfig {
            fn drop(&mut self) {
                pac::ADC.cs().write_clear(|w| w.set_start_many(true));
                while !pac::ADC.cs().read().ready() {}
                pac::ADC.fcs().write_clear(|w| {
                    w.set_dreq_en(true);
                    w.set_shift(true);
                    w.set_en(true);
                });
            }
        }
        let auto_reset = ResetDmaConfig;

        let dma = unsafe { dma::read(dma, r.fifo().as_ptr() as *const W, buf as *mut [W], 36) };
        // start conversions and wait for dma to finish. we can't report errors early
        // because there's no interrupt to signal them, and inspecting every element
        // of the fifo is too costly to do here.
        r.div().write_set(|w| w.set_int(div));
        r.cs().write_set(|w| w.set_start_many(true));
        dma.await;
        mem::drop(auto_reset);
        // we can't report errors before the conversions have ended since no interrupt
        // exists to report them early, and since they're exceedingly rare we probably don't
        // want to anyway.
        match r.cs().read().err_sticky() {
            false => Ok(()),
            true => Err(Error::ConversionFailed),
        }
    }

    /// Sample multiple values from a channel using DMA.
    #[inline]
    pub async fn read_many<S: AdcSample>(
        &mut self,
        ch: &mut Channel<'_>,
        buf: &mut [S],
        div: u16,
        dma: impl Peripheral<P = impl dma::Channel>,
    ) -> Result<(), Error> {
        self.read_many_inner(ch, buf, false, div, dma).await
    }

    /// Sample multiple values from a channel using DMA with errors inlined in samples.
    #[inline]
    pub async fn read_many_raw(
        &mut self,
        ch: &mut Channel<'_>,
        buf: &mut [Sample],
        div: u16,
        dma: impl Peripheral<P = impl dma::Channel>,
    ) {
        // errors are reported in individual samples
        let _ = self
            .read_many_inner(ch, unsafe { mem::transmute::<_, &mut [u16]>(buf) }, true, div, dma)
            .await;
    }
}

impl<'d> Adc<'d, Blocking> {
    /// Create ADC driver in blocking mode.
    pub fn new_blocking(_inner: impl Peripheral<P = ADC> + 'd, _config: Config) -> Self {
        Self::setup();

        Self { phantom: PhantomData }
    }
}

/// Interrupt handler.
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
    pub trait AdcSample: crate::dma::Word {}

    pub trait AdcChannel {}
}

/// ADC sample.
pub trait AdcSample: sealed::AdcSample {}

impl sealed::AdcSample for u16 {}
impl AdcSample for u16 {}

impl sealed::AdcSample for u8 {}
impl AdcSample for u8 {}

/// ADC channel.
pub trait AdcChannel: sealed::AdcChannel {}
/// ADC pin.
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
