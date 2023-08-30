#![allow(unused)]

pub(crate) const FUNC_BUS: u32 = 0;
pub(crate) const FUNC_BACKPLANE: u32 = 1;
pub(crate) const FUNC_WLAN: u32 = 2;
pub(crate) const FUNC_BT: u32 = 3;

pub(crate) const REG_BUS_CTRL: u32 = 0x0;
pub(crate) const REG_BUS_INTERRUPT: u32 = 0x04; // 16 bits - Interrupt status
pub(crate) const REG_BUS_INTERRUPT_ENABLE: u32 = 0x06; // 16 bits - Interrupt mask
pub(crate) const REG_BUS_STATUS: u32 = 0x8;
pub(crate) const REG_BUS_TEST_RO: u32 = 0x14;
pub(crate) const REG_BUS_TEST_RW: u32 = 0x18;
pub(crate) const REG_BUS_RESP_DELAY: u32 = 0x1c;
pub(crate) const WORD_LENGTH_32: u32 = 0x1;
pub(crate) const HIGH_SPEED: u32 = 0x10;
pub(crate) const INTERRUPT_HIGH: u32 = 1 << 5;
pub(crate) const WAKE_UP: u32 = 1 << 7;
pub(crate) const STATUS_ENABLE: u32 = 1 << 16;
pub(crate) const INTERRUPT_WITH_STATUS: u32 = 1 << 17;

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

pub(crate) const IOCTL_CMD_UP: u32 = 2;
pub(crate) const IOCTL_CMD_DOWN: u32 = 3;
pub(crate) const IOCTL_CMD_SET_SSID: u32 = 26;
pub(crate) const IOCTL_CMD_SET_CHANNEL: u32 = 30;
pub(crate) const IOCTL_CMD_DISASSOC: u32 = 52;
pub(crate) const IOCTL_CMD_ANTDIV: u32 = 64;
pub(crate) const IOCTL_CMD_SET_AP: u32 = 118;
pub(crate) const IOCTL_CMD_SET_VAR: u32 = 263;
pub(crate) const IOCTL_CMD_GET_VAR: u32 = 262;
pub(crate) const IOCTL_CMD_SET_PASSPHRASE: u32 = 268;

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
