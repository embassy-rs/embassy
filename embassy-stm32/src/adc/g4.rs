use core::mem;

#[allow(unused)]
#[cfg(stm32h7)]
use pac::adc::vals::{Adcaldif, Difsel, Exten};
#[allow(unused)]
#[cfg(stm32g4)]
pub use pac::adc::vals::{Adcaldif, Difsel, Exten, Rovsm, Trovs};
pub use pac::adccommon::vals::Presc;
pub use stm32_metapac::adc::vals::{Adstp, Dmacfg, Dmaen};
pub use stm32_metapac::adccommon::vals::Dual;

use super::{Adc, AdcChannel, AnyAdcChannel, Instance, Resolution, RxDma, SampleTime, blocking_delay_us};
use crate::adc::SealedAdcChannel;
use crate::dma::Transfer;
use crate::time::Hertz;
use crate::{Peri, pac, rcc};

mod ringbuffered;
pub use ringbuffered::RingBufferedAdc;

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

// Trigger source for ADC conversions¨
#[derive(Copy, Clone)]
pub struct ConversionTrigger {
    // See Table 166 and 167 in RM0440 Rev 9 for ADC1/2 External triggers
    // Note that Injected and Regular channels uses different mappings
    pub channel: u8,
    pub edge: Exten,
}

// Conversion mode for regular ADC channels
#[derive(Copy, Clone)]
pub enum RegularConversionMode {
    // Samples as fast as possible
    Continuous,
    // Sample at rate determined by external trigger
    Triggered(ConversionTrigger),
}

impl<'d, T: Instance> Adc<'d, T> {
    /// Create a new ADC driver.
    pub fn new(adc: Peri<'d, T>) -> Self {
        rcc::enable_and_reset::<T>();

        let prescaler = Prescaler::from_ker_ck(T::frequency());

        T::common_regs().ccr().modify(|w| w.set_presc(prescaler.presc()));

        let frequency = Hertz(T::frequency().0 / prescaler.divisor());
        trace!("ADC frequency set to {}", frequency);

        if frequency > MAX_ADC_CLK_FREQ {
            panic!(
                "Maximal allowed frequency for the ADC is {} MHz and it varies with different packages, refer to ST docs for more information.",
                MAX_ADC_CLK_FREQ.0 / 1_000_000
            );
        }

        let mut s = Self { adc };
        s.power_up();
        s.configure_differential_inputs();

        s.calibrate();
        blocking_delay_us(1);

        Self::enable();
        s.configure();

        s
    }

    fn power_up(&mut self) {
        T::regs().cr().modify(|reg| {
            reg.set_deeppwd(false);
            reg.set_advregen(true);
        });

        blocking_delay_us(20);
    }

    fn configure_differential_inputs(&mut self) {
        T::regs().difsel().modify(|w| {
            for n in 0..18 {
                w.set_difsel(n, Difsel::SINGLE_ENDED);
            }
        });
    }

    fn calibrate(&mut self) {
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
    }

    fn enable() {
        // Make sure bits are off
        while T::regs().cr().read().addis() {
            // spin
        }

        if !T::regs().cr().read().aden() {
            // Enable ADC
            T::regs().isr().modify(|reg| {
                reg.set_adrdy(true);
            });
            T::regs().cr().modify(|reg| {
                reg.set_aden(true);
            });

            while !T::regs().isr().read().adrdy() {
                // spin
            }
        }
    }

    fn configure(&mut self) {
        // single conversion mode, software trigger
        T::regs().cfgr().modify(|w| {
            w.set_cont(false);
            w.set_exten(Exten::DISABLED);
        });
    }

    /// Enable reading the voltage reference internal channel.
    pub fn enable_vrefint(&self) -> super::VrefInt
    where
        T: super::VrefConverter,
    {
        T::common_regs().ccr().modify(|reg| {
            reg.set_vrefen(true);
        });

        super::VrefInt {}
    }

