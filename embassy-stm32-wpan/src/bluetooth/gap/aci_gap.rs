//! GAP ACI Commands
//!
//! Higher-level GAP functions that use ACI (Application Command Interface)
//! instead of raw HCI commands. These provide more integrated functionality.

use stm32_bindings::ble::{
    Scan_Param_Phy_t, aci_gap_get_oob_data, aci_gap_set_direct_connectable, aci_gap_set_discoverable,
    aci_gap_set_non_discoverable, aci_gap_set_oob_data, aci_gap_set_undirected_connectable, aci_gap_start_scan,
    aci_gap_terminate_gap_proc, aci_gap_update_adv_data,
};

use crate::bluetooth::error::BleError;
use crate::bluetooth::hci::Status;

const BLE_STATUS_SUCCESS: u8 = 0x00;

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

/// ST `GAP_RESOLVABLE_PRIVATE_ADDR` — use with controller privacy for advertising.
pub const GAP_RESOLVABLE_PRIVATE_ADDR: u8 = 0x02;

// Advertising filter policy
#[allow(dead_code)]
pub const NO_WHITE_LIST_USE: u8 = 0x00;
#[allow(dead_code)]
pub const WHITE_LIST_FOR_ONLY_SCAN: u8 = 0x01;
#[allow(dead_code)]
pub const WHITE_LIST_FOR_ONLY_CONN: u8 = 0x02;
#[allow(dead_code)]
pub const WHITE_LIST_FOR_ALL: u8 = 0x03;

// GAP procedure codes for ACI_GAP_TERMINATE_GAP_PROC.
pub const GAP_LIMITED_DISCOVERY_PROC: u8 = 0x01;
pub const GAP_GENERAL_DISCOVERY_PROC: u8 = 0x02;
pub const GAP_OBSERVATION_PROC: u8 = 0x80;

const HCI_SCAN_TYPE_ACTIVE: u8 = 0x01;
const HCI_SCANNING_PHYS_LE_1M: u8 = 0x01;

