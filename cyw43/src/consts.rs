#![allow(unused)]

pub(crate) const FUNC_BUS: u32 = 0;
pub(crate) const FUNC_BACKPLANE: u32 = 1;
pub(crate) const FUNC_WLAN: u32 = 2;
pub(crate) const FUNC_BT: u32 = 3;

// Register addresses
pub(crate) const REG_BUS_CTRL: u32 = 0x0;
pub(crate) const REG_BUS_RESPONSE_DELAY: u32 = 0x1;
pub(crate) const REG_BUS_STATUS_ENABLE: u32 = 0x2;
pub(crate) const REG_BUS_INTERRUPT: u32 = 0x04; // 16 bits - Interrupt status
pub(crate) const REG_BUS_INTERRUPT_ENABLE: u32 = 0x06; // 16 bits - Interrupt mask
pub(crate) const REG_BUS_STATUS: u32 = 0x8;
pub(crate) const REG_BUS_TEST_RO: u32 = 0x14;
pub(crate) const REG_BUS_TEST_RW: u32 = 0x18;
pub(crate) const REG_BUS_RESP_DELAY: u32 = 0x1c;

// SPI_BUS_CONTROL Bits
pub(crate) const WORD_LENGTH_32: u32 = 0x1;
pub(crate) const ENDIAN_BIG: u32 = 0x2;
pub(crate) const CLOCK_PHASE: u32 = 0x4;
pub(crate) const CLOCK_POLARITY: u32 = 0x8;
pub(crate) const HIGH_SPEED: u32 = 0x10;
pub(crate) const INTERRUPT_POLARITY_HIGH: u32 = 0x20;
pub(crate) const WAKE_UP: u32 = 0x80;

// SPI_STATUS_ENABLE bits
pub(crate) const STATUS_ENABLE: u32 = 0x01;
pub(crate) const INTR_WITH_STATUS: u32 = 0x02;
pub(crate) const RESP_DELAY_ALL: u32 = 0x04;
pub(crate) const DWORD_PKT_LEN_EN: u32 = 0x08;
pub(crate) const CMD_ERR_CHK_EN: u32 = 0x20;
pub(crate) const DATA_ERR_CHK_EN: u32 = 0x40;

// SPI_STATUS_REGISTER bits
pub(crate) const STATUS_DATA_NOT_AVAILABLE: u32 = 0x00000001;
pub(crate) const STATUS_UNDERFLOW: u32 = 0x00000002;
pub(crate) const STATUS_OVERFLOW: u32 = 0x00000004;
pub(crate) const STATUS_F2_INTR: u32 = 0x00000008;
pub(crate) const STATUS_F3_INTR: u32 = 0x00000010;
pub(crate) const STATUS_F2_RX_READY: u32 = 0x00000020;
pub(crate) const STATUS_F3_RX_READY: u32 = 0x00000040;
pub(crate) const STATUS_HOST_CMD_DATA_ERR: u32 = 0x00000080;
pub(crate) const STATUS_F2_PKT_AVAILABLE: u32 = 0x00000100;
pub(crate) const STATUS_F2_PKT_LEN_MASK: u32 = 0x000FFE00;
pub(crate) const STATUS_F2_PKT_LEN_SHIFT: u32 = 9;
pub(crate) const STATUS_F3_PKT_AVAILABLE: u32 = 0x00100000;
pub(crate) const STATUS_F3_PKT_LEN_MASK: u32 = 0xFFE00000;
pub(crate) const STATUS_F3_PKT_LEN_SHIFT: u32 = 21;

