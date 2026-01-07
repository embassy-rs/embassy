//! ADC driver
use core::future::Future;
use core::marker::PhantomData;

use embassy_hal_internal::{Peri, PeripheralType};
use maitake_sync::WaitCell;
use paste::paste;

use crate::clocks::periph_helpers::{AdcClockSel, AdcConfig, Div4};
use crate::clocks::{ClockError, Gate, PoweredClock, enable_and_reset};
use crate::gpio::{GpioPin, SealedPin};
use crate::interrupt::typelevel::{Handler, Interrupt};
use crate::pac;
use crate::pac::adc1::cfg::{HptExdi, Pwrsel, Refsel, Tcmdres, Tprictrl, Tres};
use crate::pac::adc1::cmdh1::{Avgs, Cmpen, Next, Sts};
use crate::pac::adc1::cmdl1::Mode as ConvMode;
use crate::pac::adc1::ctrl::CalAvgs;
use crate::pac::adc1::tctrl::{Tcmd, Tpri};

/// Trigger priority policy for ADC conversions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TriggerPriorityPolicy {
    ConvPreemptImmediatelyNotAutoResumed = 0,
    ConvPreemptSoftlyNotAutoResumed = 1,
    ConvPreemptImmediatelyAutoRestarted = 4,
    ConvPreemptSoftlyAutoRestarted = 5,
    ConvPreemptImmediatelyAutoResumed = 12,
    ConvPreemptSoftlyAutoResumed = 13,
    ConvPreemptSubsequentlyNotAutoResumed = 2,
    ConvPreemptSubsequentlyAutoRestarted = 6,
    ConvPreemptSubsequentlyAutoResumed = 14,
    TriggerPriorityExceptionDisabled = 16,
}

/// Configuration for the LPADC peripheral.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Config {
    /// Control system transition to Stop and Wait power modes while
    /// ADC is converting.
    ///
    /// When enabled in Doze mode, immediate entries to Wait or Stop
    /// are allowed.
    ///
    /// When disabled, the ADC will wait for the current averaging
    /// iteration/FIFO storage to complete before acknowledging stop
    /// or wait mode entry.
    pub enable_in_doze_mode: bool,

    /// Auto-Calibration Averages.
    pub conversion_average_mode: CalAvgs,

    /// ADC analog circuits are pre-enabled and ready to execute
    /// conversions without startup delays(at the cost of higher DC
    /// current consumption).
    pub enable_analog_preliminary: bool,

    /// Power-up delay value (in ADC clock cycles)
    pub power_up_delay: u8,

    /// Reference voltage source selection
    pub reference_voltage_source: Refsel,

    /// Power configuration selection.
    pub power_level_mode: Pwrsel,

    /// Trigger priority policy for handling multiple triggers
    pub trigger_priority_policy: TriggerPriorityPolicy,

    /// Enables the ADC pausing function. When enabled, a programmable
    /// delay is inserted during command execution sequencing between
    /// LOOP iterations, between commands in a sequence, and between
    /// conversions when command is executing in "Compare Until True"
    /// configuration.
    pub enable_conv_pause: bool,

    /// Controls the duration of pausing during command execution
    /// sequencing. The pause delay is a count of (convPauseDelay*4)
    /// ADCK cycles.
    ///
    /// Only available when ADC pausing function is enabled. The
    /// available value range is in 9-bit.
    pub conv_pause_delay: u16,

    /// Power configuration (normal/deep sleep behavior)
    pub power: PoweredClock,

    /// ADC clock source selection
    pub source: AdcClockSel,

    /// Clock divider for ADC clock
    pub div: Div4,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            enable_in_doze_mode: true,
            conversion_average_mode: CalAvgs::NoAverage,
            enable_analog_preliminary: false,
            power_up_delay: 0x80,
            reference_voltage_source: Refsel::Option1,
            power_level_mode: Pwrsel::Lowest,
            trigger_priority_policy: TriggerPriorityPolicy::ConvPreemptImmediatelyNotAutoResumed,
            enable_conv_pause: false,
            conv_pause_delay: 0,
            power: PoweredClock::NormalEnabledDeepSleepDisabled,
            source: AdcClockSel::FroLfDiv,
            div: Div4::no_div(),
        }
    }
}

