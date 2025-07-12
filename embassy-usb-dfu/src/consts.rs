//! USB DFU constants, taken from
//! https://www.usb.org/sites/default/files/DFU_1.1.pdf
/// Device Firmware Upgrade class, App specific
pub(crate) const USB_CLASS_APPN_SPEC: u8 = 0xFE;
/// Device Firmware Upgrade subclass, App specific
pub(crate) const APPN_SPEC_SUBCLASS_DFU: u8 = 0x01;
#[allow(unused)]
/// USB interface alternative setting
pub(crate) const DFU_PROTOCOL_DFU: u8 = 0x02;
#[allow(unused)]
/// DFU runtime class
pub(crate) const DFU_PROTOCOL_RT: u8 = 0x01;
/// DFU functional descriptor
pub(crate) const DESC_DFU_FUNCTIONAL: u8 = 0x21;

macro_rules! define_dfu_attributes {
    ($macro:path) => {
        $macro! {
            /// Attributes supported by the DFU controller.
            pub struct DfuAttributes: u8 {
                /// Generate WillDetache sequence on bus.
                const WILL_DETACH = 0b0000_1000;
                /// Device can communicate during manifestation phase.
                const MANIFESTATION_TOLERANT = 0b0000_0100;
                /// Capable of upload.
                const CAN_UPLOAD = 0b0000_0010;
                /// Capable of download.
                const CAN_DOWNLOAD = 0b0000_0001;
            }
        }
    };
}

#[cfg(feature = "defmt")]
define_dfu_attributes!(defmt::bitflags);

#[cfg(not(feature = "defmt"))]
define_dfu_attributes!(bitflags::bitflags);

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
#[allow(unused)]
/// An indication of the state that the device is going to enter immediately following transmission of this response.
pub(crate) enum State {
    /// Device is running its normal application.
    AppIdle = 0,
    /// Device is running its normal application, has received the DFU_DETACH request, and is waiting for a USB reset.
    AppDetach = 1,
    /// Device is operating in the DFU mode and is waiting for requests.
    DfuIdle = 2,
    /// Device has received a block and is waiting for the host to solicit the status via DFU_GETSTATUS.
    DlSync = 3,
    /// Device is programming a control-write block into its nonvolatile memories.
    DlBusy = 4,
    /// Device is processing a download operation. Expecting DFU_DNLOAD requests.
    Download = 5,
    /// Device has received the final block of firmware from the host, waits for DFU_GETSTATUS to start Manifestation phase or completed this phase
    ManifestSync = 6,
    /// Device is in the Manifestation phase. Not all devices will be able to respond to DFU_GETSTATUS when in this state.
    Manifest = 7,
    /// Device has programmed its memories and is waiting for a USB reset or a power on reset.
    ManifestWaitReset = 8,
    /// The device is processing an upload operation. Expecting DFU_UPLOAD requests.
    UploadIdle = 9,
    /// An error has occurred. Awaiting the DFU_CLRSTATUS request.
    Error = 10,
}

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
#[allow(unused)]
/// An indication of the status resulting from the execution of the most recent request.
pub(crate) enum Status {
    /// No error
    Ok = 0x00,
    /// File is not targeted for use by this device
    ErrTarget = 0x01,
    /// File is for this device but fails some vendor-specific verification test
    ErrFile = 0x02,
    /// Device is unable to write memory
    ErrWrite = 0x03,
    /// Memory erase function failed
    ErrErase = 0x04,
    /// Memory erase check failed
    ErrCheckErased = 0x05,
    /// Program memory function failed
    ErrProg = 0x06,
    /// Programmed memory failed verification
    ErrVerify = 0x07,
    /// Cannot program memory due to received address that is out of range
    ErrAddress = 0x08,
    /// Received DFU_DNLOAD with wLength = 0, but device does not think it has all of the data yet
    ErrNotDone = 0x09,
    /// Device’s firmware is corrupt. It cannot return to run-time (non-DFU) operations
    ErrFirmware = 0x0A,
    /// iString indicates a vendor-specific error
    ErrVendor = 0x0B,
    /// Device detected unexpected USB reset signaling
    ErrUsbr = 0x0C,
    /// Device detected unexpected power on reset
    ErrPor = 0x0D,
    /// Something went wrong, but the device does not know what
    ErrUnknown = 0x0E,
    /// Device stalled an unexpected request
    ErrStalledPkt = 0x0F,
}

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
/// DFU requests
pub(crate) enum Request {
    /// Host instructs the device to generate a detach-attach sequence
    Detach = 0,
    /// Host initiates control-write transfers with this request, and sends a DFU_DNLOAD request
    /// with  wLength = 0 to indicate that it has completed transferring the firmware image file
    Dnload = 1,
    /// The DFU_UPLOAD request is employed by the host to solicit firmware from the device.
    Upload = 2,
    /// The host employs the DFU_GETSTATUS request to facilitate synchronization with the device.
    GetStatus = 3,
    ///  Any time the device detects an error, it waits with transition until ClrStatus
    ClrStatus = 4,
    /// Requests a report about a state of the device
    GetState = 5,
    /// Enables the host to exit from certain states and return to the DFU_IDLE state
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
