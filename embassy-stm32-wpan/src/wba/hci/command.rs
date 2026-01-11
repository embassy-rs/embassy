//! HCI Command sending using WBA BLE stack C functions
//!
//! The WBA BLE stack provides high-level HCI command functions that we can call directly.
//! These functions are synchronous and return a status code immediately, which simplifies
//! the implementation compared to packet-based HCI.

use super::types::{AdvFilterPolicy, AdvType, OwnAddressType};
use crate::wba::bindings::ble;
use crate::wba::ble::VersionInfo;
use crate::wba::error::BleError;

// The C library exports uppercase function names (HCI_RESET, etc.)
// but the bindings declare lowercase names. We need to link to the actual symbols.
#[allow(non_camel_case_types)]
type tBleStatus = u8;

#[link(name = "stm32wba_ble_stack_basic")]
unsafe extern "C" {
    #[link_name = "HCI_RESET"]
    fn hci_reset() -> tBleStatus;

    #[link_name = "HCI_READ_LOCAL_VERSION_INFORMATION"]
    fn hci_read_local_version_information(
        hci_version: *mut u8,
        hci_revision: *mut u16,
        lmp_pal_version: *mut u8,
        manufacturer_name: *mut u16,
        lmp_pal_subversion: *mut u16,
    ) -> tBleStatus;

    #[link_name = "HCI_READ_BD_ADDR"]
    fn hci_read_bd_addr(bd_addr: *mut [u8; 6]) -> tBleStatus;

    #[link_name = "HCI_SET_EVENT_MASK"]
    fn hci_set_event_mask(event_mask: *const [u8; 8]) -> tBleStatus;

    #[link_name = "HCI_LE_SET_EVENT_MASK"]
    fn hci_le_set_event_mask(le_event_mask: *const [u8; 8]) -> tBleStatus;

    #[link_name = "HCI_LE_SET_ADVERTISING_PARAMETERS"]
    fn hci_le_set_advertising_parameters(
        advertising_interval_min: u16,
        advertising_interval_max: u16,
        advertising_type: u8,
        own_address_type: u8,
        peer_address_type: u8,
        peer_address: *const [u8; 6],
        advertising_channel_map: u8,
        advertising_filter_policy: u8,
    ) -> tBleStatus;

    #[link_name = "HCI_LE_SET_ADVERTISING_DATA"]
    fn hci_le_set_advertising_data(advertising_data_length: u8, advertising_data: *const u8) -> tBleStatus;

    #[link_name = "HCI_LE_SET_SCAN_RESPONSE_DATA"]
    fn hci_le_set_scan_response_data(scan_response_data_length: u8, scan_response_data: *const u8) -> tBleStatus;

    #[link_name = "HCI_LE_SET_ADVERTISING_ENABLE"]
    fn hci_le_set_advertising_enable(advertising_enable: u8) -> tBleStatus;

    #[link_name = "HCI_LE_READ_BUFFER_SIZE_V2"]
    fn hci_le_read_buffer_size_v2(
        le_acl_data_packet_length: *mut u16,
        total_num_le_acl_data_packets: *mut u8,
        iso_data_packet_length: *mut u16,
        total_num_iso_data_packets: *mut u8,
    ) -> tBleStatus;

    #[link_name = "HCI_LE_READ_LOCAL_SUPPORTED_FEATURES_PAGE_0"]
    fn hci_le_read_local_supported_features_page_0(le_features: *mut [u8; 8]) -> tBleStatus;
}

/// BLE Success status code
const BLE_STATUS_SUCCESS: u8 = 0x00;

/// Command sender for HCI commands
///
/// This uses the WBA BLE stack's built-in HCI command functions rather than
/// sending raw HCI packets. The C functions handle packet formatting and
/// communication with the controller.
pub struct CommandSender {
    // Zero-sized type - all operations use C functions
}

impl CommandSender {
    /// Create a new CommandSender
    pub fn new() -> Self {
        Self {}
    }

    /// Check if a BLE status indicates success
    fn check_status(status: u8) -> Result<(), BleError> {
        if status == BLE_STATUS_SUCCESS {
            Ok(())
        } else {
            // Map BLE status to HCI status for error reporting
            Err(BleError::CommandFailed(super::types::Status::from_u8(status)))
        }
    }

    // ===== Controller & Baseband Commands =====

    /// Reset the BLE controller
    pub fn reset(&self) -> Result<(), BleError> {
        unsafe {
            let status = hci_reset();
            Self::check_status(status)
        }
    }

    /// Read local version information
    pub fn read_local_version(&self) -> Result<VersionInfo, BleError> {
        unsafe {
            let mut hci_version = 0u8;
            let mut hci_revision = 0u16;
            let mut lmp_pal_version = 0u8;
            let mut manufacturer_name = 0u16;
            let mut lmp_pal_subversion = 0u16;

            let status = hci_read_local_version_information(
                &mut hci_version,
                &mut hci_revision,
                &mut lmp_pal_version,
                &mut manufacturer_name,
                &mut lmp_pal_subversion,
            );

            Self::check_status(status)?;
            Ok(VersionInfo {
                hci_version,
                hci_revision,
                lmp_version: lmp_pal_version,
                manufacturer_name,
                lmp_subversion: lmp_pal_subversion,
            })
        }
    }

