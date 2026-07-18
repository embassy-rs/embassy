//! Analog to Digital Converter (ADC)

#![macro_use]

use core::future::poll_fn;
use core::hint::unreachable_unchecked;
use core::marker::PhantomData;
use core::num::NonZeroU16;
use core::task::Poll;

use embassy_hal_internal::PeripheralType;
use embassy_sync::waitqueue::AtomicWaker;

use crate::interrupt::{Interrupt, InterruptExt};
use crate::mode::{Async, Blocking, Mode};
use crate::pac::adc::{Adc as Regs, regs, vals};
use crate::{Peri, interrupt};

/// Maximum length allowed for [`Adc::irq_read_sequence`].
pub const MAX_SEQUENCE_LEN: usize = ADC_MEMCTL as usize;

/// Interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let r = T::info().regs;
        let state = T::state();

        let mis = r.cpu_int(0).mis().read().0;

        // Check if any MEMRES bits were set. irq reads will enable the IRQ for the last channel in use.
        if mis >> 8 != 0 {
            // Clear the MEMRES interrupt bits.
            r.cpu_int(0).iclr().write_value(regs::CpuInt(mis & 0xFFFF_FF00));
            state.waker.wake();
        }
    }
}

/// Sample clock source for ADC.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SampleClock {
    // TODO: ULPCLK after clock config
    /// Source ADC clock from SYSOSC.
    Sysosc,
    // TODO: HFCLK after clock config (if available)
}

/// Conversion resolution of the ADC results.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Resolution {
    /// 12-bits resolution
    Bits12,

    /// 10-bits resolution
    Bits10,

    /// 8-bits resolution
    Bits8,
}

impl Resolution {
    /// Number of bits of the resolution.
    #[inline]
    pub const fn bits(&self) -> u8 {
        match self {
            Resolution::Bits12 => 12,
            Resolution::Bits10 => 10,
            Resolution::Bits8 => 8,
        }
    }

    /// Get the maximum reading value for this resolution.
    ///
    /// This is `2**n - 1`.
    #[inline]
    pub const fn max_count(&self) -> u32 {
        (1 << self.bits()) - 1
    }
}

/// Hardware sample time comparator.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SampleTimeComparator {
    /// Use simple time comparator 0.
    Scomp0,

    /// Use sample time comparator 1.
    Scomp1,
}

/// Reference voltage (Vref) selection for the ADC channels.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Vrsel {
    /// VDDA reference
    VddaVssa = 0,

    /// External reference from pin
    ExtrefVrefm = 1,

    /// Internal reference
    IntrefVssa = 2,

    /// VDDA and VREFM connected to VREF+ and VREF- of ADC
    #[cfg(adc_neg_vref)]
    VddaVrefm = 3,

    /// INTREF and VREFM connected to VREF+ and VREF- of ADC
    #[cfg(adc_neg_vref)]
    IntrefVrefm = 4,
}

/// Sample conversion parameters.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Conversion {
    // TODO: Bitpack a regs::Memctl for smaller size?
    /// Voltage reference selection.
    pub vrsel: Vrsel,

    /// Sample time period.
    pub stime: SampleTimeComparator,
    // TODO: AVG, BCS, TRIG, WINCOMP
}

impl Default for Conversion {
    #[inline]
    fn default() -> Self {
        Self {
            vrsel: Vrsel::VddaVssa,
            stime: SampleTimeComparator::Scomp0,
        }
    }
}

/// ADC configuration.
#[derive(Copy, Clone)]
#[non_exhaustive]
pub struct Config {
    /// Resolution of the ADC conversion. The number of bits used to represent an ADC measurement.
    pub resolution: Resolution,

    /// Sample clock source.
    pub sample_clk: SampleClock,

    /// Length of the sample period 0 in ADC sample clock cycles.
    ///
    /// This is used when [`SampleTimeComparator::Scomp0`] is selected when sampling.
    pub sample_period_0: NonZeroU16,

    /// Length of the sample period 1 in ADC sample clock cycles.
    ///
    /// This is used when [`SampleTimeComparator::Scomp1`] is selected when sampling.
    pub sample_period_1: NonZeroU16,
}

impl Config {
    /// Maximum number of sample clocks that may be performed when sampling.
    pub const MAX_SAMPLE_PERIOD: NonZeroU16 = NonZeroU16::new((1 << 9) - 1).unwrap();
}

impl Default for Config {
    fn default() -> Self {
        Self {
            resolution: Resolution::Bits12,
            sample_clk: SampleClock::Sysosc,
            // TODO: What should these be by default?
            sample_period_0: NonZeroU16::new(50).unwrap(),
            sample_period_1: NonZeroU16::new(50).unwrap(),
        }
    }
}

