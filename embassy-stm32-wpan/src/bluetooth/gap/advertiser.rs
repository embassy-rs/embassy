//! GAP advertising helper functions.
//!
//! Low-level advertising operations called by `Ble::start_advertising()`,
//! `Ble::stop_advertising()`, and related methods in `ble.rs`.
//! State management (is_advertising flag, LL enable/disable) lives in `ble.rs`.

use super::aci_gap::{ADV_DIRECT_IND, ADV_DIRECT_IND_LOW_DUTY, ADV_IND, ADV_NONCONN_IND, ADV_SCAN_IND};
use super::types::{AdvData, AdvParams, AdvType};
use crate::bluetooth::error::BleError;
use crate::bluetooth::hci::CommandSender;

/// Configure advertising parameters and data in the host stack.
///
/// Validates the advertising data, extracts the device name and service UUIDs,
/// and calls aci_gap_set_discoverable. Does not enable LL advertising —
/// that is the caller's responsibility via le_set_advertise_enable.
pub(crate) fn configure(
    params: &AdvParams,
    adv_data: &AdvData,
    scan_rsp_data: Option<&AdvData>,
) -> Result<(), BleError> {
    // Validate advertising data length
    if adv_data.len() > 31 {
        return Err(BleError::InvalidParameter);
    }
    if let Some(scan_rsp) = scan_rsp_data {
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
        AdvType::ConnectableUndirected => ADV_IND,
        AdvType::ConnectableDirectedHighDuty => ADV_DIRECT_IND,
        AdvType::ScannableUndirected => ADV_SCAN_IND,
        AdvType::NonConnectableUndirected => ADV_NONCONN_IND,
        AdvType::ConnectableDirectedLowDuty => ADV_DIRECT_IND_LOW_DUTY,
    };

    // Use aci_gap_set_discoverable - the high-level ACI command
    // This configures advertising parameters and data in the host stack
    super::aci_gap::set_discoverable(
        aci_adv_type,
        params.interval_min,
        params.interval_max,
        params.own_addr_type as u8,
        params.filter_policy as u8,
        local_name,
        service_uuid_bytes,
    )
}

/// Remove advertising configuration from the host stack.
///
/// Calls aci_gap_set_non_discoverable. Does not disable LL advertising —
/// that is the caller's responsibility via le_set_advertise_enable.
pub(crate) fn unconfigure() -> Result<(), BleError> {
    super::aci_gap::set_non_discoverable()
}

/// Update advertising data while advertising is active.
///
/// Note: Some BLE controllers may not support updating advertising data
/// while advertising is active. If this fails, consider stopping and
/// restarting advertising with new data.
pub(crate) fn update_adv_data(cmd: &CommandSender, adv_data: &AdvData) -> Result<(), BleError> {
    if adv_data.len() > 31 {
        return Err(BleError::InvalidParameter);
    }
    cmd.le_set_advertising_data(adv_data.build())
}

/// Update scan response data while advertising is active.
///
/// Note: Some BLE controllers may not support updating scan response data
/// while advertising is active. If this fails, consider stopping and
/// restarting advertising with new data.
pub(crate) fn update_scan_rsp_data(cmd: &CommandSender, scan_rsp_data: &AdvData) -> Result<(), BleError> {
    if scan_rsp_data.len() > 31 {
        return Err(BleError::InvalidParameter);
    }
    cmd.le_set_scan_response_data(scan_rsp_data.build())
}

/// Extract local name from advertising data
pub(crate) fn extract_local_name(adv_data: &[u8]) -> Option<&[u8]> {
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

/// Extract 16-bit service UUID bytes from advertising data.
/// Returns raw bytes in the format expected by aci_gap_set_discoverable.
pub(crate) fn extract_service_uuids_16(adv_data: &[u8]) -> Option<&[u8]> {
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
            return Some(&adv_data[offset + 2..offset + 1 + len]);
        }

        offset += 1 + len;
    }
    None
}
