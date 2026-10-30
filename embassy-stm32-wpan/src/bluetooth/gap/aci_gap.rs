//! GAP ACI Commands
//!
//! Higher-level GAP functions that use ACI (Application Command Interface)
//! instead of raw HCI commands. These provide more integrated functionality.
//!
//! Every command is declared by `stm32wb-hci` and executed through the
//! [`ControllerAdapter`], which calls the same BLE stack functions.

use bt_hci::cmd::{AsyncCmd, SyncCmd};
use bt_hci::param::BdAddr;
use stm32wb_hci::aci::durations::{
    AdvInterval, DirectAdvInterval, ExtScanInterval, ExtScanWindow, ScanDuration, ScanPeriod,
};
use stm32wb_hci::aci::flags::ScanningPhys;
use stm32wb_hci::aci::gap::{
    GapExtStartScan, GapGetOobData, GapSetDirectConnectable, GapSetDiscoverable, GapSetNonDiscoverable, GapSetOobData,
    GapSetUndirectedConnectable, GapTerminateGapProc, GapUpdateAdvData, ScanParamPhy,
};
use stm32wb_hci::aci::values::{
    AddressType, AdvertisingType, ConnectableOwnAddressType, DirectedAdvertisingType, FilterDuplicates, GapProcedure,
    OobDataType, OobDevice, OwnAddressType, ScanProcedure, ScanType, ScanningFilterPolicy, UndirectedAdvFilterPolicy,
};

use crate::bluetooth::error::BleError;
use crate::controller::ControllerAdapter;

// Advertising types for aci_gap_set_discoverable
#[allow(dead_code)]
pub const ADV_IND: u8 = 0x00; // Connectable undirected
#[allow(dead_code)]
pub const ADV_DIRECT_IND: u8 = 0x01; // Connectable directed
#[allow(dead_code)]
pub const ADV_SCAN_IND: u8 = 0x02; // Scannable undirected
#[allow(dead_code)]
pub const ADV_NONCONN_IND: u8 = 0x03; // Non-connectable undirected
#[allow(dead_code)]
pub const ADV_DIRECT_IND_LOW_DUTY: u8 = 0x04; // Connectable directed low duty cycle

const HCI_SCAN_TYPE_ACTIVE: u8 = 0x01;

/// Run a GAP discovery procedure through `ACI_GAP_START_SCAN`.
///
/// CubeWBA 1.10.0 dropped the legacy `aci_gap_start_{observation,limited,general}_discovery_proc`
/// commands, so the discovery procedures go through `aci_gap_start_scan` now
/// (declared as [`GapExtStartScan`] by `stm32wb-hci`, same opcode and layout).
#[allow(clippy::too_many_arguments)]
async fn start_scan(
    controller: &ControllerAdapter<'_>,
    procedure: ScanProcedure,
    scan_interval: u16,
    scan_window: u16,
    scan_type: u8,
    own_address_type: u8,
    filter_duplicates: bool,
    filter_policy: u8,
) -> Result<(), BleError> {
    let phy_param = |scan_type| -> Result<ScanParamPhy, BleError> {
        Ok(ScanParamPhy {
            scan_type: ScanType::try_from(scan_type).map_err(|_| BleError::InvalidParameter)?,
            scan_interval: ExtScanInterval::from_units(scan_interval).ok_or(BleError::InvalidParameter)?,
            scan_window: ExtScanWindow::from_units(scan_window).ok_or(BleError::InvalidParameter)?,
        })
    };
    // ACI_GAP_START_SCAN requires two PHY parameter records even when only
    // LE 1M is selected. The second record is ignored in that configuration.
    let scan_params = [phy_param(scan_type)?, phy_param(scan_type)?];

    GapExtStartScan::new(
        0,
        procedure,
        OwnAddressType::try_from(own_address_type).map_err(|_| BleError::InvalidParameter)?,
        if filter_duplicates {
            FilterDuplicates::Enabled
        } else {
            FilterDuplicates::Disabled
        },
        ScanDuration::CONTINUOUS,
        ScanPeriod::CONTINUOUS,
        ScanningFilterPolicy::try_from(filter_policy).map_err(|_| BleError::InvalidParameter)?,
        ScanningPhys::LE_1M,
        scan_params,
    )
    .exec(controller)
    .await?;

    Ok(())
}

