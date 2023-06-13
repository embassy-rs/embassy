use super::PacketHeader;

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct Command {
    pub command_code: u16,
    pub payload_len: u8,
    pub payload: [u8; 255],
}

impl Default for Command {
    fn default() -> Self {
        Self {
            command_code: 0,
            payload_len: 0,
            payload: [0u8; 255],
        }
    }
}

#[repr(C, packed)]
#[derive(Copy, Clone, Default)]
pub struct CommandSerial {
    pub typ: u8,
    pub command: Command,
}

#[repr(C, packed)]
#[derive(Copy, Clone, Default)]
pub struct CommandPacket {
    pub header: PacketHeader,
    pub cmd_serial: CommandSerial,
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
