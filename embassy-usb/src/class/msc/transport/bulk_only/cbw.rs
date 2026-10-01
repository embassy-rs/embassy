use core::mem::{size_of, size_of_val};

use embassy_usb_driver::Direction;

/// Signature that identifies this packet as CBW
pub const CBW_SIGNATURE: u32 = 0x43425355;

/// A wrapper that identifies a command sent from the host to the
/// device on the OUT endpoint. Describes the data transfer IN or OUT
/// that should happen immediatly after this wrapper is received.
/// Little Endian
#[repr(packed)]
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct CommandBlockWrapper {
    /// Signature that identifies this packet as CBW
    /// Must contain 0x43425355
    pub signature: u32,
    /// Tag sent by the host. Must be echoed back to host in tag
    /// field of the command status wrapper sent after the command
    /// has been executed/rejected. Host uses it to positively
    /// associate a CSW with the corresponding CBW
    pub tag: u32,
    /// Number of bytes of data that the host expects to receive on
    /// the IN or OUT endpoint (as indicated by the direction field)
    /// during the execution of this command. If this field is zero,
    /// must respond directly with CSW
    pub data_transfer_length: u32,
    /// Direction of transfer initiated by this command.
    /// 0b0XXXXXXX = OUT from host to device
    /// 0b1XXXXXXX = IN from device to host
    /// X bits are obsolete or reserved
    pub direction: u8,
    /// The device Logical Unit Number (LUN) to which the command is
    /// for. For devices that don't support multiple LUNs the host will
    /// set this field to zero.
    /// Devices that don't support multiple LUNS must not ignore this
    /// field and apply all commands to LUN 0, [see General Problems with Commands](http://janaxelson.com/device_errors.htm)
    pub lun: u8,
    /// The number of valid bytes in data field
    pub data_length: u8,
    /// The command set specific data for this command
    pub data: [u8; 16],
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum CommandBlockWrapperDeserializeError {
    BufferTooShort,
    InvalidSignature,
    InvalidDirection,
    InvalidDataLength,
}

impl CommandBlockWrapper {
    pub fn from_bytes(buf: &[u8]) -> Result<CommandBlockWrapper, CommandBlockWrapperDeserializeError> {
        if buf.len() < size_of::<Self>() {
            return Err(CommandBlockWrapperDeserializeError::BufferTooShort);
        }

        let cbw = unsafe { core::ptr::read(buf.as_ptr() as *const Self) };

        if cbw.signature != CBW_SIGNATURE {
            return Err(CommandBlockWrapperDeserializeError::InvalidSignature);
        }

        if cbw.direction & 0b01111111 != 0 {
            return Err(CommandBlockWrapperDeserializeError::InvalidDirection);
        }

        if cbw.data_length as usize > size_of_val(&cbw.data) {
            return Err(CommandBlockWrapperDeserializeError::InvalidDataLength);
        }

        Ok(cbw)
    }

    pub fn dir(&self) -> Direction {
        if self.direction == 0x80 {
            Direction::In
        } else {
            Direction::Out
        }
    }

    pub fn data(&self) -> &[u8] {
        &self.data[..self.data_length as usize]
    }
}
