#![no_std]
#![no_main]
#![allow(incomplete_features)]
#![feature(async_fn_in_trait, type_alias_impl_trait, concat_bytes)]
#![deny(unused_must_use)]

// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;

mod bus;
mod consts;
mod countries;
mod events;
mod structs;

use core::cell::Cell;
use core::cmp::{max, min};
use core::slice;

use ch::driver::LinkState;
use embassy_futures::yield_now;
use embassy_net_driver_channel as ch;
use embassy_time::{block_for, Duration, Timer};
use embedded_hal_1::digital::OutputPin;

use crate::bus::Bus;
pub use crate::bus::SpiBusCyw43;
use crate::consts::*;
use crate::events::Event;
use crate::structs::*;

const MTU: usize = 1514;

#[derive(Clone, Copy)]
pub enum IoctlType {
    Get = 0,
    Set = 2,
}

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

#[derive(Clone, Copy)]
enum IoctlState {
    Idle,

    Pending {
        kind: IoctlType,
        cmd: u32,
        iface: u32,
        buf: *mut [u8],
    },
    Sent {
        buf: *mut [u8],
    },
    Done {
        resp_len: usize,
    },
}

pub struct State {
    ioctl_state: Cell<IoctlState>,
    ch: ch::State<MTU, 4, 4>,
}

impl State {
    pub fn new() -> Self {
        Self {
            ioctl_state: Cell::new(IoctlState::Idle),
            ch: ch::State::new(),
        }
    }
}

pub struct Control<'a> {
    state_ch: ch::StateRunner<'a>,
    ioctl_state: &'a Cell<IoctlState>,
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
            _ => 2,
        }
    }
}

impl<'a> Control<'a> {
    pub async fn init(&mut self, clm: &[u8]) {
        const CHUNK_SIZE: usize = 1024;

        info!("Downloading CLM...");

        let mut offs = 0;
        for chunk in clm.chunks(CHUNK_SIZE) {
            let mut flag = DOWNLOAD_FLAG_HANDLER_VER;
            if offs == 0 {
                flag |= DOWNLOAD_FLAG_BEGIN;
            }
            offs += chunk.len();
            if offs == clm.len() {
                flag |= DOWNLOAD_FLAG_END;
            }

            let header = DownloadHeader {
                flag,
                dload_type: DOWNLOAD_TYPE_CLM,
                len: chunk.len() as _,
                crc: 0,
            };
            let mut buf = [0; 8 + 12 + CHUNK_SIZE];
            buf[0..8].copy_from_slice(b"clmload\x00");
            buf[8..20].copy_from_slice(&header.to_bytes());
            buf[20..][..chunk.len()].copy_from_slice(&chunk);
            self.ioctl(IoctlType::Set, IOCTL_CMD_SET_VAR, 0, &mut buf[..8 + 12 + chunk.len()])
                .await;
        }

        // check clmload ok
        assert_eq!(self.get_iovar_u32("clmload_status").await, 0);

        info!("Configuring misc stuff...");

        // Disable tx gloming which transfers multiple packets in one request.
        // 'glom' is short for "conglomerate" which means "gather together into
        // a compact mass".
        self.set_iovar_u32("bus:txglom", 0).await;
        self.set_iovar_u32("apsta", 1).await;

        // read MAC addr.
        let mut mac_addr = [0; 6];
        assert_eq!(self.get_iovar("cur_etheraddr", &mut mac_addr).await, 6);
        info!("mac addr: {:02x}", mac_addr);

        let country = countries::WORLD_WIDE_XX;
        let country_info = CountryInfo {
            country_abbrev: [country.code[0], country.code[1], 0, 0],
            country_code: [country.code[0], country.code[1], 0, 0],
            rev: if country.rev == 0 { -1 } else { country.rev as _ },
        };
        self.set_iovar("country", &country_info.to_bytes()).await;

        // set country takes some time, next ioctls fail if we don't wait.
        Timer::after(Duration::from_millis(100)).await;

        // Set antenna to chip antenna
        self.ioctl_set_u32(IOCTL_CMD_ANTDIV, 0, 0).await;

        self.set_iovar_u32("bus:txglom", 0).await;
        Timer::after(Duration::from_millis(100)).await;
        //self.set_iovar_u32("apsta", 1).await; // this crashes, also we already did it before...??
        //Timer::after(Duration::from_millis(100)).await;
        self.set_iovar_u32("ampdu_ba_wsize", 8).await;
        Timer::after(Duration::from_millis(100)).await;
        self.set_iovar_u32("ampdu_mpdu", 4).await;
        Timer::after(Duration::from_millis(100)).await;
        //self.set_iovar_u32("ampdu_rx_factor", 0).await; // this crashes

        //Timer::after(Duration::from_millis(100)).await;

        // evts
        let mut evts = EventMask {
            iface: 0,
            events: [0xFF; 24],
        };

        // Disable spammy uninteresting events.
        evts.unset(Event::RADIO);
        evts.unset(Event::IF);
        evts.unset(Event::PROBREQ_MSG);
        evts.unset(Event::PROBREQ_MSG_RX);
        evts.unset(Event::PROBRESP_MSG);
        evts.unset(Event::PROBRESP_MSG);

        self.set_iovar("bsscfg:event_msgs", &evts.to_bytes()).await;

        Timer::after(Duration::from_millis(100)).await;

        // set wifi up
        self.ioctl(IoctlType::Set, IOCTL_CMD_UP, 0, &mut []).await;

        Timer::after(Duration::from_millis(100)).await;

        self.ioctl_set_u32(110, 0, 1).await; // SET_GMODE = auto
        self.ioctl_set_u32(142, 0, 0).await; // SET_BAND = any

        Timer::after(Duration::from_millis(100)).await;

        self.state_ch.set_ethernet_address(mac_addr);
        self.state_ch.set_link_state(LinkState::Up); // TODO do on join/leave

        info!("INIT DONE");
    }