/// Analog to Digital driver.
pub struct Adc<'d, T: Instance, M: Mode> {
    #[allow(unused)]
    adc: crate::Peri<'d, T>,
    _mode: PhantomData<M>,
}

impl<'d, T: Instance, M: Mode> Adc<'d, T, M> {
    pub fn new_blocking(peri: Peri<'d, T>, config: Config) -> Adc<'d, T, Blocking> {
        Self::setup(config);
        Adc {
            adc: peri,
            _mode: PhantomData,
        }
    }

    /// Read an ADC pin.
    pub fn blocking_read<'a>(&mut self, channel: impl BorrowedChannel<'a, T>, conversion: Conversion) -> u16
    where
        'd: 'a,
    {
        let r = T::info().regs;
        let channel = channel.reborrow_adc();

        // Wait until ADC is not converting to start.
        //
        // This is needed a future which started sampling could have been dropped half way through.
        while r.ctl0().read().enc() {}

        Self::setup_sequence([(channel.get_hw_channel(), conversion)].into_iter());

        r.ctl0().modify(|w| {
            w.set_enc(true);
        });

        r.ctl1().modify(|w| {
            w.set_sc(vals::Sc::START);
        });

        // Wait for conversion
        while r.ctl0().read().enc() {}
        r.memres(0).read().data()
    }

    pub fn resolution(&self) -> Resolution {
        let r = T::info().regs;
        let ctl2 = r.ctl2().read();
        from_res(ctl2.res())
    }

    pub fn set_resolution(&mut self, resolution: Resolution) {
        let r = T::info().regs;

        r.ctl2().modify(|w| {
            w.set_res(to_res(resolution));
        });
    }

    pub fn set_scomp0(&mut self, period: NonZeroU16) {
        assert!(period <= Config::MAX_SAMPLE_PERIOD);
        let r = T::info().regs;

        r.scomp0().write(|w| {
            w.set_val(period.get());
        });
    }

    pub fn scomp0(&self) -> u16 {
        let r = T::info().regs;
        r.scomp0().read().val()
    }

    pub fn set_scomp1(&mut self, period: NonZeroU16) {
        assert!(period <= Config::MAX_SAMPLE_PERIOD);
        let r = T::info().regs;

        r.scomp1().write(|w| {
            w.set_val(period.get());
        });
    }

    pub fn scomp1(&self) -> u16 {
        let r = T::info().regs;
        r.scomp0().read().val()
    }
}

impl<'d, T: Instance> Adc<'d, T, Async> {
    pub fn new_async(
        peri: Peri<'d, T>,
        _irq: impl crate::interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        config: Config,
    ) -> Self {
        Self::setup(config);
        unsafe { T::info().interrupt.enable() };
        Self {
            adc: peri,
            _mode: PhantomData,
        }
    }

    /// Read an ADC pin asynchronously using the irq handler.
    pub async fn irq_read<'a>(&mut self, channel: impl BorrowedChannel<'a, T>, conversion: Conversion) -> u16
    where
        T: 'a,
    {
        let r = T::info().regs;
        let channel = channel.reborrow_adc();

        // Wait until ADC is not converting to start.
        //
        // This is needed a future which started sampling could have been dropped half way through.
        Self::wait_for_conversion().await;
        Self::setup_sequence([(channel.get_hw_channel(), conversion)].into_iter());

        // Write is used to zero the other MEMRES interrupt bits.
        r.cpu_int(0).imask().write(|w| {
            w.set_memresifg(0, true);
        });

        r.ctl0().modify(|w| {
            w.set_enc(true);
        });

        r.ctl1().modify(|w| {
            w.set_sc(vals::Sc::START);
        });

        Self::wait_for_conversion().await;
        r.memres(0).read().data()
    }

    /// Read one or multiple ADC regular channels using the irq handler.
    ///
    /// `sequence` iterator and `readings` must have the same length.
    pub async fn irq_read_sequence<'a>(
        &mut self,
        sequence: impl ExactSizeIterator<Item = (BorrowedAdcChannel<'a, T>, Conversion)>,
        readings: &mut [u16],
    ) where
        T: 'a,
    {
        assert!(sequence.len() != 0, "Read sequence cannot be empty");
        assert!(
            sequence.len() == readings.len(),
            "Sequence length must be equal to readings length"
        );
        assert!(
            sequence.len() <= MAX_SEQUENCE_LEN,
            "Asynchronous read sequence cannot be more than {} in length",
            MAX_SEQUENCE_LEN
        );

        let sequence_len = sequence.len();
        let r = T::info().regs;

        // Wait until ADC is not converting to start.
        //
        // This is needed a future which started sampling could have been dropped half way through.
        Self::wait_for_conversion().await;
        Self::setup_sequence(sequence.map(|(ch, conv)| (ch.get_hw_channel(), conv)));

        // Only wake up when the last bit is set.
        //
        // Write is used to zero the other MEMRES interrupt bits.
        r.cpu_int(0).imask().write(|w| {
            w.set_memresifg(sequence_len - 1, true);
        });

        r.ctl0().modify(|w| {
            w.set_enc(true);
        });

        r.ctl1().modify(|w| {
            w.set_sc(vals::Sc::START);
        });

        Self::wait_for_conversion().await;

        for (i, reading) in readings.iter_mut().enumerate() {
            *reading = r.memres(i).read().data();
        }
    }

    // TODO: DMA driven ADC
}

