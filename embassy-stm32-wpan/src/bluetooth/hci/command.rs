//! HCI Command sending using WBA BLE stack C functions
//!
//! The WBA BLE stack provides high-level HCI command functions that we can call directly.
//! These functions are synchronous and return a status code immediately, which simplifies
//! the implementation compared to packet-based HCI.

use stm32_bindings::ble::{
    aci_hal_le_tx_test_packet_number, hci_disconnect, hci_le_connection_cte_request_enable,
    hci_le_connection_cte_response_enable, hci_le_connection_update, hci_le_create_connection,
    hci_le_create_connection_cancel, hci_le_read_advertising_physical_channel_tx_power,
    hci_le_read_antenna_information, hci_le_read_buffer_size_v2, hci_le_read_local_supported_features_page_0,
    hci_le_read_phy, hci_le_receiver_test, hci_le_receiver_test_v2, hci_le_set_advertising_data,
    hci_le_set_advertising_enable, hci_le_set_advertising_parameters, hci_le_set_connection_cte_receive_parameters,
    hci_le_set_connection_cte_transmit_parameters, hci_le_set_data_length, hci_le_set_event_mask, hci_le_set_phy,
    hci_le_set_random_address, hci_le_set_scan_enable, hci_le_set_scan_parameters, hci_le_set_scan_response_data,
    hci_le_test_end, hci_le_transmitter_test, hci_le_transmitter_test_v2, hci_read_bd_addr,
    hci_read_local_version_information, hci_reset, hci_set_event_mask,
};
use stm32wb_hci::BdAddrType;
use stm32wb_hci::host::OwnAddressType;

use crate::bluetooth::VersionInfo;
use crate::bluetooth::error::BleError;
use crate::bluetooth::hci::types;

/// BLE Success status code
const BLE_STATUS_SUCCESS: u8 = 0x00;

/// Enable or disable LE advertising in the link layer.
///
/// Standalone function for use outside the normal CommandSender flow
/// (e.g., re-enabling advertising after disconnect, or kicking the LL
/// after runner startup when ACI_GAP_SET_DISCOVERABLE didn't enable it).
pub fn le_set_advertising_enable(enable: bool) -> Result<(), BleError> {
    let status = unsafe { hci_le_set_advertising_enable(if enable { 1 } else { 0 }) };
    if status == BLE_STATUS_SUCCESS {
        Ok(())
    } else {
        Err(BleError::CommandFailed(super::types::Status::from_u8(status)))
    }
}

