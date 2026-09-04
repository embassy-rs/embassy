//! Digital-to-analog converter (DAC).
//!
//! Register programming follows the vendor `tremo_dac` driver and the SDK
//! `dac/sw_trigger` / `dac/sine_wave` examples. Enabling the converter also
//! programs the AFEC analog window registers used by `dac_cmd()`.
//!
//! The fixed analog output is **GPIO09** (`PA9`). This driver places that pin
//! in analog mode while the DAC is owned.
//!
//! # DMA
//!
//! [`Dac::set_dma_enabled`] only toggles `CR.DMA_EN`. Buffered waveform
//! playback is not implemented here. Applications that need the sine-wave
//! pattern should configure a memory-to-peripheral DMA channel whose
//! destination is [`Dac::dhr_address`] and whose request selection is
//! [`crate::dma::Request::Dac`] (vendor handshake `DMA_HANDSHAKE_DACCTRL`),
//! typically paced by a BSTIMER TRGO trigger.

use crate::afec::analog;
use crate::gpio::Flex;
use crate::pac::dac::cr::{MaskAmpSel, TrigSrcSel, TrigTypeSel, WaveSel};
use crate::rcc::{self, Peripheral};
use crate::{Peri, pac, peripherals};

/// Maximum 12-bit sample accepted by [`Dac::set_holding`] / [`Dac::set`].
pub const MAX_VALUE: u16 = 0x0fff;

const SWTRIG: u32 = 1 << 0;
const SR_UNDERFLOW: u32 = 1 << 0;
const SR_EMPTY: u32 = 1 << 1;

/// Analog-window bits touched by vendor `dac_cmd()`.
const ANALOG_11_POWERDOWN: u32 = 3 << 24;
const ANALOG_12_POWERDOWN: u32 = 1 << 8;
const ANALOG_27_OUTPUT_ENABLE: u32 = 3 << 11;

/// DAC trigger source (`CR.TRIG_SRC_SEL` / `TRIG_EN`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum TriggerSource {
    /// GPTIMER1 TRGO.
    Gptimer1Trgo,
    /// GPTIMER0 TRGO.
    Gptimer0Trgo,
    /// BSTIMER1 TRGO.
    Bstimer1Trgo,
    /// BSTIMER0 TRGO.
    Bstimer0Trgo,
    /// External GPIO6 edge.
    Gpio6,
    /// External GPIO24 edge.
    Gpio24,
    /// External GPIO43 edge.
    Gpio43,
    /// Software trigger via [`Dac::software_trigger`].
    Software,
    /// No external/software trigger (`TRIG_EN` cleared).
    None,
}

/// Edge polarity for hardware trigger sources (`CR.TRIG_TYPE_SEL`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum TriggerType {
    /// Rising edge.
    RisingEdge,
    /// Falling edge.
    FallingEdge,
    /// Both edges.
    RisingFallingEdge,
}

/// Built-in wave generator selection (`CR.WAVE_SEL`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum WaveType {
    /// No wave generation; convert programmed samples.
    None,
    /// Pseudo-noise wave.
    Noise,
    /// Triangle wave.
    Triangle,
}

/// Wave amplitude / LFSR mask (`CR.MASK_AMP_SEL`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum WaveLevel {
    /// Amplitude / mask 1.
    Level1,
    /// Amplitude / mask 3.
    Level3,
    /// Amplitude / mask 7.
    Level7,
    /// Amplitude / mask 15.
    Level15,
    /// Amplitude / mask 31.
    Level31,
    /// Amplitude / mask 63.
    Level63,
    /// Amplitude / mask 127.
    Level127,
    /// Amplitude / mask 255.
    Level255,
    /// Amplitude / mask 511.
    Level511,
    /// Amplitude / mask 1023.
    Level1023,
}

/// DAC interrupt / status flags (`SR`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Interrupt {
    /// Holding-buffer underflow.
    Underflow,
    /// Holding buffer empty.
    Empty,
}

/// DAC configuration matching `dac_config_t`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Config {
    /// Trigger source.
    pub trigger_source: TriggerSource,
    /// Trigger edge type (meaningful for GPIO / timer sources).
    pub trigger_type: TriggerType,
    /// Built-in wave type.
    pub wave_type: WaveType,
    /// Wave amplitude / mask level.
    pub wave_level: WaveLevel,
}

