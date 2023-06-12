use crate::evt::{EvtPacket, EvtSerial};
use crate::{PacketHeader, TL_EVT_HEADER_SIZE};

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct Cmd {
    pub cmd_code: u16,
    pub payload_len: u8,
    pub payload: [u8; 255],
}

impl Default for Cmd {
    fn default() -> Self {
        Self {
            cmd_code: 0,
            payload_len: 0,
            payload: [0u8; 255],
        }
    }
}

#[derive(Copy, Clone, Default)]
#[repr(C, packed)]
pub struct CmdSerial {
    pub ty: u8,
    pub cmd: Cmd,
}

#[derive(Copy, Clone, Default)]
#[repr(C, packed)]
pub struct CmdPacket {
    pub header: PacketHeader,
    pub cmdserial: CmdSerial,
}

impl CmdPacket {
    /// Writes an underlying CmdPacket into the provided buffer.
    /// Returns a number of bytes that were written.
    /// Returns an error if event kind is unknown or if provided buffer size is not enough.
    #[allow(clippy::result_unit_err)]
    pub fn write(&self, buf: &mut [u8]) -> Result<usize, ()> {
        unsafe {
            let cmd_ptr: *const CmdPacket = self;
            let self_as_evt_ptr: *const EvtPacket = cmd_ptr.cast();
            let evt_serial: *const EvtSerial = &(*self_as_evt_ptr).evt_serial;

            let acl_data: *const AclDataPacket = cmd_ptr.cast();
            let acl_serial: *const AclDataSerial = &(*acl_data).acl_data_serial;
            let acl_serial_buf: *const u8 = acl_serial.cast();

            let len = (*evt_serial).evt.payload_len as usize + TL_EVT_HEADER_SIZE;
            if len > buf.len() {
                return Err(());
            }

            core::ptr::copy(acl_serial_buf, buf.as_mut_ptr(), len);

            Ok(len)
        }
    }
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct AclDataSerial {
    pub ty: u8,
    pub handle: u16,
    pub length: u16,
    pub acl_data: [u8; 1],
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct AclDataPacket {
    pub header: PacketHeader,
    pub acl_data_serial: AclDataSerial,
}
