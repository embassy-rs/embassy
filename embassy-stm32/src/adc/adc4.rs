#[cfg(stm32u5)]
use pac::adc::vals::{Adc4Dmacfg as Dmacfg, Adc4Exten as Exten, Adc4OversamplingRatio as OversamplingRatio};
#[allow(unused)]
#[cfg(stm32wba)]
use pac::adc::vals::{Chselrmod, Cont, Dmacfg, Exten, OversamplingRatio, Ovss, Smpsel};

use super::{AdcChannel, AnyAdcChannel, RxDma4, blocking_delay_us};
use crate::dma::Transfer;
#[cfg(stm32u5)]
pub use crate::pac::adc::regs::Adc4Chselrmod0 as Chselr;
#[cfg(stm32wba)]
pub use crate::pac::adc::regs::Chselr;
#[cfg(stm32u5)]
pub use crate::pac::adc::vals::{Adc4Presc as Presc, Adc4Res as Resolution, Adc4SampleTime as SampleTime};
#[cfg(stm32wba)]
pub use crate::pac::adc::vals::{Presc, Res as Resolution, SampleTime};
use crate::time::Hertz;
use crate::{Peri, pac, rcc};

const MAX_ADC_CLK_FREQ: Hertz = Hertz::mhz(55);

/// Default VREF voltage used for sample conversion to millivolts.
pub const VREF_DEFAULT_MV: u32 = 3300;
/// VREF voltage used for factory calibration of VREFINTCAL register.
pub const VREF_CALIB_MV: u32 = 3300;

impl<'d, T: Instance> super::SealedSpecialConverter<super::VrefInt> for Adc4<'d, T> {
    const CHANNEL: u8 = 0;
}

impl<'d, T: Instance> super::SealedSpecialConverter<super::Temperature> for Adc4<'d, T> {
    const CHANNEL: u8 = 13;
}

impl<'d, T: Instance> super::SealedSpecialConverter<super::Vcore> for Adc4<'d, T> {
    const CHANNEL: u8 = 12;
}

impl<'d, T: Instance> super::SealedSpecialConverter<super::Vbat> for Adc4<'d, T> {
    const CHANNEL: u8 = 14;
}

impl<'d, T: Instance> super::SealedSpecialConverter<super::Dac> for Adc4<'d, T> {
    const CHANNEL: u8 = 21;
}

#[derive(Copy, Clone)]
pub enum DacChannel {
    OUT1,
    OUT2,
}

/// Number of samples used for averaging.
#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
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

pub trait Instance: SealedInstance + crate::PeripheralType + crate::rcc::RccPeripheral {
    type Interrupt: crate::interrupt::typelevel::Interrupt;
}

pub struct Adc4<'d, T: Instance> {
    #[allow(unused)]
    adc: crate::Peri<'d, T>,
}

#[derive(Copy, Clone, Debug)]
pub enum Adc4Error {
    InvalidSequence,
    DMAError,
}

impl<'d, T: Instance> Adc4<'d, T> {
    /// Create a new ADC driver.
    pub fn new(adc: Peri<'d, T>) -> Self {
        rcc::enable_and_reset::<T>();
        let prescaler = Prescaler::from_ker_ck(T::frequency());

        T::regs().ccr().modify(|w| w.set_presc(prescaler.presc()));

        let frequency = Hertz(T::frequency().0 / prescaler.divisor());
        info!("ADC4 frequency set to {}", frequency);

        if frequency > MAX_ADC_CLK_FREQ {
            panic!(
                "Maximal allowed frequency for ADC4 is {} MHz and it varies with different packages, refer to ST docs for more information.",
                MAX_ADC_CLK_FREQ.0 / 1_000_000
            );
        }

        T::regs().isr().modify(|w| {
            w.set_ldordy(true);
        });
        T::regs().cr().modify(|w| {
            w.set_advregen(true);
        });
        while !T::regs().isr().read().ldordy() {}

        T::regs().isr().modify(|w| {
            w.set_ldordy(true);
        });

        T::regs().cr().modify(|w| w.set_adcal(true));
        while T::regs().cr().read().adcal() {}
        T::regs().isr().modify(|w| w.set_eocal(true));

        blocking_delay_us(1);

        Self::enable();

        // single conversion mode, software trigger
        T::regs().cfgr1().modify(|w| {
            #[cfg(stm32u5)]
            w.set_cont(false);
            #[cfg(stm32wba)]
            w.set_cont(Cont::SINGLE);
            w.set_discen(false);
            w.set_exten(Exten::DISABLED);
            #[cfg(stm32u5)]
            w.set_chselrmod(false);
            #[cfg(stm32wba)]
            w.set_chselrmod(Chselrmod::ENABLE_INPUT);
        });

        // only use one channel at the moment
        T::regs().smpr().modify(|w| {
            #[cfg(stm32u5)]
            for i in 0..24 {
                w.set_smpsel(i, false);
            }
            #[cfg(stm32wba)]
            for i in 0..14 {
                w.set_smpsel(i, Smpsel::SMP1);
            }
        });

        Self { adc }
    }

