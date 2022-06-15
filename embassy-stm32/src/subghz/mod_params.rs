/// Bandwidth options for [`FskModParams`].
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum FskBandwidth {
    /// 4.8 kHz double-sideband
    Bw4 = 0x1F,
    /// 5.8 kHz double-sideband
    Bw5 = 0x17,
    /// 7.3 kHz double-sideband
    Bw7 = 0x0F,
    /// 9.7 kHz double-sideband
    Bw9 = 0x1E,
    /// 11.7 kHz double-sideband
    Bw11 = 0x16,
    /// 14.6 kHz double-sideband
    Bw14 = 0x0E,
    /// 19.5 kHz double-sideband
    Bw19 = 0x1D,
    /// 23.4 kHz double-sideband
    Bw23 = 0x15,
    /// 29.3 kHz double-sideband
    Bw29 = 0x0D,
    /// 39.0 kHz double-sideband
    Bw39 = 0x1C,
    /// 46.9 kHz double-sideband
    Bw46 = 0x14,
    /// 58.6 kHz double-sideband
    Bw58 = 0x0C,
    /// 78.2 kHz double-sideband
    Bw78 = 0x1B,
    /// 93.8 kHz double-sideband
    Bw93 = 0x13,
    /// 117.3 kHz double-sideband
    Bw117 = 0x0B,
    /// 156.2 kHz double-sideband
    Bw156 = 0x1A,
    /// 187.2 kHz double-sideband
    Bw187 = 0x12,
    /// 234.3 kHz double-sideband
    Bw234 = 0x0A,
    /// 312.0 kHz double-sideband
    Bw312 = 0x19,
    /// 373.6 kHz double-sideband
    Bw373 = 0x11,
    /// 467.0 kHz double-sideband
    Bw467 = 0x09,
}

impl FskBandwidth {
    /// Get the bandwidth in hertz.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::FskBandwidth;
    ///
    /// assert_eq!(FskBandwidth::Bw4.hertz(), 4_800);
    /// assert_eq!(FskBandwidth::Bw5.hertz(), 5_800);
    /// assert_eq!(FskBandwidth::Bw7.hertz(), 7_300);
    /// assert_eq!(FskBandwidth::Bw9.hertz(), 9_700);
    /// assert_eq!(FskBandwidth::Bw11.hertz(), 11_700);
    /// assert_eq!(FskBandwidth::Bw14.hertz(), 14_600);
    /// assert_eq!(FskBandwidth::Bw19.hertz(), 19_500);
    /// assert_eq!(FskBandwidth::Bw23.hertz(), 23_400);
    /// assert_eq!(FskBandwidth::Bw29.hertz(), 29_300);
    /// assert_eq!(FskBandwidth::Bw39.hertz(), 39_000);
    /// assert_eq!(FskBandwidth::Bw46.hertz(), 46_900);
    /// assert_eq!(FskBandwidth::Bw58.hertz(), 58_600);
    /// assert_eq!(FskBandwidth::Bw78.hertz(), 78_200);
    /// assert_eq!(FskBandwidth::Bw93.hertz(), 93_800);
    /// assert_eq!(FskBandwidth::Bw117.hertz(), 117_300);
    /// assert_eq!(FskBandwidth::Bw156.hertz(), 156_200);
    /// assert_eq!(FskBandwidth::Bw187.hertz(), 187_200);
    /// assert_eq!(FskBandwidth::Bw234.hertz(), 234_300);
    /// assert_eq!(FskBandwidth::Bw312.hertz(), 312_000);
    /// assert_eq!(FskBandwidth::Bw373.hertz(), 373_600);
    /// assert_eq!(FskBandwidth::Bw467.hertz(), 467_000);
    /// ```
    pub const fn hertz(&self) -> u32 {
        match self {
            FskBandwidth::Bw4 => 4_800,
            FskBandwidth::Bw5 => 5_800,
            FskBandwidth::Bw7 => 7_300,
            FskBandwidth::Bw9 => 9_700,
            FskBandwidth::Bw11 => 11_700,
            FskBandwidth::Bw14 => 14_600,
            FskBandwidth::Bw19 => 19_500,
            FskBandwidth::Bw23 => 23_400,
            FskBandwidth::Bw29 => 29_300,
            FskBandwidth::Bw39 => 39_000,
            FskBandwidth::Bw46 => 46_900,
            FskBandwidth::Bw58 => 58_600,
            FskBandwidth::Bw78 => 78_200,
            FskBandwidth::Bw93 => 93_800,
            FskBandwidth::Bw117 => 117_300,
            FskBandwidth::Bw156 => 156_200,
            FskBandwidth::Bw187 => 187_200,
            FskBandwidth::Bw234 => 234_300,
            FskBandwidth::Bw312 => 312_000,
            FskBandwidth::Bw373 => 373_600,
            FskBandwidth::Bw467 => 467_000,
        }
    }

