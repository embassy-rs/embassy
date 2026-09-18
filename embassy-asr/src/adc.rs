//! Analog-to-digital converter (ADC).
//!
//! Register programming follows the vendor `tremo_adc` driver and the SDK
//! `adc/single_mode` / `adc/continue_mode` examples. Analog power-up and
//! reference selection use the AFEC analog window via [`crate::afec::analog`].
//!
//! # Documented external channels
//!
//! Datasheet GPIO mux (Analog column) and SDK sample-channel numbering
//! (`ADC_SAMPLE_CHAN_0` is reserved, so `ADC_INn` maps to sample channel
//! `n + 1`):
//!
//! | Datasheet | Pin   | [`SampleChannel`] |
//! |-----------|-------|-------------------|
//! | ADC_IN0   | PA11  | [`SampleChannel::In0`] |
//! | ADC_IN1   | PA8   | [`SampleChannel::In1`] |
//! | ADC_IN2   | PA5   | [`SampleChannel::In2`] |
//! | ADC_IN3   | PA4   | [`SampleChannel::In3`] |
//! | ADC_IN7   | PC15  | [`SampleChannel::In7`] |
//!
//! Pins are placed in GPIO analog mode (vendor `GPIO_MODE_ANALOG`) with AF0.
//! The ADC path does not use a non-zero alternate-function number.
//!
//! # Calibration
//!
//! Factory gain/offset constants live at `0x1000_2030` / `0x1000_2034` (not in
//! the PAC). Convert raw codes with [`Calibration::to_voltage`] using the
//! internal 1.2 V reference assumed by the SDK examples.
//!
//! # DMA
//!
//! [`Adc::set_dma_enabled`] only toggles `CFGR.DMAEN`. Applications that need
//! DMA should configure a peripheral-to-memory channel whose source is
//! [`Adc::dr_address`] and whose request is [`crate::dma::Request::Adc`].

use core::future::poll_fn;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicBool, Ordering};
use core::task::Poll;

use embassy_hal_internal::interrupt::Priority;
use embassy_sync::waitqueue::AtomicWaker;

use crate::afec::analog;
use crate::gpio::{AlternateFunction, Flex, Pin};
use crate::interrupt::typelevel::{Binding, Handler, Interrupt};
use crate::mode::{Async, Blocking, Mode};
use crate::rcc::{self, Peripheral};
use crate::{Peri, interrupt, pac, peripherals};

/// Maximum useful sequence length (hardware slots 0..=15; slot value 0 ends).
pub const MAX_SEQUENCE_LEN: usize = 16;

/// 12-bit full-scale code.
pub const MAX_VALUE: u16 = 0x0fff;

/// Internal reference voltage used by the SDK calibration examples (volts).
pub const INTERNAL_VREF_VOLTS: f32 = 1.2;

const CR_ENABLE: u32 = 1 << 0;
const CR_DISABLE: u32 = 1 << 1;
const CR_START: u32 = 1 << 2;
const CR_STOP: u32 = 1 << 3;

const CFGR_CLK_DIV_MASK: u32 = 0x0fff;
const CFGR_DMA_ENABLE: u32 = 1 << 12;
const CFGR_TRG_SOURCE_MASK: u32 = 0x1e000;
const CFGR_TRG_POLARITY_MASK: u32 = 0x60000;
const CFGR_OVERRUN_MODE: u32 = 1 << 19;
const CFGR_CONV_MODE_MASK: u32 = 0x300000;
const CFGR_CONV_MODE_SINGLE: u32 = 0x0;
const CFGR_CONV_MODE_CONTINUE: u32 = 0x100000;
const CFGR_WAIT_MODE: u32 = 1 << 22;

/// Vendor `adc_init()`: clear AFEC analog REG_11 bits `[9:6]`.
const ANALOG_11_ADC_INIT_MASK: u32 = 0xf << 6;
/// Vendor `adc_config_ref_voltage(INTERNAL)`: REG_12 bit 6.
const ANALOG_12_INTERNAL_REF: u32 = 1 << 6;
/// Vendor `adc_enable_vbat31`: REG_2C bit 25.
const ANALOG_2C_VBAT31: u32 = 1 << 25;

