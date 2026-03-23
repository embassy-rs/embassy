use embassy_net_driver_channel as ch;
use embassy_net_driver_channel::driver::HardwareAddress;
use heapless::{String, Vec};
use micropb::{MessageDecode, MessageEncode, PbEncoder};

use crate::ioctl::{LinkState, Shared};
use crate::proto::{self, Rpc as CtrlMsg};
use crate::{EspConfig, Security, StaConfig};

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

#[allow(unused)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
enum WifiInterface {
    Sta = 0,
    Ap = 1,
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

/// WiFi scan type.
#[allow(unused)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
enum ScanType {
    /// Active scan.
    Active = 0,
    /// Passive scan.
    Passive = 1,
}

/// WiFi status.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ApInfo {
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

impl TryFrom<proto::wifi_ap_record> for ApInfo {
    type Error = Error;
    fn try_from(value: proto::wifi_ap_record) -> Result<Self, Self::Error> {
        let ssid = String::from_utf8(value.ssid).map_err(|_| Error::Internal)?;
        let mut bssid = [0u8; 6];
        bssid.copy_from_slice(&value.bssid);
        Ok(Self {
            ssid,
            bssid,
            rssi: value.rssi,
            channel: value.primary,
            security: value.authmode.try_into()?,
        })
    }
}

pub(crate) const MAX_AP_LIST_ENTRIES: usize = 16;

/// Result of a Wifi scan operation
#[derive(Default, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ApList {
    /// List of APs
    pub entries: Vec<ApInfo, MAX_AP_LIST_ENTRIES>,
}

