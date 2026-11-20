//! GATT Server implementation
//!
//! This is a thin Rust wrapper around ST's GATT server implementation.

use bt_hci::cmd::SyncCmd;
use stm32wb_hci::aci::flags::{GattDescEventMask as HciGattDescEventMask, UpdateType};
use stm32wb_hci::aci::gatt::{
    GattAddChar, GattAddCharDesc, GattAddService, GattDelChar, GattDelService, GattPermitRead, GattPermitWrite,
    GattReadHandleValue, GattSetDescValue, GattSetEventMask, GattStoreDb, GattUpdateCharValue, GattUpdateCharValueExt,
};
use stm32wb_hci::aci::ranges::{AttAppError, EncKeySize};
use stm32wb_hci::aci::values::PermitStatus;
use stm32wb_hci::wire::AttBearer;

use super::types::{
    AccessPermissions, CharProperties, CharacteristicHandle, DescriptorHandle, GattEventMask, SecurityPermissions,
    ServiceHandle, ServiceType, Uuid,
};
use crate::bluetooth::error::BleError;
use crate::controller::ControllerAdapter;

fn to_bearer(conn_handle: u16) -> Result<AttBearer, BleError> {
    AttBearer::from_raw(conn_handle).ok_or(BleError::InvalidParameter)
}

/// Maps the raw encryption key size the public API takes to the range the
/// command encodes. The stack only documents 7-16; 0 ("no encryption", the
/// common case) is sent as the minimum, which the stack ignores unless the
/// security permissions require encryption.
fn to_enc_key_size(encryption_key_size: u8) -> Result<EncKeySize, BleError> {
    if encryption_key_size == 0 {
        Ok(EncKeySize::MIN)
    } else {
        EncKeySize::new(encryption_key_size).ok_or(BleError::InvalidParameter)
    }
}

/// GATT Server
///
/// Provides methods for creating and managing GATT services and characteristics.
pub struct GattServer {
    _private: (),
}

