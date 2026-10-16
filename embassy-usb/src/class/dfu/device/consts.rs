//! USB DFU constants and types.

pub(crate) const USB_CLASS_APPN_SPEC: u8 = 0xFE;
pub(crate) const APPN_SPEC_SUBCLASS_DFU: u8 = 0x01;
#[allow(unused)]
pub(crate) const DFU_PROTOCOL_DFU: u8 = 0x02;
#[allow(unused)]
pub(crate) const DFU_PROTOCOL_RT: u8 = 0x01;
pub(crate) const DESC_DFU_FUNCTIONAL: u8 = 0x21;

#[cfg(feature = "defmt")]
defmt::bitflags! {
    /// Attributes supported by the DFU controller.
    pub struct DfuAttributes: u8 {
        /// Generate WillDetach sequence on bus.
        const WILL_DETACH = 0b0000_1000;
        /// Device can communicate during manifestation phase.
        const MANIFESTATION_TOLERANT = 0b0000_0100;
        /// Capable of upload.
        const CAN_UPLOAD = 0b0000_0010;
        /// Capable of download.
        const CAN_DOWNLOAD = 0b0000_0001;
    }
}

#[cfg(not(feature = "defmt"))]
bitflags::bitflags! {
    /// Attributes supported by the DFU controller.
    pub struct DfuAttributes: u8 {
        /// Generate WillDetach sequence on bus.
        const WILL_DETACH = 0b0000_1000;
        /// Device can communicate during manifestation phase.
        const MANIFESTATION_TOLERANT = 0b0000_0100;
        /// Capable of upload.
        const CAN_UPLOAD = 0b0000_0010;
        /// Capable of download.
        const CAN_DOWNLOAD = 0b0000_0001;
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
#[allow(unused)]
pub(crate) enum State {
    AppIdle = 0,
    AppDetach = 1,
    DfuIdle = 2,
    DlSync = 3,
    DlBusy = 4,
    Download = 5,
    ManifestSync = 6,
    Manifest = 7,
    ManifestWaitReset = 8,
    UploadIdle = 9,
    Error = 10,
}

/// DFU status codes indicating the result of the most recent request.
#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
#[allow(unused)]
pub enum Status {
    /// No error.
    Ok = 0x00,
    /// File is not targeted for use by this device.
    ErrTarget = 0x01,
    /// File is for this device but fails some vendor-specific verification test.
    ErrFile = 0x02,
    /// Device is unable to write memory.
    ErrWrite = 0x03,
    /// Memory erase function failed.
    ErrErase = 0x04,
    /// Memory erase check failed.
    ErrCheckErased = 0x05,
    /// Program memory function failed.
    ErrProg = 0x06,
    /// Programmed memory failed verification.
    ErrVerify = 0x07,
    /// Cannot program memory due to received address that is out of range.
    ErrAddress = 0x08,
    /// Received DFU_DNLOAD with wLength = 0, but device does not think it has all of the data yet.
    ErrNotDone = 0x09,
    /// Device's firmware is corrupt. It cannot return to run-time (non-DFU) operations.
    ErrFirmware = 0x0A,
    /// iString indicates a vendor-specific error.
    ErrVendor = 0x0B,
    /// Device detected unexpected USB reset signaling.
    ErrUsbr = 0x0C,
    /// Device detected unexpected power on reset.
    ErrPor = 0x0D,
    /// Something went wrong, but the device does not know what.
    ErrUnknown = 0x0E,
    /// Device stalled an unexpected request.
    ErrStalledPkt = 0x0F,
}

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum Request {
    Detach = 0,
    Dnload = 1,
    Upload = 2,
    GetStatus = 3,
    ClrStatus = 4,
    GetState = 5,
    Abort = 6,
}

impl TryFrom<u8> for Request {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Request::Detach),
            1 => Ok(Request::Dnload),
            2 => Ok(Request::Upload),
            3 => Ok(Request::GetStatus),
            4 => Ok(Request::ClrStatus),
            5 => Ok(Request::GetState),
            6 => Ok(Request::Abort),
            _ => Err(()),
        }
    }
}
