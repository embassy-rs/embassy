use pac::adc::vals::Scandir;
#[allow(unused)]
use pac::adc::vals::{Adstp, Align, Ckmode, Dmacfg, Exten, Ovrmod, Ovsr};
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

const MAX_ADC_CLK_FREQ: Hertz = Hertz::mhz(25);

const TIME_ADC_VOLTAGE_REGUALTOR_STARTUP_US: u32 = 20;

const TEMP_CHANNEL: u8 = 9;
const VREF_CHANNEL: u8 = 10;

const NUM_HW_CHANNELS: u8 = 22;
const CHSELR_SQ_SIZE: usize = 8;
const CHSELR_SQ_MAX_CHANNEL: u8 = 14;
const CHSELR_SQ_SEQUENCE_END_MARKER: u8 = 0b1111;

// NOTE: Vrefint/Temperature/Vbat are not available on all ADCs,
// this currently cannot be modeled with stm32-data,
// so these are available from the software on all ADCs.
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

#[derive(Debug)]
pub enum Prescaler {
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

    #[allow(unused)]
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

#[cfg(feature = "defmt")]
impl<'a> defmt::Format for Prescaler {
    fn format(&self, fmt: defmt::Formatter) {
        match self {
            Prescaler::NotDivided => defmt::write!(fmt, "Prescaler::NotDivided"),
            Prescaler::DividedBy2 => defmt::write!(fmt, "Prescaler::DividedBy2"),
            Prescaler::DividedBy4 => defmt::write!(fmt, "Prescaler::DividedBy4"),
            Prescaler::DividedBy6 => defmt::write!(fmt, "Prescaler::DividedBy6"),
            Prescaler::DividedBy8 => defmt::write!(fmt, "Prescaler::DividedBy8"),
            Prescaler::DividedBy10 => defmt::write!(fmt, "Prescaler::DividedBy10"),
            Prescaler::DividedBy12 => defmt::write!(fmt, "Prescaler::DividedBy12"),
            Prescaler::DividedBy16 => defmt::write!(fmt, "Prescaler::DividedBy16"),
            Prescaler::DividedBy32 => defmt::write!(fmt, "Prescaler::DividedBy32"),
            Prescaler::DividedBy64 => defmt::write!(fmt, "Prescaler::DividedBy64"),
            Prescaler::DividedBy128 => defmt::write!(fmt, "Prescaler::DividedBy128"),
            Prescaler::DividedBy256 => defmt::write!(fmt, "Prescaler::DividedBy256"),
        }
    }
}