impl GattServer {
    /// Create a new GATT server instance
    pub(crate) const fn new() -> Self {
        Self { _private: () }
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
    /// let service_uuid = Uuid::Uuid16(0x1234);
    /// let handle = gatt.add_service(controller, service_uuid, ServiceType::Primary, 10).await?;
    /// ```
    pub async fn add_service(
        &mut self,
        controller: &ControllerAdapter<'_>,
        uuid: Uuid,
        service_type: ServiceType,
        max_attribute_records: u8,
    ) -> Result<ServiceHandle, BleError> {
        let service = GattAddService::new(uuid, service_type, max_attribute_records)
            .exec(controller)
            .await?;
        Ok(ServiceHandle(service.service_handle))
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
    /// let char_uuid = Uuid::Uuid16(0x2A00); // Device Name
    /// let props = CharProperties::READ | CharProperties::WRITE;
    /// let handle = gatt.add_characteristic(
    ///     controller,
    ///     service_handle,
    ///     char_uuid,
    ///     20,
    ///     props,
    ///     SecurityPermissions::empty(),
    ///     GattEventMask::ATTRIBUTE_WRITE,
    ///     0,
    ///     true,
    /// ).await?;
    /// ```
    pub async fn add_characteristic(
        &mut self,
        controller: &ControllerAdapter<'_>,
        service_handle: ServiceHandle,
        uuid: Uuid,
        value_length: u16,
        properties: CharProperties,
        security: SecurityPermissions,
        event_mask: GattEventMask,
        encryption_key_size: u8,
        is_variable: bool,
    ) -> Result<CharacteristicHandle, BleError> {
        let enc_key_size = to_enc_key_size(encryption_key_size)?;

        let char = GattAddChar::new(
            service_handle.0,
            uuid,
            value_length,
            properties,
            security,
            event_mask,
            enc_key_size,
            is_variable,
        )
        .exec(controller)
        .await?;
        Ok(CharacteristicHandle(char.char_handle))
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
    /// gatt.update_characteristic_value(controller, service_handle, char_handle, 0, value).await?;
    /// ```
    pub async fn update_characteristic_value(
        &mut self,
        controller: &ControllerAdapter<'_>,
        service_handle: ServiceHandle,
        char_handle: CharacteristicHandle,
        offset: u8,
        value: &[u8],
    ) -> Result<(), BleError> {
        if value.len() > 255 {
            return Err(BleError::InvalidParameter);
        }

        let cmd = GattUpdateCharValue::try_new(service_handle.0, char_handle.0, offset, value)
            .map_err(|_| BleError::InvalidParameter)?;
        cmd.exec(controller).await?;
        Ok(())
    }

    /// Delete a service from the GATT database
    ///
    /// # Parameters
    ///
    /// - `service_handle`: Handle of the service to delete
    pub async fn delete_service(
        &mut self,
        controller: &ControllerAdapter<'_>,
        service_handle: ServiceHandle,
    ) -> Result<(), BleError> {
        GattDelService::new(service_handle.0).exec(controller).await?;
        Ok(())
    }

    /// Delete a characteristic from a service
    ///
    /// # Parameters
    ///
    /// - `service_handle`: Handle of the service
    /// - `char_handle`: Handle of the characteristic to delete
    pub async fn delete_characteristic(
        &mut self,
        controller: &ControllerAdapter<'_>,
        service_handle: ServiceHandle,
        char_handle: CharacteristicHandle,
    ) -> Result<(), BleError> {
        GattDelChar::new(service_handle.0, char_handle.0)
            .exec(controller)
            .await?;
        Ok(())
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
    pub async fn notify(
        &self,
        controller: &ControllerAdapter<'_>,
        conn_handle: u16,
        service_handle: ServiceHandle,
        char_handle: CharacteristicHandle,
        value: &[u8],
    ) -> Result<(), BleError> {
        if value.len() > 251 {
            return Err(BleError::InvalidParameter);
        }

        let cmd = GattUpdateCharValueExt::try_new(
            to_bearer(conn_handle)?,
            service_handle.0,
            char_handle.0,
            UpdateType::NOTIFICATION,
            value.len() as u16,
            0, // offset
            value,
        )
        .map_err(|_| BleError::InvalidParameter)?;
        cmd.exec(controller).await?;
        Ok(())
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
    pub async fn indicate(
        &self,
        controller: &ControllerAdapter<'_>,
        conn_handle: u16,
        service_handle: ServiceHandle,
        char_handle: CharacteristicHandle,
        value: &[u8],
    ) -> Result<(), BleError> {
        if value.len() > 251 {
            return Err(BleError::InvalidParameter);
        }

        let cmd = GattUpdateCharValueExt::try_new(
            to_bearer(conn_handle)?,
            service_handle.0,
            char_handle.0,
            UpdateType::INDICATION,
            value.len() as u16,
            0, // offset
            value,
        )
        .map_err(|_| BleError::InvalidParameter)?;
        cmd.exec(controller).await?;
        Ok(())
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
    pub async fn read_value(
        &self,
        controller: &ControllerAdapter<'_>,
        attr_handle: u16,
        buffer: &mut [u8],
    ) -> Result<usize, BleError> {
        let resp = GattReadHandleValue::new(attr_handle, 0, buffer.len() as u16)
            .exec(controller)
            .await?;

        let value = resp.value.as_slice();
        let len = value.len().min(buffer.len());
        buffer[..len].copy_from_slice(&value[..len]);
        Ok(resp.length as usize)
    }

    /// Set the GATT event mask
    ///
    /// Controls which GATT events are reported to the application.
    ///
    /// # Parameters
    ///
    /// - `mask`: Bitmask of events to enable (use GattEventMaskBits constants)
    pub async fn set_event_mask(&self, controller: &ControllerAdapter<'_>, mask: u32) -> Result<(), BleError> {
        GattSetEventMask::new(mask).exec(controller).await?;
        Ok(())
    }

    /// Respond to a read permit request.
    ///
    /// Use this after receiving a `GattEvent::ReadPermitRequest`.
    /// When denying, `error_code` is returned to the client and `attr_handle`
    /// selects the first denied attribute (0 denies all).
    pub async fn permit_read(
        &self,
        controller: &ControllerAdapter<'_>,
        conn_handle: u16,
        allow: bool,
        error_code: u8,
        attr_handle: u16,
    ) -> Result<(), BleError> {
        // The error code is only sent to the client when denying; when
        // allowing, use a placeholder because the command encodes only
        // application error codes.
        let error_code = if allow {
            AttAppError::INSUFFICIENT_AUTHORIZATION
        } else {
            AttAppError::try_from(error_code).map_err(|_| BleError::InvalidParameter)?
        };
        let read_status = if allow {
            PermitStatus::Allowed
        } else {
            PermitStatus::Denied
        };
        GattPermitRead::new(to_bearer(conn_handle)?, read_status, error_code, attr_handle)
            .exec(controller)
            .await?;
        Ok(())
    }

    /// Allow a pending read permit request.
    pub async fn allow_read(&self, controller: &ControllerAdapter<'_>, conn_handle: u16) -> Result<(), BleError> {
        self.permit_read(controller, conn_handle, true, 0, 0).await
    }

    /// Deny a pending read permit request.
    ///
    /// - `error_code`: ATT error code to send to the client
    /// - `attr_handle`: first denied attribute handle (0 denies all in the request)
    pub async fn deny_read(
        &self,
        controller: &ControllerAdapter<'_>,
        conn_handle: u16,
        error_code: u8,
        attr_handle: u16,
    ) -> Result<(), BleError> {
        self.permit_read(controller, conn_handle, false, error_code, attr_handle)
            .await
    }

    /// Respond to a write permit request.
    ///
    /// Use this after receiving a `GattEvent::WritePermitRequest`.
    /// If allowing, pass the value that was requested to be written.
    pub async fn permit_write(
        &self,
        controller: &ControllerAdapter<'_>,
        conn_handle: u16,
        attr_handle: u16,
        allow: bool,
        error_code: u8,
        value: &[u8],
    ) -> Result<(), BleError> {
        if value.len() > 255 {
            return Err(BleError::InvalidParameter);
        }
        let write_status = if allow {
            PermitStatus::Allowed
        } else {
            PermitStatus::Denied
        };
        let cmd = GattPermitWrite::try_new(to_bearer(conn_handle)?, attr_handle, write_status, error_code, value)
            .map_err(|_| BleError::InvalidParameter)?;
        cmd.exec(controller).await?;
        Ok(())
    }

    /// Allow a pending write permit request using the requested value.
    pub async fn allow_write(
        &self,
        controller: &ControllerAdapter<'_>,
        conn_handle: u16,
        attr_handle: u16,
        value: &[u8],
    ) -> Result<(), BleError> {
        self.permit_write(controller, conn_handle, attr_handle, true, 0, value)
            .await
    }

    /// Deny a pending write permit request.
    pub async fn deny_write(
        &self,
        controller: &ControllerAdapter<'_>,
        conn_handle: u16,
        attr_handle: u16,
        error_code: u8,
    ) -> Result<(), BleError> {
        self.permit_write(controller, conn_handle, attr_handle, false, error_code, &[])
            .await
    }

    /// Add a characteristic descriptor to a characteristic.
    ///
    /// ST allocates descriptor handles after the currently allocated handles,
    /// so call this immediately after the `add_characteristic` that created
    /// `char_handle` (before adding further characteristics/descriptors).
    ///
    /// # Parameters
    ///
    /// - `value_max_len`: maximum descriptor value length in bytes
    /// - `value`: initial descriptor value
    /// - `security`: security requirements (encryption/authentication)
    /// - `access`: attribute access permissions (read/write/...)
    #[allow(clippy::too_many_arguments)]
    pub async fn add_descriptor(
        &mut self,
        controller: &ControllerAdapter<'_>,
        service_handle: ServiceHandle,
        char_handle: CharacteristicHandle,
        uuid: Uuid,
        value_max_len: u8,
        value: &[u8],
        security: SecurityPermissions,
        access: AccessPermissions,
        event_mask: GattEventMask,
        encryption_key_size: u8,
        is_variable: bool,
    ) -> Result<DescriptorHandle, BleError> {
        if value.len() > 255 {
            return Err(BleError::InvalidParameter);
        }

        let event_mask = HciGattDescEventMask::from_bits(event_mask.bits()).ok_or(BleError::InvalidParameter)?;
        let enc_key_size = to_enc_key_size(encryption_key_size)?;

        let cmd = GattAddCharDesc::try_new(
            service_handle.0,
            char_handle.0,
            uuid,
            value_max_len,
            value,
            security,
            access,
            event_mask,
            enc_key_size,
            is_variable,
        )
        .map_err(|_| BleError::InvalidParameter)?;
        let desc = cmd.exec(controller).await?;
        Ok(DescriptorHandle(desc.char_desc_handle))
    }

    /// Update the value of a descriptor created with
    /// [`add_descriptor`](Self::add_descriptor).
    pub async fn update_descriptor_value(
        &mut self,
        controller: &ControllerAdapter<'_>,
        service_handle: ServiceHandle,
        char_handle: CharacteristicHandle,
        desc_handle: DescriptorHandle,
        offset: u16,
        value: &[u8],
    ) -> Result<(), BleError> {
        if value.len() > 255 {
            return Err(BleError::InvalidParameter);
        }
        let cmd = GattSetDescValue::try_new(service_handle.0, char_handle.0, desc_handle.0, offset, value)
            .map_err(|_| BleError::InvalidParameter)?;
        cmd.exec(controller).await?;
        Ok(())
    }

    /// Force the stack to persist the GATT database.
    ///
    /// The stack already saves the database per active connection on
    /// disconnect; use this to snapshot it immediately instead.
    pub async fn store_db(&self, controller: &ControllerAdapter<'_>) -> Result<(), BleError> {
        GattStoreDb::new().exec(controller).await?;
        Ok(())
    }
}
