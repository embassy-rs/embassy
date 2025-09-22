#![macro_use]

use core::future::poll_fn;
use core::marker::PhantomData;
use core::task::Poll;

use embassy_hal_internal::{impl_peripheral, PeripheralType};
use embassy_sync::waitqueue::AtomicWaker;

use crate::interrupt::{Interrupt, InterruptExt};
use crate::mode::{Async, Blocking, Mode};
use crate::pac::adc::{vals, Adc as Regs};
use crate::{interrupt, Peri};

/// Interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        // Mis is cleared upon reading iidx
        let iidx = T::info().regs.cpu_int(0).iidx().read().stat();
        // TODO: Running in sequence mode, we get an interrupt per finished result. It would be
        // nice to wake up only after all results are finished.
        if vals::CpuIntIidxStat::MEMRESIFG0 <= iidx && iidx <= vals::CpuIntIidxStat::MEMRESIFG23 {
            T::state().waker.wake();
        }
    }
}

// Constants from the metapac crate
const ADC_VRSEL: u8 = crate::_generated::ADC_VRSEL;
const ADC_MEMCTL: u8 = crate::_generated::ADC_MEMCTL;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// Conversion resolution of the ADC results.
pub enum Resolution {
    /// 12-bits resolution
    BIT12,

    /// 10-bits resolution
    BIT10,

    /// 8-bits resolution
    BIT8,
}

impl Resolution {
    /// Number of bits of the resolution.
    pub fn bits(&self) -> u8 {
        match self {
            Resolution::BIT12 => 12,
            Resolution::BIT10 => 10,
            Resolution::BIT8 => 8,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// Reference voltage (Vref) selection for the ADC channels.
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

/// ADC configuration.
#[derive(Copy, Clone)]
#[non_exhaustive]
pub struct Config {
    /// Resolution of the ADC conversion. The number of bits used to represent an ADC measurement.
    pub resolution: Resolution,
    /// ADC voltage reference selection.
    ///
    /// This value is used when reading a single channel. When reading a sequence
    /// the vr_select is provided per channel.
    pub vr_select: Vrsel,
    /// The sample time in number of ADC sample clock cycles.
    pub sample_time: u16,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            resolution: Resolution::BIT12,
            vr_select: Vrsel::VddaVssa,
            sample_time: 50,
        }
    }
}

/// ADC (Analog to Digial Converter) Driver.
pub struct Adc<'d, T: Instance, M: Mode> {
    #[allow(unused)]
    adc: crate::Peri<'d, T>,
    info: &'static Info,
    state: &'static State,
    config: Config,
    _phantom: PhantomData<M>,
}

impl<'d, T: Instance> Adc<'d, T, Blocking> {
    /// A new blocking ADC driver instance.
    pub fn new_blocking(peri: Peri<'d, T>, config: Config) -> Self {
        let mut this = Self {
            adc: peri,
            info: T::info(),
            state: T::state(),
            config,
            _phantom: PhantomData,
        };
        this.setup();
        this
    }
}

impl<'d, T: Instance> Adc<'d, T, Async> {
    /// A new asynchronous ADC driver instance.
    pub fn new_async(
        peri: Peri<'d, T>,
        config: Config,
        _irqs: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
    ) -> Self {
        let mut this = Self {
            adc: peri,
            info: T::info(),
            state: T::state(),
            config,
            _phantom: PhantomData,
        };
        this.setup();
        unsafe {
            this.info.interrupt.enable();
        }
        this
    }
}

impl<'d, T: Instance, M: Mode> Adc<'d, T, M> {
    const SINGLE_CHANNEL: u8 = 0;

