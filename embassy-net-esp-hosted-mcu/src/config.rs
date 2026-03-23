#![allow(missing_docs)]
// TODO: Add documentation

use heapless::Vec;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum BufferType {
    Static = 0,
    Dynamic = 1,
}

#[derive(Default, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub(crate) struct AdditionalFeatures {
    wpa3_sae: bool,
    cache_tx_buffer: bool,
    ftm_initiator: bool,
    ftm_responder: bool,
    gcmp: bool,
    gmac: bool,
    ieee802_11r: bool,
    enterprise: bool,
    bss_max_idle: bool,
}

impl From<AdditionalFeatures> for u64 {
    fn from(features: AdditionalFeatures) -> Self {
        let mut value = 0;
        if features.wpa3_sae {
            value |= 1 << 0;
        }
        if features.cache_tx_buffer {
            value |= 1 << 1;
        }
        if features.ftm_initiator {
            value |= 1 << 2;
        }
        if features.ftm_responder {
            value |= 1 << 3;
        }
        if features.gcmp {
            value |= 1 << 4;
        }
        if features.gmac {
            value |= 1 << 5;
        }
        if features.ieee802_11r {
            value |= 1 << 6;
        }
        if features.enterprise {
            value |= 1 << 7;
        }
        if features.bss_max_idle {
            value |= 1 << 8;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct EspConfig {
    pub static_rx_buf_num: u8,
    pub dynamic_rx_buf_num: u8,
    pub tx_buf_type: BufferType,
    pub static_tx_buf_num: u8,
    pub dynamic_tx_buf_num: u8,
    pub rx_mgmt_buf_type: BufferType,
    pub rx_mgmt_buf_num: u8,
}

impl From<EspConfig> for crate::proto::wifi_init_config {
    fn from(value: EspConfig) -> Self {
        let mut cfg = crate::proto::wifi_init_config::default();
        cfg.set_static_rx_buf_num(value.static_rx_buf_num as i32);
        cfg.set_dynamic_rx_buf_num(value.dynamic_rx_buf_num as i32);
        cfg.set_tx_buf_type(value.tx_buf_type as i32);
        cfg.set_static_tx_buf_num(value.static_tx_buf_num as i32);
        cfg.set_dynamic_tx_buf_num(value.dynamic_tx_buf_num as i32);
        cfg.set_rx_mgmt_buf_type(value.rx_mgmt_buf_type as i32);
        cfg.set_rx_mgmt_buf_num(value.rx_mgmt_buf_num as i32);
        cfg.set_cache_tx_buf_num(0);
        cfg.set_csi_enable(0);
        cfg.set_ampdu_rx_enable(1);
        cfg.set_ampdu_tx_enable(1);
        cfg.set_amsdu_tx_enable(0);
        cfg.set_nvs_enable(1);
        cfg.set_nano_enable(0);
        cfg.set_rx_ba_win(6);
        cfg.set_wifi_task_core_id(0);
        cfg.set_beacon_max_len(752);
        cfg.set_mgmt_sbuf_num(32);
        let feature_caps = AdditionalFeatures {
            wpa3_sae: true,
            gmac: true,
            bss_max_idle: true,
            ..Default::default()
        };
        cfg.set_feature_caps(feature_caps.into());
        cfg.set_sta_disconnected_pm(true);
        cfg.set_espnow_max_encrypt_num(7);
        cfg.set_tx_hetb_queue_num(3);
        cfg.set_dump_hesigb_enable(0);
        cfg.set_magic(0x1F2F3F4F);

        cfg
    }
}

/// WiFi authentication mode.
#[allow(unused)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Security {
    /// Authenticate mode : open
    WifiAuthOpen,
    /// Authenticate mode : WEP
    WifiAuthWep,
    /// Authenticate mode : WPA_PSK
    WifiAuthWpaPsk,
    /// Authenticate mode : WPA2_PSK
    WifiAuthWpa2Psk,
    /// Authenticate mode : WPA_WPA2_PSK
    WifiAuthWpaWpa2Psk,
    /// Authenticate mode : Wi-Fi EAP security, treated the same as WifiAuthWpa2Enterprise
    WifiAuthEnterprise,
    /// = WifiAuthEnterprise,  Authenticate mode : WPA2-Enterprise security
    WifiAuthWpa2Enterprise,
    /// Authenticate mode : WPA3_PSK
    WifiAuthWpa3Psk,
    /// Authenticate mode : WPA2_WPA3_PSK
    WifiAuthWpa2Wpa3Psk,
    /// Authenticate mode : WAPI_PSK
    WifiAuthWapiPsk,
    /// Authenticate mode : OWE
    WifiAuthOwe,
    /// Authenticate mode : WPA3_ENT_SUITE_B_192_BIT
    WifiAuthWpa3Ent192,
    /// This authentication mode will yield same result as WifiAuthWpa3Psk and not recommended to be used. It will be deprecated in future, please use WifiAuthWpa3Psk instead.
    WifiAuthWpa3ExtPsk,
    /// This authentication mode will yield same result as WifiAuthWpa3Psk and not recommended to be used. It will be deprecated in future, please use WifiAuthWpa3Psk instead.
    WifiAuthWpa3ExtPskMixedMode,
    /// Authenticate mode : DPP
    WifiAuthDpp,
    /// Authenticate mode : WPA3-Enterprise Only Mode
    WifiAuthWpa3Enterprise,
    /// Authenticate mode : WPA3-Enterprise Transition Mode
    WifiAuthWpa2Wpa3Enterprise,
    /// Authenticate mode : WPA-Enterprise security
    WifiAuthWpaEnterprise,
}

impl TryFrom<i32> for Security {
    type Error = crate::Error;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Security::WifiAuthOpen),
            1 => Ok(Security::WifiAuthWep),
            2 => Ok(Security::WifiAuthWpaPsk),
            3 => Ok(Security::WifiAuthWpa2Psk),
            4 => Ok(Security::WifiAuthWpaWpa2Psk),
            5 => Ok(Security::WifiAuthEnterprise), // this or WifiAuthWpa2Enterprise? According to the documentation, they are treated the same
            6 => Ok(Security::WifiAuthWpa3Psk),
            7 => Ok(Security::WifiAuthWpa2Wpa3Psk),
            8 => Ok(Security::WifiAuthWapiPsk),
            9 => Ok(Security::WifiAuthOwe),
            10 => Ok(Security::WifiAuthWpa3Ent192),
            11 => Ok(Security::WifiAuthWpa3ExtPsk),
            12 => Ok(Security::WifiAuthWpa3ExtPskMixedMode),
            13 => Ok(Security::WifiAuthDpp),
            14 => Ok(Security::WifiAuthWpa3Enterprise),
            15 => Ok(Security::WifiAuthWpa2Wpa3Enterprise),
            16 => Ok(Security::WifiAuthWpaEnterprise),
            _ => Err(crate::Error::Internal),
        }
    }
}