    /// Convert from a raw bit value.
    ///
    /// Invalid values will be returned in the `Err` variant of the result.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::FskBandwidth;
    ///
    /// assert_eq!(FskBandwidth::from_bits(0x1F), Ok(FskBandwidth::Bw4));
    /// assert_eq!(FskBandwidth::from_bits(0x17), Ok(FskBandwidth::Bw5));
    /// assert_eq!(FskBandwidth::from_bits(0x0F), Ok(FskBandwidth::Bw7));
    /// assert_eq!(FskBandwidth::from_bits(0x1E), Ok(FskBandwidth::Bw9));
    /// assert_eq!(FskBandwidth::from_bits(0x16), Ok(FskBandwidth::Bw11));
    /// assert_eq!(FskBandwidth::from_bits(0x0E), Ok(FskBandwidth::Bw14));
    /// assert_eq!(FskBandwidth::from_bits(0x1D), Ok(FskBandwidth::Bw19));
    /// assert_eq!(FskBandwidth::from_bits(0x15), Ok(FskBandwidth::Bw23));
    /// assert_eq!(FskBandwidth::from_bits(0x0D), Ok(FskBandwidth::Bw29));
    /// assert_eq!(FskBandwidth::from_bits(0x1C), Ok(FskBandwidth::Bw39));
    /// assert_eq!(FskBandwidth::from_bits(0x14), Ok(FskBandwidth::Bw46));
    /// assert_eq!(FskBandwidth::from_bits(0x0C), Ok(FskBandwidth::Bw58));
    /// assert_eq!(FskBandwidth::from_bits(0x1B), Ok(FskBandwidth::Bw78));
    /// assert_eq!(FskBandwidth::from_bits(0x13), Ok(FskBandwidth::Bw93));
    /// assert_eq!(FskBandwidth::from_bits(0x0B), Ok(FskBandwidth::Bw117));
    /// assert_eq!(FskBandwidth::from_bits(0x1A), Ok(FskBandwidth::Bw156));
    /// assert_eq!(FskBandwidth::from_bits(0x12), Ok(FskBandwidth::Bw187));
    /// assert_eq!(FskBandwidth::from_bits(0x0A), Ok(FskBandwidth::Bw234));
    /// assert_eq!(FskBandwidth::from_bits(0x19), Ok(FskBandwidth::Bw312));
    /// assert_eq!(FskBandwidth::from_bits(0x11), Ok(FskBandwidth::Bw373));
    /// assert_eq!(FskBandwidth::from_bits(0x09), Ok(FskBandwidth::Bw467));
    /// assert_eq!(FskBandwidth::from_bits(0x00), Err(0x00));
    /// ```
    pub const fn from_bits(bits: u8) -> Result<Self, u8> {
        match bits {
            0x1F => Ok(Self::Bw4),
            0x17 => Ok(Self::Bw5),
            0x0F => Ok(Self::Bw7),
            0x1E => Ok(Self::Bw9),
            0x16 => Ok(Self::Bw11),
            0x0E => Ok(Self::Bw14),
            0x1D => Ok(Self::Bw19),
            0x15 => Ok(Self::Bw23),
            0x0D => Ok(Self::Bw29),
            0x1C => Ok(Self::Bw39),
            0x14 => Ok(Self::Bw46),
            0x0C => Ok(Self::Bw58),
            0x1B => Ok(Self::Bw78),
            0x13 => Ok(Self::Bw93),
            0x0B => Ok(Self::Bw117),
            0x1A => Ok(Self::Bw156),
            0x12 => Ok(Self::Bw187),
            0x0A => Ok(Self::Bw234),
            0x19 => Ok(Self::Bw312),
            0x11 => Ok(Self::Bw373),
            0x09 => Ok(Self::Bw467),
            x => Err(x),
        }
    }
}

impl Ord for FskBandwidth {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.hertz().cmp(&other.hertz())
    }
}

impl PartialOrd for FskBandwidth {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.hertz().cmp(&other.hertz()))
    }
}

/// Pulse shaping options for [`FskModParams`].
#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum FskPulseShape {
    /// No filtering applied.
    None = 0b00,
    /// Gaussian BT 0.3
    Bt03 = 0x08,
    /// Gaussian BT 0.5
    Bt05 = 0x09,
    /// Gaussian BT 0.7
    Bt07 = 0x0A,
    /// Gaussian BT 1.0
    Bt10 = 0x0B,
}

/// Bitrate argument for [`FskModParams::set_bitrate`] and
/// [`BpskModParams::set_bitrate`].
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct FskBitrate {
    bits: u32,
}

