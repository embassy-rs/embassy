//! GATT Server implementation
//!
//! This is a thin Rust wrapper around ST's GATT server implementation.

use super::types::{
    CharProperties, CharacteristicHandle, GattEventMask, SecurityPermissions, ServiceHandle, ServiceType, Uuid,
    UuidType,
};
use crate::wba::bindings::ble;
use crate::wba::error::BleError;

// The C library exports uppercase function names
#[allow(non_camel_case_types)]
type tBleStatus = u8;

#[link(name = "stm32wba_ble_stack_basic")]
unsafe extern "C" {
    #[link_name = "ACI_GATT_INIT"]
    fn aci_gatt_init() -> tBleStatus;

    #[link_name = "ACI_GATT_ADD_SERVICE"]
    fn aci_gatt_add_service(
        service_uuid_type: u8,
        service_uuid: *const u8,
        service_type: u8,
        max_attribute_records: u8,
        service_handle: *mut u16,
    ) -> tBleStatus;

    #[link_name = "ACI_GATT_ADD_CHAR"]
    fn aci_gatt_add_char(
        service_handle: u16,
        char_uuid_type: u8,
        char_uuid: *const u8,
        char_value_length: u16,
        char_properties: u8,
        security_permissions: u8,
        gatt_evt_mask: u8,
        enc_key_size: u8,
        is_variable: u8,
        char_handle: *mut u16,
    ) -> tBleStatus;

    #[link_name = "ACI_GATT_UPDATE_CHAR_VALUE"]
    fn aci_gatt_update_char_value(
        service_handle: u16,
        char_handle: u16,
        val_offset: u8,
        char_value_length: u8,
        char_value: *const u8,
    ) -> tBleStatus;

    #[link_name = "ACI_GATT_UPDATE_CHAR_VALUE_EXT"]
    fn aci_gatt_update_char_value_ext(
        conn_handle: u16,
        service_handle: u16,
        char_handle: u16,
        update_type: u8,
        char_length: u16,
        value_offset: u16,
        value_length: u8,
        value: *const u8,
    ) -> tBleStatus;

    #[link_name = "ACI_GATT_READ_HANDLE_VALUE"]
    fn aci_gatt_read_handle_value(
        attr_handle: u16,
        offset: u16,
        value_length_requested: u16,
        value_length: *mut u16,
        value_offset: *mut u16,
        value: *mut u8,
    ) -> tBleStatus;

    #[link_name = "ACI_GATT_SET_EVENT_MASK"]
    fn aci_gatt_set_event_mask(event_mask: u32) -> tBleStatus;
}

/// Update type for aci_gatt_update_char_value_ext
#[allow(dead_code)]
mod update_type {
    /// Update locally only, do not notify/indicate
    pub const LOCAL_ONLY: u8 = 0x00;
    /// Send notification to connected client
    pub const NOTIFICATION: u8 = 0x01;
    /// Send indication to connected client
    pub const INDICATION: u8 = 0x02;
}

const BLE_STATUS_SUCCESS: u8 = 0x00;

/// GATT Server
///
/// Provides methods for creating and managing GATT services and characteristics.
pub struct GattServer {
    initialized: bool,
}

impl GattServer {
    /// Create a new GATT server instance
    ///
    /// Note: You must call `init()` before adding services or characteristics.
    pub fn new() -> Self {
        Self { initialized: false }
    }

    /// Initialize the GATT server
    ///
    /// This adds the GATT service with Service Changed Characteristic.
    /// Must be called before using any GATT features.
    pub fn init(&mut self) -> Result<(), BleError> {
        if self.initialized {
            return Ok(());
        }

        unsafe {
            let status = aci_gatt_init();
            if status == BLE_STATUS_SUCCESS {
                self.initialized = true;
                Ok(())
            } else {
                Err(BleError::CommandFailed(crate::wba::hci::types::Status::from_u8(status)))
            }
        }
    }