impl From<Security> for i32 {
    fn from(value: Security) -> Self {
        match value {
            Security::WifiAuthOpen => 0,
            Security::WifiAuthWep => 1,
            Security::WifiAuthWpaPsk => 2,
            Security::WifiAuthWpa2Psk => 3,
            Security::WifiAuthWpaWpa2Psk => 4,
            Security::WifiAuthEnterprise | Security::WifiAuthWpa2Enterprise => 5,
            Security::WifiAuthWpa3Psk => 6,
            Security::WifiAuthWpa2Wpa3Psk => 7,
            Security::WifiAuthWapiPsk => 8,
            Security::WifiAuthOwe => 9,
            Security::WifiAuthWpa3Ent192 => 10,
            Security::WifiAuthWpa3ExtPsk => 11,
            Security::WifiAuthWpa3ExtPskMixedMode => 12,
            Security::WifiAuthDpp => 13,
            Security::WifiAuthWpa3Enterprise => 14,
            Security::WifiAuthWpa2Wpa3Enterprise => 15,
            Security::WifiAuthWpaEnterprise => 16,
        }
    }
}

pub(crate) struct StaConfig {
    pub ssid: Vec<u8, 32>,
    pub password: Vec<u8, 32>,
    pub security: Security,
}

impl StaConfig {
    pub(crate) fn new(
        ssid: &str,
        password: &str,
        security: Security,
    ) -> Result<Self, crate::Error> {
        Ok(Self {
            ssid: Vec::from_slice(ssid.as_bytes()).map_err(|_| crate::Error::Internal)?,
            password: Vec::from_slice(password.as_bytes()).map_err(|_| crate::Error::Internal)?,
            security,
        })
    }
}

impl From<StaConfig> for crate::proto::wifi_sta_config {
    fn from(value: StaConfig) -> Self {
        let mut sta_config = crate::proto::wifi_sta_config::default();
        sta_config.set_ssid(value.ssid);
        sta_config.set_password(value.password);
        sta_config.set_bssid_set(false);
        sta_config.set_listen_interval(DEFAULT_LISTEN_INTERVAL);
        sta_config.set_channel(NO_CHANNEL_PREFERENCE);
        sta_config.set_failure_retry_cnt(DEFAULT_FAILURE_RETRY_CNT);
        let mut threshold = crate::proto::wifi_scan_threshold::default();
        threshold.set_authmode(value.security.into());
        sta_config.set_threshold(threshold);
        sta_config.set_sae_h2e_identifier(heapless::Vec::from_slice(&[0u8; 32]).unwrap());
        sta_config.set_sae_pwe_h2e(DEFAULT_SAE_PWE_H2E);

        sta_config
    }
}

const NO_CHANNEL_PREFERENCE: u32 = 0;
const DEFAULT_LISTEN_INTERVAL: u32 = 3;
const DEFAULT_FAILURE_RETRY_CNT: u32 = 0;
const DEFAULT_SAE_PWE_H2E: i32 = 3;