pub(crate) const REG_BACKPLANE_GPIO_SELECT: u32 = 0x10005;
pub(crate) const REG_BACKPLANE_GPIO_OUTPUT: u32 = 0x10006;
pub(crate) const REG_BACKPLANE_GPIO_ENABLE: u32 = 0x10007;
pub(crate) const REG_BACKPLANE_FUNCTION2_WATERMARK: u32 = 0x10008;
pub(crate) const REG_BACKPLANE_DEVICE_CONTROL: u32 = 0x10009;
pub(crate) const REG_BACKPLANE_BACKPLANE_ADDRESS_LOW: u32 = 0x1000A;
pub(crate) const REG_BACKPLANE_BACKPLANE_ADDRESS_MID: u32 = 0x1000B;
pub(crate) const REG_BACKPLANE_BACKPLANE_ADDRESS_HIGH: u32 = 0x1000C;
pub(crate) const REG_BACKPLANE_FRAME_CONTROL: u32 = 0x1000D;
pub(crate) const REG_BACKPLANE_CHIP_CLOCK_CSR: u32 = 0x1000E;
pub(crate) const REG_BACKPLANE_PULL_UP: u32 = 0x1000F;
pub(crate) const REG_BACKPLANE_READ_FRAME_BC_LOW: u32 = 0x1001B;
pub(crate) const REG_BACKPLANE_READ_FRAME_BC_HIGH: u32 = 0x1001C;
pub(crate) const REG_BACKPLANE_WAKEUP_CTRL: u32 = 0x1001E;
pub(crate) const REG_BACKPLANE_SLEEP_CSR: u32 = 0x1001F;

pub(crate) const I_HMB_SW_MASK: u32 = 0x000000f0;
pub(crate) const I_HMB_FC_CHANGE: u32 = 1 << 5;
pub(crate) const SDIO_INT_STATUS: u32 = 0x20;
pub(crate) const SDIO_INT_HOST_MASK: u32 = 0x24;

pub(crate) const SPI_F2_WATERMARK: u8 = 0x20;

pub(crate) const BACKPLANE_WINDOW_SIZE: usize = 0x8000;
pub(crate) const BACKPLANE_ADDRESS_MASK: u32 = 0x7FFF;
pub(crate) const BACKPLANE_ADDRESS_32BIT_FLAG: u32 = 0x08000;
pub(crate) const BACKPLANE_MAX_TRANSFER_SIZE: usize = 64;
// Active Low Power (ALP) clock constants
pub(crate) const BACKPLANE_ALP_AVAIL_REQ: u8 = 0x08;
pub(crate) const BACKPLANE_ALP_AVAIL: u8 = 0x40;

// Broadcom AMBA (Advanced Microcontroller Bus Architecture) Interconnect
// (AI) pub (crate) constants
pub(crate) const AI_IOCTRL_OFFSET: u32 = 0x408;
pub(crate) const AI_IOCTRL_BIT_FGC: u8 = 0x0002;
pub(crate) const AI_IOCTRL_BIT_CLOCK_EN: u8 = 0x0001;
pub(crate) const AI_IOCTRL_BIT_CPUHALT: u8 = 0x0020;

pub(crate) const AI_RESETCTRL_OFFSET: u32 = 0x800;
pub(crate) const AI_RESETCTRL_BIT_RESET: u8 = 1;

pub(crate) const AI_RESETSTATUS_OFFSET: u32 = 0x804;

pub(crate) const TEST_PATTERN: u32 = 0x12345678;
pub(crate) const FEEDBEAD: u32 = 0xFEEDBEAD;

// SPI_INTERRUPT_REGISTER and SPI_INTERRUPT_ENABLE_REGISTER Bits
pub(crate) const IRQ_DATA_UNAVAILABLE: u16 = 0x0001; // Requested data not available; Clear by writing a "1"
pub(crate) const IRQ_F2_F3_FIFO_RD_UNDERFLOW: u16 = 0x0002;
pub(crate) const IRQ_F2_F3_FIFO_WR_OVERFLOW: u16 = 0x0004;
pub(crate) const IRQ_COMMAND_ERROR: u16 = 0x0008; // Cleared by writing 1
pub(crate) const IRQ_DATA_ERROR: u16 = 0x0010; // Cleared by writing 1
pub(crate) const IRQ_F2_PACKET_AVAILABLE: u16 = 0x0020;
pub(crate) const IRQ_F3_PACKET_AVAILABLE: u16 = 0x0040;
pub(crate) const IRQ_F1_OVERFLOW: u16 = 0x0080; // Due to last write. Bkplane has pending write requests
pub(crate) const IRQ_MISC_INTR0: u16 = 0x0100;
pub(crate) const IRQ_MISC_INTR1: u16 = 0x0200;
pub(crate) const IRQ_MISC_INTR2: u16 = 0x0400;
pub(crate) const IRQ_MISC_INTR3: u16 = 0x0800;
pub(crate) const IRQ_MISC_INTR4: u16 = 0x1000;
pub(crate) const IRQ_F1_INTR: u16 = 0x2000;
pub(crate) const IRQ_F2_INTR: u16 = 0x4000;
pub(crate) const IRQ_F3_INTR: u16 = 0x8000;