    pub async fn set_power_management(&mut self, mode: PowerManagementMode) {
        // power save mode
        let mode_num = mode.mode();
        if mode_num == 2 {
            self.set_iovar_u32("pm2_sleep_ret", mode.sleep_ret_ms() as u32).await;
            self.set_iovar_u32("bcn_li_bcn", mode.beacon_period() as u32).await;
            self.set_iovar_u32("bcn_li_dtim", mode.dtim_period() as u32).await;
            self.set_iovar_u32("assoc_listen", mode.assoc() as u32).await;
        }
        self.ioctl_set_u32(86, 0, mode_num).await;
    }

    pub async fn join_open(&mut self, ssid: &str) {
        self.set_iovar_u32("ampdu_ba_wsize", 8).await;

        self.ioctl_set_u32(134, 0, 0).await; // wsec = open
        self.set_iovar_u32x2("bsscfg:sup_wpa", 0, 0).await;
        self.ioctl_set_u32(20, 0, 1).await; // set_infra = 1
        self.ioctl_set_u32(22, 0, 0).await; // set_auth = open (0)

        let mut i = SsidInfo {
            len: ssid.len() as _,
            ssid: [0; 32],
        };
        i.ssid[..ssid.len()].copy_from_slice(ssid.as_bytes());
        self.ioctl(IoctlType::Set, IOCTL_CMD_SET_SSID, 0, &mut i.to_bytes())
            .await; // set_ssid

        info!("JOINED");
    }

    pub async fn join_wpa2(&mut self, ssid: &str, passphrase: &str) {
        self.set_iovar_u32("ampdu_ba_wsize", 8).await;

        self.ioctl_set_u32(134, 0, 4).await; // wsec = wpa2
        self.set_iovar_u32x2("bsscfg:sup_wpa", 0, 1).await;
        self.set_iovar_u32x2("bsscfg:sup_wpa2_eapver", 0, 0xFFFF_FFFF).await;
        self.set_iovar_u32x2("bsscfg:sup_wpa_tmo", 0, 2500).await;

        Timer::after(Duration::from_millis(100)).await;

        let mut pfi = PassphraseInfo {
            len: passphrase.len() as _,
            flags: 1,
            passphrase: [0; 64],
        };
        pfi.passphrase[..passphrase.len()].copy_from_slice(passphrase.as_bytes());
        self.ioctl(IoctlType::Set, IOCTL_CMD_SET_PASSPHRASE, 0, &mut pfi.to_bytes())
            .await; // WLC_SET_WSEC_PMK

        self.ioctl_set_u32(20, 0, 1).await; // set_infra = 1
        self.ioctl_set_u32(22, 0, 0).await; // set_auth = 0 (open)
        self.ioctl_set_u32(165, 0, 0x80).await; // set_wpa_auth

        let mut i = SsidInfo {
            len: ssid.len() as _,
            ssid: [0; 32],
        };
        i.ssid[..ssid.len()].copy_from_slice(ssid.as_bytes());
        self.ioctl(IoctlType::Set, 26, 0, &mut i.to_bytes()).await; // set_ssid

        info!("JOINED");
    }