/// Enable or disable LE scanning in the link layer.
///
/// Standalone function for kicking the LL after runner startup when
/// the scan enable command was issued before the runner was active.
pub fn le_set_scan_enable(enable: bool, filter_duplicates: bool) -> Result<(), BleError> {
    let status = unsafe { hci_le_set_scan_enable(if enable { 1 } else { 0 }, if filter_duplicates { 1 } else { 0 }) };
    if status == BLE_STATUS_SUCCESS {
        Ok(())
    } else {
        Err(BleError::CommandFailed(super::types::Status::from_u8(status)))
    }
}

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
            let status = hci_set_event_mask(&mask_bytes as *const [u8; 8] as *const u8);
            Self::check_status(status)
        }
    }

    /// Read BD_ADDR (Bluetooth device address)
    pub fn read_bd_addr(&self) -> Result<[u8; 6], BleError> {
        unsafe {
            let mut addr = [0u8; 6];
            let status = hci_read_bd_addr(&mut addr as *mut [u8; 6] as *mut u8);
            Self::check_status(status)?;
            Ok(addr)
        }
    }

    // ===== LE Controller Commands =====

    /// Set LE event mask
    pub fn le_set_event_mask(&self, mask: u64) -> Result<(), BleError> {
        unsafe {
            let mask_bytes = mask.to_le_bytes();
            let status = hci_le_set_event_mask(&mask_bytes as *const [u8; 8] as *const u8);
            Self::check_status(status)
        }
    }

    /// Set random address
    pub fn le_set_random_address(&self, address: &[u8; 6]) -> Result<(), BleError> {
        unsafe {
            let status = hci_le_set_random_address(address.as_ptr());
            Self::check_status(status)
        }
    }

    /// Set advertising parameters
    pub fn le_set_advertising_parameters(
        &self,
        interval_min: u16,
        interval_max: u16,
        adv_type: types::AdvType,
        own_addr_type: OwnAddressType,
        peer_addr_type: u8,
        peer_addr: &[u8; 6],
        channel_map: u8,
        filter_policy: types::AdvFilterPolicy,
    ) -> Result<(), BleError> {
        unsafe {
            let status = hci_le_set_advertising_parameters(
                interval_min,
                interval_max,
                adv_type as u8,
                own_addr_type as u8,
                peer_addr_type,
                peer_addr as *const [u8; 6] as *const u8,
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
            let status = hci_le_read_advertising_physical_channel_tx_power(&mut tx_power);
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
            let status = hci_le_read_local_supported_features_page_0(&mut features as *mut [u8; 8] as *mut u8);
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
        peer_addr: BdAddrType,
        own_addr_type: OwnAddressType,
        interval_min: u16,
        interval_max: u16,
        latency: u16,
        supervision_timeout: u16,
        ce_length_min: u16,
        ce_length_max: u16,
    ) -> Result<(), BleError> {
        let mut peer_addr_bytes = [0u8; 7];
        peer_addr.copy_into_slice(&mut peer_addr_bytes);

        let peer_addr_ptr: &[u8; 6] = &peer_addr_bytes[1..7].try_into().unwrap();

        unsafe {
            let filter_policy = if use_filter_accept_list { 1 } else { 0 };
            let status = hci_le_create_connection(
                scan_interval,
                scan_window,
                filter_policy,
                peer_addr_bytes[0],
                peer_addr_ptr as *const [u8; 6] as *const u8,
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

    // ===== Direction Finding / CTE Commands =====

    /// Set CTE transmit parameters for a connection (peripheral/tag side).
    ///
    /// Configure which CTE types the peripheral can transmit and the antenna switching
    /// pattern used for AoD. Call before enabling CTE transmit response.
    ///
    /// - `cte_types`: Bit field — bit 0=AoA, bit 1=AoD 1μs slots, bit 2=AoD 2μs slots
    /// - `antenna_ids`: Antenna switching pattern IDs (2–75 elements)
    pub fn le_set_connection_cte_transmit_parameters(
        &self,
        handle: u16,
        cte_types: u8,
        antenna_ids: &[u8],
    ) -> Result<(), BleError> {
        if antenna_ids.len() < 2 || antenna_ids.len() > 75 {
            return Err(BleError::InvalidParameter);
        }
        unsafe {
            let status = hci_le_set_connection_cte_transmit_parameters(
                handle,
                cte_types,
                antenna_ids.len() as u8,
                antenna_ids.as_ptr(),
            );
            Self::check_status(status)
        }
    }

    /// Enable or disable CTE response for a connection (peripheral/tag side).
    ///
    /// When enabled, the peripheral responds with CTE whenever the central requests it
    /// (BT spec 7.8.86 `HCI_LE_Connection_CTE_Response_Enable`).
    /// Call `le_set_connection_cte_transmit_parameters` first.
    pub fn le_connection_cte_response_enable(&self, handle: u16, enable: bool) -> Result<(), BleError> {
        unsafe {
            let status = hci_le_connection_cte_response_enable(handle, if enable { 1 } else { 0 });
            Self::check_status(status)
        }
    }

    /// Set CTE receive (IQ sampling) parameters for a connection (central/locator side).
    ///
    /// Configure IQ sample collection for incoming CTE packets. When enabled, IQ reports
    /// arrive as `LeConnectionIqReport` events.
    ///
    /// - `sampling_enable`: true to enable IQ sampling
    /// - `slot_durations`: 0x01 = 1μs slots, 0x02 = 2μs slots
    /// - `antenna_ids`: Antenna switching pattern (2–75 elements; ignored if sampling disabled)
    pub fn le_set_connection_cte_receive_parameters(
        &self,
        handle: u16,
        sampling_enable: bool,
        slot_durations: u8,
        antenna_ids: &[u8],
    ) -> Result<(), BleError> {
        if sampling_enable && (antenna_ids.len() < 2 || antenna_ids.len() > 75) {
            return Err(BleError::InvalidParameter);
        }
        unsafe {
            let status = hci_le_set_connection_cte_receive_parameters(
                handle,
                if sampling_enable { 1 } else { 0 },
                slot_durations,
                antenna_ids.len() as u8,
                antenna_ids.as_ptr(),
            );
            Self::check_status(status)
        }
    }

    /// Enable or disable CTE requests for a connection (central/locator side).
    ///
    /// When enabled, the central periodically asks the peripheral to send CTE.
    /// IQ samples arrive via `LeConnectionIqReport` events.
    ///
    /// - `enable`: true to start CTE requests
    /// - `request_interval`: 0 = request once; N = request every N connection events
    /// - `requested_cte_length`: Requested CTE length in 8μs units (range: 2–20)
    /// - `requested_cte_type`: 0x00=AoA, 0x01=AoD 1μs, 0x02=AoD 2μs
    pub fn le_connection_cte_request_enable(
        &self,
        handle: u16,
        enable: bool,
        request_interval: u16,
        requested_cte_length: u8,
        requested_cte_type: u8,
    ) -> Result<(), BleError> {
        unsafe {
            let status = hci_le_connection_cte_request_enable(
                handle,
                if enable { 1 } else { 0 },
                request_interval,
                requested_cte_length,
                requested_cte_type,
            );
            Self::check_status(status)
        }
    }

    /// Read antenna information from the controller.
    ///
    /// Returns `(switching_sampling_rates, num_antennae, max_pattern_length, max_cte_length)`.
    /// - `switching_sampling_rates`: Bit field of supported switching/sampling rates
    /// - `num_antennae`: Number of antennae supported by the controller
    /// - `max_pattern_length`: Maximum supported switching pattern length
    /// - `max_cte_length`: Maximum supported CTE length in 8μs units
    pub fn le_read_antenna_information(&self) -> Result<(u8, u8, u8, u8), BleError> {
        unsafe {
            let mut switching_rates = 0u8;
            let mut num_antennae = 0u8;
            let mut max_pattern_len = 0u8;
            let mut max_cte_len = 0u8;
            let status = hci_le_read_antenna_information(
                &mut switching_rates,
                &mut num_antennae,
                &mut max_pattern_len,
                &mut max_cte_len,
            );
            Self::check_status(status)?;
            Ok((switching_rates, num_antennae, max_pattern_len, max_cte_len))
        }
    }
}

impl Default for CommandSender {
    fn default() -> Self {
        Self::new()
    }
}

/// Start a DTM receiver test on the given channel (v1, 1M PHY).
///
/// `rx_channel`: 0–39. Frequency = 2402 + (2 × N) MHz.
/// Call `le_test_end()` to stop and read the received packet count.
pub(crate) fn le_receiver_test(rx_channel: u8) -> Result<(), BleError> {
    let status = unsafe { hci_le_receiver_test(rx_channel) };
    if status == BLE_STATUS_SUCCESS {
        Ok(())
    } else {
        Err(BleError::CommandFailed(super::types::Status::from_u8(status)))
    }
}

/// Start a DTM receiver test with PHY selection (v2).
///
/// `modulation_index`: 0x00 = standard, 0x01 = stable.
pub(crate) fn le_receiver_test_v2(
    rx_channel: u8,
    phy: super::types::DtmRxPhy,
    modulation_index: u8,
) -> Result<(), BleError> {
    let status = unsafe { hci_le_receiver_test_v2(rx_channel, phy as u8, modulation_index) };
    if status == BLE_STATUS_SUCCESS {
        Ok(())
    } else {
        Err(BleError::CommandFailed(super::types::Status::from_u8(status)))
    }
}

/// Start a DTM transmitter test (v1, 1M PHY).
///
/// `tx_channel`: 0–39. Frequency = 2402 + (2 × N) MHz.
/// `test_data_length`: payload bytes per packet, 0–255.
pub(crate) fn le_transmitter_test(
    tx_channel: u8,
    test_data_length: u8,
    packet_payload: types::DtmPacketPayload,
) -> Result<(), BleError> {
    let status = unsafe { hci_le_transmitter_test(tx_channel, test_data_length, packet_payload as u8) };
    if status == BLE_STATUS_SUCCESS {
        Ok(())
    } else {
        Err(BleError::CommandFailed(types::Status::from_u8(status)))
    }
}

/// Start a DTM transmitter test with PHY selection (v2).
pub(crate) fn le_transmitter_test_v2(
    tx_channel: u8,
    test_data_length: u8,
    packet_payload: types::DtmPacketPayload,
    phy: types::DtmTxPhy,
) -> Result<(), BleError> {
    let status = unsafe { hci_le_transmitter_test_v2(tx_channel, test_data_length, packet_payload as u8, phy as u8) };
    if status == BLE_STATUS_SUCCESS {
        Ok(())
    } else {
        Err(BleError::CommandFailed(types::Status::from_u8(status)))
    }
}

/// End a DTM test and return the received packet count.
///
/// For a **receiver** test: returns the number of packets received.
/// For a **transmitter** test: always returns 0 (per BLE spec Vol 4 Part E §7.8.30).
pub(crate) fn le_test_end() -> Result<u16, BleError> {
    let mut num_packets: u16 = 0;
    let status = unsafe { hci_le_test_end(&mut num_packets) };
    if status == BLE_STATUS_SUCCESS {
        Ok(num_packets)
    } else {
        Err(BleError::CommandFailed(super::types::Status::from_u8(status)))
    }
}

/// Read the number of TX packets sent during an active transmitter test.
///
/// ST proprietary command. Call this while the test is running to monitor
/// progress without ending the test.
pub(crate) fn aci_hal_tx_test_packet_number() -> Result<u32, BleError> {
    let mut num_packets: u32 = 0;
    let status = unsafe { aci_hal_le_tx_test_packet_number(&mut num_packets) };
    if status == BLE_STATUS_SUCCESS {
        Ok(num_packets)
    } else {
        Err(BleError::CommandFailed(super::types::Status::from_u8(status)))
    }
}
