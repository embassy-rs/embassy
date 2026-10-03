//! Bulk-Only Transport wrappers (MSC BBB r1.0 §5), shared by the device and host MSC classes.

pub(crate) const CBW_LEN: usize = 31;
pub(crate) const CSW_LEN: usize = 13;

pub(crate) const CBW_SIGNATURE: u32 = 0x4342_5355; // "USBC"
pub(crate) const CSW_SIGNATURE: u32 = 0x5342_5355; // "USBS"

/// `bmCBWFlags` direction bit: data flows device to host.
pub(crate) const CBW_FLAG_IN: u8 = 0x80;

pub(crate) const CSW_STATUS_PASSED: u8 = 0x00;
pub(crate) const CSW_STATUS_FAILED: u8 = 0x01;
pub(crate) const CSW_STATUS_PHASE_ERROR: u8 = 0x02;

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
}

/// Command Status Wrapper.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub(crate) struct Csw {
    pub tag: u32,
    pub residue: u32,
    pub status: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub(crate) enum ParseCbwError {
    InvalidLength,
    InvalidSignature,
    InvalidCbLength,
}

pub(crate) fn parse_cbw(raw: &[u8]) -> Result<Cbw, ParseCbwError> {
    if raw.len() != CBW_LEN {
        return Err(ParseCbwError::InvalidLength);
    }
    let signature = u32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]);
    if signature != CBW_SIGNATURE {
        return Err(ParseCbwError::InvalidSignature);
    }
    let cb_length = raw[14] & 0x1f;
    if cb_length == 0 || cb_length > 16 {
        return Err(ParseCbwError::InvalidCbLength);
    }
    let mut cb = [0u8; 16];
    cb.copy_from_slice(&raw[15..31]);
    Ok(Cbw {
        tag: u32::from_le_bytes([raw[4], raw[5], raw[6], raw[7]]),
        data_transfer_length: u32::from_le_bytes([raw[8], raw[9], raw[10], raw[11]]),
        flags: raw[12],
        lun: raw[13] & 0x0f,
        cb_length,
        cb,
    })
}

pub(crate) fn encode_csw(csw: Csw) -> [u8; CSW_LEN] {
    let mut out = [0u8; CSW_LEN];
    out[0..4].copy_from_slice(&CSW_SIGNATURE.to_le_bytes());
    out[4..8].copy_from_slice(&csw.tag.to_le_bytes());
    out[8..12].copy_from_slice(&csw.residue.to_le_bytes());
    out[12] = csw.status;
    out
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

        let cbw = parse_cbw(&raw).unwrap();
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
        core::assert_eq!(parse_cbw(&raw), Err(ParseCbwError::InvalidSignature));
    }

    #[test]
    fn rejects_bad_cb_length() {
        let mut raw = [0u8; 31];
        raw[0..4].copy_from_slice(&CBW_SIGNATURE.to_le_bytes());
        raw[14] = 0;
        core::assert_eq!(parse_cbw(&raw), Err(ParseCbwError::InvalidCbLength));
    }

    #[test]
    fn masks_lun_nibble() {
        let mut raw = [0u8; 31];
        raw[0..4].copy_from_slice(&CBW_SIGNATURE.to_le_bytes());
        raw[13] = 0xF2;
        raw[14] = 6;
        let cbw = parse_cbw(&raw).unwrap();
        core::assert_eq!(cbw.lun, 2);
    }

    #[test]
    fn encodes_csw() {
        let raw = encode_csw(Csw {
            tag: 0xAABB_CCDD,
            residue: 0x0102_0304,
            status: CSW_STATUS_FAILED,
        });

        core::assert_eq!(&raw[0..4], &CSW_SIGNATURE.to_le_bytes());
        core::assert_eq!(&raw[4..8], &0xAABB_CCDDu32.to_le_bytes());
        core::assert_eq!(&raw[8..12], &0x0102_0304u32.to_le_bytes());
        core::assert_eq!(raw[12], CSW_STATUS_FAILED);
    }
}
