/// Value indicating that the Type 2 Tag contains NFC Forum defined data.
const NFC_T2T_NFC_FORUM_DEFINED_DATA: u8 = 0xE1;

/// Value used for calculating the first BCC byte of a Type 2 Tag serial number.
const NFC_T2T_UID_BCC_CASCADE_BYTE: u8 = 0x88;

/// Supported major version of the Type 2 Tag specification.
const NFC_T2T_SUPPORTED_MAJOR_VERSION: u8 = 1;

/// Supported minor version of the Type 2 Tag specification.
const NFC_T2T_SUPPORTED_MINOR_VERSION: u8 = 0;

/// Type 2 Tag block size in bytes.
const NFC_T2T_BLOCK_SIZE: u8 = 4;

/// Offset of the Capability Container area in the Type 2 Tag.
const NFC_T2T_CC_BLOCK_OFFSET: u8 = 12;

/// Offset of the data area in the Type 2 Tag.
const NFC_T2T_FIRST_DATA_BLOCK_OFFSET: u8 = 16;

#[derive(Default)]
pub struct NfcType2Capabilities {
    major_ver: u8,
    minor_ver: u8,
    data_area_size: u8,
    read_access: u8,
    write_access: u8,
}

#[derive(Default)]
pub struct NfcType2TagSerial {
    manufacturer_id: u8,
    serial_nr_l: u16,
    serial_nr_h: u32,
    check_byte_0: u8,
    check_byte_1: u8,
    internal: u8,
}

#[derive(Default)]
pub struct NfcType2Tag<const MAX_BLOCKS: usize> {
    sn: NfcType2TagSerial,
    lock_bytes: u16,
    capabilities: NfcType2Capabilities,
    max_tlv: u16,
    tlv_block_array: [u8; MAX_BLOCKS],
    count_tlv: u16,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    VersionNotSupported,
}

pub fn parsee_type2<const N: usize>() -> Result<NfcType2Tag<N>> {}
