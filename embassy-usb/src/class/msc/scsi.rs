//! SCSI transparent command set (SPC-3, SBC-3) shared by the device and host MSC classes.

// Opcodes.
pub(crate) const SCSI_TEST_UNIT_READY: u8 = 0x00;
pub(crate) const SCSI_REQUEST_SENSE: u8 = 0x03;
pub(crate) const SCSI_READ_6: u8 = 0x08;
pub(crate) const SCSI_WRITE_6: u8 = 0x0a;
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
pub(crate) const SCSI_REPORT_LUNS: u8 = 0xa0;
pub(crate) const SCSI_READ_12: u8 = 0xa8;
pub(crate) const SCSI_WRITE_12: u8 = 0xaa;
pub(crate) const SCSI_SA_READ_CAPACITY_16: u8 = 0x10;

// INQUIRY CDB byte 1 (SPC-3 §6.4).
/// Enable Vital Product Data.
pub(crate) const INQUIRY_EVPD: u8 = 0x01;
/// Command Support Data (obsolete).
pub(crate) const INQUIRY_CMDDT: u8 = 0x02;

// START STOP UNIT CDB byte 4 (SBC-3 §5.25).
pub(crate) const SSU_START: u8 = 0x01;
pub(crate) const SSU_LOEJ: u8 = 0x02;
pub(crate) const SSU_POWER_CONDITION_MASK: u8 = 0xf0;

// READ(6) / WRITE(6) CDB (SBC-3).
/// Byte 1 bits holding the top of the 21-bit LBA.
pub(crate) const RW6_LBA_MSB_MASK: u8 = 0x1f;
/// Transfer length 0 in READ(6) / WRITE(6) means 256 blocks.
pub(crate) const RW6_ZERO_LENGTH_BLOCKS: u32 = 256;

// SERVICE ACTION IN(16) CDB (SPC-4).
/// Byte 1 bits holding the service action.
pub(crate) const SERVICE_ACTION_MASK: u8 = 0x1f;

// REPORT LUNS CDB byte 2, SELECT REPORT (SPC-4 §6.33).
/// All logical units, excluding well-known logical units.
pub(crate) const RL_SELECT_ALL: u8 = 0x00;
/// Well-known logical units only.
pub(crate) const RL_SELECT_WELL_KNOWN: u8 = 0x01;
/// All logical units, including well-known logical units.
pub(crate) const RL_SELECT_ALL_AND_WELL_KNOWN: u8 = 0x02;
/// Allocation lengths below this are rejected.
pub(crate) const RL_MIN_ALLOCATION_LENGTH: usize = 4;

// Standard INQUIRY data (SPC-3 §6.4.2).
pub(crate) const INQUIRY_DATA_LEN: usize = 36;
/// Byte 0 bits holding the peripheral device type.
pub(crate) const PERIPHERAL_DEVICE_TYPE_MASK: u8 = 0x1f;
/// Byte 0: peripheral qualifier 000b, direct-access block device.
pub(crate) const PERIPHERAL_DIRECT_ACCESS: u8 = 0x00;
/// Byte 0: peripheral qualifier 011b (no device on this logical unit), unknown device type.
pub(crate) const PERIPHERAL_NO_LUN: u8 = 0x7f;
/// Byte 1: removable medium.
pub(crate) const INQUIRY_RMB: u8 = 0x80;
/// Byte 2: conforms to SPC-2.
pub(crate) const INQUIRY_VERSION_SPC2: u8 = 0x04;
/// Byte 3: response data format.
pub(crate) const INQUIRY_RESPONSE_DATA_FORMAT: u8 = 0x02;

// Vital product data pages (SPC-3 §7.6).
pub(crate) const VPD_PAGE_SUPPORTED_PAGES: u8 = 0x00;
pub(crate) const VPD_PAGE_UNIT_SERIAL_NUMBER: u8 = 0x80;
pub(crate) const VPD_PAGE_DEVICE_IDENTIFICATION: u8 = 0x83;
/// Identification descriptor code set: ASCII.
pub(crate) const VPD_CODE_SET_ASCII: u8 = 0x02;
/// Identification descriptor association and type: logical unit, vendor-specific.
pub(crate) const VPD_DESIGNATOR_VENDOR_SPECIFIC: u8 = 0x00;

// MODE SENSE parameter header (SPC-3 §7.4.3).
/// Device-specific parameter bit: the medium is write-protected.
pub(crate) const MODE_DSP_WP: u8 = 0x80;