    /// Set event mask
    pub fn set_event_mask(&self, mask: u64) -> Result<(), BleError> {
        unsafe {
            let mask_bytes = mask.to_le_bytes();
            let status = hci_set_event_mask(&mask_bytes as *const [u8; 8]);
            Self::check_status(status)
        }
    }

    /// Read BD_ADDR (Bluetooth device address)
    pub fn read_bd_addr(&self) -> Result<[u8; 6], BleError> {
        unsafe {
            let mut addr = [0u8; 6];
            let status = hci_read_bd_addr(&mut addr as *mut [u8; 6]);
            Self::check_status(status)?;
            Ok(addr)
        }
    }

    // ===== LE Controller Commands =====

    /// Set LE event mask
    pub fn le_set_event_mask(&self, mask: u64) -> Result<(), BleError> {
        unsafe {
            let mask_bytes = mask.to_le_bytes();
            let status = hci_le_set_event_mask(&mask_bytes as *const [u8; 8]);
            Self::check_status(status)
        }
    }

    /// Set random address
    pub fn le_set_random_address(&self, address: &[u8; 6]) -> Result<(), BleError> {
        unsafe {
            let status = ble::hci_le_set_random_address(address.as_ptr());
            Self::check_status(status)
        }
    }

    /// Set advertising parameters
    pub fn le_set_advertising_parameters(
        &self,
        interval_min: u16,
        interval_max: u16,
        adv_type: AdvType,
        own_addr_type: OwnAddressType,
        peer_addr_type: u8,
        peer_addr: &[u8; 6],
        channel_map: u8,
        filter_policy: AdvFilterPolicy,
    ) -> Result<(), BleError> {
        unsafe {
            let status = hci_le_set_advertising_parameters(
                interval_min,
                interval_max,
                adv_type as u8,
                own_addr_type as u8,
                peer_addr_type,
                peer_addr as *const [u8; 6],
                channel_map,
                filter_policy as u8,
            );
            Self::check_status(status)
        }
    }

    /// Read advertising physical channel TX power
    pub fn le_read_advertising_channel_tx_power(&self) -> Result<i8, BleError> {
        unsafe {
            let mut tx_power = 0u8;
            let status = ble::hci_le_read_advertising_physical_channel_tx_power(&mut tx_power);
            Self::check_status(status)?;
            Ok(tx_power as i8)
        }
    }

    /// Set advertising data
    pub fn le_set_advertising_data(&self, data: &[u8]) -> Result<(), BleError> {
        if data.len() > 31 {
            return Err(BleError::InvalidParameter);
        }

        unsafe {
            let status = hci_le_set_advertising_data(data.len() as u8, data.as_ptr());
            Self::check_status(status)
        }
    }

    /// Set scan response data
    pub fn le_set_scan_response_data(&self, data: &[u8]) -> Result<(), BleError> {
        if data.len() > 31 {
            return Err(BleError::InvalidParameter);
        }

        unsafe {
            let status = hci_le_set_scan_response_data(data.len() as u8, data.as_ptr());
            Self::check_status(status)
        }
    }

    /// Enable or disable advertising
    pub fn le_set_advertise_enable(&self, enable: bool) -> Result<(), BleError> {
        unsafe {
            let status = hci_le_set_advertising_enable(if enable { 1 } else { 0 });
            Self::check_status(status)
        }
    }

    /// Set scan parameters
    pub fn le_set_scan_parameters(
        &self,
        scan_type: u8,
        scan_interval: u16,
        scan_window: u16,
        own_addr_type: OwnAddressType,
        filter_policy: u8,
    ) -> Result<(), BleError> {
        unsafe {
            let status = ble::hci_le_set_scan_parameters(
                scan_type,
                scan_interval,
                scan_window,
                own_addr_type as u8,
                filter_policy,
            );
            Self::check_status(status)
        }
    }

    /// Enable or disable scanning
    pub fn le_set_scan_enable(&self, enable: bool, filter_duplicates: bool) -> Result<(), BleError> {
        unsafe {
            let status = ble::hci_le_set_scan_enable(if enable { 1 } else { 0 }, if filter_duplicates { 1 } else { 0 });
            Self::check_status(status)
        }
    }

    /// Read buffer size (v2 - includes ISO parameters)
    pub fn le_read_buffer_size(&self) -> Result<(u16, u8, u16, u8), BleError> {
        unsafe {
            let mut acl_packet_length = 0u16;
            let mut acl_num_packets = 0u8;
            let mut iso_packet_length = 0u16;
            let mut iso_num_packets = 0u8;
            let status = hci_le_read_buffer_size_v2(
                &mut acl_packet_length,
                &mut acl_num_packets,
                &mut iso_packet_length,
                &mut iso_num_packets,
            );
            Self::check_status(status)?;
            Ok((acl_packet_length, acl_num_packets, iso_packet_length, iso_num_packets))
        }
    }

    /// Read local supported LE features (page 0)
    pub fn le_read_local_supported_features(&self) -> Result<[u8; 8], BleError> {
        unsafe {
            let mut features = [0u8; 8];
            let status = hci_le_read_local_supported_features_page_0(&mut features as *mut [u8; 8]);
            Self::check_status(status)?;
            Ok(features)
        }
    }
}

impl Default for CommandSender {
    fn default() -> Self {
        Self::new()
    }
}
