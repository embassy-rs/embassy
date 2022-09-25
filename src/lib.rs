#![no_std]
#![no_main]
#![feature(type_alias_impl_trait, concat_bytes)]
#![deny(unused_must_use)]

// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;

mod countries;
mod events;
mod structs;

use core::cell::Cell;
use core::cmp::{max, min};
use core::slice;
use core::sync::atomic::Ordering;
use core::task::Waker;

use atomic_polyfill::AtomicBool;
use embassy_futures::yield_now;
use embassy_net::{PacketBoxExt, PacketBuf};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::{block_for, Duration, Timer};
use embedded_hal_1::digital::blocking::OutputPin;
use embedded_hal_async::spi::{SpiBusRead, SpiBusWrite, SpiDevice};

use self::structs::*;
use crate::events::Event;

fn swap16(x: u32) -> u32 {
    x.rotate_left(16)
}

// CYW_SPID command structure constants.
const WRITE: bool = true;
const READ: bool = false;
const INC_ADDR: bool = true;
#[allow(unused)]
const FIXED_ADDR: bool = false;

fn cmd_word(write: bool, incr: bool, func: u32, addr: u32, len: u32) -> u32 {
    (write as u32) << 31 | (incr as u32) << 30 | (func & 0b11) << 28 | (addr & 0x1FFFF) << 11 | (len & 0x7FF)
}

fn slice8_mut(x: &mut [u32]) -> &mut [u8] {
    let len = x.len() * 4;
    unsafe { slice::from_raw_parts_mut(x.as_mut_ptr() as _, len) }
}

const FUNC_BUS: u32 = 0;
const FUNC_BACKPLANE: u32 = 1;
const FUNC_WLAN: u32 = 2;
const FUNC_BT: u32 = 3;

const REG_BUS_CTRL: u32 = 0x0;
const REG_BUS_INTERRUPT: u32 = 0x04; // 16 bits - Interrupt status
const REG_BUS_INTERRUPT_ENABLE: u32 = 0x06; // 16 bits - Interrupt mask
const REG_BUS_STATUS: u32 = 0x8;
const REG_BUS_TEST_RO: u32 = 0x14;
const REG_BUS_TEST_RW: u32 = 0x18;
const REG_BUS_RESP_DELAY: u32 = 0x1c;
const WORD_LENGTH_32: u32 = 0x1;
const HIGH_SPEED: u32 = 0x10;

// SPI_STATUS_REGISTER bits
const STATUS_DATA_NOT_AVAILABLE: u32 = 0x00000001;
const STATUS_UNDERFLOW: u32 = 0x00000002;
const STATUS_OVERFLOW: u32 = 0x00000004;
const STATUS_F2_INTR: u32 = 0x00000008;
const STATUS_F3_INTR: u32 = 0x00000010;
const STATUS_F2_RX_READY: u32 = 0x00000020;
const STATUS_F3_RX_READY: u32 = 0x00000040;
const STATUS_HOST_CMD_DATA_ERR: u32 = 0x00000080;
const STATUS_F2_PKT_AVAILABLE: u32 = 0x00000100;
const STATUS_F2_PKT_LEN_MASK: u32 = 0x000FFE00;
const STATUS_F2_PKT_LEN_SHIFT: u32 = 9;
const STATUS_F3_PKT_AVAILABLE: u32 = 0x00100000;
const STATUS_F3_PKT_LEN_MASK: u32 = 0xFFE00000;
const STATUS_F3_PKT_LEN_SHIFT: u32 = 21;