/// Start advertising using `aci_gap_set_discoverable`
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
#[allow(clippy::too_many_arguments)]
pub async fn set_discoverable(
    controller: &ControllerAdapter<'_>,
    adv_type: u8,
    interval_min: u16,
    interval_max: u16,
    own_addr_type: u8,
    filter_policy: u8,
    local_name: Option<&[u8]>,
    service_uuid_bytes: Option<&[u8]>,
) -> Result<(), BleError> {
    trace!("set_discoverable: preparing to call ACI_GAP_SET_DISCOVERABLE");

    let cmd = GapSetDiscoverable::try_new(
        AdvertisingType::try_from(adv_type).map_err(|_| BleError::InvalidParameter)?,
        AdvInterval::from_units(interval_min).ok_or(BleError::InvalidParameter)?,
        AdvInterval::from_units(interval_max).ok_or(BleError::InvalidParameter)?,
        OwnAddressType::try_from(own_addr_type).map_err(|_| BleError::InvalidParameter)?,
        filter_policy,
        local_name.unwrap_or(&[]),
        service_uuid_bytes.unwrap_or(&[]),
        // slave_conn_interval_min/max (use default)
        stm32wb_hci::aci::durations::PreferredConnInterval::OMITTED,
        stm32wb_hci::aci::durations::PreferredConnInterval::OMITTED,
    )
    .map_err(|_| BleError::InvalidParameter)?;

    trace!(
        "set_discoverable: calling ACI_GAP_SET_DISCOVERABLE (type={}, int_min={}, int_max={})",
        adv_type, interval_min, interval_max
    );

    cmd.exec(controller)
        .await
        .inspect_err(|e| error!("aci_gap_set_discoverable failed: {}", e))
        .map_err(BleError::from)
}

/// Start directed connectable advertising.
///
/// Unlike [`set_discoverable`], which only supports undirected advertising
/// types, this is the ST command for high/low duty cycle connectable directed
/// advertising (Core Spec Vol 3, Part C, 9.3.3).
///
/// * `directed_type`: `ADV_DIRECT_IND` (0x01, high duty) or
///   `ADV_DIRECT_IND_LOW_DUTY` (0x04).
/// * `peer_addr_type`: 0x00 public / 0x01 random.
/// * High duty cycle directed advertising stops automatically after 1.28 s if
///   no connection is established.
#[allow(clippy::too_many_arguments)]
pub async fn set_direct_connectable(
    controller: &ControllerAdapter<'_>,
    own_addr_type: u8,
    directed_type: u8,
    peer_addr_type: u8,
    peer_addr: &[u8; 6],
    interval_min: u16,
    interval_max: u16,
) -> Result<(), BleError> {
    GapSetDirectConnectable::new(
        ConnectableOwnAddressType::try_from(own_addr_type).map_err(|_| BleError::InvalidParameter)?,
        DirectedAdvertisingType::try_from(directed_type).map_err(|_| BleError::InvalidParameter)?,
        AddressType::try_from(peer_addr_type).map_err(|_| BleError::InvalidParameter)?,
        BdAddr::new(*peer_addr),
        DirectAdvInterval::from_units(interval_min).ok_or(BleError::InvalidParameter)?,
        DirectAdvInterval::from_units(interval_max).ok_or(BleError::InvalidParameter)?,
    )
    .exec(controller)
    .await
    .inspect_err(|e| error!("aci_gap_set_direct_connectable failed: {}", e))
    .map_err(BleError::from)
}

/// Start the GAP observation procedure (passive or active scanning).
///
/// Advertising reports arrive as standard `HCI_LE_Advertising_Report` events
/// via `BLECB_Indication`.  This is the correct way to scan on the WBA BLE
/// stack — raw `HCI_LE_SET_SCAN_ENABLE` starts the radio but does not route
/// reports through the host layer.
///
/// CubeWBA 1.10.0 renamed the observation procedure `GAP_GENERIC_SCAN_PROC`;
/// `stm32wb-hci` models that as [`ScanProcedure::GenericScan`].
#[allow(clippy::too_many_arguments)]
pub async fn start_observation(
    controller: &ControllerAdapter<'_>,
    scan_interval: u16,
    scan_window: u16,
    scan_type: u8,
    own_address_type: u8,
    filter_duplicates: bool,
    filter_policy: u8,
) -> Result<(), BleError> {
    start_scan(
        controller,
        ScanProcedure::GenericScan,
        scan_interval,
        scan_window,
        scan_type,
        own_address_type,
        filter_duplicates,
        filter_policy,
    )
    .await
}

/// Start the GAP limited discovery procedure.
///
/// This uses active scanning and reports only peripherals in limited
/// discoverable mode.
pub async fn start_limited_discovery(
    controller: &ControllerAdapter<'_>,
    scan_interval: u16,
    scan_window: u16,
    own_address_type: u8,
    filter_duplicates: bool,
) -> Result<(), BleError> {
    start_scan(
        controller,
        ScanProcedure::LimitedDiscovery,
        scan_interval,
        scan_window,
        HCI_SCAN_TYPE_ACTIVE,
        own_address_type,
        filter_duplicates,
        0,
    )
    .await
}

/// Start the GAP general discovery procedure.
///
/// This uses active scanning and reports all discovered peripherals.
pub async fn start_general_discovery(
    controller: &ControllerAdapter<'_>,
    scan_interval: u16,
    scan_window: u16,
    own_address_type: u8,
    filter_duplicates: bool,
) -> Result<(), BleError> {
    start_scan(
        controller,
        ScanProcedure::GeneralDiscovery,
        scan_interval,
        scan_window,
        HCI_SCAN_TYPE_ACTIVE,
        own_address_type,
        filter_duplicates,
        0,
    )
    .await
}

