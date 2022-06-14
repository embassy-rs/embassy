/// Preamble detection length for [`GenericPacketParams`].
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum PreambleDetection {
    /// Preamble detection disabled.
    Disabled = 0x0,
    /// 8-bit preamble detection.
    Bit8 = 0x4,
    /// 16-bit preamble detection.
    Bit16 = 0x5,
    /// 24-bit preamble detection.
    Bit24 = 0x6,
    /// 32-bit preamble detection.
    Bit32 = 0x7,
}

/// Address comparison/filtering for [`GenericPacketParams`].
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum AddrComp {
    /// Address comparison/filtering disabled.
    Disabled = 0x0,
    /// Address comparison/filtering on node address.
    Node = 0x1,
    /// Address comparison/filtering on node and broadcast addresses.
    Broadcast = 0x2,
}

/// Packet header type.
///
/// Argument of [`GenericPacketParams::set_header_type`] and
/// [`LoRaPacketParams::set_header_type`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum HeaderType {
    /// Fixed; payload length and header field not added to packet.
    Fixed,
    /// Variable; payload length and header field added to packet.
    Variable,
}

impl HeaderType {
    pub(crate) const fn to_bits_generic(self) -> u8 {
        match self {
            HeaderType::Fixed => 0,
            HeaderType::Variable => 1,
        }
    }

    pub(crate) const fn to_bits_lora(self) -> u8 {
        match self {
            HeaderType::Fixed => 1,
            HeaderType::Variable => 0,
        }
    }
}

/// CRC type definition for [`GenericPacketParams`].
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum CrcType {
    /// 1-byte CRC.
    Byte1 = 0x0,
    /// CRC disabled.
    Disabled = 0x1,
    /// 2-byte CRC.
    Byte2 = 0x2,
    /// 1-byte inverted CRC.
    Byte1Inverted = 0x4,
    /// 2-byte inverted CRC.
    Byte2Inverted = 0x6,
}

/// Packet parameters for [`set_packet_params`].
///
/// [`set_packet_params`]: super::SubGhz::set_packet_params
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct GenericPacketParams {
    buf: [u8; 10],
}