/// Configuration for a conversion command.
///
/// Defines the parameters for a single ADC conversion operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConvCommandConfig {
    pub chained_next_command_number: Next,
    pub enable_auto_channel_increment: bool,
    pub loop_count: u8,
    pub hardware_average_mode: Avgs,
    pub sample_time_mode: Sts,
    pub hardware_compare_mode: Cmpen,
    pub hardware_compare_value_high: u32,
    pub hardware_compare_value_low: u32,
    pub conversion_resolution_mode: ConvMode,
    pub enable_wait_trigger: bool,
}

impl Default for ConvCommandConfig {
    fn default() -> Self {
        Self {
            chained_next_command_number: Next::NoNextCmdTerminateOnFinish,
            enable_auto_channel_increment: false,
            loop_count: 0,
            hardware_average_mode: Avgs::NoAverage,
            sample_time_mode: Sts::Sample3p5,
            hardware_compare_mode: Cmpen::DisabledAlwaysStoreResult,
            hardware_compare_value_high: 0,
            hardware_compare_value_low: 0,
            conversion_resolution_mode: ConvMode::Data12Bits,
            enable_wait_trigger: false,
        }
    }
}

/// Configuration for a conversion trigger.
///
/// Defines how a trigger initiates ADC conversions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConvTriggerConfig {
    pub target_command_id: Tcmd,
    pub delay_power: u8,
    pub priority: Tpri,
    pub enable_hardware_trigger: bool,
}

impl Default for ConvTriggerConfig {
    fn default() -> Self {
        ConvTriggerConfig {
            target_command_id: Tcmd::NotValid,
            delay_power: 0,
            priority: Tpri::HighestPriority,
            enable_hardware_trigger: false,
        }
    }
}

/// Shorthand for `Result<T>`.
pub type Result<T> = core::result::Result<T, Error>;

/// ADC Error types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// FIFO is empty, no conversion result available
    FifoEmpty,
    /// Invalid configuration
    InvalidConfig,
    /// Clock configuration error.
    ClockSetup(ClockError),
}

/// Result of an ADC conversion.
///
/// Contains the conversion value and metadata about the conversion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConvResult {
    pub command_id_source: u8,
    pub loop_count_index: u8,
    pub trigger_id_source: u8,
    pub conv_value: u16,
}

/// ADC interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

/// ADC driver instance.
pub struct Adc<'a, M: Mode> {
    _inst: PhantomData<&'a M>,
    channel_idx: u8,
    info: Info,
}

impl<'a> Adc<'a, Blocking> {
    /// Create a new blocking instance of the ADC driver.
    /// # Arguments
    /// * `_inst` - ADC peripheral instance
    /// * `pin` - GPIO pin to use for ADC
    /// * `config` - ADC configuration
    pub fn new_blocking<T: Instance>(
        _inst: Peri<'a, T>,
        pin: Peri<'a, impl AdcPin<T>>,
        config: Config,
    ) -> Result<Self> {
        Self::new_inner(_inst, pin, config)
    }

    /// Enable ADC interrupts.
    ///
    /// Enables the interrupt sources specified in the bitmask.
    ///
    /// # Arguments
    /// * `mask` - Bitmask of interrupt sources to enable
    pub fn enable_interrupt(&mut self, mask: u32) {
        self.info.regs.ie().modify(|r, w| unsafe { w.bits(r.bits() | mask) });
    }

    /// Disable ADC interrupts.
    ///
    /// Disables the interrupt sources specified in the bitmask.
    ///
    /// # Arguments
    /// * `mask` - Bitmask of interrupt sources to disable
    pub fn disable_interrupt(&mut self, mask: u32) {
        self.info.regs.ie().modify(|r, w| unsafe { w.bits(r.bits() & !mask) });
    }

    pub fn set_fifo_watermark(&mut self, watermark: u8) -> Result<()> {
        if watermark > 0b111 {
            return Err(Error::InvalidConfig);
        }
        self.info
            .regs
            .fctrl0()
            .modify(|_r, w| unsafe { w.fwmark().bits(watermark) });
        Ok(())
    }

