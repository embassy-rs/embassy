//! GAP Advertiser implementation

use super::types::{AdvData, AdvParams};
use crate::wba::error::BleError;
use crate::wba::hci::command::CommandSender;
use crate::wba::hci::types::AddressType;

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

        // Set advertising parameters
        self.cmd.le_set_advertising_parameters(
            params.interval_min,
            params.interval_max,
            params.adv_type.into(),
            params.own_addr_type,
            AddressType::Public as u8, // Peer address type (not used for undirected advertising)
            &[0; 6],                   // Peer address (not used for undirected advertising)
            params.channel_map,
            params.filter_policy,
        )?;

        // Set advertising data
        self.cmd.le_set_advertising_data(adv_data.build())?;

        // Set scan response data if provided
        if let Some(scan_rsp) = scan_rsp_data {
            self.cmd.le_set_scan_response_data(scan_rsp.build())?;
        }

        // Enable advertising
        self.cmd.le_set_advertise_enable(true)?;

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

        self.cmd.le_set_advertise_enable(false)?;
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

impl<'d> Drop for Advertiser<'d> {
    fn drop(&mut self) {
        // Best effort to stop advertising when advertiser is dropped
        let _ = self.stop();
    }
}