const CALIB_GAIN_ADDR: usize = 0x1000_2030;
const CALIB_DCO_ADDR: usize = 0x1000_2034;

static WAKER: AtomicWaker = AtomicWaker::new();
static OVERRUN: AtomicBool = AtomicBool::new(false);

/// ADC interrupt handler.
///
/// Bind with [`crate::bind_interrupts!`]:
///
/// ```ignore
/// bind_interrupts!(struct Irqs {
///     ADC => embassy_asr::adc::InterruptHandler;
/// });
/// ```
pub struct InterruptHandler {
    _private: (),
}

impl Handler<interrupt::typelevel::ADC> for InterruptHandler {
    unsafe fn on_interrupt() {
        let regs = pac::Adc::steal();
        let isr = regs.isr().read();

        if isr.overrun().bit_is_set() {
            OVERRUN.store(true, Ordering::Relaxed);
            // PAC gap: ISR has no `Resettable` impl; write-1-to-clear via bits.
            unsafe {
                regs.isr().write_with_zero(|w| w.bits(1 << 2));
            }
        }

        if isr.eoc().bit_is_set() || isr.eos().bit_is_set() {
            // Mask further IRQs; the waiter re-enables as needed.
            regs.ier().modify(|_, w| {
                w.eoc().clear_bit();
                w.eos().clear_bit();
                w.overrun().clear_bit()
            });
            WAKER.wake();
        }
    }
}

/// ADC kernel clock source (`RCC.CR2.ADC_CLK_SEL`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ClockSource {
    /// PCLK1.
    Pclk1,
    /// SYSCLK.
    Sysclk,
    /// PLL output (PAC enumerant; not used by the SDK ADC examples).
    Pll,
    /// Internal 48 MHz RC oscillator (SDK examples).
    Rco48m,
}

/// Reference voltage selection (`adc_config_ref_voltage`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum RefVoltage {
    /// External reference.
    External,
    /// Internal reference (SDK default after `adc_config_ref_voltage(INTERNAL)`).
    Internal,
}

/// Conversion mode (`adc_config_conv_mode`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ConversionMode {
    /// Convert the programmed sequence once per start.
    Single,
    /// Convert the programmed sequence repeatedly until stopped.
    Continuous,
}

/// Hardware sample-channel index written into `SEQR0` / `SEQR1`.
///
/// Values match `adc_sample_chan_t`. Channel 0 is reserved by the vendor and
/// terminates a programmed sequence when left in an unused slot.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum SampleChannel {
    /// Datasheet `ADC_IN0` on PA11 → sample channel 1.
    In0 = 0x1,
    /// Datasheet `ADC_IN1` on PA8 → sample channel 2.
    In1 = 0x2,
    /// Datasheet `ADC_IN2` on PA5 → sample channel 3.
    In2 = 0x3,
    /// Datasheet `ADC_IN3` on PA4 → sample channel 4.
    In3 = 0x4,
    /// Datasheet `ADC_IN7` on PC15 → sample channel 8.
    In7 = 0x8,
}

impl SampleChannel {
    /// Raw SEQR nibble (`1..=15`). Channel `0` is reserved.
    #[inline]
    pub const fn raw(self) -> u8 {
        self as u8
    }

    /// Construct from a raw sample-channel index.
    ///
    /// Returns [`None`] for the reserved channel `0` or values above `15`.
    #[inline]
    pub const fn from_raw(raw: u8) -> Option<Self> {
        match raw {
            0x1 => Some(Self::In0),
            0x2 => Some(Self::In1),
            0x3 => Some(Self::In2),
            0x4 => Some(Self::In3),
            0x8 => Some(Self::In7),
            _ => None,
        }
    }

    /// Any non-reserved sample-channel index for advanced / undocumented muxes
    /// (for example after enabling the VBAT/3 path).
    #[inline]
    pub const fn from_raw_unchecked(raw: u8) -> Option<RawSampleChannel> {
        RawSampleChannel::new(raw)
    }
}

/// Unchecked sample-channel index in `1..=15`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct RawSampleChannel(u8);

impl RawSampleChannel {
    /// Create a raw channel index, rejecting the reserved `0` and values `> 15`.
    #[inline]
    pub const fn new(raw: u8) -> Option<Self> {
        if raw == 0 || raw > 15 { None } else { Some(Self(raw)) }
    }