    /// Trigger ADC conversion(s) via software.
    ///
    /// Initiates conversion(s) for the trigger(s) specified in the bitmask.
    /// Each bit in the mask corresponds to a trigger ID (bit 0 = trigger 0, etc.).
    ///
    /// # Arguments
    /// * `trigger_id_mask` - Bitmask of trigger IDs to activate (bit N = trigger N)
    ///
    /// # Returns
    /// * `Ok(())` if the triger mask was valid
    /// * `Err(Error::InvalidConfig)` if the mask was greater than `0b1111`
    pub fn do_software_trigger(&self, trigger_id_mask: u8) -> Result<()> {
        if trigger_id_mask > 0b1111 {
            return Err(Error::InvalidConfig);
        }
        self.info
            .regs
            .swtrig()
            .write(|w| unsafe { w.bits(trigger_id_mask as u32) });
        Ok(())
    }

    /// Set conversion command configuration.
    ///
    /// Configures a conversion command slot with the specified parameters.
    /// Commands define how conversions are performed (channel, resolution, etc.).
    ///
    /// # Arguments
    /// * `index` - Command index (Must be in range 1..=7)
    /// * `config` - Command configuration
    ///
    /// # Returns
    /// * `Ok(())` if the command was configured successfully
    /// * `Err(Error::InvalidConfig)` if the index is out of range
    pub fn set_conv_command_config(&self, index: usize, config: &ConvCommandConfig) -> Result<()> {
        self.set_conv_command_config_inner(index, config)
    }

    /// Set conversion trigger configuration.
    ///
    /// Configures a trigger to initiate conversions. Triggers can be
    /// activated by software or hardware signals.
    ///
    /// # Arguments
    /// * `trigger_id` - Trigger index (0..=3)
    /// * `config` - Trigger configuration
    pub fn set_conv_trigger_config(&self, trigger_id: usize, config: &ConvTriggerConfig) -> Result<()> {
        self.set_conv_trigger_config_inner(trigger_id, config)
    }

    /// Reset the FIFO buffer.
    ///
    /// Clears all pending conversion results from the FIFO.
    pub fn do_reset_fifo(&self) {
        self.info.regs.ctrl().modify(|_, w| w.rstfifo0().trigger_reset());
    }

    /// Get conversion result from FIFO.
    ///
    /// Reads and returns the next conversion result from the FIFO.
    /// Returns `None` if the FIFO is empty.
    ///
    /// # Returns
    /// - `Some(ConvResult)` if a result is available
    /// - `Err(Error::FifoEmpty)` if the FIFO is empty
    pub fn get_conv_result(&self) -> Result<ConvResult> {
        self.get_conv_result_inner()
    }
}

impl<'a> Adc<'a, Async> {
    /// Initialize ADC with interrupt support.
    ///
    /// # Arguments
    /// * `_inst` - ADC peripheral instance
    /// * `pin` - GPIO pin to use for ADC
    /// * `_irq` - Interrupt binding for this ADC instance
    /// * `config` - ADC configuration
    pub fn new_async<T: Instance>(
        _inst: Peri<'a, T>,
        pin: Peri<'a, impl AdcPin<T>>,
        _irq: impl crate::interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'a,
        config: Config,
    ) -> Result<Self> {
        let adc = Self::new_inner(_inst, pin, config)?;

        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        let cfg = ConvCommandConfig {
            chained_next_command_number: Next::NoNextCmdTerminateOnFinish,
            enable_auto_channel_increment: false,
            loop_count: 0,
            hardware_average_mode: Avgs::NoAverage,
            sample_time_mode: Sts::Sample3p5,
            hardware_compare_mode: Cmpen::DisabledAlwaysStoreResult,
            hardware_compare_value_high: 0,
            hardware_compare_value_low: 0,
            conversion_resolution_mode: ConvMode::Data16Bits,
            enable_wait_trigger: false,
        };

        // We always use command 1, so this cannot fail
        _ = adc.set_conv_command_config_inner(1, &cfg);

        let cfg = ConvTriggerConfig {
            target_command_id: Tcmd::ExecuteCmd1,
            delay_power: 0,
            priority: Tpri::HighestPriority,
            enable_hardware_trigger: false,
        };

        // We always use trigger 0, so this cannot fail
        _ = adc.set_conv_trigger_config_inner(0, &cfg);

        // We always set the watermark to 0 (trigger when 1 is available)
        adc.info.regs.fctrl0().modify(|_r, w| unsafe { w.fwmark().bits(0) });

        Ok(adc)
    }

