//! Host configuration data (`ACI_HAL_WRITE_CONFIG_DATA` / `ACI_HAL_READ_CONFIG_DATA`).
//!
//! These offsets carry the stack's identity root, encryption root, random
//! address and Secure Connections key-type selection. The ACI commands are
//! declared by `stm32wb-hci` and executed through the [`ControllerAdapter`].

use bt_hci::cmd::SyncCmd;
use stm32wb_hci::aci::general::{AciReadConfigData, AciWriteConfigData};
use stm32wb_hci::aci::values::{ConfigDataOffset, ReadableConfigDataOffset};

use crate::bluetooth::error::BleError;
use crate::controller::ControllerAdapter;

/// Write `value` at `offset` in the host's configuration data.
pub async fn write(controller: &ControllerAdapter<'_>, offset: u8, value: &[u8]) -> Result<(), BleError> {
    let offset = ConfigDataOffset::try_from(offset).map_err(|_| BleError::InvalidParameter)?;
    let cmd = AciWriteConfigData::try_new(offset, value).map_err(|_| BleError::InvalidParameter)?;
    cmd.exec(controller).await?;
    Ok(())
}

/// Read configuration data at `offset` into `out`, filling all of it.
///
/// Fails with [`BleError::InvalidParameter`] when the host returns fewer bytes
/// than were asked for.
pub async fn read(controller: &ControllerAdapter<'_>, offset: u8, out: &mut [u8]) -> Result<(), BleError> {
    let offset = ReadableConfigDataOffset::try_from(offset).map_err(|_| BleError::InvalidParameter)?;
    let resp = AciReadConfigData::new(offset).exec(controller).await?;
    if resp.data.len() != out.len() {
        return Err(BleError::InvalidParameter);
    }
    out.copy_from_slice(&resp.data);
    Ok(())
}
