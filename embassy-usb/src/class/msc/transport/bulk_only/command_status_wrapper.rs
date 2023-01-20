use core::mem::size_of;
use core::ptr::copy_nonoverlapping;

/// Signature that identifies this packet as CSW
pub const CSW_SIGNATURE: u32 = 0x53425355;

/// The status of a command
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum CommandStatus {
    /// Ok, command completed successfully
    CommandOk = 0x00,
    /// Error, command failed
    CommandError = 0x01,
    /// Fatal device error, reset required
    PhaseError = 0x02,
}

/// A wrapper that identifies a command sent from the host to the
/// device on the OUT endpoint. Describes the data transfer IN or OUT
/// that should happen immediatly after this wrapper is received.
/// Little Endian
#[repr(packed)]
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct CommandStatusWrapper {
    /// Signature that identifies this packet as CSW
    /// Must contain 0x53425355
    pub signature: u32,
    /// Tag that matches this CSW back to the CBW that initiated it.
    /// Must be copied from CBW tag field. Host uses it to positively
    /// associate a CSW with the corresponding CBW
    pub tag: u32,
    /// Difference between the expected data length from CSW.data_transfer_length
    /// and the the actual amount of data sent or received. Cannot be greater
    /// than data_transfer_length. Non-zero for an OUT (host to device) transfer
    /// likely means there was an error whereas non-zero on IN (device to host) may
    /// mean the host allocated enough space for an extended/complete result but
    /// a shorter result was sent.
    pub data_residue: u32,
    /// The status of the command
    /// 0x00 = Command succeeded
    /// 0x01 = Command failed
    /// 0x02 = Phase error. Causes the host to perform a reset recovery on the
    ///        device. This indicates the device state machine has got messed up
    ///        or similar unrecoverable condition. Processing further CBWs without
    ///        a reset gives indeterminate results.
    pub status: u8,
}

impl CommandStatusWrapper {
    pub fn new(tag: u32, data_residue: u32, status: CommandStatus) -> Self {
        Self {
            signature: CSW_SIGNATURE,
            tag,
            data_residue,
            status: status as _,
        }
    }

    pub fn to_bytes<'d>(&self, buf: &'d mut [u8]) -> &'d [u8] {
        let len = size_of::<Self>();

        assert!(buf.len() >= len);
        unsafe { copy_nonoverlapping(self as *const _ as *const u8, buf.as_mut_ptr(), len) }
        &buf[..len]
    }
}
