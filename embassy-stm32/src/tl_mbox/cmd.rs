use super::consts::TlPacketType;
use super::PacketHeader;

#[repr(C, packed)]
#[derive(Copy, Clone)]
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

#[repr(C, packed)]
#[derive(Copy, Clone, Default)]
pub struct CommandSerial {
    pub typ: u8,
    pub cmd: Cmd,
}

#[repr(C, packed)]
#[derive(Copy, Clone, Default)]
pub struct CommandPacket {
    pub header: PacketHeader,
    pub cmd_serial: CommandSerial,
}

impl CommandPacket {
    /// Copies the provided buffer into a [`CommandPacket`]
    pub unsafe fn copy_into_packet_from_slice(pcmd_packet: *mut CommandPacket, buf: &[u8], packet_type: TlPacketType) {
        (*pcmd_packet).cmd_serial.typ = packet_type as u8;

        let pcmd_serial: *mut CommandSerial = &mut (*pcmd_packet).cmd_serial;
        let pcmd_serial_buf: *mut u8 = pcmd_serial.cast();

        core::ptr::copy(buf.as_ptr(), pcmd_serial_buf, buf.len());
    }
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct AclDataSerial {
    pub ty: u8,
    pub handle: u16,
    pub length: u16,
    pub acl_data: [u8; 1],
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct AclDataPacket {
    pub header: PacketHeader,
    pub acl_data_serial: AclDataSerial,
}