// READ FORMAT CAPACITIES data (UFI §4.10).
/// Descriptor type: formatted media, current capacity.
pub(crate) const RFC_FORMATTED_MEDIA: u8 = 0x02;

// Additional sense codes.
pub(crate) const ASC_WRITE_ERROR: u8 = 0x0c;
pub(crate) const ASC_UNRECOVERED_READ_ERROR: u8 = 0x11;
pub(crate) const ASC_INVALID_COMMAND_OPERATION_CODE: u8 = 0x20;
pub(crate) const ASC_LOGICAL_BLOCK_ADDRESS_OUT_OF_RANGE: u8 = 0x21;
pub(crate) const ASC_INVALID_FIELD_IN_CDB: u8 = 0x24;
pub(crate) const ASC_LOGICAL_UNIT_NOT_SUPPORTED: u8 = 0x25;
pub(crate) const ASC_WRITE_PROTECTED: u8 = 0x27;
pub(crate) const ASCQ_NONE: u8 = 0x00;

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

/// Fixed-format sense data byte 2 bits holding the sense key.
const SENSE_KEY_MASK: u8 = 0x0f;

/// Length of fixed-format sense data.
pub(crate) const SENSE_DATA_LEN: usize = 18;
/// Response code: current error, fixed format.
const SENSE_RESPONSE_CODE_CURRENT: u8 = 0x70;
/// Bytes following the additional sense length field (byte 7).
const SENSE_ADDITIONAL_LENGTH: u8 = 10;

impl SenseKey {
    pub(crate) fn from_bits(b: u8) -> Self {
        match b & SENSE_KEY_MASK {
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
    pub(crate) const NO_SENSE: Self = Self::new(SenseKey::NoSense, 0);
    pub(crate) const INVALID_COMMAND_OPERATION_CODE: Self =
        Self::new(SenseKey::IllegalRequest, ASC_INVALID_COMMAND_OPERATION_CODE);
    pub(crate) const INVALID_FIELD_IN_CDB: Self = Self::new(SenseKey::IllegalRequest, ASC_INVALID_FIELD_IN_CDB);
    pub(crate) const LOGICAL_BLOCK_ADDRESS_OUT_OF_RANGE: Self =
        Self::new(SenseKey::IllegalRequest, ASC_LOGICAL_BLOCK_ADDRESS_OUT_OF_RANGE);
    pub(crate) const LOGICAL_UNIT_NOT_SUPPORTED: Self =
        Self::new(SenseKey::IllegalRequest, ASC_LOGICAL_UNIT_NOT_SUPPORTED);
    pub(crate) const WRITE_PROTECTED: Self = Self::new(SenseKey::DataProtect, ASC_WRITE_PROTECTED);
    pub(crate) const WRITE_ERROR: Self = Self::new(SenseKey::MediumError, ASC_WRITE_ERROR);
    pub(crate) const UNRECOVERED_READ_ERROR: Self = Self::new(SenseKey::MediumError, ASC_UNRECOVERED_READ_ERROR);

    const fn new(key: SenseKey, asc: u8) -> Self {
        Self {
            key,
            asc,
            ascq: ASCQ_NONE,
        }
    }

    pub(crate) fn parse(raw: &[u8; SENSE_DATA_LEN]) -> Self {
        Self {
            key: SenseKey::from_bits(raw[2]),
            asc: raw[12],
            ascq: raw[13],
        }
    }

    pub(crate) fn encode(&self) -> [u8; SENSE_DATA_LEN] {
        let mut out = [0u8; SENSE_DATA_LEN];
        out[0] = SENSE_RESPONSE_CODE_CURRENT;
        out[2] = self.key as u8;
        out[7] = SENSE_ADDITIONAL_LENGTH;
        out[12] = self.asc;
        out[13] = self.ascq;
        out
    }
}

/// READ CAPACITY(10) / READ CAPACITY(16) parameter data (SBC-3 §5.10, §5.11).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub(crate) struct Capacity {
    pub last_lba: u64,
    pub block_size: u32,
}

impl Capacity {
    pub const LEN_10: usize = 8;
    pub const LEN_16: usize = 32;
    /// READ CAPACITY(10) reports this LBA when the last one doesn't fit in 32 bits.
    pub const LBA_10_OVERFLOW: u32 = u32::MAX;

    pub fn parse_10(raw: &[u8; Self::LEN_10]) -> Self {
        Self {
            last_lba: be32(&raw[0..4]) as u64,
            block_size: be32(&raw[4..8]),
        }
    }

