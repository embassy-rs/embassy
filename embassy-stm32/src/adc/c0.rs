#[allow(unused)]
use pac::adc::vals::{Adstp, Align, Ckmode, Dmacfg, Exten, Ovrmod, Ovsr};
use pac::adccommon::vals::Presc;
use stm32_metapac::adc::vals::{SampleTime, Scandir};

use super::{Adc, Instance, Resolution, blocking_delay_us};
use crate::adc::{AdcRegs, ConversionMode};
use crate::time::Hertz;
use crate::{Peri, pac, rcc};

/// Default VREF voltage used for sample conversion to millivolts.
pub const VREF_DEFAULT_MV: u32 = 3300;
/// VREF voltage used for factory calibration of VREFINTCAL register.
pub const VREF_CALIB_MV: u32 = 3300;

const MAX_ADC_CLK_FREQ: Hertz = Hertz::mhz(25);

const TIME_ADC_VOLTAGE_REGUALTOR_STARTUP_US: u32 = 20;

const CHSELR_SQ_SIZE: usize = 8;
const CHSELR_SQ_MAX_CHANNEL: u8 = 14;
const CHSELR_SQ_SEQUENCE_END_MARKER: u8 = 0b1111;

impl<T: Instance> super::SealedSpecialConverter<super::VrefInt> for T {
    const CHANNEL: u8 = 10;
}

impl<T: Instance> super::SealedSpecialConverter<super::Temperature> for T {
    const CHANNEL: u8 = 9;
}

fn from_ker_ck(frequency: Hertz) -> Presc {
    let raw_prescaler = rcc::raw_prescaler(frequency.0, MAX_ADC_CLK_FREQ.0);
    match raw_prescaler {
        0 => Presc::DIV1,
        1 => Presc::DIV2,
        2..=3 => Presc::DIV4,
        4..=5 => Presc::DIV6,
        6..=7 => Presc::DIV8,
        8..=9 => Presc::DIV10,
        10..=11 => Presc::DIV12,
        _ => unimplemented!(),
    }
}

impl AdcRegs for crate::pac::adc::Adc {
    fn data(&self) -> *mut u16 {
        crate::pac::adc::Adc::dr(*self).as_ptr() as *mut u16
    }

    fn enable(&self) {
        if !self.cr().read().aden() {
            self.isr().modify(|w| w.set_adrdy(true));
            self.cr().modify(|w| w.set_aden(true));
            // ADRDY is "ADC ready". Wait until it will be True.
            while !self.isr().read().adrdy() {}
        }
    }

    fn start(&self) {
        // Start conversion
        self.cr().modify(|reg| {
            reg.set_adstart(true);
        });
    }

    fn stop(&self) {
        if self.cr().read().adstart() && !self.cr().read().addis() {
            self.cr().modify(|reg| {
                reg.set_adstp(Adstp::STOP);
            });
            while self.cr().read().adstart() {}
        }

        // Reset configuration.
        self.cfgr1().modify(|reg| {
            reg.set_cont(false);
            reg.set_dmacfg(Dmacfg::from_bits(0));
            reg.set_dmaen(false);
        });
    }

    fn configure_dma(&self, conversion_mode: super::ConversionMode) {
        match conversion_mode {
            ConversionMode::Singular => {
                // Enable overrun control, so no new DMA requests will be generated until
                // previous DR values is read.
                self.isr().modify(|reg| {
                    reg.set_ovr(true);
                });

                // Set continuous mode with oneshot dma.
                self.cfgr1().modify(|reg| {
                    reg.set_discen(false);
                    reg.set_cont(true);
                    reg.set_dmacfg(Dmacfg::DMA_ONE_SHOT);
                    reg.set_dmaen(true);
                    reg.set_ovrmod(Ovrmod::PRESERVE);
                });
            }
        }
    }