    /// Set the number of averages
    pub fn set_averages(&mut self, avgs: Avgs) {
        // TODO: we should probably return a result or wait for idle?
        // "A write to a CMD buffer while that CMD buffer is controlling the ADC operation may cause unpredictable behavior."
        self.info.regs.cmdh1().modify(|_r, w| w.avgs().variant(avgs));
    }

    /// Set the sample time
    pub fn set_sample_time(&mut self, st: Sts) {
        // TODO: we should probably return a result or wait for idle?
        // "A write to a CMD buffer while that CMD buffer is controlling the ADC operation may cause unpredictable behavior."
        self.info.regs.cmdh1().modify(|_r, w| w.sts().variant(st));
    }

    pub fn set_resolution(&mut self, mode: ConvMode) {
        // TODO: we should probably return a result or wait for idle?
        // "A write to a CMD buffer while that CMD buffer is controlling the ADC operation may cause unpredictable behavior."
        self.info.regs.cmdl1().modify(|_r, w| w.mode().variant(mode));
    }

    fn wait_idle(&mut self) -> impl Future<Output = core::result::Result<(), maitake_sync::Closed>> + use<'_> {
        self.info
            .wait_cell
            .wait_for(|| self.info.regs.ie().read().fwmie0().bit_is_clear())
    }

    /// Read ADC value asynchronously.
    ///
    /// Performs a single ADC conversion and returns the result when the ADC interrupt is triggered.
    ///
    /// The function:
    /// 1. Enables the FIFO watermark interrupt
    /// 2. Triggers a software conversion on trigger 0
    /// 3. Waits for the conversion to complete
    /// 4. Returns the conversion result
    ///
    /// # Returns
    /// 16-bit ADC conversion value
    pub async fn read(&mut self) -> Result<u16> {
        // If we cancelled a previous read, we might still be busy, wait
        // until the interrupt is cleared (done by the interrupt)
        _ = self.wait_idle().await;

        // Clear the fifo
        self.info.regs.ctrl().modify(|_, w| w.rstfifo0().trigger_reset());

        // Trigger a new conversion
        self.info.regs.ie().modify(|_r, w| w.fwmie0().set_bit());
        self.info.regs.swtrig().write(|w| w.swt0().set_bit());

        // Wait for completion
        _ = self.wait_idle().await;

        self.get_conv_result_inner().map(|r| r.conv_value)
    }
}