impl FskBitrate {
    /// Create a new `FskBitrate` from a bitrate in bits per second.
    ///
    /// This the resulting value will be rounded down, and will saturate if
    /// `bps` is outside of the theoretical limits.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::FskBitrate;
    ///
    /// const BITRATE: FskBitrate = FskBitrate::from_bps(9600);
    /// assert_eq!(BITRATE.as_bps(), 9600);
    /// ```
    pub const fn from_bps(bps: u32) -> Self {
        const MAX: u32 = 0x00FF_FFFF;
        if bps == 0 {
            Self { bits: MAX }
        } else {
            let bits: u32 = 32 * 32_000_000 / bps;
            if bits > MAX {
                Self { bits: MAX }
            } else {
                Self { bits }
            }
        }
    }

    /// Create a new `FskBitrate` from a raw bit value.
    ///
    /// bits = 32 × 32 MHz / bitrate
    ///
    /// **Note:** Only the first 24 bits of the `u32` are used, the `bits`
    /// argument will be masked.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::FskBitrate;
    ///
    /// const BITRATE: FskBitrate = FskBitrate::from_raw(0x7D00);
    /// assert_eq!(BITRATE.as_bps(), 32_000);
    /// ```
    pub const fn from_raw(bits: u32) -> Self {
        Self {
            bits: bits & 0x00FF_FFFF,
        }
    }

    /// Return the bitrate in bits per second, rounded down.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::FskBitrate;
    ///
    /// const BITS_PER_SEC: u32 = 9600;
    /// const BITRATE: FskBitrate = FskBitrate::from_bps(BITS_PER_SEC);
    /// assert_eq!(BITRATE.as_bps(), BITS_PER_SEC);
    /// ```
    pub const fn as_bps(&self) -> u32 {
        if self.bits == 0 {
            0
        } else {
            32 * 32_000_000 / self.bits
        }
    }

    pub(crate) const fn into_bits(self) -> u32 {
        self.bits
    }
}

impl Ord for FskBitrate {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.as_bps().cmp(&other.as_bps())
    }
}

impl PartialOrd for FskBitrate {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.as_bps().cmp(&other.as_bps()))
    }
}

/// Frequency deviation argument for [`FskModParams::set_fdev`]
#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct FskFdev {
    bits: u32,
}

impl FskFdev {
    /// Create a new `FskFdev` from a frequency deviation in hertz, rounded
    /// down.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::FskFdev;
    ///
    /// const FDEV: FskFdev = FskFdev::from_hertz(31_250);
    /// assert_eq!(FDEV.as_hertz(), 31_250);
    /// ```
    pub const fn from_hertz(hz: u32) -> Self {
        Self {
            bits: ((hz as u64) * (1 << 25) / 32_000_000) as u32 & 0x00FF_FFFF,
        }
    }

    /// Create a new `FskFdev` from a raw bit value.
    ///
    /// bits = fdev × 2<sup>25</sup> / 32 MHz
    ///
    /// **Note:** Only the first 24 bits of the `u32` are used, the `bits`
    /// argument will be masked.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::FskFdev;
    ///
    /// const FDEV: FskFdev = FskFdev::from_raw(0x8000);
    /// assert_eq!(FDEV.as_hertz(), 31_250);
    /// ```
    pub const fn from_raw(bits: u32) -> Self {
        Self {
            bits: bits & 0x00FF_FFFF,
        }
    }

    /// Return the frequency deviation in hertz, rounded down.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::FskFdev;
    ///
    /// const HERTZ: u32 = 31_250;
    /// const FDEV: FskFdev = FskFdev::from_hertz(HERTZ);
    /// assert_eq!(FDEV.as_hertz(), HERTZ);
    /// ```
    pub const fn as_hertz(&self) -> u32 {
        ((self.bits as u64) * 32_000_000 / (1 << 25)) as u32
    }

    pub(crate) const fn into_bits(self) -> u32 {
        self.bits
    }
}

/// (G)FSK modulation parameters.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct FskModParams {
    buf: [u8; 9],
}

impl FskModParams {
    /// Create a new `FskModParams` struct.
    ///
    /// This is the same as `default`, but in a `const` function.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::FskModParams;
    ///
    /// const MOD_PARAMS: FskModParams = FskModParams::new();
    /// ```
    pub const fn new() -> FskModParams {
        FskModParams {
            buf: [
                super::OpCode::SetModulationParams as u8,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
            ],
        }
        .set_bitrate(FskBitrate::from_bps(50_000))
        .set_pulse_shape(FskPulseShape::None)
        .set_bandwidth(FskBandwidth::Bw58)
        .set_fdev(FskFdev::from_hertz(25_000))
    }