impl GenericPacketParams {
    /// Create a new `GenericPacketParams`.
    ///
    /// This is the same as `default`, but in a `const` function.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::GenericPacketParams;
    ///
    /// const PKT_PARAMS: GenericPacketParams = GenericPacketParams::new();
    /// assert_eq!(PKT_PARAMS, GenericPacketParams::default());
    /// ```
    pub const fn new() -> GenericPacketParams {
        const OPCODE: u8 = super::OpCode::SetPacketParams as u8;
        // const variable ensure the compile always optimizes the methods
        const NEW: GenericPacketParams = GenericPacketParams {
            buf: [OPCODE, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        }
        .set_preamble_len(1)
        .set_preamble_detection(PreambleDetection::Disabled)
        .set_sync_word_len(0)
        .set_addr_comp(AddrComp::Disabled)
        .set_header_type(HeaderType::Fixed)
        .set_payload_len(1);

        NEW
    }

    /// Preamble length in number of symbols.
    ///
    /// Values of zero are invalid, and will automatically be set to 1.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::GenericPacketParams;
    ///
    /// const PKT_PARAMS: GenericPacketParams = GenericPacketParams::new().set_preamble_len(0x1234);
    /// # assert_eq!(PKT_PARAMS.as_slice()[1], 0x12);
    /// # assert_eq!(PKT_PARAMS.as_slice()[2], 0x34);
    /// ```
    #[must_use = "preamble_length returns a modified GenericPacketParams"]
    pub const fn set_preamble_len(mut self, mut len: u16) -> GenericPacketParams {
        if len == 0 {
            len = 1
        }
        self.buf[1] = ((len >> 8) & 0xFF) as u8;
        self.buf[2] = (len & 0xFF) as u8;
        self
    }

    /// Preamble detection length in number of bit symbols.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::{GenericPacketParams, PreambleDetection};
    ///
    /// const PKT_PARAMS: GenericPacketParams =
    ///     GenericPacketParams::new().set_preamble_detection(PreambleDetection::Bit8);
    /// # assert_eq!(PKT_PARAMS.as_slice()[3], 0x4);
    /// ```
    #[must_use = "set_preamble_detection returns a modified GenericPacketParams"]
    pub const fn set_preamble_detection(
        mut self,
        pb_det: PreambleDetection,
    ) -> GenericPacketParams {
        self.buf[3] = pb_det as u8;
        self
    }

    /// Sync word length in number of bit symbols.
    ///
    /// Valid values are `0x00` - `0x40` for 0 to 64-bits respectively.
    /// Values that exceed the maximum will saturate at `0x40`.
    ///
    /// # Example
    ///
    /// Set the sync word length to 4 bytes (16 bits).
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::GenericPacketParams;
    ///
    /// const PKT_PARAMS: GenericPacketParams = GenericPacketParams::new().set_sync_word_len(16);
    /// # assert_eq!(PKT_PARAMS.as_slice()[4], 0x10);
    /// ```
    #[must_use = "set_sync_word_len returns a modified GenericPacketParams"]
    pub const fn set_sync_word_len(mut self, len: u8) -> GenericPacketParams {
        const MAX: u8 = 0x40;
        if len > MAX {
            self.buf[4] = MAX;
        } else {
            self.buf[4] = len;
        }
        self
    }

    /// Address comparison/filtering.
    ///
    /// # Example
    ///
    /// Enable address on the node address.
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::{AddrComp, GenericPacketParams};
    ///
    /// const PKT_PARAMS: GenericPacketParams =
    ///     GenericPacketParams::new().set_addr_comp(AddrComp::Node);
    /// # assert_eq!(PKT_PARAMS.as_slice()[5], 0x01);
    /// ```
    #[must_use = "set_addr_comp returns a modified GenericPacketParams"]
    pub const fn set_addr_comp(mut self, addr_comp: AddrComp) -> GenericPacketParams {
        self.buf[5] = addr_comp as u8;
        self
    }

    /// Header type definition.
    ///
    /// **Note:** The reference manual calls this packet type, but that results
    /// in a conflicting variable name for the modulation scheme, which the
    /// reference manual also calls packet type.
    ///
    /// # Example
    ///
    /// Set the header type to a variable length.
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::{GenericPacketParams, HeaderType};
    ///
    /// const PKT_PARAMS: GenericPacketParams =
    ///     GenericPacketParams::new().set_header_type(HeaderType::Variable);
    /// # assert_eq!(PKT_PARAMS.as_slice()[6], 0x01);
    /// ```
    #[must_use = "set_header_type returns a modified GenericPacketParams"]
    pub const fn set_header_type(mut self, header_type: HeaderType) -> GenericPacketParams {
        self.buf[6] = header_type.to_bits_generic();
        self
    }

    /// Set the payload length in bytes.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::GenericPacketParams;
    ///
    /// const PKT_PARAMS: GenericPacketParams = GenericPacketParams::new().set_payload_len(12);
    /// # assert_eq!(PKT_PARAMS.as_slice()[7], 12);
    /// ```
    #[must_use = "set_payload_len returns a modified GenericPacketParams"]
    pub const fn set_payload_len(mut self, len: u8) -> GenericPacketParams {
        self.buf[7] = len;
        self
    }

    /// CRC type definition.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::{CrcType, GenericPacketParams};
    ///
    /// const PKT_PARAMS: GenericPacketParams =
    ///     GenericPacketParams::new().set_crc_type(CrcType::Byte2Inverted);
    /// # assert_eq!(PKT_PARAMS.as_slice()[8], 0x6);
    /// ```
    #[must_use = "set_payload_len returns a modified GenericPacketParams"]
    pub const fn set_crc_type(mut self, crc_type: CrcType) -> GenericPacketParams {
        self.buf[8] = crc_type as u8;
        self
    }

    /// Whitening enable.
    ///
    /// # Example
    ///
    /// Enable whitening.
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::GenericPacketParams;
    ///
    /// const PKT_PARAMS: GenericPacketParams = GenericPacketParams::new().set_whitening_enable(true);
    /// # assert_eq!(PKT_PARAMS.as_slice()[9], 1);
    /// ```
    #[must_use = "set_whitening_enable returns a modified GenericPacketParams"]
    pub const fn set_whitening_enable(mut self, en: bool) -> GenericPacketParams {
        self.buf[9] = en as u8;
        self
    }

    /// Extracts a slice containing the packet.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::{
    ///     AddrComp, CrcType, GenericPacketParams, HeaderType, PreambleDetection,
    /// };
    ///
    /// const PKT_PARAMS: GenericPacketParams = GenericPacketParams::new()
    ///     .set_preamble_len(8)
    ///     .set_preamble_detection(PreambleDetection::Disabled)
    ///     .set_sync_word_len(2)
    ///     .set_addr_comp(AddrComp::Disabled)
    ///     .set_header_type(HeaderType::Fixed)
    ///     .set_payload_len(128)
    ///     .set_crc_type(CrcType::Byte2)
    ///     .set_whitening_enable(true);
    ///
    /// assert_eq!(
    ///     PKT_PARAMS.as_slice(),
    ///     &[0x8C, 0x00, 0x08, 0x00, 0x02, 0x00, 0x00, 0x80, 0x02, 0x01]
    /// );
    /// ```
    pub const fn as_slice(&self) -> &[u8] {
        &self.buf
    }
}

impl Default for GenericPacketParams {
    fn default() -> Self {
        Self::new()
    }
}

/// Packet parameters for [`set_lora_packet_params`].
///
/// [`set_lora_packet_params`]: super::SubGhz::set_lora_packet_params
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LoRaPacketParams {
    buf: [u8; 7],
}

impl LoRaPacketParams {
    /// Create a new `GenericPacketParams`.
    ///
    /// This is the same as `default`, but in a `const` function.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::LoRaPacketParams;
    ///
    /// const PKT_PARAMS: LoRaPacketParams = LoRaPacketParams::new();
    /// assert_eq!(PKT_PARAMS, LoRaPacketParams::default());
    /// ```
    pub const fn new() -> LoRaPacketParams {
        const OPCODE: u8 = super::OpCode::SetPacketParams as u8;
        // const variable ensure the compile always optimizes the methods
        const NEW: LoRaPacketParams = LoRaPacketParams {
            buf: [OPCODE, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        }
        .set_preamble_len(1)
        .set_header_type(HeaderType::Fixed)
        .set_payload_len(1)
        .set_crc_en(true)
        .set_invert_iq(false);

        NEW
    }

    /// Preamble length in number of symbols.
    ///
    /// Values of zero are invalid, and will automatically be set to 1.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::LoRaPacketParams;
    ///
    /// const PKT_PARAMS: LoRaPacketParams = LoRaPacketParams::new().set_preamble_len(0x1234);
    /// # assert_eq!(PKT_PARAMS.as_slice()[1], 0x12);
    /// # assert_eq!(PKT_PARAMS.as_slice()[2], 0x34);
    /// ```
    #[must_use = "preamble_length returns a modified LoRaPacketParams"]
    pub const fn set_preamble_len(mut self, mut len: u16) -> LoRaPacketParams {
        if len == 0 {
            len = 1
        }
        self.buf[1] = ((len >> 8) & 0xFF) as u8;
        self.buf[2] = (len & 0xFF) as u8;
        self
    }

    /// Header type (fixed or variable).
    ///
    /// # Example
    ///
    /// Set the payload type to a fixed length.
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::{HeaderType, LoRaPacketParams};
    ///
    /// const PKT_PARAMS: LoRaPacketParams = LoRaPacketParams::new().set_header_type(HeaderType::Fixed);
    /// # assert_eq!(PKT_PARAMS.as_slice()[3], 0x01);
    /// ```
    #[must_use = "set_header_type returns a modified LoRaPacketParams"]
    pub const fn set_header_type(mut self, header_type: HeaderType) -> LoRaPacketParams {
        self.buf[3] = header_type.to_bits_lora();
        self
    }

    /// Set the payload length in bytes.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::LoRaPacketParams;
    ///
    /// const PKT_PARAMS: LoRaPacketParams = LoRaPacketParams::new().set_payload_len(12);
    /// # assert_eq!(PKT_PARAMS.as_slice()[4], 12);
    /// ```
    #[must_use = "set_payload_len returns a modified LoRaPacketParams"]
    pub const fn set_payload_len(mut self, len: u8) -> LoRaPacketParams {
        self.buf[4] = len;
        self
    }

    /// CRC enable.
    ///
    /// # Example
    ///
    /// Enable CRC.
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::LoRaPacketParams;
    ///
    /// const PKT_PARAMS: LoRaPacketParams = LoRaPacketParams::new().set_crc_en(true);
    /// # assert_eq!(PKT_PARAMS.as_slice()[5], 0x1);
    /// ```
    #[must_use = "set_crc_en returns a modified LoRaPacketParams"]
    pub const fn set_crc_en(mut self, en: bool) -> LoRaPacketParams {
        self.buf[5] = en as u8;
        self
    }

    /// IQ setup.
    ///
    /// # Example
    ///
    /// Use an inverted IQ setup.
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::LoRaPacketParams;
    ///
    /// const PKT_PARAMS: LoRaPacketParams = LoRaPacketParams::new().set_invert_iq(true);
    /// # assert_eq!(PKT_PARAMS.as_slice()[6], 0x1);
    /// ```
    #[must_use = "set_invert_iq returns a modified LoRaPacketParams"]
    pub const fn set_invert_iq(mut self, invert: bool) -> LoRaPacketParams {
        self.buf[6] = invert as u8;
        self
    }

    /// Extracts a slice containing the packet.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::{HeaderType, LoRaPacketParams};
    ///
    /// const PKT_PARAMS: LoRaPacketParams = LoRaPacketParams::new()
    ///     .set_preamble_len(5 * 8)
    ///     .set_header_type(HeaderType::Fixed)
    ///     .set_payload_len(64)
    ///     .set_crc_en(true)
    ///     .set_invert_iq(true);
    ///
    /// assert_eq!(
    ///     PKT_PARAMS.as_slice(),
    ///     &[0x8C, 0x00, 0x28, 0x01, 0x40, 0x01, 0x01]
    /// );
    /// ```
    pub const fn as_slice(&self) -> &[u8] {
        &self.buf
    }
}

impl Default for LoRaPacketParams {
    fn default() -> Self {
        Self::new()
    }
}

/// Packet parameters for [`set_lora_packet_params`].
///
/// [`set_lora_packet_params`]: super::SubGhz::set_lora_packet_params
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct BpskPacketParams {
    buf: [u8; 2],
}

impl BpskPacketParams {
    /// Create a new `BpskPacketParams`.
    ///
    /// This is the same as `default`, but in a `const` function.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::BpskPacketParams;
    ///
    /// const PKT_PARAMS: BpskPacketParams = BpskPacketParams::new();
    /// assert_eq!(PKT_PARAMS, BpskPacketParams::default());
    /// ```
    pub const fn new() -> BpskPacketParams {
        BpskPacketParams {
            buf: [super::OpCode::SetPacketParams as u8, 0x00],
        }
    }

    /// Set the payload length in bytes.
    ///
    /// The length includes preamble, sync word, device ID, and CRC.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::BpskPacketParams;
    ///
    /// const PKT_PARAMS: BpskPacketParams = BpskPacketParams::new().set_payload_len(12);
    /// # assert_eq!(PKT_PARAMS.as_slice()[1], 12);
    /// ```
    #[must_use = "set_payload_len returns a modified BpskPacketParams"]
    pub const fn set_payload_len(mut self, len: u8) -> BpskPacketParams {
        self.buf[1] = len;
        self
    }

    /// Extracts a slice containing the packet.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::{BpskPacketParams, HeaderType};
    ///
    /// const PKT_PARAMS: BpskPacketParams = BpskPacketParams::new().set_payload_len(24);
    ///
    /// assert_eq!(PKT_PARAMS.as_slice(), &[0x8C, 24]);
    /// ```
    pub const fn as_slice(&self) -> &[u8] {
        &self.buf
    }
}

impl Default for BpskPacketParams {
    fn default() -> Self {
        Self::new()
    }
}