    /// Raw SEQR nibble.
    #[inline]
    pub const fn raw(self) -> u8 {
        self.0
    }
}

impl From<SampleChannel> for RawSampleChannel {
    #[inline]
    fn from(value: SampleChannel) -> Self {
        Self(value.raw())
    }
}

/// Pins that map to a documented ADC input.
pub trait AdcPin: Pin {
    /// Documented sample channel for this pin.
    const CHANNEL: SampleChannel;
}

macro_rules! impl_adc_pin {
    ($pin:ident, $channel:expr) => {
        impl AdcPin for peripherals::$pin {
            const CHANNEL: SampleChannel = $channel;
        }
    };
}

impl_adc_pin!(PA11, SampleChannel::In0);
impl_adc_pin!(PA8, SampleChannel::In1);
impl_adc_pin!(PA5, SampleChannel::In2);
impl_adc_pin!(PA4, SampleChannel::In3);
impl_adc_pin!(PC15, SampleChannel::In7);

/// Owned analog input pin configured for ADC sampling.
pub struct AdcChannel<'d> {
    _pin: Flex<'d>,
    channel: SampleChannel,
}

impl<'d> AdcChannel<'d> {
    /// Put `pin` into analog mode with AF0 and retain its sample channel.
    pub fn new<P: AdcPin>(pin: Peri<'d, P>) -> Self {
        make_adc_channel(pin)
    }

    /// Sample channel served by this pin.
    #[inline]
    pub fn channel(&self) -> SampleChannel {
        self.channel
    }
}

fn make_adc_channel<'d, P: AdcPin>(pin: Peri<'d, P>) -> AdcChannel<'d> {
    let channel = P::CHANNEL;
    let mut pin = Flex::new(pin);
    pin.set_alternate_function(AlternateFunction::Function0);
    pin.set_as_analog();
    AdcChannel { _pin: pin, channel }
}

/// Factory calibration constants from `adc_get_calibration_value`.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Calibration {
    /// Gain correction factor.
    pub gain: f32,
    /// DC offset in volts.
    pub dco: f32,
    /// Whether the differential OTP fields were used.
    pub differential: bool,
}

impl Calibration {
    /// Read factory calibration (`adc_get_calibration_value`).
    pub fn read(differential: bool) -> Self {
        // SAFETY: fixed factory OTP addresses used by the vendor SDK.
        let gain_word = unsafe { core::ptr::read_volatile(CALIB_GAIN_ADDR as *const u32) };
        let dco_word = unsafe { core::ptr::read_volatile(CALIB_DCO_ADDR as *const u32) };

        let (gain, dco) = if differential {
            let gain = 1.0 + 0.002 * (((gain_word & 0x1fe000) >> 13) as f32);
            let dco = -0.256 + 0.001 * (((dco_word & 0x3fe000) >> 13) as f32);
            (gain, dco)
        } else {
            let gain = 1.0 + 0.002 * (((gain_word & 0x1fe0) >> 5) as f32);
            let dco = -0.256 + 0.001 * (((dco_word & 0x1ff0) >> 4) as f32);
            (gain, dco)
        };

        Self {
            gain,
            dco,
            differential,
        }
    }

    /// Convert a single-ended raw code to volts (SDK formula, 1.2 V Vref).
    pub fn to_voltage(self, raw: u16) -> f32 {
        let code = f32::from(raw & MAX_VALUE);
        ((INTERNAL_VREF_VOLTS / 4096.0) * code - self.dco) / self.gain
    }

    /// Convert a differential raw code to volts (SDK `single_mode_diff` formula).
    pub fn to_voltage_diff(self, raw: u16) -> f32 {
        let code = raw & MAX_VALUE;
        let signed = if code >= 0x800 {
            (code - 0x800) as i32
        } else {
            -((0x800 - code) as i32)
        };
        ((INTERNAL_VREF_VOLTS / 2048.0) * (signed as f32) - self.dco) / self.gain
    }
}

