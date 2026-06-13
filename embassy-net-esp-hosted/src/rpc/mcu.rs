use heapless::{String, Vec};
use micropb::MessageDecode;

use super::{HostedEvent, IoctlCtx, RpcBackend, check_resp};
use crate::control::{Error, Security, Status};
use crate::proto::mcu::Rpc_::Payload;
use crate::proto::mcu::{
    Rpc, Rpc_Req_ConfigHeartbeat, Rpc_Req_GetCoprocessorFwVersion, Rpc_Req_GetMacAddress, Rpc_Req_OTAActivate,
    Rpc_Req_OTABegin, Rpc_Req_OTAEnd, Rpc_Req_OTAWrite, Rpc_Req_SetMode, Rpc_Req_WifiConnect, Rpc_Req_WifiDisconnect,
    Rpc_Req_WifiInit, Rpc_Req_WifiSetConfig, Rpc_Req_WifiStaGetApInfo, Rpc_Req_WifiStart, RpcId, RpcType, wifi_config,
    wifi_config_, wifi_init_config, wifi_scan_threshold, wifi_sta_config,
};
#[cfg(feature = "bluetooth")]
use crate::proto::mcu::{Rpc_Req_FeatureControl, RpcFeature, RpcFeatureCommand, RpcFeatureOption};
use crate::{FwVersion, InterfaceType, WifiMode};

macro_rules! exchange {
    ($ctx:ident, $req_variant:ident, $resp_variant:ident, $req:expr) => {{
        let mut msg = Rpc {
            msg_id: RpcId::$req_variant,
            msg_type: RpcType::Req,
            uid: 0,
            payload: Some(Payload::$req_variant($req)),
        };

        debug!("ioctl req: {:?}", msg);
        $ctx.exchange(&mut msg).await?;
        debug!("ioctl resp: {:?}", msg);

        let Some(Payload::$resp_variant(resp)) = msg.payload else {
            return Err(Error::Internal);
        };
        check_resp(resp.resp)?;

        resp
    }};
}

#[allow(unused)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
enum WifiInterface {
    Sta = 0,
    Ap = 1,
}

/// FG (`esp_hosted_config.proto`) RPC backend.
#[derive(Clone, Copy, Debug, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct McuBackend;

impl RpcBackend for McuBackend {
    #[inline]
    fn encode_ioctl(&self, buffer: &mut [u8], req: &[u8]) -> usize {
        let req_len = req.len();

        buffer[0..10].copy_from_slice(b"\x01\x06\x00RPCRsp\x02");
        buffer[10..12].copy_from_slice(&(req_len as u16).to_le_bytes());
        buffer[12..][..req_len].copy_from_slice(req);

        req_len + 12
    }