    pub fn encode_10(&self) -> [u8; Self::LEN_10] {
        let last_lba = u32::try_from(self.last_lba).unwrap_or(Self::LBA_10_OVERFLOW);
        let mut out = [0u8; Self::LEN_10];
        out[0..4].copy_from_slice(&last_lba.to_be_bytes());
        out[4..8].copy_from_slice(&self.block_size.to_be_bytes());
        out
    }

    pub fn parse_16(raw: &[u8; Self::LEN_16]) -> Self {
        Self {
            last_lba: (be32(&raw[0..4]) as u64) << 32 | be32(&raw[4..8]) as u64,
            block_size: be32(&raw[8..12]),
        }
    }

    pub fn encode_16(&self) -> [u8; Self::LEN_16] {
        let mut out = [0u8; Self::LEN_16];
        out[0..8].copy_from_slice(&self.last_lba.to_be_bytes());
        out[8..12].copy_from_slice(&self.block_size.to_be_bytes());
        out
    }
}

fn be32(b: &[u8]) -> u32 {
    u32::from_be_bytes([b[0], b[1], b[2], b[3]])
}

/// A decoded READ or WRITE command (6, 10, 12 or 16 byte CDB).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub(crate) struct ReadWrite {
    pub write: bool,
    pub lba: u64,
    pub blocks: u32,
}

impl ReadWrite {
    /// Decodes `cb` if it holds a READ or WRITE command, else returns `None`.
    ///
    /// `cb` is the command block as long as the host sent it, so a truncated CDB returns `None`.
    pub fn parse(cb: &[u8]) -> Option<Self> {
        let cdb_len = match *cb.first()? {
            SCSI_READ_6 | SCSI_WRITE_6 => 6,
            SCSI_READ_10 | SCSI_WRITE_10 => 10,
            SCSI_READ_12 | SCSI_WRITE_12 => 12,
            SCSI_READ_16 | SCSI_WRITE_16 => 16,
            _ => return None,
        };
        if cb.len() < cdb_len {
            return None;
        }

        let (lba, blocks) = match cb[0] {
            SCSI_READ_6 | SCSI_WRITE_6 => {
                let lba = u32::from_be_bytes([0, cb[1] & RW6_LBA_MSB_MASK, cb[2], cb[3]]);
                let blocks = match cb[4] {
                    0 => RW6_ZERO_LENGTH_BLOCKS,
                    n => n as u32,
                };
                (lba as u64, blocks)
            }
            SCSI_READ_10 | SCSI_WRITE_10 => (be32(&cb[2..6]) as u64, u16::from_be_bytes([cb[7], cb[8]]) as u32),
            SCSI_READ_12 | SCSI_WRITE_12 => (be32(&cb[2..6]) as u64, be32(&cb[6..10])),
            SCSI_READ_16 | SCSI_WRITE_16 => {
                let lba = (be32(&cb[2..6]) as u64) << 32 | be32(&cb[6..10]) as u64;
                (lba, be32(&cb[10..14]))
            }
            _ => return None,
        };
        let write = matches!(cb[0], SCSI_WRITE_6 | SCSI_WRITE_10 | SCSI_WRITE_12 | SCSI_WRITE_16);
        Some(Self { write, lba, blocks })
    }

    /// Encodes the command into `buf` and returns the CDB, a prefix of `buf`.
    ///
    /// Uses READ(10) / WRITE(10) when the LBA and block count fit, for compatibility, else the 16-byte form.
    pub fn encode<'a>(&self, buf: &'a mut [u8; 16]) -> &'a [u8] {
        if let (Ok(lba), Ok(blocks)) = (u32::try_from(self.lba), u16::try_from(self.blocks)) {
            let op = if self.write { SCSI_WRITE_10 } else { SCSI_READ_10 };
            buf[..10].copy_from_slice(&rw10_cdb(op, lba, blocks));
            &buf[..10]
        } else {
            let op = if self.write { SCSI_WRITE_16 } else { SCSI_READ_16 };
            *buf = rw16_cdb(op, self.lba, self.blocks);
            buf
        }
    }
}

/// A decoded INQUIRY command.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub(crate) struct Inquiry {
    pub evpd: bool,
    pub cmd_dt: bool,
    pub page_code: u8,
    pub allocation_length: usize,
}