/// ADC configuration.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Config {
    /// Kernel clock source. SDK examples use [`ClockSource::Rco48m`].
    pub clock_source: ClockSource,
    /// `CFGR` clock divider (`0..=0xFFF`). SDK examples use `20` (~150 kHz).
    pub clock_division: u16,
    /// Reference voltage selection.
    pub ref_voltage: RefVoltage,
    /// When true, keep the previous sample on overrun (clear `OVERRUN_MODE`).
    pub retain_on_overrun: bool,
    /// Wait mode (`CFGR.WAIT_MODE`).
    pub wait_mode: bool,
}

impl Config {
    /// Defaults matching the SDK ADC examples after `adc_init` + clock div 20
    /// and an internal reference.
    pub const fn new() -> Self {
        Self {
            clock_source: ClockSource::Rco48m,
            clock_division: 20,
            ref_voltage: RefVoltage::Internal,
            retain_on_overrun: true,
            wait_mode: false,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

/// ADC driver error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// Sequence was empty or longer than [`MAX_SEQUENCE_LEN`].
    InvalidSequence,
    /// Clock divider exceeded `0xFFF`.
    InvalidClockDivision,
    /// Differential mode is only defined for sample channels 1..=8.
    InvalidDifferentialChannel,
    /// Data register was overwritten before it was read.
    Overrun,
    /// Continuous conversion is already running.
    Busy,
    /// Continuous conversion is not running.
    NotRunning,
}

/// Owned ASR6601 ADC.
pub struct Adc<'d, M: Mode = Blocking> {
    _peri: Peri<'d, peripherals::ADC>,
    continuous: bool,
    _mode: PhantomData<M>,
}

impl<'d> Adc<'d, Blocking> {
    /// Enable clocks, apply `config`, run vendor `adc_init`, and leave the ADC
    /// disabled until a conversion is started.
    pub fn new_blocking(peri: Peri<'d, peripherals::ADC>, config: Config) -> Result<Self, Error> {
        Self::new_inner(peri, config)
    }
}

impl<'d> Adc<'d, Async> {
    /// Create an interrupt-driven ADC.
    ///
    /// `irq` proves the ADC vector is bound to [`InterruptHandler`].
    pub fn new(
        peri: Peri<'d, peripherals::ADC>,
        _irq: impl Binding<interrupt::typelevel::ADC, InterruptHandler> + 'd,
        config: Config,
    ) -> Result<Self, Error> {
        let this = Self::new_inner(peri, config)?;
        interrupt::typelevel::ADC::unpend();
        interrupt::typelevel::ADC::set_priority(Priority::P3);
        unsafe {
            interrupt::typelevel::ADC::enable();
        }
        Ok(this)
    }
}

impl<'d, M: Mode> Adc<'d, M> {
    fn new_inner(peri: Peri<'d, peripherals::ADC>, config: Config) -> Result<Self, Error> {
        if config.clock_division as u32 > CFGR_CLK_DIV_MASK {
            return Err(Error::InvalidClockDivision);
        }

        set_adc_clock_source(config.clock_source);
        let _ = rcc::enable_peripheral(Peripheral::Adc);
        let _ = rcc::reset_peripheral(Peripheral::Adc);

        // Vendor `adc_init()`.
        analog::REG_11.modify(ANALOG_11_ADC_INIT_MASK, 0);
        delay_us(100);

        match config.ref_voltage {
            RefVoltage::External => analog::REG_12.clear_bits(ANALOG_12_INTERNAL_REF),
            RefVoltage::Internal => analog::REG_12.set_bits(ANALOG_12_INTERNAL_REF),
        }

        let this = Self {
            _peri: peri,
            continuous: false,
            _mode: PhantomData,
        };
        this.apply_cfgr_defaults(config);
        Ok(this)
    }

    fn apply_cfgr_defaults(&self, config: Config) {
        let regs = Self::regs();
        regs.cfgr().modify(|r, w| {
            let mut bits = r.bits();
            bits = (bits & !CFGR_CLK_DIV_MASK) | u32::from(config.clock_division);
            // Soft trigger, rising/falling cleared.
            bits &= !CFGR_TRG_SOURCE_MASK;
            bits &= !CFGR_TRG_POLARITY_MASK;
            if config.retain_on_overrun {
                bits &= !CFGR_OVERRUN_MODE;
            } else {
                bits |= CFGR_OVERRUN_MODE;
            }
            if config.wait_mode {
                bits |= CFGR_WAIT_MODE;
            } else {
                bits &= !CFGR_WAIT_MODE;
            }
            bits = (bits & !CFGR_CONV_MODE_MASK) | CFGR_CONV_MODE_SINGLE;
            unsafe { w.bits(bits) }
        });
    }