    #[inline]
    fn process_serial_data<'pl>(&mut self, payload: &'pl [u8]) -> Option<(bool, &'pl [u8])> {
        if payload.len() < 12 {
            warn!("serial rx: too short");
            return None;
        }

        let is_event = match &payload[..10] {
            b"\x01\x06\x00RPCRsp\x02" => false,
            b"\x01\x06\x00RPCEvt\x02" => true,
            _ => {
                warn!("serial rx: bad tlv");
                return None;
            }
        };

        let len = u16::from_le_bytes(payload[10..12].try_into().unwrap()) as usize;
        if payload.len() < 12 + len {
            warn!("serial rx: too short 2");
            return None;
        }

        Some((is_event, &payload[12..][..len]))
    }

    fn encode_iface_type(&self, iface_type: InterfaceType) -> Option<u8> {
        match iface_type {
            InterfaceType::Invalid => Some(0),
            InterfaceType::Sta => Some(1),
            InterfaceType::Ap => Some(2),
            InterfaceType::Serial => Some(3),
            InterfaceType::Hci => Some(4),
            InterfaceType::Priv => Some(5),
            InterfaceType::Test => Some(6),
        }
    }

    fn decode_iface_type(&self, iface_type: u8) -> Option<InterfaceType> {
        match iface_type {
            1 => Some(InterfaceType::Sta),
            3 => Some(InterfaceType::Serial),
            4 => Some(InterfaceType::Hci),
            _ => None,
        }
    }

    async fn init_radio(&self, ctx: &mut IoctlCtx<'_>) -> Result<(), Error> {
        let mut req = Rpc_Req_WifiInit::default();
        req.set_cfg(
            // TODO: this should be user-configurable, either in build-time or runtime
            EspConfig {
                static_rx_buf_num: 10,
                dynamic_rx_buf_num: 32,
                tx_buf_type: BufferType::Dynamic,
                static_tx_buf_num: 10,
                dynamic_tx_buf_num: 32,
                rx_mgmt_buf_type: BufferType::Dynamic,
                rx_mgmt_buf_num: 32,
            }
            .into(),
        );

        exchange!(ctx, ReqWifiInit, RespWifiInit, req);

        #[cfg(feature = "bluetooth")]
        {
            // Since esp-hosted-mcu v2.5.2 the BT controller is disabled by default and must be
            // explicitly initialized and enabled via the FeatureControl RPC before the BT host
            // stack can talk to it over the HCI interface.
            match self.get_fw_version(ctx).await? {
                #[cfg(feature = "esp-hosted-fg")]
                FwVersion::Fg { .. } => return Err(Error::Internal),

                FwVersion::Mcu { major, minor, patch } => {
                    if major < 2 || (major == 2 && minor < 5) || (major == 2 && minor == 5 && patch < 2) {
                        return Ok(());
                    }
                }
            }

            debug!("Enabling Bluetooth though FeatureControl");

            let req = Rpc_Req_FeatureControl {
                feature: RpcFeature::FeatureBluetooth,
                command: RpcFeatureCommand::FeatureCommandBtInit,
                option: RpcFeatureOption::FeatureOptionNone,
            };
            exchange!(ctx, ReqFeatureControl, RespFeatureControl, req);

            let req = Rpc_Req_FeatureControl {
                feature: RpcFeature::FeatureBluetooth,
                command: RpcFeatureCommand::FeatureCommandBtEnable,
                option: RpcFeatureOption::FeatureOptionNone,
            };
            exchange!(ctx, ReqFeatureControl, RespFeatureControl, req);
        }

        Ok(())
    }

    async fn config_heartbeat(&self, ctx: &mut IoctlCtx<'_>, secs: u32) -> Result<(), Error> {
        let req = Rpc_Req_ConfigHeartbeat {
            enable: true,
            duration: secs as i32,
        };
        exchange!(ctx, ReqConfigHeartbeat, RespConfigHeartbeat, req);
        Ok(())
    }

    async fn start_wifi(&self, ctx: &mut IoctlCtx<'_>) -> Result<(), Error> {
        let req = Rpc_Req_WifiStart::default();
        exchange!(ctx, ReqWifiStart, RespWifiStart, req);
        Ok(())
    }

    async fn set_mode(&self, ctx: &mut IoctlCtx<'_>, mode: WifiMode) -> Result<(), Error> {
        let req = Rpc_Req_SetMode { mode: mode as i32 };
        exchange!(ctx, ReqSetWifiMode, RespSetWifiMode, req);
        Ok(())
    }

    async fn get_mac_addr(&self, ctx: &mut IoctlCtx<'_>) -> Result<[u8; 6], Error> {
        let req = Rpc_Req_GetMacAddress {
            mode: WifiInterface::Sta as i32,
        };
        let resp = exchange!(ctx, ReqGetMacAddress, RespGetMacAddress, req);
        Ok(resp.mac[..6].try_into().unwrap())
    }

    async fn connect_ap(&self, ctx: &mut IoctlCtx<'_>, ssid: &str, pwd: &str) -> Result<(), Error> {
        let mut req = Rpc_Req_WifiSetConfig::default();
        req.set_iface(WifiInterface::Sta as _);
        let sta_config = StaConfig::new(ssid, pwd, Security::Wpa2Psk)?;
        let wifi_config = wifi_config {
            u: Some(wifi_config_::U::Sta(sta_config.into())),
        };
        req.set_cfg(wifi_config);
        exchange!(ctx, ReqWifiSetConfig, RespWifiSetConfig, req);

        self.start_wifi(ctx).await?;

        let req = Rpc_Req_WifiConnect::default();
        exchange!(ctx, ReqWifiConnect, RespWifiConnect, req);

        Ok(())
    }

    async fn disconnect_ap(&self, ctx: &mut IoctlCtx<'_>) -> Result<(), Error> {
        let req = Rpc_Req_WifiDisconnect::default();
        exchange!(ctx, ReqWifiDisconnect, RespWifiDisconnect, req);
        Ok(())
    }

    async fn get_status(&self, ctx: &mut IoctlCtx<'_>) -> Result<Status, Error> {
        let req = Rpc_Req_WifiStaGetApInfo::default();
        let resp = exchange!(ctx, ReqWifiStaGetApInfo, RespWifiStaGetApInfo, req);
        let ap = &resp.ap_record;
        let ssid = core::str::from_utf8(&ap.ssid).map_err(|_| Error::Internal)?;
        let ssid = String::try_from(ssid.trim_end_matches('\0')).map_err(|_| Error::Internal)?;
        let mut bssid = [0u8; 6];
        bssid.copy_from_slice(&ap.bssid[0..6]);
        Ok(Status {
            ssid,
            bssid,
            rssi: ap.rssi as _,
            channel: ap.primary,
            security: map_mcu_security(ap.authmode),
        })
    }

    async fn get_fw_version(&self, ctx: &mut IoctlCtx<'_>) -> Result<FwVersion, Error> {
        let req = Rpc_Req_GetCoprocessorFwVersion::default();

        let mut msg = Rpc {
            msg_id: RpcId::ReqGetCoprocessorFwVersion,
            msg_type: RpcType::Req,
            uid: 0,
            payload: Some(Payload::ReqGetCoprocessorFwversion(req)),
        };

        debug!("ioctl req: {:?}", msg);
        ctx.exchange(&mut msg).await?;
        debug!("ioctl resp: {:?}", msg);

        let Some(Payload::RespGetCoprocessorFwversion(resp)) = msg.payload else {
            return Err(Error::Internal);
        };
        check_resp(resp.resp)?;

        debug!("raw: {:?}", resp);
        Ok(FwVersion::Mcu {
            major: resp.major1,
            minor: resp.minor1,
            patch: resp.patch1,
        })
    }

    async fn ota_begin(&self, ctx: &mut IoctlCtx<'_>) -> Result<(), Error> {
        let req = Rpc_Req_OTABegin::default();

        exchange!(ctx, ReqOtaBegin, RespOtaBegin, req);
        Ok(())
    }

    async fn ota_write(&self, ctx: &mut IoctlCtx<'_>, chunk: &[u8]) -> Result<(), Error> {
        let req = Rpc_Req_OTAWrite {
            ota_data: heapless::Vec::from_slice(chunk).unwrap(),
        };
        exchange!(ctx, ReqOtaWrite, RespOtaWrite, req);
        Ok(())
    }

    async fn ota_end(&self, ctx: &mut IoctlCtx<'_>) -> Result<(), Error> {
        let req = Rpc_Req_OTAEnd {};
        exchange!(ctx, ReqOtaEnd, RespOtaEnd, req);

        let req = Rpc_Req_OTAActivate {};
        exchange!(ctx, ReqOtaActivate, RespOtaActivate, req);
        Ok(())
    }

    #[inline]
    fn normalize_event(&self, raw: &[u8]) -> Option<HostedEvent> {
        let mut event = Rpc::default();
        if event.decode_from_bytes(raw).is_err() {
            warn!("failed to parse event");
            return None;
        }

        debug!("event: {:?}", &event);

        let payload = event.payload.as_ref()?;
        match payload {
            Payload::EventEspInit(_) => Some(HostedEvent::Init),
            Payload::EventHeartbeat(_) => Some(HostedEvent::Heartbeat),
            Payload::EventStaConnected(e) => Some(HostedEvent::StaConnected { resp: e.resp }),
            Payload::EventStaDisconnected(e) => Some(HostedEvent::StaDisconnected {
                reason: e.sta_disconnected().map(|d| d.reason).unwrap_or(0),
            }),
            _ => None,
        }
    }
}

