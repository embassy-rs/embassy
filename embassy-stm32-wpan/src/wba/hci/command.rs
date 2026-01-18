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

    #[link_name = "HCI_LE_SET_SCAN_PARAMETERS"]
    fn hci_le_set_scan_parameters(
        le_scan_type: u8,
        le_scan_interval: u16,
        le_scan_window: u16,
        own_address_type: u8,
        scanning_filter_policy: u8,
    ) -> tBleStatus;

    #[link_name = "HCI_LE_SET_SCAN_ENABLE"]
    fn hci_le_set_scan_enable(le_scan_enable: u8, filter_duplicates: u8) -> tBleStatus;

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

    // Connection management commands
    #[link_name = "HCI_DISCONNECT"]
    fn hci_disconnect(connection_handle: u16, reason: u8) -> tBleStatus;

    #[link_name = "HCI_LE_READ_PHY"]
    fn hci_le_read_phy(connection_handle: u16, tx_phy: *mut u8, rx_phy: *mut u8) -> tBleStatus;

    #[link_name = "HCI_LE_SET_PHY"]
    fn hci_le_set_phy(connection_handle: u16, all_phys: u8, tx_phys: u8, rx_phys: u8, phy_options: u16) -> tBleStatus;

    #[link_name = "HCI_LE_CONNECTION_UPDATE"]
    fn hci_le_connection_update(
        connection_handle: u16,
        conn_interval_min: u16,
        conn_interval_max: u16,
        conn_latency: u16,
        supervision_timeout: u16,
        minimum_ce_length: u16,
        maximum_ce_length: u16,
    ) -> tBleStatus;

    #[link_name = "HCI_LE_CREATE_CONNECTION"]
    fn hci_le_create_connection(
        le_scan_interval: u16,
        le_scan_window: u16,
        initiator_filter_policy: u8,
        peer_address_type: u8,
        peer_address: *const [u8; 6],
        own_address_type: u8,
        conn_interval_min: u16,
        conn_interval_max: u16,
        max_latency: u16,
        supervision_timeout: u16,
        min_ce_length: u16,
        max_ce_length: u16,
    ) -> tBleStatus;

    #[link_name = "HCI_LE_CREATE_CONNECTION_CANCEL"]
    fn hci_le_create_connection_cancel() -> tBleStatus;

    #[link_name = "HCI_LE_SET_DATA_LENGTH"]
    fn hci_le_set_data_length(connection_handle: u16, tx_octets: u16, tx_time: u16) -> tBleStatus;
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
            let status = hci_le_set_scan_parameters(
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
            let status = hci_le_set_scan_enable(if enable { 1 } else { 0 }, if filter_duplicates { 1 } else { 0 });
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

    // ===== Connection Management Commands =====

    /// Disconnect a connection
    ///
    /// # Parameters
    ///
    /// - `handle`: Connection handle
    /// - `reason`: Disconnect reason (e.g., 0x13 = Remote User Terminated, 0x16 = Local Host Terminated)
    pub fn disconnect(&self, handle: u16, reason: u8) -> Result<(), BleError> {
        unsafe {
            let status = hci_disconnect(handle, reason);
            Self::check_status(status)
        }
    }

    /// Read the current PHY for a connection
    ///
    /// # Returns
    ///
    /// Tuple of (tx_phy, rx_phy) where:
    /// - 0x01: LE 1M PHY
    /// - 0x02: LE 2M PHY
    /// - 0x03: LE Coded PHY
    pub fn le_read_phy(&self, handle: u16) -> Result<(u8, u8), BleError> {
        unsafe {
            let mut tx_phy = 0u8;
            let mut rx_phy = 0u8;
            let status = hci_le_read_phy(handle, &mut tx_phy, &mut rx_phy);
            Self::check_status(status)?;
            Ok((tx_phy, rx_phy))
        }
    }

    /// Request a PHY change for a connection
    ///
    /// # Parameters
    ///
    /// - `handle`: Connection handle
    /// - `all_phys`: Bit field: bit 0 = no TX PHY preference, bit 1 = no RX PHY preference
    /// - `tx_phys`: Bitmask of preferred TX PHYs (0x01=1M, 0x02=2M, 0x04=Coded)
    /// - `rx_phys`: Bitmask of preferred RX PHYs
    /// - `phy_options`: Coded PHY options (0=no preference, 1=S=2, 2=S=8)
    pub fn le_set_phy(
        &self,
        handle: u16,
        all_phys: u8,
        tx_phys: u8,
        rx_phys: u8,
        phy_options: u16,
    ) -> Result<(), BleError> {
        unsafe {
            let status = hci_le_set_phy(handle, all_phys, tx_phys, rx_phys, phy_options);
            Self::check_status(status)
        }
    }

    /// Request connection parameter update
    ///
    /// # Parameters
    ///
    /// - `handle`: Connection handle
    /// - `interval_min`: Minimum connection interval (units of 1.25ms, range: 6-3200)
    /// - `interval_max`: Maximum connection interval (units of 1.25ms, range: 6-3200)
    /// - `latency`: Slave latency (range: 0-499)
    /// - `supervision_timeout`: Supervision timeout (units of 10ms, range: 10-3200)
    /// - `ce_length_min`: Minimum connection event length (units of 0.625ms)
    /// - `ce_length_max`: Maximum connection event length (units of 0.625ms)
    pub fn le_connection_update(
        &self,
        handle: u16,
        interval_min: u16,
        interval_max: u16,
        latency: u16,
        supervision_timeout: u16,
        ce_length_min: u16,
        ce_length_max: u16,
    ) -> Result<(), BleError> {
        unsafe {
            let status = hci_le_connection_update(
                handle,
                interval_min,
                interval_max,
                latency,
                supervision_timeout,
                ce_length_min,
                ce_length_max,
            );
            Self::check_status(status)
        }
    }

    /// Create a connection to a peripheral device (Central role)
    ///
    /// This initiates a connection to an advertising peripheral device.
    /// The connection complete event will be received when connection is established.
    ///
    /// # Parameters
    ///
    /// - `scan_interval`: Scan interval (units of 0.625ms)
    /// - `scan_window`: Scan window (units of 0.625ms)
    /// - `use_filter_accept_list`: If true, connect to any device in filter accept list
    /// - `peer_addr_type`: Peer address type (0=Public, 1=Random)
    /// - `peer_addr`: Peer device address
    /// - `own_addr_type`: Own address type
    /// - `interval_min`: Minimum connection interval (units of 1.25ms)
    /// - `interval_max`: Maximum connection interval (units of 1.25ms)
    /// - `latency`: Maximum slave latency
    /// - `supervision_timeout`: Supervision timeout (units of 10ms)
    /// - `ce_length_min`: Minimum connection event length
    /// - `ce_length_max`: Maximum connection event length
    #[allow(clippy::too_many_arguments)]
    pub fn le_create_connection(
        &self,
        scan_interval: u16,
        scan_window: u16,
        use_filter_accept_list: bool,
        peer_addr_type: u8,
        peer_addr: &[u8; 6],
        own_addr_type: OwnAddressType,
        interval_min: u16,
        interval_max: u16,
        latency: u16,
        supervision_timeout: u16,
        ce_length_min: u16,
        ce_length_max: u16,
    ) -> Result<(), BleError> {
        unsafe {
            let filter_policy = if use_filter_accept_list { 1 } else { 0 };
            let status = hci_le_create_connection(
                scan_interval,
                scan_window,
                filter_policy,
                peer_addr_type,
                peer_addr as *const [u8; 6],
                own_addr_type as u8,
                interval_min,
                interval_max,
                latency,
                supervision_timeout,
                ce_length_min,
                ce_length_max,
            );
            Self::check_status(status)
        }
    }

    /// Cancel an ongoing connection attempt
    pub fn le_create_connection_cancel(&self) -> Result<(), BleError> {
        unsafe {
            let status = hci_le_create_connection_cancel();
            Self::check_status(status)
        }
    }

    /// Set data length for a connection
    ///
    /// Allows the Host to suggest maximum transmission packet size and maximum packet transmission time.
    ///
    /// # Parameters
    ///
    /// - `handle`: Connection handle
    /// - `tx_octets`: Preferred maximum number of payload octets (27-251)
    /// - `tx_time`: Preferred maximum number of microseconds for TX (328-17040)
    pub fn le_set_data_length(&self, handle: u16, tx_octets: u16, tx_time: u16) -> Result<(), BleError> {
        unsafe {
            let status = hci_le_set_data_length(handle, tx_octets, tx_time);
            Self::check_status(status)
        }
    }
}

impl Default for CommandSender {
    fn default() -> Self {
        Self::new()
    }
}
