use super::evt::CommandStatusEvent;
use super::PacketHeader;

#[derive(PartialEq)]
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

pub const TL_PACKET_HEADER_SIZE: usize = core::mem::size_of::<PacketHeader>();
pub const TL_EVT_HEADER_SIZE: usize = 3;
pub const TL_CS_EVT_SIZE: usize = core::mem::size_of::<CommandStatusEvent>();

pub const CFG_TL_BLE_EVT_QUEUE_LENGTH: usize = 5;
pub const CFG_TL_BLE_MOST_EVENT_PAYLOAD_SIZE: usize = 255;
pub const TL_BLE_EVENT_FRAME_SIZE: usize = TL_EVT_HEADER_SIZE + CFG_TL_BLE_MOST_EVENT_PAYLOAD_SIZE;

pub const POOL_SIZE: usize = CFG_TL_BLE_EVT_QUEUE_LENGTH * 4 * divc(TL_PACKET_HEADER_SIZE + TL_BLE_EVENT_FRAME_SIZE, 4);

pub const fn divc(x: usize, y: usize) -> usize {
    (x + y - 1) / y
}

pub const TL_BLE_EVT_CS_PACKET_SIZE: usize = TL_EVT_HEADER_SIZE + TL_CS_EVT_SIZE;
#[allow(dead_code)]
pub const TL_BLE_EVT_CS_BUFFER_SIZE: usize = TL_PACKET_HEADER_SIZE + TL_BLE_EVT_CS_PACKET_SIZE;
