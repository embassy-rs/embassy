/// For RX:
/// Excess data is IGNORED, only the number of bytes which fit
/// into the element are stored.
///
/// For TX:
/// If DLC is higher than the data field size, excess bytes are
/// transmitted as 0xCC (padding bytes).
#[derive(Clone, Copy, Debug)]
pub enum DataFieldSize {
    /// Supports storing DLC up to 8 bytes.
    B8 = 0b000,
    /// Supports storing DLC up to 12 bytes.
    B12 = 0b001,
    /// Supports storing DLC up to 16 bytes.
    B16 = 0b010,
    /// Supports storing DLC up to 20 bytes.
    B20 = 0b011,
    /// Supports storing DLC up to 24 bytes.
    B24 = 0b100,
    /// Supports storing DLC up to 32 bytes.
    B32 = 0b101,
    /// Supports storing DLC up to 48 bytes.
    B48 = 0b110,
    /// Supports storing DLC up to 64 bytes.
    B64 = 0b111,
}

impl DataFieldSize {
    pub(crate) fn reg_value(self) -> u8 {
        self as u8
    }

    /// Returns the max byte size for this setting.
    pub fn byte_size(self) -> usize {
        match self {
            DataFieldSize::B8 => 8,
            DataFieldSize::B12 => 12,
            DataFieldSize::B16 => 16,
            DataFieldSize::B20 => 20,
            DataFieldSize::B24 => 24,
            DataFieldSize::B32 => 32,
            DataFieldSize::B48 => 48,
            DataFieldSize::B64 => 64,
        }
    }

    pub(crate) fn word_size(self) -> usize {
        self.byte_size() / 4
    }
}

/// Configuration for an Rx FIFO
#[derive(Clone, Copy, Debug)]
pub struct RxFifoConfig {
    /// 0: Disabled
    /// 1-64: Watermark interrupt level
    /// >64: Disabled
    pub watermark_interrupt_level: u8,
    /// 0-64: Number of RX FIFO elements
    pub fifo_size: u8,

    /// The data size for each Tx buffer. This will indicate the max
    /// data length you will be able to send.
    ///
    /// If you are using Classic CAN only, there is no reason to set
    /// this to any value above B8.
    ///
    /// If you receive a frame with data that doesn't fit within the
    /// configured data field size, the data will be truncated.
    pub data_field_size: DataFieldSize,
}

impl RxFifoConfig {
    /// Configuration which disables the FIFO.
    pub const DISABLED: Self = RxFifoConfig {
        watermark_interrupt_level: 0,
        fifo_size: 0,
        data_field_size: DataFieldSize::B8,
    };
}

/// Configuration for an RX Buffer
#[derive(Clone, Copy, Debug)]
pub struct RxBufferConfig {
    /// 0-64: Number of RX Buffer elements
    pub size: u8,

    /// The data size for each Tx buffer. This will indicate the max
    /// data length you will be able to send.
    ///
    /// If you are using Classic CAN only, there is no reason to set
    /// this to any value above B8.
    ///
    /// If you receive a frame with data that doesn't fit within the
    /// configured data field size, the data will be truncated.
    pub data_field_size: DataFieldSize,
}

impl RxBufferConfig {
    /// Configuration which disables the buffer.
    pub const DISABLED: Self = RxBufferConfig {
        size: 0,
        data_field_size: DataFieldSize::B8,
    };
}

/// Configuration for TX buffers
#[derive(Clone, Copy, Debug)]
pub struct TxConfig {
    /// Number of elements reserved for TX Queue.
    /// NOTE: queue_size + dedicated_size may not be greater than 32.
    ///
    /// 0-32: Number of TX buffers used for TX FIFO/Priority queue
    pub queue_size: u8,
    /// Number of elements reserved for Dedicated TX buffers.
    /// NOTE: queue_size + dedicated_size may not be greater than 32.
    ///
    /// 0-32: Number of TX buffers used for TX dedicated buffers
    pub dedicated_size: u8,
    /// The data size for each Tx buffer. This will indicate the max
    /// data length you will be able to send.
    ///
    /// If you are using Classic CAN only, there is no reason to set
    /// this to any value above B8.
    pub data_field_size: DataFieldSize,
}

/// Configuration for Message RAM layout.
///
/// The default layout is as follows:
/// * 32 standard and extended filters
/// * 32 x 64 byte dedicated Rx buffers
/// * 16 x 64 byte elements in Rx queue 0 and 1
/// * 8 x 64 byte dedicated Tx buffers
/// * 24 x 64 byte Tx queue
#[derive(Clone, Copy, Debug)]
pub struct MessageRamConfig {
    /// 0-128: Number of standard Message ID filter elements
    /// >128: Interpreted as 128
    pub standard_id_filter_size: u8,
    /// 0-64: Number of extended Message ID filter elements
    /// >64: Interpreted as 64
    pub extended_id_filter_size: u8,

    /// Configuration for Rx FIFO 0
    pub rx_fifo_0: RxFifoConfig,
    /// Configuration for Rx FIFO 1
    pub rx_fifo_1: RxFifoConfig,
    /// Configuration for Rx Buffers
    pub rx_buffer: RxBufferConfig,

    /// Configuration for Tx FIFO/Queue and dedicated buffers
    pub tx: TxConfig,
}

impl MessageRamConfig {
    /// Contains a sane default Message RAM config for CAN Classic only frames.
    pub const CAN_CLASSIC_DEFAULT: MessageRamConfig = MessageRamConfig {
        standard_id_filter_size: 32,
        extended_id_filter_size: 32,
        rx_fifo_0: RxFifoConfig {
            watermark_interrupt_level: 0,
            fifo_size: 16,
            data_field_size: DataFieldSize::B8,
        },
        rx_fifo_1: RxFifoConfig {
            watermark_interrupt_level: 0,
            fifo_size: 16,
            data_field_size: DataFieldSize::B8,
        },
        rx_buffer: RxBufferConfig {
            size: 16,
            data_field_size: DataFieldSize::B8,
        },
        tx: TxConfig {
            queue_size: 24,
            dedicated_size: 8,
            data_field_size: DataFieldSize::B8,
        },
    };

    /// Contains a sane default message RAM config supporting CAN FD frames.
    pub const CAN_FD_DEFAULT: MessageRamConfig = MessageRamConfig {
        standard_id_filter_size: 32,
        extended_id_filter_size: 32,
        rx_fifo_0: RxFifoConfig {
            watermark_interrupt_level: 0,
            fifo_size: 6,
            data_field_size: DataFieldSize::B64,
        },
        rx_fifo_1: RxFifoConfig {
            watermark_interrupt_level: 0,
            fifo_size: 6,
            data_field_size: DataFieldSize::B64,
        },
        rx_buffer: RxBufferConfig {
            size: 4,
            data_field_size: DataFieldSize::B64,
        },
        tx: TxConfig {
            queue_size: 12,
            dedicated_size: 4,
            data_field_size: DataFieldSize::B64,
        },
    };
}

impl Default for MessageRamConfig {
    fn default() -> Self {
        Self::CAN_FD_DEFAULT
    }
}