impl<'a, M: Mode> Adc<'a, M> {
    /// Internal initialization function shared by `new_async` and `new_blocking`.
    fn new_inner<T: Instance>(_inst: Peri<'a, T>, pin: Peri<'a, impl AdcPin<T>>, config: Config) -> Result<Self> {
        let info = T::info();
        let adc = info.regs;

        _ = unsafe {
            enable_and_reset::<T>(&AdcConfig {
                power: config.power,
                source: config.source,
                div: config.div,
            })
            .map_err(Error::ClockSetup)?
        };

        pin.mux();

        /* Reset the module. */
        adc.ctrl().modify(|_, w| w.rst().held_in_reset());
        adc.ctrl().modify(|_, w| w.rst().released_from_reset());

        adc.ctrl().modify(|_, w| w.rstfifo0().trigger_reset());

        /* Disable the module before setting configuration. */
        adc.ctrl().modify(|_, w| w.adcen().disabled());

        /* Configure the module generally. */
        adc.ctrl().modify(|_, w| w.dozen().bit(config.enable_in_doze_mode));

        /* Set calibration average mode. */
        adc.ctrl()
            .modify(|_, w| w.cal_avgs().variant(config.conversion_average_mode));

        adc.cfg().write(|w| unsafe {
            w.pwren().bit(config.enable_analog_preliminary);

            w.pudly()
                .bits(config.power_up_delay)
                .refsel()
                .variant(config.reference_voltage_source)
                .pwrsel()
                .variant(config.power_level_mode)
                .tprictrl()
                .variant(match config.trigger_priority_policy {
                    TriggerPriorityPolicy::ConvPreemptSoftlyNotAutoResumed
                    | TriggerPriorityPolicy::ConvPreemptSoftlyAutoRestarted
                    | TriggerPriorityPolicy::ConvPreemptSoftlyAutoResumed => Tprictrl::FinishCurrentOnPriority,
                    TriggerPriorityPolicy::ConvPreemptSubsequentlyNotAutoResumed
                    | TriggerPriorityPolicy::ConvPreemptSubsequentlyAutoRestarted
                    | TriggerPriorityPolicy::ConvPreemptSubsequentlyAutoResumed => Tprictrl::FinishSequenceOnPriority,
                    _ => Tprictrl::AbortCurrentOnPriority,
                })
                .tres()
                .variant(match config.trigger_priority_policy {
                    TriggerPriorityPolicy::ConvPreemptImmediatelyAutoRestarted
                    | TriggerPriorityPolicy::ConvPreemptSoftlyAutoRestarted
                    | TriggerPriorityPolicy::ConvPreemptImmediatelyAutoResumed
                    | TriggerPriorityPolicy::ConvPreemptSoftlyAutoResumed
                    | TriggerPriorityPolicy::ConvPreemptSubsequentlyAutoRestarted
                    | TriggerPriorityPolicy::ConvPreemptSubsequentlyAutoResumed => Tres::Enabled,
                    _ => Tres::Disabled,
                })
                .tcmdres()
                .variant(match config.trigger_priority_policy {
                    TriggerPriorityPolicy::ConvPreemptImmediatelyAutoResumed
                    | TriggerPriorityPolicy::ConvPreemptSoftlyAutoResumed
                    | TriggerPriorityPolicy::ConvPreemptSubsequentlyAutoResumed
                    | TriggerPriorityPolicy::TriggerPriorityExceptionDisabled => Tcmdres::Enabled,
                    _ => Tcmdres::Disabled,
                })
                .hpt_exdi()
                .variant(match config.trigger_priority_policy {
                    TriggerPriorityPolicy::TriggerPriorityExceptionDisabled => HptExdi::Disabled,
                    _ => HptExdi::Enabled,
                })
        });

        if config.enable_conv_pause {
            adc.pause()
                .modify(|_, w| unsafe { w.pauseen().enabled().pausedly().bits(config.conv_pause_delay) });
        } else {
            adc.pause().write(|w| unsafe { w.bits(0) });
        }

        adc.fctrl0().write(|w| unsafe { w.fwmark().bits(0) });

        // Enable ADC
        adc.ctrl().modify(|_, w| w.adcen().enabled());

        Ok(Self {
            _inst: PhantomData,
            channel_idx: pin.channel(),
            info,
        })
    }

    /// Perform offset calibration.
    /// Waits for calibration to complete before returning.
    pub fn do_offset_calibration(&self) {
        // Enable calibration mode
        self.info
            .regs
            .ctrl()
            .modify(|_, w| w.calofs().offset_calibration_request_pending());

        // Wait for calibration to complete (polling status register)
        while self.info.regs.stat().read().cal_rdy().is_not_set() {}
    }

    /// Calculate gain conversion result from gain adjustment factor.
    ///
    /// # Arguments
    /// * `gain_adjustment` - Gain adjustment factor
    ///
    /// # Returns
    /// Gain calibration register value
    pub fn get_gain_conv_result(&self, mut gain_adjustment: f32) -> u32 {
        let mut gcra_array = [0u32; 17];
        let mut gcalr: u32 = 0;

        for i in (1..=17).rev() {
            let shift = 16 - (i - 1);
            let step = 1.0 / (1u32 << shift) as f32;
            let tmp = (gain_adjustment / step) as u32;
            gcra_array[i - 1] = tmp;
            gain_adjustment -= tmp as f32 * step;
        }

        for i in (1..=17).rev() {
            gcalr += gcra_array[i - 1] << (i - 1);
        }
        gcalr
    }