    /// Get the bitrate.
    ///
    /// # Example
    ///
    /// Setting the bitrate to 32,000 bits per second.
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::{FskBitrate, FskModParams};
    ///
    /// const BITRATE: FskBitrate = FskBitrate::from_bps(32_000);
    /// const MOD_PARAMS: FskModParams = FskModParams::new().set_bitrate(BITRATE);
    /// assert_eq!(MOD_PARAMS.bitrate(), BITRATE);
    /// ```
    pub const fn bitrate(&self) -> FskBitrate {
        let raw: u32 = u32::from_be_bytes([0, self.buf[1], self.buf[2], self.buf[3]]);
        FskBitrate::from_raw(raw)
    }

    /// Set the bitrate.
    ///
    /// # Example
    ///
    /// Setting the bitrate to 32,000 bits per second.
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::{FskBitrate, FskModParams};
    ///
    /// const BITRATE: FskBitrate = FskBitrate::from_bps(32_000);
    /// const MOD_PARAMS: FskModParams = FskModParams::new().set_bitrate(BITRATE);
    /// # assert_eq!(MOD_PARAMS.as_slice()[1], 0x00);
    /// # assert_eq!(MOD_PARAMS.as_slice()[2], 0x7D);
    /// # assert_eq!(MOD_PARAMS.as_slice()[3], 0x00);
    /// ```
    #[must_use = "set_bitrate returns a modified FskModParams"]
    pub const fn set_bitrate(mut self, bitrate: FskBitrate) -> FskModParams {
        let bits: u32 = bitrate.into_bits();
        self.buf[1] = ((bits >> 16) & 0xFF) as u8;
        self.buf[2] = ((bits >> 8) & 0xFF) as u8;
        self.buf[3] = (bits & 0xFF) as u8;
        self
    }

    /// Set the pulse shaping.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::{FskModParams, FskPulseShape};
    ///
    /// const MOD_PARAMS: FskModParams = FskModParams::new().set_pulse_shape(FskPulseShape::Bt03);
    /// # assert_eq!(MOD_PARAMS.as_slice()[4], 0x08);
    /// ```
    #[must_use = "set_pulse_shape returns a modified FskModParams"]
    pub const fn set_pulse_shape(mut self, shape: FskPulseShape) -> FskModParams {
        self.buf[4] = shape as u8;
        self
    }

    /// Get the bandwidth.
    ///
    /// Values that do not correspond to a valid [`FskBandwidth`] will be
    /// returned in the `Err` variant of the result.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::{FskBandwidth, FskModParams};
    ///
    /// const MOD_PARAMS: FskModParams = FskModParams::new().set_bandwidth(FskBandwidth::Bw9);
    /// assert_eq!(MOD_PARAMS.bandwidth(), Ok(FskBandwidth::Bw9));
    /// ```
    pub const fn bandwidth(&self) -> Result<FskBandwidth, u8> {
        FskBandwidth::from_bits(self.buf[5])
    }

    /// Set the bandwidth.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::{FskBandwidth, FskModParams};
    ///
    /// const MOD_PARAMS: FskModParams = FskModParams::new().set_bandwidth(FskBandwidth::Bw9);
    /// # assert_eq!(MOD_PARAMS.as_slice()[5], 0x1E);
    /// ```
    #[must_use = "set_pulse_shape returns a modified FskModParams"]
    pub const fn set_bandwidth(mut self, bw: FskBandwidth) -> FskModParams {
        self.buf[5] = bw as u8;
        self
    }

    /// Get the frequency deviation.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::{FskFdev, FskModParams};
    ///
    /// const FDEV: FskFdev = FskFdev::from_hertz(31_250);
    /// const MOD_PARAMS: FskModParams = FskModParams::new().set_fdev(FDEV);
    /// assert_eq!(MOD_PARAMS.fdev(), FDEV);
    /// ```
    pub const fn fdev(&self) -> FskFdev {
        let raw: u32 = u32::from_be_bytes([0, self.buf[6], self.buf[7], self.buf[8]]);
        FskFdev::from_raw(raw)
    }