impl Config {
    /// Defaults used by the vendor software-trigger example after `memset` +
    /// `trigger_src = SOFTWARE`: software trigger, rising edge, no wave.
    pub const fn new() -> Self {
        Self {
            trigger_source: TriggerSource::Software,
            trigger_type: TriggerType::RisingEdge,
            wave_type: WaveType::None,
            wave_level: WaveLevel::Level1,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

/// DAC driver error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// Sample is outside the 12-bit range `0..=4095`.
    InvalidData,
    /// [`Dac::software_trigger`] / [`Dac::set`] requires [`TriggerSource::Software`].
    TriggerNotSoftware,
}

/// Owned ASR6601 DAC.
pub struct Dac<'d> {
    _peri: Peri<'d, peripherals::DAC>,
    _pin: Flex<'d>,
    trigger_source: TriggerSource,
}

impl<'d> Dac<'d> {
    /// Enable the DAC clock, apply `config`, put `PA9` in analog mode, and
    /// enable the converter (digital `DAC_EN` plus vendor analog sequence).
    pub fn new(peri: Peri<'d, peripherals::DAC>, pin: Peri<'d, peripherals::PA9>, config: Config) -> Self {
        let _ = rcc::enable_peripheral(Peripheral::Dac);
        let _ = rcc::reset_peripheral(Peripheral::Dac);

        let mut pin = Flex::new(pin);
        pin.set_as_analog();

        let mut this = Self {
            _peri: peri,
            _pin: pin,
            trigger_source: config.trigger_source,
        };
        this.configure(config);
        this.enable();
        this
    }

    /// Reconfigure trigger and wave settings (`dac_init`).
    ///
    /// Does not change the enable state or analog window.
    pub fn configure(&mut self, config: Config) {
        self.trigger_source = config.trigger_source;

        let trig_src = match config.trigger_source {
            TriggerSource::Gptimer1Trgo => Some(TrigSrcSel::Gptimer1Trgo),
            TriggerSource::Gptimer0Trgo => Some(TrigSrcSel::Gptimer0Trgo),
            TriggerSource::Bstimer1Trgo => Some(TrigSrcSel::Bstimer1Trgo),
            TriggerSource::Bstimer0Trgo => Some(TrigSrcSel::Bstimer0Trgo),
            TriggerSource::Gpio6 => Some(TrigSrcSel::Gpio6),
            TriggerSource::Gpio24 => Some(TrigSrcSel::Gpio24),
            TriggerSource::Gpio43 => Some(TrigSrcSel::Gpio43),
            TriggerSource::Software => Some(TrigSrcSel::Software),
            TriggerSource::None => None,
        };
        let trig_type = match config.trigger_type {
            TriggerType::RisingEdge => TrigTypeSel::RisingEdge,
            TriggerType::FallingEdge => TrigTypeSel::FallingEdge,
            TriggerType::RisingFallingEdge => TrigTypeSel::RisingFallingEdge,
        };
        let wave = match config.wave_type {
            WaveType::None => WaveSel::None,
            WaveType::Noise => WaveSel::Noise,
            WaveType::Triangle => WaveSel::Triangle,
        };
        let level = match config.wave_level {
            WaveLevel::Level1 => MaskAmpSel::Value1,
            WaveLevel::Level3 => MaskAmpSel::Value3,
            WaveLevel::Level7 => MaskAmpSel::Value7,
            WaveLevel::Level15 => MaskAmpSel::Value15,
            WaveLevel::Level31 => MaskAmpSel::Value31,
            WaveLevel::Level63 => MaskAmpSel::Value63,
            WaveLevel::Level127 => MaskAmpSel::Value127,
            WaveLevel::Level255 => MaskAmpSel::Value255,
            WaveLevel::Level511 => MaskAmpSel::Value511,
            WaveLevel::Level1023 => MaskAmpSel::Value1023,
        };

        Self::regs().cr().modify(|_, w| {
            match trig_src {
                Some(src) => {
                    w.trig_src_sel().variant(src);
                    w.trig_en().set_bit();
                }
                None => {
                    w.trig_en().clear_bit();
                }
            }
            w.trig_type_sel().variant(trig_type);
            w.wave_sel().variant(wave);
            w.mask_amp_sel().variant(level);
            w
        });
    }

    /// Enable the DAC channel (`dac_cmd(true)`).
    pub fn enable(&mut self) {
        // Vendor order: clear power-down bits, enable analog output, then DAC_EN.
        analog::REG_11.clear_bits(ANALOG_11_POWERDOWN);
        analog::REG_12.clear_bits(ANALOG_12_POWERDOWN);
        analog::REG_27.set_bits(ANALOG_27_OUTPUT_ENABLE);

        Self::regs().cr().modify(|_, w| w.dac_en().set_bit());
    }

