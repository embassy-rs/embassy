//! ADC driver
use core::future::Future;
use core::marker::PhantomData;
use core::ops::RangeInclusive;

use embassy_hal_internal::{Peri, PeripheralType};
use maitake_sync::WaitCell;
use paste::paste;

use crate::clocks::periph_helpers::{AdcClockSel, AdcConfig, Div4};
use crate::clocks::{ClockError, Gate, PoweredClock, WakeGuard, enable_and_reset};
use crate::gpio::{AnyPin, GpioPin, SealedPin};
use crate::interrupt::typelevel::{Handler, Interrupt};
use crate::pac::adc::vals::{
    Avgs, CalAvgs, CalRdy, CalReq, Calofs, Cmpen, Dozen, Gcc0Rdy, HptExdi, Loop as HwLoop, Mode as ConvMode, Next,
    Pwrsel, Refsel, Rst, Rstfifo0, Sts, Tcmd, Tpri, Tprictrl,
};
use crate::pac::port::vals::Mux;
use crate::pac::{self};

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
    pub calibration_average_mode: CalAvgs,

    /// When true, the ADC analog circuits are pre-enabled and ready to execute
    /// conversions without startup delays (at the cost of higher DC
    /// current consumption).
    pub power_pre_enabled: bool,

    /// Power-up delay value (in ADC clock cycles)
    pub power_up_delay: u8,

    /// Reference voltage source selection
    pub reference_voltage_source: Refsel,

    /// Power configuration selection.
    pub power_level_mode: Pwrsel,

    /// Trigger priority policy for handling multiple triggers
    pub trigger_priority_policy: TriggerPriorityPolicy,

    /// Controls the duration of pausing during command execution
    /// sequencing. The pause delay is a count of (convPauseDelay*4)
    /// ADCK cycles.
    ///
    /// The available value range is in 9-bit.
    /// When None, the pausing function is not enabled
    pub conv_pause_delay: Option<u16>,

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
            calibration_average_mode: CalAvgs::NO_AVERAGE,
            power_pre_enabled: false,
            power_up_delay: 0x80,
            reference_voltage_source: Refsel::OPTION_1,
            power_level_mode: Pwrsel::LOWEST,
            trigger_priority_policy: TriggerPriorityPolicy::ConvPreemptImmediatelyNotAutoResumed,
            conv_pause_delay: None,
            power: PoweredClock::NormalEnabledDeepSleepDisabled,
            source: AdcClockSel::FroLfDiv,
            div: Div4::no_div(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CommandId {
    Cmd1 = 1,
    Cmd2 = 2,
    Cmd3 = 3,
    Cmd4 = 4,
    Cmd5 = 5,
    Cmd6 = 6,
    Cmd7 = 7,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Compare {
    /// Do not perform compare operation. Always store the conversion result to the FIFO.
    Disabled,
    /// Store conversion result to FIFO at end
    /// of averaging only if compare is true. If compare is false do not store
    /// the result to the FIFO. In either the true or false condition, the LOOP
    /// setting is considered and increments the LOOP counter before deciding
    /// whether the current command has completed or additional LOOP
    /// iterations are required.
    StoreIf(CompareFunction),
    /// Store conversion result to FIFO at end of
    /// averaging only if compare is true. Once the true condition is found the
    /// LOOP setting is considered and increments the LOOP counter before
    /// deciding whether the current command has completed or additional
    /// LOOP iterations are required. If the compare is false do not store the
    /// result to the FIFO. The conversion is repeated without consideration of
    /// LOOP setting and does not increment the LOOP counter.
    SkipUntil(CompareFunction),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompareFunction {
    OutsideRange(RangeInclusive<u16>),
    LessThan(u16),
    GreaterThan(u16),
    InsideRange(RangeInclusive<u16>),
}

impl CompareFunction {
    /// Get the CVL & CVH values
    fn get_vals(&self) -> (u16, u16) {
        match self {
            CompareFunction::OutsideRange(range) => {
                assert!(!range.is_empty());
                (*range.start(), *range.end())
            }
            CompareFunction::LessThan(val) => (*val, u16::MAX),
            CompareFunction::GreaterThan(val) => (0, *val),
            CompareFunction::InsideRange(range) => {
                assert!(!range.is_empty());
                (*range.end(), *range.start())
            }
        }
    }
}

/// A command that can be executed by the ADC
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Command {
    /// When true, if
    increment_channel: bool,
    /// The number of times the command is repeated. Range = `0..=15`.
    /// If [Self::increment_channel] is true, the repeats happen on different channels
    loop_count: u8,

    config: CommandConfig,
}

impl Command {
    /// A command that does one conversion on a channel
    pub fn new_single(channel: (), config: CommandConfig) -> Self {
        Self {
            increment_channel: false,
            loop_count: 0,
            config,
        }
    }

    /// A command that does multiple conversions on a channel.
    /// - `num_loops`: The amount of times the command is repeated. Range: `1..=15`
    pub fn new_looping(channel: (), num_loops: u8, config: CommandConfig) -> Result<Self, Error> {
        if !(1..=15).contains(&num_loops) {
            return Err(Error::InvalidConfig);
        }

        Ok(Self {
            increment_channel: false,
            loop_count: num_loops,
            config,
        })
    }

    /// A command that does multiple conversions on multiple channels
    pub fn new_multichannel<const N: usize>(channels: [(); N], config: CommandConfig) -> Self {}
}

/// Configuration for a conversion command
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandConfig {
    /// The command that will be executed next.
    ///
    /// If None, conversion will end
    pub chained_command: Option<CommandId>,
    /// The averaging done on a conversion
    pub averaging: Avgs,
    /// The sampling time of a conversion
    pub sample_time: Sts,
    /// The compare function being used
    pub compare: Compare,
    /// The resolution of a conversion
    pub resolution: ConvMode,
    /// When false, the command will not wait for a trigger once the command sequence has been started.
    /// When true, a trigger is required before the command is started.
    pub wait_for_trigger: bool,
}

impl Default for CommandConfig {
    fn default() -> Self {
        Self {
            chained_command: None,
            averaging: Avgs::NO_AVERAGE,
            sample_time: Sts::SAMPLE_3P5,
            compare: Compare::Disabled,
            resolution: ConvMode::DATA_12_BITS,
            wait_for_trigger: false,
        }
    }
}

/// Configuration for a conversion trigger.
///
/// Defines how a trigger initiates ADC conversions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Trigger {
    pub target_command_id: CommandId,
    pub delay_power: u8,
    pub priority: Tpri,
    pub enable_hardware_trigger: bool,
}

impl Default for Trigger {
    fn default() -> Self {
        Trigger {
            target_command_id: CommandId::Cmd1,
            delay_power: 0,
            priority: Tpri::HIGHEST_PRIORITY,
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
    pub command: CommandId,
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
    pin: Peri<'a, AnyPin>,
    channel: u8,
    info: &'static Info,
    _wg: Option<WakeGuard>,
}

impl<'a> Adc<'a, Blocking> {
    /// Create a new blocking instance of the ADC driver.
    ///
    /// Any external pin will be placed into Disabled state upon Drop.
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
        self.info.regs().ie().modify(|w| w.0 |= mask);
    }

    /// Disable ADC interrupts.
    ///
    /// Disables the interrupt sources specified in the bitmask.
    ///
    /// # Arguments
    /// * `mask` - Bitmask of interrupt sources to disable
    pub fn disable_interrupt(&mut self, mask: u32) {
        self.info.regs().ie().modify(|w| w.0 &= !mask);
    }

    pub fn set_fifo_watermark(&mut self, watermark: u8) -> Result<()> {
        if watermark > 0b111 {
            return Err(Error::InvalidConfig);
        }
        self.info.regs().fctrl0().modify(|w| w.set_fwmark(watermark));
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
        self.info.regs().swtrig().write(|w| w.0 = trigger_id_mask as u32);
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
    pub fn set_conv_command_config(&self, index: usize, config: &Command) -> Result<()> {
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
    pub fn set_conv_trigger_config(&self, trigger_id: usize, config: &Trigger) -> Result<()> {
        self.set_conv_trigger_config_inner(trigger_id, config)
    }

    /// Reset the FIFO buffer.
    ///
    /// Clears all pending conversion results from the FIFO.
    pub fn do_reset_fifo(&self) {
        self.info
            .regs()
            .ctrl()
            .modify(|w| w.set_rstfifo0(Rstfifo0::TRIGGER_RESET));
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
    /// Any external pin will be placed into Disabled state upon Drop.
    pub fn new_async<T: Instance>(
        _inst: Peri<'a, T>,
        pin: Peri<'a, impl AdcPin<T>>,
        _irq: impl crate::interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'a,
        config: Config,
    ) -> Result<Self> {
        let adc = Self::new_inner(_inst, pin, config)?;

        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        let cfg = Command {
            chained_command: None,
            increment_channel: false,
            loop_: Loop::None,
            averaging: Avgs::NO_AVERAGE,
            sample_time: Sts::SAMPLE_3P5,
            hardware_compare_mode: Cmpen::DISABLED_ALWAYS_STORE_RESULT,
            hardware_compare_value_high: 0,
            hardware_compare_value_low: 0,
            conversion_resolution_mode: ConvMode::DATA_16_BITS,
            enable_wait_trigger: false,
        };

        // We always use command 1, so this cannot fail
        _ = adc.set_conv_command_config_inner(1, &cfg);

        let cfg = Trigger {
            target_command_id: CommandId::Cmd1,
            delay_power: 0,
            priority: Tpri::HIGHEST_PRIORITY,
            enable_hardware_trigger: false,
        };

        // We always use trigger 0, so this cannot fail
        _ = adc.set_conv_trigger_config_inner(0, &cfg);

        // We always set the watermark to 0 (trigger when 1 is available)
        adc.info.regs().fctrl0().modify(|w| w.set_fwmark(0));

        Ok(adc)
    }

    /// Set the number of averages
    pub fn set_averages(&mut self, avgs: Avgs) {
        // TODO: we should probably return a result or wait for idle?
        // "A write to a CMD buffer while that CMD buffer is controlling the ADC operation may cause unpredictable behavior."
        self.info.regs().cmdh1().modify(|w| w.set_avgs(avgs));
    }

    /// Set the sample time
    pub fn set_sample_time(&mut self, st: Sts) {
        // TODO: we should probably return a result or wait for idle?
        // "A write to a CMD buffer while that CMD buffer is controlling the ADC operation may cause unpredictable behavior."
        self.info.regs().cmdh1().modify(|w| w.set_sts(st));
    }

    pub fn set_resolution(&mut self, mode: ConvMode) {
        // TODO: we should probably return a result or wait for idle?
        // "A write to a CMD buffer while that CMD buffer is controlling the ADC operation may cause unpredictable behavior."
        self.info.regs().cmdl1().modify(|w| w.set_mode(mode));
    }

    fn wait_idle(&mut self) -> impl Future<Output = core::result::Result<(), maitake_sync::Closed>> + use<'_> {
        self.info
            .wait_cell()
            .wait_for(|| !self.info.regs().ie().read().fwmie0())
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
        self.info
            .regs()
            .ctrl()
            .modify(|w| w.set_rstfifo0(Rstfifo0::TRIGGER_RESET));

        // Trigger a new conversion
        self.info.regs().ie().modify(|w| w.set_fwmie0(true));
        self.info.regs().swtrig().write(|w| w.set_swt(0, true));

        // Wait for completion
        _ = self.wait_idle().await;

        self.get_conv_result_inner().map(|r| r.conv_value)
    }
}

impl<'a, M: Mode> Adc<'a, M> {
    /// Internal initialization function shared by `new_async` and `new_blocking`.
    fn new_inner<T: Instance>(_inst: Peri<'a, T>, pin: Peri<'a, impl AdcPin<T>>, config: Config) -> Result<Self> {
        let info = T::info();
        let adc = info.regs();

        let parts = unsafe {
            enable_and_reset::<T>(&AdcConfig {
                power: config.power,
                source: config.source,
                div: config.div,
            })
            .map_err(Error::ClockSetup)?
        };

        pin.mux();

        /* Reset the module. */
        adc.ctrl().modify(|w| w.set_rst(Rst::HELD_IN_RESET));
        adc.ctrl().modify(|w| w.set_rst(Rst::RELEASED_FROM_RESET));

        adc.ctrl().modify(|w| w.set_rstfifo0(Rstfifo0::TRIGGER_RESET));

        /* Disable the module before setting configuration. */
        adc.ctrl().modify(|w| w.set_adcen(false));

        /* Configure the module generally. */
        adc.ctrl().modify(|w| {
            w.set_dozen(if config.enable_in_doze_mode {
                Dozen::ENABLED
            } else {
                Dozen::DISABLED
            })
        });

        /* Set calibration average mode. */
        adc.ctrl().modify(|w| w.set_cal_avgs(config.calibration_average_mode));

        adc.cfg().write(|w| {
            w.set_pwren(config.power_pre_enabled);

            w.set_pudly(config.power_up_delay);
            w.set_refsel(config.reference_voltage_source);
            w.set_pwrsel(config.power_level_mode);
            w.set_tprictrl(match config.trigger_priority_policy {
                TriggerPriorityPolicy::ConvPreemptSoftlyNotAutoResumed
                | TriggerPriorityPolicy::ConvPreemptSoftlyAutoRestarted
                | TriggerPriorityPolicy::ConvPreemptSoftlyAutoResumed => Tprictrl::FINISH_CURRENT_ON_PRIORITY,
                TriggerPriorityPolicy::ConvPreemptSubsequentlyNotAutoResumed
                | TriggerPriorityPolicy::ConvPreemptSubsequentlyAutoRestarted
                | TriggerPriorityPolicy::ConvPreemptSubsequentlyAutoResumed => Tprictrl::FINISH_SEQUENCE_ON_PRIORITY,
                _ => Tprictrl::ABORT_CURRENT_ON_PRIORITY,
            });
            w.set_tres(matches!(
                config.trigger_priority_policy,
                TriggerPriorityPolicy::ConvPreemptImmediatelyAutoRestarted
                    | TriggerPriorityPolicy::ConvPreemptSoftlyAutoRestarted
                    | TriggerPriorityPolicy::ConvPreemptImmediatelyAutoResumed
                    | TriggerPriorityPolicy::ConvPreemptSoftlyAutoResumed
                    | TriggerPriorityPolicy::ConvPreemptSubsequentlyAutoRestarted
                    | TriggerPriorityPolicy::ConvPreemptSubsequentlyAutoResumed
            ));
            w.set_tcmdres(matches!(
                config.trigger_priority_policy,
                TriggerPriorityPolicy::ConvPreemptImmediatelyAutoResumed
                    | TriggerPriorityPolicy::ConvPreemptSoftlyAutoResumed
                    | TriggerPriorityPolicy::ConvPreemptSubsequentlyAutoResumed
                    | TriggerPriorityPolicy::TriggerPriorityExceptionDisabled
            ));
            w.set_hpt_exdi(match config.trigger_priority_policy {
                TriggerPriorityPolicy::TriggerPriorityExceptionDisabled => HptExdi::DISABLED,
                _ => HptExdi::ENABLED,
            });
        });

        if let Some(pause_delay) = config.conv_pause_delay {
            adc.pause().write(|w| {
                w.set_pauseen(true);
                w.set_pausedly(pause_delay);
            });
        } else {
            adc.pause().write(|w| w.set_pauseen(false));
        }

        adc.fctrl0().write(|w| w.set_fwmark(0));

        // Enable ADC
        adc.ctrl().modify(|w| w.set_adcen(true));

        Ok(Self {
            _inst: PhantomData,
            channel: pin.channel(),
            pin: pin.into(),
            info,
            _wg: parts.wake_guard,
        })
    }

    /// Perform offset calibration.
    /// Waits for calibration to complete before returning.
    pub fn do_offset_calibration(&self) {
        // Enable calibration mode
        self.info
            .regs()
            .ctrl()
            .modify(|w| w.set_calofs(Calofs::OFFSET_CALIBRATION_REQUEST_PENDING));

        // Wait for calibration to complete (polling status register)
        while self.info.regs().stat().read().cal_rdy() == CalRdy::NOT_SET {}
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
            .regs()
            .ctrl()
            .modify(|w| w.set_cal_req(CalReq::CALIBRATION_REQUEST_PENDING));

        while self.info.regs().gcc0().read().rdy() == Gcc0Rdy::GAIN_CAL_NOT_VALID {}

        let mut gcca = self.info.regs().gcc0().read().gain_cal() as u32;
        if gcca & 0x8000 != 0 {
            gcca |= !0xFFFF;
        }

        let gcra = 131072.0 / (131072.0 - gcca as f32);

        // Write to GCR0
        self.info.regs().gcr0().write(|w| w.0 = self.get_gain_conv_result(gcra));

        self.info.regs().gcr0().modify(|w| w.set_rdy(true));

        // Wait for calibration to complete (polling status register)
        while self.info.regs().stat().read().cal_rdy() == CalRdy::NOT_SET {}
    }

    fn set_conv_command_config_inner(&self, index: usize, config: &Command) -> Result<()> {
        let (cmdl, cmdh) = match index {
            1 => (self.info.regs().cmdl1(), self.info.regs().cmdh1()),
            2 => (self.info.regs().cmdl2(), self.info.regs().cmdh2()),
            3 => (self.info.regs().cmdl3(), self.info.regs().cmdh3()),
            4 => (self.info.regs().cmdl4(), self.info.regs().cmdh4()),
            5 => (self.info.regs().cmdl5(), self.info.regs().cmdh5()),
            6 => (self.info.regs().cmdl6(), self.info.regs().cmdh6()),
            7 => (self.info.regs().cmdl7(), self.info.regs().cmdh7()),
            _ => return Err(Error::InvalidConfig),
        };

        cmdl.write(|w| {
            w.set_adch(self.channel);
            w.set_mode(config.conversion_resolution_mode)
        });

        cmdh.write(|w| {
            w.set_next(config.chained_command);
            w.set_loop_(config.loop_);
            w.set_avgs(config.averaging);
            w.set_sts(config.sample_time);
            w.set_cmpen(config.hardware_compare_mode);
            w.set_wait_trig(config.enable_wait_trigger);
            w.set_lwi(config.increment_channel);
        });

        Ok(())
    }

    fn set_conv_trigger_config_inner(&self, trigger_id: usize, config: &Trigger) -> Result<()> {
        // 0..4 are valid
        if trigger_id >= 4 {
            return Err(Error::InvalidConfig);
        }

        let tctrl = &self.info.regs().tctrl(trigger_id);

        tctrl.write(|w| {
            w.set_tcmd(config.target_command_id);
            w.set_tdly(config.delay_power);
            w.set_tpri(config.priority);
            if config.enable_hardware_trigger {
                w.set_hten(true);
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
        let fifo = self.info.regs().resfifo0().read();
        if !fifo.valid() {
            return Err(Error::FifoEmpty);
        }

        Ok(ConvResult {
            command: fifo.cmdsrc() as u8,
            loop_count_index: fifo.loopcnt() as u8,
            trigger_id_source: fifo.tsrc() as u8,
            conv_value: fifo.d(),
        })
    }
}

impl<'a, M: Mode> Drop for Adc<'a, M> {
    fn drop(&mut self) {
        self.pin.set_as_disabled();
    }
}

impl<T: Instance> Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        T::PERF_INT_INCR();
        T::info().regs().ie().modify(|w| w.set_fwmie0(false));
        T::info().wait_cell().wake();
    }
}

mod sealed {
    /// Seal a trait
    pub trait Sealed {}

    /// Sealed pin trait
    pub trait SealedAdcPin<T: super::Instance> {}
}

struct Info {
    regs: pac::adc::Adc,
    wait_cell: WaitCell,
}

unsafe impl Sync for Info {}

impl Info {
    #[inline(always)]
    fn regs(&self) -> pac::adc::Adc {
        self.regs
    }

    #[inline(always)]
    fn wait_cell(&self) -> &WaitCell {
        &self.wait_cell
    }
}

trait SealedInstance {
    fn info() -> &'static Info;
}

/// ADC Instance
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + Gate<MrccPeriphConfig = AdcConfig> {
    /// Interrupt for this ADC instance.
    type Interrupt: Interrupt;
    const PERF_INT_INCR: fn();
}

macro_rules! impl_instance {
    ($($n:expr),*) => {
        $(
            paste!{
                impl SealedInstance for crate::peripherals::[<ADC $n>] {
                    fn info() -> &'static Info {
                        static INFO: Info =
                        Info {
                            regs: pac::[<ADC $n>],
                            wait_cell: WaitCell::new(),
                        };
                        &INFO
                    }
                }

                impl Instance for crate::peripherals::[<ADC $n>] {
                    type Interrupt = crate::interrupt::typelevel::[<ADC $n>];
                    const PERF_INT_INCR: fn() = crate::perf_counters::[<incr_interrupt_adc $n>];
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
    ($pin:ident, $peri:ident, $channel:literal) => {
        impl sealed::SealedAdcPin<crate::peripherals::$peri> for crate::peripherals::$pin {}

        impl AdcPin<crate::peripherals::$peri> for crate::peripherals::$pin {
            #[inline]
            fn channel(&self) -> u8 {
                $channel
            }

            #[inline]
            fn mux(&self) {
                // Set to digital GPIO with input buffer disabled and no pull-ups.
                // TODO also clear digital output value?
                self.set_pull(crate::gpio::Pull::Disabled);
                self.set_slew_rate(crate::gpio::SlewRate::Fast.into());
                self.set_drive_strength(crate::gpio::DriveStrength::Normal.into());
                self.set_function(Mux::MUX0);
            }
        }
    };
}

impl_pin!(P2_0, ADC0, 0);
impl_pin!(P2_4, ADC0, 1);
impl_pin!(P2_15, ADC0, 2);
impl_pin!(P2_3, ADC0, 3);
impl_pin!(P2_2, ADC0, 4);
impl_pin!(P2_12, ADC0, 5);
impl_pin!(P2_16, ADC0, 6);
impl_pin!(P2_7, ADC0, 7);
impl_pin!(P0_18, ADC0, 8);
impl_pin!(P0_19, ADC0, 9);
impl_pin!(P0_20, ADC0, 10);
impl_pin!(P0_21, ADC0, 11);
impl_pin!(P0_22, ADC0, 12);
impl_pin!(P0_23, ADC0, 13);
#[cfg(feature = "jtag-extras-as-gpio")]
impl_pin!(P0_3, ADC0, 14);
#[cfg(feature = "jtag-extras-as-gpio")]
impl_pin!(P0_6, ADC0, 15);
impl_pin!(P1_0, ADC0, 16);
impl_pin!(P1_1, ADC0, 17);
impl_pin!(P1_2, ADC0, 18);
impl_pin!(P1_3, ADC0, 19);
impl_pin!(P1_4, ADC0, 20);
impl_pin!(P1_5, ADC0, 21);
impl_pin!(P1_6, ADC0, 22);
impl_pin!(P1_7, ADC0, 23);

// ???
// impl_pin!(P1_10, ADC0, 255);

impl_pin!(P2_1, ADC1, 0);
impl_pin!(P2_5, ADC1, 1);
impl_pin!(P2_19, ADC1, 2);
impl_pin!(P2_6, ADC1, 3);
impl_pin!(P2_3, ADC1, 4);
impl_pin!(P2_13, ADC1, 5);
impl_pin!(P2_17, ADC1, 6);
impl_pin!(P2_7, ADC1, 7);
impl_pin!(P1_10, ADC1, 8);
impl_pin!(P1_11, ADC1, 9);
impl_pin!(P1_12, ADC1, 10);
impl_pin!(P1_13, ADC1, 11);
impl_pin!(P1_14, ADC1, 12);
impl_pin!(P1_15, ADC1, 13);
// ???
// impl_pin!(P1_16, ADC1, 255);
// impl_pin!(P1_17, ADC1, 255);
// impl_pin!(P1_18, ADC1, 255);
// impl_pin!(P1_19, ADC1, 255);
// ???
impl_pin!(P3_31, ADC1, 20);
impl_pin!(P3_30, ADC1, 21);
impl_pin!(P3_29, ADC1, 22);

impl_pin!(P2_4, ADC2, 0);
impl_pin!(P2_10, ADC2, 1);
impl_pin!(P4_4, ADC2, 2);
// impl_pin!(P2_24, ADC2, 255); ???
impl_pin!(P2_16, ADC2, 4);
impl_pin!(P2_12, ADC2, 5);
impl_pin!(P2_20, ADC2, 6);
impl_pin!(P2_7, ADC2, 7);
#[cfg(feature = "swd-swo-as-gpio")]
impl_pin!(P0_2, ADC2, 8);
// ???
// impl_pin!(P0_4, ADC2, 255);
// impl_pin!(P0_5, ADC2, 255);
// impl_pin!(P0_6, ADC2, 255);
// impl_pin!(P0_7, ADC2, 255);
// impl_pin!(P0_12, ADC2, 255);
// impl_pin!(P0_13, ADC2, 255);
// ???
impl_pin!(P0_14, ADC2, 14);
impl_pin!(P0_15, ADC2, 15);
// ???
// impl_pin!(P4_0, ADC2, 255);
// impl_pin!(P4_1, ADC2, 255);
// ???
impl_pin!(P4_2, ADC2, 18);
impl_pin!(P4_3, ADC2, 19);
//impl_pin!(P4_4, ADC2, 20); // Conflit with ADC2_A3 and ADC2_A20 using the same pin
impl_pin!(P4_5, ADC2, 21);
impl_pin!(P4_6, ADC2, 22);
impl_pin!(P4_7, ADC2, 23);

impl_pin!(P2_5, ADC3, 0);
impl_pin!(P2_11, ADC3, 1);
impl_pin!(P2_23, ADC3, 2);
// impl_pin!(P2_25, ADC3, 255); // ???
impl_pin!(P2_17, ADC3, 4);
impl_pin!(P2_13, ADC3, 5);
impl_pin!(P2_21, ADC3, 6);
impl_pin!(P2_7, ADC3, 7);
// ???
// impl_pin!(P3_2, ADC3, 255);
// impl_pin!(P3_3, ADC3, 255);
// impl_pin!(P3_4, ADC3, 255);
// impl_pin!(P3_5, ADC3, 255);
// ???
impl_pin!(P3_6, ADC3, 12);
impl_pin!(P3_7, ADC3, 13);
impl_pin!(P3_12, ADC3, 14);
impl_pin!(P3_13, ADC3, 15);
impl_pin!(P3_14, ADC3, 16);
impl_pin!(P3_15, ADC3, 17);
impl_pin!(P3_20, ADC3, 18);
impl_pin!(P3_21, ADC3, 19);
impl_pin!(P3_22, ADC3, 20);
// ???
// impl_pin!(P3_23, ADC3, 255);
// impl_pin!(P3_24, ADC3, 255);
// impl_pin!(P3_25, ADC3, 255);
// ???
