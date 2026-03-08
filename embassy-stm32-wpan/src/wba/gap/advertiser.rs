//! GAP Advertiser implementation

use super::types::{AdvData, AdvParams, AdvType};
use crate::wba::error::BleError;
use crate::wba::hci::command::CommandSender;

/// BLE Advertiser
///
/// Provides high-level API for BLE advertising (broadcaster/peripheral role).
///
/// # Example
///
/// ```no_run
/// use embassy_stm32_wpan::wba::gap::{Advertiser, AdvData, AdvParams};
///
/// let cmd_sender = CommandSender::new();
/// let mut advertiser = Advertiser::new(&cmd_sender);
///
/// // Create advertising data
/// let mut adv_data = AdvData::new();
/// adv_data.add_flags(0x06).unwrap();  // General discoverable, BR/EDR not supported
/// adv_data.add_name("MyDevice").unwrap();
///
/// // Start advertising with default parameters
/// advertiser.start(AdvParams::default(), adv_data, None).await.unwrap();
/// ```
pub struct Advertiser<'d> {
    cmd: &'d CommandSender,
    is_advertising: bool,
}

impl<'d> Advertiser<'d> {
    /// Create a new advertiser
    pub fn new(cmd: &'d CommandSender) -> Self {
        Self {
            cmd,
            is_advertising: false,
        }
    }

    /// Start advertising
    ///
    /// # Parameters
    ///
    /// - `params`: Advertising parameters (interval, type, address type, etc.)
    /// - `adv_data`: Advertising data (up to 31 bytes)
    /// - `scan_rsp_data`: Optional scan response data (up to 31 bytes)
    ///
    /// # Returns
    ///
    /// - `Ok(())` if advertising started successfully
    /// - `Err(BleError)` if an error occurred
    ///
    /// # Notes
    ///
    /// This function will stop any ongoing advertising before starting new advertising.
    pub fn start(
        &mut self,
        params: AdvParams,
        adv_data: AdvData,
        scan_rsp_data: Option<AdvData>,
    ) -> Result<(), BleError> {
        // Stop advertising if already advertising
        if self.is_advertising {
            self.stop()?;
        }

        // Validate advertising data length
        if adv_data.len() > 31 {
            return Err(BleError::InvalidParameter);
        }

        if let Some(ref scan_rsp) = scan_rsp_data {
            if scan_rsp.len() > 31 {
                return Err(BleError::InvalidParameter);
            }
        }

        // Extract device name from advertising data if present
        let adv_bytes = adv_data.build();
        let local_name = extract_local_name(adv_bytes);

        // Extract service UUID bytes from advertising data if present
        let service_uuid_bytes = extract_service_uuids_16(adv_bytes);

        // Convert AdvType to ACI advertising type value
        let aci_adv_type = match params.adv_type {
            AdvType::ConnectableUndirected => super::aci_gap::ADV_IND,
            AdvType::ConnectableDirectedHighDuty => super::aci_gap::ADV_DIRECT_IND,
            AdvType::ScannableUndirected => super::aci_gap::ADV_SCAN_IND,
            AdvType::NonConnectableUndirected => super::aci_gap::ADV_NONCONN_IND,
            AdvType::ConnectableDirectedLowDuty => super::aci_gap::ADV_DIRECT_IND_LOW_DUTY,
        };

        // Use aci_gap_set_discoverable - the high-level ACI command
        // This properly configures Link Layer scheduling for advertising
        super::aci_gap::set_discoverable(
            aci_adv_type,
            params.interval_min,
            params.interval_max,
            params.own_addr_type as u8,
            params.filter_policy as u8,
            local_name,
            service_uuid_bytes,
        )?;

        self.is_advertising = true;

        Ok(())
    }

    /// Stop advertising
    ///
    /// # Returns
    ///
    /// - `Ok(())` if advertising stopped successfully
    /// - `Err(BleError)` if an error occurred
    pub fn stop(&mut self) -> Result<(), BleError> {
        if !self.is_advertising {
            return Ok(());
        }

        // Use aci_gap_set_non_discoverable - the high-level ACI command
        super::aci_gap::set_non_discoverable()?;
        self.is_advertising = false;

        Ok(())
    }

    /// Check if currently advertising
    pub fn is_advertising(&self) -> bool {
        self.is_advertising
    }

    /// Update advertising data without stopping advertising
    ///
    /// Note: Some BLE controllers may not support updating advertising data
    /// while advertising is active. If this fails, consider stopping and
    /// restarting advertising.
    pub fn update_adv_data(&mut self, adv_data: AdvData) -> Result<(), BleError> {
        if adv_data.len() > 31 {
            return Err(BleError::InvalidParameter);
        }

        self.cmd.le_set_advertising_data(adv_data.build())
    }

    /// Update scan response data without stopping advertising
    ///
    /// Note: Some BLE controllers may not support updating scan response data
    /// while advertising is active. If this fails, consider stopping and
    /// restarting advertising.
    pub fn update_scan_rsp_data(&mut self, scan_rsp_data: AdvData) -> Result<(), BleError> {
        if scan_rsp_data.len() > 31 {
            return Err(BleError::InvalidParameter);
        }

        self.cmd.le_set_scan_response_data(scan_rsp_data.build())
    }
}

/// Extract local name from advertising data
fn extract_local_name(adv_data: &[u8]) -> Option<&[u8]> {
    let mut offset = 0;
    while offset < adv_data.len() {
        let len = adv_data[offset] as usize;
        if len == 0 {
            break;
        }
        if offset + len >= adv_data.len() {
            break;
        }

        let ad_type = adv_data[offset + 1];
        // AD_TYPE_COMPLETE_LOCAL_NAME = 0x09
        // AD_TYPE_SHORTENED_LOCAL_NAME = 0x08
        if ad_type == 0x09 || ad_type == 0x08 {
            return Some(&adv_data[offset + 2..offset + 1 + len]);
        }

        offset += 1 + len;
    }
    None
}

/// Extract 16-bit service UUID bytes from advertising data
/// Returns raw bytes in format expected by aci_gap_set_discoverable
fn extract_service_uuids_16(adv_data: &[u8]) -> Option<&[u8]> {
    let mut offset = 0;
    while offset < adv_data.len() {
        let len = adv_data[offset] as usize;
        if len == 0 {
            break;
        }
        if offset + len >= adv_data.len() {
            break;
        }

        let ad_type = adv_data[offset + 1];
        // AD_TYPE_16_BIT_SERV_UUID = 0x02 (incomplete list)
        // AD_TYPE_16_BIT_SERV_UUID_CMPLT_LIST = 0x03 (complete list)
        if ad_type == 0x02 || ad_type == 0x03 {
            // Return the UUID data (without length and type bytes)
            return Some(&adv_data[offset + 2..offset + 1 + len]);
        }

        offset += 1 + len;
    }
    None
}

impl<'d> Drop for Advertiser<'d> {
    fn drop(&mut self) {
        // Best effort to stop advertising when advertiser is dropped
        let _ = self.stop();
    }
}
