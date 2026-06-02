//! Minimal GATT client API for core BLE workflows.
//!
//! This module intentionally focuses on the common commands needed for
//! bring-up: service/characteristic discovery and basic read/write flows.

use stm32_bindings::ble;

use crate::bluetooth::error::BleError;
use crate::bluetooth::hci::Status;

const BLE_STATUS_SUCCESS: u8 = 0x00;

/// Minimal GATT client helper.
pub struct GattClient {
    _private: (),
}

impl GattClient {
    pub(crate) const fn new() -> Self {
        Self { _private: () }
    }

    /// Discover all primary services on the peer.
    pub fn discover_all_primary_services(&self, conn_handle: u16) -> Result<(), BleError> {
        let status = unsafe { ble::aci_gatt_disc_all_primary_services(conn_handle) };
        check_status(status)
    }

    /// Discover all characteristics in a service handle range.
    pub fn discover_all_characteristics(
        &self,
        conn_handle: u16,
        service_start_handle: u16,
        service_end_handle: u16,
    ) -> Result<(), BleError> {
        let status = unsafe {
            ble::aci_gatt_disc_all_char_of_service(conn_handle, service_start_handle, service_end_handle)
        };
        check_status(status)
    }

    /// Discover all descriptors for a characteristic value handle.
    pub fn discover_all_descriptors(
        &self,
        conn_handle: u16,
        char_value_handle: u16,
        char_end_handle: u16,
    ) -> Result<(), BleError> {
        let status = unsafe { ble::aci_gatt_disc_all_char_desc(conn_handle, char_value_handle, char_end_handle) };
        check_status(status)
    }

    /// Read a characteristic or descriptor value.
    pub fn read_value(&self, conn_handle: u16, attr_handle: u16) -> Result<(), BleError> {
        let status = unsafe { ble::aci_gatt_read_char_value(conn_handle, attr_handle) };
        check_status(status)
    }

    /// Write a characteristic or descriptor value with response.
    pub fn write_value(&self, conn_handle: u16, attr_handle: u16, value: &[u8]) -> Result<(), BleError> {
        if value.len() > 255 {
            return Err(BleError::InvalidParameter);
        }
        let status =
            unsafe { ble::aci_gatt_write_char_value(conn_handle, attr_handle, value.len() as u8, value.as_ptr()) };
        check_status(status)
    }

    /// Write a characteristic or descriptor value without response.
    pub fn write_without_response(&self, conn_handle: u16, attr_handle: u16, value: &[u8]) -> Result<(), BleError> {
        if value.len() > 255 {
            return Err(BleError::InvalidParameter);
        }
        let status =
            unsafe { ble::aci_gatt_write_without_resp(conn_handle, attr_handle, value.len() as u8, value.as_ptr()) };
        check_status(status)
    }

    /// Confirm an indication from the peer.
    pub fn confirm_indication(&self, conn_handle: u16) -> Result<(), BleError> {
        let status = unsafe { ble::aci_gatt_confirm_indication(conn_handle) };
        check_status(status)
    }
}

fn check_status(status: u8) -> Result<(), BleError> {
    if status == BLE_STATUS_SUCCESS {
        Ok(())
    } else {
        Err(BleError::CommandFailed(Status::from_u8(status)))
    }
}
