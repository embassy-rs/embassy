#[cfg(stm32g4)]
use pac::adc::regs::Difsel as DifselReg;
#[allow(unused)]
#[cfg(stm32g4)]
pub use pac::adc::vals::{Adcaldif, Adstp, Difsel, Dmacfg, Dmaen, Exten, Rovsm, Trovs};
#[allow(unused)]
#[cfg(stm32h7)]
use pac::adc::vals::{Adcaldif, Difsel, Exten};
pub use pac::adccommon::vals::{Dual, Presc};

use super::{
    Adc, AnyAdcChannel, ConversionMode, ConversionTrigger, Instance, RegularConversionMode, Resolution, RxDma,
    SampleTime, blocking_delay_us,
};
use crate::adc::{AdcRegs, BasicAdcRegs, SealedAdcChannel};
use crate::pac::adc::regs::{Smpr, Smpr2, Sqr1, Sqr2, Sqr3, Sqr4};
use crate::time::Hertz;
use crate::{Peri, pac, rcc};

mod injected;
pub use injected::InjectedAdc;

/// Default VREF voltage used for sample conversion to millivolts.
pub const VREF_DEFAULT_MV: u32 = 3300;
/// VREF voltage used for factory calibration of VREFINTCAL register.
pub const VREF_CALIB_MV: u32 = 3300;

const NR_INJECTED_RANKS: usize = 4;

/// Max single ADC operation clock frequency
#[cfg(stm32g4)]
const MAX_ADC_CLK_FREQ: Hertz = Hertz::mhz(60);
#[cfg(stm32h7)]
const MAX_ADC_CLK_FREQ: Hertz = Hertz::mhz(50);

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

/// ADC configuration
#[derive(Default)]
pub struct AdcConfig {
    pub dual_mode: Option<Dual>,
    pub resolution: Option<Resolution>,
    #[cfg(stm32g4)]
    pub oversampling_shift: Option<u8>,
    #[cfg(stm32g4)]
    pub oversampling_ratio: Option<u8>,
    #[cfg(stm32g4)]
    pub oversampling_mode: Option<(Rovsm, Trovs, bool)>,
}

impl super::AdcRegs for crate::pac::adc::Adc {
    fn data(&self) -> *mut u16 {
        crate::pac::adc::Adc::dr(*self).as_ptr() as *mut u16
    }

    fn enable(&self) {
        // Make sure bits are off
        while self.cr().read().addis() {
            // spin
        }

        if !self.cr().read().aden() {
            // Enable ADC
            self.isr().modify(|reg| {
                reg.set_adrdy(true);
            });
            self.cr().modify(|reg| {
                reg.set_aden(true);
            });

            while !self.isr().read().adrdy() {
                // spin
            }
        }
    }

    fn start(&self) {
        self.cr().modify(|reg| {
            reg.set_adstart(true);
        });
    }

    fn stop(&self) {
        if self.cr().read().adstart() && !self.cr().read().addis() {
            self.cr().modify(|reg| {
                reg.set_adstp(Adstp::STOP);
            });
            // The software must poll ADSTART until the bit is reset before assuming the
            // ADC is completely stopped
            while self.cr().read().adstart() {}
        }

        // Disable dma control and continuous conversion, if enabled
        self.cfgr().modify(|reg| {
            reg.set_cont(false);
            reg.set_dmaen(Dmaen::DISABLE);
        });
    }

    fn convert(&self) {
        self.isr().modify(|reg| {
            reg.set_eos(true);
            reg.set_eoc(true);
        });

        // Start conversion
        self.cr().modify(|reg| {
            reg.set_adstart(true);
        });

        while !self.isr().read().eos() {
            // spin
        }
    }

