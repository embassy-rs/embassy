use crate::events::Event;

macro_rules! impl_bytes {
    ($t:ident) => {
        impl $t {
            pub const SIZE: usize = core::mem::size_of::<Self>();

            #[allow(unused)]
            pub fn to_bytes(&self) -> [u8; Self::SIZE] {
                unsafe { core::mem::transmute(*self) }
            }

            #[allow(unused)]
            pub fn from_bytes(bytes: &[u8; Self::SIZE]) -> Self {
                unsafe { core::mem::transmute(*bytes) }
            }
        }
    };
}

#[derive(Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
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
impl_bytes!(SdpcmHeader);

#[derive(Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(C)]
pub struct CdcHeader {
    pub cmd: u32,
    pub len: u32,
    pub flags: u16,
    pub id: u16,
    pub status: u32,
}
impl_bytes!(CdcHeader);

pub const BDC_VERSION: u8 = 2;
pub const BDC_VERSION_SHIFT: u8 = 4;

#[derive(Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(C)]
pub struct BcdHeader {
    pub flags: u8,
    /// 802.1d Priority (low 3 bits)
    pub priority: u8,
    pub flags2: u8,
    /// Offset from end of BDC header to packet data, in 4-uint8_t words. Leaves room for optional headers.
    pub data_offset: u8,
}
impl_bytes!(BcdHeader);

#[derive(Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(C)]
pub struct EthernetHeader {
    pub destination_mac: [u8; 6],
    pub source_mac: [u8; 6],
    pub ether_type: u16,
}

impl EthernetHeader {
    pub fn byteswap(&mut self) {
        self.ether_type = self.ether_type.to_be();
    }
}

#[derive(Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(C)]
pub struct EventHeader {
    pub subtype: u16,
    pub length: u16,
    pub version: u8,
    pub oui: [u8; 3],
    pub user_subtype: u16,
}

impl EventHeader {
    pub fn byteswap(&mut self) {
        self.subtype = self.subtype.to_be();
        self.length = self.length.to_be();
        self.user_subtype = self.user_subtype.to_be();
    }
}

#[derive(Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(C)]
pub struct EventMessage {
    /// version   
    pub version: u16,
    /// see flags below
    pub flags: u16,
    /// Message (see below)
    pub event_type: u32,
    /// Status code (see below)
    pub status: u32,
    /// Reason code (if applicable)
    pub reason: u32,
    /// WLC_E_AUTH
    pub auth_type: u32,
    /// data buf
    pub datalen: u32,
    /// Station address (if applicable)
    pub addr: [u8; 6],
    /// name of the incoming packet interface
    pub ifname: [u8; 16],
    /// destination OS i/f index
    pub ifidx: u8,
    /// source bsscfg index
    pub bsscfgidx: u8,
}
impl_bytes!(EventMessage);

impl EventMessage {
    pub fn byteswap(&mut self) {
        self.version = self.version.to_be();
        self.flags = self.flags.to_be();
        self.event_type = self.event_type.to_be();
        self.status = self.status.to_be();
        self.reason = self.reason.to_be();
        self.auth_type = self.auth_type.to_be();
        self.datalen = self.datalen.to_be();
    }
}

#[derive(Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(C)]
pub struct EventPacket {
    pub eth: EthernetHeader,
    pub hdr: EventHeader,
    pub msg: EventMessage,
}
impl_bytes!(EventPacket);

impl EventPacket {
    pub fn byteswap(&mut self) {
        self.eth.byteswap();
        self.hdr.byteswap();
        self.msg.byteswap();
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct DownloadHeader {
    pub flag: u16, //
    pub dload_type: u16,
    pub len: u32,
    pub crc: u32,
}
impl_bytes!(DownloadHeader);

#[allow(unused)]
pub const DOWNLOAD_FLAG_NO_CRC: u16 = 0x0001;
pub const DOWNLOAD_FLAG_BEGIN: u16 = 0x0002;
pub const DOWNLOAD_FLAG_END: u16 = 0x0004;
pub const DOWNLOAD_FLAG_HANDLER_VER: u16 = 0x1000;

// Country Locale Matrix (CLM)
pub const DOWNLOAD_TYPE_CLM: u16 = 2;

#[derive(Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(C)]
pub struct CountryInfo {
    pub country_abbrev: [u8; 4],
    pub rev: i32,
    pub country_code: [u8; 4],
}
impl_bytes!(CountryInfo);

#[derive(Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(C)]
pub struct SsidInfo {
    pub len: u32,
    pub ssid: [u8; 32],
}
impl_bytes!(SsidInfo);

#[derive(Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(C)]
pub struct PassphraseInfo {
    pub len: u16,
    pub flags: u16,
    pub passphrase: [u8; 64],
}
impl_bytes!(PassphraseInfo);

#[derive(Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(C)]
pub struct EventMask {
    pub iface: u32,
    pub events: [u8; 24],
}
impl_bytes!(EventMask);

impl EventMask {
    pub fn unset(&mut self, evt: Event) {
        let evt = evt as u8 as usize;
        self.events[evt / 8] &= !(1 << (evt % 8));
    }
}