    /// Enable reading the temperature internal channel.
    pub fn enable_temperature(&self) -> super::Temperature
    where
        T: super::TemperatureConverter,
    {
        T::common_regs().ccr().modify(|reg| {
            reg.set_vsenseen(true);
        });

        super::Temperature {}
    }

    /// Enable reading the vbat internal channel.
    pub fn enable_vbat(&self) -> super::Vbat
    where
        T: super::VBatConverter,
    {
        T::common_regs().ccr().modify(|reg| {
            reg.set_vbaten(true);
        });

        super::Vbat {}
    }

    /// Enable differential channel.
    /// Caution:
    /// : When configuring the channel “i” in differential input mode, its negative input voltage VINN[i]
    /// is connected to another channel. As a consequence, this channel is no longer usable in
    /// single-ended mode or in differential mode and must never be configured to be converted.
    /// Some channels are shared between ADC1/ADC2/ADC3/ADC4/ADC5: this can make the
    /// channel on the other ADC unusable. The only exception is when ADC master and the slave
    /// operate in interleaved mode.
    #[cfg(stm32g4)]
    fn set_differential_channel(&mut self, ch: usize, enable: bool) {
        T::regs().cr().modify(|w| w.set_aden(false)); // disable adc
        T::regs().difsel().modify(|w| {
            w.set_difsel(
                ch,
                if enable {
                    Difsel::DIFFERENTIAL
                } else {
                    Difsel::SINGLE_ENDED
                },
            );
        });
        T::regs().cr().modify(|w| w.set_aden(true));
    }

    #[cfg(stm32g4)]
    pub fn set_differential(&mut self, channel: &mut impl AdcChannel<T>, enable: bool) {
        self.set_differential_channel(channel.channel() as usize, enable);
    }

    /// Set oversampling shift.
    #[cfg(stm32g4)]
    pub fn set_oversampling_shift(&mut self, shift: u8) {
        T::regs().cfgr2().modify(|reg| reg.set_ovss(shift));
    }

    /// Set oversampling ratio.
    #[cfg(stm32g4)]
    pub fn set_oversampling_ratio(&mut self, ratio: u8) {
        T::regs().cfgr2().modify(|reg| reg.set_ovsr(ratio));
    }

