#[derive(Clone, Copy)]
#[repr(C)]
pub struct SdpcmHeader {
    pub len: u16,
    pub len_inv: u16,
    /// Rx/Tx sequence number
    pub sequence: u8,
    ///  4 MSB Channel number, 4 LSB arbitrary flag
    pub channel_and_flags: u8,
    /// Length of next data frame, reserved for Tx
    pub next_length: u8,
    /// Data offset
    pub header_length: u8,
    /// Flow control bits, reserved for Tx
    pub wireless_flow_control: u8,
    /// Maximum Sequence number allowed by firmware for Tx
    pub bus_data_credit: u8,
    /// Reserved
    pub reserved: [u8; 2],
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct CdcHeader {
    pub cmd: u32,
    pub out_len: u16,
    pub in_len: u16,
    pub flags: u16,
    pub id: u16,
    pub status: u32,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct BdcHeader {
    pub flags: u8,
    /// 802.1d Priority (low 3 bits)
    pub priority: u8,
    pub flags2: u8,
    /// Offset from end of BDC header to packet data, in 4-uint8_t words. Leaves room for optional headers.
    pub data_offset: u8,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct DownloadHeader {
    pub flag: u16,
    pub dload_type: u16,
    pub len: u32,
    pub crc: u32,
}

macro_rules! impl_bytes {
    ($t:ident) => {
        impl $t {
            pub const SIZE: usize = core::mem::size_of::<Self>();

            pub fn to_bytes(&self) -> [u8; Self::SIZE] {
                unsafe { core::mem::transmute(*self) }
            }

            pub fn from_bytes(bytes: &[u8; Self::SIZE]) -> Self {
                unsafe { core::mem::transmute(*bytes) }
            }
        }
    };
}
impl_bytes!(SdpcmHeader);
impl_bytes!(CdcHeader);
impl_bytes!(BdcHeader);
impl_bytes!(DownloadHeader);
