use CtrlMsg_::Payload;
use heapless::String;
use micropb::MessageDecode;

use super::{HostedEvent, IoctlCtx, RpcBackend};
use crate::InterfaceType;
use crate::control::{Error, Security, Status};
use crate::proto::fg::{
    CtrlMsg, CtrlMsg_, CtrlMsg_Req_ConfigHeartbeat, CtrlMsg_Req_ConnectAP, CtrlMsg_Req_GetAPConfig,
    CtrlMsg_Req_GetMacAddress, CtrlMsg_Req_GetStatus, CtrlMsg_Req_OTABegin, CtrlMsg_Req_OTAEnd, CtrlMsg_Req_OTAWrite,
    CtrlMsg_Req_SetMode, CtrlMsgId, CtrlMsgType,
};

#[expect(unused)]
enum WifiMode {
    None = 0,
    Sta = 1,
    Ap = 2,
    ApSta = 3,
}

macro_rules! exchange {
    ($ctx:ident, $req_variant:ident, $req:expr) => {{
        let mut msg = CtrlMsg {
            msg_id: CtrlMsgId::$req_variant,
            msg_type: CtrlMsgType::Req,
            payload: Some(Payload::$req_variant($req)),
            req_resp_type: 0,
            uid: 0,
        };

        debug!("ioctl req: {:?}", msg);
        $ctx.exchange(&mut msg).await?;
        debug!("ioctl resp: {:?}", msg);

        msg
    }};
}

/// FG (`esp_hosted_config.proto`) RPC backend.
#[derive(Clone, Copy, Debug, Default)]
pub struct FgBackend;

impl RpcBackend for FgBackend {
    #[inline]
    fn encode_ioctl(&self, buffer: &mut [u8], req: &[u8]) -> usize {
        let req_len = req.len();

        buffer[0..12].copy_from_slice(b"\x01\x08\x00ctrlResp\x02");
        buffer[12..14].copy_from_slice(&(req_len as u16).to_le_bytes());
        buffer[14..][..req_len].copy_from_slice(req);

        req_len + 14
    }

