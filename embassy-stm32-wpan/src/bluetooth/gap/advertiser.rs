//! GAP advertising helper functions.
//!
//! Low-level advertising operations called by `Ble::start_advertising()`,
//! `Ble::stop_advertising()`, and related methods in `ble.rs`.
//! State management (is_advertising flag, LL enable/disable) lives in `ble.rs`.

use bt_hci::cmd::SyncCmd;
use bt_hci::cmd::le::{LeSetAdvData, LeSetScanResponseData};

use super::aci_gap::{ADV_DIRECT_IND, ADV_DIRECT_IND_LOW_DUTY, ADV_IND, ADV_NONCONN_IND, ADV_SCAN_IND};
use super::types::{AdvData, AdvParams, AdvType, BdAddrType};
use crate::bluetooth::error::BleError;
use crate::controller::ControllerAdapter;

/// Configure advertising parameters and data in the host stack.
///
/// Validates the advertising data, configures legacy undirected advertising
/// (or directed advertising via `aci_gap_set_direct_connectable`), and then
/// pushes the **exact** AD payload built by the caller.
///
/// `aci_gap_set_discoverable` only understands the local name and 16-bit
/// service UUIDs, and auto-adds its own Flags / TX Power AD structures.
/// Without the `update_adv_data` call below, manufacturer-specific data,
/// 128-bit UUIDs, service data and explicit flags / TX power are silently
/// dropped from connectable advertising.
///
/// The GAP command issued here puts the radio on the air by itself. Do not follow
/// it with `le_set_advertise_enable`: ST forbids raw HCI advertising control while
/// the host stack is active, and doing it anyway desynchronises the controller's
/// advertising and filter state from GAP's.
pub(crate) async fn configure(
    controller: &ControllerAdapter<'_>,
    params: &AdvParams,
    adv_data: &AdvData,
    scan_rsp_data: Option<&AdvData>,
) -> Result<(), BleError> {
    // Validate advertising data length
    if adv_data.as_bytes().len() > 31 {
        return Err(BleError::InvalidParameter);
    }
    if let Some(scan_rsp) = scan_rsp_data {
        if scan_rsp.len() > 31 {
            return Err(BleError::InvalidParameter);
        }
    }

    let directed = matches!(
        params.adv_type,
        AdvType::ConnectableDirectedHighDuty | AdvType::ConnectableDirectedLowDuty
    );

    if directed {
        // Legacy directed advertising carries no AD payload, so `adv_data`
        // (and scan responses) do not apply.
        let peer = params.peer_addr.as_ref().ok_or(BleError::InvalidParameter)?;
        let (peer_type, peer_addr) = split_addr(peer);
        let directed_type = match params.adv_type {
            AdvType::ConnectableDirectedHighDuty => ADV_DIRECT_IND,
            _ => ADV_DIRECT_IND_LOW_DUTY,
        };
        super::aci_gap::set_direct_connectable(
            controller,
            params.own_addr_type as u8,
            directed_type,
            peer_type,
            &peer_addr,
            params.interval_min,
            params.interval_max,
        )
        .await?;
        return Ok(());
    }

    let adv_bytes = adv_data.as_bytes();

    // Scan response data goes down over raw HCI, so it must be programmed before a
    // GAP command puts the radio on the air: ST's reference sets it once during
    // init, well before advertising starts, and never touches the link layer once
    // the host stack is driving. See `HCI::start_advertising`.
    if let Some(scan_rsp) = scan_rsp_data {
        update_scan_rsp_data(controller, scan_rsp).await?;
    }

    if params.privacy_undirected {
        // ST BLE_Privacy_Peripheral: undirected connectable + explicit AD bytes
        // via the GAP layer (this path already carries the full payload).
        super::aci_gap::set_undirected_connectable(
            controller,
            params.interval_min,
            params.interval_max,
            params.own_addr_type as u8,
            params.filter_policy as u8,
        )
        .await?;
        super::aci_gap::update_adv_data(controller, adv_bytes).await?;
    } else {
        // Use aci_gap_set_discoverable - the high-level ACI command. It only
        // understands the local name and 16-bit service UUID fields; the full
        // payload is applied below.
        let local_name = extract_local_name(adv_bytes);
        let service_uuid_bytes = extract_service_uuids_16(adv_bytes);
        let aci_adv_type = match params.adv_type {
            AdvType::ConnectableUndirected => ADV_IND,
            AdvType::ScannableUndirected => ADV_SCAN_IND,
            AdvType::NonConnectableUndirected => ADV_NONCONN_IND,
            AdvType::ConnectableDirectedHighDuty | AdvType::ConnectableDirectedLowDuty => {
                unreachable!("directed advertising is handled above")
            }
        };
        super::aci_gap::set_discoverable(
            controller,
            aci_adv_type,
            params.interval_min,
            params.interval_max,
            params.own_addr_type as u8,
            params.filter_policy as u8,
            local_name,
            service_uuid_bytes,
        )
        .await?;

        // Overwrite the auto-generated payload with the caller's exact AD
        // bytes (manufacturer data, 128-bit UUIDs, service data, flags, ...).
        update_adv_data(controller, adv_data).await?;
    }

    Ok(())
}

