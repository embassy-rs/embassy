use core::mem;
use core::sync::atomic::{Ordering, compiler_fence};

use super::{AnyAdcChannel, ConversionMode, Temperature, Vbat, VrefInt, blocking_delay_us};
use crate::adc::{
    Adc, AdcRegs, BasicAdcRegs, InjectedTrigger, Instance, RegularTrigger, Resolution, RxDma, SampleTime,
    SealedAdcChannel,
};
use crate::pac::adc::vals;
pub use crate::pac::adccommon::vals::Adcpre;
use crate::time::Hertz;
use crate::{Peri, rcc};

mod injected;
pub use injected::InjectedAdc;

fn clear_interrupt_flags(r: crate::pac::adc::Adc) {
    r.sr().modify(|regs| {
        regs.set_eoc(false);
        regs.set_ovr(false);
    });
}

/// Default VREF voltage used for sample conversion to millivolts.
pub const VREF_DEFAULT_MV: u32 = 3300;
/// VREF voltage used for factory calibration of VREFINTCAL register.
pub const VREF_CALIB_MV: u32 = 3300;

const NR_INJECTED_RANKS: usize = 4;

impl super::SealedSpecialConverter<super::VrefInt> for crate::peripherals::ADC1 {
    const CHANNEL: u8 = 17;
}

#[cfg(any(stm32f2, stm32f40x, stm32f41x))]
impl super::SealedSpecialConverter<super::Temperature> for crate::peripherals::ADC1 {
    const CHANNEL: u8 = 16;
}

#[cfg(not(any(stm32f2, stm32f40x, stm32f41x)))]
impl super::SealedSpecialConverter<super::Temperature> for crate::peripherals::ADC1 {
    const CHANNEL: u8 = 18;
}

impl super::SealedSpecialConverter<super::Vbat> for crate::peripherals::ADC1 {
    const CHANNEL: u8 = 18;
}

impl VrefInt {
    /// Time needed for internal voltage reference to stabilize
    pub fn start_time_us() -> u32 {
        10
    }
}

impl Temperature {
    /// Time needed for temperature sensor readings to stabilize
    pub fn start_time_us() -> u32 {
        10
    }
}

fn from_pclk2(freq: Hertz) -> Adcpre {
    // Datasheet for F2 specifies min frequency 0.6 MHz, and max 30 MHz (with VDDA 2.4-3.6V).
    #[cfg(stm32f2)]
    const MAX_FREQUENCY: Hertz = Hertz(30_000_000);
    // Datasheet for both F4 and F7 specifies min frequency 0.6 MHz, typ freq. 30 MHz and max 36 MHz.
    #[cfg(not(stm32f2))]
    const MAX_FREQUENCY: Hertz = Hertz(36_000_000);
    let raw_div = rcc::raw_prescaler(freq.0, MAX_FREQUENCY.0);
    match raw_div {
        0..=1 => Adcpre::DIV2,
        2..=3 => Adcpre::DIV4,
        4..=5 => Adcpre::DIV6,
        6..=7 => Adcpre::DIV8,
        _ => panic!("Selected PCLK2 frequency is too high for ADC with largest possible prescaler."),
    }
}

/// ADC configuration
#[derive(Default)]
pub struct AdcConfig {
    pub resolution: Option<Resolution>,
}

impl super::AdcRegs for crate::pac::adc::Adc {
    fn data(&self) -> *mut u16 {
        crate::pac::adc::Adc::dr(*self).as_ptr() as *mut u16
    }

    fn enable(&self) {
        self.cr2().modify(|reg| {
            reg.set_adon(true);
        });

        blocking_delay_us(3);
    }

    fn start(&self) {
        // Begin ADC conversions
        self.cr2().modify(|reg| {
            reg.set_swstart(true);
        });
    }

    fn stop(&self) {
        let r = self;

        // Stop ADC
        r.cr2().modify(|reg| {
            // Stop ADC
            reg.set_swstart(false);
            // Stop ADC
            reg.set_adon(false);
            // Stop DMA
            reg.set_dma(false);
        });

        r.cr1().modify(|w| {
            // Disable interrupt for end of conversion
            w.set_eocie(false);
            // Disable interrupt for overrun
            w.set_ovrie(false);
        });

        clear_interrupt_flags(*r);

        compiler_fence(Ordering::SeqCst);
    }