    /// Perform automatic gain calibration.
    pub fn do_auto_calibration(&self) {
        self.info
            .regs
            .ctrl()
            .modify(|_, w| w.cal_req().calibration_request_pending());

        while self.info.regs.gcc0().read().rdy().is_gain_cal_not_valid() {}

        let mut gcca = self.info.regs.gcc0().read().gain_cal().bits() as u32;
        if gcca & 0x8000 != 0 {
            gcca |= !0xFFFF;
        }

        let gcra = 131072.0 / (131072.0 - gcca as f32);

        // Write to GCR0
        self.info
            .regs
            .gcr0()
            .write(|w| unsafe { w.bits(self.get_gain_conv_result(gcra)) });

        self.info.regs.gcr0().modify(|_, w| w.rdy().set_bit());

        // Wait for calibration to complete (polling status register)
        while self.info.regs.stat().read().cal_rdy().is_not_set() {}
    }

    fn set_conv_command_config_inner(&self, index: usize, config: &ConvCommandConfig) -> Result<()> {
        let (cmdl, cmdh) = match index {
            1 => (self.info.regs.cmdl1(), self.info.regs.cmdh1()),
            2 => (self.info.regs.cmdl2(), self.info.regs.cmdh2()),
            3 => (self.info.regs.cmdl3(), self.info.regs.cmdh3()),
            4 => (self.info.regs.cmdl4(), self.info.regs.cmdh4()),
            5 => (self.info.regs.cmdl5(), self.info.regs.cmdh5()),
            6 => (self.info.regs.cmdl6(), self.info.regs.cmdh6()),
            7 => (self.info.regs.cmdl7(), self.info.regs.cmdh7()),
            _ => return Err(Error::InvalidConfig),
        };

        cmdl.write(|w| {
            unsafe {
                w.adch().bits(self.channel_idx);
            }
            w.mode().variant(config.conversion_resolution_mode)
        });

        cmdh.write(|w| {
            w.next().variant(config.chained_next_command_number);
            unsafe {
                w.loop_().bits(config.loop_count);
            }
            w.avgs().variant(config.hardware_average_mode);
            w.sts().variant(config.sample_time_mode);
            w.cmpen().variant(config.hardware_compare_mode);
            w.wait_trig().bit(config.enable_wait_trigger);
            w.lwi().bit(config.enable_auto_channel_increment);
            w
        });

        Ok(())
    }

    fn set_conv_trigger_config_inner(&self, trigger_id: usize, config: &ConvTriggerConfig) -> Result<()> {
        // 0..4 are valid
        if trigger_id >= 4 {
            return Err(Error::InvalidConfig);
        }

        let tctrl = &self.info.regs.tctrl(trigger_id);

        tctrl.write(|w| {
            w.tcmd().variant(config.target_command_id);
            unsafe {
                w.tdly().bits(config.delay_power);
            }
            w.tpri().variant(config.priority);
            if config.enable_hardware_trigger {
                w.hten().enabled()
            } else {
                w
            }
        });

        Ok(())
    }

    /// Get conversion result from FIFO.
    ///
    /// Reads and returns the next conversion result from the FIFO.
    /// Returns `None` if the FIFO is empty.
    ///
    /// # Returns
    /// - `Some(ConvResult)` if a result is available
    /// - `Err(Error::FifoEmpty)` if the FIFO is empty
    fn get_conv_result_inner(&self) -> Result<ConvResult> {
        let fifo = self.info.regs.resfifo0().read();
        if !fifo.valid().is_valid() {
            return Err(Error::FifoEmpty);
        }

        Ok(ConvResult {
            command_id_source: fifo.cmdsrc().bits(),
            loop_count_index: fifo.loopcnt().bits(),
            trigger_id_source: fifo.tsrc().bits(),
            conv_value: fifo.d().bits(),
        })
    }
}

impl<T: Instance> Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        T::info().regs.ie().modify(|_r, w| w.fwmie0().clear_bit());
        T::info().wait_cell.wake();
    }
}

mod sealed {
    /// Seal a trait
    pub trait Sealed {}

    /// Sealed pin trait
    pub trait SealedAdcPin<T: super::Instance> {}
}

struct Info {
    regs: &'static pac::adc0::RegisterBlock,
    wait_cell: &'static WaitCell,
}

trait SealedInstance {
    fn info() -> Info;
}

/// ADC Instance
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + Gate<MrccPeriphConfig = AdcConfig> {
    /// Interrupt for this ADC instance.
    type Interrupt: Interrupt;
}

