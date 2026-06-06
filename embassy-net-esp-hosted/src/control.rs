use embassy_net_driver_channel as ch;
use embassy_net_driver_channel::driver::{HardwareAddress, LinkState};
use heapless::String;

use crate::ioctl::Shared;
use crate::rpc::{FgBackend, IoctlCtx, RpcBackend};

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

/// WiFi security mode.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Security {
    /// Open network.
    Open,
    /// WEP.
    Wep,
    /// WPA-PSK.
    WpaPsk,
    /// WPA2-PSK.
    Wpa2Psk,
    /// WPA/WPA2-PSK.
    WpaWpa2Psk,
    /// WPA2-Enterprise.
    Wpa2Enterprise,
    /// WPA3-PSK.
    Wpa3Psk,
    /// WPA2/WPA3-PSK.
    Wpa2Wpa3Psk,
    /// Unknown security mode reported by firmware.
    Unknown(i32),
}

impl From<i32> for Security {
    fn from(val: i32) -> Self {
        match val {
            0 => Self::Open,
            1 => Self::Wep,
            2 => Self::WpaPsk,
            3 => Self::Wpa2Psk,
            4 => Self::WpaWpa2Psk,
            5 => Self::Wpa2Enterprise,
            6 => Self::Wpa3Psk,
            7 => Self::Wpa2Wpa3Psk,
            n => Self::Unknown(n),
        }
    }
}

/// Handle for managing the network and WiFI state.
pub struct Control<'a> {
    state_ch: ch::StateRunner<'a>,
    shared: &'a Shared,
    backend: FgBackend,
}

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

impl<'a> Control<'a> {
    pub(crate) fn new(state_ch: ch::StateRunner<'a>, shared: &'a Shared) -> Self {
        Self {
            state_ch,
            shared,
            backend: FgBackend,
        }
    }

    /// Initialize device.
    pub async fn init(&mut self) -> Result<(), Error> {
        debug!("wait for init event...");
        self.shared.init_wait().await;

        let mut ctx = IoctlCtx::new(self.shared);

        debug!("set heartbeat");
        self.backend.config_heartbeat(&mut ctx, 10).await?;

        debug!("set wifi mode");
        self.backend.set_sta_mode(&mut ctx).await?;

        let mac_addr = self.backend.get_mac_addr(&mut ctx).await?;
        debug!("mac addr: {:02x}", mac_addr);
        self.state_ch.set_hardware_address(HardwareAddress::Ethernet(mac_addr));

        Ok(())
    }

    /// Get the current status.
    pub async fn get_status(&mut self) -> Result<Status, Error> {
        let mut ctx = IoctlCtx::new(self.shared);
        self.backend.get_status(&mut ctx).await
    }

    /// Connect to the network identified by ssid using the provided password.
    pub async fn connect(&mut self, ssid: &str, password: &str) -> Result<(), Error> {
        let mut ctx = IoctlCtx::new(self.shared);
        self.backend.connect_ap(&mut ctx, ssid, password).await?;

        // TODO: in newer esp-hosted firmwares that added EventStationConnectedToAp
        // the connect ioctl seems to be async, so we shouldn't immediately set LinkState::Up here.
        self.state_ch.set_link_state(LinkState::Up);

        Ok(())
    }

    /// Disconnect from any currently connected network.
    pub async fn disconnect(&mut self) -> Result<(), Error> {
        let mut ctx = IoctlCtx::new(self.shared);
        self.backend.disconnect_ap(&mut ctx).await?;
        self.state_ch.set_link_state(LinkState::Down);
        Ok(())
    }

    /// Initiate a firmware update.
    pub async fn ota_begin(&mut self) -> Result<(), Error> {
        let mut ctx = IoctlCtx::new(self.shared);
        self.backend.ota_begin(&mut ctx).await
    }

    /// Write slice of firmware to a device.
    ///
    /// [`ota_begin`][Self::ota_begin] must be called first.
    ///
    /// The slice is split into chunks that can be sent across
    /// the ioctl protocol to the wifi adapter.
    pub async fn ota_write(&mut self, data: &[u8]) -> Result<(), Error> {
        let mut ctx = IoctlCtx::new(self.shared);
        for chunk in data.chunks(256) {
            self.backend.ota_write(&mut ctx, chunk).await?;
        }
        Ok(())
    }

    /// End the OTA session.
    ///
    /// [`ota_begin`][Self::ota_begin] must be called first.
    ///
    /// NOTE: Will reset the wifi adapter after 5 seconds.
    pub async fn ota_end(&mut self) -> Result<(), Error> {
        let mut ctx = IoctlCtx::new(self.shared);
        self.backend.ota_end(&mut ctx).await?;
        self.shared.ota_done();
        // Wait for re-init
        self.init().await
    }
}