    pub async fn gpio_set(&mut self, gpio_n: u8, gpio_en: bool) {
        assert!(gpio_n < 3);
        self.set_iovar_u32x2("gpioout", 1 << gpio_n, if gpio_en { 1 << gpio_n } else { 0 })
            .await
    }

    async fn set_iovar_u32x2(&mut self, name: &str, val1: u32, val2: u32) {
        let mut buf = [0; 8];
        buf[0..4].copy_from_slice(&val1.to_le_bytes());
        buf[4..8].copy_from_slice(&val2.to_le_bytes());
        self.set_iovar(name, &buf).await
    }

    async fn set_iovar_u32(&mut self, name: &str, val: u32) {
        self.set_iovar(name, &val.to_le_bytes()).await
    }

    async fn get_iovar_u32(&mut self, name: &str) -> u32 {
        let mut buf = [0; 4];
        let len = self.get_iovar(name, &mut buf).await;
        assert_eq!(len, 4);
        u32::from_le_bytes(buf)
    }

    async fn set_iovar(&mut self, name: &str, val: &[u8]) {
        info!("set {} = {:02x}", name, val);

        let mut buf = [0; 64];
        buf[..name.len()].copy_from_slice(name.as_bytes());
        buf[name.len()] = 0;
        buf[name.len() + 1..][..val.len()].copy_from_slice(val);

        let total_len = name.len() + 1 + val.len();
        self.ioctl(IoctlType::Set, IOCTL_CMD_SET_VAR, 0, &mut buf[..total_len])
            .await;
    }

    // TODO this is not really working, it always returns all zeros.
    async fn get_iovar(&mut self, name: &str, res: &mut [u8]) -> usize {
        info!("get {}", name);

        let mut buf = [0; 64];
        buf[..name.len()].copy_from_slice(name.as_bytes());
        buf[name.len()] = 0;

        let total_len = max(name.len() + 1, res.len());
        let res_len = self
            .ioctl(IoctlType::Get, IOCTL_CMD_GET_VAR, 0, &mut buf[..total_len])
            .await;

        let out_len = min(res.len(), res_len);
        res[..out_len].copy_from_slice(&buf[..out_len]);
        out_len
    }

    async fn ioctl_set_u32(&mut self, cmd: u32, iface: u32, val: u32) {
        let mut buf = val.to_le_bytes();
        self.ioctl(IoctlType::Set, cmd, iface, &mut buf).await;
    }

    async fn ioctl(&mut self, kind: IoctlType, cmd: u32, iface: u32, buf: &mut [u8]) -> usize {
        // TODO cancel ioctl on future drop.

        while !matches!(self.ioctl_state.get(), IoctlState::Idle) {
            yield_now().await;
        }

        self.ioctl_state.set(IoctlState::Pending { kind, cmd, iface, buf });

        let resp_len = loop {
            if let IoctlState::Done { resp_len } = self.ioctl_state.get() {
                break resp_len;
            }
            yield_now().await;
        };

        self.ioctl_state.set(IoctlState::Idle);

        resp_len
    }
}

pub struct Runner<'a, PWR, SPI> {
    ch: ch::Runner<'a, MTU>,
    bus: Bus<PWR, SPI>,

    ioctl_state: &'a Cell<IoctlState>,
    ioctl_id: u16,
    sdpcm_seq: u8,
    sdpcm_seq_max: u8,

    #[cfg(feature = "firmware-logs")]
    log: LogState,
}

#[cfg(feature = "firmware-logs")]
struct LogState {
    addr: u32,
    last_idx: usize,
    buf: [u8; 256],
    buf_count: usize,
}

pub type NetDriver<'a> = ch::Device<'a, MTU>;

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
    let (ch_runner, device) = ch::new(&mut state.ch, [0; 6]);
    let state_ch = ch_runner.state_runner();

    let mut runner = Runner {
        ch: ch_runner,
        bus: Bus::new(pwr, spi),

        ioctl_state: &state.ioctl_state,
        ioctl_id: 0,
        sdpcm_seq: 0,
        sdpcm_seq_max: 1,

        #[cfg(feature = "firmware-logs")]
        log: LogState {
            addr: 0,
            last_idx: 0,
            buf: [0; 256],
            buf_count: 0,
        },
    };

    runner.init(firmware).await;

    (
        device,
        Control {
            state_ch,
            ioctl_state: &state.ioctl_state,
        },
        runner,
    )
}