/// Number of samples used for averaging.
/// TODO: Implement hardware averaging setting.
#[allow(unused)]
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
    pub fn new(adc: Peri<'d, T>, sample_time: SampleTime, resolution: Resolution) -> Self {
        rcc::enable_and_reset::<T>();

        T::regs().cfgr2().modify(|w| w.set_ckmode(Ckmode::SYSCLK));

        let prescaler = Prescaler::from_ker_ck(T::frequency());
        T::common_regs().ccr().modify(|w| w.set_presc(prescaler.presc()));

        let frequency = Hertz(T::frequency().0 / prescaler.divisor());
        debug!("ADC frequency set to {}", frequency);

        if frequency > MAX_ADC_CLK_FREQ {
            panic!("Maximal allowed frequency for the ADC is {} MHz and it varies with different packages, refer to ST docs for more information.", MAX_ADC_CLK_FREQ.0 /  1_000_000 );
        }

        let mut s = Self {
            adc,
            sample_time: SampleTime::from_bits(0),
        };

        s.power_up();

        s.set_resolution(resolution);

        s.calibrate();

        s.enable();

        s.configure_default();

        s.set_sample_time_all_channels(sample_time);

        s
    }

    fn power_up(&mut self) {
        T::regs().cr().modify(|reg| {
            reg.set_advregen(true);
        });

        // "The software must wait for the ADC voltage regulator startup time."
        // See datasheet for the value.
        blocking_delay_us(TIME_ADC_VOLTAGE_REGUALTOR_STARTUP_US + 1);
    }

    fn calibrate(&mut self) {
        // We have to make sure AUTOFF is OFF, but keep its value after calibration.
        let autoff_value = T::regs().cfgr1().read().autoff();
        T::regs().cfgr1().modify(|w| w.set_autoff(false));

        T::regs().cr().modify(|w| w.set_adcal(true));

        // "ADCAL bit stays at 1 during all the calibration sequence."
        // "It is then cleared by hardware as soon the calibration completes."
        while T::regs().cr().read().adcal() {}

        debug!("ADC calibration value: {}.", T::regs().dr().read().data());

        T::regs().cfgr1().modify(|w| w.set_autoff(autoff_value));
    }

    fn enable(&mut self) {
        T::regs().isr().modify(|w| w.set_adrdy(true));
        T::regs().cr().modify(|w| w.set_aden(true));
        // ADRDY is "ADC ready". Wait until it will be True.
        while !T::regs().isr().read().adrdy() {}
    }

    fn configure_default(&mut self) {
        // single conversion mode, software trigger
        T::regs().cfgr1().modify(|w| {
            w.set_cont(false);
            w.set_exten(Exten::DISABLED);
            w.set_align(Align::RIGHT);
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
        debug!("Ensure that sample time is set to more than temperature sensor T_start from the datasheet!");
        T::common_regs().ccr().modify(|reg| {
            reg.set_tsen(true);
        });

        Temperature {}
    }

    /// Set the ADC sample time.
    /// Shall only be called when ADC is not converting.
    pub fn set_sample_time_all_channels(&mut self, sample_time: SampleTime) {
        self.sample_time = sample_time;

        // Set all channels to use SMP1 field as source.
        T::regs().smpr().modify(|w| {
            w.smpsel(0);
            w.set_smp1(sample_time);
        });
    }

    /// Set the ADC resolution.
    pub fn set_resolution(&mut self, resolution: Resolution) {
        T::regs().cfgr1().modify(|reg| reg.set_res(resolution));
    }

    /// Perform a single conversion.
    fn convert(&mut self) -> u16 {
        // Set single conversion mode.
        T::regs().cfgr1().modify(|w| w.set_cont(false));

        // Start conversion
        T::regs().cr().modify(|reg| {
            reg.set_adstart(true);
        });

        // Waiting for End Of Conversion (EOC).
        while !T::regs().isr().read().eoc() {}

        T::regs().dr().read().data() as u16
    }

    pub fn blocking_read(&mut self, channel: &mut impl AdcChannel<T>) -> u16 {
        Self::configure_channel(channel);
        T::regs().cfgr1().write(|reg| {
            reg.set_chselrmod(false);
            reg.set_align(Align::RIGHT);
        });
        self.convert()
    }

    fn setup_channel_sequencer<'a>(channel_sequence: impl ExactSizeIterator<Item = &'a mut AnyAdcChannel<T>>) {
        assert!(
            channel_sequence.len() <= CHSELR_SQ_SIZE,
            "Seqenced read set cannot be more than {} in size.",
            CHSELR_SQ_SIZE
        );
        let mut last_sq_set: usize = 0;
        T::regs().chselr_sq().write(|w| {
            for (i, channel) in channel_sequence.enumerate() {
                assert!(
                    channel.channel() <= CHSELR_SQ_MAX_CHANNEL,
                    "Sequencer only support HW channels smaller than {}.",
                    CHSELR_SQ_MAX_CHANNEL
                );
                w.set_sq(i, channel.channel());
                last_sq_set = i;
            }

            for i in (last_sq_set + 1)..CHSELR_SQ_SIZE {
                w.set_sq(i, CHSELR_SQ_SEQUENCE_END_MARKER);
            }
        });

        Self::apply_channel_conf()
    }

    async fn dma_convert(&mut self, rx_dma: Peri<'_, impl RxDma<T>>, readings: &mut [u16]) {
        // Enable overrun control, so no new DMA requests will be generated until
        // previous DR values is read.
        T::regs().isr().modify(|reg| {
            reg.set_ovr(true);
        });

        // Set continuous mode with oneshot dma.
        T::regs().cfgr1().modify(|reg| {
            reg.set_discen(false);
            reg.set_cont(true);
            reg.set_dmacfg(Dmacfg::DMA_ONE_SHOT);
            reg.set_dmaen(true);
            reg.set_ovrmod(Ovrmod::PRESERVE);
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

        // Start conversion.
        T::regs().cr().modify(|reg| {
            reg.set_adstart(true);
        });

        // Wait for conversion sequence to finish.
        transfer.await;

        // Ensure conversions are finished.
        Self::cancel_conversions();

        // Reset configuration.
        T::regs().cfgr1().modify(|reg| {
            reg.set_cont(false);
            reg.set_dmacfg(Dmacfg::from_bits(0));
            reg.set_dmaen(false);
        });
    }

    /// Read one or multiple ADC channels using DMA in hardware order.
    /// Readings will be ordered based on **hardware** ADC channel number and `scandir` setting.
    /// Readings won't be in the same order as in the `set`!
    ///
    /// In STM32C0, channels bigger than 14 cannot be read using sequencer, so you have to use
    /// either blocking reads or use the mechanism to read in HW order (CHSELRMOD=0).
    /// TODO(chudsaviet): externalize generic code and merge with read().
    pub async fn read_in_hw_order(
        &mut self,
        rx_dma: Peri<'_, impl RxDma<T>>,
        hw_channel_selection: u32,
        scandir: Scandir,
        readings: &mut [u16],
    ) {
        assert!(
            hw_channel_selection != 0,
            "Some bits in `hw_channel_selection` shall be set."
        );
        assert!(
            (hw_channel_selection >> NUM_HW_CHANNELS) == 0,
            "STM32C0 only have {} ADC channels. `hw_channel_selection` cannot have bits higher than this number set.",
            NUM_HW_CHANNELS
        );
        // To check for correct readings slice size, we shall solve Hamming weight problem,
        // which is either slow or memory consuming.
        // Since we have limited resources, we don't do it here.
        // Not doing this have a great potential for a bug through.

        // Ensure no conversions are ongoing.
        Self::cancel_conversions();

        T::regs().cfgr1().modify(|reg| {
            reg.set_chselrmod(false);
            reg.set_scandir(scandir);
            reg.set_align(Align::RIGHT);
        });

        // Set required channels for multi-convert.
        unsafe { (T::regs().chselr().as_ptr() as *mut u32).write_volatile(hw_channel_selection) }

        Self::apply_channel_conf();

        self.dma_convert(rx_dma, readings).await
    }

    // Read ADC channels in specified order using DMA (CHSELRMOD = 1).
    // In STM32C0, only lower 14 ADC channels can be read this way.
    // For other channels, use `read_in_hw_order()` or blocking read.
    pub async fn read(
        &mut self,
        rx_dma: Peri<'_, impl RxDma<T>>,
        channel_sequence: impl ExactSizeIterator<Item = &mut AnyAdcChannel<T>>,
        readings: &mut [u16],
    ) {
        assert!(
            channel_sequence.len() != 0,
            "Asynchronous read channel sequence cannot be empty."
        );
        assert!(
            channel_sequence.len() == readings.len(),
            "Channel sequence length must be equal to readings length."
        );

        // Ensure no conversions are ongoing.
        Self::cancel_conversions();

        T::regs().cfgr1().modify(|reg| {
            reg.set_chselrmod(true);
            reg.set_align(Align::RIGHT);
        });

        Self::setup_channel_sequencer(channel_sequence);

        self.dma_convert(rx_dma, readings).await
    }

    fn configure_channel(channel: &mut impl AdcChannel<T>) {
        channel.setup();
        // write() because we want all other bits to be set to 0.
        T::regs()
            .chselr()
            .write(|w| w.set_chsel(channel.channel().into(), true));

        Self::apply_channel_conf();
    }

    fn apply_channel_conf() {
        // Trigger and wait for the channel selection procedure to complete.
        T::regs().isr().modify(|w| w.set_ccrdy(false));
        while !T::regs().isr().read().ccrdy() {}
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