pub(crate) const CHANNEL_TYPE_CONTROL: u8 = 0;
pub(crate) const CHANNEL_TYPE_EVENT: u8 = 1;
pub(crate) const CHANNEL_TYPE_DATA: u8 = 2;

// CYW_SPID command structure constants.
pub(crate) const WRITE: bool = true;
pub(crate) const READ: bool = false;
pub(crate) const INC_ADDR: bool = true;
pub(crate) const FIXED_ADDR: bool = false;

pub(crate) const AES_ENABLED: u32 = 0x0004;
pub(crate) const WPA2_SECURITY: u32 = 0x00400000;

pub(crate) const MIN_PSK_LEN: usize = 8;
pub(crate) const MAX_PSK_LEN: usize = 64;

// Bluetooth firmware extraction constants.
pub(crate) const BTFW_ADDR_MODE_UNKNOWN: i32 = 0;
pub(crate) const BTFW_ADDR_MODE_EXTENDED: i32 = 1;
pub(crate) const BTFW_ADDR_MODE_SEGMENT: i32 = 2;
pub(crate) const BTFW_ADDR_MODE_LINEAR32: i32 = 3;

pub(crate) const BTFW_HEX_LINE_TYPE_DATA: u8 = 0;
pub(crate) const BTFW_HEX_LINE_TYPE_END_OF_DATA: u8 = 1;
pub(crate) const BTFW_HEX_LINE_TYPE_EXTENDED_SEGMENT_ADDRESS: u8 = 2;
pub(crate) const BTFW_HEX_LINE_TYPE_EXTENDED_ADDRESS: u8 = 4;
pub(crate) const BTFW_HEX_LINE_TYPE_ABSOLUTE_32BIT_ADDRESS: u8 = 5;

// Bluetooth constants.
pub(crate) const SPI_RESP_DELAY_F1: u32 = 0x001d;
pub(crate) const WHD_BUS_SPI_BACKPLANE_READ_PADD_SIZE: u8 = 4;

pub(crate) const BT2WLAN_PWRUP_WAKE: u32 = 3;
pub(crate) const BT2WLAN_PWRUP_ADDR: u32 = 0x640894;

pub(crate) const BT_CTRL_REG_ADDR: u32 = 0x18000c7c;
pub(crate) const HOST_CTRL_REG_ADDR: u32 = 0x18000d6c;
pub(crate) const WLAN_RAM_BASE_REG_ADDR: u32 = 0x18000d68;

pub(crate) const BTSDIO_REG_DATA_VALID_BITMASK: u32 = 1 << 1;
pub(crate) const BTSDIO_REG_BT_AWAKE_BITMASK: u32 = 1 << 8;
pub(crate) const BTSDIO_REG_WAKE_BT_BITMASK: u32 = 1 << 17;
pub(crate) const BTSDIO_REG_SW_RDY_BITMASK: u32 = 1 << 24;
pub(crate) const BTSDIO_REG_FW_RDY_BITMASK: u32 = 1 << 24;

pub(crate) const BTSDIO_FWBUF_SIZE: u32 = 0x1000;
pub(crate) const BTSDIO_OFFSET_HOST_WRITE_BUF: u32 = 0;
pub(crate) const BTSDIO_OFFSET_HOST_READ_BUF: u32 = BTSDIO_FWBUF_SIZE;