    /// Disable the DAC channel (`dac_cmd(false)`).
    pub fn disable(&mut self) {
        Self::regs().cr().modify(|_, w| w.dac_en().clear_bit());

        analog::REG_11.set_bits(ANALOG_11_POWERDOWN);
        analog::REG_12.set_bits(ANALOG_12_POWERDOWN);
        analog::REG_27.clear_bits(ANALOG_27_OUTPUT_ENABLE);
    }

    /// Write a 12-bit sample into the data-holding register (`dac_write_data`).
    ///
    /// With a trigger source enabled, the value is transferred to the output
    /// register on the next trigger.
    pub fn set_holding(&mut self, value: u16) -> Result<(), Error> {
        if value > MAX_VALUE {
            return Err(Error::InvalidData);
        }
        // PAC gap: DHR has no field accessors.
        unsafe {
            Self::regs().dhr().write_with_zero(|w| w.bits(u32::from(value)));
        }
        Ok(())
    }

    /// Pulse the software trigger (`dac_software_trigger_cmd(true)`).
    pub fn software_trigger(&mut self) -> Result<(), Error> {
        if self.trigger_source != TriggerSource::Software {
            return Err(Error::TriggerNotSoftware);
        }
        // PAC gap: SWTRIGR has no field accessors. Vendor writes bit 0.
        unsafe {
            Self::regs().swtrigr().write_with_zero(|w| w.bits(SWTRIG));
        }
        Ok(())
    }

    /// Write a sample and, for [`TriggerSource::Software`], immediately
    /// trigger conversion (SDK `sw_trigger` sequence).
    pub fn set(&mut self, value: u16) -> Result<(), Error> {
        self.set_holding(value)?;
        if self.trigger_source == TriggerSource::Software {
            self.software_trigger()?;
        }
        Ok(())
    }

    /// Read the data-output register (`dac_read_output_data`).
    pub fn read_output(&self) -> u16 {
        // PAC gap: DOR has no field accessors.
        (Self::regs().dor().read().bits() & u32::from(MAX_VALUE)) as u16
    }

    /// Enable or disable DAC DMA requests (`dac_dma_cmd`).
    ///
    /// See the module-level DMA note: this does not start a transfer.
    pub fn set_dma_enabled(&mut self, enabled: bool) {
        Self::regs().cr().modify(|_, w| w.dma_en().bit(enabled));
    }

    /// Whether `CR.DMA_EN` is set.
    pub fn is_dma_enabled(&self) -> bool {
        Self::regs().cr().read().dma_en().bit_is_set()
    }

    /// Address of `DHR` for DMA destination programming.
    #[inline]
    pub fn dhr_address(&self) -> *mut u32 {
        Self::regs().dhr().as_ptr()
    }

    /// Enable or disable a DAC interrupt source (`dac_config_interrupt`).
    pub fn set_interrupt_enabled(&mut self, interrupt: Interrupt, enabled: bool) {
        Self::regs().cr().modify(|_, w| match interrupt {
            Interrupt::Underflow => w.intr_underflow_en().bit(enabled),
            Interrupt::Empty => w.intr_empty_en().bit(enabled),
        });
    }

    /// Whether a status flag is set (`dac_get_interrupt_status`).
    pub fn interrupt_status(&self, interrupt: Interrupt) -> bool {
        // PAC gap: SR has no field accessors.
        let sr = Self::regs().sr().read().bits();
        match interrupt {
            Interrupt::Underflow => sr & SR_UNDERFLOW != 0,
            Interrupt::Empty => sr & SR_EMPTY != 0,
        }
    }

    /// Clear a status flag by writing its bit (`dac_clear_interrupt`).
    pub fn clear_interrupt(&mut self, interrupt: Interrupt) {
        let bit = match interrupt {
            Interrupt::Underflow => SR_UNDERFLOW,
            Interrupt::Empty => SR_EMPTY,
        };
        // PAC gap: SR has no field accessors.
        unsafe {
            Self::regs().sr().write_with_zero(|w| w.bits(bit));
        }
    }

    #[inline]
    fn regs() -> pac::Dac {
        // Owned Peri token proves exclusivity; steal only obtains the handle.
        unsafe { pac::Dac::steal() }
    }
}

impl Drop for Dac<'_> {
    fn drop(&mut self) {
        self.disable();
        let _ = rcc::disable_peripheral(Peripheral::Dac);
    }
}