/// Terminate a running GAP procedure.
pub async fn terminate_gap_proc(controller: &ControllerAdapter<'_>, procedure: GapProcedure) -> Result<(), BleError> {
    GapTerminateGapProc::new(procedure).exec(controller).await?;
    Ok(())
}

/// Stop the running observation/generic-scan procedure.
pub async fn stop_observation(controller: &ControllerAdapter<'_>) -> Result<(), BleError> {
    terminate_gap_proc(controller, to_gap_procedure(ScanProcedure::GenericScan)).await
}

pub(crate) fn to_gap_procedure(proc: ScanProcedure) -> GapProcedure {
    match proc {
        ScanProcedure::LimitedDiscovery => GapProcedure::LimitedDiscovery,
        ScanProcedure::GeneralDiscovery => GapProcedure::GeneralDiscovery,
        // CubeWBA 1.10.0 renamed the observation procedure GAP_GENERIC_SCAN_PROC.
        ScanProcedure::GenericScan => GapProcedure::GenericScan,
    }
}

/// Start undirected connectable advertising (ST privacy peripheral mode).
pub async fn set_undirected_connectable(
    controller: &ControllerAdapter<'_>,
    interval_min: u16,
    interval_max: u16,
    own_addr_type: u8,
    filter_policy: u8,
) -> Result<(), BleError> {
    GapSetUndirectedConnectable::new(
        AdvInterval::from_units(interval_min).ok_or(BleError::InvalidParameter)?,
        AdvInterval::from_units(interval_max).ok_or(BleError::InvalidParameter)?,
        ConnectableOwnAddressType::try_from(own_addr_type).map_err(|_| BleError::InvalidParameter)?,
        UndirectedAdvFilterPolicy::try_from(filter_policy).map_err(|_| BleError::InvalidParameter)?,
    )
    .exec(controller)
    .await
    .inspect_err(|e| error!("aci_gap_set_undirected_connectable failed: {}", e))
    .map_err(BleError::from)
}

/// Push AD payload after [`set_undirected_connectable`].
pub async fn update_adv_data(controller: &ControllerAdapter<'_>, adv_data: &[u8]) -> Result<(), BleError> {
    if adv_data.is_empty() || adv_data.len() > 31 {
        return Err(BleError::InvalidParameter);
    }
    let cmd = GapUpdateAdvData::try_new(adv_data).map_err(|_| BleError::InvalidParameter)?;
    cmd.exec(controller).await?;
    Ok(())
}

/// Stop advertising using `aci_gap_set_non_discoverable`
pub async fn set_non_discoverable(controller: &ControllerAdapter<'_>) -> Result<(), BleError> {
    GapSetNonDiscoverable::new().exec(controller).await?;
    Ok(())
}

// ===== Secure Connections out-of-band (OOB) =====

/// Generate this device's Secure Connections OOB material.
///
/// `ACI_GAP_SET_OOB_DATA` with `Device_Type = Local` and a zero length, which
/// makes the host derive the `r`/`c` pair from its local P-256 keypair.
///
/// The keypair has to exist first, otherwise the host answers
/// `SMP_SC_Local_Public_Key_Unavailable` (0x06) — issue
/// [`HCI::le_read_local_p256_public_key`](crate::HCI::le_read_local_p256_public_key) beforehand.
pub async fn generate_local_oob_data(controller: &ControllerAdapter<'_>) -> Result<(), BleError> {
    // A zero `oob_data_len` makes the stack derive the material; the address
    // and payload bytes are not read in that case.
    GapSetOobData::new(
        OobDevice::Local,
        AddressType::Public,
        BdAddr::new([0; 6]),
        OobDataType::TemporaryKey,
        0,
        [0; 16],
    )
    .exec(controller)
    .await?;
    Ok(())
}

/// Read back one half of this device's Secure Connections OOB material.
///
/// The address return parameters of `ACI_GAP_GET_OOB_DATA` describe this device and
/// are not needed by a caller that already knows its own identity address, so
/// they are discarded here.
pub async fn read_local_oob_data(controller: &ControllerAdapter<'_>, kind: OobDataType) -> Result<[u8; 16], BleError> {
    let resp = GapGetOobData::new(kind).exec(controller).await?;
    if resp.oob_data_len as usize == resp.oob_data.len() {
        Ok(resp.oob_data)
    } else {
        Err(BleError::InvalidParameter)
    }
}

/// Program the peer's Secure Connections OOB material into the host security
/// database, so pairing with `address` can use it.
pub async fn set_remote_oob_data(
    controller: &ControllerAdapter<'_>,
    address_type: u8,
    address: &[u8; 6],
    kind: OobDataType,
    data: &[u8; 16],
) -> Result<(), BleError> {
    GapSetOobData::new(
        OobDevice::Remote,
        AddressType::try_from(address_type).map_err(|_| BleError::InvalidParameter)?,
        BdAddr::new(*address),
        kind,
        data.len() as u8,
        *data,
    )
    .exec(controller)
    .await?;
    Ok(())
}
