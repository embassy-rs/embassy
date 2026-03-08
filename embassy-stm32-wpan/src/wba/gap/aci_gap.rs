//! GAP ACI Commands
//!
//! Higher-level GAP functions that use ACI (Application Command Interface)
//! instead of raw HCI commands. These provide more integrated functionality.

use crate::wba::error::BleError;
use crate::wba::hci::types::Status;

#[allow(non_camel_case_types)]
type tBleStatus = u8;

const BLE_STATUS_SUCCESS: u8 = 0x00;

// Advertising types for aci_gap_set_discoverable
#[allow(dead_code)]
pub const ADV_IND: u8 = 0x00;              // Connectable undirected
#[allow(dead_code)]
pub const ADV_DIRECT_IND: u8 = 0x01;       // Connectable directed
#[allow(dead_code)]
pub const ADV_SCAN_IND: u8 = 0x02;         // Scannable undirected
#[allow(dead_code)]
pub const ADV_NONCONN_IND: u8 = 0x03;      // Non-connectable undirected
#[allow(dead_code)]
pub const ADV_DIRECT_IND_LOW_DUTY: u8 = 0x04;  // Connectable directed low duty cycle

// Advertising filter policy
#[allow(dead_code)]
pub const NO_WHITE_LIST_USE: u8 = 0x00;
#[allow(dead_code)]
pub const WHITE_LIST_FOR_ONLY_SCAN: u8 = 0x01;
#[allow(dead_code)]
pub const WHITE_LIST_FOR_ONLY_CONN: u8 = 0x02;
#[allow(dead_code)]
pub const WHITE_LIST_FOR_ALL: u8 = 0x03;

#[link(name = "stm32wba_ble_stack_basic")]
unsafe extern "C" {
    /// Set device in discoverable mode
    ///
    /// This is the high-level ACI command used by ST for advertising.
    /// It properly configures the Link Layer and schedules advertising work.
    #[link_name = "ACI_GAP_SET_DISCOVERABLE"]
    fn aci_gap_set_discoverable(
        advertising_type: u8,
        advertising_interval_min: u16,
        advertising_interval_max: u16,
        own_address_type: u8,
        advertising_filter_policy: u8,
        local_name_length: u8,
        local_name: *const u8,
        service_uuid_length: u8,
        service_uuid_list: *const u8,
        slave_conn_interval_min: u16,
        slave_conn_interval_max: u16,
    ) -> tBleStatus;

    /// Set device in non-discoverable mode (stop advertising)
    #[link_name = "ACI_GAP_SET_NON_DISCOVERABLE"]
    fn aci_gap_set_non_discoverable() -> tBleStatus;
}

/// Start advertising using aci_gap_set_discoverable
///
/// This is the proper way to start advertising on ST's BLE stack.
/// It configures advertising parameters and triggers Link Layer scheduling.
///
/// # Parameters
///
/// - `adv_type`: Advertising type (ADV_IND, ADV_NONCONN_IND, etc.)
/// - `interval_min/max`: Advertising interval in units of 0.625ms
/// - `own_addr_type`: 0=public, 1=random
/// - `filter_policy`: Advertising filter policy
/// - `local_name`: Device name bytes to include in advertising
/// - `service_uuid_bytes`: Raw bytes of 16-bit service UUIDs (little-endian)
///
pub fn set_discoverable(
    adv_type: u8,
    interval_min: u16,
    interval_max: u16,
    own_addr_type: u8,
    filter_policy: u8,
    local_name: Option<&[u8]>,
    service_uuid_bytes: Option<&[u8]>,
) -> Result<(), BleError> {
    #[cfg(feature = "defmt")]
    defmt::trace!("set_discoverable: preparing to call ACI_GAP_SET_DISCOVERABLE");

    unsafe {
        let (name_ptr, name_len) = match local_name {
            Some(name) => (name.as_ptr(), name.len() as u8),
            None => (core::ptr::null(), 0),
        };

        let (uuid_ptr, uuid_len) = match service_uuid_bytes {
            Some(uuid_bytes) => (uuid_bytes.as_ptr(), uuid_bytes.len() as u8),
            None => (core::ptr::null(), 0),
        };

        #[cfg(feature = "defmt")]
        defmt::trace!("set_discoverable: calling ACI_GAP_SET_DISCOVERABLE (type={}, int_min={}, int_max={})",
            adv_type, interval_min, interval_max);

        let status = aci_gap_set_discoverable(
            adv_type,
            interval_min,
            interval_max,
            own_addr_type,
            filter_policy,
            name_len,
            name_ptr,
            uuid_len,
            uuid_ptr,
            0,     // slave_conn_interval_min (use default)
            0,     // slave_conn_interval_max (use default)
        );

        #[cfg(feature = "defmt")]
        defmt::trace!("set_discoverable: ACI_GAP_SET_DISCOVERABLE returned: 0x{:02X}", status);

        if status == BLE_STATUS_SUCCESS {
            #[cfg(feature = "defmt")]
            defmt::info!("aci_gap_set_discoverable succeeded");
            Ok(())
        } else {
            #[cfg(feature = "defmt")]
            defmt::error!("aci_gap_set_discoverable failed: 0x{:02X}", status);
            Err(BleError::CommandFailed(Status::from_u8(status)))
        }
    }
}

/// Stop advertising using aci_gap_set_non_discoverable
pub fn set_non_discoverable() -> Result<(), BleError> {
    unsafe {
        let status = aci_gap_set_non_discoverable();

        if status == BLE_STATUS_SUCCESS {
            Ok(())
        } else {
            Err(BleError::CommandFailed(Status::from_u8(status)))
        }
    }
}