impl<'a, PWR, SPI> Runner<'a, PWR, SPI>
where
    PWR: OutputPin,
    SPI: SpiBusCyw43,
{
    async fn init(&mut self, firmware: &[u8]) {
        self.bus.init().await;

        // Init ALP (Active Low Power) clock
        self.bus
            .write8(FUNC_BACKPLANE, REG_BACKPLANE_CHIP_CLOCK_CSR, BACKPLANE_ALP_AVAIL_REQ)
            .await;
        info!("waiting for clock...");
        while self.bus.read8(FUNC_BACKPLANE, REG_BACKPLANE_CHIP_CLOCK_CSR).await & BACKPLANE_ALP_AVAIL == 0 {}
        info!("clock ok");

        let chip_id = self.bus.bp_read16(0x1800_0000).await;
        info!("chip ID: {}", chip_id);

        // Upload firmware.
        self.core_disable(Core::WLAN).await;
        self.core_reset(Core::SOCSRAM).await;
        self.bus.bp_write32(CHIP.socsram_base_address + 0x10, 3).await;
        self.bus.bp_write32(CHIP.socsram_base_address + 0x44, 0).await;

        let ram_addr = CHIP.atcm_ram_base_address;

        info!("loading fw");
        self.bus.bp_write(ram_addr, firmware).await;

        info!("loading nvram");
        // Round up to 4 bytes.
        let nvram_len = (NVRAM.len() + 3) / 4 * 4;
        self.bus
            .bp_write(ram_addr + CHIP.chip_ram_size - 4 - nvram_len as u32, NVRAM)
            .await;

        let nvram_len_words = nvram_len as u32 / 4;
        let nvram_len_magic = (!nvram_len_words << 16) | nvram_len_words;
        self.bus
            .bp_write32(ram_addr + CHIP.chip_ram_size - 4, nvram_len_magic)
            .await;

        // Start core!
        info!("starting up core...");
        self.core_reset(Core::WLAN).await;
        assert!(self.core_is_up(Core::WLAN).await);

        while self.bus.read8(FUNC_BACKPLANE, REG_BACKPLANE_CHIP_CLOCK_CSR).await & 0x80 == 0 {}

        // "Set up the interrupt mask and enable interrupts"
        self.bus.bp_write32(CHIP.sdiod_core_base_address + 0x24, 0xF0).await;

        // "Lower F2 Watermark to avoid DMA Hang in F2 when SD Clock is stopped."
        // Sounds scary...
        self.bus
            .write8(FUNC_BACKPLANE, REG_BACKPLANE_FUNCTION2_WATERMARK, 32)
            .await;

        // wait for wifi startup
        info!("waiting for wifi init...");
        while self.bus.read32(FUNC_BUS, REG_BUS_STATUS).await & STATUS_F2_RX_READY == 0 {}

        // Some random configs related to sleep.
        // These aren't needed if we don't want to sleep the bus.
        // TODO do we need to sleep the bus to read the irq line, due to
        // being on the same pin as MOSI/MISO?

        /*
        let mut val = self.bus.read8(FUNC_BACKPLANE, REG_BACKPLANE_WAKEUP_CTRL).await;
        val |= 0x02; // WAKE_TILL_HT_AVAIL
        self.bus.write8(FUNC_BACKPLANE, REG_BACKPLANE_WAKEUP_CTRL, val).await;
        self.bus.write8(FUNC_BUS, 0xF0, 0x08).await; // SDIOD_CCCR_BRCM_CARDCAP.CMD_NODEC = 1
        self.bus.write8(FUNC_BACKPLANE, REG_BACKPLANE_CHIP_CLOCK_CSR, 0x02).await; // SBSDIO_FORCE_HT

        let mut val = self.bus.read8(FUNC_BACKPLANE, REG_BACKPLANE_SLEEP_CSR).await;
        val |= 0x01; // SBSDIO_SLPCSR_KEEP_SDIO_ON
        self.bus.write8(FUNC_BACKPLANE, REG_BACKPLANE_SLEEP_CSR, val).await;
         */

        // clear pulls
        self.bus.write8(FUNC_BACKPLANE, REG_BACKPLANE_PULL_UP, 0).await;
        let _ = self.bus.read8(FUNC_BACKPLANE, REG_BACKPLANE_PULL_UP).await;

        // start HT clock
        //self.bus.write8(FUNC_BACKPLANE, REG_BACKPLANE_CHIP_CLOCK_CSR, 0x10).await;
        //info!("waiting for HT clock...");
        //while self.bus.read8(FUNC_BACKPLANE, REG_BACKPLANE_CHIP_CLOCK_CSR).await & 0x80 == 0 {}
        //info!("clock ok");

        #[cfg(feature = "firmware-logs")]
        self.log_init().await;

        info!("init done ");
    }

    #[cfg(feature = "firmware-logs")]
    async fn log_init(&mut self) {
        // Initialize shared memory for logging.

        let addr = CHIP.atcm_ram_base_address + CHIP.chip_ram_size - 4 - CHIP.socram_srmem_size;
        let shared_addr = self.bus.bp_read32(addr).await;
        info!("shared_addr {:08x}", shared_addr);

        let mut shared = [0; SharedMemData::SIZE];
        self.bus.bp_read(shared_addr, &mut shared).await;
        let shared = SharedMemData::from_bytes(&shared);
        info!("shared: {:08x}", shared);

        self.log.addr = shared.console_addr + 8;
    }

    #[cfg(feature = "firmware-logs")]
    async fn log_read(&mut self) {
        // Read log struct
        let mut log = [0; SharedMemLog::SIZE];
        self.bus.bp_read(self.log.addr, &mut log).await;
        let log = SharedMemLog::from_bytes(&log);

        let idx = log.idx as usize;

        // If pointer hasn't moved, no need to do anything.
        if idx == self.log.last_idx {
            return;
        }

        // Read entire buf for now. We could read only what we need, but then we
        // run into annoying alignment issues in `bp_read`.
        let mut buf = [0; 0x400];
        self.bus.bp_read(log.buf, &mut buf).await;

        while self.log.last_idx != idx as usize {
            let b = buf[self.log.last_idx];
            if b == b'\r' || b == b'\n' {
                if self.log.buf_count != 0 {
                    let s = unsafe { core::str::from_utf8_unchecked(&self.log.buf[..self.log.buf_count]) };
                    debug!("LOGS: {}", s);
                    self.log.buf_count = 0;
                }
            } else if self.log.buf_count < self.log.buf.len() {
                self.log.buf[self.log.buf_count] = b;
                self.log.buf_count += 1;
            }

            self.log.last_idx += 1;
            if self.log.last_idx == 0x400 {
                self.log.last_idx = 0;
            }
        }
    }

    pub async fn run(mut self) -> ! {
        let mut buf = [0; 512];
        loop {
            #[cfg(feature = "firmware-logs")]
            self.log_read().await;

            // Send stuff
            // TODO flow control not yet complete
            if !self.has_credit() {
                warn!("TX stalled");
            } else {
                if let IoctlState::Pending { kind, cmd, iface, buf } = self.ioctl_state.get() {
                    self.send_ioctl(kind, cmd, iface, unsafe { &*buf }).await;
                    self.ioctl_state.set(IoctlState::Sent { buf });
                }
                if !self.has_credit() {
                    warn!("TX stalled");
                } else {
                    if let Some(packet) = self.ch.try_tx_buf() {
                        trace!("tx pkt {:02x}", &packet[..packet.len().min(48)]);

                        let mut buf = [0; 512];
                        let buf8 = slice8_mut(&mut buf);

                        let total_len = SdpcmHeader::SIZE + BcdHeader::SIZE + packet.len();

                        let seq = self.sdpcm_seq;
                        self.sdpcm_seq = self.sdpcm_seq.wrapping_add(1);

                        let sdpcm_header = SdpcmHeader {
                            len: total_len as u16, // TODO does this len need to be rounded up to u32?
                            len_inv: !total_len as u16,
                            sequence: seq,
                            channel_and_flags: CHANNEL_TYPE_DATA,
                            next_length: 0,
                            header_length: SdpcmHeader::SIZE as _,
                            wireless_flow_control: 0,
                            bus_data_credit: 0,
                            reserved: [0, 0],
                        };

                        let bcd_header = BcdHeader {
                            flags: BDC_VERSION << BDC_VERSION_SHIFT,
                            priority: 0,
                            flags2: 0,
                            data_offset: 0,
                        };
                        trace!("tx {:?}", sdpcm_header);
                        trace!("    {:?}", bcd_header);

                        buf8[0..SdpcmHeader::SIZE].copy_from_slice(&sdpcm_header.to_bytes());
                        buf8[SdpcmHeader::SIZE..][..BcdHeader::SIZE].copy_from_slice(&bcd_header.to_bytes());
                        buf8[SdpcmHeader::SIZE + BcdHeader::SIZE..][..packet.len()].copy_from_slice(packet);

                        let total_len = (total_len + 3) & !3; // round up to 4byte

                        trace!("    {:02x}", &buf8[..total_len.min(48)]);

                        self.bus.wlan_write(&buf[..(total_len / 4)]).await;
                        self.ch.tx_done();
                    }
                }
            }

            // Receive stuff
            let irq = self.bus.read16(FUNC_BUS, REG_BUS_INTERRUPT).await;

            if irq & IRQ_F2_PACKET_AVAILABLE != 0 {
                let mut status = 0xFFFF_FFFF;
                while status == 0xFFFF_FFFF {
                    status = self.bus.read32(FUNC_BUS, REG_BUS_STATUS).await;
                }

                if status & STATUS_F2_PKT_AVAILABLE != 0 {
                    let len = (status & STATUS_F2_PKT_LEN_MASK) >> STATUS_F2_PKT_LEN_SHIFT;
                    self.bus.wlan_read(&mut buf, len).await;
                    trace!("rx {:02x}", &slice8_mut(&mut buf)[..(len as usize).min(48)]);
                    self.rx(&slice8_mut(&mut buf)[..len as usize]);
                }
            }

            // TODO use IRQs
            yield_now().await;
        }
    }

    fn rx(&mut self, packet: &[u8]) {
        if packet.len() < SdpcmHeader::SIZE {
            warn!("packet too short, len={}", packet.len());
            return;
        }

        let sdpcm_header = SdpcmHeader::from_bytes(packet[..SdpcmHeader::SIZE].try_into().unwrap());
        trace!("rx {:?}", sdpcm_header);
        if sdpcm_header.len != !sdpcm_header.len_inv {
            warn!("len inv mismatch");
            return;
        }
        if sdpcm_header.len as usize != packet.len() {
            // TODO: is this guaranteed??
            warn!("len from header doesn't match len from spi");
            return;
        }

        self.update_credit(&sdpcm_header);

        let channel = sdpcm_header.channel_and_flags & 0x0f;

        let payload = &packet[sdpcm_header.header_length as _..];

        match channel {
            CHANNEL_TYPE_CONTROL => {
                if payload.len() < CdcHeader::SIZE {
                    warn!("payload too short, len={}", payload.len());
                    return;
                }

                let cdc_header = CdcHeader::from_bytes(payload[..CdcHeader::SIZE].try_into().unwrap());
                trace!("    {:?}", cdc_header);

                if let IoctlState::Sent { buf } = self.ioctl_state.get() {
                    if cdc_header.id == self.ioctl_id {
                        if cdc_header.status != 0 {
                            // TODO: propagate error instead
                            panic!("IOCTL error {=i32}", cdc_header.status as i32);
                        }

                        let resp_len = cdc_header.len as usize;
                        info!("IOCTL Response: {:02x}", &payload[CdcHeader::SIZE..][..resp_len]);

                        (unsafe { &mut *buf }[..resp_len]).copy_from_slice(&payload[CdcHeader::SIZE..][..resp_len]);
                        self.ioctl_state.set(IoctlState::Done { resp_len });
                    }
                }
            }
            CHANNEL_TYPE_EVENT => {
                let bcd_header = BcdHeader::from_bytes(&payload[..BcdHeader::SIZE].try_into().unwrap());
                trace!("    {:?}", bcd_header);

                let packet_start = BcdHeader::SIZE + 4 * bcd_header.data_offset as usize;

                if packet_start + EventPacket::SIZE > payload.len() {
                    warn!("BCD event, incomplete header");
                    return;
                }
                let bcd_packet = &payload[packet_start..];
                trace!("    {:02x}", &bcd_packet[..(bcd_packet.len() as usize).min(36)]);

                let mut event_packet = EventPacket::from_bytes(&bcd_packet[..EventPacket::SIZE].try_into().unwrap());
                event_packet.byteswap();

                const ETH_P_LINK_CTL: u16 = 0x886c; // HPNA, wlan link local tunnel, according to linux if_ether.h
                if event_packet.eth.ether_type != ETH_P_LINK_CTL {
                    warn!(
                        "unexpected ethernet type 0x{:04x}, expected Broadcom ether type 0x{:04x}",
                        event_packet.eth.ether_type, ETH_P_LINK_CTL
                    );
                    return;
                }
                const BROADCOM_OUI: &[u8] = &[0x00, 0x10, 0x18];
                if event_packet.hdr.oui != BROADCOM_OUI {
                    warn!(
                        "unexpected ethernet OUI {:02x}, expected Broadcom OUI {:02x}",
                        event_packet.hdr.oui, BROADCOM_OUI
                    );
                    return;
                }
                const BCMILCP_SUBTYPE_VENDOR_LONG: u16 = 32769;
                if event_packet.hdr.subtype != BCMILCP_SUBTYPE_VENDOR_LONG {
                    warn!("unexpected subtype {}", event_packet.hdr.subtype);
                    return;
                }

                const BCMILCP_BCM_SUBTYPE_EVENT: u16 = 1;
                if event_packet.hdr.user_subtype != BCMILCP_BCM_SUBTYPE_EVENT {
                    warn!("unexpected user_subtype {}", event_packet.hdr.subtype);
                    return;
                }

                if event_packet.msg.datalen as usize >= (bcd_packet.len() - EventMessage::SIZE) {
                    warn!("BCD event, incomplete data");
                    return;
                }

                let evt_data = &bcd_packet[EventMessage::SIZE..][..event_packet.msg.datalen as usize];
                debug!(
                    "=== EVENT {}: {} {:02x}",
                    events::Event::from(event_packet.msg.event_type as u8),
                    event_packet.msg,
                    evt_data
                );
            }
            CHANNEL_TYPE_DATA => {
                let bcd_header = BcdHeader::from_bytes(&payload[..BcdHeader::SIZE].try_into().unwrap());
                trace!("    {:?}", bcd_header);

                let packet_start = BcdHeader::SIZE + 4 * bcd_header.data_offset as usize;
                if packet_start > payload.len() {
                    warn!("packet start out of range.");
                    return;
                }
                let packet = &payload[packet_start..];
                trace!("rx pkt {:02x}", &packet[..(packet.len() as usize).min(48)]);

                match self.ch.try_rx_buf() {
                    Some(buf) => {
                        buf[..packet.len()].copy_from_slice(packet);
                        self.ch.rx_done(packet.len())
                    }
                    None => warn!("failed to push rxd packet to the channel."),
                }
            }
            _ => {}
        }
    }

    fn update_credit(&mut self, sdpcm_header: &SdpcmHeader) {
        if sdpcm_header.channel_and_flags & 0xf < 3 {
            let mut sdpcm_seq_max = sdpcm_header.bus_data_credit;
            if sdpcm_seq_max.wrapping_sub(self.sdpcm_seq) > 0x40 {
                sdpcm_seq_max = self.sdpcm_seq + 2;
            }
            self.sdpcm_seq_max = sdpcm_seq_max;
        }
    }

    fn has_credit(&self) -> bool {
        self.sdpcm_seq != self.sdpcm_seq_max && self.sdpcm_seq_max.wrapping_sub(self.sdpcm_seq) & 0x80 == 0
    }

    async fn send_ioctl(&mut self, kind: IoctlType, cmd: u32, iface: u32, data: &[u8]) {
        let mut buf = [0; 512];
        let buf8 = slice8_mut(&mut buf);

        let total_len = SdpcmHeader::SIZE + CdcHeader::SIZE + data.len();

        let sdpcm_seq = self.sdpcm_seq;
        self.sdpcm_seq = self.sdpcm_seq.wrapping_add(1);
        self.ioctl_id = self.ioctl_id.wrapping_add(1);

        let sdpcm_header = SdpcmHeader {
            len: total_len as u16, // TODO does this len need to be rounded up to u32?
            len_inv: !total_len as u16,
            sequence: sdpcm_seq,
            channel_and_flags: CHANNEL_TYPE_CONTROL,
            next_length: 0,
            header_length: SdpcmHeader::SIZE as _,
            wireless_flow_control: 0,
            bus_data_credit: 0,
            reserved: [0, 0],
        };

        let cdc_header = CdcHeader {
            cmd: cmd,
            len: data.len() as _,
            flags: kind as u16 | (iface as u16) << 12,
            id: self.ioctl_id,
            status: 0,
        };
        trace!("tx {:?}", sdpcm_header);
        trace!("    {:?}", cdc_header);

        buf8[0..SdpcmHeader::SIZE].copy_from_slice(&sdpcm_header.to_bytes());
        buf8[SdpcmHeader::SIZE..][..CdcHeader::SIZE].copy_from_slice(&cdc_header.to_bytes());
        buf8[SdpcmHeader::SIZE + CdcHeader::SIZE..][..data.len()].copy_from_slice(data);

        let total_len = (total_len + 3) & !3; // round up to 4byte

        trace!("    {:02x}", &buf8[..total_len.min(48)]);

        self.bus.wlan_write(&buf[..total_len / 4]).await;
    }

    async fn core_disable(&mut self, core: Core) {
        let base = core.base_addr();

        // Dummy read?
        let _ = self.bus.bp_read8(base + AI_RESETCTRL_OFFSET).await;

        // Check it isn't already reset
        let r = self.bus.bp_read8(base + AI_RESETCTRL_OFFSET).await;
        if r & AI_RESETCTRL_BIT_RESET != 0 {
            return;
        }

        self.bus.bp_write8(base + AI_IOCTRL_OFFSET, 0).await;
        let _ = self.bus.bp_read8(base + AI_IOCTRL_OFFSET).await;

        block_for(Duration::from_millis(1));

        self.bus
            .bp_write8(base + AI_RESETCTRL_OFFSET, AI_RESETCTRL_BIT_RESET)
            .await;
        let _ = self.bus.bp_read8(base + AI_RESETCTRL_OFFSET).await;
    }

    async fn core_reset(&mut self, core: Core) {
        self.core_disable(core).await;

        let base = core.base_addr();
        self.bus
            .bp_write8(base + AI_IOCTRL_OFFSET, AI_IOCTRL_BIT_FGC | AI_IOCTRL_BIT_CLOCK_EN)
            .await;
        let _ = self.bus.bp_read8(base + AI_IOCTRL_OFFSET).await;

        self.bus.bp_write8(base + AI_RESETCTRL_OFFSET, 0).await;

        Timer::after(Duration::from_millis(1)).await;

        self.bus
            .bp_write8(base + AI_IOCTRL_OFFSET, AI_IOCTRL_BIT_CLOCK_EN)
            .await;
        let _ = self.bus.bp_read8(base + AI_IOCTRL_OFFSET).await;

        Timer::after(Duration::from_millis(1)).await;
    }

    async fn core_is_up(&mut self, core: Core) -> bool {
        let base = core.base_addr();

        let io = self.bus.bp_read8(base + AI_IOCTRL_OFFSET).await;
        if io & (AI_IOCTRL_BIT_FGC | AI_IOCTRL_BIT_CLOCK_EN) != AI_IOCTRL_BIT_CLOCK_EN {
            debug!("core_is_up: returning false due to bad ioctrl {:02x}", io);
            return false;
        }

        let r = self.bus.bp_read8(base + AI_RESETCTRL_OFFSET).await;
        if r & (AI_RESETCTRL_BIT_RESET) != 0 {
            debug!("core_is_up: returning false due to bad resetctrl {:02x}", r);
            return false;
        }

        true
    }
}

