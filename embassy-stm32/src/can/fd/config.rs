//! Configuration for FDCAN Module
// Note: This file is copied and modified from fdcan crate by Richard Meadows

use core::num::{NonZeroU16, NonZeroU8};

/// Configures the bit timings.
///
/// You can use <http://www.bittiming.can-wiki.info/> to calculate the `btr` parameter. Enter
/// parameters as follows:
///
/// - *Clock Rate*: The input clock speed to the CAN peripheral (*not* the CPU clock speed).
///   This is the clock rate of the peripheral bus the CAN peripheral is attached to (eg. APB1).
/// - *Sample Point*: Should normally be left at the default value of 87.5%.
/// - *SJW*: Should normally be left at the default value of 1.
///
/// Then copy the `CAN_BUS_TIME` register value from the table and pass it as the `btr`
/// parameter to this method.
#[derive(Clone, Copy, Debug)]
pub struct NominalBitTiming {
    /// Value by which the oscillator frequency is divided for generating the bit time quanta. The bit
    /// time is built up from a multiple of this quanta. Valid values are 1 to 512.
    pub prescaler: NonZeroU16,
    /// Valid values are 1 to 128.
    pub seg1: NonZeroU8,
    /// Valid values are 1 to 255.
    pub seg2: NonZeroU8,
    /// Valid values are 1 to 128.
    pub sync_jump_width: NonZeroU8,
}
impl NominalBitTiming {
    #[inline]
    pub(crate) fn nbrp(&self) -> u16 {
        u16::from(self.prescaler) & 0x1FF
    }
    #[inline]
    pub(crate) fn ntseg1(&self) -> u8 {
        u8::from(self.seg1)
    }
    #[inline]
    pub(crate) fn ntseg2(&self) -> u8 {
        u8::from(self.seg2) & 0x7F
    }
    #[inline]
    pub(crate) fn nsjw(&self) -> u8 {
        u8::from(self.sync_jump_width) & 0x7F
    }
}

impl Default for NominalBitTiming {
    #[inline]
    fn default() -> Self {
        // Kernel Clock 8MHz, Bit rate: 500kbit/s. Corresponds to a NBTP
        // register value of 0x0600_0A03
        Self {
            prescaler: NonZeroU16::new(1).unwrap(),
            seg1: NonZeroU8::new(11).unwrap(),
            seg2: NonZeroU8::new(4).unwrap(),
            sync_jump_width: NonZeroU8::new(4).unwrap(),
        }
    }
}

/// Configures the data bit timings for the FdCan Variable Bitrates.
/// This is not used when frame_transmit is set to anything other than AllowFdCanAndBRS.
#[derive(Clone, Copy, Debug)]
pub struct DataBitTiming {
    /// Tranceiver Delay Compensation
    pub transceiver_delay_compensation: bool,
    ///  The value by which the oscillator frequency is divided to generate the bit time quanta. The bit
    ///  time is built up from a multiple of this quanta. Valid values for the Baud Rate Prescaler are 1
    ///  to 31.
    pub prescaler: NonZeroU16,
    /// Valid values are 1 to 31.
    pub seg1: NonZeroU8,
    /// Valid values are 1 to 15.
    pub seg2: NonZeroU8,
    /// Must always be smaller than DTSEG2, valid values are 1 to 15.
    pub sync_jump_width: NonZeroU8,
}
impl DataBitTiming {
    // #[inline]
    // fn tdc(&self) -> u8 {
    //     let tsd = self.transceiver_delay_compensation as u8;
    //     //TODO: stm32g4 does not export the TDC field
    //     todo!()
    // }
    #[inline]
    pub(crate) fn dbrp(&self) -> u8 {
        (u16::from(self.prescaler) & 0x001F) as u8
    }
    #[inline]
    pub(crate) fn dtseg1(&self) -> u8 {
        u8::from(self.seg1) & 0x1F
    }
    #[inline]
    pub(crate) fn dtseg2(&self) -> u8 {
        u8::from(self.seg2) & 0x0F
    }
    #[inline]
    pub(crate) fn dsjw(&self) -> u8 {
        u8::from(self.sync_jump_width) & 0x0F
    }
}

