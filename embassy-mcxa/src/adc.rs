//! ADC driver
use core::marker::PhantomData;
use core::ops::{Deref, RangeInclusive};

use embassy_hal_internal::{Peri, PeripheralType};
use maitake_sync::WaitCell;
use nxp_pac::adc::vals::{AdcActive, TcompIe, TcompInt};
use paste::paste;

use crate::clocks::periph_helpers::{AdcClockSel, AdcConfig, Div4, PreEnableParts};
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

/// The reference voltage used by the ADC
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum ReferenceVoltage {
    #[default]
    VrefHReferencePin = 0b00,
    VrefI = 0b01,
    VddaAnaPin = 0b10,
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
    pub reference_voltage_source: ReferenceVoltage,

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
            reference_voltage_source: Default::default(),
            power_level_mode: Pwrsel::LOWEST,
            trigger_priority_policy: TriggerPriorityPolicy::ConvPreemptImmediatelyNotAutoResumed,
            conv_pause_delay: None,
            power: PoweredClock::NormalEnabledDeepSleepDisabled,
            source: AdcClockSel::FroLfDiv,
            div: Div4::no_div(),
        }
    }
}

/// The ID for a command
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum CommandId {
    Cmd1 = 1,
    Cmd2 = 2,
    Cmd3 = 3,
    Cmd4 = 4,
    Cmd5 = 5,
    Cmd6 = 6,
    Cmd7 = 7,
}

impl From<u8> for CommandId {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::Cmd1,
            2 => Self::Cmd2,
            3 => Self::Cmd3,
            4 => Self::Cmd4,
            5 => Self::Cmd5,
            6 => Self::Cmd6,
            7 => Self::Cmd7,
            _ => unreachable!(),
        }
    }
}

/// Select the compare functionality of the ADC
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

impl Compare {
    fn cmp_en(&self) -> Cmpen {
        match self {
            Compare::Disabled => Cmpen::DISABLED_ALWAYS_STORE_RESULT,
            Compare::StoreIf(_) => Cmpen::COMPARE_RESULT_STORE_IF_TRUE,
            Compare::SkipUntil(_) => Cmpen::COMPARE_RESULT_KEEP_CONVERTING_UNTIL_TRUE_STORE_IF_TRUE,
        }
    }

    /// Get the CVL & CVH values
    fn get_vals(&self) -> (u16, u16) {
        match self {
            Compare::Disabled => (0, 0),
            Compare::StoreIf(compare_function) | Compare::SkipUntil(compare_function) => compare_function.get_vals(),
        }
    }
}

/// Type that specifies the function used for the compare featue of the ADC.
///
/// This determines the `CVL` & `CVH` values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompareFunction {
    /// The compare will succeed when the value is *not* in the specified range
    OutsideRange(RangeInclusive<u16>),
    /// The compare will succeed when the value is lower than the specified value
    LessThan(u16),
    /// The compare will succeed when the value is higher than the specified value
    GreaterThan(u16),
    /// The compare will succeed when the value is in the specified range
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

enum Channels<'a, T> {
    Single([Peri<'a, AnyAdcPin<T>>; 1]),
    Multi(&'a [Peri<'a, AnyAdcPin<T>>]),
}

impl<'a, T> Deref for Channels<'a, T> {
    type Target = [Peri<'a, AnyAdcPin<T>>];

    fn deref(&self) -> &Self::Target {
        match self {
            Channels::Single(single) => single,
            Channels::Multi(multi) => multi,
        }
    }
}

/// A command that can be executed by the ADC
pub struct Command<'a, T> {
    /// When true, if
    increment_channel: bool,
    /// The number of times the command is run. Range = `0..=15`.
    /// If [Self::increment_channel] is true, the repeats happen on different channels
    loop_count: u8,

    config: CommandConfig,
    channels: Channels<'a, T>,
}

impl<'a, T: Instance> Command<'a, T> {
    /// A command that does one conversion on a channel
    pub fn new_single(channel: Peri<'a, impl Into<AnyAdcPin<T>> + PeripheralType>, config: CommandConfig) -> Self {
        Self {
            increment_channel: false,
            loop_count: 0,
            config,
            channels: Channels::Single([channel.into()]),
        }
    }

    /// A command that does multiple conversions on a channel.
    /// - `num_loops`: The amount of times the command is run. Range: `1..=16`
    pub fn new_looping(
        channel: Peri<'a, impl Into<AnyAdcPin<T>> + PeripheralType>,
        num_loops: u8,
        config: CommandConfig,
    ) -> Result<Self, Error> {
        if !(1..=16).contains(&num_loops) {
            return Err(Error::InvalidConfig);
        }

        Ok(Self {
            increment_channel: false,
            loop_count: num_loops - 1,
            config,
            channels: Channels::Single([channel.into()]),
        })
    }