    fn configure_sequence(&self, sequence: impl ExactSizeIterator<Item = ((u8, bool), Self::SampleTime)>) {
        // TODO: get sequencer working
        let mut needs_hw = sequence.len() == 1 || sequence.len() > CHSELR_SQ_SIZE || true;
        let mut is_ordered_up = true;
        let mut is_ordered_down = true;

        let sequence_len = sequence.len();
        let mut hw_channel_selection: u32 = 0;
        let mut last_channel: u8 = 0;
        let mut sample_time: Self::SampleTime = SampleTime::CYCLES2_5;

        self.chselr_sq().write(|w| {
            for (i, ((channel, _), _sample_time)) in sequence.enumerate() {
                assert!(
                    sample_time == _sample_time || i == 0,
                    "C0 only supports one sample time for the sequence."
                );

                sample_time = _sample_time;
                needs_hw = needs_hw || channel > CHSELR_SQ_MAX_CHANNEL;
                is_ordered_up = is_ordered_up && (channel > last_channel || i == 0);
                is_ordered_down = is_ordered_down && (channel < last_channel || i == 0);
                hw_channel_selection += 1 << channel;
                last_channel = channel;

                if !needs_hw {
                    w.set_sq(i, channel);
                }
            }

            for i in sequence_len..CHSELR_SQ_SIZE {
                w.set_sq(i, CHSELR_SQ_SEQUENCE_END_MARKER);
            }
        });

        if needs_hw {
            assert!(
                sequence_len <= CHSELR_SQ_SIZE || is_ordered_up || is_ordered_down,
                "Sequencer is required because of unordered channels, but read set cannot be more than {} in size.",
                CHSELR_SQ_SIZE
            );
            assert!(
                sequence_len > CHSELR_SQ_SIZE || is_ordered_up || is_ordered_down,
                "Sequencer is required because of unordered channels, but only support HW channels smaller than {}.",
                CHSELR_SQ_MAX_CHANNEL
            );

            // Set required channels for multi-convert.
            unsafe { (self.chselr().as_ptr() as *mut u32).write_volatile(hw_channel_selection) }
        }

        self.smpr().modify(|w| {
            w.smpsel(0);
            w.set_smp1(sample_time);
        });

        self.cfgr1().modify(|reg| {
            reg.set_chselrmod(!needs_hw);
            reg.set_align(Align::RIGHT);
            reg.set_scandir(if is_ordered_up { Scandir::UP } else { Scandir::BACK });
        });

        // Trigger and wait for the channel selection procedure to complete.
        self.isr().modify(|w| w.set_ccrdy(false));
        while !self.isr().read().ccrdy() {}
    }

    fn convert(&self) {
        // Set single conversion mode.
        self.cfgr1().modify(|w| w.set_cont(false));

        // Start conversion
        self.cr().modify(|reg| {
            reg.set_adstart(true);
        });

        // Waiting for End Of Conversion (EOC).
        while !self.isr().read().eoc() {}
    }
}

impl<'d, T: Instance<Regs = crate::pac::adc::Adc>> Adc<'d, T> {
    /// Create a new ADC driver.
    pub fn new(adc: Peri<'d, T>, resolution: Resolution) -> Self {
        rcc::enable_and_reset::<T>();

        T::regs().cfgr2().modify(|w| w.set_ckmode(Ckmode::SYSCLK));

        let prescaler = from_ker_ck(T::frequency());
        T::common_regs().ccr().modify(|w| w.set_presc(prescaler));

        let frequency = T::frequency() / prescaler;
        debug!("ADC frequency set to {}", frequency);

        if frequency > MAX_ADC_CLK_FREQ {
            panic!(
                "Maximal allowed frequency for the ADC is {} MHz and it varies with different packages, refer to ST docs for more information.",
                MAX_ADC_CLK_FREQ.0 / 1_000_000
            );
        }

        T::regs().cr().modify(|reg| {
            reg.set_advregen(true);
        });

        // "The software must wait for the ADC voltage regulator startup time."
        // See datasheet for the value.
        blocking_delay_us(TIME_ADC_VOLTAGE_REGUALTOR_STARTUP_US as u64 + 1);

        T::regs().cfgr1().modify(|reg| reg.set_res(resolution));

        // We have to make sure AUTOFF is OFF, but keep its value after calibration.
        let autoff_value = T::regs().cfgr1().read().autoff();
        T::regs().cfgr1().modify(|w| w.set_autoff(false));

        T::regs().cr().modify(|w| w.set_adcal(true));

        // "ADCAL bit stays at 1 during all the calibration sequence."
        // "It is then cleared by hardware as soon the calibration completes."
        while T::regs().cr().read().adcal() {}

        debug!("ADC calibration value: {}.", T::regs().dr().read().data());

        T::regs().cfgr1().modify(|w| w.set_autoff(autoff_value));

        T::regs().enable();

        // single conversion mode, software trigger
        T::regs().cfgr1().modify(|w| {
            w.set_cont(false);
            w.set_exten(Exten::DISABLED);
            w.set_align(Align::RIGHT);
        });

        Self { adc }
    }

    /// Enable reading the voltage reference internal channel.
    pub fn enable_vrefint(&self) -> super::VrefInt {
        T::common_regs().ccr().modify(|reg| {
            reg.set_vrefen(true);
        });

        super::VrefInt {}
    }

    /// Enable reading the temperature internal channel.
    pub fn enable_temperature(&self) -> super::Temperature {
        debug!("Ensure that sample time is set to more than temperature sensor T_start from the datasheet!");
        T::common_regs().ccr().modify(|reg| {
            reg.set_tsen(true);
        });

        super::Temperature {}
    }
}