    /// Configure a documented pin as an ADC input.
    pub fn configure_pin<P: AdcPin>(&mut self, pin: Peri<'d, P>) -> AdcChannel<'d> {
        make_adc_channel(pin)
    }

    /// Enable or disable the VBAT/3 measurement path (`adc_enable_vbat31`).
    ///
    /// The datasheet / SDK do not document which sample channel reads VBAT/3;
    /// use [`RawSampleChannel`] once the channel is known for your silicon.
    pub fn set_vbat31_enabled(&mut self, enabled: bool) {
        if enabled {
            analog::REG_2C.set_bits(ANALOG_2C_VBAT31);
        } else {
            analog::REG_2C.clear_bits(ANALOG_2C_VBAT31);
        }
    }

    /// Enable or disable differential sampling for a channel (`adc_config_sample_chan_diff`).
    ///
    /// Only sample channels 1..=8 have `DIFFSEL` bits.
    pub fn set_differential(&mut self, channel: impl Into<RawSampleChannel>, differential: bool) -> Result<(), Error> {
        let ch = channel.into().raw();
        if !(1..=8).contains(&ch) {
            return Err(Error::InvalidDifferentialChannel);
        }
        let mask = 1u32 << ch;
        let regs = Self::regs();
        regs.diffsel().modify(|r, w| {
            let bits = if differential {
                r.bits() | mask
            } else {
                r.bits() & !mask
            };
            unsafe { w.bits(bits) }
        });
        Ok(())
    }

    /// Program the sample sequence (`adc_config_sample_sequence` for each slot).
    ///
    /// Unused slots are cleared to `0` so the hardware sequence ends after
    /// `channels.len()` conversions (matching SDK examples).
    pub fn set_sequence(&mut self, channels: &[RawSampleChannel]) -> Result<(), Error> {
        if channels.is_empty() || channels.len() > MAX_SEQUENCE_LEN {
            return Err(Error::InvalidSequence);
        }

        let mut seqr0 = 0u32;
        let mut seqr1 = 0u32;
        for (i, ch) in channels.iter().enumerate() {
            let nibble = u32::from(ch.raw()) & 0xf;
            if i < 8 {
                seqr0 |= nibble << (4 * i);
            } else {
                seqr1 |= nibble << (4 * (i - 8));
            }
        }

        let regs = Self::regs();
        unsafe {
            regs.seqr0().write_with_zero(|w| w.bits(seqr0));
            regs.seqr1().write_with_zero(|w| w.bits(seqr1));
        }
        Ok(())
    }

    /// Enable or disable ADC DMA requests (`adc_enable_dma`).
    pub fn set_dma_enabled(&mut self, enabled: bool) {
        Self::regs().cfgr().modify(|r, w| {
            let bits = if enabled {
                r.bits() | CFGR_DMA_ENABLE
            } else {
                r.bits() & !CFGR_DMA_ENABLE
            };
            unsafe { w.bits(bits) }
        });
    }

    /// Address of `DR` for DMA source programming.
    #[inline]
    pub fn dr_address(&self) -> *const u32 {
        Self::regs().dr().as_ptr()
    }

    /// Read factory calibration for the current single/diff preference.
    #[inline]
    pub fn calibration(&self, differential: bool) -> Calibration {
        Calibration::read(differential)
    }

    fn set_conversion_mode(&self, mode: ConversionMode) {
        let mode_bits = match mode {
            ConversionMode::Single => CFGR_CONV_MODE_SINGLE,
            ConversionMode::Continuous => CFGR_CONV_MODE_CONTINUE,
        };
        Self::regs().cfgr().modify(|r, w| {
            let bits = (r.bits() & !CFGR_CONV_MODE_MASK) | mode_bits;
            unsafe { w.bits(bits) }
        });
    }

    fn enable(&self) {
        let regs = Self::regs();
        if regs.cr().read().bits() == 0 {
            regs.cr().modify(|r, w| unsafe { w.bits(r.bits() | CR_ENABLE) });
        }
    }

    fn disable(&self) {
        let regs = Self::regs();
        let cr = regs.cr().read().bits();
        if cr & CR_ENABLE != 0 && cr & CR_START == 0 {
            regs.cr().modify(|r, w| unsafe { w.bits(r.bits() | CR_DISABLE) });
        }
    }

    fn start(&self) {
        let regs = Self::regs();
        let cr = regs.cr().read().bits();
        if cr & CR_ENABLE != 0 && cr & CR_DISABLE == 0 {
            regs.cr().modify(|r, w| unsafe { w.bits(r.bits() | CR_START) });
        }
    }

    fn stop(&self) {
        let regs = Self::regs();
        if regs.cr().read().bits() & CR_START != 0 {
            regs.cr().modify(|r, w| unsafe { w.bits(r.bits() | CR_STOP) });
        }
    }

    fn read_dr(&self) -> u16 {
        (Self::regs().dr().read().bits() & u32::from(MAX_VALUE)) as u16
    }

    fn clear_status(&self) {
        let regs = Self::regs();
        // ISR is write-1-to-clear for documented flags.
        // PAC gap: ISR is not `Resettable`, so use `write_with_zero`.
        unsafe {
            regs.isr().write_with_zero(|w| w.bits((1 << 0) | (1 << 1) | (1 << 2)));
        }
    }

    fn take_overrun(&self) -> bool {
        let hw = Self::regs().isr().read().overrun().bit_is_set();
        let soft = OVERRUN.swap(false, Ordering::Relaxed);
        if hw {
            unsafe {
                Self::regs().isr().write_with_zero(|w| w.bits(1 << 2));
            }
        }
        hw || soft
    }

    /// Stop conversion and disable the ADC digital enable bit.
    pub fn stop_conversion(&mut self) {
        self.stop();
        self.continuous = false;
        self.disable();
        // PAC gap: IER is not `Resettable`.
        unsafe {
            Self::regs().ier().write_with_zero(|w| w.bits(0));
        }
        self.clear_status();
    }

    /// Start continuous conversion of `channels` (SDK `ADC_CONV_MODE_CONTINUE`).
    pub fn start_continuous(&mut self, channels: &[RawSampleChannel]) -> Result<(), Error> {
        if self.continuous {
            return Err(Error::Busy);
        }
        self.set_sequence(channels)?;
        self.set_conversion_mode(ConversionMode::Continuous);
        self.clear_status();
        self.enable();
        self.start();
        self.continuous = true;
        Ok(())
    }

    /// Convenience: continuous conversion of documented channels.
    pub fn start_continuous_channels(&mut self, channels: &[SampleChannel]) -> Result<(), Error> {
        let mut raw = [RawSampleChannel(0); MAX_SEQUENCE_LEN];
        if channels.is_empty() || channels.len() > MAX_SEQUENCE_LEN {
            return Err(Error::InvalidSequence);
        }
        for (i, ch) in channels.iter().enumerate() {
            raw[i] = (*ch).into();
        }
        self.start_continuous(&raw[..channels.len()])
    }

    fn blocking_wait_eoc(&self) -> Result<(), Error> {
        let regs = Self::regs();
        loop {
            if self.take_overrun() {
                return Err(Error::Overrun);
            }
            if regs.isr().read().eoc().bit_is_set() {
                return Ok(());
            }
            core::hint::spin_loop();
        }
    }

    /// Blocking single-shot conversion of one channel (SDK single-mode subset).
    pub fn blocking_read(&mut self, channel: impl Into<RawSampleChannel>) -> Result<u16, Error> {
        if self.continuous {
            return Err(Error::Busy);
        }
        let ch = channel.into();
        self.set_sequence(&[ch])?;
        self.set_conversion_mode(ConversionMode::Single);
        self.clear_status();
        self.enable();
        self.start();
        self.blocking_wait_eoc()?;
        let value = self.read_dr();
        self.stop();
        self.disable();
        Ok(value)
    }

    /// Blocking single-shot conversion of an owned ADC pin.
    pub fn blocking_read_channel(&mut self, channel: &AdcChannel<'_>) -> Result<u16, Error> {
        self.blocking_read(channel.channel())
    }

    /// Blocking read of the next sample while continuous conversion is running.
    pub fn blocking_read_continuous(&mut self) -> Result<u16, Error> {
        if !self.continuous {
            return Err(Error::NotRunning);
        }
        self.blocking_wait_eoc()?;
        Ok(self.read_dr())
    }

    #[inline]
    fn regs() -> pac::Adc {
        unsafe { pac::Adc::steal() }
    }
}

impl<'d> Adc<'d, Async> {
    async fn wait_eoc(&self) -> Result<(), Error> {
        poll_fn(|cx| {
            if self.take_overrun() {
                return Poll::Ready(Err(Error::Overrun));
            }

            let regs = Self::regs();
            if regs.isr().read().eoc().bit_is_set() {
                return Poll::Ready(Ok(()));
            }

            WAKER.register(cx.waker());
            // Enable EOC (+ overrun) after registering to avoid lost wakes.
            regs.ier().modify(|_, w| {
                w.eoc().set_bit();
                w.overrun().set_bit()
            });

            if self.take_overrun() {
                regs.ier().modify(|_, w| {
                    w.eoc().clear_bit();
                    w.eos().clear_bit();
                    w.overrun().clear_bit()
                });
                return Poll::Ready(Err(Error::Overrun));
            }
            if regs.isr().read().eoc().bit_is_set() {
                regs.ier().modify(|_, w| {
                    w.eoc().clear_bit();
                    w.eos().clear_bit();
                    w.overrun().clear_bit()
                });
                return Poll::Ready(Ok(()));
            }

            Poll::Pending
        })
        .await
    }

