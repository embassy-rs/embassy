#![no_std]
#![no_main]
#![allow(async_fn_in_trait)]
#![deny(unused_must_use)]
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;

mod bus;
mod consts;
mod countries;
mod events;
mod ioctl;
mod structs;

mod control;
mod nvram;
mod runner;

use core::slice;

use embassy_net_driver_channel as ch;
use embedded_hal_1::digital::OutputPin;
use events::Events;
use ioctl::IoctlState;

use crate::bus::Bus;
pub use crate::bus::SpiBusCyw43;
pub use crate::control::{AddMulticastAddressError, Control, Error as ControlError, Scanner};
pub use crate::runner::Runner;
pub use crate::structs::BssInfo;

const MTU: usize = 1514;

#[allow(unused)]
#[derive(Clone, Copy, PartialEq, Eq)]
enum Core {
    WLAN = 0,
    SOCSRAM = 1,
    SDIOD = 2,
}

impl Core {
    fn base_addr(&self) -> u32 {
        match self {
            Self::WLAN => CHIP.arm_core_base_address,
            Self::SOCSRAM => CHIP.socsram_wrapper_base_address,
            Self::SDIOD => CHIP.sdiod_core_base_address,
        }
    }
}

#[allow(unused)]
struct Chip {
    arm_core_base_address: u32,
    socsram_base_address: u32,
    socsram_wrapper_base_address: u32,
    sdiod_core_base_address: u32,
    pmu_base_address: u32,
    chip_ram_size: u32,
    atcm_ram_base_address: u32,
    socram_srmem_size: u32,
    chanspec_band_mask: u32,
    chanspec_band_2g: u32,
    chanspec_band_5g: u32,
    chanspec_band_shift: u32,
    chanspec_bw_10: u32,
    chanspec_bw_20: u32,
    chanspec_bw_40: u32,
    chanspec_bw_mask: u32,
    chanspec_bw_shift: u32,
    chanspec_ctl_sb_lower: u32,
    chanspec_ctl_sb_upper: u32,
    chanspec_ctl_sb_none: u32,
    chanspec_ctl_sb_mask: u32,
}

const WRAPPER_REGISTER_OFFSET: u32 = 0x100000;

// Data for CYW43439
const CHIP: Chip = Chip {
    arm_core_base_address: 0x18003000 + WRAPPER_REGISTER_OFFSET,
    socsram_base_address: 0x18004000,
    socsram_wrapper_base_address: 0x18004000 + WRAPPER_REGISTER_OFFSET,
    sdiod_core_base_address: 0x18002000,
    pmu_base_address: 0x18000000,
    chip_ram_size: 512 * 1024,
    atcm_ram_base_address: 0,
    socram_srmem_size: 64 * 1024,
    chanspec_band_mask: 0xc000,
    chanspec_band_2g: 0x0000,
    chanspec_band_5g: 0xc000,
    chanspec_band_shift: 14,
    chanspec_bw_10: 0x0800,
    chanspec_bw_20: 0x1000,
    chanspec_bw_40: 0x1800,
    chanspec_bw_mask: 0x3800,
    chanspec_bw_shift: 11,
    chanspec_ctl_sb_lower: 0x0000,
    chanspec_ctl_sb_upper: 0x0100,
    chanspec_ctl_sb_none: 0x0000,
    chanspec_ctl_sb_mask: 0x0700,
};

/// Driver state.
pub struct State {
    ioctl_state: IoctlState,
    ch: ch::State<MTU, 4, 4>,
    events: Events,
}

impl State {
    /// Create new driver state holder.
    pub fn new() -> Self {
        Self {
            ioctl_state: IoctlState::new(),
            ch: ch::State::new(),
            events: Events::new(),
        }
    }
}

/// Power management modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerManagementMode {
    /// Custom, officially unsupported mode. Use at your own risk.
    /// All power-saving features set to their max at only a marginal decrease in power consumption
    /// as oppposed to `Aggressive`.
    SuperSave,

    /// Aggressive power saving mode.
    Aggressive,

    /// The default mode.
    PowerSave,

    /// Performance is prefered over power consumption but still some power is conserved as opposed to
    /// `None`.
    Performance,

    /// Unlike all the other PM modes, this lowers the power consumption at all times at the cost of
    /// a much lower throughput.
    ThroughputThrottling,

    /// No power management is configured. This consumes the most power.
    None,
}

impl Default for PowerManagementMode {
    fn default() -> Self {
        Self::PowerSave
    }
}

impl PowerManagementMode {
    fn sleep_ret_ms(&self) -> u16 {
        match self {
            PowerManagementMode::SuperSave => 2000,
            PowerManagementMode::Aggressive => 2000,
            PowerManagementMode::PowerSave => 200,
            PowerManagementMode::Performance => 20,
            PowerManagementMode::ThroughputThrottling => 0, // value doesn't matter
            PowerManagementMode::None => 0,                 // value doesn't matter
        }
    }

    fn beacon_period(&self) -> u8 {
        match self {
            PowerManagementMode::SuperSave => 255,
            PowerManagementMode::Aggressive => 1,
            PowerManagementMode::PowerSave => 1,
            PowerManagementMode::Performance => 1,
            PowerManagementMode::ThroughputThrottling => 0, // value doesn't matter
            PowerManagementMode::None => 0,                 // value doesn't matter
        }
    }

    fn dtim_period(&self) -> u8 {
        match self {
            PowerManagementMode::SuperSave => 255,
            PowerManagementMode::Aggressive => 1,
            PowerManagementMode::PowerSave => 1,
            PowerManagementMode::Performance => 1,
            PowerManagementMode::ThroughputThrottling => 0, // value doesn't matter
            PowerManagementMode::None => 0,                 // value doesn't matter
        }
    }

    fn assoc(&self) -> u8 {
        match self {
            PowerManagementMode::SuperSave => 255,
            PowerManagementMode::Aggressive => 10,
            PowerManagementMode::PowerSave => 10,
            PowerManagementMode::Performance => 1,
            PowerManagementMode::ThroughputThrottling => 0, // value doesn't matter
            PowerManagementMode::None => 0,                 // value doesn't matter
        }
    }

    fn mode(&self) -> u32 {
        match self {
            PowerManagementMode::ThroughputThrottling => 1,
            PowerManagementMode::None => 0,
            _ => 2,
        }
    }
}

/// Embassy-net driver.
pub type NetDriver<'a> = ch::Device<'a, MTU>;

/// Create a new instance of the CYW43 driver.
///
/// Returns a handle to the network device, control handle and a runner for driving the low level
/// stack.
pub async fn new<'a, PWR, SPI>(
    state: &'a mut State,
    pwr: PWR,
    spi: SPI,
    firmware: &[u8],
) -> (NetDriver<'a>, Control<'a>, Runner<'a, PWR, SPI>)
where
    PWR: OutputPin,
    SPI: SpiBusCyw43,
{
    let (ch_runner, device) = ch::new(&mut state.ch, ch::driver::HardwareAddress::Ethernet([0; 6]));
    let state_ch = ch_runner.state_runner();

    let mut runner = Runner::new(ch_runner, Bus::new(pwr, spi), &state.ioctl_state, &state.events);

    runner.init(firmware).await;

    (
        device,
        Control::new(state_ch, &state.events, &state.ioctl_state),
        runner,
    )
}

fn slice8_mut(x: &mut [u32]) -> &mut [u8] {
    let len = x.len() * 4;
    unsafe { slice::from_raw_parts_mut(x.as_mut_ptr() as _, len) }
}
