use core::convert::TryFrom;

#[derive(Debug)]
#[repr(C)]
pub enum TlPacketType {
    BleCmd = 0x01,
    AclData = 0x02,
    BleEvt = 0x04,

    OtCmd = 0x08,
    OtRsp = 0x09,
    CliCmd = 0x0A,
    OtNot = 0x0C,
    OtAck = 0x0D,
    CliNot = 0x0E,
    CliAck = 0x0F,

    SysCmd = 0x10,
    SysRsp = 0x11,
    SysEvt = 0x12,

    LocCmd = 0x20,
    LocRsp = 0x21,

    TracesApp = 0x40,
    TracesWl = 0x41,
}

impl TryFrom<u8> for TlPacketType {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x01 => Ok(TlPacketType::BleCmd),
            0x02 => Ok(TlPacketType::AclData),
            0x04 => Ok(TlPacketType::BleEvt),
            0x08 => Ok(TlPacketType::OtCmd),
            0x09 => Ok(TlPacketType::OtRsp),
            0x0A => Ok(TlPacketType::CliCmd),
            0x0C => Ok(TlPacketType::OtNot),
            0x0D => Ok(TlPacketType::OtAck),
            0x0E => Ok(TlPacketType::CliNot),
            0x0F => Ok(TlPacketType::CliAck),
            0x10 => Ok(TlPacketType::SysCmd),
            0x11 => Ok(TlPacketType::SysRsp),
            0x12 => Ok(TlPacketType::SysEvt),
            0x20 => Ok(TlPacketType::LocCmd),
            0x21 => Ok(TlPacketType::LocRsp),
            0x40 => Ok(TlPacketType::TracesApp),
            0x41 => Ok(TlPacketType::TracesWl),

            _ => Err(()),
        }
    }
}
