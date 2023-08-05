use core::convert::TryFrom;

use crate::evt::CsEvt;
use crate::PacketHeader;

#[derive(Debug)]
#[repr(C)]
pub enum TlPacketType {
    MacCmd = 0x00,

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
pub const TL_CS_EVT_SIZE: usize = core::mem::size_of::<CsEvt>();

/**
 * Queue length of BLE Event
 * This parameter defines the number of asynchronous events that can be stored in the HCI layer before
 * being reported to the application. When a command is sent to the BLE core coprocessor, the HCI layer
 * is waiting for the event with the Num_HCI_Command_Packets set to 1. The receive queue shall be large
 * enough to store all asynchronous events received in between.
 * When CFG_TLBLE_MOST_EVENT_PAYLOAD_SIZE is set to 27, this allow to store three 255 bytes long asynchronous events
 * between the HCI command and its event.
 * This parameter depends on the value given to CFG_TLBLE_MOST_EVENT_PAYLOAD_SIZE. When the queue size is to small,
 * the system may hang if the queue is full with asynchronous events and the HCI layer is still waiting
 * for a CC/CS event, In that case, the notification TL_BLE_HCI_ToNot() is called to indicate
 * to the application a HCI command did not receive its command event within 30s (Default HCI Timeout).
 */
pub const CFG_TL_BLE_EVT_QUEUE_LENGTH: usize = 5;
pub const CFG_TL_BLE_MOST_EVENT_PAYLOAD_SIZE: usize = 255;
pub const TL_BLE_EVENT_FRAME_SIZE: usize = TL_EVT_HEADER_SIZE + CFG_TL_BLE_MOST_EVENT_PAYLOAD_SIZE;

pub const POOL_SIZE: usize = CFG_TL_BLE_EVT_QUEUE_LENGTH * 4 * divc(TL_PACKET_HEADER_SIZE + TL_BLE_EVENT_FRAME_SIZE, 4);
pub const C_SIZE_CMD_STRING: usize = 256;

pub const fn divc(x: usize, y: usize) -> usize {
    (x + y - 1) / y
}

pub const TL_BLE_EVT_CS_PACKET_SIZE: usize = TL_EVT_HEADER_SIZE + TL_CS_EVT_SIZE;
#[allow(dead_code)]
pub const TL_BLE_EVT_CS_BUFFER_SIZE: usize = TL_PACKET_HEADER_SIZE + TL_BLE_EVT_CS_PACKET_SIZE;

pub const TL_BLEEVT_CC_OPCODE: u8 = 0x0E;
pub const TL_BLEEVT_CS_OPCODE: u8 = 0x0F;
pub const TL_BLEEVT_VS_OPCODE: u8 = 0xFF;