    fn enable() {
        T::regs().isr().write(|w| w.set_adrdy(true));
        T::regs().cr().modify(|w| w.set_aden(true));
        while !T::regs().isr().read().adrdy() {}
        T::regs().isr().write(|w| w.set_adrdy(true));
    }

    /// Enable reading the voltage reference internal channel.
    pub fn enable_vrefint(&self) -> super::VrefInt {
        T::regs().ccr().modify(|w| {
            w.set_vrefen(true);
        });

        super::VrefInt {}
    }

    /// Enable reading the temperature internal channel.
    pub fn enable_temperature(&self) -> super::Temperature {
        T::regs().ccr().modify(|w| {
            w.set_vsensesel(true);
        });

        super::Temperature {}
    }

    /// Enable reading the vbat internal channel.
    #[cfg(stm32u5)]
    pub fn enable_vbat(&self) -> super::Vbat {
        T::regs().ccr().modify(|w| {
            w.set_vbaten(true);
        });

        super::Vbat {}
    }

    /// Enable reading the vbat internal channel.
    pub fn enable_vcore(&self) -> super::Vcore {
        super::Vcore {}
    }

    /// Enable reading the vbat internal channel.
    #[cfg(stm32u5)]
    pub fn enable_dac_channel(&self, dac: DacChannel) -> super::Dac {
        let mux;
        match dac {
            DacChannel::OUT1 => mux = false,
            DacChannel::OUT2 => mux = true,
        }
        T::regs().or().modify(|w| w.set_chn21sel(mux));
        super::Dac {}
    }

    /// Set the ADC resolution.
    pub fn set_resolution(&mut self, resolution: Resolution) {
        T::regs().cfgr1().modify(|w| w.set_res(resolution.into()));
    }

    /// Set hardware averaging.
    #[cfg(stm32u5)]
    pub fn set_averaging(&mut self, averaging: Averaging) {
        let (enable, samples, right_shift) = match averaging {
            Averaging::Disabled => (false, OversamplingRatio::OVERSAMPLE2X, 0),
            Averaging::Samples2 => (true, OversamplingRatio::OVERSAMPLE2X, 1),
            Averaging::Samples4 => (true, OversamplingRatio::OVERSAMPLE4X, 2),
            Averaging::Samples8 => (true, OversamplingRatio::OVERSAMPLE8X, 3),
            Averaging::Samples16 => (true, OversamplingRatio::OVERSAMPLE16X, 4),
            Averaging::Samples32 => (true, OversamplingRatio::OVERSAMPLE32X, 5),
            Averaging::Samples64 => (true, OversamplingRatio::OVERSAMPLE64X, 6),
            Averaging::Samples128 => (true, OversamplingRatio::OVERSAMPLE128X, 7),
            Averaging::Samples256 => (true, OversamplingRatio::OVERSAMPLE256X, 8),
        };

        T::regs().cfgr2().modify(|w| {
            w.set_ovsr(samples);
            w.set_ovss(right_shift);
            w.set_ovse(enable)
        })
    }
    #[cfg(stm32wba)]
    pub fn set_averaging(&mut self, averaging: Averaging) {
        let (enable, samples, right_shift) = match averaging {
            Averaging::Disabled => (false, OversamplingRatio::OVERSAMPLE2X, Ovss::SHIFT0),
            Averaging::Samples2 => (true, OversamplingRatio::OVERSAMPLE2X, Ovss::SHIFT1),
            Averaging::Samples4 => (true, OversamplingRatio::OVERSAMPLE4X, Ovss::SHIFT2),
            Averaging::Samples8 => (true, OversamplingRatio::OVERSAMPLE8X, Ovss::SHIFT3),
            Averaging::Samples16 => (true, OversamplingRatio::OVERSAMPLE16X, Ovss::SHIFT4),
            Averaging::Samples32 => (true, OversamplingRatio::OVERSAMPLE32X, Ovss::SHIFT5),
            Averaging::Samples64 => (true, OversamplingRatio::OVERSAMPLE64X, Ovss::SHIFT6),
            Averaging::Samples128 => (true, OversamplingRatio::OVERSAMPLE128X, Ovss::SHIFT7),
            Averaging::Samples256 => (true, OversamplingRatio::OVERSAMPLE256X, Ovss::SHIFT8),
        };

        T::regs().cfgr2().modify(|w| {
            w.set_ovsr(samples);
            w.set_ovss(right_shift);
            w.set_ovse(enable)
        })
    }