const REG_BACKPLANE_GPIO_SELECT: u32 = 0x10005;
const REG_BACKPLANE_GPIO_OUTPUT: u32 = 0x10006;
const REG_BACKPLANE_GPIO_ENABLE: u32 = 0x10007;
const REG_BACKPLANE_FUNCTION2_WATERMARK: u32 = 0x10008;
const REG_BACKPLANE_DEVICE_CONTROL: u32 = 0x10009;
const REG_BACKPLANE_BACKPLANE_ADDRESS_LOW: u32 = 0x1000A;
const REG_BACKPLANE_BACKPLANE_ADDRESS_MID: u32 = 0x1000B;
const REG_BACKPLANE_BACKPLANE_ADDRESS_HIGH: u32 = 0x1000C;
const REG_BACKPLANE_FRAME_CONTROL: u32 = 0x1000D;
const REG_BACKPLANE_CHIP_CLOCK_CSR: u32 = 0x1000E;
const REG_BACKPLANE_PULL_UP: u32 = 0x1000F;
const REG_BACKPLANE_READ_FRAME_BC_LOW: u32 = 0x1001B;
const REG_BACKPLANE_READ_FRAME_BC_HIGH: u32 = 0x1001C;
const REG_BACKPLANE_WAKEUP_CTRL: u32 = 0x1001E;
const REG_BACKPLANE_SLEEP_CSR: u32 = 0x1001F;

const BACKPLANE_WINDOW_SIZE: usize = 0x8000;
const BACKPLANE_ADDRESS_MASK: u32 = 0x7FFF;
const BACKPLANE_ADDRESS_32BIT_FLAG: u32 = 0x08000;
const BACKPLANE_MAX_TRANSFER_SIZE: usize = 64;
// Active Low Power (ALP) clock constants
const BACKPLANE_ALP_AVAIL_REQ: u8 = 0x08;
const BACKPLANE_ALP_AVAIL: u8 = 0x40;

// Broadcom AMBA (Advanced Microcontroller Bus Architecture) Interconnect (AI)
// constants
const AI_IOCTRL_OFFSET: u32 = 0x408;
const AI_IOCTRL_BIT_FGC: u8 = 0x0002;
const AI_IOCTRL_BIT_CLOCK_EN: u8 = 0x0001;
const AI_IOCTRL_BIT_CPUHALT: u8 = 0x0020;

const AI_RESETCTRL_OFFSET: u32 = 0x800;
const AI_RESETCTRL_BIT_RESET: u8 = 1;

const AI_RESETSTATUS_OFFSET: u32 = 0x804;

const TEST_PATTERN: u32 = 0x12345678;
const FEEDBEAD: u32 = 0xFEEDBEAD;

// SPI_INTERRUPT_REGISTER and SPI_INTERRUPT_ENABLE_REGISTER Bits
const IRQ_DATA_UNAVAILABLE: u16 = 0x0001; // Requested data not available; Clear by writing a "1"
const IRQ_F2_F3_FIFO_RD_UNDERFLOW: u16 = 0x0002;
const IRQ_F2_F3_FIFO_WR_OVERFLOW: u16 = 0x0004;
const IRQ_COMMAND_ERROR: u16 = 0x0008; // Cleared by writing 1
const IRQ_DATA_ERROR: u16 = 0x0010; // Cleared by writing 1
const IRQ_F2_PACKET_AVAILABLE: u16 = 0x0020;
const IRQ_F3_PACKET_AVAILABLE: u16 = 0x0040;
const IRQ_F1_OVERFLOW: u16 = 0x0080; // Due to last write. Bkplane has pending write requests
const IRQ_MISC_INTR0: u16 = 0x0100;
const IRQ_MISC_INTR1: u16 = 0x0200;
const IRQ_MISC_INTR2: u16 = 0x0400;
const IRQ_MISC_INTR3: u16 = 0x0800;
const IRQ_MISC_INTR4: u16 = 0x1000;
const IRQ_F1_INTR: u16 = 0x2000;
const IRQ_F2_INTR: u16 = 0x4000;
const IRQ_F3_INTR: u16 = 0x8000;

const IOCTL_CMD_UP: u32 = 2;
const IOCTL_CMD_SET_SSID: u32 = 26;
const IOCTL_CMD_SET_VAR: u32 = 263;
const IOCTL_CMD_GET_VAR: u32 = 262;
const IOCTL_CMD_SET_PASSPHRASE: u32 = 268;

const CHANNEL_TYPE_CONTROL: u8 = 0;
const CHANNEL_TYPE_EVENT: u8 = 1;
const CHANNEL_TYPE_DATA: u8 = 2;

#[derive(Clone, Copy)]
pub enum IoctlType {
    Get = 0,
    Set = 2,
}

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

    tx_channel: Channel<NoopRawMutex, PacketBuf, 8>,
    rx_channel: Channel<NoopRawMutex, PacketBuf, 8>,
    link_up: AtomicBool,
}