    /// Enable oversampling in regular mode.
    #[cfg(stm32g4)]
    pub fn enable_regular_oversampling_mode(&mut self, mode: Rovsm, trig_mode: Trovs, enable: bool) {
        T::regs().cfgr2().modify(|reg| reg.set_trovs(trig_mode));
        T::regs().cfgr2().modify(|reg| reg.set_rovsm(mode));
        T::regs().cfgr2().modify(|reg| reg.set_rovse(enable));
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

    /// Set the ADC resolution.
    pub fn set_resolution(&mut self, resolution: Resolution) {
        T::regs().cfgr().modify(|reg| reg.set_res(resolution.into()));
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

    /// Read an ADC pin.
    pub fn blocking_read(&mut self, channel: &mut impl AdcChannel<T>, sample_time: SampleTime) -> u16 {
        channel.setup();

        self.read_channel(channel, sample_time)
    }

    /// Start regular adc conversion
    pub(super) fn start() {
        T::regs().cr().modify(|reg| {
            reg.set_adstart(true);
        });
    }

    /// Stop regular conversions
    pub(super) fn stop() {
        Self::stop_regular_conversions();
    }

    /// Teardown method for stopping regular ADC conversions
    pub(super) fn teardown_dma() {
        Self::stop_regular_conversions();

        // Disable dma control
        T::regs().cfgr().modify(|reg| {
            reg.set_dmaen(Dmaen::DISABLE);
        });
    }

    /// Read one or multiple ADC regular channels using DMA.
    ///
    /// `sequence` iterator and `readings` must have the same length.
    ///
    /// Example
    /// ```rust,ignore
    /// use embassy_stm32::adc::{Adc, AdcChannel}
    ///
    /// let mut adc = Adc::new(p.ADC1);
    /// let mut adc_pin0 = p.PA0.into();
    /// let mut adc_pin1 = p.PA1.into();
    /// let mut measurements = [0u16; 2];
    ///
    /// adc.read(
    ///     p.DMA1_CH2.reborrow(),
    ///     [
    ///         (&mut *adc_pin0, SampleTime::CYCLES160_5),
    ///         (&mut *adc_pin1, SampleTime::CYCLES160_5),
    ///     ]
    ///     .into_iter(),
    ///     &mut measurements,
    /// )
    /// .await;
    /// defmt::info!("measurements: {}", measurements);
    /// ```
    ///
    /// Note: This is not very efficient as the ADC needs to be reconfigured for each read. Use
    /// `into_ring_buffered`, `into_ring_buffered_and_injected`
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

        // Ensure no conversions are ongoing and ADC is enabled.
        Self::stop_regular_conversions();
        Self::enable();

        Self::configure_sequence(sequence.map(|(channel, sample_time)| {
            channel.setup();

            (channel.channel, sample_time)
        }));

        // Set continuous mode with oneshot dma.
        // Clear overrun flag before starting transfer.
        T::regs().isr().modify(|reg| {
            reg.set_ovr(true);
        });

        T::regs().cfgr().modify(|reg| {
            reg.set_discen(false);
            reg.set_cont(true);
            reg.set_dmacfg(Dmacfg::ONE_SHOT);
            reg.set_dmaen(Dmaen::ENABLE);
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
        Self::stop_regular_conversions();

        // Reset configuration.
        T::regs().cfgr().modify(|reg| {
            reg.set_cont(false);
        });
    }

    pub(super) fn configure_sequence(sequence: impl ExactSizeIterator<Item = (u8, SampleTime)>) {
        // Set sequence length
        T::regs().sqr1().modify(|w| {
            w.set_l(sequence.len() as u8 - 1);
        });

        // Configure channels and ranks
        for (_i, (ch, sample_time)) in sequence.enumerate() {
            let sample_time = sample_time.into();
            if ch <= 9 {
                T::regs().smpr().modify(|reg| reg.set_smp(ch as _, sample_time));
            } else {
                T::regs().smpr2().modify(|reg| reg.set_smp((ch - 10) as _, sample_time));
            }

            match _i {
                0..=3 => {
                    T::regs().sqr1().modify(|w| {
                        w.set_sq(_i, ch);
                    });
                }
                4..=8 => {
                    T::regs().sqr2().modify(|w| {
                        w.set_sq(_i - 4, ch);
                    });
                }
                9..=13 => {
                    T::regs().sqr3().modify(|w| {
                        w.set_sq(_i - 9, ch);
                    });
                }
                14..=15 => {
                    T::regs().sqr4().modify(|w| {
                        w.set_sq(_i - 14, ch);
                    });
                }
                _ => unreachable!(),
            }
        }
    }

    /// Set external trigger for regular conversion sequence
    fn set_regular_conversion_trigger(&mut self, trigger: ConversionTrigger) {
        T::regs().cfgr().modify(|r| {
            r.set_extsel(trigger.channel);
            r.set_exten(trigger.edge);
        });
        // Regular conversions uses DMA so no need to generate interrupt
        T::regs().ier().modify(|r| r.set_eosie(false));
    }

    // Dual ADC mode selection
    pub fn configure_dual_mode(&mut self, val: Dual) {
        T::common_regs().ccr().modify(|reg| {
            reg.set_dual(val);
        })
    }

    /// Configures the ADC to use a DMA ring buffer for continuous data acquisition.
    ///
    /// Use the [`read`] method to retrieve measurements from the DMA ring buffer. The read buffer
    /// should be exactly half the size of `dma_buf`. When using triggered mode, it is recommended
    /// to configure `dma_buf` as a double buffer so that one half can be read while the other half
    /// is being filled by the DMA, preventing data loss. The trigger period of the ADC effectively
    /// defines the period at which the buffer should be read.
    ///
    /// If continous conversion mode is selected, the provided `dma_buf` must be large enough to prevent
    /// DMA buffer overruns. Its length should be a multiple of the number of ADC channels being measured.
    /// For example, if 3 channels are measured and you want to store 40 samples per channel,
    /// the buffer length should be `3 * 40 = 120`.
    ///
    /// # Parameters
    /// - `dma`: The DMA peripheral used to transfer ADC data into the buffer.
    /// - `dma_buf`: The buffer where DMA stores ADC samples.
    /// - `regular_sequence`: Sequence of channels and sample times for regular ADC conversions.
    /// - `regular_conversion_mode`: Mode for regular conversions (continuous or triggered).
    ///
    /// # Returns
    /// A `RingBufferedAdc<'a, T>` instance configured for continuous DMA-based sampling.
    pub fn into_ring_buffered<'a>(
        mut self,
        dma: Peri<'a, impl RxDma<T>>,
        dma_buf: &'a mut [u16],
        sequence: impl ExactSizeIterator<Item = (AnyAdcChannel<T>, SampleTime)>,
        mode: RegularConversionMode,
    ) -> RingBufferedAdc<'a, T> {
        assert!(!dma_buf.is_empty() && dma_buf.len() <= 0xFFFF);
        assert!(sequence.len() != 0, "Asynchronous read sequence cannot be empty");
        assert!(
            sequence.len() <= 16,
            "Asynchronous read sequence cannot be more than 16 in length"
        );
        // reset conversions and enable the adc
        Self::stop_regular_conversions();
        Self::enable();

        //adc side setup

        Self::configure_sequence(sequence.map(|(mut channel, sample_time)| {
            channel.setup();

            (channel.channel, sample_time)
        }));

        // Clear overrun flag before starting transfer.
        T::regs().isr().modify(|reg| {
            reg.set_ovr(true);
        });

        T::regs().cfgr().modify(|reg| {
            reg.set_discen(false); // Convert all channels for each trigger
            reg.set_dmacfg(Dmacfg::CIRCULAR);
            reg.set_dmaen(Dmaen::ENABLE);
        });

        match mode {
            RegularConversionMode::Continuous => {
                T::regs().cfgr().modify(|reg| {
                    reg.set_cont(true);
                });
            }
            RegularConversionMode::Triggered(trigger) => {
                T::regs().cfgr().modify(|r| {
                    r.set_cont(false); // New trigger is neede for each sample to be read
                });
                self.set_regular_conversion_trigger(trigger);
            }
        }

        mem::forget(self);

        RingBufferedAdc::new(dma, dma_buf)
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
        mut self,
        sequence: [(AnyAdcChannel<T>, SampleTime); N],
        trigger: ConversionTrigger,
        interrupt: bool,
    ) -> InjectedAdc<T, N> {
        assert!(N != 0, "Read sequence cannot be empty");
        assert!(
            N <= NR_INJECTED_RANKS,
            "Read sequence cannot be more than {} in length",
            NR_INJECTED_RANKS
        );

        Self::stop_regular_conversions();
        Self::enable();

        T::regs().jsqr().modify(|w| w.set_jl(N as u8 - 1));

        for (n, (mut channel, sample_time)) in sequence.into_iter().enumerate() {
            Self::configure_channel(&mut channel, sample_time);

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

        self.set_injected_conversion_trigger(trigger);
        self.enable_injected_eos_interrupt(interrupt);
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
    pub fn into_ring_buffered_and_injected<'a, const N: usize>(
        self,
        dma: Peri<'a, impl RxDma<T>>,
        dma_buf: &'a mut [u16],
        regular_sequence: impl ExactSizeIterator<Item = (AnyAdcChannel<T>, SampleTime)>,
        regular_conversion_mode: RegularConversionMode,
        injected_sequence: [(AnyAdcChannel<T>, SampleTime); N],
        injected_trigger: ConversionTrigger,
        injected_interrupt: bool,
    ) -> (RingBufferedAdc<'a, T>, InjectedAdc<T, N>) {
        unsafe {
            (
                Self {
                    adc: self.adc.clone_unchecked(),
                }
                .into_ring_buffered(dma, dma_buf, regular_sequence, regular_conversion_mode),
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

    /// Set external trigger for injected conversion sequence
    /// Possible trigger values are seen in Table 167 in RM0440 Rev 9
    fn set_injected_conversion_trigger(&mut self, trigger: ConversionTrigger) {
        T::regs().jsqr().modify(|r| {
            r.set_jextsel(trigger.channel);
            r.set_jexten(trigger.edge);
        });
    }

    /// Enable end of injected sequence interrupt
    fn enable_injected_eos_interrupt(&mut self, enable: bool) {
        T::regs().ier().modify(|r| r.set_jeosie(enable));
    }

    fn configure_channel(channel: &mut impl AdcChannel<T>, sample_time: SampleTime) {
        // Configure channel
        Self::set_channel_sample_time(channel.channel(), sample_time);
    }

    fn read_channel(&mut self, channel: &mut impl AdcChannel<T>, sample_time: SampleTime) -> u16 {
        Self::configure_channel(channel, sample_time);
        #[cfg(stm32h7)]
        {
            T::regs().cfgr2().modify(|w| w.set_lshift(0));
            T::regs()
                .pcsel()
                .write(|w| w.set_pcsel(channel.channel() as _, Pcsel::PRESELECTED));
        }

        T::regs().sqr1().write(|reg| {
            reg.set_sq(0, channel.channel());
            reg.set_l(0);
        });

        self.convert()
    }

    fn set_channel_sample_time(ch: u8, sample_time: SampleTime) {
        let sample_time = sample_time.into();
        if ch <= 9 {
            T::regs().smpr().modify(|reg| reg.set_smp(ch as _, sample_time));
        } else {
            T::regs().smpr2().modify(|reg| reg.set_smp((ch - 10) as _, sample_time));
        }
    }

    // Stop regular conversions
    fn stop_regular_conversions() {
        if T::regs().cr().read().adstart() && !T::regs().cr().read().addis() {
            T::regs().cr().modify(|reg| {
                reg.set_adstp(Adstp::STOP);
            });
            // The software must poll ADSTART until the bit is reset before assuming the
            // ADC is completely stopped
            while T::regs().cr().read().adstart() {}
        }
    }
}

impl<T: Instance, const N: usize> InjectedAdc<T, N> {
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
    use crate::adc::{TemperatureConverter, VBatConverter, VrefConverter};

    impl TemperatureConverter for crate::peripherals::ADC1 {
        const CHANNEL: u8 = 16;
    }

    impl VrefConverter for crate::peripherals::ADC1 {
        const CHANNEL: u8 = 18;
    }

    impl VBatConverter for crate::peripherals::ADC1 {
        const CHANNEL: u8 = 17;
    }

    #[cfg(peri_adc3_common)]
    impl VrefConverter for crate::peripherals::ADC3 {
        const CHANNEL: u8 = 18;
    }

    #[cfg(peri_adc3_common)]
    impl VBatConverter for crate::peripherals::ADC3 {
        const CHANNEL: u8 = 17;
    }

    #[cfg(not(stm32g4x1))]
    impl VrefConverter for crate::peripherals::ADC4 {
        const CHANNEL: u8 = 18;
    }

    #[cfg(not(stm32g4x1))]
    impl TemperatureConverter for crate::peripherals::ADC5 {
        const CHANNEL: u8 = 4;
    }

    #[cfg(not(stm32g4x1))]
    impl VrefConverter for crate::peripherals::ADC5 {
        const CHANNEL: u8 = 18;
    }

    #[cfg(not(stm32g4x1))]
    impl VBatConverter for crate::peripherals::ADC5 {
        const CHANNEL: u8 = 17;
    }
}

// TODO this should look at each ADC individually and impl the correct channels
#[cfg(stm32h7)]
mod h7 {
    impl<T: Instance> TemperatureConverter for T {
        const CHANNEL: u8 = 18;
    }
    impl<T: Instance> VrefConverter for T {
        const CHANNEL: u8 = 19;
    }
    impl<T: Instance> VBatConverter for T {
        // TODO this should be 14 for H7a/b/35
        const CHANNEL: u8 = 17;
    }
}
