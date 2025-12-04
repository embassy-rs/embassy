use core::ptr;
use core::sync::atomic::{Ordering, compiler_fence};

use crate::consts::TlPacketType;
use crate::wb55::PacketHeader;

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
        let p_cmd_serial = (cmd_buf as *mut u8).add(size_of::<PacketHeader>());
        let p_payload = p_cmd_serial.add(size_of::<CmdSerialStub>());

        ptr::write_unaligned(
            p_cmd_serial as *mut _,
            CmdSerialStub {
                ty: packet_type as u8,
                cmd_code,
                payload_len: payload.len() as u8,
            },
        );

        ptr::copy_nonoverlapping(payload as *const _ as *const u8, p_payload, payload.len());

        compiler_fence(Ordering::Release);
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
pub struct AclDataSerialStub {
    pub ty: u8,
    pub handle: u16,
    pub length: u16,
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct AclDataPacket {
    pub header: PacketHeader,
    pub acl_data_serial: AclDataSerial,
}

impl AclDataPacket {
    pub unsafe fn write_into(cmd_buf: *mut AclDataPacket, packet_type: TlPacketType, handle: u16, payload: &[u8]) {
        let p_cmd_serial = (cmd_buf as *mut u8).add(size_of::<PacketHeader>());
        let p_payload = p_cmd_serial.add(size_of::<AclDataSerialStub>());

        ptr::write_unaligned(
            p_cmd_serial as *mut _,
            AclDataSerialStub {
                ty: packet_type as u8,
                handle: handle,
                length: payload.len() as u16,
            },
        );

        ptr::copy_nonoverlapping(payload as *const _ as *const u8, p_payload, payload.len());

        compiler_fence(Ordering::Release);
    }
}