    fn configure_dma(&self, conversion_mode: ConversionMode) {
        self.isr().modify(|reg| {
            reg.set_ovr(true);
        });

        self.cfgr().modify(|reg| {
            reg.set_discen(false); // Convert all channels for each trigger
            reg.set_dmacfg(match conversion_mode {
                ConversionMode::Singular => Dmacfg::ONE_SHOT,
                ConversionMode::Repeated(_) => Dmacfg::CIRCULAR,
            });
            reg.set_dmaen(Dmaen::ENABLE);
        });

        if let ConversionMode::Repeated(mode) = conversion_mode {
            match mode {
                RegularConversionMode::Continuous => {
                    self.cfgr().modify(|reg| {
                        reg.set_cont(true);
                    });
                }
                RegularConversionMode::Triggered(trigger) => {
                    self.cfgr().modify(|r| {
                        r.set_cont(false); // New trigger is neede for each sample to be read
                    });

                    self.cfgr().modify(|r| {
                        r.set_extsel(trigger.channel);
                        r.set_exten(trigger.edge);
                    });

                    // Regular conversions uses DMA so no need to generate interrupt
                    self.ier().modify(|r| r.set_eosie(false));
                }
            }
        }
    }

    fn configure_sequence(&self, sequence: impl ExactSizeIterator<Item = ((u8, bool), SampleTime)>) {
        self.cr().modify(|w| w.set_aden(false));

        #[cfg(stm32g4)]
        let mut difsel = DifselReg::default();
        let mut smpr = Smpr::default();
        let mut smpr2 = Smpr2::default();
        let mut sqr1 = Sqr1::default();
        let mut sqr2 = Sqr2::default();
        let mut sqr3 = Sqr3::default();
        let mut sqr4 = Sqr4::default();

        // Set sequence length
        sqr1.set_l(sequence.len() as u8 - 1);

        // Configure channels and ranks
        for (_i, ((ch, is_differential), sample_time)) in sequence.enumerate() {
            let sample_time = sample_time.into();
            if ch <= 9 {
                smpr.set_smp(ch as _, sample_time);
            } else {
                smpr2.set_smp((ch - 10) as _, sample_time);
            }

            match _i {
                0..=3 => {
                    sqr1.set_sq(_i, ch);
                }
                4..=8 => {
                    sqr2.set_sq(_i - 4, ch);
                }
                9..=13 => {
                    sqr3.set_sq(_i - 9, ch);
                }
                14..=15 => {
                    sqr4.set_sq(_i - 14, ch);
                }
                _ => unreachable!(),
            }

            #[cfg(stm32g4)]
            {
                if ch < 18 {
                    difsel.set_difsel(
                        ch.into(),
                        if is_differential {
                            Difsel::DIFFERENTIAL
                        } else {
                            Difsel::SINGLE_ENDED
                        },
                    );
                }
            }
        }

        self.smpr().write_value(smpr);
        self.smpr2().write_value(smpr2);
        self.sqr1().write_value(sqr1);
        self.sqr2().write_value(sqr2);
        self.sqr3().write_value(sqr3);
        self.sqr4().write_value(sqr4);
        #[cfg(stm32g4)]
        self.difsel().write_value(difsel);
    }
}

impl<'d, T: Instance<Regs = crate::pac::adc::Adc>> Adc<'d, T> {
    /// Create a new ADC driver.
    pub fn new(adc: Peri<'d, T>, config: AdcConfig) -> Self {
        rcc::enable_and_reset::<T>();

        let prescaler = from_ker_ck(T::frequency());

        T::common_regs().ccr().modify(|w| w.set_presc(prescaler));

        let frequency = T::frequency() / prescaler;
        trace!("ADC frequency set to {}", frequency);

        if frequency > MAX_ADC_CLK_FREQ {
            panic!(
                "Maximal allowed frequency for the ADC is {} MHz and it varies with different packages, refer to ST docs for more information.",
                MAX_ADC_CLK_FREQ.0 / 1_000_000
            );
        }

        T::regs().cr().modify(|reg| {
            reg.set_deeppwd(false);
            reg.set_advregen(true);
        });

        blocking_delay_us(20);

        T::regs().difsel().modify(|w| {
            for n in 0..18 {
                w.set_difsel(n, Difsel::SINGLE_ENDED);
            }
        });

        T::regs().cr().modify(|w| {
            w.set_adcaldif(Adcaldif::SINGLE_ENDED);
        });

        T::regs().cr().modify(|w| w.set_adcal(true));

        while T::regs().cr().read().adcal() {}

        blocking_delay_us(20);

        T::regs().cr().modify(|w| {
            w.set_adcaldif(Adcaldif::DIFFERENTIAL);
        });

        T::regs().cr().modify(|w| w.set_adcal(true));

        while T::regs().cr().read().adcal() {}

        blocking_delay_us(20);

        T::regs().enable();

        // single conversion mode, software trigger
        T::regs().cfgr().modify(|w| {
            w.set_cont(false);
            w.set_exten(Exten::DISABLED);
        });

        if let Some(dual) = config.dual_mode {
            T::common_regs().ccr().modify(|reg| {
                reg.set_dual(dual);
            })
        }

        if let Some(resolution) = config.resolution {
            T::regs().cfgr().modify(|reg| reg.set_res(resolution.into()));
        }

        #[cfg(stm32g4)]
        if let Some(shift) = config.oversampling_shift {
            T::regs().cfgr2().modify(|reg| reg.set_ovss(shift));
        }

        #[cfg(stm32g4)]
        if let Some(ratio) = config.oversampling_ratio {
            T::regs().cfgr2().modify(|reg| reg.set_ovsr(ratio));
        }

        #[cfg(stm32g4)]
        if let Some((mode, trig_mode, enable)) = config.oversampling_mode {
            T::regs().cfgr2().modify(|reg| reg.set_trovs(trig_mode));
            T::regs().cfgr2().modify(|reg| reg.set_rovsm(mode));
            T::regs().cfgr2().modify(|reg| reg.set_rovse(enable));
        }

        Self { adc }
    }

