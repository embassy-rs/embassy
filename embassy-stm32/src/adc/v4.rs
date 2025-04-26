#[cfg(not(stm32u5))]
use pac::adc::vals::{Adcaldif, Boost};
#[allow(unused)]
use pac::adc::vals::{Adstp, Difsel, Dmngt, Exten, Pcsel};
use pac::adccommon::vals::Presc;

use super::{
    blocking_delay_us, Adc, AdcChannel, AnyAdcChannel, Instance, Resolution, RxDma, SampleTime, SealedAdcChannel,
};
use crate::dma::Transfer;
use crate::time::Hertz;
use crate::{pac, rcc, Peri};

/// Default VREF voltage used for sample conversion to millivolts.
pub const VREF_DEFAULT_MV: u32 = 3300;
/// VREF voltage used for factory calibration of VREFINTCAL register.
pub const VREF_CALIB_MV: u32 = 3300;

/// Max single ADC operation clock frequency
#[cfg(stm32g4)]
const MAX_ADC_CLK_FREQ: Hertz = Hertz::mhz(60);
#[cfg(stm32h7)]
const MAX_ADC_CLK_FREQ: Hertz = Hertz::mhz(50);
#[cfg(stm32u5)]
const MAX_ADC_CLK_FREQ: Hertz = Hertz::mhz(55);

#[cfg(stm32g4)]
const VREF_CHANNEL: u8 = 18;
#[cfg(stm32g4)]
const TEMP_CHANNEL: u8 = 16;

#[cfg(stm32h7)]
const VREF_CHANNEL: u8 = 19;
#[cfg(stm32h7)]
const TEMP_CHANNEL: u8 = 18;

// TODO this should be 14 for H7a/b/35
#[cfg(not(stm32u5))]
const VBAT_CHANNEL: u8 = 17;

#[cfg(stm32u5)]
const VREF_CHANNEL: u8 = 0;
#[cfg(stm32u5)]
const TEMP_CHANNEL: u8 = 19;
#[cfg(stm32u5)]
const VBAT_CHANNEL: u8 = 18;

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
    Samples512,
    Samples1024,
}

impl<'d, T: Instance> Adc<'d, T> {
    /// Create a new ADC driver.
    pub fn new(adc: Peri<'d, T>) -> Self {
        rcc::enable_and_reset::<T>();

        let prescaler = Prescaler::from_ker_ck(T::frequency());

        T::common_regs().ccr().modify(|w| w.set_presc(prescaler.presc()));

        let frequency = Hertz(T::frequency().0 / prescaler.divisor());
        info!("ADC frequency set to {}", frequency);

        if frequency > MAX_ADC_CLK_FREQ {
            panic!("Maximal allowed frequency for the ADC is {} MHz and it varies with different packages, refer to ST docs for more information.", MAX_ADC_CLK_FREQ.0 /  1_000_000 );
        }

        #[cfg(stm32h7)]
        {
            let boost = if frequency < Hertz::khz(6_250) {
                Boost::LT6_25
            } else if frequency < Hertz::khz(12_500) {
                Boost::LT12_5
            } else if frequency < Hertz::mhz(25) {
                Boost::LT25
            } else {
                Boost::LT50
            };
            T::regs().cr().modify(|w| w.set_boost(boost));
        }
        let mut s = Self {
            adc,
            sample_time: SampleTime::from_bits(0),
        };
        s.power_up();
        s.configure_differential_inputs();

        s.calibrate();
        blocking_delay_us(1);

        s.enable();
        s.configure();

        s
    }

    fn power_up(&mut self) {
        T::regs().cr().modify(|reg| {
            reg.set_deeppwd(false);
            reg.set_advregen(true);
        });

        blocking_delay_us(10);
    }

    fn configure_differential_inputs(&mut self) {
        T::regs().difsel().modify(|w| {
            for n in 0..20 {
                w.set_difsel(n, Difsel::SINGLE_ENDED);
            }
        });
    }