pub(crate) const BTSDIO_OFFSET_HOST2BT_IN: u32 = 0x00002000;
pub(crate) const BTSDIO_OFFSET_HOST2BT_OUT: u32 = 0x00002004;
pub(crate) const BTSDIO_OFFSET_BT2HOST_IN: u32 = 0x00002008;
pub(crate) const BTSDIO_OFFSET_BT2HOST_OUT: u32 = 0x0000200C;

// Security type (authentication and encryption types are combined using bit mask)
#[allow(non_camel_case_types)]
#[derive(Copy, Clone, PartialEq)]
#[repr(u32)]
pub(crate) enum Security {
    OPEN = 0,
    WPA2_AES_PSK = WPA2_SECURITY | AES_ENABLED,
}

#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
#[repr(u8)]
pub enum EStatus {
    /// operation was successful
    SUCCESS = 0,
    /// operation failed
    FAIL = 1,
    /// operation timed out
    TIMEOUT = 2,
    /// failed due to no matching network found
    NO_NETWORKS = 3,
    /// operation was aborted
    ABORT = 4,
    /// protocol failure: packet not ack'd
    NO_ACK = 5,
    /// AUTH or ASSOC packet was unsolicited
    UNSOLICITED = 6,
    /// attempt to assoc to an auto auth configuration
    ATTEMPT = 7,
    /// scan results are incomplete
    PARTIAL = 8,
    /// scan aborted by another scan
    NEWSCAN = 9,
    /// scan aborted due to assoc in progress
    NEWASSOC = 10,
    /// 802.11h quiet period started
    _11HQUIET = 11,
    /// user disabled scanning (WLC_SET_SCANSUPPRESS)
    SUPPRESS = 12,
    /// no allowable channels to scan
    NOCHANS = 13,
    /// scan aborted due to CCX fast roam
    CCXFASTRM = 14,
    /// abort channel select
    CS_ABORT = 15,
}

impl PartialEq<EStatus> for u32 {
    fn eq(&self, other: &EStatus) -> bool {
        *self == *other as Self
    }
}

#[allow(dead_code)]
pub(crate) struct FormatStatus(pub u32);

#[cfg(feature = "defmt")]
impl defmt::Format for FormatStatus {
    fn format(&self, fmt: defmt::Formatter) {
        macro_rules! implm {
            ($($name:ident),*) => {
                $(
                    if self.0 & $name > 0 {
                        defmt::write!(fmt, " | {}", &stringify!($name)[7..]);
                    }
                )*
            };
        }

        implm!(
            STATUS_DATA_NOT_AVAILABLE,
            STATUS_UNDERFLOW,
            STATUS_OVERFLOW,
            STATUS_F2_INTR,
            STATUS_F3_INTR,
            STATUS_F2_RX_READY,
            STATUS_F3_RX_READY,
            STATUS_HOST_CMD_DATA_ERR,
            STATUS_F2_PKT_AVAILABLE,
            STATUS_F3_PKT_AVAILABLE
        );
    }
}

#[cfg(feature = "log")]
impl core::fmt::Debug for FormatStatus {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
        macro_rules! implm {
            ($($name:ident),*) => {
                $(
                    if self.0 & $name > 0 {
                        core::write!(fmt, " | {}", &stringify!($name)[7..])?;
                    }
                )*
            };
        }

        implm!(
            STATUS_DATA_NOT_AVAILABLE,
            STATUS_UNDERFLOW,
            STATUS_OVERFLOW,
            STATUS_F2_INTR,
            STATUS_F3_INTR,
            STATUS_F2_RX_READY,
            STATUS_F3_RX_READY,
            STATUS_HOST_CMD_DATA_ERR,
            STATUS_F2_PKT_AVAILABLE,
            STATUS_F3_PKT_AVAILABLE
        );
        Ok(())
    }
}

#[cfg(feature = "log")]
impl core::fmt::Display for FormatStatus {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(self, f)
    }
}

#[allow(dead_code)]
pub(crate) struct FormatInterrupt(pub u16);