    /// Async single-shot conversion of one channel.
    pub async fn read(&mut self, channel: impl Into<RawSampleChannel>) -> Result<u16, Error> {
        if self.continuous {
            return Err(Error::Busy);
        }
        let ch = channel.into();
        self.set_sequence(&[ch])?;
        self.set_conversion_mode(ConversionMode::Single);
        self.clear_status();
        self.enable();
        self.start();
        self.wait_eoc().await?;
        let value = self.read_dr();
        self.stop();
        self.disable();
        Ok(value)
    }

    /// Async single-shot conversion of an owned ADC pin.
    pub async fn read_channel(&mut self, channel: &AdcChannel<'_>) -> Result<u16, Error> {
        self.read(channel.channel()).await
    }

    /// Async read of the next sample while continuous conversion is running.
    pub async fn read_continuous(&mut self) -> Result<u16, Error> {
        if !self.continuous {
            return Err(Error::NotRunning);
        }
        self.wait_eoc().await?;
        Ok(self.read_dr())
    }
}

impl<'d, M: Mode> Drop for Adc<'d, M> {
    fn drop(&mut self) {
        self.stop_conversion();
        analog::REG_2C.clear_bits(ANALOG_2C_VBAT31);
        let _ = rcc::disable_peripheral(Peripheral::Adc);
    }
}

fn set_adc_clock_source(source: ClockSource) {
    let sel = match source {
        ClockSource::Pclk1 => pac::rcc::cr2::AdcClkSel::Pclk1,
        ClockSource::Sysclk => pac::rcc::cr2::AdcClkSel::Sysclk,
        ClockSource::Pll => pac::rcc::cr2::AdcClkSel::Pll,
        ClockSource::Rco48m => pac::rcc::cr2::AdcClkSel::Rco48m,
    };

    critical_section::with(|_| {
        let rcc = unsafe { pac::Rcc::steal() };
        // Vendor `rcc_set_adc_clk_source`: gate off, wait sync, then select.
        if rcc.sr1().read().adc_clk_en_sync().bit_is_set() {
            rcc.cgr0().modify(|_, w| w.adc_clk_en().clear_bit());
            while rcc.sr1().read().adc_clk_en_sync().bit_is_set() {
                core::hint::spin_loop();
            }
        }
        rcc.cr2().modify(|_, w| w.adc_clk_sel().variant(sel));
    });
}

fn delay_us(us: u32) {
    let hz = rcc::clocks().map(|c| c.hclk_hz).unwrap_or(48_000_000);
    let cycles = (u64::from(hz) * u64::from(us)).div_ceil(1_000_000);
    cortex_m::asm::delay(cycles.min(u64::from(u32::MAX)) as u32);
}
