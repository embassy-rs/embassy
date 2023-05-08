use crate::events::Event;
use crate::fmt::Bytes;

macro_rules! impl_bytes {
    ($t:ident) => {
        impl $t {
            pub const SIZE: usize = core::mem::size_of::<Self>();

            #[allow(unused)]
            pub fn to_bytes(&self) -> [u8; Self::SIZE] {
                unsafe { core::mem::transmute(*self) }
            }

            #[allow(unused)]
            pub fn from_bytes(bytes: &[u8; Self::SIZE]) -> &Self {
                let alignment = core::mem::align_of::<Self>();
                assert_eq!(
                    bytes.as_ptr().align_offset(alignment),
                    0,
                    "{} is not aligned",
                    core::any::type_name::<Self>()
                );
                unsafe { core::mem::transmute(bytes) }
            }

            #[allow(unused)]
            pub fn from_bytes_mut(bytes: &mut [u8; Self::SIZE]) -> &mut Self {
                let alignment = core::mem::align_of::<Self>();
                assert_eq!(
                    bytes.as_ptr().align_offset(alignment),
                    0,
                    "{} is not aligned",
                    core::any::type_name::<Self>()
                );

                unsafe { core::mem::transmute(bytes) }
            }
        }
    };
}

#[derive(Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(C)]
pub struct SharedMemData {
    pub flags: u32,
    pub trap_addr: u32,
    pub assert_exp_addr: u32,
    pub assert_file_addr: u32,
    pub assert_line: u32,
    pub console_addr: u32,
    pub msgtrace_addr: u32,
    pub fwid: u32,
}
impl_bytes!(SharedMemData);

#[derive(Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(C)]
pub struct SharedMemLog {
    pub buf: u32,
    pub buf_size: u32,
    pub idx: u32,
    pub out_idx: u32,
}
impl_bytes!(SharedMemLog);

#[derive(Debug, Clone, Copy)]
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

impl SdpcmHeader {
    pub fn parse(packet: &mut [u8]) -> Option<(&mut Self, &mut [u8])> {
        let packet_len = packet.len();
        if packet_len < Self::SIZE {
            warn!("packet too short, len={}", packet.len());
            return None;
        }
        let (sdpcm_header, sdpcm_packet) = packet.split_at_mut(Self::SIZE);
        let sdpcm_header = Self::from_bytes_mut(sdpcm_header.try_into().unwrap());
        trace!("rx {:?}", sdpcm_header);

        if sdpcm_header.len != !sdpcm_header.len_inv {
            warn!("len inv mismatch");
            return None;
        }

        if sdpcm_header.len as usize != packet_len {
            warn!("len from header doesn't match len from spi");
            return None;
        }

        let sdpcm_packet = &mut sdpcm_packet[(sdpcm_header.header_length as usize - Self::SIZE)..];
        Some((sdpcm_header, sdpcm_packet))
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed(2))]
pub struct CdcHeader {
    pub cmd: u32,
    pub len: u32,
    pub flags: u16,
    pub id: u16,
    pub status: u32,
}
impl_bytes!(CdcHeader);

#[cfg(feature = "defmt")]
impl defmt::Format for CdcHeader {
    fn format(&self, fmt: defmt::Formatter) {
        fn copy<T: Copy>(t: T) -> T {
            t
        }

        defmt::write!(
            fmt,
            "CdcHeader{{cmd: {=u32:08x}, len: {=u32:08x}, flags: {=u16:04x}, id: {=u16:04x}, status: {=u32:08x}}}",
            copy(self.cmd),
            copy(self.len),
            copy(self.flags),
            copy(self.id),
            copy(self.status),
        )
    }
}

impl CdcHeader {
    pub fn parse(packet: &mut [u8]) -> Option<(&mut Self, &mut [u8])> {
        if packet.len() < Self::SIZE {
            warn!("payload too short, len={}", packet.len());
            return None;
        }

        let (cdc_header, payload) = packet.split_at_mut(Self::SIZE);
        let cdc_header = Self::from_bytes_mut(cdc_header.try_into().unwrap());

        let payload = &mut payload[..cdc_header.len as usize];
        Some((cdc_header, payload))
    }
}

pub const BDC_VERSION: u8 = 2;
pub const BDC_VERSION_SHIFT: u8 = 4;

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(C)]
pub struct BdcHeader {
    pub flags: u8,
    /// 802.1d Priority (low 3 bits)
    pub priority: u8,
    pub flags2: u8,
    /// Offset from end of BDC header to packet data, in 4-uint8_t words. Leaves room for optional headers.
    pub data_offset: u8,
}
impl_bytes!(BdcHeader);

