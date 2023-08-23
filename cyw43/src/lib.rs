#![no_std]
#![no_main]
#![allow(incomplete_features)]
#![feature(async_fn_in_trait, type_alias_impl_trait, concat_bytes)]
#![deny(unused_must_use)]

// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;

mod ble_connector;
mod bluetooth;
mod bus;
mod consts;
mod control;
mod countries;
mod events;
mod ioctl;
mod nvram;
mod runner;
mod structs;
mod utilities;

use embassy_net_driver_channel as net_driver_channel;
use embedded_hal_1::digital::OutputPin;
use events::Events;
use ioctl::IoctlState;

pub use crate::ble_connector::BleConnector;
use crate::bus::Bus;
pub use crate::bus::SpiBusCyw43;
pub use crate::control::{Control, Error as ControlError};
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
    bluetooth_base_address: u32,
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
    bluetooth_base_address: 0x19000000,
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

pub struct State {
    ioctl_state: IoctlState,
    net_driver_channel: net_driver_channel::State<MTU, 4, 4>,
    events: Events,
}

impl State {
    pub fn new() -> Self {
        Self {
            ioctl_state: IoctlState::new(),
            net_driver_channel: net_driver_channel::State::new(),
            events: Events::new(),
        }
    }
}

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

pub type NetDriver<'a> = net_driver_channel::Device<'a, MTU>;

pub async fn new<'a, PWR, SPI>(
    state: &'a mut State,
    pwr: PWR,
    spi: SPI,
    firmware: &[u8],
    bluetooth_enabled: bool,
) -> (NetDriver<'a>, Control<'a>, Runner<'a, PWR, SPI>)
where
    PWR: OutputPin,
    SPI: SpiBusCyw43,
{
    let (net_driver_channel_runner, device) = net_driver_channel::new(
        &mut state.net_driver_channel,
        net_driver_channel::driver::HardwareAddress::Ethernet([0; 6]),
    );
    let net_driver_channel_state_runner = net_driver_channel_runner.state_runner();
    let bus = Bus::new(pwr, spi, bluetooth_enabled);
    let mut runner = Runner::new(net_driver_channel_runner, bus, &state.ioctl_state, &state.events);
    runner.init(firmware, bluetooth_enabled).await;
    let control = Control::new(
        net_driver_channel_state_runner,
        &state.events,
        &state.ioctl_state,
        bluetooth_enabled,
    );
    (device, control, runner)
}