    fn convert(&self) {
        // clear end of conversion flag
        self.sr().modify(|reg| {
            reg.set_eoc(false);
        });

        // Start conversion
        self.cr2().modify(|reg| {
            reg.set_swstart(true);
        });

        while self.sr().read().strt() == false {
            // spin //wait for actual start
        }
        while self.sr().read().eoc() == false {
            // spin //wait for finish
        }
    }

    fn configure_dma(&self, conversion_mode: ConversionMode) {
        match conversion_mode {
            ConversionMode::Repeated(trigger) => {
                let r = self;
                // Clear all interrupts
                r.sr().modify(|regs| {
                    regs.set_eoc(false);
                    regs.set_ovr(false);
                    regs.set_strt(false);
                });

                r.cr1().modify(|w| {
                    // Enable interrupt for end of conversion
                    w.set_eocie(true);
                    // Enable interrupt for overrun
                    w.set_ovrie(true);
                    // Scanning conversions of multiple channels
                    w.set_scan(true);
                    // Continuous conversion mode
                    w.set_discen(false);
                });

                r.cr2().modify(|w| {
                    // Enable DMA mode
                    w.set_dma(true);
                    // DMA requests are issues as long as DMA=1 and data are converted.
                    w.set_dds(vals::Dds::CONTINUOUS);
                    // EOC flag is set at the end of each conversion.
                    w.set_eocs(vals::Eocs::EACH_CONVERSION);
                });

                match trigger.signal {
                    u8::MAX => {
                        // continuous conversion
                        r.cr2().modify(|w| {
                            // Enable continuous conversions
                            w.set_cont(true);
                        });
                    }
                    _ => {
                        r.cr2().modify(|w| {
                            // Disable continuous conversions
                            w.set_cont(false);
                            // Trigger detection edge
                            w.set_exten(trigger.edge);
                            // Trigger channel
                            w.set_extsel(trigger.signal);
                        })
                    }
                };
            }
        }
    }

    fn configure_sequence(&self, sequence: impl ExactSizeIterator<Item = ((u8, bool), SampleTime)>) {
        self.cr2().modify(|reg| {
            reg.set_adon(true);
        });

        // Check the sequence is long enough
        self.sqr1().modify(|r| {
            r.set_l((sequence.len() - 1).try_into().unwrap());
        });

        for (i, ((ch, _), sample_time)) in sequence.enumerate() {
            // Set the channel in the right sequence field.
            self.sqr3().modify(|w| w.set_sq(i, ch));

            let sample_time = sample_time.into();
            if ch <= 9 {
                self.smpr2().modify(|reg| reg.set_smp(ch as _, sample_time));
            } else {
                self.smpr1().modify(|reg| reg.set_smp((ch - 10) as _, sample_time));
            }
        }
    }
}

