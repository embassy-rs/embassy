use embassy_net_driver_channel as ch;
use embassy_net_driver_channel::driver::{HardwareAddress, LinkState};
use heapless::String;
use micropb::{MessageDecode, MessageEncode, PbEncoder};

use crate::ioctl::Shared;
use crate::proto::{self, CtrlMsg};

/// Errors reported by control.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// The operation failed with the given error code.
    Failed(u32),
    /// The operation timed out.
    Timeout,
    /// Internal error.
    Internal,
}

/// Handle for managing the network and WiFI state.
pub struct Control<'a> {
    state_ch: ch::StateRunner<'a>,
    shared: &'a Shared,
}

/// WiFi mode.
#[allow(unused)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
enum WifiMode {
    /// No mode.
    None = 0,
    /// Client station.
    Sta = 1,
    /// Access point mode.
    Ap = 2,
    /// Repeater mode.
    ApSta = 3,
}

pub use proto::Ctrl_WifiSecProt as Security;

/// WiFi status.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Status {
    /// Service Set Identifier.
    pub ssid: String<32>,
    /// Basic Service Set Identifier.
    pub bssid: [u8; 6],
    /// Received Signal Strength Indicator.
    pub rssi: i32,
    /// WiFi channel.
    pub channel: u32,
    /// Security mode.
    pub security: Security,
}

macro_rules! ioctl {
    ($self:ident, $req_variant:ident, $resp_variant:ident, $req:ident, $resp:ident) => {
        let mut msg = proto::CtrlMsg {
            msg_id: proto::CtrlMsgId::$req_variant,
            msg_type: proto::CtrlMsgType::Req,
            payload: Some(proto::CtrlMsg_::Payload::$req_variant($req)),
            req_resp_type: 0,
            uid: 0,
        };
        $self.ioctl(&mut msg).await?;
        #[allow(unused_mut)]
        let Some(proto::CtrlMsg_::Payload::$resp_variant(mut $resp)) = msg.payload else {
            warn!("unexpected response variant");
            return Err(Error::Internal);
        };
        if $resp.resp != 0 {
            return Err(Error::Failed($resp.resp as u32));
        }
    };
}

impl<'a> Control<'a> {
    pub(crate) fn new(state_ch: ch::StateRunner<'a>, shared: &'a Shared) -> Self {
        Self { state_ch, shared }
    }

    /// Initialize device.
    pub async fn init(&mut self) -> Result<(), Error> {
        debug!("wait for init event...");
        self.shared.init_wait().await;

        debug!("set heartbeat");
        self.set_heartbeat(10).await?;

        debug!("set wifi mode");
        self.set_wifi_mode(WifiMode::Sta as _).await?;

        let mac_addr = self.get_mac_addr().await?;
        debug!("mac addr: {:02x}", mac_addr);
        self.state_ch.set_hardware_address(HardwareAddress::Ethernet(mac_addr));

        Ok(())
    }

    /// Get the current status.
    pub async fn get_status(&mut self) -> Result<Status, Error> {
        let req = proto::CtrlMsg_Req_GetAPConfig {};
        ioctl!(self, ReqGetApConfig, RespGetApConfig, req, resp);
        let ssid = core::str::from_utf8(&resp.ssid).map_err(|_| Error::Internal)?;
        let ssid = String::try_from(ssid.trim_end_matches('\0')).map_err(|_| Error::Internal)?;
        let bssid_str = core::str::from_utf8(&resp.bssid).map_err(|_| Error::Internal)?;
        Ok(Status {
            ssid,
            bssid: parse_mac(bssid_str)?,
            rssi: resp.rssi as _,
            channel: resp.chnl as u32,
            security: resp.sec_prot,
        })
    }

    /// Connect to the network identified by ssid using the provided password.
    pub async fn connect(&mut self, ssid: &str, password: &str) -> Result<(), Error> {
        const WIFI_BAND_MODE_AUTO: i32 = 3; // 2.4GHz + 5GHz

        let req = proto::CtrlMsg_Req_ConnectAP {
            ssid: unwrap!(String::try_from(ssid)),
            pwd: unwrap!(String::try_from(password)),
            bssid: String::new(),
            listen_interval: 3,
            is_wpa3_supported: true,
            band_mode: WIFI_BAND_MODE_AUTO,
        };
        ioctl!(self, ReqConnectAp, RespConnectAp, req, resp);

        // TODO: in newer esp-hosted firmwares that added EventStationConnectedToAp
        // the connect ioctl seems to be async, so we shouldn't immediately set LinkState::Up here.
        self.state_ch.set_link_state(LinkState::Up);

        Ok(())
    }

