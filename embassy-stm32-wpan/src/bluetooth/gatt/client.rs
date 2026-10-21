//! Minimal GATT client API for core BLE workflows.
//!
//! This module intentionally focuses on the common commands needed for
//! bring-up: service/characteristic discovery and basic read/write flows.

use bt_hci::cmd::{AsyncCmd, SyncCmd};
use bt_hci::param::ConnHandle;
use stm32wb_hci::aci::att::{AttExecuteWriteReq, AttPrepareWriteReq};
use stm32wb_hci::aci::gatt::{
    GattConfirmIndication, GattDiscAllCharDesc, GattDiscAllCharOfService, GattDiscAllPrimaryServices,
    GattDiscCharByUuid, GattDiscPrimaryServiceByUuid, GattExchangeConfig, GattFindIncludedServices, GattReadCharValue,
    GattReadLongCharValue, GattReadMultipleCharValue, GattReadMultipleVarCharValue, GattReadUsingCharUuid,
    GattWriteCharReliable, GattWriteCharValue, GattWriteLongCharValue, GattWriteWithoutResp, HandleEntry,
};
use stm32wb_hci::wire::AttBearer;

use crate::bluetooth::error::BleError;
use crate::bluetooth::gatt::types::Uuid;
use crate::controller::ControllerAdapter;

/// The ATT bearer the GATT procedures run on: the unenhanced bearer of the
/// connection `conn_handle`.
fn to_bearer(conn_handle: u16) -> Result<AttBearer, BleError> {
    AttBearer::from_raw(conn_handle).ok_or(BleError::InvalidParameter)
}

/// A connection handle for commands that address the connection itself
/// rather than an ATT bearer.
fn to_conn_handle(conn_handle: u16) -> Result<ConnHandle, BleError> {
    if conn_handle > 0x0EFF {
        Err(BleError::InvalidParameter)
    } else {
        Ok(ConnHandle::new(conn_handle))
    }
}

/// Minimal GATT client helper.
pub struct GattClient {
    _private: (),
}

impl GattClient {
    pub(crate) const fn new() -> Self {
        Self { _private: () }
    }

    /// Discover all primary services on the peer.
    pub async fn discover_all_primary_services(
        &self,
        controller: &ControllerAdapter<'_>,
        conn_handle: u16,
    ) -> Result<(), BleError> {
        GattDiscAllPrimaryServices::new(to_bearer(conn_handle)?)
            .exec(controller)
            .await?;
        Ok(())
    }

    /// Discover primary services by UUID.
    pub async fn discover_primary_service_by_uuid(
        &self,
        controller: &ControllerAdapter<'_>,
        conn_handle: u16,
        uuid: Uuid,
    ) -> Result<(), BleError> {
        GattDiscPrimaryServiceByUuid::new(to_bearer(conn_handle)?, uuid)
            .exec(controller)
            .await?;
        Ok(())
    }

    /// Discover all characteristics in a service handle range.
    ///
    /// Expected event sequence:
    /// - zero or more `GattClientEvent::CharacteristicFound`
    /// - final `GattClientEvent::ProcedureComplete`
    pub async fn discover_all_characteristics(
        &self,
        controller: &ControllerAdapter<'_>,
        conn_handle: u16,
        service_start_handle: u16,
        service_end_handle: u16,
    ) -> Result<(), BleError> {
        GattDiscAllCharOfService::new(to_bearer(conn_handle)?, service_start_handle, service_end_handle)
            .exec(controller)
            .await?;
        Ok(())
    }

    /// Discover all descriptors for a characteristic value handle.
    ///
    /// Expected event sequence:
    /// - zero or more `GattClientEvent::DescriptorFound`
    /// - final `GattClientEvent::ProcedureComplete`
    pub async fn discover_all_descriptors(
        &self,
        controller: &ControllerAdapter<'_>,
        conn_handle: u16,
        char_value_handle: u16,
        char_end_handle: u16,
    ) -> Result<(), BleError> {
        GattDiscAllCharDesc::new(to_bearer(conn_handle)?, char_value_handle, char_end_handle)
            .exec(controller)
            .await?;
        Ok(())
    }

