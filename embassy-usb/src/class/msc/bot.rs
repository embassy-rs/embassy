//! Bulk-Only Transport wrappers (MSC BBB r1.0 §5), shared by the device and host MSC classes.

pub(crate) const CBW_LEN: usize = 31;
pub(crate) const CSW_LEN: usize = 13;

pub(crate) const CBW_SIGNATURE: u32 = 0x4342_5355; // "USBC"
pub(crate) const CSW_SIGNATURE: u32 = 0x5342_5355; // "USBS"

/// `bmCBWFlags` direction bit: data flows device to host.
pub(crate) const CBW_FLAG_IN: u8 = 0x80;

/// `bLUN` bits holding the logical unit number.
const CBW_LUN_MASK: u8 = 0x0f;
/// `bCBWCBLength` bits holding the command block length.
const CBW_CB_LENGTH_MASK: u8 = 0x1f;

/// Command Block Wrapper.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub(crate) struct Cbw {
    pub tag: u32,
    pub data_transfer_length: u32,
    pub flags: u8,
    pub lun: u8,
    pub cb_length: u8,
    pub cb: [u8; 16],
}

impl Cbw {
    pub fn direction_in(&self) -> bool {
        self.flags & CBW_FLAG_IN != 0
    }

    pub fn parse(raw: &[u8]) -> Result<Self, ParseCbwError> {
        if raw.len() != CBW_LEN {
            return Err(ParseCbwError::InvalidLength);
        }
        if le32(&raw[0..4]) != CBW_SIGNATURE {
            return Err(ParseCbwError::InvalidSignature);
        }
        let cb_length = raw[14] & CBW_CB_LENGTH_MASK;
        if cb_length == 0 || cb_length > 16 {
            return Err(ParseCbwError::InvalidCbLength);
        }
        let mut cb = [0u8; 16];
        cb.copy_from_slice(&raw[15..31]);
        Ok(Self {
            tag: le32(&raw[4..8]),
            data_transfer_length: le32(&raw[8..12]),
            flags: raw[12],
            lun: raw[13] & CBW_LUN_MASK,
            cb_length,
            cb,
        })
    }

    pub fn encode(&self) -> [u8; CBW_LEN] {
        let mut out = [0u8; CBW_LEN];
        out[0..4].copy_from_slice(&CBW_SIGNATURE.to_le_bytes());
        out[4..8].copy_from_slice(&self.tag.to_le_bytes());
        out[8..12].copy_from_slice(&self.data_transfer_length.to_le_bytes());
        out[12] = self.flags;
        out[13] = self.lun & CBW_LUN_MASK;
        out[14] = self.cb_length & CBW_CB_LENGTH_MASK;
        out[15..31].copy_from_slice(&self.cb);
        out
    }
}

/// `bCSWStatus` (MSC BBB r1.0 §5.2, Table 5.3).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub(crate) enum CswStatus {
    Passed = 0x00,
    Failed = 0x01,
    PhaseError = 0x02,
}

/// Command Status Wrapper.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub(crate) struct Csw {
    pub tag: u32,
    pub residue: u32,
    pub status: CswStatus,
}

impl Csw {
    pub fn parse(raw: &[u8; CSW_LEN]) -> Result<Self, ParseCswError> {
        if le32(&raw[0..4]) != CSW_SIGNATURE {
            return Err(ParseCswError::InvalidSignature);
        }
        let status = match raw[12] {
            0x00 => CswStatus::Passed,
            0x01 => CswStatus::Failed,
            0x02 => CswStatus::PhaseError,
            _ => return Err(ParseCswError::InvalidStatus),
        };
        Ok(Self {
            tag: le32(&raw[4..8]),
            residue: le32(&raw[8..12]),
            status,
        })
    }