    fn calibrate(&mut self) {
        T::regs().cr().modify(|w| {
            #[cfg(not(adc_u5))]
            w.set_adcaldif(Adcaldif::SINGLE_ENDED);
            w.set_adcallin(true);
        });

        T::regs().cr().modify(|w| w.set_adcal(true));

        while T::regs().cr().read().adcal() {}
    }

    fn enable(&mut self) {
        T::regs().isr().write(|w| w.set_adrdy(true));
        T::regs().cr().modify(|w| w.set_aden(true));
        while !T::regs().isr().read().adrdy() {}
        T::regs().isr().write(|w| w.set_adrdy(true));
    }

    fn configure(&mut self) {
        // single conversion mode, software trigger
        T::regs().cfgr().modify(|w| {
            w.set_cont(false);
            w.set_exten(Exten::DISABLED);
        });
    }

    /// Enable reading the voltage reference internal channel.
    pub fn enable_vrefint(&self) -> VrefInt {
        T::common_regs().ccr().modify(|reg| {
            reg.set_vrefen(true);
        });

        VrefInt {}
    }

    /// Enable reading the temperature internal channel.
    pub fn enable_temperature(&self) -> Temperature {
        T::common_regs().ccr().modify(|reg| {
            reg.set_vsenseen(true);
        });

        Temperature {}
    }

    /// Enable reading the vbat internal channel.
    pub fn enable_vbat(&self) -> Vbat {
        T::common_regs().ccr().modify(|reg| {
            reg.set_vbaten(true);
        });

        Vbat {}
    }

    /// Set the ADC sample time.
    pub fn set_sample_time(&mut self, sample_time: SampleTime) {
        self.sample_time = sample_time;
    }

    /// Get the ADC sample time.
    pub fn sample_time(&self) -> SampleTime {
        self.sample_time
    }

    /// Set the ADC resolution.
    pub fn set_resolution(&mut self, resolution: Resolution) {
        T::regs().cfgr().modify(|reg| reg.set_res(resolution.into()));
    }