    /// Set the frequency deviation.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::{FskFdev, FskModParams};
    ///
    /// const FDEV: FskFdev = FskFdev::from_hertz(31_250);
    /// const MOD_PARAMS: FskModParams = FskModParams::new().set_fdev(FDEV);
    /// # assert_eq!(MOD_PARAMS.as_slice()[6], 0x00);
    /// # assert_eq!(MOD_PARAMS.as_slice()[7], 0x80);
    /// # assert_eq!(MOD_PARAMS.as_slice()[8], 0x00);
    /// ```
    #[must_use = "set_fdev returns a modified FskModParams"]
    pub const fn set_fdev(mut self, fdev: FskFdev) -> FskModParams {
        let bits: u32 = fdev.into_bits();
        self.buf[6] = ((bits >> 16) & 0xFF) as u8;
        self.buf[7] = ((bits >> 8) & 0xFF) as u8;
        self.buf[8] = (bits & 0xFF) as u8;
        self
    }
    /// Returns `true` if the modulation parameters are valid.
    ///
    /// The bandwidth must be chosen so that:
    ///
    /// [`FskBandwidth`] > [`FskBitrate`] + 2 × [`FskFdev`] + frequency error
    ///
    /// Where frequency error = 2 × HSE32<sub>FREQ</sub> error.
    ///
    /// The datasheet (DS13293 Rev 1) gives these requirements for the HSE32
    /// frequency tolerance:
    ///
    /// * Initial: ±10 ppm
    /// * Over temperature (-20 to 70 °C): ±10 ppm
    /// * Aging over 10 years: ±10 ppm
    ///
    /// # Example
    ///
    /// Checking valid parameters at compile-time
    ///
    /// ```
    /// extern crate static_assertions as sa;
    /// use stm32wlxx_hal::subghz::{FskBandwidth, FskBitrate, FskFdev, FskModParams, FskPulseShape};
    ///
    /// const MOD_PARAMS: FskModParams = FskModParams::new()
    ///     .set_bitrate(FskBitrate::from_bps(20_000))
    ///     .set_pulse_shape(FskPulseShape::Bt03)
    ///     .set_bandwidth(FskBandwidth::Bw58)
    ///     .set_fdev(FskFdev::from_hertz(10_000));
    ///
    /// // 30 PPM is wost case (if the HSE32 crystal meets requirements)
    /// sa::const_assert!(MOD_PARAMS.is_valid(30));
    /// ```
    #[must_use = "the return value indicates if the modulation parameters are valid"]
    pub const fn is_valid(&self, ppm: u8) -> bool {
        let bw: u32 = match self.bandwidth() {
            Ok(bw) => bw.hertz(),
            Err(_) => return false,
        };
        let br: u32 = self.bitrate().as_bps();
        let fdev: u32 = self.fdev().as_hertz();
        let hse_err: u32 = 32 * (ppm as u32);
        let freq_err: u32 = 2 * hse_err;

        bw > br + 2 * fdev + freq_err
    }

    /// Returns `true` if the modulation parameters are valid for a worst-case
    /// crystal tolerance.
    ///
    /// This is equivalent to [`is_valid`](Self::is_valid) with a `ppm` argument
    /// of 30.
    #[must_use = "the return value indicates if the modulation parameters are valid"]
    pub const fn is_valid_worst_case(&self) -> bool {
        self.is_valid(30)
    }

    /// Extracts a slice containing the packet.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::{FskBandwidth, FskBitrate, FskFdev, FskModParams, FskPulseShape};
    ///
    /// const BITRATE: FskBitrate = FskBitrate::from_bps(20_000);
    /// const PULSE_SHAPE: FskPulseShape = FskPulseShape::Bt03;
    /// const BW: FskBandwidth = FskBandwidth::Bw58;
    /// const FDEV: FskFdev = FskFdev::from_hertz(10_000);
    ///
    /// const MOD_PARAMS: FskModParams = FskModParams::new()
    ///     .set_bitrate(BITRATE)
    ///     .set_pulse_shape(PULSE_SHAPE)
    ///     .set_bandwidth(BW)
    ///     .set_fdev(FDEV);
    ///
    /// assert_eq!(
    ///     MOD_PARAMS.as_slice(),
    ///     &[0x8B, 0x00, 0xC8, 0x00, 0x08, 0x0C, 0x00, 0x28, 0xF5]
    /// );
    /// ```
    pub const fn as_slice(&self) -> &[u8] {
        &self.buf
    }
}

impl Default for FskModParams {
    fn default() -> Self {
        Self::new()
    }
}

/// LoRa spreading factor.
///
/// Argument of [`LoRaModParams::set_sf`].
///
/// Higher spreading factors improve receiver sensitivity, but reduce bit rate
/// and increase power consumption.
#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum SpreadingFactor {
    /// Spreading factor 5.
    Sf5 = 0x05,
    /// Spreading factor 6.
    Sf6 = 0x06,
    /// Spreading factor 7.
    Sf7 = 0x07,
    /// Spreading factor 8.
    Sf8 = 0x08,
    /// Spreading factor 9.
    Sf9 = 0x09,
    /// Spreading factor 10.
    Sf10 = 0x0A,
    /// Spreading factor 11.
    Sf11 = 0x0B,
    /// Spreading factor 12.
    Sf12 = 0x0C,
}