    /// Add a service to the GATT database
    ///
    /// # Parameters
    ///
    /// - `uuid`: Service UUID (16-bit or 128-bit)
    /// - `service_type`: Primary or secondary service
    /// - `max_attribute_records`: Maximum number of attribute records (characteristics + descriptors)
    ///
    /// # Returns
    ///
    /// Service handle on success
    ///
    /// # Example
    ///
    /// ```no_run
    /// let service_uuid = Uuid::from_u16(0x1234);
    /// let handle = gatt.add_service(service_uuid, ServiceType::Primary, 10)?;
    /// ```
    pub fn add_service(
        &mut self,
        uuid: Uuid,
        service_type: ServiceType,
        max_attribute_records: u8,
    ) -> Result<ServiceHandle, BleError> {
        if !self.initialized {
            return Err(BleError::NotInitialized);
        }

        let mut service_handle: u16 = 0;

        unsafe {
            let status = match uuid {
                Uuid::Uuid16(uuid16) => {
                    let uuid_bytes = uuid16.to_le_bytes();
                    aci_gatt_add_service(
                        UuidType::Uuid16 as u8,
                        uuid_bytes.as_ptr() as *const _,
                        service_type as u8,
                        max_attribute_records,
                        &mut service_handle,
                    )
                }
                Uuid::Uuid128(uuid128) => aci_gatt_add_service(
                    UuidType::Uuid128 as u8,
                    uuid128.as_ptr() as *const _,
                    service_type as u8,
                    max_attribute_records,
                    &mut service_handle,
                ),
            };

            if status == BLE_STATUS_SUCCESS {
                Ok(ServiceHandle(service_handle))
            } else {
                Err(BleError::CommandFailed(crate::wba::hci::types::Status::from_u8(status)))
            }
        }
    }

    /// Add a characteristic to a service
    ///
    /// # Parameters
    ///
    /// - `service_handle`: Handle of the service
    /// - `uuid`: Characteristic UUID (16-bit or 128-bit)
    /// - `value_length`: Maximum length of characteristic value
    /// - `properties`: Characteristic properties (read, write, notify, etc.)
    /// - `security`: Security permissions
    /// - `event_mask`: Events to be notified about
    /// - `encryption_key_size`: Encryption key size (7-16 bytes, 0 = no encryption)
    /// - `is_variable`: If true, the characteristic value can have variable length
    ///
    /// # Returns
    ///
    /// Characteristic handle on success
    ///
    /// # Example
    ///
    /// ```no_run
    /// let char_uuid = Uuid::from_u16(0x2A00); // Device Name
    /// let props = CharProperties::READ | CharProperties::WRITE;
    /// let handle = gatt.add_characteristic(
    ///     service_handle,
    ///     char_uuid,
    ///     20,
    ///     props,
    ///     SecurityPermissions::NONE,
    ///     GattEventMask::ATTRIBUTE_MODIFIED,
    ///     0,
    ///     true,
    /// )?;
    /// ```
    pub fn add_characteristic(
        &mut self,
        service_handle: ServiceHandle,
        uuid: Uuid,
        value_length: u16,
        properties: CharProperties,
        security: SecurityPermissions,
        event_mask: GattEventMask,
        encryption_key_size: u8,
        is_variable: bool,
    ) -> Result<CharacteristicHandle, BleError> {
        if !self.initialized {
            return Err(BleError::NotInitialized);
        }

        let mut char_handle: u16 = 0;

        unsafe {
            let status = match uuid {
                Uuid::Uuid16(uuid16) => {
                    let uuid_bytes = uuid16.to_le_bytes();
                    aci_gatt_add_char(
                        service_handle.0,
                        UuidType::Uuid16 as u8,
                        uuid_bytes.as_ptr() as *const _,
                        value_length,
                        properties.0,
                        security.0,
                        event_mask.0,
                        encryption_key_size,
                        is_variable as u8,
                        &mut char_handle,
                    )
                }
                Uuid::Uuid128(uuid128) => aci_gatt_add_char(
                    service_handle.0,
                    UuidType::Uuid128 as u8,
                    uuid128.as_ptr() as *const _,
                    value_length,
                    properties.0,
                    security.0,
                    event_mask.0,
                    encryption_key_size,
                    is_variable as u8,
                    &mut char_handle,
                ),
            };

            if status == BLE_STATUS_SUCCESS {
                Ok(CharacteristicHandle(char_handle))
            } else {
                Err(BleError::CommandFailed(crate::wba::hci::types::Status::from_u8(status)))
            }
        }
    }