    /// Discover included services in a primary service handle range.
    pub async fn discover_included_services(
        &self,
        controller: &ControllerAdapter<'_>,
        conn_handle: u16,
        service_start_handle: u16,
        service_end_handle: u16,
    ) -> Result<(), BleError> {
        GattFindIncludedServices::new(to_bearer(conn_handle)?, service_start_handle, service_end_handle)
            .exec(controller)
            .await?;
        Ok(())
    }

    /// Discover characteristics in a service range matching a specific UUID.
    pub async fn discover_characteristics_by_uuid(
        &self,
        controller: &ControllerAdapter<'_>,
        conn_handle: u16,
        service_start_handle: u16,
        service_end_handle: u16,
        uuid: Uuid,
    ) -> Result<(), BleError> {
        GattDiscCharByUuid::new(to_bearer(conn_handle)?, service_start_handle, service_end_handle, uuid)
            .exec(controller)
            .await?;
        Ok(())
    }

    /// Read a characteristic or descriptor value.
    pub async fn read_value(
        &self,
        controller: &ControllerAdapter<'_>,
        conn_handle: u16,
        attr_handle: u16,
    ) -> Result<(), BleError> {
        GattReadCharValue::new(to_bearer(conn_handle)?, attr_handle)
            .exec(controller)
            .await?;
        Ok(())
    }

    /// Read one or more characteristics by UUID in a handle range.
    pub async fn read_using_characteristic_uuid(
        &self,
        controller: &ControllerAdapter<'_>,
        conn_handle: u16,
        start_handle: u16,
        end_handle: u16,
        uuid: Uuid,
    ) -> Result<(), BleError> {
        GattReadUsingCharUuid::new(to_bearer(conn_handle)?, start_handle, end_handle, uuid)
            .exec(controller)
            .await?;
        Ok(())
    }

    /// Read multiple characteristic values in one ATT procedure.
    ///
    /// `attr_handles` must contain 2..=126 handles.
    pub async fn read_multiple_values(
        &self,
        controller: &ControllerAdapter<'_>,
        conn_handle: u16,
        attr_handles: &[u16],
    ) -> Result<(), BleError> {
        let entries = to_handle_entries(attr_handles)?;
        let cmd = GattReadMultipleCharValue::try_new(to_bearer(conn_handle)?, &entries)
            .map_err(|_| BleError::InvalidParameter)?;
        cmd.exec(controller).await?;
        Ok(())
    }

    /// Read multiple variable-length characteristic values in one ATT procedure.
    ///
    /// `attr_handles` must contain 2..=126 handles.
    pub async fn read_multiple_variable_values(
        &self,
        controller: &ControllerAdapter<'_>,
        conn_handle: u16,
        attr_handles: &[u16],
    ) -> Result<(), BleError> {
        let entries = to_handle_entries(attr_handles)?;
        let cmd = GattReadMultipleVarCharValue::try_new(to_bearer(conn_handle)?, &entries)
            .map_err(|_| BleError::InvalidParameter)?;
        cmd.exec(controller).await?;
        Ok(())
    }

    /// Read a long characteristic/descriptor value from a given offset.
    pub async fn read_long_value(
        &self,
        controller: &ControllerAdapter<'_>,
        conn_handle: u16,
        attr_handle: u16,
        offset: u16,
    ) -> Result<(), BleError> {
        GattReadLongCharValue::new(to_bearer(conn_handle)?, attr_handle, offset)
            .exec(controller)
            .await?;
        Ok(())
    }

    /// Write a characteristic or descriptor value with response.
    pub async fn write_value(
        &self,
        controller: &ControllerAdapter<'_>,
        conn_handle: u16,
        attr_handle: u16,
        value: &[u8],
    ) -> Result<(), BleError> {
        if value.len() > 255 {
            return Err(BleError::InvalidParameter);
        }
        let cmd = GattWriteCharValue::try_new(to_bearer(conn_handle)?, attr_handle, value)
            .map_err(|_| BleError::InvalidParameter)?;
        cmd.exec(controller).await?;
        Ok(())
    }

    /// Read a descriptor by handle (semantic alias of `read_value`).
    pub async fn read_descriptor(
        &self,
        controller: &ControllerAdapter<'_>,
        conn_handle: u16,
        descriptor_handle: u16,
    ) -> Result<(), BleError> {
        self.read_value(controller, conn_handle, descriptor_handle).await
    }

