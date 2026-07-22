//! I3C configuration.

use crate::gpio::{AfType, OutputType, Pull, Speed};
use crate::time::Hertz;

/// SCL waveform timing in kernel clock cycles.
///
/// These values correspond to the fields in `TIMINGR0` and `TIMINGR1`.
/// The ST CubeMX / `I3C_CtrlTimingComputation` utility can compute them from
/// desired bus frequencies; the defaults match the Nucleo-N657X0 I3C examples.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct BusTiming {
    /// SCL low duration in I3C push-pull phases (`TIMINGR0.SCLL_PP`).
    pub scl_pp_low: u8,
    /// SCL high duration for I3C messages (`TIMINGR0.SCLH_I3C`).
    pub scl_i3c_high: u8,
    /// SCL low duration in open-drain phases (`TIMINGR0.SCLL_OD`).
    pub scl_od_low: u8,
    /// SCL high duration for legacy I2C messages (`TIMINGR0.SCLH_I2C`).
    pub scl_i2c_high: u8,
    /// Bus free duration (`TIMINGR1.FREE`).
    pub bus_free: u8,
    /// Bus idle / hot-join duration (`TIMINGR1.AVAL`).
    pub bus_idle: u8,
    /// SDA hold time: `true` = 1.5 cycles, `false` = 0.5 cycles (`TIMINGR1.SDA_HD`).
    pub sda_hold_1_5: bool,
    /// Activity state of new controller (`TIMINGR1.ASNCR`), typically 0.
    pub wait_time: u8,
}

impl Default for BusTiming {
    fn default() -> Self {
        // Values from NUCLEO-N657X0 I3C_Controller_Direct_Command_DMA example.
        Self {
            scl_pp_low: 0x07,
            scl_i3c_high: 0x07,
            scl_od_low: 0x47,
            scl_i2c_high: 0x00,
            bus_free: 0x28,
            bus_idle: 0xc6,
            sda_hold_1_5: true,
            wait_time: 0,
        }
    }
}

/// Controller-specific options written to `TIMINGR2` and `CFGR`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ControllerOptions {
    /// Dynamic address used when the controller acts as a target (0 = none).
    pub dynamic_addr: u8,
    /// Controller clock stall time in kernel cycles (`TIMINGR2.STALL`).
    pub stall_time: u8,
    /// Acknowledge hot-join requests from targets.
    pub hot_join_allowed: bool,
    /// Enable SDA high-keeper.
    pub high_keeper_sda: bool,
}

impl Default for ControllerOptions {
    fn default() -> Self {
        Self {
            dynamic_addr: 0,
            stall_time: 0,
            hot_join_allowed: false,
            high_keeper_sda: false,
        }
    }
}

/// I3C controller configuration.
#[non_exhaustive]
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Config {
    /// Target I3C push-pull bus frequency (informational; actual timing comes from [`BusTiming`]).
    pub push_pull_freq: Hertz,
    /// Target I3C open-drain bus frequency (informational).
    pub open_drain_freq: Hertz,
    /// Bus waveform timing registers.
    pub timing: BusTiming,
    /// Controller role options.
    pub controller: ControllerOptions,
    /// GPIO slew rate for SCL/SDA.
    pub gpio_speed: Speed,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            push_pull_freq: Hertz::mhz(12),
            open_drain_freq: Hertz::khz(400),
            timing: BusTiming::default(),
            controller: ControllerOptions::default(),
            gpio_speed: Speed::VeryHigh,
        }
    }
}

impl Config {
    pub(super) fn scl_af(&self) -> AfType {
        #[cfg(gpio_v1)]
        return AfType::output(OutputType::PushPull, self.gpio_speed);
        #[cfg(gpio_v2)]
        return AfType::output_pull(OutputType::PushPull, self.gpio_speed, Pull::None);
    }

    pub(super) fn sda_af(&self) -> AfType {
        #[cfg(gpio_v1)]
        return AfType::output(OutputType::PushPull, self.gpio_speed);
        #[cfg(gpio_v2)]
        return AfType::output_pull(OutputType::PushPull, self.gpio_speed, Pull::None);
    }
}