/// Split a peer address into `(address_type, address_bytes)` where the address
/// type is 0x00 for public and 0x01 for random.
fn split_addr(addr: &BdAddrType) -> (u8, [u8; 6]) {
    (addr.kind(), addr.bytes())
}

/// Remove advertising configuration from the host stack.
///
/// Calls `aci_gap_set_non_discoverable`, which also takes the radio off the air.
/// Do not pair this with `le_set_advertise_enable(false)`; see [`configure`].
pub(crate) async fn unconfigure(controller: &ControllerAdapter<'_>) -> Result<(), BleError> {
    super::aci_gap::set_non_discoverable(controller).await
}

/// Update advertising data while advertising is active.
///
/// Note: Some BLE controllers may not support updating advertising data
/// while advertising is active. If this fails, consider stopping and
/// restarting advertising with new data.
pub(crate) async fn update_adv_data(controller: &ControllerAdapter<'_>, adv_data: &AdvData) -> Result<(), BleError> {
    if adv_data.as_bytes().len() > 31 {
        return Err(BleError::InvalidParameter);
    }
    let bytes = adv_data.as_bytes();
    let mut data = [0u8; 31];
    data[..bytes.len()].copy_from_slice(&bytes);
    LeSetAdvData::new(bytes.len() as u8, data).exec(controller).await?;
    Ok(())
}

/// Update scan response data while advertising is active.
///
/// Note: Some BLE controllers may not support updating scan response data
/// while advertising is active. If this fails, consider stopping and
/// restarting advertising with new data.
pub(crate) async fn update_scan_rsp_data(
    controller: &ControllerAdapter<'_>,
    scan_rsp_data: &AdvData,
) -> Result<(), BleError> {
    if scan_rsp_data.as_bytes().len() > 31 {
        return Err(BleError::InvalidParameter);
    }
    let bytes = scan_rsp_data.as_bytes();
    let mut data = [0u8; 31];
    data[..bytes.len()].copy_from_slice(&bytes);
    LeSetScanResponseData::new(bytes.len() as u8, data)
        .exec(controller)
        .await?;
    Ok(())
}

/// Extract local name AD field from advertising data.
///
/// Returns the slice `[ad_type, name_bytes...]` — the AD-type byte (0x08 or 0x09)
/// is kept as the first byte because `aci_gap_set_discoverable` expects it that
/// way and counts it in `local_name_length`.
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
            return Some(&adv_data[offset + 1..offset + 1 + len]);
        }

        offset += 1 + len;
    }
    None
}

/// Extract 16-bit service UUID AD field from advertising data.
///
/// Returns the slice `[ad_type, uuid_bytes_le...]` — the AD-type byte (0x02 or 0x03)
/// is kept as the first byte because `aci_gap_set_discoverable` expects it that way
/// and counts it in `service_uuid_length`.
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
            return Some(&adv_data[offset + 1..offset + 1 + len]);
        }

        offset += 1 + len;
    }
    None
}