macro_rules! ioctl {
    ($self:ident, $req_variant:tt, $resp_variant:tt, $req:ident, $resp:ident) => {
        let mut msg = proto::Rpc {
            msg_id: proto::RpcId::$req_variant,
            msg_type: proto::RpcType::Req,
            uid: 0,
            payload: Some(proto::Rpc_::Payload::$req_variant($req)),
        };
        $self.ioctl(&mut msg).await?;
        #[allow(unused_mut)]
        let Some(proto::Rpc_::Payload::$resp_variant(mut $resp)) = msg.payload else {
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
    pub async fn init(&mut self, config: EspConfig) -> Result<(), Error> {
        debug!("wait for init event...");
        self.shared.init_wait().await;

        debug!("set heartbeat");
        self.set_heartbeat(10).await?;

        debug!("set wifi init config");
        self.set_wifi_init_config(config).await?;

        debug!("set wifi mode");
        self.set_wifi_mode(WifiMode::Sta as _).await?;

        debug!("start wifi");
        self.start_wifi().await?;

        let mac_addr = self.get_mac_addr().await?;
        self.state_ch
            .set_hardware_address(HardwareAddress::Ethernet(mac_addr));

        Ok(())
    }

    async fn set_wifi_init_config(&mut self, config: EspConfig) -> Result<(), Error> {
        let mut req = proto::Rpc_Req_WifiInit::default();
        req.set_cfg(config.into());

        ioctl!(self, ReqWifiInit, RespWifiInit, req, resp);

        Ok(())
    }

    /// Get the current status.
    /// This may fail if the device is not connected to an AP.
    pub async fn get_status(&mut self) -> Result<ApInfo, Error> {
        let req = proto::Rpc_Req_WifiStaGetApInfo::default();
        ioctl!(self, ReqWifiStaGetApInfo, RespWifiStaGetApInfo, req, resp);
        let ap = &resp.ap_record;
        let ssid = core::str::from_utf8(&ap.ssid).map_err(|_| Error::Internal)?;
        let ssid = String::try_from(ssid.trim_end_matches('\0')).map_err(|_| Error::Internal)?;
        let mut bssid = [0u8; 6];
        bssid.copy_from_slice(&ap.bssid[0..6]);
        Ok(ApInfo {
            ssid,
            bssid,
            rssi: ap.rssi as _,
            channel: ap.primary,
            security: Security::try_from(ap.authmode)?,
        })
    }

    async fn set_sta_config(&mut self, ssid: &str, password: &str) -> Result<(), Error> {
        let mut req = proto::Rpc_Req_WifiSetConfig::default();
        req.set_iface(0); // 0 = STA, 1 = AP
        let sta_config = StaConfig::new(ssid, password, Security::WifiAuthWpa2Psk)?;
        let wifi_config = proto::wifi_config {
            u: Some(proto::wifi_config_::U::Sta(sta_config.into())),
        };
        req.set_cfg(wifi_config);

        ioctl!(self, ReqWifiSetConfig, RespWifiSetConfig, req, resp);

        Ok(())
    }

    async fn start_wifi(&mut self) -> Result<(), Error> {
        let req = proto::Rpc_Req_WifiStart::default();

        ioctl!(self, ReqWifiStart, RespWifiStart, req, resp);

        Ok(())
    }

    async fn connect_to_ap(&mut self) -> Result<(), Error> {
        let req = proto::Rpc_Req_WifiConnect::default();

        ioctl!(self, ReqWifiConnect, RespWifiConnect, req, resp);

        Ok(())
    }

    /// Connect to the network identified by ssid using the provided password.
    pub async fn connect(&mut self, ssid: &str, password: &str) -> Result<bool, Error> {
        self.set_sta_config(ssid, password).await?;
        self.start_wifi().await?;
        self.connect_to_ap().await?;

        let link_state = self.shared.link_event_wait().await;
        if link_state != LinkState::Up {
            return Ok(false);
        }

        Ok(true)
    }

    /// Disconnect from any currently connected network.
    pub async fn disconnect(&mut self) -> Result<(), Error> {
        let req = proto::Rpc_Req_WifiDisconnect::default();
        ioctl!(self, ReqWifiDisconnect, RespWifiDisconnect, req, resp);

        let link_state = self.shared.link_event_wait().await;
        if link_state != LinkState::Down {
            return Err(Error::Internal);
        }

        Ok(())
    }

    /// Initiate a firmware update.
    pub async fn ota_begin(&mut self) -> Result<(), Error> {
        let req = proto::Rpc_Req_OTABegin::default();

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
            let mut req = proto::Rpc_Req_OTAWrite::default();
            req.set_ota_data(heapless::Vec::from_slice(chunk).unwrap());
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
        let req = proto::Rpc_Req_OTAEnd::default();
        ioctl!(self, ReqOtaEnd, RespOtaEnd, req, resp);

        let req = proto::Rpc_Req_OTAActivate::default();
        ioctl!(self, ReqOtaActivate, RespOtaActivate, req, resp);
        self.shared.ota_done();

        Ok(())
    }

    /// Issue a reset request to the device.
    pub async fn reset_device(&mut self) -> Result<(), Error> {
        self.shared.issue_reset_request().await;
        Ok(())
    }

    /// duration in seconds, clamped to [10, 3600]
    async fn set_heartbeat(&mut self, duration: u32) -> Result<(), Error> {
        let req = proto::Rpc_Req_ConfigHeartbeat {
            enable: true,
            duration: duration as i32,
        };
        ioctl!(self, ReqConfigHeartbeat, RespConfigHeartbeat, req, resp);
        Ok(())
    }

    async fn get_mac_addr(&mut self) -> Result<[u8; 6], Error> {
        let req = proto::Rpc_Req_GetMacAddress {
            mode: WifiInterface::Sta as _,
        };
        ioctl!(self, ReqGetMacAddress, RespGetMacAddress, req, resp);
        let mut mac = [0u8; 6];
        mac.copy_from_slice(&resp.mac[..6]);
        Ok(mac)
    }

    async fn set_wifi_mode(&mut self, mode: u32) -> Result<(), Error> {
        let req = proto::Rpc_Req_SetMode { mode: mode as i32 };
        ioctl!(self, ReqSetWifiMode, RespSetWifiMode, req, resp);

        Ok(())
    }

    /// Get the list of APs in the vicinity.
    pub async fn get_ap_list(&mut self) -> Result<ApList, Error> {
        self.issue_ap_scan_command().await?;
        self.retrieve_ap_scan_results().await
    }

    async fn issue_ap_scan_command(&mut self) -> Result<(), Error> {
        let mut req = proto::Rpc_Req_WifiScanStart::default();
        req.set_block(true);
        let mut scan_config = proto::wifi_scan_config::default();
        scan_config.set_show_hidden(true);
        scan_config.set_scan_type(ScanType::Active as _);
        let mut scan_time = proto::wifi_scan_time::default();
        scan_time.set_active(proto::wifi_active_scan_time { min: 80, max: 120 });
        scan_config.set_scan_time(scan_time);
        scan_config.set_home_chan_dwell_time(30);
        req.set_config(scan_config);

        ioctl!(self, ReqWifiScanStart, RespWifiScanStart, req, resp);

        Ok(())
    }

    async fn retrieve_ap_scan_results(&mut self) -> Result<ApList, Error> {
        let length = self.get_ap_scan_length().await?;

        let mut ap_list = ApList::default();

        for _ in 0..length {
            let req = proto::Rpc_Req_WifiScanGetApRecord::default();

            ioctl!(
                self,
                ReqWifiScanGetApRecord,
                RespWifiScanGetApRecord,
                req,
                resp
            );

            ap_list
                .entries
                .push(ApInfo::try_from(resp.ap_record)?)
                .map_err(|_| Error::Internal)?;
        }

        Ok(ap_list)
    }

    async fn get_ap_scan_length(&mut self) -> Result<usize, Error> {
        let req = proto::Rpc_Req_WifiScanGetApNum::default();
        ioctl!(self, ReqWifiScanGetApNum, RespWifiScanGetApNum, req, resp);

        Ok(resp.number as usize)
    }

    async fn ioctl(&mut self, msg: &mut CtrlMsg) -> Result<(), Error> {
        debug!("ioctl req: {:?}", &msg);

        // Theoretical max overhead is 29 bytes. Biggest message is ota write with 256 bytes.
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
