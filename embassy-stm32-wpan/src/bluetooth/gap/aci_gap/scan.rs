//! GAP scanning ACI commands (observer and central roles).
//!
//! The peripheral-only BLE stack has no scanning, so this module is only
//! built for the other stacks.

use bt_hci::cmd::{AsyncCmd, SyncCmd};
use stm32wb_hci::aci::durations::{ExtScanInterval, ExtScanWindow, ScanDuration, ScanPeriod};
use stm32wb_hci::aci::flags::ScanningPhys;
use stm32wb_hci::aci::gap::{GapExtStartScan, GapTerminateGapProc, ScanParamPhy};
use stm32wb_hci::aci::values::{
    FilterDuplicates, GapProcedure, OwnAddressType, ScanProcedure, ScanType, ScanningFilterPolicy,
};

use crate::bluetooth::error::BleError;
use crate::controller::ControllerAdapter;

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
