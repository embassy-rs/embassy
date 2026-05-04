#![no_std]
#![no_main]
#![allow(async_fn_in_trait)]
#![allow(unsafe_op_in_unsafe_fn)]
#![deny(unused_must_use)]
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;

#[cfg(feature = "bluetooth")]
/// Bluetooth module.
pub mod bluetooth;
mod chip;
mod consts;
mod control;
mod countries;
mod events;
mod ioctl;
mod runner;
mod sdio;
mod spi;
mod structs;
mod util;

use core::result;
use core::sync::atomic::AtomicBool;

pub use aligned::{A4, Aligned};
use embassy_net_driver_channel as ch;
use embedded_hal_1::digital::OutputPin;
use events::Events;
use ioctl::IoctlState;

pub use crate::control::{
    AddMulticastAddressError, Control, JoinAuth, JoinError, JoinOptions, ScanOptions, ScanType, Scanner,
};
pub use crate::runner::Runner;
pub use crate::sdio::{SdioBus, SdioBusCyw43};
pub use crate::spi::{SpiBus, SpiBusCyw43};
pub use crate::structs::BssInfo;

const MTU: usize = 1514;

/// cyw43 Error type
#[derive(Debug)]
pub struct Error;

type Result<T> = result::Result<T, Error>;

trait WithContext: Sized {
    #[track_caller]
    fn ctx(self, context: &'static str) -> Self;
}

impl<T> WithContext for Result<T> {
    fn ctx(self, context: &'static str) -> Self {
        if self.is_err() {
            error!("- {}", context);
        }

        self
    }
}

#[allow(unused)]
#[derive(Clone, Copy, PartialEq, Eq)]
enum Core {
    WLAN = 0,
    SOCSRAM = 1,
    SDIOD = 2,
}

#[allow(unused)]
pub(crate) struct ChipInfo {
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

/// Marker trait for chip types supported by this driver.
#[allow(private_bounds)]
pub trait Chip: SealedChip {}

impl<T: SealedChip> Chip for T {}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ChipId {
    C43439,
    C4373,
}

impl PartialEq<u16> for ChipId {
    fn eq(&self, other: &u16) -> bool {
        (match *self {
            ChipId::C43439 => 43439,
            ChipId::C4373 => 4373,
        }) == *other
    }
}

trait SealedChip: Copy {
    const INFO: ChipInfo;
    const ID: ChipId;

    fn id(&self) -> ChipId {
        Self::ID
    }

    fn base_addr(&self, core: Core) -> u32 {
        match core {
            Core::WLAN => self.arm_core_base_address(),
            Core::SOCSRAM => self.socsram_wrapper_base_address(),
            Core::SDIOD => self.sdiod_core_base_address(),
        }
    }

    fn arm_core_base_address(&self) -> u32 {
        Self::INFO.arm_core_base_address
    }

    fn socsram_base_address(&self) -> u32 {
        Self::INFO.socsram_base_address
    }

    #[allow(dead_code)]
    fn bluetooth_base_address(&self) -> u32 {
        Self::INFO.bluetooth_base_address
    }

    fn socsram_wrapper_base_address(&self) -> u32 {
        Self::INFO.socsram_wrapper_base_address
    }

    fn sdiod_core_base_address(&self) -> u32 {
        Self::INFO.sdiod_core_base_address
    }

    #[allow(dead_code)]
    fn pmu_base_address(&self) -> u32 {
        Self::INFO.pmu_base_address
    }

    #[allow(dead_code)]
    fn chip_ram_size(&self) -> u32 {
        Self::INFO.chip_ram_size
    }

    fn atcm_ram_base_address(&self) -> u32 {
        Self::INFO.atcm_ram_base_address
    }

    #[allow(dead_code)]
    fn socram_srmem_size(&self) -> u32 {
        Self::INFO.socram_srmem_size
    }

    #[allow(dead_code)]
    fn chanspec_band_mask(&self) -> u32 {
        Self::INFO.chanspec_band_mask
    }

    #[allow(dead_code)]
    fn chanspec_band_2g(&self) -> u32 {
        Self::INFO.chanspec_band_2g
    }

    #[allow(dead_code)]
    fn chanspec_band_5g(&self) -> u32 {
        Self::INFO.chanspec_band_5g
    }

    #[allow(dead_code)]
    fn chanspec_band_shift(&self) -> u32 {
        Self::INFO.chanspec_band_shift
    }

    #[allow(dead_code)]
    fn chanspec_bw_10(&self) -> u32 {
        Self::INFO.chanspec_bw_10
    }

    #[allow(dead_code)]
    fn chanspec_bw_20(&self) -> u32 {
        Self::INFO.chanspec_bw_20
    }