#[cfg(feature = "defmt")]
impl defmt::Format for FormatInterrupt {
    fn format(&self, fmt: defmt::Formatter) {
        macro_rules! implm {
            ($($name:ident),*) => {
                $(
                    if self.0 & $name > 0 {
                        defmt::write!(fmt, " | {}", &stringify!($name)[4..]);
                    }
                )*
            };
        }

        implm!(
            IRQ_DATA_UNAVAILABLE,
            IRQ_F2_F3_FIFO_RD_UNDERFLOW,
            IRQ_F2_F3_FIFO_WR_OVERFLOW,
            IRQ_COMMAND_ERROR,
            IRQ_DATA_ERROR,
            IRQ_F2_PACKET_AVAILABLE,
            IRQ_F3_PACKET_AVAILABLE,
            IRQ_F1_OVERFLOW,
            IRQ_MISC_INTR0,
            IRQ_MISC_INTR1,
            IRQ_MISC_INTR2,
            IRQ_MISC_INTR3,
            IRQ_MISC_INTR4,
            IRQ_F1_INTR,
            IRQ_F2_INTR,
            IRQ_F3_INTR
        );
    }
}

#[cfg(feature = "log")]
impl core::fmt::Debug for FormatInterrupt {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
        macro_rules! implm {
            ($($name:ident),*) => {
                $(
                    if self.0 & $name > 0 {
                        core::write!(fmt, " | {}", &stringify!($name)[7..])?;
                    }
                )*
            };
        }

        implm!(
            IRQ_DATA_UNAVAILABLE,
            IRQ_F2_F3_FIFO_RD_UNDERFLOW,
            IRQ_F2_F3_FIFO_WR_OVERFLOW,
            IRQ_COMMAND_ERROR,
            IRQ_DATA_ERROR,
            IRQ_F2_PACKET_AVAILABLE,
            IRQ_F3_PACKET_AVAILABLE,
            IRQ_F1_OVERFLOW,
            IRQ_MISC_INTR0,
            IRQ_MISC_INTR1,
            IRQ_MISC_INTR2,
            IRQ_MISC_INTR3,
            IRQ_MISC_INTR4,
            IRQ_F1_INTR,
            IRQ_F2_INTR,
            IRQ_F3_INTR
        );
        Ok(())
    }
}

#[cfg(feature = "log")]
impl core::fmt::Display for FormatInterrupt {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(self, f)
    }
}

