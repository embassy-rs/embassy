//! ADC driver.
use core::future::{poll_fn, Future};
use core::marker::PhantomData;
use core::mem;
use core::sync::atomic::{compiler_fence, Ordering};
use core::task::Poll;

use embassy_sync::waitqueue::AtomicWaker;

use crate::gpio::{self, AnyPin, Pull, SealedPin as GpioPin};
use crate::interrupt::typelevel::Binding;
use crate::interrupt::InterruptExt;
use crate::pac::dma::vals::TreqSel;
use crate::peripherals::{ADC, ADC_TEMP_SENSOR};
use crate::{dma, interrupt, pac, peripherals, Peri, RegExt};

static WAKER: AtomicWaker = AtomicWaker::new();

/// ADC config.
#[non_exhaustive]
#[derive(Default)]
pub struct Config {}

enum Source<'p> {
    Pin(Peri<'p, AnyPin>),
    TempSensor(Peri<'p, ADC_TEMP_SENSOR>),
}

/// ADC channel.
pub struct Channel<'p>(Source<'p>);

impl<'p> Channel<'p> {
    /// Create a new ADC channel from pin with the provided [Pull] configuration.
    pub fn new_pin(pin: Peri<'p, impl AdcPin + 'p>, pull: Pull) -> Self {
        pin.pad_ctrl().modify(|w| {
            #[cfg(feature = "_rp235x")]
            w.set_iso(false);
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
        Self(Source::Pin(pin.into()))
    }

    /// Create a new ADC channel for the internal temperature sensor.
    pub fn new_temp_sensor(s: Peri<'p, ADC_TEMP_SENSOR>) -> Self {
        let r = pac::ADC;
        r.cs().write_set(|w| w.set_ts_en(true));
        Self(Source::TempSensor(s))
    }

    fn channel(&self) -> u8 {
        #[cfg(any(feature = "rp2040", feature = "rp235xa"))]
        const CH_OFFSET: u8 = 26;
        #[cfg(feature = "rp235xb")]
        const CH_OFFSET: u8 = 40;

        #[cfg(any(feature = "rp2040", feature = "rp235xa"))]
        const TS_CHAN: u8 = 4;
        #[cfg(feature = "rp235xb")]
        const TS_CHAN: u8 = 8;

        match &self.0 {
            // this requires adc pins to be sequential and matching the adc channels,
            // which is the case for rp2040/rp235xy
            Source::Pin(p) => p._pin() - CH_OFFSET,
            Source::TempSensor(_) => TS_CHAN,
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
            false => Ok(r.result().read().result()),
        }
    }
}

impl<'d> Adc<'d, Async> {
    /// Create ADC driver in async mode.
    pub fn new(
        _inner: Peri<'d, ADC>,
        _irq: impl Binding<interrupt::typelevel::ADC_IRQ_FIFO, InterruptHandler>,
        _config: Config,
    ) -> Self {
        Self::setup();

        // Setup IRQ
        interrupt::ADC_IRQ_FIFO.unpend();
        unsafe { interrupt::ADC_IRQ_FIFO.enable() };

        Self { phantom: PhantomData }
    }

    fn wait_for_ready() -> impl Future<Output = ()> {
        let r = Self::regs();

        poll_fn(move |cx| {
            WAKER.register(cx.waker());

            r.inte().write(|w| w.set_fifo(true));
            compiler_fence(Ordering::SeqCst);

            if r.cs().read().ready() {
                return Poll::Ready(());
            }
            Poll::Pending
        })
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
            false => Ok(r.result().read().result()),
        }
    }

    // Note for refactoring: we don't require the actual Channels here, just the channel numbers.
    // The public api is responsible for asserting ownership of the actual Channels.
    async fn read_many_inner<W: dma::Word>(
        &mut self,
        channels: impl Iterator<Item = u8>,
        buf: &mut [W],
        fcs_err: bool,
        div: u16,
        dma: Peri<'_, impl dma::Channel>,
    ) -> Result<(), Error> {
        #[cfg(feature = "rp2040")]
        let mut rrobin = 0_u8;
        #[cfg(feature = "_rp235x")]
        let mut rrobin = 0_u16;
        for c in channels {
            rrobin |= 1 << c;
        }
        let first_ch = rrobin.trailing_zeros() as u8;
        if rrobin.count_ones() == 1 {
            rrobin = 0;
        }

        let r = Self::regs();
        // clear previous errors and set channel
        r.cs().modify(|w| {
            w.set_ainsel(first_ch);
            w.set_rrobin(rrobin);
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

        let dma = unsafe { dma::read(dma, r.fifo().as_ptr() as *const W, buf as *mut [W], TreqSel::ADC) };
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

    /// Sample multiple values from multiple channels using DMA.
    /// Samples are stored in an interleaved fashion inside the buffer.
    /// `div` is the integer part of the clock divider and can be calculated with `floor(48MHz / sample_rate * num_channels - 1)`
    /// Any `div` value of less than 96 will have the same effect as setting it to 0
    #[inline]
    pub async fn read_many_multichannel<S: AdcSample>(
        &mut self,
        ch: &mut [Channel<'_>],
        buf: &mut [S],
        div: u16,
        dma: Peri<'_, impl dma::Channel>,
    ) -> Result<(), Error> {
        self.read_many_inner(ch.iter().map(|c| c.channel()), buf, false, div, dma)
            .await
    }

    /// Sample multiple values from multiple channels using DMA, with errors inlined in samples.
    /// Samples are stored in an interleaved fashion inside the buffer.
    /// `div` is the integer part of the clock divider and can be calculated with `floor(48MHz / sample_rate * num_channels - 1)`
    /// Any `div` value of less than 96 will have the same effect as setting it to 0
    #[inline]
    pub async fn read_many_multichannel_raw(
        &mut self,
        ch: &mut [Channel<'_>],
        buf: &mut [Sample],
        div: u16,
        dma: Peri<'_, impl dma::Channel>,
    ) {
        // errors are reported in individual samples
        let _ = self
            .read_many_inner(
                ch.iter().map(|c| c.channel()),
                unsafe { mem::transmute::<_, &mut [u16]>(buf) },
                true,
                div,
                dma,
            )
            .await;
    }

    /// Sample multiple values from a channel using DMA.
    /// `div` is the integer part of the clock divider and can be calculated with `floor(48MHz / sample_rate - 1)`
    /// Any `div` value of less than 96 will have the same effect as setting it to 0
    #[inline]
    pub async fn read_many<S: AdcSample>(
        &mut self,
        ch: &mut Channel<'_>,
        buf: &mut [S],
        div: u16,
        dma: Peri<'_, impl dma::Channel>,
    ) -> Result<(), Error> {
        self.read_many_inner([ch.channel()].into_iter(), buf, false, div, dma)
            .await
    }

    /// Sample multiple values from a channel using DMA, with errors inlined in samples.
    /// `div` is the integer part of the clock divider and can be calculated with `floor(48MHz / sample_rate - 1)`
    /// Any `div` value of less than 96 will have the same effect as setting it to 0
    #[inline]
    pub async fn read_many_raw(
        &mut self,
        ch: &mut Channel<'_>,
        buf: &mut [Sample],
        div: u16,
        dma: Peri<'_, impl dma::Channel>,
    ) {
        // errors are reported in individual samples
        let _ = self
            .read_many_inner(
                [ch.channel()].into_iter(),
                unsafe { mem::transmute::<_, &mut [u16]>(buf) },
                true,
                div,
                dma,
            )
            .await;
    }
}

impl<'d> Adc<'d, Blocking> {
    /// Create ADC driver in blocking mode.
    pub fn new_blocking(_inner: Peri<'d, ADC>, _config: Config) -> Self {
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

trait SealedAdcSample: crate::dma::Word {}
trait SealedAdcChannel {}

/// ADC sample.
#[allow(private_bounds)]
pub trait AdcSample: SealedAdcSample {}

impl SealedAdcSample for u16 {}
impl AdcSample for u16 {}

impl SealedAdcSample for u8 {}
impl AdcSample for u8 {}

/// ADC channel.
#[allow(private_bounds)]
pub trait AdcChannel: SealedAdcChannel {}
/// ADC pin.
pub trait AdcPin: AdcChannel + gpio::Pin {}

macro_rules! impl_pin {
    ($pin:ident, $channel:expr) => {
        impl SealedAdcChannel for peripherals::$pin {}
        impl AdcChannel for peripherals::$pin {}
        impl AdcPin for peripherals::$pin {}
    };
}

#[cfg(any(feature = "rp235xa", feature = "rp2040"))]
impl_pin!(PIN_26, 0);
#[cfg(any(feature = "rp235xa", feature = "rp2040"))]
impl_pin!(PIN_27, 1);
#[cfg(any(feature = "rp235xa", feature = "rp2040"))]
impl_pin!(PIN_28, 2);
#[cfg(any(feature = "rp235xa", feature = "rp2040"))]
impl_pin!(PIN_29, 3);

#[cfg(feature = "rp235xb")]
impl_pin!(PIN_40, 0);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_41, 1);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_42, 2);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_43, 3);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_44, 4);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_45, 5);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_46, 6);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_47, 7);

impl SealedAdcChannel for peripherals::ADC_TEMP_SENSOR {}
impl AdcChannel for peripherals::ADC_TEMP_SENSOR {}