    /// Update a characteristic value
    ///
    /// # Parameters
    ///
    /// - `service_handle`: Handle of the service
    /// - `char_handle`: Handle of the characteristic
    /// - `offset`: Offset at which to write the value
    /// - `value`: Value to write
    ///
    /// # Example
    ///
    /// ```no_run
    /// let value = b"Hello";
    /// gatt.update_characteristic_value(service_handle, char_handle, 0, value)?;
    /// ```
    pub fn update_characteristic_value(
        &mut self,
        service_handle: ServiceHandle,
        char_handle: CharacteristicHandle,
        offset: u8,
        value: &[u8],
    ) -> Result<(), BleError> {
        if !self.initialized {
            return Err(BleError::NotInitialized);
        }

        if value.len() > 255 {
            return Err(BleError::InvalidParameter);
        }

        unsafe {
            let status = aci_gatt_update_char_value(
                service_handle.0,
                char_handle.0,
                offset,
                value.len() as u8,
                value.as_ptr(),
            );

            if status == BLE_STATUS_SUCCESS {
                Ok(())
            } else {
                Err(BleError::CommandFailed(crate::wba::hci::types::Status::from_u8(status)))
            }
        }
    }

    /// Delete a service from the GATT database
    ///
    /// # Parameters
    ///
    /// - `service_handle`: Handle of the service to delete
    pub fn delete_service(&mut self, service_handle: ServiceHandle) -> Result<(), BleError> {
        if !self.initialized {
            return Err(BleError::NotInitialized);
        }

        unsafe {
            let status = ble::aci_gatt_del_service(service_handle.0);

            if status == BLE_STATUS_SUCCESS {
                Ok(())
            } else {
                Err(BleError::CommandFailed(crate::wba::hci::types::Status::from_u8(status)))
            }
        }
    }

    /// Delete a characteristic from a service
    ///
    /// # Parameters
    ///
    /// - `service_handle`: Handle of the service
    /// - `char_handle`: Handle of the characteristic to delete
    pub fn delete_characteristic(
        &mut self,
        service_handle: ServiceHandle,
        char_handle: CharacteristicHandle,
    ) -> Result<(), BleError> {
        if !self.initialized {
            return Err(BleError::NotInitialized);
        }

        unsafe {
            let status = ble::aci_gatt_del_char(service_handle.0, char_handle.0);

            if status == BLE_STATUS_SUCCESS {
                Ok(())
            } else {
                Err(BleError::CommandFailed(crate::wba::hci::types::Status::from_u8(status)))
            }
        }
    }

    /// Send a notification to a connected client
    ///
    /// This sends a notification (unconfirmed) containing the characteristic value
    /// to the specified connection. The client must have enabled notifications
    /// via the CCCD for this characteristic.
    ///
    /// # Parameters
    ///
    /// - `conn_handle`: Connection handle to send notification to
    /// - `service_handle`: Handle of the service
    /// - `char_handle`: Handle of the characteristic
    /// - `value`: Value to send in the notification
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the notification was queued successfully
    /// - `Err(BleError)` if the notification failed
    ///
    /// # Note
    ///
    /// The `GattNotificationComplete` event will be generated when the notification
    /// has been transmitted.
    pub fn notify(
        &self,
        conn_handle: u16,
        service_handle: ServiceHandle,
        char_handle: CharacteristicHandle,
        value: &[u8],
    ) -> Result<(), BleError> {
        if !self.initialized {
            return Err(BleError::NotInitialized);
        }

        if value.len() > 251 {
            return Err(BleError::InvalidParameter);
        }

        unsafe {
            let status = aci_gatt_update_char_value_ext(
                conn_handle,
                service_handle.0,
                char_handle.0,
                update_type::NOTIFICATION,
                value.len() as u16,
                0, // offset
                value.len() as u8,
                value.as_ptr(),
            );

            if status == BLE_STATUS_SUCCESS {
                Ok(())
            } else {
                Err(BleError::CommandFailed(crate::wba::hci::types::Status::from_u8(status)))
            }
        }
    }