    /// A command that does multiple conversions on multiple channels
    pub fn new_multichannel(channels: &'a [Peri<'a, AnyAdcPin<T>>], config: CommandConfig) -> Result<Self, Error> {
        if !(1..=15).contains(&channels.len()) {
            return Err(Error::InvalidConfig);
        }

        let mut next_channel = channels[0].channel + 1;
        for pin in channels.iter().skip(1) {
            if pin.channel != next_channel {
                return Err(Error::InvalidConfig);
            }
            next_channel = pin.channel + 1;
        }

        Ok(Self {
            increment_channel: true,
            loop_count: channels.len() as u8,
            config,
            channels: Channels::Multi(channels),
        })
    }
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
    /// The command that is triggered by this trigger
    pub target_command_id: CommandId,
    pub delay_power: u8,
    /// The priority level of the trigger
    pub priority: Tpri,
    pub enable_hardware_trigger: bool,
    pub resync: bool,
    pub synchronous: bool,
}

impl Default for Trigger {
    fn default() -> Self {
        Trigger {
            target_command_id: CommandId::Cmd1,
            delay_power: 0,
            priority: Tpri::HIGHEST_PRIORITY,
            enable_hardware_trigger: false,
            resync: false,
            synchronous: false,
        }
    }
}

/// ADC Error types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// FIFO is empty, no conversion result available
    FifoEmpty,
    /// FIFO is empty, but the adc is active and a new conversion will be ready soon
    FifoPending,
    /// Invalid configuration
    InvalidConfig,
    /// Too many commands
    TooManyCommands,
    /// Too many triggers
    TooManyTriggers,
    /// Tried to call a trigger that was not configured
    NoTrigger,
    /// Clock configuration error.
    ClockSetup(ClockError),
}

/// Result of an ADC conversion.
///
/// Contains the conversion value and metadata about the conversion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Conversion {
    /// The command that performed this conversion
    pub command: CommandId,
    /// For a looping command, the loop index. For a multichannel command, the channel index.
    pub loop_channel_index: u8,
    /// The trigger that triggered the command to run
    pub trigger_id_source: u8,
    /// The raw value from the ADC
    pub conv_value: u16,
}

/// ADC driver instance.
pub struct Adc<'a, M: Mode> {
    commands: &'a [Command<'a, ()>],
    num_triggers: u8,
    info: &'static Info,
    _wg: Option<WakeGuard>,
    _mode: PhantomData<M>,
}

impl<'a> Adc<'a, Blocking> {
    /// Create a new blocking instance of the ADC
    pub fn new_blocking<T: Instance>(
        _inst: Peri<'a, T>,
        commands: &'a [Command<'a, T>],
        triggers: &[Trigger],
        config: Config,
    ) -> Result<Self, Error> {
        // Safety:
        // We transmute the ADC instance to a `()`. This is fine since the `T` is only a phantomdata.
        // Because we're now in this function, we don't need this info anymore.
        let commands = unsafe { core::mem::transmute::<&[Command<'_, T>], &[Command<'_, ()>]>(commands) };

        let parts = unsafe {
            enable_and_reset::<T>(&AdcConfig {
                power: config.power,
                source: config.source,
                div: config.div,
            })
            .map_err(Error::ClockSetup)?
        };

        Self::new_inner(T::info(), commands, triggers, config, parts)
    }
}

impl<'a> Adc<'a, Async> {
    /// Create a new async instance of the ADC
    pub fn new_async<T: Instance>(
        _inst: Peri<'a, T>,
        _irq: impl crate::interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'a,
        commands: &'a [Command<'a, T>],
        triggers: &[Trigger],
        config: Config,
    ) -> Result<Self, Error> {
        // Safety:
        // We transmute the ADC instance to a `()`. This is fine since the `T` is only a phantomdata.
        // Because we're now in this function, we don't need this info anymore.
        let commands = unsafe { core::mem::transmute::<&[Command<'_, T>], &[Command<'_, ()>]>(commands) };

        let parts = unsafe {
            enable_and_reset::<T>(&AdcConfig {
                power: config.power,
                source: config.source,
                div: config.div,
            })
            .map_err(Error::ClockSetup)?
        };

        let adc = Self::new_inner(T::info(), commands, triggers, config, parts)?;

        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        Ok(adc)
    }

    /// Reads the current conversion result from the fifo or waits for the next one if it's pending.
    ///
    /// If no conversion is pending, None is returned.
    pub async fn wait_get_conversion(&mut self) -> Option<Conversion> {
        self.info
            .wait_cell()
            .wait_for_value(|| {
                // Enable the interrupts. They get disabled in the interrupt handler
                self.info.regs().ie().write(|reg| {
                    reg.set_fwmie0(true);
                    reg.set_tcomp_ie(TcompIe::ALL_TRIGGER_COMPLETES_ENABLED);
                });

                match self.try_get_conversion() {
                    Ok(result) => Some(Some(result)),
                    Err(Error::FifoPending) => None,
                    Err(Error::FifoEmpty) => Some(None),
                    _ => unreachable!(),
                }
            })
            .await
            .unwrap()
    }

    /// Reads the current conversion result from the fifo or waits for the next one even if no conversion is currently pending.
    ///
    /// If no conversion is pending, None is returned.
    pub async fn wait_conversion(&mut self) -> Conversion {
        self.info
            .wait_cell()
            .wait_for_value(|| {
                // Enable the interrupts. They get disabled in the interrupt handler
                self.info.regs().ie().write(|reg| {
                    reg.set_fwmie0(true);
                    reg.set_tcomp_ie(TcompIe::ALL_TRIGGER_COMPLETES_ENABLED);
                });

                match self.try_get_conversion() {
                    Ok(result) => Some(result),
                    Err(Error::FifoPending) => None,
                    Err(Error::FifoEmpty) => None,
                    _ => unreachable!(),
                }
            })
            .await
            .unwrap()
    }
}

