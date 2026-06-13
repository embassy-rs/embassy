use embassy_net_driver_channel as ch;
use embassy_net_driver_channel::driver::{HardwareAddress, LinkState};
use heapless::String;

use crate::Backend;
use crate::ioctl::Shared;
use crate::rpc::{IoctlCtx, RpcBackend};

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
    /// Wi-Fi EAP security, treated the same as Wpa2Enterprise
    Enterprise,
    /// Wi-Fi EAP security, Authenticate mode : WPA2-Enterprise security
    Wpa2Enterprise,
    /// WPA3-PSK.
    Wpa3Psk,
    /// WPA2/WPA3-PSK.
    Wpa2Wpa3Psk,
    /// WAPI-PSK.
    WapiPsk,
    /// Opportunistic Wireless Encryption.
    Owe,
    /// WPA‑EAP‑Suite‑B‑192.
    Wpa3Ent192,
    /// This authentication mode will yield same result as Wpa3Psk and not recommended to be used. It will be deprecated in future, please use Wpa3Psk instead.
    Wpa3ExtPsk,
    /// This authentication mode will yield same result as Wpa3Psk and not recommended to be used. It will be deprecated in future, please use Wpa3Psk instead.
    Wpa3ExtPskMixedMode,
    /// Device Provisioning Protocol.
    Dpp,
    /// WPA3-Enterprise Only Mode.
    Wpa3Enterprise,
    /// WPA3-Enterprise Transition Mode.
    Wpa2Wpa3Enterprise,
    /// WPA-Enterprise security.
    WpaEnterprise,
    /// Unknown security mode reported by firmware.
    Unknown(i32),
}

/// Handle for managing the network and WiFI state.
pub struct Control<'a> {
    state_ch: ch::StateRunner<'a>,
    shared: &'a Shared,
    backend: Backend,
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

/// Firmware version.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum FwVersion {
    /// esp-hosted-fg version
    #[cfg(feature = "esp-hosted-fg")]
    Fg {
        /// First major version component.
        major1: u32,
        /// Second major version component.
        major2: u32,
        /// Minor version component.
        minor: u32,
        /// First patch version component.
        rev_patch1: u32,
        /// Second patch version component.
        rev_patch2: u32,
    },

    /// esp-hosted-mcu version
    #[cfg(feature = "esp-hosted-mcu")]
    Mcu {
        /// Major version component.
        major: u32,
        /// Minor version component.
        minor: u32,
        /// Patch version component.
        patch: u32,
    },
}

#[expect(unused)]
pub(crate) enum WifiMode {
    None = 0,
    Sta = 1,
    Ap = 2,
    ApSta = 3,
}

impl<'a> Control<'a> {
    pub(crate) fn new(state_ch: ch::StateRunner<'a>, shared: &'a Shared) -> Self {
        Self {
            state_ch,
            shared,
            backend: Backend::default(),
        }
    }

    /// Initialize device.
    pub async fn init(&mut self) -> Result<(), Error> {
        debug!("wait for init event...");
        self.backend = self.shared.init_wait().await;

        let mut ctx = IoctlCtx::new(self.shared);

        debug!("set heartbeat");
        self.backend.config_heartbeat(&mut ctx, 10).await?;

        debug!("wifi_init");
        self.backend.wifi_init(&mut ctx).await?;

        debug!("set wifi mode");
        self.backend.set_mode(&mut ctx, WifiMode::Sta).await?;

        debug!("start wifi");
        self.backend.start_wifi(&mut ctx).await?;

        let mac_addr = self.backend.get_mac_addr(&mut ctx).await?;
        #[cfg(feature = "log")]
        debug!("mac addr: {:02x?}", mac_addr);
        #[cfg(feature = "defmt")]
        debug!("mac addr: {=[u8]:02x}", mac_addr);
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
        self.shared.connect_begin();

        let mut ctx = IoctlCtx::new(self.shared);
        self.backend.connect_ap(&mut ctx, ssid, password).await?;

        self.shared.connect_wait().await.map_err(Error::Failed)?;

        Ok(())
    }

    /// Disconnect from any currently connected network.
    pub async fn disconnect(&mut self) -> Result<(), Error> {
        let mut ctx = IoctlCtx::new(self.shared);
        self.backend.disconnect_ap(&mut ctx).await?;
        self.state_ch.set_link_state(LinkState::Down);
        Ok(())
    }

    /// Return the firmware version of the device.
    pub async fn get_fw_version(&mut self) -> Result<FwVersion, Error> {
        let mut ctx = IoctlCtx::new(self.shared);
        self.backend.get_fw_version(&mut ctx).await
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