    pub fn encode(&self) -> [u8; CSW_LEN] {
        let mut out = [0u8; CSW_LEN];
        out[0..4].copy_from_slice(&CSW_SIGNATURE.to_le_bytes());
        out[4..8].copy_from_slice(&self.tag.to_le_bytes());
        out[8..12].copy_from_slice(&self.residue.to_le_bytes());
        out[12] = self.status as u8;
        out
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub(crate) enum ParseCbwError {
    InvalidLength,
    InvalidSignature,
    InvalidCbLength,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub(crate) enum ParseCswError {
    InvalidSignature,
    InvalidStatus,
}

fn le32(b: &[u8]) -> u32 {
    u32::from_le_bytes([b[0], b[1], b[2], b[3]])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::class::msc::scsi::SCSI_INQUIRY;

    #[test]
    fn parses_valid_cbw() {
        let mut raw = [0u8; 31];
        raw[0..4].copy_from_slice(&CBW_SIGNATURE.to_le_bytes());
        raw[4..8].copy_from_slice(&0x1122_3344u32.to_le_bytes());
        raw[8..12].copy_from_slice(&0x5566_7788u32.to_le_bytes());
        raw[12] = 0x80;
        raw[13] = 0x00;
        raw[14] = 10;
        raw[15] = SCSI_INQUIRY;

        let cbw = Cbw::parse(&raw).unwrap();
        core::assert_eq!(cbw.tag, 0x1122_3344);
        core::assert_eq!(cbw.data_transfer_length, 0x5566_7788);
        core::assert!(cbw.direction_in());
        core::assert_eq!(cbw.cb_length, 10);
        core::assert_eq!(cbw.cb[0], SCSI_INQUIRY);
    }

    #[test]
    fn rejects_bad_signature() {
        let mut raw = [0u8; 31];
        raw[14] = 6;
        core::assert_eq!(Cbw::parse(&raw), Err(ParseCbwError::InvalidSignature));
    }

    #[test]
    fn rejects_bad_cb_length() {
        let mut raw = [0u8; 31];
        raw[0..4].copy_from_slice(&CBW_SIGNATURE.to_le_bytes());
        raw[14] = 0;
        core::assert_eq!(Cbw::parse(&raw), Err(ParseCbwError::InvalidCbLength));
    }

    #[test]
    fn masks_lun_nibble() {
        let mut raw = [0u8; 31];
        raw[0..4].copy_from_slice(&CBW_SIGNATURE.to_le_bytes());
        raw[13] = 0xF2;
        raw[14] = 6;
        let cbw = Cbw::parse(&raw).unwrap();
        core::assert_eq!(cbw.lun, 2);
    }

    #[test]
    fn encodes_csw() {
        let csw = Csw {
            tag: 0xAABB_CCDD,
            residue: 0x0102_0304,
            status: CswStatus::Failed,
        };
        let raw = csw.encode();

        core::assert_eq!(&raw[0..4], &CSW_SIGNATURE.to_le_bytes());
        core::assert_eq!(&raw[4..8], &0xAABB_CCDDu32.to_le_bytes());
        core::assert_eq!(&raw[8..12], &0x0102_0304u32.to_le_bytes());
        core::assert_eq!(raw[12], CswStatus::Failed as u8);
        core::assert_eq!(Csw::parse(&raw), Ok(csw));
    }

    #[test]
    fn cbw_round_trips() {
        let mut cb = [0u8; 16];
        cb[0] = SCSI_INQUIRY;
        cb[4] = 36;
        let cbw = Cbw {
            tag: 7,
            data_transfer_length: 36,
            flags: CBW_FLAG_IN,
            lun: 3,
            cb_length: 6,
            cb,
        };
        core::assert_eq!(Cbw::parse(&cbw.encode()), Ok(cbw));
    }

    #[test]
    fn rejects_bad_csw() {
        let mut raw = Csw {
            tag: 1,
            residue: 0,
            status: CswStatus::Passed,
        }
        .encode();
        raw[12] = 0x03;
        core::assert_eq!(Csw::parse(&raw), Err(ParseCswError::InvalidStatus));
        raw[0] = 0;
        core::assert_eq!(Csw::parse(&raw), Err(ParseCswError::InvalidSignature));
    }
}