macro_rules! impl_instance {
    ($($n:expr),*) => {
        $(
            paste!{
                impl SealedInstance for crate::peripherals::[<ADC $n>] {
                    fn info() -> Info {
                        static WAIT_CELL: WaitCell = WaitCell::new();
                        Info {
                            regs: unsafe { &*pac::[<Adc $n>]::ptr() },
                            wait_cell: &WAIT_CELL,
                        }
                    }
                }

                impl Instance for crate::peripherals::[<ADC $n>] {
                    type Interrupt = crate::interrupt::typelevel::[<ADC $n>];
                }
            }
        )*
    };
}

impl_instance!(0, 1, 2, 3);

pub trait AdcPin<T: Instance>: sealed::SealedAdcPin<T> + GpioPin + PeripheralType {
    /// The channel to be used
    fn channel(&self) -> u8;

    /// Set the given pin to the correct muxing state
    fn mux(&self);
}

/// Driver mode.
#[allow(private_bounds)]
pub trait Mode: sealed::Sealed {}

/// Blocking mode.
pub struct Blocking;
impl sealed::Sealed for Blocking {}
impl Mode for Blocking {}

/// Async mode.
pub struct Async;
impl sealed::Sealed for Async {}
impl Mode for Async {}

macro_rules! impl_pin {
    ($pin:ident, $peri:ident, $func:ident, $channel:literal) => {
        impl sealed::SealedAdcPin<crate::peripherals::$peri> for crate::peripherals::$pin {}

        impl AdcPin<crate::peripherals::$peri> for crate::peripherals::$pin {
            #[inline]
            fn channel(&self) -> u8 {
                $channel
            }

            #[inline]
            fn mux(&self) {
                self.set_pull(crate::gpio::Pull::Disabled);
                self.set_slew_rate(crate::gpio::SlewRate::Fast.into());
                self.set_drive_strength(crate::gpio::DriveStrength::Normal.into());
                self.set_function(crate::pac::port0::pcr0::Mux::$func);
            }
        }
    };
}

impl_pin!(P2_0, ADC0, Mux0, 0);
impl_pin!(P2_4, ADC0, Mux0, 1);
impl_pin!(P2_15, ADC0, Mux0, 2);
impl_pin!(P2_3, ADC0, Mux0, 3);
impl_pin!(P2_2, ADC0, Mux0, 4);
impl_pin!(P2_12, ADC0, Mux0, 5);
impl_pin!(P2_16, ADC0, Mux0, 6);
impl_pin!(P2_7, ADC0, Mux0, 7);
impl_pin!(P0_18, ADC0, Mux0, 8);
impl_pin!(P0_19, ADC0, Mux0, 9);
impl_pin!(P0_20, ADC0, Mux0, 10);
impl_pin!(P0_21, ADC0, Mux0, 11);
impl_pin!(P0_22, ADC0, Mux0, 12);
impl_pin!(P0_23, ADC0, Mux0, 13);
impl_pin!(P0_3, ADC0, Mux0, 14);
impl_pin!(P0_6, ADC0, Mux0, 15);
impl_pin!(P1_0, ADC0, Mux0, 16);
impl_pin!(P1_1, ADC0, Mux0, 17);
impl_pin!(P1_2, ADC0, Mux0, 18);
impl_pin!(P1_3, ADC0, Mux0, 19);
impl_pin!(P1_4, ADC0, Mux0, 20);
impl_pin!(P1_5, ADC0, Mux0, 21);
impl_pin!(P1_6, ADC0, Mux0, 22);
impl_pin!(P1_7, ADC0, Mux0, 23);

// ???
// impl_pin!(P1_10, ADC0, Mux0, 255);

