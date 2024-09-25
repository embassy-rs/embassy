pub use crate::pac::adc::vals::Adc4Res as Resolution;
pub use crate::pac::adc::vals::Adc4SampleTime as SampleTime;
pub use crate::pac::adc::vals::Adc4Presc as Presc;
pub use crate::pac::adc::regs::Adc4Chselrmod0;

#[allow(unused)]
use pac::adc::vals::{Adc4Exten, Adc4OversamplingRatio};

use super::{
    blocking_delay_us, AdcChannel, SealedAdcChannel
};
use crate::time::Hertz;
use crate::{pac, rcc, Peripheral};

const MAX_ADC_CLK_FREQ: Hertz = Hertz::mhz(55);

/// Default VREF voltage used for sample conversion to millivolts.
pub const VREF_DEFAULT_MV: u32 = 3300;
/// VREF voltage used for factory calibration of VREFINTCAL register.
pub const VREF_CALIB_MV: u32 = 3300;

const VREF_CHANNEL: u8 = 0;
const VCORE_CHANNEL: u8 = 12;
const TEMP_CHANNEL: u8 = 13;
const VBAT_CHANNEL: u8 = 14;
const DAC_CHANNEL: u8 = 21;

// NOTE: Vrefint/Temperature/Vbat are not available on all ADCs, this currently cannot be modeled with stm32-data, so these are available from the software on all ADCs
/// Internal voltage reference channel.
pub struct VrefInt;
impl<T: Instance> AdcChannel<T> for VrefInt {}
impl<T: Instance> SealedAdcChannel<T> for VrefInt {
    fn channel(&self) -> u8 {
        VREF_CHANNEL
    }
}

/// Internal temperature channel.
pub struct Temperature;
impl<T: Instance> AdcChannel<T> for Temperature {}
impl<T: Instance> SealedAdcChannel<T> for Temperature {
    fn channel(&self) -> u8 {
        TEMP_CHANNEL
    }
}

/// Internal battery voltage channel.
pub struct Vbat;
impl<T: Instance> AdcChannel<T> for Vbat {}
impl<T: Instance> SealedAdcChannel<T> for Vbat {
    fn channel(&self) -> u8 {
        VBAT_CHANNEL
    }
}

/// Internal DAC channel.
pub struct Dac;
impl<T: Instance> AdcChannel<T> for Dac {}
impl<T: Instance> SealedAdcChannel<T> for Dac {
    fn channel(&self) -> u8 {
        DAC_CHANNEL
    }
}

/// Internal Vcore channel.
pub struct Vcore;
impl<T: Instance> AdcChannel<T> for Vcore {}
impl<T: Instance> SealedAdcChannel<T> for Vcore {
    fn channel(&self) -> u8 {
        VCORE_CHANNEL
    }
}

pub enum DacChannel {
    OUT1,
    OUT2
}

/// Number of samples used for averaging.
pub enum Averaging {
    Disabled,
    Samples2,
    Samples4,
    Samples8,
    Samples16,
    Samples32,
    Samples64,
    Samples128,
    Samples256,
}

pub const fn resolution_to_max_count(res: Resolution) -> u32 {
    match res {
        Resolution::BITS12 => (1 << 12) - 1,
        Resolution::BITS10 => (1 << 10) - 1,
        Resolution::BITS8 => (1 << 8) - 1,
        Resolution::BITS6 => (1 << 6) - 1,
        #[allow(unreachable_patterns)]
        _ => core::unreachable!(),
    }
}

// NOTE (unused): The prescaler enum closely copies the hardware capabilities,
// but high prescaling doesn't make a lot of sense in the current implementation and is ommited.
#[allow(unused)]
enum Prescaler {
    NotDivided,
    DividedBy2,
    DividedBy4,
    DividedBy6,
    DividedBy8,
    DividedBy10,
    DividedBy12,
    DividedBy16,
    DividedBy32,
    DividedBy64,
    DividedBy128,
    DividedBy256,
}

impl Prescaler {
    fn from_ker_ck(frequency: Hertz) -> Self {
        let raw_prescaler = frequency.0 / MAX_ADC_CLK_FREQ.0;
        match raw_prescaler {
            0 => Self::NotDivided,
            1 => Self::DividedBy2,
            2..=3 => Self::DividedBy4,
            4..=5 => Self::DividedBy6,
            6..=7 => Self::DividedBy8,
            8..=9 => Self::DividedBy10,
            10..=11 => Self::DividedBy12,
            _ => unimplemented!(),
        }
    }