    /// Read an ADC channel.
    pub fn blocking_read(&mut self, channel: &mut impl AdcChannel<T>, sample_time: SampleTime) -> u16 {
        T::regs().smpr().modify(|w| {
            w.set_smp(0, sample_time);
        });

        channel.setup();

        // Select channel
        #[cfg(stm32wba)]
        {
            T::regs().chselr().write_value(Chselr(0_u32));
            T::regs().chselr().modify(|w| {
                w.set_chsel0(channel.channel() as usize, true);
            });
        }
        #[cfg(stm32u5)]
        {
            T::regs().chselrmod0().write_value(Chselr(0_u32));
            T::regs().chselrmod0().modify(|w| {
                w.set_chsel(channel.channel() as usize, true);
            });
        }

        // Reset interrupts
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

    /// Read one or multiple ADC channels using DMA.
    ///
    /// `sequence` iterator and `readings` must have the same length.
    /// The channels in `sequence` must be in ascending order.
    ///
    /// Example
    /// ```rust,ignore
    /// use embassy_stm32::adc::adc4;
    /// use embassy_stm32::adc::AdcChannel;
    ///
    /// let mut adc4 = adc4::Adc4::new(p.ADC4);
    /// let mut adc4_pin1 = p.PC1;
    /// let mut adc4_pin2 = p.PC0;
    /// let mut.into()d41 = adc4_pin1.into();
    /// let mut.into()d42 = adc4_pin2.into();
    /// let mut measurements = [0u16; 2];
    /// // not that the channels must be in ascending order
    /// adc4.read(
    ///     &mut p.GPDMA1_CH1,
    ///    [
    ///        &mut.into()d42,
    ///        &mut.into()d41,
    ///    ]
    ///    .into_iter(),
    ///    &mut measurements,
    /// ).await.unwrap();
    /// ```
    pub async fn read(
        &mut self,
        rx_dma: Peri<'_, impl RxDma4<T>>,
        sequence: impl ExactSizeIterator<Item = &mut AnyAdcChannel<T>>,
        readings: &mut [u16],
    ) -> Result<(), Adc4Error> {
        assert!(sequence.len() != 0, "Asynchronous read sequence cannot be empty");
        assert!(
            sequence.len() == readings.len(),
            "Sequence length must be equal to readings length"
        );

        // Ensure no conversions are ongoing
        Self::cancel_conversions();

        T::regs().isr().modify(|reg| {
            reg.set_ovr(true);
            reg.set_eos(true);
            reg.set_eoc(true);
        });

        T::regs().cfgr1().modify(|reg| {
            reg.set_dmaen(true);
            reg.set_dmacfg(Dmacfg::ONE_SHOT);
            #[cfg(stm32u5)]
            reg.set_chselrmod(false);
            #[cfg(stm32wba)]
            reg.set_chselrmod(Chselrmod::ENABLE_INPUT)
        });

        // Verify and activate sequence
        let mut prev_channel: i16 = -1;
        #[cfg(stm32wba)]
        T::regs().chselr().write_value(Chselr(0_u32));
        #[cfg(stm32u5)]
        T::regs().chselrmod0().write_value(Chselr(0_u32));
        for channel in sequence {
            let channel_num = channel.channel;
            if channel_num as i16 <= prev_channel {
                return Err(Adc4Error::InvalidSequence);
            };
            prev_channel = channel_num as i16;

            #[cfg(stm32wba)]
            T::regs().chselr().modify(|w| {
                w.set_chsel0(channel.channel as usize, true);
            });
            #[cfg(stm32u5)]
            T::regs().chselrmod0().modify(|w| {
                w.set_chsel(channel.channel as usize, true);
            });
        }

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

        transfer.await;

        // Ensure conversions are finished.
        Self::cancel_conversions();

        // Reset configuration.
        T::regs().cfgr1().modify(|reg| {
            reg.set_dmaen(false);
        });

        if T::regs().isr().read().ovr() {
            Err(Adc4Error::DMAError)
        } else {
            Ok(())
        }
    }

    fn cancel_conversions() {
        if T::regs().cr().read().adstart() && !T::regs().cr().read().addis() {
            T::regs().cr().modify(|reg| {
                reg.set_adstp(true);
            });
            while T::regs().cr().read().adstart() {}
        }
    }
}