    fn setup(&mut self) {
        let config = &self.config;
        assert!(
            (config.vr_select as u8) < ADC_VRSEL,
            "Reference voltage selection out of bounds"
        );
        // Reset adc
        self.info.regs.gprcm(0).rstctl().write(|reg| {
            reg.set_resetstkyclr(true);
            reg.set_resetassert(true);
            reg.set_key(vals::ResetKey::KEY);
        });

        // Power up adc
        self.info.regs.gprcm(0).pwren().modify(|reg| {
            reg.set_enable(true);
            reg.set_key(vals::PwrenKey::KEY);
        });

        // Wait for cycles similar to TI power setup
        cortex_m::asm::delay(16);

        // Set clock config
        self.info.regs.gprcm(0).clkcfg().modify(|reg| {
            reg.set_key(vals::ClkcfgKey::KEY);
            reg.set_sampclk(vals::Sampclk::SYSOSC);
        });
        self.info.regs.ctl0().modify(|reg| {
            reg.set_sclkdiv(vals::Sclkdiv::DIV_BY_4);
        });
        self.info.regs.clkfreq().modify(|reg| {
            reg.set_frange(vals::Frange::RANGE24TO32);
        });

        // Init single conversion with software trigger and auto sampling
        //
        // We use sequence to support sequence operation in the future, but only set up a single
        // channel
        self.info.regs.ctl1().modify(|reg| {
            reg.set_conseq(vals::Conseq::SEQUENCE);
            reg.set_sampmode(vals::Sampmode::AUTO);
            reg.set_trigsrc(vals::Trigsrc::SOFTWARE);
        });
        let res = match config.resolution {
            Resolution::BIT12 => vals::Res::BIT_12,
            Resolution::BIT10 => vals::Res::BIT_10,
            Resolution::BIT8 => vals::Res::BIT_8,
        };
        self.info.regs.ctl2().modify(|reg| {
            // Startadd detemines the channel used in single mode.
            reg.set_startadd(Self::SINGLE_CHANNEL);
            reg.set_endadd(Self::SINGLE_CHANNEL);
            reg.set_res(res);
            reg.set_df(false);
        });

        // Set the sample time used by all channels for now
        self.info.regs.scomp0().modify(|reg| {
            reg.set_val(config.sample_time);
        });
    }

    fn setup_blocking_channel(&mut self, channel: &Peri<'d, impl AdcChannel<T>>) {
        channel.setup();

        // CTL0.ENC must be 0 to write the MEMCTL register
        while self.info.regs.ctl0().read().enc() {
            // Wait until the ADC is not in conversion mode
        }

        // Conversion mem config
        let vrsel = vals::Vrsel::from_bits(self.config.vr_select as u8);
        self.info.regs.memctl(Self::SINGLE_CHANNEL as usize).modify(|reg| {
            reg.set_chansel(channel.channel());
            reg.set_vrsel(vrsel);
            reg.set_stime(vals::Stime::SEL_SCOMP0);
            reg.set_avgen(false);
            reg.set_bcsen(false);
            reg.set_trig(vals::Trig::AUTO_NEXT);
            reg.set_wincomp(false);
        });
        self.info.regs.ctl2().modify(|reg| {
            // Set end address to the number of used channels
            reg.set_endadd(Self::SINGLE_CHANNEL);
        });
    }

    fn enable_conversion(&mut self) {
        // Enable conversion
        self.info.regs.ctl0().modify(|reg| {
            reg.set_enc(true);
        });
    }

    fn start_conversion(&mut self) {
        // Start conversion
        self.info.regs.ctl1().modify(|reg| {
            reg.set_sc(vals::Sc::START);
        });
    }

    fn conversion_result(&mut self, channel_id: usize) -> u16 {
        // Read result
        self.info.regs.memres(channel_id).read().data()
    }

    /// Read one ADC channel in blocking mode using the config provided at initialization.
    pub fn blocking_read(&mut self, channel: &Peri<'d, impl AdcChannel<T>>) -> u16 {
        self.setup_blocking_channel(channel);
        self.enable_conversion();
        self.start_conversion();

        while self.info.regs.ctl0().read().enc() {}

        self.conversion_result(Self::SINGLE_CHANNEL as usize)
    }
}

impl<'d, T: Instance> Adc<'d, T, Async> {
    /// Maximum length allowed for [`Self::read_sequence`].
    pub const MAX_SEQUENCE_LEN: usize = ADC_MEMCTL as usize;