    fn divisor(&self) -> u32 {
        match self {
            Prescaler::NotDivided => 1,
            Prescaler::DividedBy2 => 2,
            Prescaler::DividedBy4 => 4,
            Prescaler::DividedBy6 => 6,
            Prescaler::DividedBy8 => 8,
            Prescaler::DividedBy10 => 10,
            Prescaler::DividedBy12 => 12,
            Prescaler::DividedBy16 => 16,
            Prescaler::DividedBy32 => 32,
            Prescaler::DividedBy64 => 64,
            Prescaler::DividedBy128 => 128,
            Prescaler::DividedBy256 => 256,
        }
    }

    fn presc(&self) -> Presc {
        match self {
            Prescaler::NotDivided => Presc::DIV1,
            Prescaler::DividedBy2 => Presc::DIV2,
            Prescaler::DividedBy4 => Presc::DIV4,
            Prescaler::DividedBy6 => Presc::DIV6,
            Prescaler::DividedBy8 => Presc::DIV8,
            Prescaler::DividedBy10 => Presc::DIV10,
            Prescaler::DividedBy12 => Presc::DIV12,
            Prescaler::DividedBy16 => Presc::DIV16,
            Prescaler::DividedBy32 => Presc::DIV32,
            Prescaler::DividedBy64 => Presc::DIV64,
            Prescaler::DividedBy128 => Presc::DIV128,
            Prescaler::DividedBy256 => Presc::DIV256,
        }
    }
}

pub trait SealedInstance {
    #[allow(unused)]
    fn regs() -> crate::pac::adc::Adc4;
}

pub trait Instance: SealedInstance + crate::Peripheral<P = Self> + crate::rcc::RccPeripheral {
    type Interrupt: crate::interrupt::typelevel::Interrupt;
}

pub struct Adc4<'d, T: Instance> {
    #[allow(unused)]
    adc: crate::PeripheralRef<'d, T>,
}

impl<'d, T: Instance> Adc4<'d, T> {
    /// Create a new ADC driver.
    pub fn new(adc: impl Peripheral<P = T> + 'd) -> Self {
        embassy_hal_internal::into_ref!(adc);
        rcc::enable_and_reset::<T>();
        let prescaler = Prescaler::from_ker_ck(T::frequency());

        T::regs().ccr().modify(|w| w.set_presc(prescaler.presc()));

        let frequency = Hertz(T::frequency().0 / prescaler.divisor());
        info!("ADC4 frequency set to {} Hz", frequency.0);

        if frequency > MAX_ADC_CLK_FREQ {
            panic!("Maximal allowed frequency for ADC4 is {} MHz and it varies with different packages, refer to ST docs for more information.", MAX_ADC_CLK_FREQ.0 /  1_000_000 );
        }

        let mut s = Self {
            adc,
        };

        s.power_up();

        s.calibrate();
        blocking_delay_us(1);

        s.enable();
        s.configure();

        s
    }

    fn power_up(&mut self) {
        T::regs().isr().modify(|reg| {
            reg.set_ldordy(true);
        });
        T::regs().cr().modify(|reg| {
            reg.set_advregen(true);
        });
        while !T::regs().isr().read().ldordy() { };

        T::regs().isr().modify(|reg| {
            reg.set_ldordy(true);
        });
    }

    fn calibrate(&mut self) {
        T::regs().cr().modify(|w| w.set_adcal(true));
        while T::regs().cr().read().adcal() {}
        T::regs().isr().modify(|w| w.set_eocal(true));
    }

    fn enable(&mut self) {
        T::regs().isr().write(|w| w.set_adrdy(true));
        T::regs().cr().modify(|w| w.set_aden(true));
        while !T::regs().isr().read().adrdy() {}
        T::regs().isr().write(|w| w.set_adrdy(true));
    }

    fn configure(&mut self) {
        // single conversion mode, software trigger
        T::regs().cfgr1().modify(|w| {
            w.set_cont(false);
            w.set_exten(Adc4Exten::DISABLED);
        });

        // only use one channel at the moment
        T::regs().smpr().modify(|w| {
            for i in 0..24 {
                w.set_smpsel(i, false);
            }
        });
    }