impl Default for DataBitTiming {
    #[inline]
    fn default() -> Self {
        // Kernel Clock 8MHz, Bit rate: 500kbit/s. Corresponds to a DBTP
        // register value of 0x0000_0A33
        Self {
            transceiver_delay_compensation: false,
            prescaler: NonZeroU16::new(1).unwrap(),
            seg1: NonZeroU8::new(11).unwrap(),
            seg2: NonZeroU8::new(4).unwrap(),
            sync_jump_width: NonZeroU8::new(4).unwrap(),
        }
    }
}

/// Configures which modes to use
/// Individual headers can contain a desire to be send via FdCan
/// or use Bit rate switching. But if this general setting does not allow
/// that, only classic CAN is used instead.
#[derive(Clone, Copy, Debug)]
pub enum FrameTransmissionConfig {
    /// Only allow Classic CAN message Frames
    ClassicCanOnly,
    /// Allow (non-brs) FdCAN Message Frames
    AllowFdCan,
    /// Allow FdCAN Message Frames and allow Bit Rate Switching
    AllowFdCanAndBRS,
}

///
#[derive(Clone, Copy, Debug)]
pub enum ClockDivider {
    /// Divide by 1
    _1 = 0b0000,
    /// Divide by 2
    _2 = 0b0001,
    /// Divide by 4
    _4 = 0b0010,
    /// Divide by 6
    _6 = 0b0011,
    /// Divide by 8
    _8 = 0b0100,
    /// Divide by 10
    _10 = 0b0101,
    /// Divide by 12
    _12 = 0b0110,
    /// Divide by 14
    _14 = 0b0111,
    /// Divide by 16
    _16 = 0b1000,
    /// Divide by 18
    _18 = 0b1001,
    /// Divide by 20
    _20 = 0b1010,
    /// Divide by 22
    _22 = 0b1011,
    /// Divide by 24
    _24 = 0b1100,
    /// Divide by 26
    _26 = 0b1101,
    /// Divide by 28
    _28 = 0b1110,
    /// Divide by 30
    _30 = 0b1111,
}

/// Prescaler of the Timestamp counter
#[derive(Clone, Copy, Debug)]
pub enum TimestampPrescaler {
    /// 1
    _1 = 1,
    /// 2
    _2 = 2,
    /// 3
    _3 = 3,
    /// 4
    _4 = 4,
    /// 5
    _5 = 5,
    /// 6
    _6 = 6,
    /// 7
    _7 = 7,
    /// 8
    _8 = 8,
    /// 9
    _9 = 9,
    /// 10
    _10 = 10,
    /// 11
    _11 = 11,
    /// 12
    _12 = 12,
    /// 13
    _13 = 13,
    /// 14
    _14 = 14,
    /// 15
    _15 = 15,
    /// 16
    _16 = 16,
}

/// Selects the source of the Timestamp counter
#[derive(Clone, Copy, Debug)]
pub enum TimestampSource {
    /// The Timestamp counter is disabled
    None,
    /// Using the FdCan input clock as the Timstamp counter's source,
    /// and using a specific prescaler
    Prescaler(TimestampPrescaler),
    /// Using TIM3 as a source
    FromTIM3,
}

/// How to handle frames in the global filter
#[derive(Clone, Copy, Debug)]
pub enum NonMatchingFilter {
    /// Frames will go to Fifo0 when they do no match any specific filter
    IntoRxFifo0 = 0b00,
    /// Frames will go to Fifo1 when they do no match any specific filter
    IntoRxFifo1 = 0b01,
    /// Frames will be rejected when they do not match any specific filter
    Reject = 0b11,
}

/// How to handle frames which do not match a specific filter
#[derive(Clone, Copy, Debug)]
pub struct GlobalFilter {
    /// How to handle non-matching standard frames
    pub handle_standard_frames: NonMatchingFilter,

    /// How to handle non-matching extended frames
    pub handle_extended_frames: NonMatchingFilter,

    /// How to handle remote standard frames
    pub reject_remote_standard_frames: bool,

