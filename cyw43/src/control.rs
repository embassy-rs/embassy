use core::cmp::{max, min};
use core::iter::zip;

use embassy_net_driver_channel as ch;
use embassy_net_driver_channel::driver::{HardwareAddress, LinkState};
use embassy_time::{Duration, Timer};

use crate::consts::*;
use crate::events::{Event, EventSubscriber, Events};
use crate::fmt::Bytes;
use crate::ioctl::{IoctlState, IoctlType};
use crate::structs::*;
use crate::{countries, events, PowerManagementMode};

/// Control errors.
#[derive(Debug)]
pub struct Error {
    /// Status code.
    pub status: u32,
}

/// Multicast errors.
#[derive(Debug)]
pub enum AddMulticastAddressError {
    /// Not a multicast address.
    NotMulticast,
    /// No free address slots.
    NoFreeSlots,
}

/// Control driver.
pub struct Control<'a> {
    state_ch: ch::StateRunner<'a>,
    events: &'a Events,
    ioctl_state: &'a IoctlState,
}

/// WiFi scan type.
#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ScanType {
    /// Active scan: the station actively transmits probes that make APs respond.
    /// Faster, but uses more power.
    Active,
    /// Passive scan: the station doesn't transmit any probes, just listens for beacons.
    /// Slower, but uses less power.
    Passive,
}

/// Scan options.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub struct ScanOptions {
    /// SSID to scan for.
    pub ssid: Option<heapless::String<32>>,
    /// If set to `None`, all APs will be returned. If set to `Some`, only APs
    /// with the specified BSSID will be returned.
    pub bssid: Option<[u8; 6]>,
    /// Number of probes to send on each channel.
    pub nprobes: Option<u16>,
    /// Time to spend waiting on the home channel.
    pub home_time: Option<Duration>,
    /// Scan type: active or passive.
    pub scan_type: ScanType,
    /// Period of time to wait on each channel when passive scanning.
    pub dwell_time: Option<Duration>,
}

impl Default for ScanOptions {
    fn default() -> Self {
        Self {
            ssid: None,
            bssid: None,
            nprobes: None,
            home_time: None,
            scan_type: ScanType::Passive,
            dwell_time: None,
        }
    }
}

/// Authentication type, used in [`JoinOptions::auth`].
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum JoinAuth {
    /// Open network
    Open,
    /// WPA only
    Wpa,
    /// WPA2 only
    Wpa2,
    /// WPA3 only
    Wpa3,
    /// WPA2 + WPA3
    Wpa2Wpa3,
}

/// Options for [`Control::join`].
#[derive(Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub struct JoinOptions<'a> {
    /// Authentication type. Default `Wpa2Wpa3`.
    pub auth: JoinAuth,
    /// Enable TKIP encryption. Default false.
    pub cipher_tkip: bool,
    /// Enable AES encryption. Default true.
    pub cipher_aes: bool,
    /// Passphrase. Default empty.
    pub passphrase: &'a [u8],
    /// If false, `passphrase` is the human-readable passphrase string.
    /// If true, `passphrase` is the result of applying the PBKDF2 hash to the
    /// passphrase string. This makes it possible to avoid storing unhashed passwords.
    ///
    /// This is not compatible with WPA3.
    /// Default false.
    pub passphrase_is_prehashed: bool,
}

impl<'a> JoinOptions<'a> {
    /// Create a new `JoinOptions` for joining open networks.
    pub fn new_open() -> Self {
        Self {
            auth: JoinAuth::Open,
            cipher_tkip: false,
            cipher_aes: false,
            passphrase: &[],
            passphrase_is_prehashed: false,
        }
    }

    /// Create a new `JoinOptions` for joining encrypted networks.
    ///
    /// Defaults to supporting WPA2+WPA3 with AES only, you may edit
    /// the returned options to change this.
    pub fn new(passphrase: &'a [u8]) -> Self {
        let mut this = Self::default();
        this.passphrase = passphrase;
        this
    }
}

