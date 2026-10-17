//! Direct command execution (`direct` feature).
//!
//! By default [`ControllerAdapter`] executes every bt-hci command the same
//! way: it encodes the command as an HCI packet, hands it to
//! `BleStack_Request`, and decodes the reply packet. With the `direct` feature
//! those blanket implementations are disabled. Instead, every command
//! `stm32wb-hci` implements [`Direct`] for runs by calling its own function in
//! ST's BLE stack, the way `nrf-sdc` does: `LeSetAdvParams` calls
//! `hci_le_set_advertising_parameters`, `GapInit` calls `aci_gap_init`.
//!
//! `stm32wb-hci` generates those calls from its catalog of the stack, for the
//! commands the selected stack implements. Any other command fails to compile.

use bt_hci::cmd::{AsyncCmd, Error as CmdError, SyncCmd};
use bt_hci::controller::{ControllerCmdAsync, ControllerCmdSync};
use embedded_io::ErrorKind;
use stm32wb_hci::direct::{Direct, Error};

use crate::wba::controller::ControllerAdapter;

fn cmd_error(error: Error) -> CmdError<ErrorKind> {
    match error {
        Error::Hci(error) => CmdError::Hci(error),
        Error::Invalid => CmdError::Io(ErrorKind::InvalidData),
    }
}

impl<'d, C> ControllerCmdSync<C> for ControllerAdapter<'d>
where
    C: SyncCmd + Direct<Output = C::Return>,
{
    async fn exec(&self, cmd: &C) -> Result<C::Return, CmdError<ErrorKind>> {
        // SAFETY: the controller exists only while the stack is initialized,
        // and `call` holds it so no other call into the stack runs meanwhile.
        self.call(|| unsafe { cmd.call() }).map_err(cmd_error)
    }
}

impl<'d, C> ControllerCmdAsync<C> for ControllerAdapter<'d>
where
    C: AsyncCmd + Direct<Output = ()>,
{
    async fn exec(&self, cmd: &C) -> Result<(), CmdError<ErrorKind>> {
        // SAFETY: as for `ControllerCmdSync`.
        self.call(|| unsafe { cmd.call() }).map_err(cmd_error)
    }
}