/// Peripheral instance trait.
#[allow(private_bounds)]
pub trait Instance: PeripheralType + SealedInstance {
    type Interrupt: crate::interrupt::typelevel::Interrupt;
}

/// A type-erased borrowed channel for the given ADC instance.
///
/// The borrowed channel cannot consume the channel source because it might need to run drop code.
pub struct BorrowedAdcChannel<'a, T> {
    pub(crate) channel: u8,
    pub(crate) _marker: PhantomData<&'a mut T>,
}

impl<T> BorrowedAdcChannel<'_, T> {
    pub fn get_hw_channel(&self) -> u8 {
        self.channel
    }
}

impl<T: Instance> AdcChannel<T> for BorrowedAdcChannel<'_, T> {}
impl<T: Instance> SealedAdcChannel<T> for BorrowedAdcChannel<'_, T> {
    fn channel(&self) -> u8 {
        self.channel
    }
}

#[allow(private_bounds)]
pub trait BorrowedChannel<'a, T>: SealedBorrowedChannel<'a, T> {}
impl<'a, T, C: SealedBorrowedChannel<'a, T>> BorrowedChannel<'a, T> for C {}

impl<'a, T, C: AdcChannel<T>> SealedBorrowedChannel<'a, T> for &'a mut C {
    #[inline]
    fn reborrow_adc(self) -> BorrowedAdcChannel<'a, T> {
        self.reborrow_adc()
    }
}

impl<'a, T> SealedBorrowedChannel<'a, T> for BorrowedAdcChannel<'a, T> {
    #[inline]
    fn reborrow_adc(self) -> BorrowedAdcChannel<'a, T> {
        self
    }
}

/// ADC channel.
#[allow(private_bounds)]
pub trait AdcChannel<T>: SealedAdcChannel<T> + Sized {
    #[allow(unused_mut)]
    fn reborrow_adc<'a>(&'a mut self) -> BorrowedAdcChannel<'a, T> {
        self.setup();

        BorrowedAdcChannel {
            channel: self.channel(),
            _marker: PhantomData,
        }
    }
}

// Impl details

// Constants from the metapac crate
const ADC_VRSEL: u8 = crate::_generated::ADC_VRSEL;
const ADC_MEMCTL: u8 = crate::_generated::ADC_MEMCTL;

impl<'d, T: Instance, M: Mode> Adc<'d, T, M> {
    fn setup(config: Config) {
        assert!(config.sample_period_0 <= Config::MAX_SAMPLE_PERIOD);
        assert!(config.sample_period_1 <= Config::MAX_SAMPLE_PERIOD);

        let r = T::info().regs;

        r.gprcm(0).rstctl().write(|w| {
            w.set_resetstkyclr(true);
            w.set_resetassert(true);
            w.set_key(vals::ResetKey::KEY);
        });

        r.gprcm(0).pwren().modify(|reg| {
            reg.set_enable(true);
            reg.set_key(vals::PwrenKey::KEY);
        });

        // Wait for power up
        cortex_m::asm::delay(16);

        r.gprcm(0).clkcfg().write(|w| {
            w.set_key(vals::ClkcfgKey::KEY);
            w.set_sampclk(vals::Sampclk::SYSOSC);
        });

        // FIXME: Consider clock config
        // This code assumes the 24/32 MHz boot frequency
        r.ctl0().write(|w| {
            w.set_enc(false);
            // TODO: power down config
            w.set_pwrdn(vals::Pwrdn::MANUAL);
            w.set_sclkdiv(vals::Sclkdiv::DIV_BY_4);
        });

        r.clkfreq().write(|w| {
            w.set_frange(vals::Frange::RANGE24TO32);
        });

        r.ctl1().write(|w| {
            w.set_trigsrc(vals::Trigsrc::SOFTWARE);
            w.set_sc(vals::Sc::STOP);
            w.set_conseq(vals::Conseq::SEQUENCE);
            w.set_sampmode(vals::Sampmode::AUTO);
            w.set_avgn(vals::Avgn::DISABLE);
            w.set_avgd(0);
        });

        r.ctl2().write(|w| {
            // Binary unsigned
            w.set_df(false);
            w.set_res(to_res(config.resolution));
            w.set_rstsampcapen(false);
            w.set_dmaen(false);
            w.set_fifoen(false);
            w.set_sampcnt(vals::Sampcnt::MIN);
            w.set_startadd(0);
            w.set_endadd(0);
        });

        r.scomp0().write(|w| {
            w.set_val(config.sample_period_0.get());
        });

        r.scomp1().write(|w| {
            w.set_val(config.sample_period_1.get());
        });
    }

