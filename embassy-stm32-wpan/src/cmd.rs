use core::ptr;

use crate::consts::TlPacketType;
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
pub struct CmdSerialStub {
    pub ty: u8,
    pub cmd_code: u16,
    pub payload_len: u8,
}

#[derive(Copy, Clone, Default)]
#[repr(C, packed)]
pub struct CmdPacket {
    pub header: PacketHeader,
    pub cmdserial: CmdSerial,
}

impl CmdPacket {
    pub unsafe fn write_into(cmd_buf: *mut CmdPacket, packet_type: TlPacketType, cmd_code: u16, payload: &[u8]) {
        let p_cmd_serial = &mut (*cmd_buf).cmdserial as *mut _ as *mut CmdSerialStub;
        let p_payload = &mut (*cmd_buf).cmdserial.cmd.payload as *mut _;

        ptr::write_volatile(
            p_cmd_serial,
            CmdSerialStub {
                ty: packet_type as u8,
                cmd_code: cmd_code,
                payload_len: payload.len() as u8,
            },
        );

        ptr::copy_nonoverlapping(payload as *const _ as *const u8, p_payload, payload.len());
    }

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