impl From<SpreadingFactor> for u8 {
    fn from(sf: SpreadingFactor) -> Self {
        sf as u8
    }
}

/// LoRa bandwidth.
///
/// Argument of [`LoRaModParams::set_bw`].
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum LoRaBandwidth {
    /// 7.81 kHz
    Bw7 = 0x00,
    /// 10.42 kHz
    Bw10 = 0x08,
    /// 15.63 kHz
    Bw15 = 0x01,
    /// 20.83 kHz
    Bw20 = 0x09,
    /// 31.25 kHz
    Bw31 = 0x02,
    /// 41.67 kHz
    Bw41 = 0x0A,
    /// 62.50 kHz
    Bw62 = 0x03,
    /// 125 kHz
    Bw125 = 0x04,
    /// 250 kHz
    Bw250 = 0x05,
    /// 500 kHz
    Bw500 = 0x06,
}

impl LoRaBandwidth {
    /// Get the bandwidth in hertz.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::LoRaBandwidth;
    ///
    /// assert_eq!(LoRaBandwidth::Bw7.hertz(), 7_810);
    /// assert_eq!(LoRaBandwidth::Bw10.hertz(), 10_420);
    /// assert_eq!(LoRaBandwidth::Bw15.hertz(), 15_630);
    /// assert_eq!(LoRaBandwidth::Bw20.hertz(), 20_830);
    /// assert_eq!(LoRaBandwidth::Bw31.hertz(), 31_250);
    /// assert_eq!(LoRaBandwidth::Bw41.hertz(), 41_670);
    /// assert_eq!(LoRaBandwidth::Bw62.hertz(), 62_500);
    /// assert_eq!(LoRaBandwidth::Bw125.hertz(), 125_000);
    /// assert_eq!(LoRaBandwidth::Bw250.hertz(), 250_000);
    /// assert_eq!(LoRaBandwidth::Bw500.hertz(), 500_000);
    /// ```
    pub const fn hertz(&self) -> u32 {
        match self {
            LoRaBandwidth::Bw7 => 7_810,
            LoRaBandwidth::Bw10 => 10_420,
            LoRaBandwidth::Bw15 => 15_630,
            LoRaBandwidth::Bw20 => 20_830,
            LoRaBandwidth::Bw31 => 31_250,
            LoRaBandwidth::Bw41 => 41_670,
            LoRaBandwidth::Bw62 => 62_500,
            LoRaBandwidth::Bw125 => 125_000,
            LoRaBandwidth::Bw250 => 250_000,
            LoRaBandwidth::Bw500 => 500_000,
        }
    }
}

impl Ord for LoRaBandwidth {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.hertz().cmp(&other.hertz())
    }
}

impl PartialOrd for LoRaBandwidth {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.hertz().cmp(&other.hertz()))
    }
}

/// LoRa forward error correction coding rate.
///
/// Argument of [`LoRaModParams::set_cr`].
///
/// A higher coding rate provides better immunity to interference at the expense
/// of longer transmission time.
/// In normal conditions [`CodingRate::Cr45`] provides the best trade off.
/// In case of strong interference, a higher coding rate may be used.
#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum CodingRate {
    /// No forward error correction coding rate 4/4
    ///
    /// Overhead ratio of 1
    Cr44 = 0x00,
    /// Forward error correction coding rate 4/5
    ///
    /// Overhead ratio of 1.25
    Cr45 = 0x1,
    /// Forward error correction coding rate 4/6
    ///
    /// Overhead ratio of 1.5
    Cr46 = 0x2,
    /// Forward error correction coding rate 4/7
    ///
    /// Overhead ratio of 1.75
    Cr47 = 0x3,
    /// Forward error correction coding rate 4/8
    ///
    /// Overhead ratio of 2
    Cr48 = 0x4,
}

/// LoRa modulation parameters.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]

pub struct LoRaModParams {
    buf: [u8; 5],
}

impl LoRaModParams {
    /// Create a new `LoRaModParams` struct.
    ///
    /// This is the same as `default`, but in a `const` function.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::LoRaModParams;
    ///
    /// const MOD_PARAMS: LoRaModParams = LoRaModParams::new();
    /// assert_eq!(MOD_PARAMS, LoRaModParams::default());
    /// ```
    pub const fn new() -> LoRaModParams {
        LoRaModParams {
            buf: [
                super::OpCode::SetModulationParams as u8,
                0x05, // valid spreading factor
                0x00,
                0x00,
                0x00,
            ],
        }
    }

