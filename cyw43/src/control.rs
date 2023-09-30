use core::cmp::{max, min};

use ch::driver::LinkState;
use embassy_net_driver_channel as ch;
use embassy_time::{Duration, Timer};

pub use crate::bus::SpiBusCyw43;
use crate::consts::*;
use crate::events::{Event, EventSubscriber, Events};
use crate::fmt::Bytes;
use crate::ioctl::{IoctlState, IoctlType};
use crate::structs::*;
use crate::{countries, events, PowerManagementMode};

#[derive(Debug)]
pub struct Error {
    pub status: u32,
}

pub struct Control<'a> {
    state_ch: ch::StateRunner<'a>,
    events: &'a Events,
    ioctl_state: &'a IoctlState,
}

impl<'a> Control<'a> {
    pub(crate) fn new(state_ch: ch::StateRunner<'a>, event_sub: &'a Events, ioctl_state: &'a IoctlState) -> Self {
        Self {
            state_ch,
            events: event_sub,
            ioctl_state,
        }
    }

    pub async fn init(&mut self, clm: &[u8]) {
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
            self.ioctl(IoctlType::Set, IOCTL_CMD_SET_VAR, 0, &mut buf[..8 + 12 + chunk.len()])
                .await;
        }

        // check clmload ok
        assert_eq!(self.get_iovar_u32("clmload_status").await, 0);

        debug!("Configuring misc stuff...");

        // Disable tx gloming which transfers multiple packets in one request.
        // 'glom' is short for "conglomerate" which means "gather together into
        // a compact mass".
        self.set_iovar_u32("bus:txglom", 0).await;
        self.set_iovar_u32("apsta", 1).await;

        // read MAC addr.
        let mut mac_addr = [0; 6];
        assert_eq!(self.get_iovar("cur_etheraddr", &mut mac_addr).await, 6);
        debug!("mac addr: {:02x}", Bytes(&mac_addr));

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
        evts.unset(Event::ROAM);

        self.set_iovar("bsscfg:event_msgs", &evts.to_bytes()).await;

        Timer::after(Duration::from_millis(100)).await;

        // set wifi up
        self.up().await;

        Timer::after(Duration::from_millis(100)).await;

        self.ioctl_set_u32(110, 0, 1).await; // SET_GMODE = auto
        self.ioctl_set_u32(142, 0, 0).await; // SET_BAND = any

        Timer::after(Duration::from_millis(100)).await;

        self.state_ch.set_ethernet_address(mac_addr);

        debug!("INIT DONE");
    }

    /// Set the WiFi interface up.
    async fn up(&mut self) {
        self.ioctl(IoctlType::Set, IOCTL_CMD_UP, 0, &mut []).await;
    }

    /// Set the interface down.
    async fn down(&mut self) {
        self.ioctl(IoctlType::Set, IOCTL_CMD_DOWN, 0, &mut []).await;
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

    pub async fn join_open(&mut self, ssid: &str) -> Result<(), Error> {
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

        self.wait_for_join(i).await
    }

    pub async fn join_wpa2(&mut self, ssid: &str, passphrase: &str) -> Result<(), Error> {
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

        self.wait_for_join(i).await
    }

    async fn wait_for_join(&mut self, i: SsidInfo) -> Result<(), Error> {
        self.events.mask.enable(&[Event::SET_SSID, Event::AUTH]);
        let mut subscriber = self.events.queue.subscriber().unwrap();
        // the actual join operation starts here
        // we make sure to enable events before so we don't miss any

        // set_ssid
        self.ioctl(IoctlType::Set, IOCTL_CMD_SET_SSID, 0, &mut i.to_bytes())
            .await;

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

    pub async fn gpio_set(&mut self, gpio_n: u8, gpio_en: bool) {
        assert!(gpio_n < 3);
        self.set_iovar_u32x2("gpioout", 1 << gpio_n, if gpio_en { 1 << gpio_n } else { 0 })
            .await
    }

    pub async fn start_ap_open(&mut self, ssid: &str, channel: u8) {
        self.start_ap(ssid, "", Security::OPEN, channel).await;
    }

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
        self.ioctl_set_u32(IOCTL_CMD_SET_AP, 0, 1).await;

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
        self.ioctl_set_u32(IOCTL_CMD_SET_CHANNEL, 0, channel as u32).await;

        // Set security
        self.set_iovar_u32x2("bsscfg:wsec", 0, (security as u32) & 0xFF).await;

        if security != Security::OPEN {
            self.set_iovar_u32x2("bsscfg:wpa_auth", 0, 0x0084).await; // wpa_auth = WPA2_AUTH_PSK | WPA_AUTH_PSK

            Timer::after(Duration::from_millis(100)).await;

            // Set passphrase
            let mut pfi = PassphraseInfo {
                len: passphrase.as_bytes().len() as _,
                flags: 1, // WSEC_PASSPHRASE
                passphrase: [0; 64],
            };
            pfi.passphrase[..passphrase.as_bytes().len()].copy_from_slice(passphrase.as_bytes());
            self.ioctl(IoctlType::Set, IOCTL_CMD_SET_PASSPHRASE, 0, &mut pfi.to_bytes())
                .await;
        }

        // Change mutlicast rate from 1 Mbps to 11 Mbps
        self.set_iovar_u32("2g_mrate", 11000000 / 500000).await;

        // Start AP
        self.set_iovar_u32x2("bss", 0, 1).await; // bss = BSS_UP
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
        self.set_iovar_v::<64>(name, val).await
    }

    async fn set_iovar_v<const BUFSIZE: usize>(&mut self, name: &str, val: &[u8]) {
        debug!("set {} = {:02x}", name, Bytes(val));

        let mut buf = [0; BUFSIZE];
        buf[..name.len()].copy_from_slice(name.as_bytes());
        buf[name.len()] = 0;
        buf[name.len() + 1..][..val.len()].copy_from_slice(val);

        let total_len = name.len() + 1 + val.len();
        self.ioctl(IoctlType::Set, IOCTL_CMD_SET_VAR, 0, &mut buf[..total_len])
            .await;
    }

    // TODO this is not really working, it always returns all zeros.
    async fn get_iovar(&mut self, name: &str, res: &mut [u8]) -> usize {
        debug!("get {}", name);

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
    pub async fn scan(&mut self) -> Scanner<'_> {
        const SCANTYPE_PASSIVE: u8 = 1;

        let scan_params = ScanParams {
            version: 1,
            action: 1,
            sync_id: 1,
            ssid_len: 0,
            ssid: [0; 32],
            bssid: [0xff; 6],
            bss_type: 2,
            scan_type: SCANTYPE_PASSIVE,
            nprobes: !0,
            active_time: !0,
            passive_time: !0,
            home_time: !0,
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
        self.ioctl(IoctlType::Set, IOCTL_CMD_DISASSOC, 0, &mut []).await;
        info!("Disassociated")
    }
}

pub struct Scanner<'a> {
    subscriber: EventSubscriber<'a>,
    events: &'a Events,
}

impl Scanner<'_> {
    /// wait for the next found network
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