impl<'d, T> Adc<'d, T>
where
    T: Instance<Regs = crate::pac::adc::Adc>,
{
    pub fn new(adc: Peri<'d, T>) -> Self {
        Self::new_with_config(adc, Default::default())
    }

    pub fn new_with_config(adc: Peri<'d, T>, config: AdcConfig) -> Self {
        rcc::enable_and_reset::<T>();

        let presc = from_pclk2(T::frequency());
        T::common_regs().ccr().modify(|w| w.set_adcpre(presc));
        T::regs().enable();

        if let Some(resolution) = config.resolution {
            T::regs().cr1().modify(|reg| reg.set_res(resolution.into()));
        }

        Self { adc }
    }

    /// Enables internal voltage reference and returns [VrefInt], which can be used in
    /// [Adc::read_internal()] to perform conversion.
    pub fn enable_vrefint(&self) -> VrefInt {
        T::common_regs().ccr().modify(|reg| {
            reg.set_tsvrefe(true);
        });

        VrefInt {}
    }

    /// Enables internal temperature sensor and returns [Temperature], which can be used in
    /// [Adc::read_internal()] to perform conversion.
    ///
    /// On STM32F42 and STM32F43 this can not be used together with [Vbat]. If both are enabled,
    /// temperature sensor will return vbat value.
    pub fn enable_temperature(&self) -> Temperature {
        T::common_regs().ccr().modify(|reg| {
            reg.set_tsvrefe(true);
        });

        Temperature {}
    }

    /// Enables vbat input and returns [Vbat], which can be used in
    /// [Adc::read_internal()] to perform conversion.
    pub fn enable_vbat(&self) -> Vbat {
        T::common_regs().ccr().modify(|reg| {
            reg.set_vbate(true);
        });

        Vbat {}
    }
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
        trigger: impl InjectedTrigger<T>,
        edge: vals::Exten,
        interrupt: bool,
    ) -> InjectedAdc<'a, T, N> {
        assert!(N != 0, "Read sequence cannot be empty");
        assert!(
            N <= NR_INJECTED_RANKS,
            "Read sequence cannot be more than {} in length",
            NR_INJECTED_RANKS
        );

        T::regs().enable();

        T::regs().cr1().modify(|w| w.set_jauto(false));
        // Set injected sequence length
        T::regs().jsqr().modify(|w| w.set_jl(N as u8 - 1));

        for (n, (channel, sample_time)) in sequence.iter().enumerate() {
            let sample_time = sample_time.clone().into();
            if channel.channel() <= 9 {
                T::regs()
                    .smpr2()
                    .modify(|reg| reg.set_smp(channel.channel() as _, sample_time));
            } else {
                T::regs()
                    .smpr1()
                    .modify(|reg| reg.set_smp((channel.channel() - 10) as _, sample_time));
            }

            // On adc_v2/F4, injected JSQ rank field placement depends on the
            // programmed sequence length (JL). ST's HAL uses:
            //   shift = 5 * ((rank + 3) - sequence_len)
            // with rank starting at 1.
            let idx = n + (4 - N);

            T::regs().jsqr().modify(|w| w.set_jsq(idx, channel.channel()));
        }

        T::regs().cr1().modify(|w| {
            w.set_scan(true);
            w.set_jdiscen(false);
            w.set_jeocie(interrupt);
        });
        T::regs().cr2().modify(|w| {
            w.set_jextsel(trigger.signal());
            w.set_jexten(edge);
        });

        Self::start_injected_conversions();

        mem::forget(self);

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
        regular_trigger: impl RegularTrigger<T>,
        regular_edge: vals::Exten,
        injected_sequence: [(AnyAdcChannel<'b, T>, SampleTime); N],
        injected_trigger: impl InjectedTrigger<T>,
        injected_edge: vals::Exten,
        injected_interrupt: bool,
    ) -> (super::RingBufferedAdc<'a, T>, InjectedAdc<'b, T, N>) {
        unsafe {
            (
                Self {
                    adc: self.adc.clone_unchecked(),
                }
                .into_ring_buffered(
                    dma,
                    dma_buf,
                    _irq,
                    regular_sequence,
                    regular_trigger,
                    regular_edge,
                ),
                Self {
                    adc: self.adc.clone_unchecked(),
                }
                .setup_injected_conversions(
                    injected_sequence,
                    injected_trigger,
                    injected_edge,
                    injected_interrupt,
                ),
            )
        }
    }

    /// Stop injected conversions
    pub(super) fn stop_injected_conversions() {
        // No true "abort injected conversion" primitive on adc_v2.
        // Best practical stop: disable external injected triggering.
        T::regs().cr2().modify(|w| w.set_jexten(vals::Exten::DISABLED));
        T::regs().cr1().modify(|w| w.set_jeocie(false));
        T::regs().sr().modify(|w| {
            w.set_jeoc(false);
            w.set_jstrt(false);
        });
    }
    /// Start injected ADC conversion
    pub(super) fn start_injected_conversions() {
        T::regs().sr().modify(|w| {
            w.set_jeoc(false);
            w.set_jstrt(false);
        });

        // On STM32F4 adc_v2, externally-triggered injected conversions are armed
        // by JEXTEN and start on the next trigger event. JSWSTART is only valid
        // for pure software-triggered injected conversions.
        if T::regs().cr2().read().jexten() == vals::Exten::DISABLED {
            T::regs().cr2().modify(|w| w.set_jswstart(true));
        }
    }
}
impl<'a, T: Instance<Regs = crate::pac::adc::Adc>, const N: usize> InjectedAdc<'a, T, N> {
    /// Read sampled data from all injected ADC injected ranks
    /// Clear the JEOC and JSTRT flags to allow a new injected sequence
    pub(super) fn read_injected_data() -> [u16; N] {
        let mut data = [0u16; N];
        for i in 0..N {
            data[i] = T::regs().jdr(i).read().jdata();
        }

        // Clear JEOC and JSTRT
        T::regs().sr().modify(|w| {
            w.set_jeoc(false);
            w.set_jstrt(false);
        });
        data
    }
}

impl<'d, T: Instance> Drop for Adc<'d, T> {
    fn drop(&mut self) {
        T::regs().stop();

        rcc::disable::<T>();
    }
}