    #[inline]
    fn process_serial_data<'pl>(&mut self, payload: &'pl [u8]) -> Option<(bool, &'pl [u8])> {
        if payload.len() < 14 {
            warn!("serial rx: too short");
            return None;
        }

        let is_event = match &payload[..12] {
            b"\x01\x08\x00ctrlResp\x02" => false,
            b"\x01\x08\x00ctrlEvnt\x02" => true,
            _ => {
                warn!("serial rx: bad tlv");
                return None;
            }
        };

        let len = u16::from_le_bytes(payload[12..14].try_into().unwrap()) as usize;
        if payload.len() < 14 + len {
            warn!("serial rx: too short 2");
            return None;
        }

        Some((is_event, &payload[14..][..len]))
    }

    fn encode_iface_type(&self, iface_type: InterfaceType) -> u8 {
        iface_type as u8
    }

    fn decode_iface_type(&self, iface_type: u8) -> Option<InterfaceType> {
        match iface_type {
            0 => Some(InterfaceType::Sta),
            2 => Some(InterfaceType::Serial),
            _ => None,
        }
    }

    async fn config_heartbeat(&self, ctx: &mut IoctlCtx<'_>, secs: u32) -> Result<(), Error> {
        let req = CtrlMsg_Req_ConfigHeartbeat {
            enable: true,
            duration: secs as i32,
        };
        let msg = exchange!(ctx, ReqConfigHeartbeat, req);
        let Some(Payload::RespConfigHeartbeat(resp)) = msg.payload else {
            return Err(Error::Internal);
        };
        check_resp(resp.resp)
    }

    async fn set_sta_mode(&self, ctx: &mut IoctlCtx<'_>) -> Result<(), Error> {
        let req = CtrlMsg_Req_SetMode {
            mode: WifiMode::Sta as i32,
        };
        let msg = exchange!(ctx, ReqSetWifiMode, req);
        let Some(Payload::RespSetWifiMode(resp)) = msg.payload else {
            return Err(Error::Internal);
        };
        check_resp(resp.resp)
    }

    async fn get_mac_addr(&self, ctx: &mut IoctlCtx<'_>) -> Result<[u8; 6], Error> {
        let req = CtrlMsg_Req_GetMacAddress {
            mode: WifiMode::Sta as i32,
        };
        let msg = exchange!(ctx, ReqGetMacAddress, req);
        let Some(Payload::RespGetMacAddress(resp)) = msg.payload else {
            return Err(Error::Internal);
        };
        check_resp(resp.resp)?;
        let mac_str = core::str::from_utf8(&resp.mac).map_err(|_| Error::Internal)?;
        parse_mac(mac_str)
    }

    async fn connect_ap(&self, ctx: &mut IoctlCtx<'_>, ssid: &str, pwd: &str) -> Result<(), Error> {
        const WIFI_BAND_MODE_AUTO: i32 = 3;

        let req = CtrlMsg_Req_ConnectAP {
            ssid: unwrap!(String::try_from(ssid)),
            pwd: unwrap!(String::try_from(pwd)),
            bssid: String::new(),
            listen_interval: 3,
            is_wpa3_supported: true,
            band_mode: WIFI_BAND_MODE_AUTO,
        };
        let msg = exchange!(ctx, ReqConnectAp, req);
        let Some(Payload::RespConnectAp(resp)) = msg.payload else {
            return Err(Error::Internal);
        };
        check_resp(resp.resp)
    }

    async fn disconnect_ap(&self, ctx: &mut IoctlCtx<'_>) -> Result<(), Error> {
        let req = CtrlMsg_Req_GetStatus {};
        let msg = exchange!(ctx, ReqDisconnectAp, req);
        let Some(Payload::RespDisconnectAp(resp)) = msg.payload else {
            return Err(Error::Internal);
        };
        check_resp(resp.resp)
    }

    async fn get_status(&self, ctx: &mut IoctlCtx<'_>) -> Result<Status, Error> {
        let req = CtrlMsg_Req_GetAPConfig {};
        let msg = exchange!(ctx, ReqGetApConfig, req);
        let Some(Payload::RespGetApConfig(resp)) = msg.payload else {
            return Err(Error::Internal);
        };
        check_resp(resp.resp)?;
        let ssid = core::str::from_utf8(&resp.ssid).map_err(|_| Error::Internal)?;
        let ssid = String::try_from(ssid.trim_end_matches('\0')).map_err(|_| Error::Internal)?;
        let bssid_str = core::str::from_utf8(&resp.bssid).map_err(|_| Error::Internal)?;
        Ok(Status {
            ssid,
            bssid: parse_mac(bssid_str)?,
            rssi: resp.rssi as _,
            channel: resp.chnl as u32,
            security: map_fg_security(resp.sec_prot.0),
        })
    }

    async fn ota_begin(&self, ctx: &mut IoctlCtx<'_>) -> Result<(), Error> {
        let req = CtrlMsg_Req_OTABegin {};
        let msg = exchange!(ctx, ReqOtaBegin, req);
        let Some(Payload::RespOtaBegin(resp)) = msg.payload else {
            return Err(Error::Internal);
        };
        check_resp(resp.resp)
    }

    async fn ota_write(&self, ctx: &mut IoctlCtx<'_>, chunk: &[u8]) -> Result<(), Error> {
        let req = CtrlMsg_Req_OTAWrite {
            ota_data: heapless::Vec::from_slice(chunk).unwrap(),
        };
        let msg = exchange!(ctx, ReqOtaWrite, req);
        let Some(Payload::RespOtaWrite(resp)) = msg.payload else {
            return Err(Error::Internal);
        };
        check_resp(resp.resp)
    }

    async fn ota_end(&self, ctx: &mut IoctlCtx<'_>) -> Result<(), Error> {
        let req = CtrlMsg_Req_OTAEnd {};
        let msg = exchange!(ctx, ReqOtaEnd, req);
        let Some(Payload::RespOtaEnd(resp)) = msg.payload else {
            return Err(Error::Internal);
        };
        check_resp(resp.resp)
    }

    #[inline]
    fn normalize_event(&self, raw: &[u8]) -> Option<HostedEvent> {
        let mut event = CtrlMsg::default();
        if event.decode_from_bytes(raw).is_err() {
            warn!("failed to parse event");
            return None;
        }

        debug!("event: {:?}", &event);

        let payload = event.payload.as_ref()?;
        match payload {
            Payload::EventEspInit(_) => Some(HostedEvent::Init),
            Payload::EventHeartbeat(_) => Some(HostedEvent::Heartbeat),
            Payload::EventStationConnectedToAp(e) => Some(HostedEvent::StaConnected { resp: e.resp }),
            Payload::EventStationDisconnectFromAp(e) => Some(HostedEvent::StaDisconnected { reason: e.reason }),
            _ => None,
        }
    }
}

fn map_fg_security(val: i32) -> Security {
    match val {
        0 => Security::Open,
        1 => Security::Wep,
        2 => Security::WpaPsk,
        3 => Security::Wpa2Psk,
        4 => Security::WpaWpa2Psk,
        5 => Security::Wpa2Enterprise,
        6 => Security::Wpa3Psk,
        7 => Security::Wpa2Wpa3Psk,
        n => Security::Unknown(n),
    }
}

fn check_resp(resp: i32) -> Result<(), Error> {
    if resp != 0 {
        Err(Error::Failed(resp as u32))
    } else {
        Ok(())
    }
}

fn parse_mac(mac: &str) -> Result<[u8; 6], Error> {
    fn nibble_from_hex(b: u8) -> Result<u8, Error> {
        match b {
            b'0'..=b'9' => Ok(b - b'0'),
            b'a'..=b'f' => Ok(b + 0xa - b'a'),
            b'A'..=b'F' => Ok(b + 0xa - b'A'),
            _ => {
                warn!("invalid hex digit {}", b);
                Err(Error::Internal)
            }
        }
    }

    let mac = mac.as_bytes();
    let mut res = [0; 6];
    if mac.len() != 17 {
        warn!("unexpected MAC length");
        return Err(Error::Internal);
    }
    for (i, b) in res.iter_mut().enumerate() {
        *b = (nibble_from_hex(mac[i * 3])? << 4) | nibble_from_hex(mac[i * 3 + 1])?
    }
    Ok(res)
}