impl<'a, M: Mode> Adc<'a, M> {
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
    /// * [Error::NoTrigger] if the mask is calling a trigger that's not configured
    pub fn do_software_trigger(&mut self, trigger_id_mask: u8) -> Result<(), Error> {
        if (8 - trigger_id_mask.leading_zeros()) > self.num_triggers as u32 {
            return Err(Error::NoTrigger);
        }
        self.info.regs().swtrig().write(|w| w.0 = trigger_id_mask as u32);
        Ok(())
    }

    /// Reset the FIFO buffer.
    ///
    /// Clears all pending conversion results from the FIFO.
    pub fn do_reset_fifo(&mut self) {
        self.info
            .regs()
            .ctrl()
            .modify(|w| w.set_rstfifo0(Rstfifo0::TRIGGER_RESET));
    }

    /// Get conversion result from FIFO.
    ///
    /// Returns:
    /// - `Ok(ConvResult)` if a result is available
    /// - [Error::FifoEmpty] if the FIFO is empty
    /// - [Error::FifoPending] if the FIFO is empty, but the adc is active
    pub fn try_get_conversion(&mut self) -> Result<Conversion, Error> {
        if self.info.regs().fctrl0().read().fcount() == 0 {
            if self.info.regs().stat().read().adc_active() == AdcActive::BUSY {
                return Err(Error::FifoPending);
            }
            return Err(Error::FifoEmpty);
        }

        let fifo = self.info.regs().resfifo0().read();

        Ok(Conversion {
            command: (fifo.cmdsrc() as u8).into(),
            loop_channel_index: fifo.loopcnt() as u8,
            trigger_id_source: fifo.tsrc() as u8,
            conv_value: fifo.d(),
        })
    }