    /// Send an indication to a connected client
    ///
    /// This sends an indication (confirmed) containing the characteristic value
    /// to the specified connection. The client must have enabled indications
    /// via the CCCD for this characteristic. The server will wait for confirmation
    /// from the client.
    ///
    /// # Parameters
    ///
    /// - `conn_handle`: Connection handle to send indication to
    /// - `service_handle`: Handle of the service
    /// - `char_handle`: Handle of the characteristic
    /// - `value`: Value to send in the indication
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the indication was queued successfully
    /// - `Err(BleError)` if the indication failed
    ///
    /// # Note
    ///
    /// The `GattIndicationComplete` event will be generated when the client
    /// confirms receipt of the indication.
    pub fn indicate(
        &self,
        conn_handle: u16,
        service_handle: ServiceHandle,
        char_handle: CharacteristicHandle,
        value: &[u8],
    ) -> Result<(), BleError> {
        if !self.initialized {
            return Err(BleError::NotInitialized);
        }

        if value.len() > 251 {
            return Err(BleError::InvalidParameter);
        }

        unsafe {
            let status = aci_gatt_update_char_value_ext(
                conn_handle,
                service_handle.0,
                char_handle.0,
                update_type::INDICATION,
                value.len() as u16,
                0, // offset
                value.len() as u8,
                value.as_ptr(),
            );

            if status == BLE_STATUS_SUCCESS {
                Ok(())
            } else {
                Err(BleError::CommandFailed(crate::wba::hci::types::Status::from_u8(status)))
            }
        }
    }

    /// Read a characteristic value from the local GATT database
    ///
    /// # Parameters
    ///
    /// - `attr_handle`: Attribute handle to read
    /// - `buffer`: Buffer to store the value
    ///
    /// # Returns
    ///
    /// Number of bytes read on success
    pub fn read_value(&self, attr_handle: u16, buffer: &mut [u8]) -> Result<usize, BleError> {
        if !self.initialized {
            return Err(BleError::NotInitialized);
        }

        unsafe {
            let mut value_length: u16 = 0;
            let mut value_offset: u16 = 0;

            let status = aci_gatt_read_handle_value(
                attr_handle,
                0,
                buffer.len() as u16,
                &mut value_length,
                &mut value_offset,
                buffer.as_mut_ptr(),
            );

            if status == BLE_STATUS_SUCCESS {
                Ok(value_length as usize)
            } else {
                Err(BleError::CommandFailed(crate::wba::hci::types::Status::from_u8(status)))
            }
        }
    }

    /// Set the GATT event mask
    ///
    /// Controls which GATT events are reported to the application.
    ///
    /// # Parameters
    ///
    /// - `mask`: Bitmask of events to enable (use GattEventMaskBits constants)
    pub fn set_event_mask(&self, mask: u32) -> Result<(), BleError> {
        if !self.initialized {
            return Err(BleError::NotInitialized);
        }

        unsafe {
            let status = aci_gatt_set_event_mask(mask);

            if status == BLE_STATUS_SUCCESS {
                Ok(())
            } else {
                Err(BleError::CommandFailed(crate::wba::hci::types::Status::from_u8(status)))
            }
        }
    }
}

impl Default for GattServer {
    fn default() -> Self {
        Self::new()
    }
}