    /// Set hardware averaging.
    pub fn set_averaging(&mut self, averaging: Averaging) {
        let (enable, samples, right_shift) = match averaging {
            Averaging::Disabled => (false, 0, 0),
            Averaging::Samples2 => (true, 1, 1),
            Averaging::Samples4 => (true, 3, 2),
            Averaging::Samples8 => (true, 7, 3),
            Averaging::Samples16 => (true, 15, 4),
            Averaging::Samples32 => (true, 31, 5),
            Averaging::Samples64 => (true, 63, 6),
            Averaging::Samples128 => (true, 127, 7),
            Averaging::Samples256 => (true, 255, 8),
            Averaging::Samples512 => (true, 511, 9),
            Averaging::Samples1024 => (true, 1023, 10),
        };

        T::regs().cfgr2().modify(|reg| {
            reg.set_rovse(enable);
            reg.set_ovsr(samples);
            reg.set_ovss(right_shift);
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

    /// Read one or multiple ADC channels using DMA.
    ///
    /// `sequence` iterator and `readings` must have the same length.
    ///
    /// Example
    /// ```rust,ignore
    /// use embassy_stm32::adc::{Adc, AdcChannel}
    ///
    /// let mut adc = Adc::new(p.ADC1);
    /// let mut adc_pin0 = p.PA0.into();
    /// let mut adc_pin2 = p.PA2.into();
    /// let mut measurements = [0u16; 2];
    ///
    /// adc.read_async(
    ///     p.DMA2_CH0,
    ///     [
    ///         (&mut *adc_pin0, SampleTime::CYCLES112),
    ///         (&mut *adc_pin2, SampleTime::CYCLES112),
    ///     ]
    ///     .into_iter(),
    ///     &mut measurements,
    /// )
    /// .await;
    /// defmt::info!("measurements: {}", measurements);
    /// ```
    pub async fn read(
        &mut self,
        rx_dma: Peri<'_, impl RxDma<T>>,
        sequence: impl ExactSizeIterator<Item = (&mut AnyAdcChannel<T>, SampleTime)>,
        readings: &mut [u16],
    ) {
        assert!(sequence.len() != 0, "Asynchronous read sequence cannot be empty");
        assert!(
            sequence.len() == readings.len(),
            "Sequence length must be equal to readings length"
        );
        assert!(
            sequence.len() <= 16,
            "Asynchronous read sequence cannot be more than 16 in length"
        );

        // Ensure no conversions are ongoing
        Self::cancel_conversions();

        // Set sequence length
        T::regs().sqr1().modify(|w| {
            w.set_l(sequence.len() as u8 - 1);
        });

        // Configure channels and ranks
        for (i, (channel, sample_time)) in sequence.enumerate() {
            Self::configure_channel(channel, sample_time);
            match i {
                0..=3 => {
                    T::regs().sqr1().modify(|w| {
                        w.set_sq(i, channel.channel());
                    });
                }
                4..=8 => {
                    T::regs().sqr2().modify(|w| {
                        w.set_sq(i - 4, channel.channel());
                    });
                }
                9..=13 => {
                    T::regs().sqr3().modify(|w| {
                        w.set_sq(i - 9, channel.channel());
                    });
                }
                14..=15 => {
                    T::regs().sqr4().modify(|w| {
                        w.set_sq(i - 14, channel.channel());
                    });
                }
                _ => unreachable!(),
            }
        }

        // Set continuous mode with oneshot dma.
        // Clear overrun flag before starting transfer.

        T::regs().isr().modify(|reg| {
            reg.set_ovr(true);
        });
        T::regs().cfgr().modify(|reg| {
            reg.set_cont(true);
            reg.set_dmngt(Dmngt::DMA_ONE_SHOT);
        });

        let request = rx_dma.request();
        let transfer = unsafe {
            Transfer::new_read(
                rx_dma,
                request,
                T::regs().dr().as_ptr() as *mut u16,
                readings,
                Default::default(),
            )
        };

        // Start conversion
        T::regs().cr().modify(|reg| {
            reg.set_adstart(true);
        });

        // Wait for conversion sequence to finish.
        transfer.await;

        // Ensure conversions are finished.
        Self::cancel_conversions();

        // Reset configuration.
        T::regs().cfgr().modify(|reg| {
            reg.set_cont(false);
            reg.set_dmngt(Dmngt::from_bits(0));
        });
    }

    fn configure_channel(channel: &mut impl AdcChannel<T>, sample_time: SampleTime) {
        channel.setup();

        let channel = channel.channel();

        Self::set_channel_sample_time(channel, sample_time);

        #[cfg(any(stm32h7, stm32u5))]
        {
            T::regs().cfgr2().modify(|w| w.set_lshift(0));
            T::regs()
                .pcsel()
                .modify(|w| w.set_pcsel(channel as _, Pcsel::PRESELECTED));
        }
    }

    fn read_channel(&mut self, channel: &mut impl AdcChannel<T>) -> u16 {
        Self::configure_channel(channel, self.sample_time);

        T::regs().sqr1().modify(|reg| {
            reg.set_sq(0, channel.channel());
            reg.set_l(0);
        });

        self.convert()
    }

    fn set_channel_sample_time(ch: u8, sample_time: SampleTime) {
        let sample_time = sample_time.into();
        if ch <= 9 {
            T::regs().smpr(0).modify(|reg| reg.set_smp(ch as _, sample_time));
        } else {
            T::regs().smpr(1).modify(|reg| reg.set_smp((ch - 10) as _, sample_time));
        }
    }

    fn cancel_conversions() {
        if T::regs().cr().read().adstart() && !T::regs().cr().read().addis() {
            T::regs().cr().modify(|reg| {
                reg.set_adstp(Adstp::STOP);
            });
            while T::regs().cr().read().adstart() {}
        }
    }
}