impl State {
    pub fn new() -> Self {
        Self {
            ioctl_state: Cell::new(IoctlState::Idle),

            tx_channel: Channel::new(),
            rx_channel: Channel::new(),
            link_up: AtomicBool::new(true), // TODO set up/down as we join/deassociate
        }
    }
}

pub struct Control<'a> {
    state: &'a State,
}

impl<'a> Control<'a> {
    pub async fn init(&mut self, clm: &[u8]) -> NetDevice<'a> {
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

        self.ioctl_set_u32(64, 0, 0).await; // WLC_SET_ANTDIV

        self.set_iovar_u32("bus:txglom", 0).await;
        Timer::after(Duration::from_millis(100)).await;
        //self.set_iovar_u32("apsta", 1).await; // this crashes, also we already did it before...??
        Timer::after(Duration::from_millis(100)).await;
        self.set_iovar_u32("ampdu_ba_wsize", 8).await;
        Timer::after(Duration::from_millis(100)).await;
        self.set_iovar_u32("ampdu_mpdu", 4).await;
        Timer::after(Duration::from_millis(100)).await;
        //self.set_iovar_u32("ampdu_rx_factor", 0).await; // this crashes

        Timer::after(Duration::from_millis(100)).await;

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

        // power save mode 2
        self.set_iovar_u32("pm2_sleep_ret", 0xc8).await;
        self.set_iovar_u32("bcn_li_bcn", 1).await;
        self.set_iovar_u32("bcn_li_dtim", 1).await;
        self.set_iovar_u32("assoc_listen", 10).await;
        self.ioctl_set_u32(0x86, 0, 2).await;

        self.ioctl_set_u32(110, 0, 1).await; // SET_GMODE = auto
        self.ioctl_set_u32(142, 0, 0).await; // SET_BAND = any

        Timer::after(Duration::from_millis(100)).await;

        info!("INIT DONE");

        NetDevice {
            state: self.state,
            mac_addr,
        }
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

        while !matches!(self.state.ioctl_state.get(), IoctlState::Idle) {
            yield_now().await;
        }

        self.state
            .ioctl_state
            .set(IoctlState::Pending { kind, cmd, iface, buf });

        let resp_len = loop {
            if let IoctlState::Done { resp_len } = self.state.ioctl_state.get() {
                break resp_len;
            }
            yield_now().await;
        };

        self.state.ioctl_state.set(IoctlState::Idle);

        resp_len
    }
}

pub struct NetDevice<'a> {
    state: &'a State,
    mac_addr: [u8; 6],
}

impl<'a> embassy_net::Device for NetDevice<'a> {
    fn register_waker(&mut self, waker: &Waker) {
        // loopy loopy wakey wakey
        waker.wake_by_ref()
    }

    fn link_state(&mut self) -> embassy_net::LinkState {
        match self.state.link_up.load(Ordering::Relaxed) {
            true => embassy_net::LinkState::Up,
            false => embassy_net::LinkState::Down,
        }
    }

    fn capabilities(&self) -> embassy_net::DeviceCapabilities {
        let mut caps = embassy_net::DeviceCapabilities::default();
        caps.max_transmission_unit = 1514; // 1500 IP + 14 ethernet header
        caps.medium = embassy_net::Medium::Ethernet;
        caps
    }

    fn is_transmit_ready(&mut self) -> bool {
        true
    }

    fn transmit(&mut self, pkt: PacketBuf) {
        if self.state.tx_channel.try_send(pkt).is_err() {
            warn!("TX failed")
        }
    }

    fn receive(&mut self) -> Option<PacketBuf> {
        self.state.rx_channel.try_recv().ok()
    }

    fn ethernet_address(&self) -> [u8; 6] {
        self.mac_addr
    }
}

pub struct Runner<'a, PWR, SPI> {
    state: &'a State,

    pwr: PWR,
    spi: SPI,

    ioctl_id: u16,
    sdpcm_seq: u8,
    backplane_window: u32,

    sdpcm_seq_max: u8,
}