    /// Enable reading the voltage reference internal channel.
    pub fn enable_vrefint(&self) -> super::VrefInt
    where
        T: super::SpecialConverter<super::VrefInt>,
    {
        T::common_regs().ccr().modify(|reg| {
            reg.set_vrefen(true);
        });

        super::VrefInt {}
    }

    /// Enable reading the temperature internal channel.
    pub fn enable_temperature(&self) -> super::Temperature
    where
        T: super::SpecialConverter<super::Temperature>,
    {
        T::common_regs().ccr().modify(|reg| {
            reg.set_vsenseen(true);
        });

        super::Temperature {}
    }

    /// Enable reading the vbat internal channel.
    pub fn enable_vbat(&self) -> super::Vbat
    where
        T: super::SpecialConverter<super::Vbat>,
    {
        T::common_regs().ccr().modify(|reg| {
            reg.set_vbaten(true);
        });

        super::Vbat {}
    }

    // Reads that are not implemented as INJECTED in "blocking_read"
    // #[cfg(stm32g4)]
    // pub fn enalble_injected_oversampling_mode(&mut self, enable: bool) {
    //     T::regs().cfgr2().modify(|reg| reg.set_jovse(enable));
    // }

    // #[cfg(stm32g4)]
    // pub fn enable_oversampling_regular_injected_mode(&mut self, enable: bool) {
    //     // the regularoversampling mode is forced to resumed mode (ROVSM bit ignored),
    //     T::regs().cfgr2().modify(|reg| reg.set_rovse(enable));
    //     T::regs().cfgr2().modify(|reg| reg.set_jovse(enable));
    // }

