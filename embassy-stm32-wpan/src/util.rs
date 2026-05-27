//! Crate utils

use core::future::poll_fn;
use core::sync::atomic::{AtomicBool, Ordering};
use core::task::Poll;

use embassy_sync::waitqueue::AtomicWaker;

#[allow(unused)]
pub struct Flag {
    state: AtomicBool,
    waker: AtomicWaker,
}

#[allow(unused)]
impl Flag {
    pub const fn new(state: bool) -> Self {
        Self {
            state: AtomicBool::new(state),
            waker: AtomicWaker::new(),
        }
    }

    pub fn set_high(&self) {
        if !self.state.swap(true, Ordering::AcqRel) {
            self.waker.wake();
        }
    }

    pub fn set_low(&self) {
        if self.state.swap(false, Ordering::AcqRel) {
            self.waker.wake();
        }
    }

    pub async fn wait_for_high(&self) {
        poll_fn(|cx| {
            self.waker.register(cx.waker());

            if !self.state.load(Ordering::Acquire) {
                Poll::Pending
            } else {
                Poll::Ready(())
            }
        })
        .await;
    }

    pub async fn wait_for_low(&self) {
        poll_fn(|cx| {
            self.waker.register(cx.waker());

            if self.state.load(Ordering::Acquire) {
                Poll::Pending
            } else {
                Poll::Ready(())
            }
        })
        .await;
    }
}

#[cfg(feature = "bt-hci")]
pub fn to_err(e: bt_hci::FromHciBytesError) -> embedded_io::ErrorKind {
    use bt_hci::FromHciBytesError;
    use embedded_io::ErrorKind;

    match e {
        FromHciBytesError::InvalidSize => ErrorKind::InvalidInput,
        FromHciBytesError::InvalidValue => ErrorKind::InvalidData,
    }
}

#[cfg(feature = "bt-hci")]
pub fn make_cc_with_cs<'a>(
    buf: &'a [u8],
) -> Result<bt_hci::event::CommandCompleteWithStatus<'a>, bt_hci::cmd::Error<embedded_io::ErrorKind>> {
    use bt_hci::cmd::Error as CmdError;
    use bt_hci::event::{CommandComplete, CommandCompleteWithStatus, CommandStatus, EventKind};
    use bt_hci::param::RemainingBytes;
    use bt_hci::{ControllerToHostPacket, FromHciBytes};
    use embedded_io::ErrorKind;

    let (pkt, _) = ControllerToHostPacket::from_hci_bytes(buf)
        .map_err(to_err)
        .map_err(CmdError::Io)?;

    let ControllerToHostPacket::Event(ref event) = pkt else {
        return Err(CmdError::Io(ErrorKind::InvalidData));
    };

    match event.kind {
        EventKind::CommandComplete => {
            let e = CommandComplete::from_hci_bytes_complete(event.data)
                .map_err(to_err)
                .map_err(CmdError::Io)?;

            e.try_into().map_err(to_err).map_err(CmdError::Io)
        }
        EventKind::CommandStatus => {
            let e = CommandStatus::from_hci_bytes_complete(event.data)
                .map_err(to_err)
                .map_err(CmdError::Io)?;

            Ok(CommandCompleteWithStatus {
                num_hci_cmd_pkts: 0,
                cmd_opcode: e.cmd_opcode,
                status: e.status,
                return_param_bytes: RemainingBytes::default(),
            })
        }
        _ => return Err(CmdError::Io(ErrorKind::InvalidData)),
    }
}

/// Parse an HCI event, with fallbacks for vendor payloads that stm32wb-hci does not yet decode.
#[cfg(all(feature = "bt-hci", feature = "wba"))]
pub fn parse_event_with_fallback(
    event_type: u8,
    payload: &[u8],
) -> Result<stm32wb_hci::Event, stm32wb_hci::event::Error> {
    stm32wb_hci::Event::from_kind_and_payload(event_type, payload)
}

/// Returns true when the vendor payload is an ST HAL firmware warning.
#[cfg(all(feature = "bt-hci", feature = "wba"))]
pub fn vendor_event_is_hal_firmware_warning(payload: &[u8], warning: u8) -> bool {
    payload.len() >= 3 && u16::from_le_bytes([payload[0], payload[1]]) == 0x0006 && payload[2] == warning
}