fn map_mcu_security(val: i32) -> Security {
    match val {
        0 => Security::Open,
        1 => Security::Wep,
        2 => Security::WpaPsk,
        3 => Security::Wpa2Psk,
        4 => Security::WpaWpa2Psk,
        5 => Security::Enterprise,
        6 => Security::Wpa3Psk,
        7 => Security::Wpa2Wpa3Psk,
        8 => Security::WapiPsk,
        9 => Security::Owe,
        10 => Security::Wpa3Ent192,
        11 => Security::Wpa3ExtPsk,
        12 => Security::Wpa3ExtPskMixedMode,
        13 => Security::Dpp,
        14 => Security::Wpa3Enterprise,
        15 => Security::Wpa2Wpa3Enterprise,
        16 => Security::WpaEnterprise,
        n => Security::Unknown(n),
    }
}

fn encode_mcu_security(security: Security) -> i32 {
    match security {
        Security::Open => 0,
        Security::Wep => 1,
        Security::WpaPsk => 2,
        Security::Wpa2Psk => 3,
        Security::WpaWpa2Psk => 4,
        Security::Enterprise | Security::Wpa2Enterprise => 5,
        Security::Wpa3Psk => 6,
        Security::Wpa2Wpa3Psk => 7,
        Security::WapiPsk => 8,
        Security::Owe => 9,
        Security::Wpa3Ent192 => 10,
        Security::Wpa3ExtPsk => 11,
        Security::Wpa3ExtPskMixedMode => 12,
        Security::Dpp => 13,
        Security::Wpa3Enterprise => 14,
        Security::Wpa2Wpa3Enterprise => 15,
        Security::WpaEnterprise => 16,
        Security::Unknown(n) => n,
    }
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

impl From<EspConfig> for wifi_init_config {
    fn from(value: EspConfig) -> Self {
        let mut cfg = wifi_init_config::default();
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

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum BufferType {
    // Static = 0,
    Dynamic = 1,
}

struct StaConfig {
    pub ssid: Vec<u8, 32>,
    pub password: Vec<u8, 32>,
    pub security: Security,
}

impl StaConfig {
    pub(crate) fn new(ssid: &str, password: &str, security: Security) -> Result<Self, Error> {
        Ok(Self {
            ssid: Vec::from_slice(ssid.as_bytes()).map_err(|_| Error::Internal)?,
            password: Vec::from_slice(password.as_bytes()).map_err(|_| Error::Internal)?,
            security,
        })
    }
}

impl From<StaConfig> for wifi_sta_config {
    fn from(value: StaConfig) -> Self {
        let mut sta_config = wifi_sta_config::default();
        sta_config.set_ssid(value.ssid);
        sta_config.set_password(value.password);
        sta_config.set_bssid_set(false);
        sta_config.set_listen_interval(DEFAULT_LISTEN_INTERVAL);
        sta_config.set_channel(NO_CHANNEL_PREFERENCE);
        sta_config.set_failure_retry_cnt(DEFAULT_FAILURE_RETRY_CNT);
        sta_config.set_threshold({
            let mut threshold = wifi_scan_threshold::default();
            threshold.set_authmode(encode_mcu_security(value.security));
            threshold
        });
        sta_config.set_sae_h2e_identifier(heapless::Vec::from_slice(&[0u8; 32]).unwrap());
        sta_config.set_sae_pwe_h2e(DEFAULT_SAE_PWE_H2E);

        sta_config
    }
}

const NO_CHANNEL_PREFERENCE: u32 = 0;
const DEFAULT_LISTEN_INTERVAL: u32 = 3;
const DEFAULT_FAILURE_RETRY_CNT: u32 = 0;
const DEFAULT_SAE_PWE_H2E: i32 = 3;