    /// Configures the ADC for injected conversions.
    ///
    /// Injected conversions are separate from the regular conversion sequence and are typically
    /// triggered by software or an external event. This method sets up a fixed-length sequence of
    /// injected channels with specified sample times, the trigger source, and whether the end-of-sequence
    /// interrupt should be enabled.
    ///
    /// # Parameters
    /// - `sequence`: An array of tuples containing the ADC channels and their sample times. The length
    ///   `N` determines the number of injected ranks to configure (maximum 4 for STM32).
    /// - `trigger`: The trigger source that starts the injected conversion sequence.
    /// - `interrupt`: If `true`, enables the end-of-sequence (JEOS) interrupt for injected conversions.
    ///
    /// # Returns
    /// An `InjectedAdc<T, N>` instance that represents the configured injected sequence. The returned
    /// type encodes the sequence length `N` in its type, ensuring that reads return exactly `N` samples.
    ///
    /// # Panics
    /// This function will panic if:
    /// - `sequence` is empty.
    /// - `sequence` length exceeds the maximum number of injected ranks (`NR_INJECTED_RANKS`).
    ///
    /// # Notes
    /// - Injected conversions can run independently of regular ADC conversions.
    /// - The order of channels in `sequence` determines the rank order in the injected sequence.
    /// - Accessing samples beyond `N` will result in a panic; use the returned type
    ///   `InjectedAdc<T, N>` to enforce bounds at compile time.
    pub fn setup_injected_conversions<'a, const N: usize>(
        self,
        sequence: [(AnyAdcChannel<'a, T>, SampleTime); N],
        trigger: ConversionTrigger,
        interrupt: bool,
    ) -> InjectedAdc<'a, T, N> {
        assert!(N != 0, "Read sequence cannot be empty");
        assert!(
            N <= NR_INJECTED_RANKS,
            "Read sequence cannot be more than {} in length",
            NR_INJECTED_RANKS
        );

        T::regs().enable();

        T::regs().jsqr().modify(|w| w.set_jl(N as u8 - 1));

        for (n, (channel, sample_time)) in sequence.iter().enumerate() {
            let sample_time = sample_time.clone().into();
            if channel.channel() <= 9 {
                T::regs()
                    .smpr()
                    .modify(|reg| reg.set_smp(channel.channel() as _, sample_time));
            } else {
                T::regs()
                    .smpr2()
                    .modify(|reg| reg.set_smp((channel.channel() - 10) as _, sample_time));
            }

            let idx = match n {
                0..=3 => n,
                4..=8 => n - 4,
                9..=13 => n - 9,
                14..=15 => n - 14,
                _ => unreachable!(),
            };

            T::regs().jsqr().modify(|w| w.set_jsq(idx, channel.channel()));
        }

        T::regs().cfgr().modify(|reg| reg.set_jdiscen(false));

        // Set external trigger for injected conversion sequence
        // Possible trigger values are seen in Table 167 in RM0440 Rev 9
        T::regs().jsqr().modify(|r| {
            r.set_jextsel(trigger.channel);
            r.set_jexten(trigger.edge);
        });

        // Enable end of injected sequence interrupt
        T::regs().ier().modify(|r| r.set_jeosie(interrupt));

        Self::start_injected_conversions();

        InjectedAdc::new(sequence) // InjectedAdc<'a, T, N> now borrows the channels
    }