/// A decoded SCSI command, as received by the device.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub(crate) enum Command {
    TestUnitReady,
    RequestSense {
        allocation_length: usize,
    },
    Inquiry(Inquiry),
    ModeSense6 {
        allocation_length: usize,
    },
    ModeSense10 {
        allocation_length: usize,
    },
    ReadFormatCapacities {
        allocation_length: usize,
    },
    ReadCapacity10,
    ReadCapacity16 {
        allocation_length: usize,
    },
    Read(ReadWrite),
    Write(ReadWrite),
    StartStopUnit {
        eject: bool,
    },
    PreventAllowMediumRemoval,
    SynchronizeCache10,
    ReportLuns {
        select_report: u8,
        allocation_length: usize,
    },
    /// An opcode this class doesn't implement, or a READ / WRITE / SERVICE ACTION IN(16) CDB
    /// shorter than its opcode requires.
    Unsupported {
        opcode: u8,
    },
}

/// The data phase a command expects (MSC BBB r1.0 §6.7).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub(crate) enum DataPhase {
    NoData,
    DataIn,
    DataOut,
}

impl Command {
    /// Decodes the command block of a CBW, of which the host sent the first `len` bytes.
    pub fn parse(cb: &[u8; 16], len: usize) -> Self {
        if let Some(rw) = ReadWrite::parse(&cb[..len]) {
            return if rw.write { Self::Write(rw) } else { Self::Read(rw) };
        }

        let be16 = |i: usize| u16::from_be_bytes([cb[i], cb[i + 1]]) as usize;
        let be32 = |i: usize| u32::from_be_bytes([cb[i], cb[i + 1], cb[i + 2], cb[i + 3]]) as usize;
        match cb[0] {
            SCSI_TEST_UNIT_READY => Self::TestUnitReady,
            SCSI_REQUEST_SENSE => Self::RequestSense {
                allocation_length: cb[4] as usize,
            },
            SCSI_INQUIRY => Self::Inquiry(Inquiry {
                evpd: cb[1] & INQUIRY_EVPD != 0,
                cmd_dt: cb[1] & INQUIRY_CMDDT != 0,
                page_code: cb[2],
                allocation_length: be16(3),
            }),
            SCSI_MODE_SENSE_6 => Self::ModeSense6 {
                allocation_length: cb[4] as usize,
            },
            SCSI_MODE_SENSE_10 => Self::ModeSense10 {
                allocation_length: be16(7),
            },
            SCSI_READ_FORMAT_CAPACITIES => Self::ReadFormatCapacities {
                allocation_length: be16(7),
            },
            SCSI_READ_CAPACITY_10 => Self::ReadCapacity10,
            SCSI_SERVICE_ACTION_IN_16 if len >= 16 && cb[1] & SERVICE_ACTION_MASK == SCSI_SA_READ_CAPACITY_16 => {
                Self::ReadCapacity16 {
                    allocation_length: be32(10),
                }
            }
            SCSI_START_STOP_UNIT => Self::StartStopUnit { eject: is_eject(cb[4]) },
            SCSI_PREVENT_ALLOW_MEDIUM_REMOVAL => Self::PreventAllowMediumRemoval,
            SCSI_SYNCHRONIZE_CACHE_10 => Self::SynchronizeCache10,
            SCSI_REPORT_LUNS => Self::ReportLuns {
                select_report: cb[2],
                allocation_length: be32(6),
            },
            opcode => Self::Unsupported { opcode },
        }
    }

    /// The data phase this command expects, or `None` for unsupported commands.
    pub fn data_phase(&self) -> Option<DataPhase> {
        match self {
            Self::TestUnitReady
            | Self::StartStopUnit { .. }
            | Self::PreventAllowMediumRemoval
            | Self::SynchronizeCache10 => Some(DataPhase::NoData),
            Self::RequestSense { .. }
            | Self::Inquiry(_)
            | Self::ModeSense6 { .. }
            | Self::ModeSense10 { .. }
            | Self::ReadFormatCapacities { .. }
            | Self::ReadCapacity10
            | Self::ReadCapacity16 { .. }
            | Self::Read(_)
            | Self::ReportLuns { .. } => Some(DataPhase::DataIn),
            Self::Write(_) => Some(DataPhase::DataOut),
            Self::Unsupported { .. } => None,
        }
    }
}

/// Whether START STOP UNIT CDB byte 4 asks to eject the medium (SBC-3 §5.25).
///
/// LoEj and Start are ignored unless the power condition field is zero.
fn is_eject(byte4: u8) -> bool {
    let power_condition = byte4 & SSU_POWER_CONDITION_MASK;
    let load_eject = byte4 & SSU_LOEJ != 0;
    let start = byte4 & SSU_START != 0;
    power_condition == 0 && load_eject && !start
}