pub async fn new<'a, PWR, SPI>(
    state: &'a State,
    pwr: PWR,
    spi: SPI,
    firmware: &[u8],
) -> (Control<'a>, Runner<'a, PWR, SPI>)
where
    PWR: OutputPin,
    SPI: SpiDevice,
    SPI::Bus: SpiBusRead<u32> + SpiBusWrite<u32>,
{
    let mut runner = Runner {
        state,
        pwr,
        spi,

        ioctl_id: 0,
        sdpcm_seq: 0,
        backplane_window: 0xAAAA_AAAA,

        sdpcm_seq_max: 1,
    };

    runner.init(firmware).await;

    (Control { state }, runner)
}

impl<'a, PWR, SPI> Runner<'a, PWR, SPI>
where
    PWR: OutputPin,
    SPI: SpiDevice,
    SPI::Bus: SpiBusRead<u32> + SpiBusWrite<u32>,
{
    async fn init(&mut self, firmware: &[u8]) {
        // Reset
        self.pwr.set_low().unwrap();
        Timer::after(Duration::from_millis(20)).await;
        self.pwr.set_high().unwrap();
        Timer::after(Duration::from_millis(250)).await;

        info!("waiting for ping...");
        while self.read32_swapped(REG_BUS_TEST_RO).await != FEEDBEAD {}
        info!("ping ok");

        self.write32_swapped(REG_BUS_TEST_RW, TEST_PATTERN).await;
        let val = self.read32_swapped(REG_BUS_TEST_RW).await;
        assert_eq!(val, TEST_PATTERN);

        // 32-bit word length, little endian (which is the default endianess).
        self.write32_swapped(REG_BUS_CTRL, WORD_LENGTH_32 | HIGH_SPEED).await;

        let val = self.read32(FUNC_BUS, REG_BUS_TEST_RO).await;
        assert_eq!(val, FEEDBEAD);
        let val = self.read32(FUNC_BUS, REG_BUS_TEST_RW).await;
        assert_eq!(val, TEST_PATTERN);

        // No response delay in any of the funcs.
        // seems to break backplane??? eat the 4-byte delay instead, that's what the vendor drivers do...
        //self.write32(FUNC_BUS, REG_BUS_RESP_DELAY, 0).await;

        // Init ALP (Active Low Power) clock
        self.write8(FUNC_BACKPLANE, REG_BACKPLANE_CHIP_CLOCK_CSR, BACKPLANE_ALP_AVAIL_REQ)
            .await;
        info!("waiting for clock...");
        while self.read8(FUNC_BACKPLANE, REG_BACKPLANE_CHIP_CLOCK_CSR).await & BACKPLANE_ALP_AVAIL == 0 {}
        info!("clock ok");

        let chip_id = self.bp_read16(0x1800_0000).await;
        info!("chip ID: {}", chip_id);

        // Upload firmware.
        self.core_disable(Core::WLAN).await;
        self.core_reset(Core::SOCSRAM).await;
        self.bp_write32(CHIP.socsram_base_address + 0x10, 3).await;
        self.bp_write32(CHIP.socsram_base_address + 0x44, 0).await;

        let ram_addr = CHIP.atcm_ram_base_address;

        info!("loading fw");
        self.bp_write(ram_addr, firmware).await;

        info!("loading nvram");
        // Round up to 4 bytes.
        let nvram_len = (NVRAM.len() + 3) / 4 * 4;
        self.bp_write(ram_addr + CHIP.chip_ram_size - 4 - nvram_len as u32, NVRAM)
            .await;

        let nvram_len_words = nvram_len as u32 / 4;
        let nvram_len_magic = (!nvram_len_words << 16) | nvram_len_words;
        self.bp_write32(ram_addr + CHIP.chip_ram_size - 4, nvram_len_magic)
            .await;

        // Start core!
        info!("starting up core...");
        self.core_reset(Core::WLAN).await;
        assert!(self.core_is_up(Core::WLAN).await);

        while self.read8(FUNC_BACKPLANE, REG_BACKPLANE_CHIP_CLOCK_CSR).await & 0x80 == 0 {}

        // "Set up the interrupt mask and enable interrupts"
        self.bp_write32(CHIP.sdiod_core_base_address + 0x24, 0xF0).await;

        // "Lower F2 Watermark to avoid DMA Hang in F2 when SD Clock is stopped."
        // Sounds scary...
        self.write8(FUNC_BACKPLANE, REG_BACKPLANE_FUNCTION2_WATERMARK, 32).await;

        // wait for wifi startup
        info!("waiting for wifi init...");
        while self.read32(FUNC_BUS, REG_BUS_STATUS).await & STATUS_F2_RX_READY == 0 {}

        // Some random configs related to sleep.
        // These aren't needed if we don't want to sleep the bus.
        // TODO do we need to sleep the bus to read the irq line, due to
        // being on the same pin as MOSI/MISO?

        /*
        let mut val = self.read8(FUNC_BACKPLANE, REG_BACKPLANE_WAKEUP_CTRL).await;
        val |= 0x02; // WAKE_TILL_HT_AVAIL
        self.write8(FUNC_BACKPLANE, REG_BACKPLANE_WAKEUP_CTRL, val).await;
        self.write8(FUNC_BUS, 0xF0, 0x08).await; // SDIOD_CCCR_BRCM_CARDCAP.CMD_NODEC = 1
        self.write8(FUNC_BACKPLANE, REG_BACKPLANE_CHIP_CLOCK_CSR, 0x02).await; // SBSDIO_FORCE_HT

        let mut val = self.read8(FUNC_BACKPLANE, REG_BACKPLANE_SLEEP_CSR).await;
        val |= 0x01; // SBSDIO_SLPCSR_KEEP_SDIO_ON
        self.write8(FUNC_BACKPLANE, REG_BACKPLANE_SLEEP_CSR, val).await;
         */

        // clear pulls
        self.write8(FUNC_BACKPLANE, REG_BACKPLANE_PULL_UP, 0).await;
        let _ = self.read8(FUNC_BACKPLANE, REG_BACKPLANE_PULL_UP).await;

        // start HT clock
        //self.write8(FUNC_BACKPLANE, REG_BACKPLANE_CHIP_CLOCK_CSR, 0x10).await;
        //info!("waiting for HT clock...");
        //while self.read8(FUNC_BACKPLANE, REG_BACKPLANE_CHIP_CLOCK_CSR).await & 0x80 == 0 {}
        //info!("clock ok");

        info!("init done ");
    }

    pub async fn run(mut self) -> ! {
        let mut buf = [0; 512];
        loop {
            // Send stuff
            // TODO flow control not yet complete
            if !self.has_credit() {
                warn!("TX stalled");
            } else {
                if let IoctlState::Pending { kind, cmd, iface, buf } = self.state.ioctl_state.get() {
                    self.send_ioctl(kind, cmd, iface, unsafe { &*buf }).await;
                    self.state.ioctl_state.set(IoctlState::Sent { buf });
                }
                if !self.has_credit() {
                    warn!("TX stalled");
                } else {
                    if let Ok(p) = self.state.tx_channel.try_recv() {
                        self.send_packet(&p).await;
                    }
                }
            }

            // Receive stuff
            let irq = self.read16(FUNC_BUS, REG_BUS_INTERRUPT).await;

            if irq & IRQ_F2_PACKET_AVAILABLE != 0 {
                let mut status = 0xFFFF_FFFF;
                while status == 0xFFFF_FFFF {
                    status = self.read32(FUNC_BUS, REG_BUS_STATUS).await;
                }

                if status & STATUS_F2_PKT_AVAILABLE != 0 {
                    let len = (status & STATUS_F2_PKT_LEN_MASK) >> STATUS_F2_PKT_LEN_SHIFT;

                    let cmd = cmd_word(READ, INC_ADDR, FUNC_WLAN, 0, len);

                    self.spi
                        .transaction(|bus| {
                            let bus = unsafe { &mut *bus };
                            async {
                                bus.write(&[cmd]).await?;
                                bus.read(&mut buf[..(len as usize + 3) / 4]).await?;
                                Ok(())
                            }
                        })
                        .await
                        .unwrap();

                    trace!("rx {:02x}", &slice8_mut(&mut buf)[..(len as usize).min(48)]);

                    self.rx(&slice8_mut(&mut buf)[..len as usize]);
                }
            }

            // TODO use IRQs
            yield_now().await;
        }
    }

    async fn send_packet(&mut self, packet: &[u8]) {
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
            flags: 0x20,
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

        let cmd = cmd_word(WRITE, INC_ADDR, FUNC_WLAN, 0, total_len as _);
        self.spi
            .transaction(|bus| {
                let bus = unsafe { &mut *bus };
                async {
                    bus.write(&[cmd]).await?;
                    bus.write(&buf[..(total_len / 4)]).await?;
                    Ok(())
                }
            })
            .await
            .unwrap();
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

                if let IoctlState::Sent { buf } = self.state.ioctl_state.get() {
                    if cdc_header.id == self.ioctl_id {
                        assert_eq!(cdc_header.status, 0); // todo propagate error instead

                        let resp_len = cdc_header.len as usize;
                        info!("IOCTL Response: {:02x}", &payload[CdcHeader::SIZE..][..resp_len]);

                        (unsafe { &mut *buf }[..resp_len]).copy_from_slice(&payload[CdcHeader::SIZE..][..resp_len]);
                        self.state.ioctl_state.set(IoctlState::Done { resp_len });
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

                let mut p = unwrap!(embassy_net::PacketBox::new(embassy_net::Packet::new()));
                p[..packet.len()].copy_from_slice(packet);

                if let Err(_) = self.state.rx_channel.try_send(p.slice(0..packet.len())) {
                    warn!("failed to push rxd packet to the channel.")
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

        let cmd = cmd_word(WRITE, INC_ADDR, FUNC_WLAN, 0, total_len as _);

        self.spi
            .transaction(|bus| {
                let bus = unsafe { &mut *bus };
                async {
                    bus.write(&[cmd]).await?;
                    bus.write(&buf[..total_len / 4]).await?;
                    Ok(())
                }
            })
            .await
            .unwrap();
    }

    async fn core_disable(&mut self, core: Core) {
        let base = core.base_addr();

        // Dummy read?
        let _ = self.bp_read8(base + AI_RESETCTRL_OFFSET).await;

        // Check it isn't already reset
        let r = self.bp_read8(base + AI_RESETCTRL_OFFSET).await;
        if r & AI_RESETCTRL_BIT_RESET != 0 {
            return;
        }

        self.bp_write8(base + AI_IOCTRL_OFFSET, 0).await;
        let _ = self.bp_read8(base + AI_IOCTRL_OFFSET).await;

        block_for(Duration::from_millis(1));

        self.bp_write8(base + AI_RESETCTRL_OFFSET, AI_RESETCTRL_BIT_RESET).await;
        let _ = self.bp_read8(base + AI_RESETCTRL_OFFSET).await;
    }

    async fn core_reset(&mut self, core: Core) {
        self.core_disable(core).await;

        let base = core.base_addr();
        self.bp_write8(base + AI_IOCTRL_OFFSET, AI_IOCTRL_BIT_FGC | AI_IOCTRL_BIT_CLOCK_EN)
            .await;
        let _ = self.bp_read8(base + AI_IOCTRL_OFFSET).await;

        self.bp_write8(base + AI_RESETCTRL_OFFSET, 0).await;

        Timer::after(Duration::from_millis(1)).await;

        self.bp_write8(base + AI_IOCTRL_OFFSET, AI_IOCTRL_BIT_CLOCK_EN).await;
        let _ = self.bp_read8(base + AI_IOCTRL_OFFSET).await;

        Timer::after(Duration::from_millis(1)).await;
    }

    async fn core_is_up(&mut self, core: Core) -> bool {
        let base = core.base_addr();

        let io = self.bp_read8(base + AI_IOCTRL_OFFSET).await;
        if io & (AI_IOCTRL_BIT_FGC | AI_IOCTRL_BIT_CLOCK_EN) != AI_IOCTRL_BIT_CLOCK_EN {
            debug!("core_is_up: returning false due to bad ioctrl {:02x}", io);
            return false;
        }

        let r = self.bp_read8(base + AI_RESETCTRL_OFFSET).await;
        if r & (AI_RESETCTRL_BIT_RESET) != 0 {
            debug!("core_is_up: returning false due to bad resetctrl {:02x}", r);
            return false;
        }

        true
    }

    async fn bp_read(&mut self, mut addr: u32, mut data: &mut [u32]) {
        // It seems the HW force-aligns the addr
        // to 2 if data.len() >= 2
        // to 4 if data.len() >= 4
        // To simplify, enforce 4-align for now.
        assert!(addr % 4 == 0);

        while !data.is_empty() {
            // Ensure transfer doesn't cross a window boundary.
            let window_offs = addr & BACKPLANE_ADDRESS_MASK;
            let window_remaining = BACKPLANE_WINDOW_SIZE - window_offs as usize;

            let len = data.len().min(BACKPLANE_MAX_TRANSFER_SIZE).min(window_remaining);

            self.backplane_set_window(addr).await;

            let cmd = cmd_word(READ, INC_ADDR, FUNC_BACKPLANE, window_offs, len as u32);

            self.spi
                .transaction(|bus| {
                    let bus = unsafe { &mut *bus };
                    async {
                        bus.write(&[cmd]).await?;

                        // 4-byte response delay.
                        let mut junk = [0; 1];
                        bus.read(&mut junk).await?;

                        // Read data
                        bus.read(&mut data[..len / 4]).await?;
                        Ok(())
                    }
                })
                .await
                .unwrap();

            // Advance ptr.
            addr += len as u32;
            data = &mut data[len / 4..];
        }
    }

    async fn bp_write(&mut self, mut addr: u32, mut data: &[u8]) {
        // It seems the HW force-aligns the addr
        // to 2 if data.len() >= 2
        // to 4 if data.len() >= 4
        // To simplify, enforce 4-align for now.
        assert!(addr % 4 == 0);

        let mut buf = [0u32; BACKPLANE_MAX_TRANSFER_SIZE / 4];

        while !data.is_empty() {
            // Ensure transfer doesn't cross a window boundary.
            let window_offs = addr & BACKPLANE_ADDRESS_MASK;
            let window_remaining = BACKPLANE_WINDOW_SIZE - window_offs as usize;

            let len = data.len().min(BACKPLANE_MAX_TRANSFER_SIZE).min(window_remaining);
            slice8_mut(&mut buf)[..len].copy_from_slice(&data[..len]);

            self.backplane_set_window(addr).await;

            let cmd = cmd_word(WRITE, INC_ADDR, FUNC_BACKPLANE, window_offs, len as u32);

            self.spi
                .transaction(|bus| {
                    let bus = unsafe { &mut *bus };
                    async {
                        bus.write(&[cmd]).await?;
                        bus.write(&buf[..(len + 3) / 4]).await?;
                        Ok(())
                    }
                })
                .await
                .unwrap();

            // Advance ptr.
            addr += len as u32;
            data = &data[len..];
        }
    }

    async fn bp_read8(&mut self, addr: u32) -> u8 {
        self.backplane_readn(addr, 1).await as u8
    }

    async fn bp_write8(&mut self, addr: u32, val: u8) {
        self.backplane_writen(addr, val as u32, 1).await
    }

    async fn bp_read16(&mut self, addr: u32) -> u16 {
        self.backplane_readn(addr, 2).await as u16
    }

    async fn bp_write16(&mut self, addr: u32, val: u16) {
        self.backplane_writen(addr, val as u32, 2).await
    }

    async fn bp_read32(&mut self, addr: u32) -> u32 {
        self.backplane_readn(addr, 4).await
    }

    async fn bp_write32(&mut self, addr: u32, val: u32) {
        self.backplane_writen(addr, val, 4).await
    }

    async fn backplane_readn(&mut self, addr: u32, len: u32) -> u32 {
        self.backplane_set_window(addr).await;

        let mut bus_addr = addr & BACKPLANE_ADDRESS_MASK;
        if len == 4 {
            bus_addr |= BACKPLANE_ADDRESS_32BIT_FLAG
        }
        self.readn(FUNC_BACKPLANE, bus_addr, len).await
    }

    async fn backplane_writen(&mut self, addr: u32, val: u32, len: u32) {
        self.backplane_set_window(addr).await;

        let mut bus_addr = addr & BACKPLANE_ADDRESS_MASK;
        if len == 4 {
            bus_addr |= BACKPLANE_ADDRESS_32BIT_FLAG
        }
        self.writen(FUNC_BACKPLANE, bus_addr, val, len).await
    }

    async fn backplane_set_window(&mut self, addr: u32) {
        let new_window = addr & !BACKPLANE_ADDRESS_MASK;

        if (new_window >> 24) as u8 != (self.backplane_window >> 24) as u8 {
            self.write8(
                FUNC_BACKPLANE,
                REG_BACKPLANE_BACKPLANE_ADDRESS_HIGH,
                (new_window >> 24) as u8,
            )
            .await;
        }
        if (new_window >> 16) as u8 != (self.backplane_window >> 16) as u8 {
            self.write8(
                FUNC_BACKPLANE,
                REG_BACKPLANE_BACKPLANE_ADDRESS_MID,
                (new_window >> 16) as u8,
            )
            .await;
        }
        if (new_window >> 8) as u8 != (self.backplane_window >> 8) as u8 {
            self.write8(
                FUNC_BACKPLANE,
                REG_BACKPLANE_BACKPLANE_ADDRESS_LOW,
                (new_window >> 8) as u8,
            )
            .await;
        }
        self.backplane_window = new_window;
    }

    async fn read8(&mut self, func: u32, addr: u32) -> u8 {
        self.readn(func, addr, 1).await as u8
    }

    async fn write8(&mut self, func: u32, addr: u32, val: u8) {
        self.writen(func, addr, val as u32, 1).await
    }

    async fn read16(&mut self, func: u32, addr: u32) -> u16 {
        self.readn(func, addr, 2).await as u16
    }

    async fn write16(&mut self, func: u32, addr: u32, val: u16) {
        self.writen(func, addr, val as u32, 2).await
    }

    async fn read32(&mut self, func: u32, addr: u32) -> u32 {
        self.readn(func, addr, 4).await
    }

    async fn write32(&mut self, func: u32, addr: u32, val: u32) {
        self.writen(func, addr, val, 4).await
    }

    async fn readn(&mut self, func: u32, addr: u32, len: u32) -> u32 {
        let cmd = cmd_word(READ, INC_ADDR, func, addr, len);
        let mut buf = [0; 1];

        self.spi
            .transaction(|bus| {
                let bus = unsafe { &mut *bus };
                async {
                    bus.write(&[cmd]).await?;
                    if func == FUNC_BACKPLANE {
                        // 4-byte response delay.
                        bus.read(&mut buf).await?;
                    }
                    bus.read(&mut buf).await?;
                    Ok(())
                }
            })
            .await
            .unwrap();

        buf[0]
    }

    async fn writen(&mut self, func: u32, addr: u32, val: u32, len: u32) {
        let cmd = cmd_word(WRITE, INC_ADDR, func, addr, len);

        self.spi
            .transaction(|bus| {
                let bus = unsafe { &mut *bus };
                async {
                    bus.write(&[cmd, val]).await?;
                    Ok(())
                }
            })
            .await
            .unwrap();
    }

    async fn read32_swapped(&mut self, addr: u32) -> u32 {
        let cmd = cmd_word(READ, INC_ADDR, FUNC_BUS, addr, 4);
        let mut buf = [0; 1];

        self.spi
            .transaction(|bus| {
                let bus = unsafe { &mut *bus };
                async {
                    bus.write(&[swap16(cmd)]).await?;
                    bus.read(&mut buf).await?;
                    Ok(())
                }
            })
            .await
            .unwrap();

        swap16(buf[0])
    }

    async fn write32_swapped(&mut self, addr: u32, val: u32) {
        let cmd = cmd_word(WRITE, INC_ADDR, FUNC_BUS, addr, 4);

        self.spi
            .transaction(|bus| {
                let bus = unsafe { &mut *bus };
                async {
                    bus.write(&[swap16(cmd), swap16(val)]).await?;
                    Ok(())
                }
            })
            .await
            .unwrap();
    }
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