    #[allow(dead_code)]
    fn chanspec_bw_40(&self) -> u32 {
        Self::INFO.chanspec_bw_40
    }

    #[allow(dead_code)]
    fn chanspec_bw_mask(&self) -> u32 {
        Self::INFO.chanspec_bw_mask
    }

    #[allow(dead_code)]
    fn chanspec_bw_shift(&self) -> u32 {
        Self::INFO.chanspec_bw_shift
    }

    #[allow(dead_code)]
    fn chanspec_ctl_sb_lower(&self) -> u32 {
        Self::INFO.chanspec_ctl_sb_lower
    }

    #[allow(dead_code)]
    fn chanspec_ctl_sb_upper(&self) -> u32 {
        Self::INFO.chanspec_ctl_sb_upper
    }

    #[allow(dead_code)]
    fn chanspec_ctl_sb_none(&self) -> u32 {
        Self::INFO.chanspec_ctl_sb_none
    }

    #[allow(dead_code)]
    fn chanspec_ctl_sb_mask(&self) -> u32 {
        Self::INFO.chanspec_ctl_sb_mask
    }
}

/// CYW43439 Wi-Fi + Bluetooth combo chip (used on Raspberry Pi Pico W).
#[derive(Clone, Copy, Debug)]
pub struct Cyw43439;

impl SealedChip for Cyw43439 {
    const ID: ChipId = ChipId::C43439;
    const INFO: ChipInfo = ChipInfo {
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
}

/// CYW4373 Wi-Fi + Bluetooth combo chip (Murata LBAD0ZZ1DZ / 2BC module).
#[derive(Clone, Copy, Debug)]
pub struct Cyw4373;

impl SealedChip for Cyw4373 {
    const ID: ChipId = ChipId::C4373;
    const INFO: ChipInfo = ChipInfo {
        arm_core_base_address: 0x18002000 + WRAPPER_REGISTER_OFFSET,
        socsram_base_address: 0x18004000,
        bluetooth_base_address: 0x19000000,
        socsram_wrapper_base_address: 0x18004000 + WRAPPER_REGISTER_OFFSET,
        sdiod_core_base_address: 0x18005000,
        pmu_base_address: 0x18000000,
        chip_ram_size: 0xE0000,          // 896 KB
        atcm_ram_base_address: 0x160000, // ARM core vectors from here; firmware must go here
        socram_srmem_size: 0,
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
}

/// Driver state.
pub struct State {
    ioctl_state: IoctlState,
    net: NetState,
    #[cfg(feature = "bluetooth")]
    bt: bluetooth::BtState,
}

struct NetState {
    ch: ch::State<MTU, 4, 4>,
    events: Events,
    secure_network: AtomicBool,
}

impl State {
    /// Create new driver state holder.
    pub const fn new() -> Self {
        Self {
            ioctl_state: IoctlState::new(),
            net: NetState {
                ch: ch::State::new(),
                events: Events::new(),
                secure_network: AtomicBool::new(false),
            },
            #[cfg(feature = "bluetooth")]
            bt: bluetooth::BtState::new(),
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
    firmware: &Aligned<A4, [u8]>,
    nvram: &Aligned<A4, [u8]>,
) -> (NetDriver<'a>, Control<'a>, Runner<'a, SpiBus<PWR, SPI>, Cyw43439>)
where
    PWR: OutputPin,
    SPI: SpiBusCyw43,
{
    let (ch_runner, device) = ch::new(&mut state.net.ch, ch::driver::HardwareAddress::Ethernet([0; 6]));
    let state_ch = ch_runner.state_runner();

    let mut runner = Runner::new(
        ch_runner,
        SpiBus::new(pwr, spi),
        Cyw43439,
        &state.ioctl_state,
        &state.net.events,
        &state.net.secure_network,
        #[cfg(feature = "bluetooth")]
        None,
    );

    runner.init(firmware, nvram, None, &()).await.unwrap();
    let control = Control::new(
        state_ch,
        &state.net.events,
        &state.ioctl_state,
        &state.net.secure_network,
    );

    (device, control, runner)
}

#[deprecated(note = "please use `new_43439_sdio` instead")]
/// Create a new instance of the CYW43 driver.
///
/// Returns a handle to the network device, control handle and a runner for driving the low level
/// stack.
pub async fn new_sdio<'a, SDIO>(
    state: &'a mut State,
    sdio: SDIO,
    firmware: &Aligned<A4, [u8]>,
    nvram: &Aligned<A4, [u8]>,
) -> (NetDriver<'a>, Control<'a>, Runner<'a, SdioBus<SDIO>, Cyw43439>)
where
    SDIO: SdioBusCyw43<64>,
{
    new_43439_sdio(state, sdio, firmware, nvram).await.unwrap()
}

/// Create a new instance of the CYW43 driver.
///
/// Returns a handle to the network device, control handle and a runner for driving the low level
/// stack.
pub async fn new_43439_sdio<'a, SDIO>(
    state: &'a mut State,
    sdio: SDIO,
    firmware: &Aligned<A4, [u8]>,
    nvram: &Aligned<A4, [u8]>,
) -> Result<(NetDriver<'a>, Control<'a>, Runner<'a, SdioBus<SDIO>, Cyw43439>)>
where
    SDIO: SdioBusCyw43<64>,
{
    let (ch_runner, device) = ch::new(&mut state.net.ch, ch::driver::HardwareAddress::Ethernet([0; 6]));
    let state_ch = ch_runner.state_runner();

    let mut runner = Runner::new(
        ch_runner,
        SdioBus::new(sdio),
        Cyw43439,
        &state.ioctl_state,
        &state.net.events,
        &state.net.secure_network,
        #[cfg(feature = "bluetooth")]
        None,
    );

    let config = sdio::Config {
        max_f: 50_000_000,
        out_of_band_irq: false,
    };

    runner.init(firmware, nvram, None, &config).await?;
    let control = Control::new(
        state_ch,
        &state.net.events,
        &state.ioctl_state,
        &state.net.secure_network,
    );

    Ok((device, control, runner))
}

/// Create a new instance of the CYW4373 SDIO driver, returning an error on init failure.
pub async fn new_4373_sdio<'a, SDIO>(
    state: &'a mut State,
    sdio: SDIO,
    firmware: &Aligned<A4, [u8]>,
    nvram: &Aligned<A4, [u8]>,
) -> Result<(NetDriver<'a>, Control<'a>, Runner<'a, SdioBus<SDIO>, Cyw4373>)>
where
    SDIO: SdioBusCyw43<64>,
{
    let (ch_runner, device) = ch::new(&mut state.net.ch, ch::driver::HardwareAddress::Ethernet([0; 6]));
    let state_ch = ch_runner.state_runner();

    let mut runner = Runner::new(
        ch_runner,
        SdioBus::new(sdio),
        Cyw4373,
        &state.ioctl_state,
        &state.net.events,
        &state.net.secure_network,
        #[cfg(feature = "bluetooth")]
        None,
    );

    let config = sdio::Config {
        max_f: 12_500_000,
        out_of_band_irq: false,
    };

    runner.init(firmware, nvram, None, &config).await?;
    let control = Control::new(
        state_ch,
        &state.net.events,
        &state.ioctl_state,
        &state.net.secure_network,
    );

    Ok((device, control, runner))
}

/// Create a new instance of the CYW43 driver.
///
/// Returns a handle to the network device, control handle and a runner for driving the low level
/// stack.
#[cfg(feature = "bluetooth")]
pub async fn new_with_bluetooth<'a, PWR, SPI>(
    state: &'a mut State,
    pwr: PWR,
    spi: SPI,
    wifi_firmware: &Aligned<A4, [u8]>,
    bluetooth_firmware: &Aligned<A4, [u8]>,
    nvram: &Aligned<A4, [u8]>,
) -> (
    NetDriver<'a>,
    bluetooth::BtDriver<'a>,
    Control<'a>,
    Runner<'a, SpiBus<PWR, SPI>, Cyw43439>,
)
where
    PWR: OutputPin,
    SPI: SpiBusCyw43,
{
    let (ch_runner, device) = ch::new(&mut state.net.ch, ch::driver::HardwareAddress::Ethernet([0; 6]));
    let state_ch = ch_runner.state_runner();

    let (bt_runner, bt_driver) = bluetooth::new(&mut state.bt);
    let mut runner = Runner::new(
        ch_runner,
        SpiBus::new(pwr, spi),
        Cyw43439,
        &state.ioctl_state,
        &state.net.events,
        &state.net.secure_network,
        #[cfg(feature = "bluetooth")]
        Some(bt_runner),
    );

    runner
        .init(wifi_firmware, nvram, Some(bluetooth_firmware), &())
        .await
        .unwrap();
    let control = Control::new(
        state_ch,
        &state.net.events,
        &state.ioctl_state,
        &state.net.secure_network,
    );

    (device, bt_driver, control, runner)
}

/// Include bytes aligned to A4 in the binary
#[macro_export]
macro_rules! aligned_bytes {
    ($path:expr) => {{
        {
            static BYTES: &cyw43::Aligned<cyw43::A4, [u8]> = &cyw43::Aligned(*include_bytes!($path));

            BYTES
        }
    }};
}