    /// How to handle remote extended frames
    pub reject_remote_extended_frames: bool,
}
impl GlobalFilter {
    /// Reject all non-matching and remote frames
    pub const fn reject_all() -> Self {
        Self {
            handle_standard_frames: NonMatchingFilter::Reject,
            handle_extended_frames: NonMatchingFilter::Reject,
            reject_remote_standard_frames: true,
            reject_remote_extended_frames: true,
        }
    }

    /// How to handle non-matching standard frames
    pub const fn set_handle_standard_frames(mut self, filter: NonMatchingFilter) -> Self {
        self.handle_standard_frames = filter;
        self
    }
    /// How to handle non-matching exteded frames
    pub const fn set_handle_extended_frames(mut self, filter: NonMatchingFilter) -> Self {
        self.handle_extended_frames = filter;
        self
    }
    /// How to handle remote standard frames
    pub const fn set_reject_remote_standard_frames(mut self, filter: bool) -> Self {
        self.reject_remote_standard_frames = filter;
        self
    }
    /// How to handle remote extended frames
    pub const fn set_reject_remote_extended_frames(mut self, filter: bool) -> Self {
        self.reject_remote_extended_frames = filter;
        self
    }
}
impl Default for GlobalFilter {
    #[inline]
    fn default() -> Self {
        Self {
            handle_standard_frames: NonMatchingFilter::IntoRxFifo0,
            handle_extended_frames: NonMatchingFilter::IntoRxFifo0,
            reject_remote_standard_frames: false,
            reject_remote_extended_frames: false,
        }
    }
}

/// TX buffer operation mode
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TxBufferMode {
    /// TX FIFO operation - In this mode CAN frames are trasmitted strictly in write order.
    Fifo,
    /// TX priority queue operation - In this mode CAN frames are transmitted according to CAN priority.
    Priority,
}

impl From<TxBufferMode> for crate::pac::can::vals::Tfqm {
    fn from(value: TxBufferMode) -> Self {
        match value {
            TxBufferMode::Priority => Self::QUEUE,
            TxBufferMode::Fifo => Self::FIFO,
        }
    }
}

impl From<crate::pac::can::vals::Tfqm> for TxBufferMode {
    fn from(value: crate::pac::can::vals::Tfqm) -> Self {
        match value {
            crate::pac::can::vals::Tfqm::QUEUE => Self::Priority,
            crate::pac::can::vals::Tfqm::FIFO => Self::Fifo,
        }
    }
}

/// FdCan Config Struct
#[derive(Clone, Copy, Debug)]
pub struct FdCanConfig {
    /// Nominal Bit Timings
    pub nbtr: NominalBitTiming,
    /// (Variable) Data Bit Timings
    pub dbtr: DataBitTiming,
    /// Enables or disables automatic retransmission of messages
    ///
    /// If this is enabled, the CAN peripheral will automatically try to retransmit each frame
    /// util it can be sent. Otherwise, it will try only once to send each frame.
    ///
    /// Automatic retransmission is enabled by default.
    pub automatic_retransmit: bool,
    /// The transmit pause feature is intended for use in CAN systems where the CAN message
    /// identifiers are permanently specified to specific values and cannot easily be changed.
    ///
    /// These message identifiers can have a higher CAN arbitration priority than other defined
    /// messages, while in a specific application their relative arbitration priority must be inverse.
    ///
    /// This may lead to a case where one ECU sends a burst of CAN messages that cause
    /// another ECU CAN messages to be delayed because that other messages have a lower
    /// CAN arbitration priority.
    pub transmit_pause: bool,
    /// Enabled or disables the pausing between transmissions
    ///
    /// This feature looses up burst transmissions coming from a single node and it protects against
    /// "babbling idiot" scenarios where the application program erroneously requests too many
    /// transmissions.
    pub frame_transmit: FrameTransmissionConfig,
    /// Non Isoe Mode
    /// If this is set, the FDCAN uses the CAN FD frame format as specified by the Bosch CAN
    /// FD Specification V1.0.
    pub non_iso_mode: bool,
    /// Edge Filtering: Two consecutive dominant tq required to detect an edge for hard synchronization
    pub edge_filtering: bool,
    /// Enables protocol exception handling
    pub protocol_exception_handling: bool,
    /// Sets the general clock divider for this FdCAN instance
    pub clock_divider: ClockDivider,
    /// Sets the timestamp source
    pub timestamp_source: TimestampSource,
    /// Configures the Global Filter
    pub global_filter: GlobalFilter,
    /// TX buffer mode (FIFO or priority queue)
    pub tx_buffer_mode: TxBufferMode,
}