    async fn wait_for_conversion(&self) {
        let info = self.info;
        let state = self.state;
        poll_fn(move |cx| {
            state.waker.register(cx.waker());

            if !info.regs.ctl0().read().enc() {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await;
    }

    fn setup_async_channel(&self, id: usize, channel: &Peri<'d, impl AdcChannel<T>>, vrsel: Vrsel) {
        let vrsel = vals::Vrsel::from_bits(vrsel as u8);
        // Conversion mem config
        self.info.regs.memctl(id).modify(|reg| {
            reg.set_chansel(channel.channel());
            reg.set_vrsel(vrsel);
            reg.set_stime(vals::Stime::SEL_SCOMP0);
            reg.set_avgen(false);
            reg.set_bcsen(false);
            reg.set_trig(vals::Trig::AUTO_NEXT);
            reg.set_wincomp(false);
        });

        // Clear interrupt status
        self.info.regs.cpu_int(0).iclr().write(|reg| {
            reg.set_memresifg(id, true);
        });
        // Enable interrupt
        self.info.regs.cpu_int(0).imask().modify(|reg| {
            reg.set_memresifg(id, true);
        });
    }

    /// Read one ADC channel asynchronously using the config provided at initialization.
    pub async fn read_channel(&mut self, channel: &Peri<'d, impl AdcChannel<T>>) -> u16 {
        channel.setup();

        // CTL0.ENC must be 0 to write the MEMCTL register
        self.wait_for_conversion().await;

        self.info.regs.ctl2().modify(|reg| {
            // Set end address to the number of used channels
            reg.set_endadd(Self::SINGLE_CHANNEL);
        });

        self.setup_async_channel(Self::SINGLE_CHANNEL as usize, channel, self.config.vr_select);

        self.enable_conversion();
        self.start_conversion();
        self.wait_for_conversion().await;

        self.conversion_result(Self::SINGLE_CHANNEL as usize)
    }

    /// Read one or multiple ADC channels using the Vrsel provided per channel.
    ///
    /// `sequence` iterator and `readings` must have the same length.
    ///
    /// Example
    /// ```rust,ignore
    /// use embassy_mspm0::adc::{Adc, AdcChannel, Vrsel};
    ///
    /// let mut adc = Adc::new_async(p.ADC0, adc_config, Irqs);
    /// let sequence = [(&p.PA22.into(), Vrsel::VddaVssa), (&p.PA20.into(), Vrsel::VddaVssa)];
    /// let mut readings = [0u16; 2];
    ///
    /// adc.read_sequence(sequence.into_iter(), &mut readings).await;
    /// defmt::info!("Measurements: {}", readings);
    /// ```
    pub async fn read_sequence<'a>(
        &mut self,
        sequence: impl ExactSizeIterator<Item = (&'a Peri<'d, AnyAdcChannel<T>>, Vrsel)>,
        readings: &mut [u16],
    ) where
        'd: 'a,
    {
        assert!(sequence.len() != 0, "Asynchronous read sequence cannot be empty");
        assert!(
            sequence.len() == readings.len(),
            "Sequence length must be equal to readings length"
        );
        assert!(
            sequence.len() <= Self::MAX_SEQUENCE_LEN,
            "Asynchronous read sequence cannot be more than {} in length",
            Self::MAX_SEQUENCE_LEN
        );

        // CTL0.ENC must be 0 to write the MEMCTL register
        self.wait_for_conversion().await;

        self.info.regs.ctl2().modify(|reg| {
            // Set end address to the number of used channels
            reg.set_endadd((sequence.len() - 1) as u8);
        });

        for (i, (channel, vrsel)) in sequence.enumerate() {
            self.setup_async_channel(i, channel, vrsel);
        }
        self.enable_conversion();
        self.start_conversion();
        self.wait_for_conversion().await;

        for (i, r) in readings.iter_mut().enumerate() {
            *r = self.conversion_result(i);
        }
    }
}

/// Peripheral instance trait.
#[allow(private_bounds)]
pub trait Instance: PeripheralType + SealedInstance {
    type Interrupt: crate::interrupt::typelevel::Interrupt;
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

/// A type-erased channel for a given ADC instance.
///
/// This is useful in scenarios where you need the ADC channels to have the same type, such as
/// storing them in an array.
pub struct AnyAdcChannel<T> {
    pub(crate) channel: u8,
    pub(crate) _phantom: PhantomData<T>,
}

impl_peripheral!(AnyAdcChannel<T: Instance>);
impl<T: Instance> AdcChannel<T> for AnyAdcChannel<T> {}
impl<T: Instance> SealedAdcChannel<T> for AnyAdcChannel<T> {
    fn channel(&self) -> u8 {
        self.channel
    }
}

impl<T> AnyAdcChannel<T> {
    #[allow(unused)]
    pub(crate) fn get_hw_channel(&self) -> u8 {
        self.channel
    }
}

/// ADC channel.
#[allow(private_bounds)]
pub trait AdcChannel<T>: PeripheralType + Into<AnyAdcChannel<T>> + SealedAdcChannel<T> + Sized {}

pub(crate) trait SealedAdcChannel<T> {
    fn setup(&self) {}

    fn channel(&self) -> u8;
}

macro_rules! impl_adc_pin {
    ($inst: ident, $pin: ident, $ch: expr) => {
        impl crate::adc::AdcChannel<peripherals::$inst> for crate::peripherals::$pin {}
        impl crate::adc::SealedAdcChannel<peripherals::$inst> for crate::peripherals::$pin {
            fn setup(&self) {
                crate::gpio::SealedPin::set_as_analog(self);
            }

            fn channel(&self) -> u8 {
                $ch
            }
        }

        impl From<crate::peripherals::$pin> for crate::adc::AnyAdcChannel<peripherals::$inst> {
            fn from(val: crate::peripherals::$pin) -> Self {
                crate::adc::SealedAdcChannel::<peripherals::$inst>::setup(&val);

                Self {
                    channel: crate::adc::SealedAdcChannel::<peripherals::$inst>::channel(&val),
                    _phantom: core::marker::PhantomData,
                }
            }
        }
    };
}