    /// Disconnect from any currently connected network.
    pub async fn disconnect(&mut self) -> Result<(), Error> {
        let req = proto::CtrlMsg_Req_GetStatus {};
        ioctl!(self, ReqDisconnectAp, RespDisconnectAp, req, resp);
        self.state_ch.set_link_state(LinkState::Down);
        Ok(())
    }

    /// Initiate a firmware update.
    pub async fn ota_begin(&mut self) -> Result<(), Error> {
        let req = proto::CtrlMsg_Req_OTABegin {};
        ioctl!(self, ReqOtaBegin, RespOtaBegin, req, resp);
        Ok(())
    }

    /// Write slice of firmware to a device.
    ///
    /// [`ota_begin`] must be called first.
    ///
    /// The slice is split into chunks that can be sent across
    /// the ioctl protocol to the wifi adapter.
    pub async fn ota_write(&mut self, data: &[u8]) -> Result<(), Error> {
        for chunk in data.chunks(256) {
            let req = proto::CtrlMsg_Req_OTAWrite {
                ota_data: heapless::Vec::from_slice(chunk).unwrap(),
            };
            ioctl!(self, ReqOtaWrite, RespOtaWrite, req, resp);
        }
        Ok(())
    }

    /// End the OTA session.
    ///
    /// [`ota_begin`] must be called first.
    ///
    /// NOTE: Will reset the wifi adapter after 5 seconds.
    pub async fn ota_end(&mut self) -> Result<(), Error> {
        let req = proto::CtrlMsg_Req_OTAEnd {};
        ioctl!(self, ReqOtaEnd, RespOtaEnd, req, resp);
        self.shared.ota_done();
        // Wait for re-init
        self.init().await?;
        Ok(())
    }

    /// duration in seconds, clamped to [10, 3600]
    async fn set_heartbeat(&mut self, duration: u32) -> Result<(), Error> {
        let req = proto::CtrlMsg_Req_ConfigHeartbeat {
            enable: true,
            duration: duration as i32,
        };
        ioctl!(self, ReqConfigHeartbeat, RespConfigHeartbeat, req, resp);
        Ok(())
    }

    async fn get_mac_addr(&mut self) -> Result<[u8; 6], Error> {
        let req = proto::CtrlMsg_Req_GetMacAddress {
            mode: WifiMode::Sta as _,
        };
        ioctl!(self, ReqGetMacAddress, RespGetMacAddress, req, resp);
        let mac_str = core::str::from_utf8(&resp.mac).map_err(|_| Error::Internal)?;
        parse_mac(mac_str)
    }

    async fn set_wifi_mode(&mut self, mode: u32) -> Result<(), Error> {
        let req = proto::CtrlMsg_Req_SetMode { mode: mode as i32 };
        ioctl!(self, ReqSetWifiMode, RespSetWifiMode, req, resp);

        Ok(())
    }

    async fn ioctl(&mut self, msg: &mut CtrlMsg) -> Result<(), Error> {
        debug!("ioctl req: {:?}", &msg);

        // Theoretical max overhead is 29 bytes. Biggest message is OTA write with 256 bytes.
        let mut buf = [0u8; 256 + 29];
        let buf_len = buf.len();

        let mut encoder = PbEncoder::new(&mut buf[..]);
        msg.encode(&mut encoder).map_err(|_| {
            warn!("failed to serialize control request");
            Error::Internal
        })?;
        let remaining = encoder.into_writer();
        let req_len = buf_len - remaining.len();

        struct CancelOnDrop<'a>(&'a Shared);

        impl CancelOnDrop<'_> {
            fn defuse(self) {
                core::mem::forget(self);
            }
        }

        impl Drop for CancelOnDrop<'_> {
            fn drop(&mut self) {
                self.0.ioctl_cancel();
            }
        }

        let ioctl = CancelOnDrop(self.shared);

        let resp_len = ioctl.0.ioctl(&mut buf, req_len).await;

        ioctl.defuse();

        msg.decode_from_bytes(&buf[..resp_len]).map_err(|_| {
            warn!("failed to deserialize control response");
            Error::Internal
        })?;
        debug!("ioctl resp: {:?}", msg);

        Ok(())
    }
}

// WHY IS THIS A STRING? WHYYYY
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