impl FdCanConfig {
    /// Configures the bit timings.
    #[inline]
    pub const fn set_nominal_bit_timing(mut self, btr: NominalBitTiming) -> Self {
        self.nbtr = btr;
        self
    }

    /// Configures the bit timings.
    #[inline]
    pub const fn set_data_bit_timing(mut self, btr: DataBitTiming) -> Self {
        self.dbtr = btr;
        self
    }

    /// Enables or disables automatic retransmission of messages
    ///
    /// If this is enabled, the CAN peripheral will automatically try to retransmit each frame
    /// util it can be sent. Otherwise, it will try only once to send each frame.
    ///
    /// Automatic retransmission is enabled by default.
    #[inline]
    pub const fn set_automatic_retransmit(mut self, enabled: bool) -> Self {
        self.automatic_retransmit = enabled;
        self
    }

    /// Enabled or disables the pausing between transmissions
    ///
    /// This feature looses up burst transmissions coming from a single node and it protects against
    /// "babbling idiot" scenarios where the application program erroneously requests too many
    /// transmissions.
    #[inline]
    pub const fn set_transmit_pause(mut self, enabled: bool) -> Self {
        self.transmit_pause = enabled;
        self
    }

    /// If this is set, the FDCAN uses the CAN FD frame format as specified by the Bosch CAN
    /// FD Specification V1.0.
    #[inline]
    pub const fn set_non_iso_mode(mut self, enabled: bool) -> Self {
        self.non_iso_mode = enabled;
        self
    }

    /// Two consecutive dominant tq required to detect an edge for hard synchronization
    #[inline]
    pub const fn set_edge_filtering(mut self, enabled: bool) -> Self {
        self.edge_filtering = enabled;
        self
    }

    /// Sets the allowed transmission types for messages.
    #[inline]
    pub const fn set_frame_transmit(mut self, fts: FrameTransmissionConfig) -> Self {
        self.frame_transmit = fts;
        self
    }

    /// Enables protocol exception handling
    #[inline]
    pub const fn set_protocol_exception_handling(mut self, peh: bool) -> Self {
        self.protocol_exception_handling = peh;
        self
    }

    /// Sets the general clock divider for this FdCAN instance
    #[inline]
    pub const fn set_clock_divider(mut self, div: ClockDivider) -> Self {
        self.clock_divider = div;
        self
    }

    /// Sets the timestamp source
    #[inline]
    pub const fn set_timestamp_source(mut self, tss: TimestampSource) -> Self {
        self.timestamp_source = tss;
        self
    }

    /// Sets the global filter settings
    #[inline]
    pub const fn set_global_filter(mut self, filter: GlobalFilter) -> Self {
        self.global_filter = filter;
        self
    }

    /// Sets the TX buffer mode (FIFO or priority queue)
    #[inline]
    pub const fn set_tx_buffer_mode(mut self, txbm: TxBufferMode) -> Self {
        self.tx_buffer_mode = txbm;
        self
    }
}

impl Default for FdCanConfig {
    #[inline]
    fn default() -> Self {
        Self {
            nbtr: NominalBitTiming::default(),
            dbtr: DataBitTiming::default(),
            automatic_retransmit: true,
            transmit_pause: false,
            frame_transmit: FrameTransmissionConfig::ClassicCanOnly,
            non_iso_mode: false,
            edge_filtering: false,
            protocol_exception_handling: true,
            clock_divider: ClockDivider::_1,
            timestamp_source: TimestampSource::None,
            global_filter: GlobalFilter::default(),
            tx_buffer_mode: TxBufferMode::Priority,
        }
    }
}
