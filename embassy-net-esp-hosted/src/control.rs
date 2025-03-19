use embassy_net_driver_channel as ch;
use embassy_net_driver_channel::driver::{HardwareAddress, LinkState};
use heapless::String;

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

pub use proto::CtrlWifiBw as Bandwidth;
pub use proto::CtrlWifiSecProt as Security;

use crate::proto::CtrlWifiMode::{Ap, Sta};

/// WiFi station status.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct StaStatus {
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

/// WiFi access point status.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ApStatus {
    /// Service Set Identifier.
    pub ssid: String<32>,
    /// Password.
    pub psk: String<32>,
    /// WiFi channel.
    pub channel: u32,
    /// Security mode.
    pub security: Security,
    /// Max number of connections.
    pub max_connections: u32,
    /// SSID Hidden
    pub hidden: bool,
    /// Bandwidth
    pub bandwidth: Bandwidth,
}

macro_rules! ioctl {
    ($self:ident, $req_variant:ident, $resp_variant:ident, $req:ident, $resp:ident) => {
        let mut msg = proto::CtrlMsg {
            msg_id: proto::CtrlMsgId::$req_variant as _,
            msg_type: proto::CtrlMsgType::Req as _,
            payload: Some(proto::CtrlMsgPayload::$req_variant($req)),
        };
        $self.ioctl(&mut msg).await?;
        #[allow(unused_mut)]
        let Some(proto::CtrlMsgPayload::$resp_variant(mut $resp)) = msg.payload
        else {
            warn!("unexpected response variant");
            return Err(Error::Internal);
        };
        if $resp.resp != 0 {
            return Err(Error::Failed($resp.resp));
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

    /// Get the current station status.
    pub async fn get_sta_status(&mut self) -> Result<StaStatus, Error> {
        let req = proto::CtrlMsgReqGetApConfig {};
        ioctl!(self, ReqGetApConfig, RespGetApConfig, req, resp);
        trim_nulls(&mut resp.ssid);
        Ok(StaStatus {
            ssid: resp.ssid,
            bssid: parse_mac(&resp.bssid)?,
            rssi: resp.rssi as _,
            channel: resp.chnl,
            security: resp.sec_prot,
        })
    }

    /// Get the current access point status.
    pub async fn get_ap_status(&mut self) -> Result<ApStatus, Error> {
        let req = proto::CtrlMsgReqGetSoftApConfig {};
        ioctl!(self, ReqGetSoftApConfig, RespGetSoftapConfig, req, resp);
        trim_nulls(&mut resp.ssid);
        Ok(ApStatus {
            ssid: resp.ssid,
            psk: resp.pwd,
            channel: resp.chnl,
            security: resp.sec_prot,
            max_connections: resp.max_conn,
            hidden: resp.ssid_hidden,
            bandwidth: Bandwidth::Ht20,
        })
    }

    /// Connect to the network identified by ssid using the provided password.
    pub async fn connect(&mut self, ssid: &str, password: &str) -> Result<(), Error> {
        let req = proto::CtrlMsgReqConnectAp {
            ssid: unwrap!(String::try_from(ssid)),
            pwd: unwrap!(String::try_from(password)),
            bssid: String::new(),
            listen_interval: 3,
            is_wpa3_supported: true,
        };
        ioctl!(self, ReqConnectAp, RespConnectAp, req, resp);
        self.state_ch.set_link_state(LinkState::Up);
        Ok(())
    }

    /// Disconnect from any currently connected network.
    pub async fn disconnect(&mut self) -> Result<(), Error> {
        let req = proto::CtrlMsgReqGetStatus {};
        ioctl!(self, ReqDisconnectAp, RespDisconnectAp, req, resp);
        self.state_ch.set_link_state(LinkState::Down);
        Ok(())
    }

    /// Set the interface to AP mode
    pub async fn set_ap_mode(&mut self) -> Result<(), Error> {
        let req = proto::CtrlMsgReqSetMode { mode: Ap as u32 };
        ioctl!(self, ReqSetWifiMode, RespSetWifiMode, req, resp);
        self.shared.set_is_ap(true);
        Ok(())
    }

    /// Set the interface to Sta mode
    pub async fn set_sta_mode(&mut self) -> Result<(), Error> {
        let req = proto::CtrlMsgReqSetMode { mode: Sta as u32 };
        ioctl!(self, ReqSetWifiMode, RespSetWifiMode, req, resp);
        self.shared.set_is_ap(false);
        Ok(())
    }

    /// Start and configure an AP
    pub async fn start_ap(&mut self, ap_config: ApStatus) -> Result<(), Error> {
        let req = proto::CtrlMsgReqStartSoftAp {
            ssid: ap_config.ssid,
            pwd: ap_config.psk,
            chnl: ap_config.channel,
            sec_prot: ap_config.security,
            max_conn: ap_config.max_connections,
            ssid_hidden: ap_config.hidden,
            bw: ap_config.bandwidth as u32,
        };
        ioctl!(self, ReqStartSoftAp, RespStartSoftAp, req, resp);
        Ok(())
    }

    /// Stop AP
    pub async fn stop_ap(&mut self) -> Result<(), Error> {
        let req = proto::CtrlMsgReqGetStatus {};
        ioctl!(self, ReqStopSoftAp, RespStopSoftAp, req, resp);
        Ok(())
    }

    /// Get Station mac address
    pub async fn get_mac_addr(&mut self) -> Result<[u8; 6], Error> {
        let req = proto::CtrlMsgReqGetMacAddress {
            mode: WifiMode::Sta as _,
        };
        ioctl!(self, ReqGetMacAddress, RespGetMacAddress, req, resp);
        parse_mac(&resp.mac)
    }

    /// duration in seconds, clamped to [10, 3600]
    async fn set_heartbeat(&mut self, duration: u32) -> Result<(), Error> {
        let req = proto::CtrlMsgReqConfigHeartbeat { enable: true, duration };
        ioctl!(self, ReqConfigHeartbeat, RespConfigHeartbeat, req, resp);
        Ok(())
    }

    async fn set_wifi_mode(&mut self, mode: u32) -> Result<(), Error> {
        let req = proto::CtrlMsgReqSetMode { mode };
        ioctl!(self, ReqSetWifiMode, RespSetWifiMode, req, resp);

        Ok(())
    }

    async fn ioctl(&mut self, msg: &mut CtrlMsg) -> Result<(), Error> {
        debug!("ioctl req: {:?}", &msg);

        let mut buf = [0u8; 256];

        let req_len = noproto::write(msg, &mut buf).map_err(|_| {
            warn!("failed to serialize control request");
            Error::Internal
        })?;

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

        *msg = noproto::read(&buf[..resp_len]).map_err(|_| {
            warn!("failed to serialize control request");
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

fn trim_nulls<const N: usize>(s: &mut String<N>) {
    while s.chars().rev().next() == Some(0 as char) {
        s.pop();
    }
}