    /// Set the spreading factor.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::{LoRaModParams, SpreadingFactor};
    ///
    /// const MOD_PARAMS: LoRaModParams = LoRaModParams::new().set_sf(SpreadingFactor::Sf7);
    /// # assert_eq!(MOD_PARAMS.as_slice(), &[0x8B, 0x07, 0x00, 0x00, 0x00]);
    /// ```
    #[must_use = "set_sf returns a modified LoRaModParams"]
    pub const fn set_sf(mut self, sf: SpreadingFactor) -> Self {
        self.buf[1] = sf as u8;
        self
    }

    /// Set the bandwidth.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::{LoRaBandwidth, LoRaModParams};
    ///
    /// const MOD_PARAMS: LoRaModParams = LoRaModParams::new().set_bw(LoRaBandwidth::Bw125);
    /// # assert_eq!(MOD_PARAMS.as_slice(), &[0x8B, 0x05, 0x04, 0x00, 0x00]);
    /// ```
    #[must_use = "set_bw returns a modified LoRaModParams"]
    pub const fn set_bw(mut self, bw: LoRaBandwidth) -> Self {
        self.buf[2] = bw as u8;
        self
    }

    /// Set the forward error correction coding rate.
    ///
    /// See [`CodingRate`] for more information.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::{CodingRate, LoRaModParams};
    ///
    /// const MOD_PARAMS: LoRaModParams = LoRaModParams::new().set_cr(CodingRate::Cr45);
    /// # assert_eq!(MOD_PARAMS.as_slice(), &[0x8B, 0x05, 0x00, 0x01, 0x00]);
    /// ```
    #[must_use = "set_cr returns a modified LoRaModParams"]
    pub const fn set_cr(mut self, cr: CodingRate) -> Self {
        self.buf[3] = cr as u8;
        self
    }

    /// Set low data rate optimization enable.
    ///
    /// For low data rates (typically high SF or low BW) and very long payloads
    /// (may last several seconds), the low data rate optimization (LDRO) can be
    /// enabled.
    /// This reduces the number of bits per symbol to the given SF minus 2,
    /// to allow the receiver to have a better tracking of the LoRa receive
    /// signal.
    /// Depending on the payload length, the low data rate optimization is
    /// usually recommended when the LoRa symbol time is equal or above
    /// 16.38 ms.
    /// When using LoRa modulation, the total frequency drift over the packet
    /// time must be kept lower than Freq_drift_max:
    ///
    /// Freq_drift_max = BW / (3 × 2SF)
    ///
    /// When possible, enabling the low data rate optimization, relaxes the
    /// total frequency drift over the packet time by 16:
    ///
    /// Freq_drift_optimise_max = 16 × Freq_drift_max
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::LoRaModParams;
    ///
    /// const MOD_PARAMS: LoRaModParams = LoRaModParams::new().set_ldro_en(true);
    /// # assert_eq!(MOD_PARAMS.as_slice(), &[0x8B, 0x05, 0x00, 0x00, 0x01]);
    /// ```
    #[must_use = "set_ldro_en returns a modified LoRaModParams"]
    pub const fn set_ldro_en(mut self, en: bool) -> Self {
        self.buf[4] = en as u8;
        self
    }

    /// Extracts a slice containing the packet.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::{CodingRate, LoRaBandwidth, LoRaModParams, SpreadingFactor};
    ///
    /// const MOD_PARAMS: LoRaModParams = LoRaModParams::new()
    ///     .set_sf(SpreadingFactor::Sf7)
    ///     .set_bw(LoRaBandwidth::Bw125)
    ///     .set_cr(CodingRate::Cr45)
    ///     .set_ldro_en(false);
    ///
    /// assert_eq!(MOD_PARAMS.as_slice(), &[0x8B, 0x07, 0x04, 0x01, 0x00]);
    /// ```
    pub const fn as_slice(&self) -> &[u8] {
        &self.buf
    }
}

impl Default for LoRaModParams {
    fn default() -> Self {
        Self::new()
    }
}

/// BPSK modulation parameters.
///
/// **Note:** There is no method to set the pulse shape because there is only
/// one valid pulse shape (Gaussian BT 0.5).
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct BpskModParams {
    buf: [u8; 5],
}

impl BpskModParams {
    /// Create a new `BpskModParams` struct.
    ///
    /// This is the same as `default`, but in a `const` function.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::BpskModParams;
    ///
    /// const MOD_PARAMS: BpskModParams = BpskModParams::new();
    /// assert_eq!(MOD_PARAMS, BpskModParams::default());
    /// ```
    pub const fn new() -> BpskModParams {
        const OPCODE: u8 = super::OpCode::SetModulationParams as u8;
        BpskModParams {
            buf: [OPCODE, 0x1A, 0x0A, 0xAA, 0x16],
        }
    }