/// Builds a READ(10) or WRITE(10) CDB.
fn rw10_cdb(op: u8, lba: u32, blocks: u16) -> [u8; 10] {
    let mut cdb = [0u8; 10];
    cdb[0] = op;
    cdb[2..6].copy_from_slice(&lba.to_be_bytes());
    cdb[7..9].copy_from_slice(&blocks.to_be_bytes());
    cdb
}

/// Builds a READ(16) or WRITE(16) CDB.
fn rw16_cdb(op: u8, lba: u64, blocks: u32) -> [u8; 16] {
    let mut cdb = [0u8; 16];
    cdb[0] = op;
    cdb[2..10].copy_from_slice(&lba.to_be_bytes());
    cdb[10..14].copy_from_slice(&blocks.to_be_bytes());
    cdb
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cb(bytes: &[u8]) -> [u8; 16] {
        let mut cb = [0u8; 16];
        cb[..bytes.len()].copy_from_slice(bytes);
        cb
    }

    #[test]
    fn rw10_cdb_encoding() {
        let expected = [0, 0, 0x12, 0x34, 0x56, 0x78, 0, 0x12, 0x34, 0];
        for op in [SCSI_READ_10, SCSI_WRITE_10] {
            let mut want = expected;
            want[0] = op;
            core::assert_eq!(rw10_cdb(op, 0x1234_5678, 0x1234), want);
        }
    }

    #[test]
    fn rw16_cdb_encoding() {
        #[rustfmt::skip]
        let expected = [
            0, 0,
            0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF,
            0xDE, 0xAD, 0xBE, 0xEF,
            0, 0,
        ];
        for op in [SCSI_READ_16, SCSI_WRITE_16] {
            let mut want = expected;
            want[0] = op;
            core::assert_eq!(rw16_cdb(op, 0x0123_4567_89AB_CDEF, 0xDEAD_BEEF), want);
        }
    }

    #[test]
    fn parses_what_the_host_builds() {
        for (op, write) in [(SCSI_READ_10, false), (SCSI_WRITE_10, true)] {
            let rw = ReadWrite::parse(&cb(&rw10_cdb(op, 0x1234_5678, 0xffff))).unwrap();
            core::assert_eq!(
                rw,
                ReadWrite {
                    write,
                    lba: 0x1234_5678,
                    blocks: 0xffff
                }
            );
        }
        for (op, write) in [(SCSI_READ_16, false), (SCSI_WRITE_16, true)] {
            let rw = ReadWrite::parse(&cb(&rw16_cdb(op, 0x0123_4567_89AB_CDEF, 0xDEAD_BEEF))).unwrap();
            core::assert_eq!(
                rw,
                ReadWrite {
                    write,
                    lba: 0x0123_4567_89AB_CDEF,
                    blocks: 0xDEAD_BEEF
                }
            );
        }
    }

    #[test]
    fn encode_picks_smallest_cdb() {
        let mut buf = [0u8; 16];
        let cases = [
            (0, 1, 10),
            (u32::MAX as u64, u16::MAX as u32, 10),
            (u32::MAX as u64 + 1, 1, 16),
            (0, u16::MAX as u32 + 1, 16),
        ];
        for (lba, blocks, len) in cases {
            for write in [false, true] {
                let rw = ReadWrite { write, lba, blocks };
                let cdb = rw.encode(&mut buf);
                core::assert_eq!(cdb.len(), len);
                core::assert_eq!(ReadWrite::parse(cdb), Some(rw));
            }
        }
    }

    #[test]
    fn parses_rw6() {
        // Bits above the 21-bit LBA in byte 1 are masked off.
        let rw = ReadWrite::parse(&cb(&[SCSI_WRITE_6, 0xff, 0x34, 0x56, 8, 0])).unwrap();
        core::assert_eq!(
            rw,
            ReadWrite {
                write: true,
                lba: 0x1f_3456,
                blocks: 8
            }
        );

        let rw = ReadWrite::parse(&cb(&[SCSI_READ_6, 0, 0, 1, 0, 0])).unwrap();
        core::assert_eq!(
            rw,
            ReadWrite {
                write: false,
                lba: 1,
                blocks: 256
            }
        );
    }

    #[test]
    fn parses_rw12() {
        let rw = ReadWrite::parse(&cb(&[SCSI_READ_12, 0, 0, 0, 0x10, 0, 0x00, 0x01, 0x00, 0x00])).unwrap();
        core::assert_eq!(
            rw,
            ReadWrite {
                write: false,
                lba: 0x1000,
                blocks: 0x1_0000
            }
        );
        core::assert!(ReadWrite::parse(&cb(&[SCSI_WRITE_12])).unwrap().write);
    }

    #[test]
    fn ignores_other_opcodes() {
        core::assert_eq!(ReadWrite::parse(&cb(&[SCSI_INQUIRY])), None);
        core::assert_eq!(ReadWrite::parse(&cb(&[SCSI_SYNCHRONIZE_CACHE_10])), None);
        core::assert_eq!(ReadWrite::parse(&[]), None);
    }

    #[test]
    fn parses_commands() {
        core::assert_eq!(
            Command::parse(&cb(&[SCSI_INQUIRY, 0x01, 0x80, 0x01, 0x02, 0]), 6),
            Command::Inquiry(Inquiry {
                evpd: true,
                cmd_dt: false,
                page_code: 0x80,
                allocation_length: 0x102,
            })
        );
        core::assert_eq!(
            Command::parse(&cb(&[SCSI_REPORT_LUNS, 0, 2, 0, 0, 0, 0, 0, 1, 0]), 12),
            Command::ReportLuns {
                select_report: 2,
                allocation_length: 0x100,
            }
        );
        core::assert_eq!(
            Command::parse(&cb(&rw10_cdb(SCSI_WRITE_10, 7, 2)), 10),
            Command::Write(ReadWrite {
                write: true,
                lba: 7,
                blocks: 2
            })
        );
        core::assert_eq!(Command::parse(&cb(&[0xff]), 6), Command::Unsupported { opcode: 0xff });
    }

    #[test]
    fn short_cdbs_are_unsupported() {
        let read10 = cb(&rw10_cdb(SCSI_READ_10, 0, 1));
        core::assert_eq!(
            Command::parse(&read10, 6),
            Command::Unsupported { opcode: SCSI_READ_10 }
        );

        let mut rc16 = [0u8; 16];
        rc16[0] = SCSI_SERVICE_ACTION_IN_16;
        rc16[1] = SCSI_SA_READ_CAPACITY_16;
        rc16[13] = 32;
        core::assert_eq!(
            Command::parse(&rc16, 16),
            Command::ReadCapacity16 { allocation_length: 32 }
        );
        core::assert_eq!(
            Command::parse(&rc16, 12),
            Command::Unsupported {
                opcode: SCSI_SERVICE_ACTION_IN_16
            }
        );
    }

    #[test]
    fn sense_data_round_trips() {
        let raw = SenseData::WRITE_PROTECTED.encode();
        core::assert_eq!(raw[0], 0x70);
        core::assert_eq!(raw[2], 0x07);
        core::assert_eq!(raw[7], 10);
        core::assert_eq!(raw[12], ASC_WRITE_PROTECTED);
        core::assert_eq!(SenseData::parse(&raw), SenseData::WRITE_PROTECTED);
    }

    #[test]
    fn capacity_round_trips() {
        let cap = Capacity {
            last_lba: 0x1234_5678,
            block_size: 512,
        };
        core::assert_eq!(cap.encode_10(), [0x12, 0x34, 0x56, 0x78, 0, 0, 0x02, 0]);
        core::assert_eq!(Capacity::parse_10(&cap.encode_10()), cap);

        let big = Capacity {
            last_lba: 0x1_0000_0000,
            block_size: 4096,
        };
        core::assert_eq!(Capacity::parse_16(&big.encode_16()), big);
        core::assert_eq!(
            Capacity::parse_10(&big.encode_10()).last_lba,
            Capacity::LBA_10_OVERFLOW as u64
        );
    }

    #[test]
    fn decodes_eject() {
        core::assert!(is_eject(0x02));
        core::assert!(!is_eject(0x03)); // load
        core::assert!(!is_eject(0x00)); // stop
        core::assert!(!is_eject(0x01)); // start
        core::assert!(!is_eject(0x32)); // power condition set, LoEj ignored
    }

    #[test]
    fn rejects_truncated_cdb() {
        let cdb = rw16_cdb(SCSI_READ_16, 1, 1);
        core::assert_eq!(ReadWrite::parse(&cdb[..10]), None);
        core::assert!(ReadWrite::parse(&cdb).is_some());
        core::assert_eq!(ReadWrite::parse(&[SCSI_READ_6, 0, 0, 0, 1]), None);
    }
}