    /// Configures ADC for both regular conversions with a ring-buffered DMA and injected conversions.
    ///
    /// # Parameters
    /// - `dma`: The DMA peripheral to use for the ring-buffered ADC transfers.
    /// - `dma_buf`: The buffer to store DMA-transferred samples for regular conversions.
    /// - `regular_sequence`: The sequence of channels and their sample times for regular conversions.
    /// - `regular_conversion_mode`: The mode for regular conversions (e.g., continuous or triggered).
    /// - `injected_sequence`: An array of channels and sample times for injected conversions (length `N`).
    /// - `injected_trigger`: The trigger source for injected conversions.
    /// - `injected_interrupt`: Whether to enable the end-of-sequence interrupt for injected conversions.
    ///
    /// Injected conversions are typically used with interrupts. If ADC1 and ADC2 are used in dual mode,
    /// it is recommended to enable interrupts only for the ADC whose sequence takes the longest to complete.
    ///
    /// # Returns
    /// A tuple containing:
    /// 1. `RingBufferedAdc<'a, T>` — the configured ADC for regular conversions using DMA.
    /// 2. `InjectedAdc<T, N>` — the configured ADC for injected conversions.
    ///
    /// # Safety
    /// This function is `unsafe` because it clones the ADC peripheral handle unchecked. Both the
    /// `RingBufferedAdc` and `InjectedAdc` take ownership of the handle and drop it independently.
    /// Ensure no other code concurrently accesses the same ADC instance in a conflicting way.
    pub fn into_ring_buffered_and_injected<'a, 'b, const N: usize, D: RxDma<T>>(
        self,
        dma: Peri<'a, D>,
        dma_buf: &'a mut [u16],
        _irq: impl crate::interrupt::typelevel::Binding<D::Interrupt, crate::dma::InterruptHandler<D>> + 'a,
        regular_sequence: impl ExactSizeIterator<Item = (AnyAdcChannel<'b, T>, <T::Regs as BasicAdcRegs>::SampleTime)>,
        regular_conversion_mode: RegularConversionMode,
        injected_sequence: [(AnyAdcChannel<'b, T>, SampleTime); N],
        injected_trigger: ConversionTrigger,
        injected_interrupt: bool,
    ) -> (super::RingBufferedAdc<'a, T>, InjectedAdc<'b, T, N>) {
        unsafe {
            (
                Self {
                    adc: self.adc.clone_unchecked(),
                }
                .into_ring_buffered(dma, dma_buf, _irq, regular_sequence, regular_conversion_mode),
                Self {
                    adc: self.adc.clone_unchecked(),
                }
                .setup_injected_conversions(injected_sequence, injected_trigger, injected_interrupt),
            )
        }
    }

    /// Stop injected conversions
    pub(super) fn stop_injected_conversions() {
        if T::regs().cr().read().adstart() && !T::regs().cr().read().addis() {
            T::regs().cr().modify(|reg| {
                reg.set_jadstp(Adstp::STOP);
            });
            // The software must poll JADSTART until the bit is reset before assuming the
            // ADC is completely stopped
            while T::regs().cr().read().jadstart() {}
        }
    }

    /// Start injected ADC conversion
    pub(super) fn start_injected_conversions() {
        T::regs().cr().modify(|reg| {
            reg.set_jadstart(true);
        });
    }
}

impl<'a, T: Instance<Regs = crate::pac::adc::Adc>, const N: usize> InjectedAdc<'a, T, N> {
    /// Read sampled data from all injected ADC injected ranks
    /// Clear the JEOS flag to allow a new injected sequence
    pub(super) fn read_injected_data() -> [u16; N] {
        let mut data = [0u16; N];
        for i in 0..N {
            data[i] = T::regs().jdr(i).read().jdata();
        }

        // Clear JEOS by writing 1
        T::regs().isr().modify(|r| r.set_jeos(true));
        data
    }
}

#[cfg(stm32g4)]
mod g4 {
    use crate::adc::{SealedSpecialConverter, Temperature, Vbat, VrefInt};

    impl SealedSpecialConverter<Temperature> for crate::peripherals::ADC1 {
        const CHANNEL: u8 = 16;
    }

    impl SealedSpecialConverter<VrefInt> for crate::peripherals::ADC1 {
        const CHANNEL: u8 = 18;
    }

    impl SealedSpecialConverter<Vbat> for crate::peripherals::ADC1 {
        const CHANNEL: u8 = 17;
    }

    #[cfg(peri_adc3_common)]
    impl SealedSpecialConverter<VrefInt> for crate::peripherals::ADC3 {
        const CHANNEL: u8 = 18;
    }

    #[cfg(peri_adc3_common)]
    impl SealedSpecialConverter<Vbat> for crate::peripherals::ADC3 {
        const CHANNEL: u8 = 17;
    }

    #[cfg(not(stm32g4x1))]
    impl SealedSpecialConverter<VrefInt> for crate::peripherals::ADC4 {
        const CHANNEL: u8 = 18;
    }

    #[cfg(not(stm32g4x1))]
    impl SealedSpecialConverter<Temperature> for crate::peripherals::ADC5 {
        const CHANNEL: u8 = 4;
    }

    #[cfg(not(stm32g4x1))]
    impl SealedSpecialConverter<VrefInt> for crate::peripherals::ADC5 {
        const CHANNEL: u8 = 18;
    }

    #[cfg(not(stm32g4x1))]
    impl SealedSpecialConverter<Vbat> for crate::peripherals::ADC5 {
        const CHANNEL: u8 = 17;
    }
}

// TODO this should look at each ADC individually and impl the correct channels
#[cfg(stm32h7)]
mod h7 {
    impl<T: Instance> SealedSpecialConverter<Temperature> for T {
        const CHANNEL: u8 = 18;
    }
    impl<T: Instance> SealedSpecialConverter<VrefInt> for T {
        const CHANNEL: u8 = 19;
    }
    impl<T: Instance> SealedSpecialConverter<Vbat> for T {
        // TODO this should be 14 for H7a/b/35
        const CHANNEL: u8 = 17;
    }
}