    /// Set the bitrate.
    ///
    /// # Example
    ///
    /// Setting the bitrate to 600 bits per second.
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::{BpskModParams, FskBitrate};
    ///
    /// const BITRATE: FskBitrate = FskBitrate::from_bps(600);
    /// const MOD_PARAMS: BpskModParams = BpskModParams::new().set_bitrate(BITRATE);
    /// # assert_eq!(MOD_PARAMS.as_slice()[1], 0x1A);
    /// # assert_eq!(MOD_PARAMS.as_slice()[2], 0x0A);
    /// # assert_eq!(MOD_PARAMS.as_slice()[3], 0xAA);
    /// ```
    #[must_use = "set_bitrate returns a modified BpskModParams"]
    pub const fn set_bitrate(mut self, bitrate: FskBitrate) -> BpskModParams {
        let bits: u32 = bitrate.into_bits();
        self.buf[1] = ((bits >> 16) & 0xFF) as u8;
        self.buf[2] = ((bits >> 8) & 0xFF) as u8;
        self.buf[3] = (bits & 0xFF) as u8;
        self
    }

    /// Extracts a slice containing the packet.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::{BpskModParams, FskBitrate};
    ///
    /// const BITRATE: FskBitrate = FskBitrate::from_bps(100);
    /// const MOD_PARAMS: BpskModParams = BpskModParams::new().set_bitrate(BITRATE);
    /// assert_eq!(MOD_PARAMS.as_slice(), [0x8B, 0x9C, 0x40, 0x00, 0x16]);
    /// ```
    pub const fn as_slice(&self) -> &[u8] {
        &self.buf
    }
}

impl Default for BpskModParams {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use super::{FskBandwidth, FskBitrate, FskFdev, LoRaBandwidth};

    #[test]
    fn fsk_bw_ord() {
        assert!((FskBandwidth::Bw4 as u8) > (FskBandwidth::Bw5 as u8));
        assert!(FskBandwidth::Bw4 < FskBandwidth::Bw5);
        assert!(FskBandwidth::Bw5 > FskBandwidth::Bw4);
    }

    #[test]
    fn lora_bw_ord() {
        assert!((LoRaBandwidth::Bw10 as u8) > (LoRaBandwidth::Bw15 as u8));
        assert!(LoRaBandwidth::Bw10 < LoRaBandwidth::Bw15);
        assert!(LoRaBandwidth::Bw15 > LoRaBandwidth::Bw10);
    }

    #[test]
    fn fsk_bitrate_ord() {
        assert!(FskBitrate::from_bps(9600) > FskBitrate::from_bps(4800));
        assert!(FskBitrate::from_bps(4800) < FskBitrate::from_bps(9600));
    }

    #[test]
    fn fsk_bitrate_as_bps_limits() {
        const ZERO: FskBitrate = FskBitrate::from_raw(0);
        const ONE: FskBitrate = FskBitrate::from_raw(1);
        const MAX: FskBitrate = FskBitrate::from_raw(u32::MAX);

        assert_eq!(ZERO.as_bps(), 0);
        assert_eq!(ONE.as_bps(), 1_024_000_000);
        assert_eq!(MAX.as_bps(), 61);
    }

    #[test]
    fn fsk_bitrate_from_bps_limits() {
        const ZERO: FskBitrate = FskBitrate::from_bps(0);
        const ONE: FskBitrate = FskBitrate::from_bps(1);
        const MAX: FskBitrate = FskBitrate::from_bps(u32::MAX);

        assert_eq!(ZERO.as_bps(), 61);
        assert_eq!(ONE.as_bps(), 61);
        assert_eq!(MAX.as_bps(), 0);
    }

    #[test]
    fn fsk_fdev_ord() {
        assert!(FskFdev::from_hertz(30_000) > FskFdev::from_hertz(20_000));
        assert!(FskFdev::from_hertz(20_000) < FskFdev::from_hertz(30_000));
    }

    #[test]
    fn fsk_fdev_as_hertz_limits() {
        const ZERO: FskFdev = FskFdev::from_raw(0);
        const ONE: FskFdev = FskFdev::from_raw(1);
        const MAX: FskFdev = FskFdev::from_raw(u32::MAX);

        assert_eq!(ZERO.as_hertz(), 0);
        assert_eq!(ONE.as_hertz(), 0);
        assert_eq!(MAX.as_hertz(), 15_999_999);
    }

    #[test]
    fn fsk_fdev_from_hertz_limits() {
        const ZERO: FskFdev = FskFdev::from_hertz(0);
        const ONE: FskFdev = FskFdev::from_hertz(1);
        const MAX: FskFdev = FskFdev::from_hertz(u32::MAX);

        assert_eq!(ZERO.as_hertz(), 0);
        assert_eq!(ONE.as_hertz(), 0);
        assert_eq!(MAX.as_hertz(), 6_967_294);
    }
}