impl_pin!(P2_1, ADC1, Mux0, 0);
impl_pin!(P2_5, ADC1, Mux0, 1);
impl_pin!(P2_19, ADC1, Mux0, 2);
impl_pin!(P2_6, ADC1, Mux0, 3);
impl_pin!(P2_3, ADC1, Mux0, 4);
impl_pin!(P2_13, ADC1, Mux0, 5);
impl_pin!(P2_17, ADC1, Mux0, 6);
impl_pin!(P2_7, ADC1, Mux0, 7);
impl_pin!(P1_10, ADC1, Mux0, 8);
impl_pin!(P1_11, ADC1, Mux0, 9);
impl_pin!(P1_12, ADC1, Mux0, 10);
impl_pin!(P1_13, ADC1, Mux0, 11);
impl_pin!(P1_14, ADC1, Mux0, 12);
impl_pin!(P1_15, ADC1, Mux0, 13);
// ???
// impl_pin!(P1_16, ADC1, Mux0, 255);
// impl_pin!(P1_17, ADC1, Mux0, 255);
// impl_pin!(P1_18, ADC1, Mux0, 255);
// impl_pin!(P1_19, ADC1, Mux0, 255);
// ???
impl_pin!(P3_31, ADC1, Mux0, 20);
impl_pin!(P3_30, ADC1, Mux0, 21);
impl_pin!(P3_29, ADC1, Mux0, 22);

impl_pin!(P2_4, ADC2, Mux0, 0);
impl_pin!(P2_10, ADC2, Mux0, 1);
impl_pin!(P4_4, ADC2, Mux0, 2);
// impl_pin!(P2_24, ADC2, Mux0, 255); ???
impl_pin!(P2_16, ADC2, Mux0, 4);
impl_pin!(P2_12, ADC2, Mux0, 5);
impl_pin!(P2_20, ADC2, Mux0, 6);
impl_pin!(P2_7, ADC2, Mux0, 7);
impl_pin!(P0_2, ADC2, Mux0, 8);
// ???
// impl_pin!(P0_4, ADC2, Mux0, 255);
// impl_pin!(P0_5, ADC2, Mux0, 255);
// impl_pin!(P0_6, ADC2, Mux0, 255);
// impl_pin!(P0_7, ADC2, Mux0, 255);
// impl_pin!(P0_12, ADC2, Mux0, 255);
// impl_pin!(P0_13, ADC2, Mux0, 255);
// ???
impl_pin!(P0_14, ADC2, Mux0, 14);
impl_pin!(P0_15, ADC2, Mux0, 15);
// ???
// impl_pin!(P4_0, ADC2, Mux0, 255);
// impl_pin!(P4_1, ADC2, Mux0, 255);
// ???
impl_pin!(P4_2, ADC2, Mux0, 18);
impl_pin!(P4_3, ADC2, Mux0, 19);
//impl_pin!(P4_4, ADC2, Mux0, 20); // Conflit with ADC2_A3 and ADC2_A20 using the same pin
impl_pin!(P4_5, ADC2, Mux0, 21);
impl_pin!(P4_6, ADC2, Mux0, 22);
impl_pin!(P4_7, ADC2, Mux0, 23);

impl_pin!(P2_5, ADC3, Mux0, 0);
impl_pin!(P2_11, ADC3, Mux0, 1);
impl_pin!(P2_23, ADC3, Mux0, 2);
// impl_pin!(P2_25, ADC3, Mux0, 255); // ???
impl_pin!(P2_17, ADC3, Mux0, 4);
impl_pin!(P2_13, ADC3, Mux0, 5);
impl_pin!(P2_21, ADC3, Mux0, 6);
impl_pin!(P2_7, ADC3, Mux0, 7);
// ???
// impl_pin!(P3_2, ADC3, Mux0, 255);
// impl_pin!(P3_3, ADC3, Mux0, 255);
// impl_pin!(P3_4, ADC3, Mux0, 255);
// impl_pin!(P3_5, ADC3, Mux0, 255);
// ???
impl_pin!(P3_6, ADC3, Mux0, 12);
impl_pin!(P3_7, ADC3, Mux0, 13);
impl_pin!(P3_12, ADC3, Mux0, 14);
impl_pin!(P3_13, ADC3, Mux0, 15);
impl_pin!(P3_14, ADC3, Mux0, 16);
impl_pin!(P3_15, ADC3, Mux0, 17);
impl_pin!(P3_20, ADC3, Mux0, 18);
impl_pin!(P3_21, ADC3, Mux0, 19);
impl_pin!(P3_22, ADC3, Mux0, 20);
// ???
// impl_pin!(P3_23, ADC3, Mux0, 255);
// impl_pin!(P3_24, ADC3, Mux0, 255);
// impl_pin!(P3_25, ADC3, Mux0, 255);
// ???
