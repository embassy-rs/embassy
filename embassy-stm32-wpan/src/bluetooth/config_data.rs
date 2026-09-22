//! Host configuration data (`ACI_HAL_WRITE_CONFIG_DATA` / `ACI_HAL_READ_CONFIG_DATA`).
//!
//! These offsets carry the stack's identity root, encryption root, random
//! address and Secure Connections key-type selection. The ACI is wrapped here so
//! applications do not have to declare the ST C symbols themselves.

use crate::bluetooth::error::BleError;
use crate::bluetooth::hci::Status;

/// `BLE_STATUS_SUCCESS`.
const BLE_STATUS_SUCCESS: u8 = 0x00;

unsafe extern "C" {
    #[link_name = "ACI_HAL_WRITE_CONFIG_DATA"]
    fn aci_hal_write_config_data(offset: u8, length: u8, value: *const u8) -> u8;

    #[link_name = "ACI_HAL_READ_CONFIG_DATA"]
    fn aci_hal_read_config_data(offset: u8, length: *mut u8, value: *mut u8) -> u8;
}

/// Write `value` at `offset` in the host's configuration data.
pub fn write(offset: u8, value: &[u8]) -> Result<(), BleError> {
    let status = unsafe { aci_hal_write_config_data(offset, value.len() as u8, value.as_ptr()) };
    if status == BLE_STATUS_SUCCESS {
        Ok(())
    } else {
        Err(BleError::CommandFailed(Status::from_u8(status)))
    }
}

/// Read configuration data at `offset` into `out`, filling all of it.
///
/// Fails with [`BleError::InvalidParameter`] when the host returns fewer bytes
/// than were asked for.
pub fn read(offset: u8, out: &mut [u8]) -> Result<(), BleError> {
    let mut len = out.len() as u8;
    let status = unsafe { aci_hal_read_config_data(offset, &mut len, out.as_mut_ptr()) };
    if status != BLE_STATUS_SUCCESS {
        return Err(BleError::CommandFailed(Status::from_u8(status)));
    }
    if len as usize != out.len() {
        return Err(BleError::InvalidParameter);
    }
    Ok(())
}