    /// Write a descriptor by handle (semantic alias of `write_value`).
    pub async fn write_descriptor(
        &self,
        controller: &ControllerAdapter<'_>,
        conn_handle: u16,
        descriptor_handle: u16,
        value: &[u8],
    ) -> Result<(), BleError> {
        self.write_value(controller, conn_handle, descriptor_handle, value)
            .await
    }

    /// Write a characteristic or descriptor value without response.
    pub async fn write_without_response(
        &self,
        controller: &ControllerAdapter<'_>,
        conn_handle: u16,
        attr_handle: u16,
        value: &[u8],
    ) -> Result<(), BleError> {
        if value.len() > 255 {
            return Err(BleError::InvalidParameter);
        }
        let cmd = GattWriteWithoutResp::try_new(to_bearer(conn_handle)?, attr_handle, value)
            .map_err(|_| BleError::InvalidParameter)?;
        cmd.exec(controller).await?;
        Ok(())
    }

    /// Write a long characteristic/descriptor value from a given offset.
    pub async fn write_long_value(
        &self,
        controller: &ControllerAdapter<'_>,
        conn_handle: u16,
        attr_handle: u16,
        offset: u16,
        value: &[u8],
    ) -> Result<(), BleError> {
        if value.len() > 255 {
            return Err(BleError::InvalidParameter);
        }
        let cmd = GattWriteLongCharValue::try_new(to_bearer(conn_handle)?, attr_handle, offset, value)
            .map_err(|_| BleError::InvalidParameter)?;
        cmd.exec(controller).await?;
        Ok(())
    }

    /// Reliably write a characteristic/descriptor value from a given offset.
    pub async fn write_reliable(
        &self,
        controller: &ControllerAdapter<'_>,
        conn_handle: u16,
        attr_handle: u16,
        offset: u16,
        value: &[u8],
    ) -> Result<(), BleError> {
        if value.len() > 255 {
            return Err(BleError::InvalidParameter);
        }
        let cmd = GattWriteCharReliable::try_new(to_bearer(conn_handle)?, attr_handle, offset, value)
            .map_err(|_| BleError::InvalidParameter)?;
        cmd.exec(controller).await?;
        Ok(())
    }

    /// Stage one fragment for a prepare-write sequence.
    pub async fn prepare_write(
        &self,
        controller: &ControllerAdapter<'_>,
        conn_handle: u16,
        attr_handle: u16,
        offset: u16,
        value: &[u8],
    ) -> Result<(), BleError> {
        if value.len() > 255 {
            return Err(BleError::InvalidParameter);
        }
        let cmd = AttPrepareWriteReq::try_new(to_conn_handle(conn_handle)?, attr_handle, offset, value)
            .map_err(|_| BleError::InvalidParameter)?;
        cmd.exec(controller).await?;
        Ok(())
    }

    /// Execute or cancel all prepared writes for the connection.
    pub async fn execute_write(
        &self,
        controller: &ControllerAdapter<'_>,
        conn_handle: u16,
        execute: bool,
    ) -> Result<(), BleError> {
        AttExecuteWriteReq::new(to_conn_handle(conn_handle)?, execute)
            .exec(controller)
            .await?;
        Ok(())
    }

    /// Request ATT exchange configuration procedure.
    pub async fn exchange_configuration(
        &self,
        controller: &ControllerAdapter<'_>,
        conn_handle: u16,
    ) -> Result<(), BleError> {
        GattExchangeConfig::new(to_conn_handle(conn_handle)?)
            .exec(controller)
            .await?;
        Ok(())
    }

    /// Confirm an indication from the peer.
    pub async fn confirm_indication(
        &self,
        controller: &ControllerAdapter<'_>,
        conn_handle: u16,
    ) -> Result<(), BleError> {
        GattConfirmIndication::new(to_bearer(conn_handle)?)
            .exec(controller)
            .await?;
        Ok(())
    }
}

fn to_handle_entries(attr_handles: &[u16]) -> Result<heapless::Vec<HandleEntry, 126>, BleError> {
    if !(2..=126).contains(&attr_handles.len()) {
        return Err(BleError::InvalidParameter);
    }

    let mut out = heapless::Vec::<HandleEntry, 126>::new();
    for &handle in attr_handles {
        out.push(HandleEntry { handle })
            .map_err(|_| BleError::InvalidParameter)?;
    }
    Ok(out)
}