impl BdcHeader {
    pub fn parse(packet: &mut [u8]) -> Option<(&mut Self, &mut [u8])> {
        if packet.len() < Self::SIZE {
            return None;
        }

        let (bdc_header, bdc_packet) = packet.split_at_mut(Self::SIZE);
        let bdc_header = Self::from_bytes_mut(bdc_header.try_into().unwrap());
        trace!("    {:?}", bdc_header);

        let packet_start = 4 * bdc_header.data_offset as usize;

        let bdc_packet = bdc_packet.get_mut(packet_start..)?;
        trace!("    {:02x}", Bytes(&bdc_packet[..bdc_packet.len().min(36)]));

        Some((bdc_header, bdc_packet))
    }
}

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

#[derive(Debug, Clone, Copy)]
// #[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(C, packed(2))]
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

#[cfg(feature = "defmt")]
impl defmt::Format for EventMessage {
    fn format(&self, fmt: defmt::Formatter) {
        let event_type = self.event_type;
        let status = self.status;
        let reason = self.reason;
        let auth_type = self.auth_type;
        let datalen = self.datalen;

        defmt::write!(
            fmt,
            "EventMessage {{ \
            version: {=u16}, \
            flags: {=u16}, \
            event_type: {=u32}, \
            status: {=u32}, \
            reason: {=u32}, \
            auth_type: {=u32}, \
            datalen: {=u32}, \
            addr: {=[u8; 6]:x}, \
            ifname: {=[u8; 16]:x}, \
            ifidx: {=u8}, \
            bsscfgidx: {=u8}, \
        }} ",
            self.version,
            self.flags,
            event_type,
            status,
            reason,
            auth_type,
            datalen,
            self.addr,
            self.ifname,
            self.ifidx,
            self.bsscfgidx
        );
    }
}

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
#[repr(C, packed(2))]
pub struct EventPacket {
    pub eth: EthernetHeader,
    pub hdr: EventHeader,
    pub msg: EventMessage,
}
impl_bytes!(EventPacket);

impl EventPacket {
    pub fn parse(packet: &mut [u8]) -> Option<(&mut Self, &mut [u8])> {
        if packet.len() < Self::SIZE {
            return None;
        }

        let (event_header, event_packet) = packet.split_at_mut(Self::SIZE);
        let event_header = Self::from_bytes_mut(event_header.try_into().unwrap());
        // warn!("event_header {:x}", event_header as *const _);
        event_header.byteswap();

        let event_packet = event_packet.get_mut(..event_header.msg.datalen as usize)?;

        Some((event_header, event_packet))
    }

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
pub struct SsidInfoWithIndex {
    pub index: u32,
    pub ssid_info: SsidInfo,
}
impl_bytes!(SsidInfoWithIndex);

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

/// Parameters for a wifi scan
#[derive(Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(C)]
pub struct ScanParams {
    pub version: u32,
    pub action: u16,
    pub sync_id: u16,
    pub ssid_len: u32,
    pub ssid: [u8; 32],
    pub bssid: [u8; 6],
    pub bss_type: u8,
    pub scan_type: u8,
    pub nprobes: u32,
    pub active_time: u32,
    pub passive_time: u32,
    pub home_time: u32,
    pub channel_num: u32,
    pub channel_list: [u16; 1],
}
impl_bytes!(ScanParams);

/// Wifi Scan Results Header, followed by `bss_count` `BssInfo`
#[derive(Clone, Copy)]
// #[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(C, packed(2))]
pub struct ScanResults {
    pub buflen: u32,
    pub version: u32,
    pub sync_id: u16,
    pub bss_count: u16,
}
impl_bytes!(ScanResults);

impl ScanResults {
    pub fn parse(packet: &mut [u8]) -> Option<(&mut ScanResults, &mut [u8])> {
        if packet.len() < ScanResults::SIZE {
            return None;
        }

        let (scan_results, bssinfo) = packet.split_at_mut(ScanResults::SIZE);
        let scan_results = ScanResults::from_bytes_mut(scan_results.try_into().unwrap());

        if scan_results.bss_count > 0 && bssinfo.len() < BssInfo::SIZE {
            warn!("Scan result, incomplete BssInfo");
            return None;
        }

        Some((scan_results, bssinfo))
    }
}

/// Wifi Scan Result
#[derive(Clone, Copy)]
// #[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(C, packed(2))]
#[non_exhaustive]
pub struct BssInfo {
    pub version: u32,
    pub length: u32,
    pub bssid: [u8; 6],
    pub beacon_period: u16,
    pub capability: u16,
    pub ssid_len: u8,
    pub ssid: [u8; 32],
    // there will be more stuff here
}
impl_bytes!(BssInfo);

impl BssInfo {
    pub fn parse(packet: &mut [u8]) -> Option<&mut Self> {
        if packet.len() < BssInfo::SIZE {
            return None;
        }

        Some(BssInfo::from_bytes_mut(
            packet[..BssInfo::SIZE].as_mut().try_into().unwrap(),
        ))
    }
}
