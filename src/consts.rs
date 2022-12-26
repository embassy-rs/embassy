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
pub(crate) const IOCTL_CMD_SET_SSID: u32 = 26;
pub(crate) const IOCTL_CMD_ANTDIV: u32 = 64;
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
