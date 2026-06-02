//! Minimal GATT client API for core BLE workflows.
//!
//! This module intentionally focuses on the common commands needed for
//! bring-up: service/characteristic discovery and basic read/write flows.

use stm32_bindings::ble;

use crate::bluetooth::error::BleError;
use crate::bluetooth::gatt::types::Uuid;
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

    /// Discover primary services by UUID.
    pub fn discover_primary_service_by_uuid(&self, conn_handle: u16, uuid: Uuid) -> Result<(), BleError> {
        let (uuid_type, uuid_union) = to_uuid_union(uuid);
        let status = unsafe { ble::aci_gatt_disc_primary_service_by_uuid(conn_handle, uuid_type, &uuid_union) };
        check_status(status)
    }

    /// Discover all characteristics in a service handle range.
    ///
    /// Expected event sequence:
    /// - zero or more `GattClientEvent::CharacteristicFound`
    /// - final `GattClientEvent::ProcedureComplete`
    pub fn discover_all_characteristics(
        &self,
        conn_handle: u16,
        service_start_handle: u16,
        service_end_handle: u16,
    ) -> Result<(), BleError> {
        let status =
            unsafe { ble::aci_gatt_disc_all_char_of_service(conn_handle, service_start_handle, service_end_handle) };
        check_status(status)
    }

    /// Discover all descriptors for a characteristic value handle.
    ///
    /// Expected event sequence:
    /// - zero or more `GattClientEvent::DescriptorFound`
    /// - final `GattClientEvent::ProcedureComplete`
    pub fn discover_all_descriptors(
        &self,
        conn_handle: u16,
        char_value_handle: u16,
        char_end_handle: u16,
    ) -> Result<(), BleError> {
        let status = unsafe { ble::aci_gatt_disc_all_char_desc(conn_handle, char_value_handle, char_end_handle) };
        check_status(status)
    }

    /// Discover included services in a primary service handle range.
    pub fn discover_included_services(
        &self,
        conn_handle: u16,
        service_start_handle: u16,
        service_end_handle: u16,
    ) -> Result<(), BleError> {
        let status =
            unsafe { ble::aci_gatt_find_included_services(conn_handle, service_start_handle, service_end_handle) };
        check_status(status)
    }

    /// Discover characteristics in a service range matching a specific UUID.
    pub fn discover_characteristics_by_uuid(
        &self,
        conn_handle: u16,
        service_start_handle: u16,
        service_end_handle: u16,
        uuid: Uuid,
    ) -> Result<(), BleError> {
        let (uuid_type, uuid_union) = to_uuid_union(uuid);
        let status = unsafe {
            ble::aci_gatt_disc_char_by_uuid(
                conn_handle,
                service_start_handle,
                service_end_handle,
                uuid_type,
                &uuid_union,
            )
        };
        check_status(status)
    }

    /// Read a characteristic or descriptor value.
    pub fn read_value(&self, conn_handle: u16, attr_handle: u16) -> Result<(), BleError> {
        let status = unsafe { ble::aci_gatt_read_char_value(conn_handle, attr_handle) };
        check_status(status)
    }

    /// Read one or more characteristics by UUID in a handle range.
    pub fn read_using_characteristic_uuid(
        &self,
        conn_handle: u16,
        start_handle: u16,
        end_handle: u16,
        uuid: Uuid,
    ) -> Result<(), BleError> {
        let (uuid_type, uuid_union) = to_uuid_union(uuid);
        let status = unsafe {
            ble::aci_gatt_read_using_char_uuid(conn_handle, start_handle, end_handle, uuid_type, &uuid_union)
        };
        check_status(status)
    }

    /// Read multiple characteristic values in one ATT procedure.
    ///
    /// `attr_handles` must contain 2..=126 handles.
    pub fn read_multiple_values(&self, conn_handle: u16, attr_handles: &[u16]) -> Result<(), BleError> {
        let entries = to_handle_entries(attr_handles)?;
        let status =
            unsafe { ble::aci_gatt_read_multiple_char_value(conn_handle, entries.len() as u8, entries.as_ptr()) };
        check_status(status)
    }

    /// Read multiple variable-length characteristic values in one ATT procedure.
    ///
    /// `attr_handles` must contain 2..=126 handles.
    pub fn read_multiple_variable_values(&self, conn_handle: u16, attr_handles: &[u16]) -> Result<(), BleError> {
        let entries = to_handle_entries(attr_handles)?;
        let status =
            unsafe { ble::aci_gatt_read_multiple_var_char_value(conn_handle, entries.len() as u8, entries.as_ptr()) };
        check_status(status)
    }

    /// Read a long characteristic/descriptor value from a given offset.
    pub fn read_long_value(&self, conn_handle: u16, attr_handle: u16, offset: u16) -> Result<(), BleError> {
        let status = unsafe { ble::aci_gatt_read_long_char_value(conn_handle, attr_handle, offset) };
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

    /// Read a descriptor by handle (semantic alias of `read_value`).
    pub fn read_descriptor(&self, conn_handle: u16, descriptor_handle: u16) -> Result<(), BleError> {
        self.read_value(conn_handle, descriptor_handle)
    }

    /// Write a descriptor by handle (semantic alias of `write_value`).
    pub fn write_descriptor(&self, conn_handle: u16, descriptor_handle: u16, value: &[u8]) -> Result<(), BleError> {
        self.write_value(conn_handle, descriptor_handle, value)
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

    /// Write a long characteristic/descriptor value from a given offset.
    pub fn write_long_value(
        &self,
        conn_handle: u16,
        attr_handle: u16,
        offset: u16,
        value: &[u8],
    ) -> Result<(), BleError> {
        if value.len() > 255 {
            return Err(BleError::InvalidParameter);
        }
        let status = unsafe {
            ble::aci_gatt_write_long_char_value(conn_handle, attr_handle, offset, value.len() as u8, value.as_ptr())
        };
        check_status(status)
    }

    /// Reliably write a characteristic/descriptor value from a given offset.
    pub fn write_reliable(
        &self,
        conn_handle: u16,
        attr_handle: u16,
        offset: u16,
        value: &[u8],
    ) -> Result<(), BleError> {
        if value.len() > 255 {
            return Err(BleError::InvalidParameter);
        }
        let status = unsafe {
            ble::aci_gatt_write_char_reliable(conn_handle, attr_handle, offset, value.len() as u8, value.as_ptr())
        };
        check_status(status)
    }

    /// Stage one fragment for a prepare-write sequence.
    pub fn prepare_write(&self, conn_handle: u16, attr_handle: u16, offset: u16, value: &[u8]) -> Result<(), BleError> {
        if value.len() > 255 {
            return Err(BleError::InvalidParameter);
        }
        let status = unsafe {
            ble::aci_att_prepare_write_req(conn_handle, attr_handle, offset, value.len() as u8, value.as_ptr())
        };
        check_status(status)
    }

    /// Execute or cancel all prepared writes for the connection.
    pub fn execute_write(&self, conn_handle: u16, execute: bool) -> Result<(), BleError> {
        let status = unsafe { ble::aci_att_execute_write_req(conn_handle, execute as u8) };
        check_status(status)
    }

    /// Request ATT exchange configuration procedure.
    pub fn exchange_configuration(&self, conn_handle: u16) -> Result<(), BleError> {
        let status = unsafe { ble::aci_gatt_exchange_config(conn_handle) };
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

fn to_uuid_union(uuid: Uuid) -> (u8, ble::UUID_t) {
    match uuid {
        Uuid::Uuid16(v) => (0x01, ble::UUID_t { UUID_16: v }),
        Uuid::Uuid128(v) => (0x02, ble::UUID_t { UUID_128: v }),
    }
}

fn to_handle_entries(attr_handles: &[u16]) -> Result<heapless::Vec<ble::Handle_Entry_t, 126>, BleError> {
    if !(2..=126).contains(&attr_handles.len()) {
        return Err(BleError::InvalidParameter);
    }

    let mut out = heapless::Vec::<ble::Handle_Entry_t, 126>::new();
    for &handle in attr_handles {
        out.push(ble::Handle_Entry_t { Handle: handle })
            .map_err(|_| BleError::InvalidParameter)?;
    }
    Ok(out)
}
