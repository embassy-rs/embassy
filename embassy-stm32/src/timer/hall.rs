//! Hall sensor interface helper using timer XOR input.
//!
//! This helper targets 3-phase BLDC Hall setups where CH1/CH2/CH3 are routed
//! into the timer input stage and optionally XOR-combined (TI1S).

use stm32_metapac::timer::vals::{self, FilterValue};

use super::low_level::{InputCaptureMode, InputCaptureSelection, Timer};
use super::{Ch1, Ch2, Ch3, Channel, GeneralInstance4Channel, TimerPin};
use crate::Peri;
use crate::gpio::{AfType, Flex, Pull};

/// Hall interface configuration.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy)]
pub struct HallConfig {
    /// Pull configuration for Hall input pins.
    pub pull: Pull,
    /// Input filter applied on capture channel 1.
    pub filter: FilterValue,
    /// Input capture prescaler factor for channel 1 (1,2,4,8).
    pub prescaler: u8,
    /// Enable CH1/CH2/CH3 XOR function (TI1S).
    pub enable_xor: bool,
    /// Trigger source for Hall/reset sequencing.
    pub trigger_source: vals::Ts,
    /// If set, configure slave reset mode to restart the counter on trigger.
    pub reset_on_trigger: bool,
}

impl Default for HallConfig {
    fn default() -> Self {
        Self {
            pull: Pull::None,
            filter: FilterValue::NoFilter,
            prescaler: 1,
            enable_xor: true,
            trigger_source: vals::Ts::Ti1fEd,
            reset_on_trigger: true,
        }
    }
}

/// Hall sensor timer interface.
pub struct HallInterface<'d, T: GeneralInstance4Channel> {
    inner: Timer<'d, T>,
    _ch1: Flex<'d>,
    _ch2: Flex<'d>,
    _ch3: Flex<'d>,
}

impl<'d, T: GeneralInstance4Channel> HallInterface<'d, T> {
    /// Create and configure a Hall sensor interface on timer CH1/CH2/CH3.
    #[allow(unused)]
    pub fn new<#[cfg(afio)] A>(
        tim: Peri<'d, T>,
        ch1: Peri<'d, if_afio!(impl TimerPin<T, Ch1, A>)>,
        ch2: Peri<'d, if_afio!(impl TimerPin<T, Ch2, A>)>,
        ch3: Peri<'d, if_afio!(impl TimerPin<T, Ch3, A>)>,
        config: HallConfig,
    ) -> Self {
        critical_section::with(|_| {
            ch1.set_low();
            ch2.set_low();
            ch3.set_low();
        });

        let inner = Timer::new(tim);
        let regs = inner.regs_gp16();

        regs.cr2().modify(|w| {
            w.set_ti1s(if config.enable_xor {
                vals::Ti1s::Xor
            } else {
                vals::Ti1s::Normal
            })
        });

        inner.set_input_capture_selection(Channel::Ch1, InputCaptureSelection::TRC);
        inner.set_input_capture_mode(Channel::Ch1, InputCaptureMode::Rising);
        inner.set_input_capture_filter(Channel::Ch1, config.filter);
        inner.set_input_capture_prescaler(Channel::Ch1, config.prescaler);
        inner.enable_channel(Channel::Ch1, true);

        if config.reset_on_trigger {
            inner.set_trigger_source(config.trigger_source);
            inner.set_slave_mode(vals::Sms::ResetMode);
        }

        inner.start();

        Self {
            inner,
            _ch1: new_pin!(ch1, AfType::input(config.pull)).unwrap(),
            _ch2: new_pin!(ch2, AfType::input(config.pull)).unwrap(),
            _ch3: new_pin!(ch3, AfType::input(config.pull)).unwrap(),
        }
    }

    /// Read the latest captured Hall event timestamp (CCR1).
    pub fn capture_ticks(&self) -> T::Word {
        self.inner.get_capture_value(Channel::Ch1)
    }

    /// Check capture interrupt flag for Hall event channel.
    pub fn event_pending(&self) -> bool {
        self.inner.get_input_interrupt(Channel::Ch1)
    }

    /// Clear capture interrupt flag for Hall event channel.
    pub fn clear_event(&self) {
        self.inner.clear_input_interrupt(Channel::Ch1);
    }
}