#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u32)]
pub(crate) enum Ioctl {
    GetMagic = 0,
    GetVersion = 1,
    Up = 2,
    Down = 3,
    GetLoop = 4,
    SetLoop = 5,
    Dump = 6,
    GetMsglevel = 7,
    SetMsglevel = 8,
    GetPromisc = 9,
    SetPromisc = 10,
    GetRate = 12,
    GetInstance = 14,
    GetInfra = 19,
    SetInfra = 20,
    GetAuth = 21,
    SetAuth = 22,
    GetBssid = 23,
    SetBssid = 24,
    GetSsid = 25,
    SetSsid = 26,
    Restart = 27,
    GetChannel = 29,
    SetChannel = 30,
    GetSrl = 31,
    SetSrl = 32,
    GetLrl = 33,
    SetLrl = 34,
    GetPlcphdr = 35,
    SetPlcphdr = 36,
    GetRadio = 37,
    SetRadio = 38,
    GetPhytype = 39,
    DumpRate = 40,
    SetRateParams = 41,
    GetKey = 44,
    SetKey = 45,
    GetRegulatory = 46,
    SetRegulatory = 47,
    GetPassiveScan = 48,
    SetPassiveScan = 49,
    Scan = 50,
    ScanResults = 51,
    Disassoc = 52,
    Reassoc = 53,
    GetRoamTrigger = 54,
    SetRoamTrigger = 55,
    GetRoamDelta = 56,
    SetRoamDelta = 57,
    GetRoamScanPeriod = 58,
    SetRoamScanPeriod = 59,
    Evm = 60,
    GetTxant = 61,
    SetTxant = 62,
    GetAntdiv = 63,
    SetAntdiv = 64,
    GetClosed = 67,
    SetClosed = 68,
    GetMaclist = 69,
    SetMaclist = 70,
    GetRateset = 71,
    SetRateset = 72,
    Longtrain = 74,
    GetBcnprd = 75,
    SetBcnprd = 76,
    GetDtimprd = 77,
    SetDtimprd = 78,
    GetSrom = 79,
    SetSrom = 80,
    GetWepRestrict = 81,
    SetWepRestrict = 82,
    GetCountry = 83,
    SetCountry = 84,
    GetPm = 85,
    SetPm = 86,
    GetWake = 87,
    SetWake = 88,
    GetForcelink = 90,
    SetForcelink = 91,
    FreqAccuracy = 92,
    CarrierSuppress = 93,
    GetPhyreg = 94,
    SetPhyreg = 95,
    GetRadioreg = 96,
    SetRadioreg = 97,
    GetRevinfo = 98,
    GetUcantdiv = 99,
    SetUcantdiv = 100,
    RReg = 101,
    WReg = 102,
    GetMacmode = 105,
    SetMacmode = 106,
    GetMonitor = 107,
    SetMonitor = 108,
    GetGmode = 109,
    SetGmode = 110,
    GetLegacyErp = 111,
    SetLegacyErp = 112,
    GetRxAnt = 113,
    GetCurrRateset = 114,
    GetScansuppress = 115,
    SetScansuppress = 116,
    GetAp = 117,
    SetAp = 118,
    GetEapRestrict = 119,
    SetEapRestrict = 120,
    ScbAuthorize = 121,
    ScbDeauthorize = 122,
    GetWdslist = 123,
    SetWdslist = 124,
    GetAtim = 125,
    SetAtim = 126,
    GetRssi = 127,
    GetPhyantdiv = 128,
    SetPhyantdiv = 129,
    ApRxOnly = 130,
    GetTxPathPwr = 131,
    SetTxPathPwr = 132,
    GetWsec = 133,
    SetWsec = 134,
    GetPhyNoise = 135,
    GetBssInfo = 136,
    GetPktcnts = 137,
    GetLazywds = 138,
    SetLazywds = 139,
    GetBandlist = 140,
    GetBand = 141,
    SetBand = 142,
    ScbDeauthenticate = 143,
    GetShortslot = 144,
    GetShortslotOverride = 145,
    SetShortslotOverride = 146,
    GetShortslotRestrict = 147,
    SetShortslotRestrict = 148,
    GetGmodeProtection = 149,
    GetGmodeProtectionOverride = 150,
    SetGmodeProtectionOverride = 151,
    Upgrade = 152,
    GetIgnoreBcns = 155,
    SetIgnoreBcns = 156,
    GetScbTimeout = 157,
    SetScbTimeout = 158,
    GetAssoclist = 159,
    GetClk = 160,
    SetClk = 161,
    GetUp = 162,
    Out = 163,
    GetWpaAuth = 164,
    SetWpaAuth = 165,
    GetUcflags = 166,
    SetUcflags = 167,
    GetPwridx = 168,
    SetPwridx = 169,
    GetTssi = 170,
    GetSupRatesetOverride = 171,
    SetSupRatesetOverride = 172,
    GetProtectionControl = 178,
    SetProtectionControl = 179,
    GetPhylist = 180,
    EncryptStrength = 181,
    DecryptStatus = 182,
    GetKeySeq = 183,
    GetScanChannelTime = 184,
    SetScanChannelTime = 185,
    GetScanUnassocTime = 186,
    SetScanUnassocTime = 187,
    GetScanHomeTime = 188,
    SetScanHomeTime = 189,
    GetScanNprobes = 190,
    SetScanNprobes = 191,
    GetPrbRespTimeout = 192,
    SetPrbRespTimeout = 193,
    GetAtten = 194,
    SetAtten = 195,
    GetShmem = 196,
    SetShmem = 197,
    SetWsecTest = 200,
    ScbDeauthenticateForReason = 201,
    TkipCountermeasures = 202,
    GetPiomode = 203,
    SetPiomode = 204,
    SetAssocPrefer = 205,
    GetAssocPrefer = 206,
    SetRoamPrefer = 207,
    GetRoamPrefer = 208,
    SetLed = 209,
    GetLed = 210,
    GetInterferenceMode = 211,
    SetInterferenceMode = 212,
    GetChannelQa = 213,
    StartChannelQa = 214,
    GetChannelSel = 215,
    StartChannelSel = 216,
    GetValidChannels = 217,
    GetFakefrag = 218,
    SetFakefrag = 219,
    GetPwroutPercentage = 220,
    SetPwroutPercentage = 221,
    SetBadFramePreempt = 222,
    GetBadFramePreempt = 223,
    SetLeapList = 224,
    GetLeapList = 225,
    GetCwmin = 226,
    SetCwmin = 227,
    GetCwmax = 228,
    SetCwmax = 229,
    GetWet = 230,
    SetWet = 231,
    GetPub = 232,
    GetKeyPrimary = 235,
    SetKeyPrimary = 236,
    GetAciArgs = 238,
    SetAciArgs = 239,
    UnsetCallback = 240,
    SetCallback = 241,
    GetRadar = 242,
    SetRadar = 243,
    SetSpectManagment = 244,
    GetSpectManagment = 245,
    WdsGetRemoteHwaddr = 246,
    WdsGetWpaSup = 247,
    SetCsScanTimer = 248,
    GetCsScanTimer = 249,
    MeasureRequest = 250,
    Init = 251,
    SendQuiet = 252,
    Keepalive = 253,
    SendPwrConstraint = 254,
    UpgradeStatus = 255,
    CurrentPwr = 256,
    GetScanPassiveTime = 257,
    SetScanPassiveTime = 258,
    LegacyLinkBehavior = 259,
    GetChannelsInCountry = 260,
    GetCountryList = 261,
    GetVar = 262,
    SetVar = 263,
    NvramGet = 264,
    NvramSet = 265,
    NvramDump = 266,
    Reboot = 267,
    SetWsecPmk = 268,
    GetAuthMode = 269,
    SetAuthMode = 270,
    GetWakeentry = 271,
    SetWakeentry = 272,
    NdconfigItem = 273,
    Nvotpw = 274,
    Otpw = 275,
    IovBlockGet = 276,
    IovModulesGet = 277,
    SoftReset = 278,
    GetAllowMode = 279,
    SetAllowMode = 280,
    GetDesiredBssid = 281,
    SetDesiredBssid = 282,
    DisassocMyap = 283,
    GetNbands = 284,
    GetBandstates = 285,
    GetWlcBssInfo = 286,
    GetAssocInfo = 287,
    GetOidPhy = 288,
    SetOidPhy = 289,
    SetAssocTime = 290,
    GetDesiredSsid = 291,
    GetChanspec = 292,
    GetAssocState = 293,
    SetPhyState = 294,
    GetScanPending = 295,
    GetScanreqPending = 296,
    GetPrevRoamReason = 297,
    SetPrevRoamReason = 298,
    GetBandstatesPi = 299,
    GetPhyState = 300,
    GetBssWpaRsn = 301,
    GetBssWpa2Rsn = 302,
    GetBssBcnTs = 303,
    GetIntDisassoc = 304,
    SetNumPeers = 305,
    GetNumBss = 306,
    GetWsecPmk = 318,
    GetRandomBytes = 319,
}

pub(crate) const WSEC_TKIP: u32 = 0x02;
pub(crate) const WSEC_AES: u32 = 0x04;

pub(crate) const AUTH_OPEN: u32 = 0x00;
pub(crate) const AUTH_SAE: u32 = 0x03;

pub(crate) const MFP_NONE: u32 = 0;
pub(crate) const MFP_CAPABLE: u32 = 1;
pub(crate) const MFP_REQUIRED: u32 = 2;

pub(crate) const WPA_AUTH_DISABLED: u32 = 0x0000;
pub(crate) const WPA_AUTH_WPA_PSK: u32 = 0x0004;
pub(crate) const WPA_AUTH_WPA2_PSK: u32 = 0x0080;
pub(crate) const WPA_AUTH_WPA3_SAE_PSK: u32 = 0x40000;
