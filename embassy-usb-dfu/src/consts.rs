
pub(crate) const USB_CLASS_APPN_SPEC: u8 = 0xFE;
pub(crate) const APPN_SPEC_SUBCLASS_DFU: u8 = 0x01;
#[allow(unused)]
pub(crate) const DFU_PROTOCOL_DFU: u8 = 0x02;
#[allow(unused)]
pub(crate) const DFU_PROTOCOL_RT: u8 = 0x01;
pub(crate) const DESC_DFU_FUNCTIONAL: u8 = 0x21;

#[cfg(feature = "defmt")]
defmt::bitflags! {
    pub struct DfuAttributes: u8 {
        const WILL_DETACH = 0b0000_1000;
        const MANIFESTATION_TOLERANT = 0b0000_0100;
        const CAN_UPLOAD = 0b0000_0010;
        const CAN_DOWNLOAD = 0b0000_0001;
    }
}

#[cfg(not(feature = "defmt"))]
bitflags::bitflags! {
    pub struct DfuAttributes: u8 {
        const WILL_DETACH = 0b0000_1000;
        const MANIFESTATION_TOLERANT = 0b0000_0100;
        const CAN_UPLOAD = 0b0000_0010;
        const CAN_DOWNLOAD = 0b0000_0001;
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
#[allow(unused)]
pub enum State {
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

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
#[allow(unused)]
pub enum Status {
    Ok = 0x00,
    ErrTarget = 0x01,
    ErrFile = 0x02,
    ErrWrite = 0x03,
    ErrErase = 0x04,
    ErrCheckErased = 0x05,
    ErrProg = 0x06,
    ErrVerify = 0x07,
    ErrAddress = 0x08,
    ErrNotDone = 0x09,
    ErrFirmware = 0x0A,
    ErrVendor = 0x0B,
    ErrUsbr = 0x0C,
    ErrPor = 0x0D,
    ErrUnknown = 0x0E,
    ErrStalledPkt = 0x0F,
}

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum Request {
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
