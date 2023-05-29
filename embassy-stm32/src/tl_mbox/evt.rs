use core::mem::MaybeUninit;

use super::cmd::{AclDataPacket, AclDataSerial, CommandSerial};
use super::consts::{TlPacketType, TL_EVT_HEADER_SIZE};
use super::mm::MemoryManager;
use super::{CommandPacket, LinkedListNode, PacketHeader};

/// the payload of [`Event`] for a command status event
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct CommandStatusEvent {
    pub status: u8,
    pub num_cmd: u8,
    pub cmd_code: u16,
}

/// the payload of [`Event`] for a command complete event
#[derive(Clone, Copy, Default)]
#[repr(C, packed)]
pub struct CommandChannelEvent {
    pub num_cmd: u8,
    pub cmd_code: u8,
    pub payload: [u8; 1],
}

impl CommandChannelEvent {
    pub unsafe fn from_node(pcmd_packet: *mut CommandPacket) -> Result<Self, ()> {
        // ST's command response data structure is really convoluted.
        //
        // for command response events on SYS channel, the header is missing
        // and one should:
        // 1. interpret the content of CMD_BUFFER as CommandPacket
        // 2. Access CommandPacket's cmdserial field and interpret its content as EvtSerial
        // 3. Access EvtSerial's evt field (as Evt) and interpret its payload as CcEvt
        // 4. CcEvt type is the actual SHCI response
        // 5. profit

        //        unsafe {
        //            let cmd: *const CommandPacket = (*TL_SYS_TABLE.as_ptr()).pcmd_buffer;
        //            let cmd_serial: *const CmdSerial = &(*cmd).cmd_serial;
        //            let evt_serial: *const EvtSerial = cmd_serial.cast();
        //            let cc = (*evt_serial).evt.payload.as_ptr().cast();
        //            *cc
        //        }

        let cmd_serial: *const CommandSerial = &(*pcmd_packet).cmd_serial;
        let evt_serial: *const EventSerial = cmd_serial.cast();
        let cc: *const CommandChannelEvent = (*evt_serial).event.payload.as_ptr().cast();

        Ok((*cc).clone())
    }
}

#[derive(Clone, Copy, Default)]
#[repr(C, packed)]
pub struct Event {
    pub event_code: u8,
    pub payload_len: u8,
    pub payload: [u8; 1],
}

#[derive(Clone, Copy, Default)]
#[repr(C, packed)]
pub struct EventSerial {
    pub kind: u8,
    pub event: Event,
}

/// This format shall be used for all events (asynchronous and command response) reported
/// by the CPU2 except for the command response of a system command where the header is not there
/// and the format to be used shall be [`EventSerial`].
///
/// ### Note:
/// Be careful that the asynchronous events reported by the CPU2 on the system channel do
/// include the header and shall use [`EventPacket`] format. Only the command response format on the
/// system channel is different.
#[derive(Clone, Copy, Default)]
#[repr(C, packed)]
pub struct EventPacket {
    pub header: PacketHeader,
    pub event_serial: EventSerial,
}

/// Smart pointer to the [`EventPacket`] that will dispose of it automatically on drop
pub struct EventBox {
    ptr: *mut EventPacket,
}

unsafe impl Send for EventBox {}
impl EventBox {
    pub(super) fn new(ptr: *mut EventPacket) -> Self {
        Self { ptr }
    }

    /// Creates an [`EventBox`] from a [`LinkedListNode`] if it exists, otherwise returns None.
    pub unsafe fn from_node(node: *mut LinkedListNode) -> Option<Result<EventBox, ()>> {
        let mut node_ptr = core::ptr::null_mut();
        if LinkedListNode::is_empty(node) {
            None
        } else {
            LinkedListNode::remove_head(node, &mut node_ptr);

            Some(Ok(EventBox::new(node_ptr.cast())))
        }
    }

    /// Copies the event data from inner pointer and returns and event structure
    #[allow(dead_code)]
    pub fn event_packet(&self) -> EventPacket {
        let mut evt = MaybeUninit::uninit();
        unsafe {
            self.ptr.copy_to(evt.as_mut_ptr(), 1);
            evt.assume_init()
        }
    }

    /// Returns the size of a buffer required to hold this event
    #[allow(dead_code)]
    pub fn size(&self) -> Result<usize, ()> {
        unsafe {
            let evt_kind = TlPacketType::try_from((*self.ptr).event_serial.kind)?;

            if evt_kind == TlPacketType::AclData {
                let acl_data: *const AclDataPacket = self.ptr.cast();
                let acl_serial: *const AclDataSerial = &(*acl_data).acl_data_serial;

                Ok((*acl_serial).length as usize + 5)
            } else {
                let evt_data: *const EventPacket = self.ptr.cast();
                let evt_serial: *const EventSerial = &(*evt_data).event_serial;

                Ok((*evt_serial).event.payload_len as usize + TL_EVT_HEADER_SIZE)
            }
        }
    }

    /// writes an underlying [`EvtPacket`] into the provided buffer. Returns the number of bytes that were
    /// written. Returns an error if event kind is unkown or if provided buffer size is not enough
    pub fn copy_into_slice(&self, buf: &mut [u8]) -> Result<usize, ()> {
        unsafe {
            let evt_kind = TlPacketType::try_from((*self.ptr).event_serial.kind)?;

            let evt_data: *const EventPacket = self.ptr.cast();
            let evt_serial: *const EventSerial = &(*evt_data).event_serial;
            let evt_serial_buf: *const u8 = evt_serial.cast();

            let acl_data: *const AclDataPacket = self.ptr.cast();
            let acl_serial: *const AclDataSerial = &(*acl_data).acl_data_serial;
            let acl_serial_buf: *const u8 = acl_serial.cast();

            if let TlPacketType::AclData = evt_kind {
                let len = (*acl_serial).length as usize + 5;
                if len > buf.len() {
                    return Err(());
                }

                core::ptr::copy(evt_serial_buf, buf.as_mut_ptr(), len);

                Ok(len)
            } else {
                let len = (*evt_serial).event.payload_len as usize + TL_EVT_HEADER_SIZE;
                if len > buf.len() {
                    return Err(());
                }

                core::ptr::copy(acl_serial_buf, buf.as_mut_ptr(), len);

                Ok(len)
            }
        }
    }
}

impl Drop for EventBox {
    fn drop(&mut self) {
        MemoryManager::drop_packet(self.ptr);
    }
}