    fn new_inner(
        info: &'static Info,
        commands: &'a [Command<'a, ()>],
        triggers: &[Trigger],
        config: Config,
        parts: PreEnableParts,
    ) -> Result<Self, Error> {
        if commands.len() > 7 {
            return Err(Error::TooManyCommands);
        }
        if triggers.len() > 4 {
            return Err(Error::TooManyTriggers);
        }

        // Commands must only chain other existing commands
        if commands.iter().any(|c| {
            c.config
                .chained_command
                .is_some_and(|cc| (cc as u8 - 1) >= commands.len() as u8)
        }) {
            return Err(Error::InvalidConfig);
        }

        // Triggers must only target existing commands
        if triggers
            .iter()
            .any(|t| (t.target_command_id as u8 - 1) >= commands.len() as u8)
        {
            return Err(Error::InvalidConfig);
        }

        let adc = info.regs();

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
            w.set_refsel(Refsel::from_bits(config.reference_voltage_source as u8));
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

        // Set fifo watermark level to 0, so any data will trigger the possible interrupt
        adc.fctrl0().write(|w| w.set_fwmark(0));

        for (index, command) in commands.iter().enumerate() {
            for channel in command.channels.deref() {
                channel.mux();
            }

            let cmdl = adc.cmdl(index);
            let cmdh = adc.cmdh(index);

            cmdl.write(|w| {
                w.set_adch(command.channels[0].channel);
                w.set_mode(command.config.resolution);
            });

            cmdh.write(|w| {
                w.set_next(Next::from_bits(
                    command.config.chained_command.map(|cc| cc as u8).unwrap_or_default(),
                ));
                w.set_loop_(HwLoop::from_bits(command.loop_count));
                w.set_avgs(command.config.averaging);
                w.set_sts(command.config.sample_time);
                w.set_cmpen(command.config.compare.cmp_en());
                w.set_wait_trig(command.config.wait_for_trigger);
                w.set_lwi(command.increment_channel);
            });

            info.regs().cv(index).write(|reg| {
                let (cvl, cvh) = command.config.compare.get_vals();
                reg.set_cvl(cvl);
                reg.set_cvh(cvh);
            });
        }

        for (index, trigger) in triggers.iter().enumerate() {
            let tctrl = adc.tctrl(index);

            tctrl.write(|w| {
                w.set_tcmd(Tcmd::from_bits(trigger.target_command_id as u8));
                w.set_tdly(trigger.delay_power);
                w.set_tpri(trigger.priority);
                w.set_hten(trigger.enable_hardware_trigger);
                w.set_rsync(trigger.resync);
                w.set_tsync(trigger.synchronous);
            });
        }

        // Enable ADC
        adc.ctrl().modify(|w| w.set_adcen(true));

        Ok(Self {
            commands,
            num_triggers: triggers.len() as u8,
            info,
            _wg: parts.wake_guard,
            _mode: PhantomData,
        })
    }

    /// Perform offset calibration.
    /// Waits for calibration to complete before returning.
    pub fn do_offset_calibration(&mut self) {
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
    fn get_gain_conv_result(&mut self, mut gain_adjustment: f32) -> u32 {
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
    pub fn do_auto_calibration(&mut self) {
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
}

impl<'a, M: Mode> Drop for Adc<'a, M> {
    fn drop(&mut self) {
        // Turn off the ADC
        self.info.regs().ctrl().modify(|reg| reg.set_adcen(false));

        // Demux all the pins
        for command in self.commands {
            for channel in command.channels.deref() {
                channel.demux();
            }
        }
    }
}

/// ADC interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        T::PERF_INT_INCR();

        T::info().regs().ie().write(|_| {});
        // Stat tcomp should go to 0 when `ie` is disabled, but it doesn't.
        // So we have to do it manually. Errata?
        T::info()
            .regs()
            .stat()
            .write(|reg| reg.set_tcomp_int(TcompInt::COMPLETION_DETECTED));
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

trait SealedInstance: Gate<MrccPeriphConfig = AdcConfig> {
    fn info() -> &'static Info;

    const PERF_INT_INCR: fn();
}

/// ADC Instance
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType {
    /// Interrupt for this ADC instance.
    type Interrupt: Interrupt;
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

                    const PERF_INT_INCR: fn() = crate::perf_counters::[<incr_interrupt_adc $n>];
                }

                impl Instance for crate::peripherals::[<ADC $n>] {
                    type Interrupt = crate::interrupt::typelevel::[<ADC $n>];
                }
            }
        )*
    };
}

impl_instance!(0, 1, 2, 3);

/// Trait implemented by any possible ADC pin
pub trait AdcPin<T: Instance>: sealed::SealedAdcPin<T> + GpioPin + PeripheralType {
    /// The channel to be used
    fn channel(&self) -> u8;

    /// Degrade the pin into an [AnyAdcPin]
    fn degrade(self) -> AnyAdcPin<T> {
        let channel = self.channel();
        AnyAdcPin {
            channel,
            pin: Some(GpioPin::degrade(self)),
            _phantom: PhantomData,
        }
    }
}

/// A type-erased ADC pin
pub struct AnyAdcPin<T> {
    channel: u8,
    pin: Option<AnyPin>,
    _phantom: PhantomData<T>,
}

impl<T> AnyAdcPin<T> {
    #[inline]
    fn mux(&self) {
        if let Some(pin) = &self.pin {
            // Set to digital GPIO with input buffer disabled and no pull-ups.
            // TODO also clear digital output value?
            pin.set_pull(crate::gpio::Pull::Disabled);
            pin.set_slew_rate(crate::gpio::SlewRate::Fast.into());
            pin.set_drive_strength(crate::gpio::DriveStrength::Normal.into());
            pin.set_function(Mux::MUX0);
        }
    }

    #[inline]
    fn demux(&self) {
        if let Some(pin) = &self.pin {
            pin.set_as_disabled()
        }
    }

    /// Get the internal temperature sensor pin
    pub fn temperature() -> Peri<'static, Self> {
        // Safety: The temp sensor doesn't gate or own anything, so it's fine to give out as many as the user asks
        unsafe {
            Peri::new_unchecked(Self {
                channel: 26,
                pin: None,
                _phantom: PhantomData,
            })
        }
    }
}

impl<T: Instance, P: AdcPin<T>> From<P> for AnyAdcPin<T> {
    fn from(value: P) -> Self {
        AdcPin::degrade(value)
    }
}

embassy_hal_internal::impl_peripheral!(AnyAdcPin<T>);

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
