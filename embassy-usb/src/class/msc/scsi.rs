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
pub(crate) const SCSI_READ_12: u8 = 0xa8;
pub(crate) const SCSI_WRITE_12: u8 = 0xaa;
pub(crate) const SCSI_SA_READ_CAPACITY_16: u8 = 0x10;

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

        let be32 = |b: &[u8]| u32::from_be_bytes([b[0], b[1], b[2], b[3]]);
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
}

/// Builds a READ(10) or WRITE(10) CDB.
pub(crate) fn rw10_cdb(op: u8, lba: u32, blocks: u16) -> [u8; 10] {
    let mut cdb = [0u8; 10];
    cdb[0] = op;
    cdb[2..6].copy_from_slice(&lba.to_be_bytes());
    cdb[7..9].copy_from_slice(&blocks.to_be_bytes());
    cdb
}

/// Builds a READ(16) or WRITE(16) CDB.
pub(crate) fn rw16_cdb(op: u8, lba: u64, blocks: u32) -> [u8; 16] {
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
    fn rejects_truncated_cdb() {
        let cdb = rw16_cdb(SCSI_READ_16, 1, 1);
        core::assert_eq!(ReadWrite::parse(&cdb[..10]), None);
        core::assert!(ReadWrite::parse(&cdb).is_some());
        core::assert_eq!(ReadWrite::parse(&[SCSI_READ_6, 0, 0, 0, 1]), None);
    }
}