    /// Enable reading the voltage reference internal channel.
    pub fn enable_vrefint(&self) -> VrefInt {
        T::regs().ccr().modify(|reg| {
            reg.set_vrefen(true);
        });

        VrefInt {}
    }

    /// Enable reading the temperature internal channel.
    pub fn enable_temperature(&self) -> Temperature {
        T::regs().ccr().modify(|reg| {
            reg.set_vsensesel(true);
        });

        Temperature {}
    }

    /// Enable reading the vbat internal channel.
    pub fn enable_vbat(&self) -> Vbat {
        T::regs().ccr().modify(|reg| {
            reg.set_vbaten(true);
        });

        Vbat {}
    }

    /// Enable reading the vbat internal channel.
    pub fn enable_vcore(&self) -> Vcore {
        Vcore {}
    }

    /// Enable reading the vbat internal channel.
    pub fn enable_dac_channel(&self, dac: DacChannel) -> Dac {
        let mux;
        match dac {
            DacChannel::OUT1 => {mux = false},
            DacChannel::OUT2 => {mux = true}
        }
        T::regs().or().modify(|w| w.set_chn21sel(mux));
        Dac {}
    }

    /// Set the ADC sample time.
    pub fn set_sample_time(&mut self, sample_time: SampleTime) {
        T::regs().smpr().modify(|w| {
            w.set_smp(0, sample_time);
        });
    }

    /// Get the ADC sample time.
    pub fn sample_time(&self) -> SampleTime {
        T::regs().smpr().read().smp(0)
    }

    /// Set the ADC resolution.
    pub fn set_resolution(&mut self, resolution: Resolution) {
        T::regs().cfgr1().modify(|reg| reg.set_res(resolution.into()));
    }

    /// Set hardware averaging.
    pub fn set_averaging(&mut self, averaging: Averaging) {
        let (enable, samples, right_shift) = match averaging {
            Averaging::Disabled => (false, Adc4OversamplingRatio::OVERSAMPLE2X, 0),
            Averaging::Samples2 => (true, Adc4OversamplingRatio::OVERSAMPLE2X, 1),
            Averaging::Samples4 => (true, Adc4OversamplingRatio::OVERSAMPLE4X, 2),
            Averaging::Samples8 => (true, Adc4OversamplingRatio::OVERSAMPLE8X, 3),
            Averaging::Samples16 => (true, Adc4OversamplingRatio::OVERSAMPLE16X, 4),
            Averaging::Samples32 => (true, Adc4OversamplingRatio::OVERSAMPLE32X, 5),
            Averaging::Samples64 => (true, Adc4OversamplingRatio::OVERSAMPLE64X, 6),
            Averaging::Samples128 => (true, Adc4OversamplingRatio::OVERSAMPLE128X, 7),
            Averaging::Samples256 => (true, Adc4OversamplingRatio::OVERSAMPLE256X, 8),
        };

        T::regs().cfgr2().modify(|reg| {
            reg.set_ovsr(samples);
            reg.set_ovss(right_shift);
            reg.set_ovse(enable)
        })
    }

    /// Perform a single conversion.
    fn convert(&mut self) -> u16 {
        T::regs().isr().modify(|reg| {
            reg.set_eos(true);
            reg.set_eoc(true);
        });

        // Start conversion
        T::regs().cr().modify(|reg| {
            reg.set_adstart(true);
        });

        while !T::regs().isr().read().eos() {
            // spin
        }

        T::regs().dr().read().0 as u16
    }

    /// Read an ADC channel.
    pub fn blocking_read(&mut self, channel: &mut impl AdcChannel<T>) -> u16 {
        self.read_channel(channel)
    }

    fn configure_channel(channel: &mut impl AdcChannel<T>) {
        channel.setup();
        T::regs().chselrmod0().write_value(Adc4Chselrmod0(0_u32));
        T::regs().chselrmod0().modify(|w| {
            w.set_chsel(channel.channel() as usize, true);
        });
    }

    fn read_channel(&mut self, channel: &mut impl AdcChannel<T>) -> u16 {
        Self::configure_channel(channel);
        let ret = self.convert();
        ret
    }
}