impl<'a> Default for JoinOptions<'a> {
    fn default() -> Self {
        Self {
            auth: JoinAuth::Wpa2Wpa3,
            cipher_tkip: false,
            cipher_aes: true,
            passphrase: &[],
            passphrase_is_prehashed: false,
        }
    }
}

impl<'a> Control<'a> {
    pub(crate) fn new(state_ch: ch::StateRunner<'a>, event_sub: &'a Events, ioctl_state: &'a IoctlState) -> Self {
        Self {
            state_ch,
            events: event_sub,
            ioctl_state,
        }
    }

    async fn load_clm(&mut self, clm: &[u8]) {
        const CHUNK_SIZE: usize = 1024;

        debug!("Downloading CLM...");

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
            self.ioctl(IoctlType::Set, Ioctl::SetVar, 0, &mut buf[..8 + 12 + chunk.len()])
                .await;
        }

        // check clmload ok
        assert_eq!(self.get_iovar_u32("clmload_status").await, 0);
    }

    /// Initialize WiFi controller.
    pub async fn init(&mut self, clm: &[u8]) {
        self.load_clm(&clm).await;

        debug!("Configuring misc stuff...");

        // Disable tx gloming which transfers multiple packets in one request.
        // 'glom' is short for "conglomerate" which means "gather together into
        // a compact mass".
        self.set_iovar_u32("bus:txglom", 0).await;
        self.set_iovar_u32("apsta", 1).await;

        // read MAC addr.
        let mac_addr = self.address().await;
        debug!("mac addr: {:02x}", Bytes(&mac_addr));

        let country = countries::WORLD_WIDE_XX;
        let country_info = CountryInfo {
            country_abbrev: [country.code[0], country.code[1], 0, 0],
            country_code: [country.code[0], country.code[1], 0, 0],
            rev: if country.rev == 0 { -1 } else { country.rev as _ },
        };
        self.set_iovar("country", &country_info.to_bytes()).await;

        // set country takes some time, next ioctls fail if we don't wait.
        Timer::after_millis(100).await;

        // Set antenna to chip antenna
        self.ioctl_set_u32(Ioctl::SetAntdiv, 0, 0).await;

        self.set_iovar_u32("bus:txglom", 0).await;
        Timer::after_millis(100).await;
        //self.set_iovar_u32("apsta", 1).await; // this crashes, also we already did it before...??
        //Timer::after_millis(100).await;
        self.set_iovar_u32("ampdu_ba_wsize", 8).await;
        Timer::after_millis(100).await;
        self.set_iovar_u32("ampdu_mpdu", 4).await;
        Timer::after_millis(100).await;
        //self.set_iovar_u32("ampdu_rx_factor", 0).await; // this crashes

        //Timer::after_millis(100).await;

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
        evts.unset(Event::ROAM);

        self.set_iovar("bsscfg:event_msgs", &evts.to_bytes()).await;

        Timer::after_millis(100).await;

        // set wifi up
        self.up().await;

        Timer::after_millis(100).await;

        self.ioctl_set_u32(Ioctl::SetGmode, 0, 1).await; // SET_GMODE = auto
        self.ioctl_set_u32(Ioctl::SetBand, 0, 0).await; // SET_BAND = any

        Timer::after_millis(100).await;

        self.state_ch.set_hardware_address(HardwareAddress::Ethernet(mac_addr));

        debug!("cyw43 control init done");
    }

    /// Set the WiFi interface up.
    async fn up(&mut self) {
        self.ioctl(IoctlType::Set, Ioctl::Up, 0, &mut []).await;
    }

    /// Set the interface down.
    async fn down(&mut self) {
        self.ioctl(IoctlType::Set, Ioctl::Down, 0, &mut []).await;
    }

    /// Set power management mode.
    pub async fn set_power_management(&mut self, mode: PowerManagementMode) {
        // power save mode
        let mode_num = mode.mode();
        if mode_num == 2 {
            self.set_iovar_u32("pm2_sleep_ret", mode.sleep_ret_ms() as u32).await;
            self.set_iovar_u32("bcn_li_bcn", mode.beacon_period() as u32).await;
            self.set_iovar_u32("bcn_li_dtim", mode.dtim_period() as u32).await;
            self.set_iovar_u32("assoc_listen", mode.assoc() as u32).await;
        }
        self.ioctl_set_u32(Ioctl::SetPm, 0, mode_num).await;
    }

    /// Join an unprotected network with the provided ssid.
    pub async fn join(&mut self, ssid: &str, options: JoinOptions<'_>) -> Result<(), Error> {
        self.set_iovar_u32("ampdu_ba_wsize", 8).await;

        if options.auth == JoinAuth::Open {
            self.ioctl_set_u32(Ioctl::SetWsec, 0, 0).await;
            self.set_iovar_u32x2("bsscfg:sup_wpa", 0, 0).await;
            self.ioctl_set_u32(Ioctl::SetInfra, 0, 1).await;
            self.ioctl_set_u32(Ioctl::SetAuth, 0, 0).await;
            self.ioctl_set_u32(Ioctl::SetWpaAuth, 0, WPA_AUTH_DISABLED).await;
        } else {
            let mut wsec = 0;
            if options.cipher_aes {
                wsec |= WSEC_AES;
            }
            if options.cipher_tkip {
                wsec |= WSEC_TKIP;
            }
            self.ioctl_set_u32(Ioctl::SetWsec, 0, wsec).await;

            self.set_iovar_u32x2("bsscfg:sup_wpa", 0, 1).await;
            self.set_iovar_u32x2("bsscfg:sup_wpa2_eapver", 0, 0xFFFF_FFFF).await;
            self.set_iovar_u32x2("bsscfg:sup_wpa_tmo", 0, 2500).await;

            Timer::after_millis(100).await;

            let (wpa12, wpa3, auth, mfp, wpa_auth) = match options.auth {
                JoinAuth::Open => unreachable!(),
                JoinAuth::Wpa => (true, false, AUTH_OPEN, MFP_NONE, WPA_AUTH_WPA_PSK),
                JoinAuth::Wpa2 => (true, false, AUTH_OPEN, MFP_CAPABLE, WPA_AUTH_WPA2_PSK),
                JoinAuth::Wpa3 => (false, true, AUTH_SAE, MFP_REQUIRED, WPA_AUTH_WPA3_SAE_PSK),
                JoinAuth::Wpa2Wpa3 => (true, true, AUTH_SAE, MFP_CAPABLE, WPA_AUTH_WPA3_SAE_PSK),
            };

            if wpa12 {
                let mut flags = 0;
                if !options.passphrase_is_prehashed {
                    flags |= 1;
                }
                let mut pfi = PassphraseInfo {
                    len: options.passphrase.len() as _,
                    flags,
                    passphrase: [0; 64],
                };
                pfi.passphrase[..options.passphrase.len()].copy_from_slice(options.passphrase);
                Timer::after_millis(3).await;
                self.ioctl(IoctlType::Set, Ioctl::SetWsecPmk, 0, &mut pfi.to_bytes())
                    .await;
            }

            if wpa3 {
                let mut pfi = SaePassphraseInfo {
                    len: options.passphrase.len() as _,
                    passphrase: [0; 128],
                };
                pfi.passphrase[..options.passphrase.len()].copy_from_slice(options.passphrase);
                Timer::after_millis(3).await;
                self.set_iovar("sae_password", &pfi.to_bytes()).await;
            }

            self.ioctl_set_u32(Ioctl::SetInfra, 0, 1).await;
            self.ioctl_set_u32(Ioctl::SetAuth, 0, auth).await;
            self.set_iovar_u32("mfp", mfp).await;
            self.ioctl_set_u32(Ioctl::SetWpaAuth, 0, wpa_auth).await;
        }

        let mut i = SsidInfo {
            len: ssid.len() as _,
            ssid: [0; 32],
        };
        i.ssid[..ssid.len()].copy_from_slice(ssid.as_bytes());

        self.wait_for_join(i).await
    }

    async fn wait_for_join(&mut self, i: SsidInfo) -> Result<(), Error> {
        self.events.mask.enable(&[Event::SET_SSID, Event::AUTH]);
        let mut subscriber = self.events.queue.subscriber().unwrap();
        // the actual join operation starts here
        // we make sure to enable events before so we don't miss any

        self.ioctl(IoctlType::Set, Ioctl::SetSsid, 0, &mut i.to_bytes()).await;

        // to complete the join, we wait for a SET_SSID event
        // we also save the AUTH status for the user, it may be interesting
        let mut auth_status = 0;
        let status = loop {
            let msg = subscriber.next_message_pure().await;
            if msg.header.event_type == Event::AUTH && msg.header.status != EStatus::SUCCESS {
                auth_status = msg.header.status;
            } else if msg.header.event_type == Event::SET_SSID {
                // join operation ends with SET_SSID event
                break msg.header.status;
            }
        };

        self.events.mask.disable_all();
        if status == EStatus::SUCCESS {
            // successful join
            self.state_ch.set_link_state(LinkState::Up);
            debug!("JOINED");
            Ok(())
        } else {
            warn!("JOIN failed with status={} auth={}", status, auth_status);
            Err(Error { status })
        }
    }

    /// Set GPIO pin on WiFi chip.
    pub async fn gpio_set(&mut self, gpio_n: u8, gpio_en: bool) {
        assert!(gpio_n < 3);
        self.set_iovar_u32x2("gpioout", 1 << gpio_n, if gpio_en { 1 << gpio_n } else { 0 })
            .await
    }

    /// Start open access point.
    pub async fn start_ap_open(&mut self, ssid: &str, channel: u8) {
        self.start_ap(ssid, "", Security::OPEN, channel).await;
    }

    /// Start WPA2 protected access point.
    pub async fn start_ap_wpa2(&mut self, ssid: &str, passphrase: &str, channel: u8) {
        self.start_ap(ssid, passphrase, Security::WPA2_AES_PSK, channel).await;
    }

    async fn start_ap(&mut self, ssid: &str, passphrase: &str, security: Security, channel: u8) {
        if security != Security::OPEN
            && (passphrase.as_bytes().len() < MIN_PSK_LEN || passphrase.as_bytes().len() > MAX_PSK_LEN)
        {
            panic!("Passphrase is too short or too long");
        }

        // Temporarily set wifi down
        self.down().await;

        // Turn off APSTA mode
        self.set_iovar_u32("apsta", 0).await;

        // Set wifi up again
        self.up().await;

        // Turn on AP mode
        self.ioctl_set_u32(Ioctl::SetAp, 0, 1).await;

        // Set SSID
        let mut i = SsidInfoWithIndex {
            index: 0,
            ssid_info: SsidInfo {
                len: ssid.as_bytes().len() as _,
                ssid: [0; 32],
            },
        };
        i.ssid_info.ssid[..ssid.as_bytes().len()].copy_from_slice(ssid.as_bytes());
        self.set_iovar("bsscfg:ssid", &i.to_bytes()).await;

        // Set channel number
        self.ioctl_set_u32(Ioctl::SetChannel, 0, channel as u32).await;

        // Set security
        self.set_iovar_u32x2("bsscfg:wsec", 0, (security as u32) & 0xFF).await;

        if security != Security::OPEN {
            self.set_iovar_u32x2("bsscfg:wpa_auth", 0, 0x0084).await; // wpa_auth = WPA2_AUTH_PSK | WPA_AUTH_PSK

            Timer::after_millis(100).await;

            // Set passphrase
            let mut pfi = PassphraseInfo {
                len: passphrase.as_bytes().len() as _,
                flags: 1, // WSEC_PASSPHRASE
                passphrase: [0; 64],
            };
            pfi.passphrase[..passphrase.as_bytes().len()].copy_from_slice(passphrase.as_bytes());
            self.ioctl(IoctlType::Set, Ioctl::SetWsecPmk, 0, &mut pfi.to_bytes())
                .await;
        }

        // Change mutlicast rate from 1 Mbps to 11 Mbps
        self.set_iovar_u32("2g_mrate", 11000000 / 500000).await;

        // Start AP
        self.set_iovar_u32x2("bss", 0, 1).await; // bss = BSS_UP
    }

    /// Closes access point.
    pub async fn close_ap(&mut self) {
        // Stop AP
        self.set_iovar_u32x2("bss", 0, 0).await; // bss = BSS_DOWN

        // Turn off AP mode
        self.ioctl_set_u32(Ioctl::SetAp, 0, 0).await;

        // Temporarily set wifi down
        self.down().await;

        // Turn on APSTA mode
        self.set_iovar_u32("apsta", 1).await;

        // Set wifi up again
        self.up().await;
    }

    /// Add specified address to the list of hardware addresses the device
    /// listens on. The address must be a Group address (I/G bit set). Up
    /// to 10 addresses are supported by the firmware. Returns the number of
    /// address slots filled after adding, or an error.
    pub async fn add_multicast_address(&mut self, address: [u8; 6]) -> Result<usize, AddMulticastAddressError> {
        // The firmware seems to ignore non-multicast addresses, so let's
        // prevent the user from adding them and wasting space.
        if address[0] & 0x01 != 1 {
            return Err(AddMulticastAddressError::NotMulticast);
        }

        let mut buf = [0; 64];
        self.get_iovar("mcast_list", &mut buf).await;

        let n = u32::from_le_bytes(buf[..4].try_into().unwrap()) as usize;
        let (used, free) = buf[4..].split_at_mut(n * 6);

        if used.chunks(6).any(|a| a == address) {
            return Ok(n);
        }

        if free.len() < 6 {
            return Err(AddMulticastAddressError::NoFreeSlots);
        }

        free[..6].copy_from_slice(&address);
        let n = n + 1;
        buf[..4].copy_from_slice(&(n as u32).to_le_bytes());

        self.set_iovar_v::<80>("mcast_list", &buf).await;
        Ok(n)
    }

    /// Retrieve the list of configured multicast hardware addresses.
    pub async fn list_multicast_addresses(&mut self, result: &mut [[u8; 6]; 10]) -> usize {
        let mut buf = [0; 64];
        self.get_iovar("mcast_list", &mut buf).await;

        let n = u32::from_le_bytes(buf[..4].try_into().unwrap()) as usize;
        let used = &buf[4..][..n * 6];

        for (addr, output) in zip(used.chunks(6), result.iter_mut()) {
            output.copy_from_slice(addr)
        }

        n
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
        self.set_iovar_v::<196>(name, val).await
    }

    async fn set_iovar_v<const BUFSIZE: usize>(&mut self, name: &str, val: &[u8]) {
        debug!("iovar set {} = {:02x}", name, Bytes(val));

        let mut buf = [0; BUFSIZE];
        buf[..name.len()].copy_from_slice(name.as_bytes());
        buf[name.len()] = 0;
        buf[name.len() + 1..][..val.len()].copy_from_slice(val);

        let total_len = name.len() + 1 + val.len();
        self.ioctl_inner(IoctlType::Set, Ioctl::SetVar, 0, &mut buf[..total_len])
            .await;
    }

    // TODO this is not really working, it always returns all zeros.
    async fn get_iovar(&mut self, name: &str, res: &mut [u8]) -> usize {
        debug!("iovar get {}", name);

        let mut buf = [0; 64];
        buf[..name.len()].copy_from_slice(name.as_bytes());
        buf[name.len()] = 0;

        let total_len = max(name.len() + 1, res.len());
        let res_len = self
            .ioctl_inner(IoctlType::Get, Ioctl::GetVar, 0, &mut buf[..total_len])
            .await;

        let out_len = min(res.len(), res_len);
        res[..out_len].copy_from_slice(&buf[..out_len]);
        out_len
    }

    async fn ioctl_set_u32(&mut self, cmd: Ioctl, iface: u32, val: u32) {
        let mut buf = val.to_le_bytes();
        self.ioctl(IoctlType::Set, cmd, iface, &mut buf).await;
    }

    async fn ioctl(&mut self, kind: IoctlType, cmd: Ioctl, iface: u32, buf: &mut [u8]) -> usize {
        if kind == IoctlType::Set {
            debug!("ioctl set {:?} iface {} = {:02x}", cmd, iface, Bytes(buf));
        }
        let n = self.ioctl_inner(kind, cmd, iface, buf).await;
        n
    }

    async fn ioctl_inner(&mut self, kind: IoctlType, cmd: Ioctl, iface: u32, buf: &mut [u8]) -> usize {
        struct CancelOnDrop<'a>(&'a IoctlState);

        impl CancelOnDrop<'_> {
            fn defuse(self) {
                core::mem::forget(self);
            }
        }

        impl Drop for CancelOnDrop<'_> {
            fn drop(&mut self) {
                self.0.cancel_ioctl();
            }
        }

        let ioctl = CancelOnDrop(self.ioctl_state);
        let resp_len = ioctl.0.do_ioctl(kind, cmd, iface, buf).await;
        ioctl.defuse();

        resp_len
    }

    /// Start a wifi scan
    ///
    /// Returns a `Stream` of networks found by the device
    ///
    /// # Note
    /// Device events are currently implemented using a bounded queue.
    /// To not miss any events, you should make sure to always await the stream.
    pub async fn scan(&mut self, scan_opts: ScanOptions) -> Scanner<'_> {
        const SCANTYPE_ACTIVE: u8 = 0;
        const SCANTYPE_PASSIVE: u8 = 1;

        let dwell_time = match scan_opts.dwell_time {
            None => !0,
            Some(t) => {
                let mut t = t.as_millis() as u32;
                if t == !0 {
                    t = !0 - 1;
                }
                t
            }
        };

        let mut active_time = !0;
        let mut passive_time = !0;
        let scan_type = match scan_opts.scan_type {
            ScanType::Active => {
                active_time = dwell_time;
                SCANTYPE_ACTIVE
            }
            ScanType::Passive => {
                passive_time = dwell_time;
                SCANTYPE_PASSIVE
            }
        };

        let scan_params = ScanParams {
            version: 1,
            action: 1,
            sync_id: 1,
            ssid_len: scan_opts.ssid.as_ref().map(|e| e.as_bytes().len() as u32).unwrap_or(0),
            ssid: scan_opts
                .ssid
                .map(|e| {
                    let mut ssid = [0; 32];
                    ssid[..e.as_bytes().len()].copy_from_slice(e.as_bytes());
                    ssid
                })
                .unwrap_or([0; 32]),
            bssid: scan_opts.bssid.unwrap_or([0xff; 6]),
            bss_type: 2,
            scan_type,
            nprobes: scan_opts.nprobes.unwrap_or(!0).into(),
            active_time,
            passive_time,
            home_time: scan_opts.home_time.map(|e| e.as_millis() as u32).unwrap_or(!0),
            channel_num: 0,
            channel_list: [0; 1],
        };

        self.events.mask.enable(&[Event::ESCAN_RESULT]);
        let subscriber = self.events.queue.subscriber().unwrap();
        self.set_iovar_v::<256>("escan", &scan_params.to_bytes()).await;

        Scanner {
            subscriber,
            events: &self.events,
        }
    }
    /// Leave the wifi, with which we are currently associated.
    pub async fn leave(&mut self) {
        self.ioctl(IoctlType::Set, Ioctl::Disassoc, 0, &mut []).await;
        info!("Disassociated")
    }

    /// Gets the MAC address of the device
    pub async fn address(&mut self) -> [u8; 6] {
        let mut mac_addr = [0; 6];
        assert_eq!(self.get_iovar("cur_etheraddr", &mut mac_addr).await, 6);
        mac_addr
    }
}

/// WiFi network scanner.
pub struct Scanner<'a> {
    subscriber: EventSubscriber<'a>,
    events: &'a Events,
}

impl Scanner<'_> {
    /// Wait for the next found network.
    pub async fn next(&mut self) -> Option<BssInfo> {
        let event = self.subscriber.next_message_pure().await;
        if event.header.status != EStatus::PARTIAL {
            self.events.mask.disable_all();
            return None;
        }

        if let events::Payload::BssInfo(bss) = event.payload {
            Some(bss)
        } else {
            None
        }
    }
}

impl Drop for Scanner<'_> {
    fn drop(&mut self) {
        self.events.mask.disable_all();
    }
}
