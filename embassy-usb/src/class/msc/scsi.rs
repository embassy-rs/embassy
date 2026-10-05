//! SCSI transparent command set (SPC-3, SBC-3) shared by the device and host MSC classes.

// Opcodes.
pub(crate) const SCSI_TEST_UNIT_READY: u8 = 0x00;
pub(crate) const SCSI_REQUEST_SENSE: u8 = 0x03;
pub(crate) const SCSI_INQUIRY: u8 = 0x12;
pub(crate) const SCSI_MODE_SENSE_6: u8 = 0x1a;
pub(crate) const SCSI_START_STOP_UNIT: u8 = 0x1b;
pub(crate) const SCSI_PREVENT_ALLOW_MEDIUM_REMOVAL: u8 = 0x1e;
pub(crate) const SCSI_READ_FORMAT_CAPACITIES: u8 = 0x23;
pub(crate) const SCSI_READ_CAPACITY_10: u8 = 0x25;
pub(crate) const SCSI_READ_10: u8 = 0x28;
pub(crate) const SCSI_WRITE_10: u8 = 0x2a;
pub(crate) const SCSI_SYNCHRONIZE_CACHE_10: u8 = 0x35;
pub(crate) const SCSI_MODE_SENSE_10: u8 = 0x5a;
pub(crate) const SCSI_READ_16: u8 = 0x88;
pub(crate) const SCSI_WRITE_16: u8 = 0x8a;
pub(crate) const SCSI_SERVICE_ACTION_IN_16: u8 = 0x9e;
pub(crate) const SCSI_SA_READ_CAPACITY_16: u8 = 0x10;

// Additional sense codes.
pub(crate) const ASC_WRITE_ERROR: u8 = 0x0c;
pub(crate) const ASC_UNRECOVERED_READ_ERROR: u8 = 0x11;
pub(crate) const ASC_INVALID_COMMAND_OPERATION_CODE: u8 = 0x20;
pub(crate) const ASC_LOGICAL_BLOCK_ADDRESS_OUT_OF_RANGE: u8 = 0x21;
pub(crate) const ASC_INVALID_FIELD_IN_CDB: u8 = 0x24;
pub(crate) const ASC_LOGICAL_UNIT_NOT_SUPPORTED: u8 = 0x25;
pub(crate) const ASC_WRITE_PROTECTED: u8 = 0x27;
pub(crate) const ASCQ_NONE: u8 = 0x00;

// Sense keys under the names the device class uses.
pub(crate) const SENSE_KEY_MEDIUM_ERROR: SenseKey = SenseKey::MediumError;
pub(crate) const SENSE_KEY_ILLEGAL_REQUEST: SenseKey = SenseKey::IllegalRequest;
pub(crate) const SENSE_KEY_DATA_PROTECT: SenseKey = SenseKey::DataProtect;

/// SCSI sense key (SPC-3 §4.5.6, Table 27).
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum SenseKey {
    /// `0x0` — no error.
    NoSense = 0x0,
    /// `0x1` — command succeeded with automatic recovery.
    RecoveredError = 0x1,
    /// `0x2` — the medium is not ready.
    NotReady = 0x2,
    /// `0x3` — unrecoverable medium error.
    MediumError = 0x3,
    /// `0x4` — non-medium hardware error.
    HardwareError = 0x4,
    /// `0x5` — illegal CDB or parameter.
    IllegalRequest = 0x5,
    /// `0x6` — reset, medium change, or parameter change.
    UnitAttention = 0x6,
    /// `0x7` — write-protected medium.
    DataProtect = 0x7,
    /// `0x8` — blank medium on a device that expected data.
    BlankCheck = 0x8,
    /// `0x9` — vendor-specific.
    VendorSpecific = 0x9,
    /// `0xA` — COPY/COMPARE aborted.
    CopyAborted = 0xA,
    /// `0xB` — target aborted the command.
    AbortedCommand = 0xB,
    /// `0xD` — volume overflow on a sequential device.
    VolumeOverflow = 0xD,
    /// `0xE` — data did not match expected values.
    Miscompare = 0xE,
    /// Any reserved sense key value.
    Reserved = 0xF,
}

impl SenseKey {
    pub(crate) fn from_bits(b: u8) -> Self {
        match b & 0x0F {
            0x0 => Self::NoSense,
            0x1 => Self::RecoveredError,
            0x2 => Self::NotReady,
            0x3 => Self::MediumError,
            0x4 => Self::HardwareError,
            0x5 => Self::IllegalRequest,
            0x6 => Self::UnitAttention,
            0x7 => Self::DataProtect,
            0x8 => Self::BlankCheck,
            0x9 => Self::VendorSpecific,
            0xA => Self::CopyAborted,
            0xB => Self::AbortedCommand,
            0xD => Self::VolumeOverflow,
            0xE => Self::Miscompare,
            _ => Self::Reserved,
        }
    }
}

/// Decoded fixed-format sense data (SPC-3 §4.5.3).
///
/// Use the raw REQUEST SENSE response if more detail is needed.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct SenseData {
    /// Sense key (byte 2, bits 0..3).
    pub key: SenseKey,
    /// Additional Sense Code (byte 12).
    pub asc: u8,
    /// Additional Sense Code Qualifier (byte 13).
    pub ascq: u8,
}

impl SenseData {
    pub(crate) const NO_SENSE: Self = Self {
        key: SenseKey::NoSense,
        asc: 0,
        ascq: 0,
    };
}