    // (channel, conversion)
    fn setup_sequence(sequence: impl ExactSizeIterator<Item = (u8, Conversion)>) {
        let r = T::info().regs;
        let len = sequence.len();

        for (i, (ch, conversion)) in sequence.enumerate() {
            assert!(
                (conversion.vrsel as u8) < ADC_VRSEL,
                "Reference voltage selection out of bounds"
            );

            r.memctl(i).write(|w| {
                w.set_chansel(ch);
                // TODO: Conversion function to not be repr dependent
                w.set_vrsel(vals::Vrsel::from_bits(conversion.vrsel as u8));
                w.set_stime(convert_stime(conversion.stime));
                // TODO: More parameters
                w.set_avgen(false);
                w.set_bcsen(false);
                w.set_trig(vals::Trig::AUTO_NEXT);
                w.set_wincomp(false);
            });
        }

        r.ctl2().modify(|w| {
            w.set_startadd(0);
            w.set_endadd((len - 1) as u8);
        });
    }

    /// Return `impl Future` to reduce async state machine size.
    #[inline]
    fn wait_for_conversion() -> impl Future<Output = ()> {
        let r = T::info().regs;
        let state = T::state();

        poll_fn(move |cx| {
            state.waker.register(cx.waker());

            if !r.ctl0().read().enc() {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
    }
}

/// Peripheral state.
pub(crate) struct State {
    waker: AtomicWaker,
}

impl State {
    pub const fn new() -> Self {
        Self {
            waker: AtomicWaker::new(),
        }
    }
}

/// Peripheral information.
pub(crate) struct Info {
    pub(crate) regs: Regs,
    pub(crate) interrupt: Interrupt,
}

/// Peripheral instance trait.
pub(crate) trait SealedInstance {
    fn info() -> &'static Info;
    fn state() -> &'static State;
}

pub(crate) trait SealedAdcChannel<T> {
    fn setup(&self) {}

    fn channel(&self) -> u8;
}

trait SealedBorrowedChannel<'a, T> {
    fn reborrow_adc(self) -> BorrowedAdcChannel<'a, T>;
}

const fn to_res(resolution: Resolution) -> vals::Res {
    match resolution {
        Resolution::Bits12 => vals::Res::BIT_12,
        Resolution::Bits10 => vals::Res::BIT_10,
        Resolution::Bits8 => vals::Res::BIT_8,
    }
}

const fn from_res(res: vals::Res) -> Resolution {
    match res {
        vals::Res::BIT_12 => Resolution::Bits12,
        vals::Res::BIT_10 => Resolution::Bits10,
        vals::Res::BIT_8 => Resolution::Bits8,
        // SAFETY: The HAL will never program an invalid valid.
        vals::Res::_RESERVED_3 => unsafe { unreachable_unchecked() },
    }
}

const fn convert_stime(stime: SampleTimeComparator) -> vals::Stime {
    match stime {
        SampleTimeComparator::Scomp0 => vals::Stime::SEL_SCOMP0,
        SampleTimeComparator::Scomp1 => vals::Stime::SEL_SCOMP1,
    }
}

macro_rules! impl_adc_instance {
    ($instance: ident) => {
        impl crate::adc::SealedInstance for crate::peripherals::$instance {
            fn info() -> &'static crate::adc::Info {
                use crate::adc::Info;
                use crate::interrupt::typelevel::Interrupt;

                static INFO: Info = Info {
                    regs: crate::pac::$instance,
                    interrupt: crate::interrupt::typelevel::$instance::IRQ,
                };
                &INFO
            }

            fn state() -> &'static crate::adc::State {
                use crate::adc::State;

                static STATE: State = State::new();
                &STATE
            }
        }

        impl crate::adc::Instance for crate::peripherals::$instance {
            type Interrupt = crate::interrupt::typelevel::$instance;
        }
    };
}

macro_rules! impl_adc_pin {
    ($inst: ident, $pin: ident, $ch: expr) => {
        impl crate::adc::AdcChannel<peripherals::$inst> for crate::Peri<'_, crate::peripherals::$pin> {}
        impl crate::adc::SealedAdcChannel<peripherals::$inst> for crate::Peri<'_, crate::peripherals::$pin> {
            fn setup(&self) {
                <crate::peripherals::$pin as crate::gpio::SealedPin>::set_as_analog(self);
            }

            fn channel(&self) -> u8 {
                $ch
            }
        }
    };
}