/// Run a GAP discovery procedure through `aci_gap_start_scan`.
///
/// 1.10.0 dropped the legacy `aci_gap_start_{observation,limited,general}_discovery_proc`
/// commands, so the discovery procedures go through `aci_gap_start_scan` now.
fn start_scan(
    procedure: u8,
    scan_interval: u16,
    scan_window: u16,
    scan_type: u8,
    own_address_type: u8,
    filter_duplicates: bool,
    filter_policy: u8,
) -> Result<(), BleError> {
    let scan_param = Scan_Param_Phy_t {
        Scan_Type: scan_type,
        Scan_Interval: scan_interval,
        Scan_Window: scan_window,
    };
    // ACI_GAP_START_SCAN requires two PHY parameter records even when only
    // LE 1M is selected. The second record is ignored in that configuration.
    let scan_params = [scan_param; 2];

    let status = unsafe {
        aci_gap_start_scan(
            0,
            procedure,
            own_address_type,
            filter_duplicates as u8,
            0,
            0,
            filter_policy,
            HCI_SCANNING_PHYS_LE_1M,
            scan_params.as_ptr(),
        )
    };

    if status == BLE_STATUS_SUCCESS {
        Ok(())
    } else {
        Err(BleError::CommandFailed(Status::from_u8(status)))
    }
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
        defmt::trace!(
            "set_discoverable: calling ACI_GAP_SET_DISCOVERABLE (type={}, int_min={}, int_max={})",
            adv_type,
            interval_min,
            interval_max
        );

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
            0, // slave_conn_interval_min (use default)
            0, // slave_conn_interval_max (use default)
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
pub fn set_direct_connectable(
    own_addr_type: u8,
    directed_type: u8,
    peer_addr_type: u8,
    peer_addr: &[u8; 6],
    interval_min: u16,
    interval_max: u16,
) -> Result<(), BleError> {
    unsafe {
        let status = aci_gap_set_direct_connectable(
            own_addr_type,
            directed_type,
            peer_addr_type,
            peer_addr.as_ptr(),
            interval_min,
            interval_max,
        );

        if status == BLE_STATUS_SUCCESS {
            #[cfg(feature = "defmt")]
            defmt::info!("aci_gap_set_direct_connectable succeeded");
            Ok(())
        } else {
            #[cfg(feature = "defmt")]
            defmt::error!("aci_gap_set_direct_connectable failed: 0x{:02X}", status);
            Err(BleError::CommandFailed(Status::from_u8(status)))
        }
    }
}

/// Start the GAP observation procedure (passive or active scanning).
///
/// Advertising reports arrive as standard `HCI_LE_Advertising_Report` events
/// via `BLECB_Indication`.  This is the correct way to scan on the WBA BLE
/// stack — raw `HCI_LE_SET_SCAN_ENABLE` starts the radio but does not route
/// reports through the host layer.
pub fn start_observation(
    scan_interval: u16,
    scan_window: u16,
    scan_type: u8,
    own_address_type: u8,
    filter_duplicates: bool,
    filter_policy: u8,
) -> Result<(), BleError> {
    start_scan(
        GAP_OBSERVATION_PROC,
        scan_interval,
        scan_window,
        scan_type,
        own_address_type,
        filter_duplicates,
        filter_policy,
    )
}

/// Start the GAP limited discovery procedure.
///
/// This uses active scanning and reports only peripherals in limited
/// discoverable mode.
pub fn start_limited_discovery(
    scan_interval: u16,
    scan_window: u16,
    own_address_type: u8,
    filter_duplicates: bool,
) -> Result<(), BleError> {
    start_scan(
        GAP_LIMITED_DISCOVERY_PROC,
        scan_interval,
        scan_window,
        HCI_SCAN_TYPE_ACTIVE,
        own_address_type,
        filter_duplicates,
        0,
    )
}

/// Start the GAP general discovery procedure.
///
/// This uses active scanning and reports all discovered peripherals.
pub fn start_general_discovery(
    scan_interval: u16,
    scan_window: u16,
    own_address_type: u8,
    filter_duplicates: bool,
) -> Result<(), BleError> {
    start_scan(
        GAP_GENERAL_DISCOVERY_PROC,
        scan_interval,
        scan_window,
        HCI_SCAN_TYPE_ACTIVE,
        own_address_type,
        filter_duplicates,
        0,
    )
}

/// Terminate a running GAP procedure by procedure code.
pub fn terminate_gap_proc(procedure_code: u8) -> Result<(), BleError> {
    let status = unsafe { aci_gap_terminate_gap_proc(procedure_code) };
    if status == BLE_STATUS_SUCCESS {
        Ok(())
    } else {
        Err(BleError::CommandFailed(Status::from_u8(status)))
    }
}

/// Stop the running GAP observation procedure (`procedure_code = 0x80`).
pub fn stop_observation() -> Result<(), BleError> {
    terminate_gap_proc(GAP_OBSERVATION_PROC)
}

/// Start undirected connectable advertising (ST privacy peripheral mode).
pub fn set_undirected_connectable(
    interval_min: u16,
    interval_max: u16,
    own_address_type: u8,
    filter_policy: u8,
) -> Result<(), BleError> {
    unsafe {
        let status = aci_gap_set_undirected_connectable(interval_min, interval_max, own_address_type, filter_policy);
        if status == BLE_STATUS_SUCCESS {
            #[cfg(feature = "defmt")]
            defmt::info!("aci_gap_set_undirected_connectable succeeded");
            Ok(())
        } else {
            #[cfg(feature = "defmt")]
            defmt::error!("aci_gap_set_undirected_connectable failed: 0x{:02X}", status);
            Err(BleError::CommandFailed(Status::from_u8(status)))
        }
    }
}

/// Push AD payload after [`set_undirected_connectable`].
pub fn update_adv_data(adv_data: &[u8]) -> Result<(), BleError> {
    if adv_data.is_empty() || adv_data.len() > 31 {
        return Err(BleError::InvalidParameter);
    }
    unsafe {
        let status = aci_gap_update_adv_data(adv_data.len() as u8, adv_data.as_ptr());
        if status == BLE_STATUS_SUCCESS {
            Ok(())
        } else {
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

// ===== Secure Connections out-of-band (OOB) =====

/// `Device_Type` for [`aci_gap_set_oob_data`]: this device's own material.
const OOB_DEVICE_TYPE_LOCAL: u8 = 0x00;
/// `Device_Type` for [`aci_gap_set_oob_data`]: the peer's material.
const OOB_DEVICE_TYPE_REMOTE: u8 = 0x01;

/// Secure Connections OOB material to read with [`read_local_oob_data`], or to
/// program with [`set_remote_oob_data`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum OobDataType {
    /// `0x01`: SC Random (`r`).
    ScRandom = 0x01,
    /// `0x02`: SC Confirm (`c`).
    ScConfirm = 0x02,
}

/// Generate this device's Secure Connections OOB material.
///
/// `ACI_GAP_SET_OOB_DATA` with `Device_Type = Local` and a zero length, which
/// makes the host derive the `r`/`c` pair from its local P-256 keypair.
///
/// The keypair has to exist first, otherwise the host answers
/// `SMP_SC_Local_Public_Key_Unavailable` (0x06) — issue
/// [`CommandSender::le_read_local_p256_public_key`] beforehand.
///
/// [`CommandSender::le_read_local_p256_public_key`]: crate::bluetooth::hci::CommandSender::le_read_local_p256_public_key
pub fn generate_local_oob_data() -> Result<(), BleError> {
    let status = unsafe { aci_gap_set_oob_data(OOB_DEVICE_TYPE_LOCAL, 0, core::ptr::null(), 0, 0, core::ptr::null()) };
    if status == BLE_STATUS_SUCCESS {
        Ok(())
    } else {
        Err(BleError::CommandFailed(Status::from_u8(status)))
    }
}

/// Read back one half of this device's Secure Connections OOB material.
///
/// The address out-parameters of `ACI_GAP_GET_OOB_DATA` describe this device and
/// are not needed by a caller that already knows its own identity address, so
/// they are discarded here.
pub fn read_local_oob_data(kind: OobDataType) -> Result<[u8; 16], BleError> {
    let mut address_type = 0u8;
    let mut address = [0u8; 6];
    let mut len = 16u8;
    let mut data = [0u8; 16];

    let status = unsafe {
        aci_gap_get_oob_data(
            kind as u8,
            &mut address_type,
            address.as_mut_ptr(),
            &mut len,
            data.as_mut_ptr(),
        )
    };
    if status == BLE_STATUS_SUCCESS && len as usize == data.len() {
        Ok(data)
    } else if status == BLE_STATUS_SUCCESS {
        Err(BleError::InvalidParameter)
    } else {
        Err(BleError::CommandFailed(Status::from_u8(status)))
    }
}

/// Program the peer's Secure Connections OOB material into the host security
/// database, so pairing with `address` can use it.
pub fn set_remote_oob_data(
    address_type: u8,
    address: &[u8; 6],
    kind: OobDataType,
    data: &[u8; 16],
) -> Result<(), BleError> {
    let status = unsafe {
        aci_gap_set_oob_data(
            OOB_DEVICE_TYPE_REMOTE,
            address_type,
            address.as_ptr(),
            kind as u8,
            data.len() as u8,
            data.as_ptr(),
        )
    };
    if status == BLE_STATUS_SUCCESS {
        Ok(())
    } else {
        Err(BleError::CommandFailed(Status::from_u8(status)))
    }
}