fn slice8_mut(x: &mut [u32]) -> &mut [u8] {
    let len = x.len() * 4;
    unsafe { slice::from_raw_parts_mut(x.as_mut_ptr() as _, len) }
}

macro_rules! nvram {
    ($($s:literal,)*) => {
        concat_bytes!($($s, b"\x00",)* b"\x00\x00")
    };
}

static NVRAM: &'static [u8] = &*nvram!(
    b"NVRAMRev=$Rev$",
    b"manfid=0x2d0",
    b"prodid=0x0727",
    b"vendid=0x14e4",
    b"devid=0x43e2",
    b"boardtype=0x0887",
    b"boardrev=0x1100",
    b"boardnum=22",
    b"macaddr=00:A0:50:b5:59:5e",
    b"sromrev=11",
    b"boardflags=0x00404001",
    b"boardflags3=0x04000000",
    b"xtalfreq=37400",
    b"nocrc=1",
    b"ag0=255",
    b"aa2g=1",
    b"ccode=ALL",
    b"pa0itssit=0x20",
    b"extpagain2g=0",
    b"pa2ga0=-168,6649,-778",
    b"AvVmid_c0=0x0,0xc8",
    b"cckpwroffset0=5",
    b"maxp2ga0=84",
    b"txpwrbckof=6",
    b"cckbw202gpo=0",
    b"legofdmbw202gpo=0x66111111",
    b"mcsbw202gpo=0x77711111",
    b"propbw202gpo=0xdd",
    b"ofdmdigfilttype=18",
    b"ofdmdigfilttypebe=18",
    b"papdmode=1",
    b"papdvalidtest=1",
    b"pacalidx2g=45",
    b"papdepsoffset=-30",
    b"papdendidx=58",
    b"ltecxmux=0",
    b"ltecxpadnum=0x0102",
    b"ltecxfnsel=0x44",
    b"ltecxgcigpio=0x01",
    b"il0macaddr=00:90:4c:c5:12:38",
    b"wl0id=0x431b",
    b"deadman_to=0xffffffff",
    b"muxenab=0x100",
    b"spurconfig=0x3",
    b"glitch_based_crsmin=1",
    b"btc_mode=1",
);
