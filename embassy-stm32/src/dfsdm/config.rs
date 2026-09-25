//! Config value types used across the driver: filter parameters, data packing
//! and shift modes, clock/output sources, and trigger/edge configuration.

use super::{Error, Instance};
use crate::time::Hertz;

// =============================================================================
// Config types
// =============================================================================

/// Source clock for the CKOUT output.
#[derive(Copy, Clone)]
pub enum CkoutSource {
    /// Source for output clock is from system clock
    System,
    /// Source for output clock is from audio clock
    Audio,
}

impl From<CkoutSource> for bool {
    fn from(value: CkoutSource) -> Self {
        match value {
            CkoutSource::System => false,
            CkoutSource::Audio => true,
        }
    }
}

/// CKOUT divider register value.
///
/// 0 = CKOUT stopped; 1..=255 = enabled (actual divider = value + 1, range 2..=256).
///
/// The CKOUT output runs at 0 to 20 MHz. Stop CKOUT (divider 0) and wait for
/// the clock to settle before changing the CKOUT source, or the output can
/// glitch. Stopping takes 4 system-clock cycles for the system source, or
/// 1 system clock plus 3 audio-clock cycles for the audio source.
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct CkoutDivider(u8);

/// Maximum Manchester recovered-clock frequency (RM0455: 0 - 10 MHz).
const MANCHESTER_MAX_RATE: u32 = 10_000_000;

impl CkoutDivider {
    /// Create from the actual divider value (2..=256).
    /// Panics if out of range.
    /// For a non-panicking variant, use [`CkoutDivider::try_from`].
    pub fn new(divider: u16) -> Self {
        assert!((2..=256).contains(&divider), "CKOUT divider must be 2..=256");
        Self((divider - 1) as u8)
    }

    /// Compute the divider to get a wanted CKOUT output frequency from the
    /// CKOUT source clock.
    ///
    /// `source` is the CKOUT input clock (selected by [`CkoutSource`]);
    /// `ckout_rate` is the wanted CKOUT frequency. The divider is rounded
    /// up so the actual CKOUT frequency never exceeds `ckout_rate`.
    ///
    /// The DFSDM kernel clock (`crate::rcc::frequency::<T>()`) must be at
    /// least 4x `ckout_rate` (SPI coding, RM0455); this is enforced here.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidConfig`] if the wanted frequency is zero, too
    /// high relative to the kernel clock, or unreachable with the 2..=256
    /// divider range.
    ///
    /// # Panics
    ///
    /// Panics if the clock is not active.
    pub fn from_frequency<T: Instance>(source: Hertz, ckout_rate: Hertz) -> Result<Self, Error> {
        if ckout_rate.0 == 0 {
            return Err(Error::InvalidConfig);
        }
        // Kernel clock (fDFSDMCLK) must be at least 4x the CKOUT rate (SPI).
        if ckout_rate.0.saturating_mul(4) > crate::rcc::frequency::<T>().0 {
            return Err(Error::InvalidConfig);
        }
        // Ceiling divide: actual = source / divider <= ckout_rate.
        let divider = source.0.div_ceil(ckout_rate.0);
        if (2..=256).contains(&divider) {
            Ok(Self((divider - 1) as u8))
        } else {
            Err(Error::InvalidConfig)
        }
    }

    /// Compute the divider for a Manchester data rate.
    ///
    /// `manchester_rate` is the expected Manchester data rate, i.e. the
    /// recovered clock frequency (RM0455: 0 to 10 MHz and < fDFSDMCLK/6). The
    /// divider is chosen so the Manchester period satisfies
    /// `(CKOUTDIV + 1) x T_source < T_manchester < 2 x CKOUTDIV x T_source`
    /// (RM0455), which puts the CKOUT output at ~1.5x the Manchester rate,
    /// inside the valid window.
    ///
    /// The DFSDM kernel clock (`crate::rcc::frequency::<T>()`) must be at
    /// least 6x the Manchester rate; this is enforced here.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidConfig`] if the rate is above 10 MHz, above
    /// fDFSDMCLK/6, or unreachable with the 2..=256 divider range.
    ///
    /// # Panics
    ///
    /// Panics if the clock is not active.
    pub fn for_manchester<T: Instance>(source: Hertz, manchester_rate: Hertz) -> Result<Self, Error> {
        if manchester_rate.0 > MANCHESTER_MAX_RATE {
            return Err(Error::InvalidConfig);
        }
        // Kernel clock (fDFSDMCLK) must be at least 6x the Manchester rate.
        if manchester_rate.0.saturating_mul(6) > crate::rcc::frequency::<T>().0 {
            return Err(Error::InvalidConfig);
        }
        // CKOUT at ~1.5x the Manchester rate: comfortably inside the valid
        // window (f_ckout/2 < f_man < f_ckout).
        let ckout = manchester_rate.0 * 3 / 2;
        Self::from_frequency::<T>(source, Hertz(ckout))
    }

    /// CKOUT disabled (register value 0).
    pub(crate) const DISABLED: Self = Self(0);

    /// Raw register value.
    pub const fn raw(self) -> u8 {
        self.0
    }
}

impl TryFrom<u16> for CkoutDivider {
    type Error = ();

    /// Try to create from the actual divider value (2..=256).
    fn try_from(divider: u16) -> Result<Self, Self::Error> {
        if (2..=256).contains(&divider) {
            Ok(Self((divider - 1) as u8))
        } else {
            Err(())
        }
    }
}

impl From<CkoutDivider> for u8 {
    fn from(value: CkoutDivider) -> Self {
        value.0
    }
}

/// AWFOSR register value.
///
/// 0 = AW filter disabled; 1..=31 = enabled (actual OSR = value + 1, range 2..=32).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct AwdFilterOsr(u8);

impl AwdFilterOsr {
    /// Create from the actual OSR value (2..=32).
    /// Panics if out of range.
    /// For a non-panicking variant, use [`AwdFilterOsr::try_from`].
    pub fn new(divider: u16) -> Self {
        assert!((2..=32).contains(&divider), "OSR must be 2..=32");
        Self((divider - 1) as u8)
    }

    /// Watchdog filter bypassed (register value 0).
    pub(crate) const BYPASSED: Self = Self(0);

    /// Raw register value.
    pub const fn raw(self) -> u8 {
        self.0
    }
}

impl TryFrom<u16> for AwdFilterOsr {
    type Error = ();

    /// Try to create from the actual OSR value (2..=32).
    fn try_from(divider: u16) -> Result<Self, Self::Error> {
        if (2..=32).contains(&divider) {
            Ok(Self((divider - 1) as u8))
        } else {
            Err(())
        }
    }
}

impl From<AwdFilterOsr> for u8 {
    fn from(value: AwdFilterOsr) -> Self {
        value.0
    }
}

/// Data packing mode in CHyDATINR register
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum DataPackingMode {
    /// input data in DFSDM_CHyDATINR register are stored only in `INDAT0[15:0]`. To empty
    /// DFSDM_CHyDATINR register one sample must be read by the DFSDM filter from channel y
    Standard = 0,
    /// Interleaved: input data in DFSDM_CHyDATINR register are stored as two samples:
    /// -first sample in `INDAT0[15:0]` (assigned to channel y)
    /// -second sample `INDAT1[15:0]` (assigned to channel y)
    /// To empty DFSDM_CHyDATINR register, two samples must be read by the digital filter from
    /// channel y (`INDAT0[15:0]` part is read as first sample and then `INDAT1[15:0]` part is read as next
    /// sample).
    Interleaved = 1,
    /// Dual: input data in DFSDM_CHyDATINR register are stored as two samples:
    /// -first sample `INDAT0[15:0]` (assigned to channel y)
    /// -second sample `INDAT1[15:0]` (assigned to channel y+1)
    /// To empty DFSDM_CHyDATINR register first sample must be read by the digital filter from channel
    /// y and second sample must be read by another digital filter from channel y+1. Dual mode is
    /// available only on even channel numbers (y = 0, 2, 4, 6), for odd channel numbers (y = 1, 3, 5, 7)
    /// DFSDM_CHyDATINR is write protected. If an even channel is set to dual mode then the following
    /// odd channel must be set into standard mode (`DATPACK[1:0]=0`) for correct cooperation with even channel.
    Dual = 2,
    // 3 = Reserved
}

/// Input pin selection for a transceiver.
#[derive(Copy, Clone)]
pub enum ChannelInput {
    /// Inputs are taken from this transceiver's own pins.
    Same,
    /// Inputs are taken from the next transceiver's pins (modulo 8).
    Neighbor,
}

impl From<ChannelInput> for bool {
    fn from(value: ChannelInput) -> Self {
        match value {
            ChannelInput::Same => false,
            ChannelInput::Neighbor => true,
        }
    }
}

/// Selects where a transceiver's parallel input data comes from.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum InputDataMux {
    /// Data comes from serial inputs
    ExternalSerial = 0,
    /// Data comes from ADC
    InternalAdc = 1,
    /// Data comes from CPU/DMA writes to the CHyDATINR register.
    InternalRegisterWrite = 2,
    // 3 = Reserved
}

/// Serial-interface clock source.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum SpiClockSelect {
    /// Clock coming from external CKIN pin of the transceiver
    /// sampling point according to [`SerialInterfaceType`]
    ExternalCkin = 0,
    /// Clock coming from the internal CKOUT output
    /// sampling point according to [`SerialInterfaceType`]
    InternalCkout = 1,
    /// Clock coming from the internal CKOUT output
    /// sampling point on each second CKOUT falling edge
    InternalCkoutFallingHalved = 2,
    /// Clock coming from the internal CKOUT output
    /// sampling point on each second CKOUT rising edge
    InternalCkoutRisingHalved = 3,
}

/// Serial interface type and sampling edge.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum SerialInterfaceType {
    /// SPI mode sampling on the rising edge of the clock
    SpiRisingEdge = 0,
    /// SPI mode sampling on the falling edge of the clock
    SpiFallingEdge = 1,
    /// Manchester coded: rising edge = logic 0, falling edge = logic 1
    ManchesterRising0 = 2,
    /// Manchester coded: rising edge = logic 1, falling edge = logic 0
    ManchesterRising1 = 3,
}

/// Filter order of the analog watchdog's fast filter.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum AwdFilterOrder {
    /// FastSinc Filter
    FastSinc = 0,
    /// Sinc1 filter
    Sinc1 = 1,
    /// Sinc2 filter
    Sinc2 = 2,
    /// Sinc3 filter
    Sinc3 = 3,
}

bitflags::bitflags! {
    /// Maps break-signal connections per source transceiver.
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct BreakSignals: u8 {
        ///BREAK0 signal connected
        const BREAK0 = 0b0001;
        ///BREAK1 signal connected
        const BREAK1 = 0b0010;
        ///BREAK2 signal connected
        const BREAK2 = 0b0100;
        ///BREAK3 signal connected
        const BREAK3 = 0b1000;
    }
}

/// Data right bit-shift for transceiver results (CFGR2.DTRBS).
///
/// 0..=31 bits, applied before offset correction; 0 = no shift.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct DataRightShift(u8);

impl DataRightShift {
    /// Create from the shift amount in bits (0..=31). Panics if out of range.
    /// For a non-panicking variant, use [`DataRightShift::try_from`].
    pub fn new(shift: u8) -> Self {
        assert!(shift <= 31, "data right shift must be 0..=31");
        Self(shift)
    }

    /// No shift (register value 0).
    pub const NONE: Self = Self(0);

    /// Raw register value.
    pub const fn raw(self) -> u8 {
        self.0
    }
}

impl TryFrom<u8> for DataRightShift {
    type Error = ();
    fn try_from(shift: u8) -> Result<Self, Self::Error> {
        if shift <= 31 { Ok(Self(shift)) } else { Err(()) }
    }
}

impl From<DataRightShift> for u8 {
    fn from(value: DataRightShift) -> Self {
        value.0
    }
}
/// Pulses to skip in the delay block (DLYR.PLSSKP).
///
/// 0..=63 serial samples skipped immediately after the write.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct PulsesToSkip(u8);

impl PulsesToSkip {
    /// Create from the number of samples to skip (0..=63). Panics if out of range.
    /// For a non-panicking variant, use [`PulsesToSkip::try_from`].
    pub fn new(pulses: u8) -> Self {
        assert!(pulses <= 63, "pulses to skip must be 0..=63");
        Self(pulses)
    }

    /// Raw register value.
    pub const fn raw(self) -> u8 {
        self.0
    }
}

impl TryFrom<u8> for PulsesToSkip {
    type Error = ();
    fn try_from(pulses: u8) -> Result<Self, Self::Error> {
        if pulses <= 63 { Ok(Self(pulses)) } else { Err(()) }
    }
}

impl From<PulsesToSkip> for u8 {
    fn from(value: PulsesToSkip) -> Self {
        value.0
    }
}

// =============================================================================
// Types specifically used for pub config
// =============================================================================

/// SPI edge mode
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SpiMode {
    /// Sampling on the rising edge of the clock
    SpiRisingEdge,
    /// Sampling on the falling edge of the clock
    SpiFallingEdge,
}

impl From<SpiMode> for SerialInterfaceType {
    fn from(value: SpiMode) -> Self {
        match value {
            SpiMode::SpiRisingEdge => Self::SpiRisingEdge,
            SpiMode::SpiFallingEdge => Self::SpiFallingEdge,
        }
    }
}

/// Manchester-encoding mode
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ManchesterMode {
    /// Rising edge = logic 0, falling edge = logic 1
    ManchesterRising0,
    /// Rising edge = logic 1, falling edge = logic 0
    ManchesterRising1,
}

impl From<ManchesterMode> for SerialInterfaceType {
    fn from(value: ManchesterMode) -> Self {
        match value {
            ManchesterMode::ManchesterRising0 => Self::ManchesterRising0,
            ManchesterMode::ManchesterRising1 => Self::ManchesterRising1,
        }
    }
}

/// Sampling edge for the internally-generated serial clock (SPICKSEL = 1).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum InternalSpiMode {
    /// Sampling on the rising edge of the clock
    SpiRising,
    /// Sampling on the falling edge of the clock
    SpiFalling,
    /// Sampling on each second CKOUT falling edge
    HalfClockFalling,
    /// Sampling on each second CKOUT rising edge
    HalfClockRising,
}

impl From<InternalSpiMode> for SpiClockSelect {
    fn from(value: InternalSpiMode) -> Self {
        match value {
            InternalSpiMode::SpiRising | InternalSpiMode::SpiFalling => SpiClockSelect::InternalCkout,
            InternalSpiMode::HalfClockFalling => SpiClockSelect::InternalCkoutFallingHalved,
            InternalSpiMode::HalfClockRising => SpiClockSelect::InternalCkoutRisingHalved,
        }
    }
}

impl From<InternalSpiMode> for SerialInterfaceType {
    fn from(value: InternalSpiMode) -> Self {
        match value {
            InternalSpiMode::SpiRising | InternalSpiMode::HalfClockRising => SerialInterfaceType::SpiRisingEdge,
            InternalSpiMode::SpiFalling | InternalSpiMode::HalfClockFalling => SerialInterfaceType::SpiFallingEdge,
        }
    }
}

/// Filter order of the analog watchdog's fast filter.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AwdFilterConfig {
    /// Filter bypassed
    Bypass,
    /// FastSinc Filter
    FastSinc(AwdFilterOsr),
    /// Sinc1 filter
    Sinc1(AwdFilterOsr),
    /// Sinc2 filter
    Sinc2(AwdFilterOsr),
    /// Sinc3 filter
    Sinc3(AwdFilterOsr),
}

impl From<AwdFilterConfig> for AwdFilterOrder {
    fn from(value: AwdFilterConfig) -> Self {
        match value {
            AwdFilterConfig::Bypass => AwdFilterOrder::FastSinc,
            AwdFilterConfig::FastSinc(_) => AwdFilterOrder::FastSinc,
            AwdFilterConfig::Sinc1(_) => AwdFilterOrder::Sinc1,
            AwdFilterConfig::Sinc2(_) => AwdFilterOrder::Sinc2,
            AwdFilterConfig::Sinc3(_) => AwdFilterOrder::Sinc3,
        }
    }
}
impl From<AwdFilterConfig> for AwdFilterOsr {
    fn from(value: AwdFilterConfig) -> Self {
        match value {
            AwdFilterConfig::Bypass => AwdFilterOsr::BYPASSED,
            AwdFilterConfig::FastSinc(osr)
            | AwdFilterConfig::Sinc1(osr)
            | AwdFilterConfig::Sinc2(osr)
            | AwdFilterConfig::Sinc3(osr) => osr,
        }
    }
}
/// Effective input bit-width feeding the sinc filter, for gain-ceiling checks.
///
/// A wider input sample eats into the 32-bit signed accumulator's headroom,
/// so the safe filter-gain ceiling depends on how many bits the input itself
/// already occupies.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum InputWidth {
    /// Serial modes: Manchester, SPI (ext/int) - 1-bit stream.
    Serial,
    /// Parallel modes: ADC mux (DATMPX=1), CPU/DMA register writes to
    /// DATINR (DATMPX=2) - 16-bit samples. Verify the exact gain model
    /// against the TRM before relying on this ceiling.
    Parallel,
}

impl InputWidth {
    const fn bits(self) -> u32 {
        match self {
            InputWidth::Serial => 1,
            InputWidth::Parallel => 16,
        }
    }
}

/// Absolute gain limit for the 32-bit signed accumulator, assuming a 1-bit
/// (serial) input.
///
/// The TRM states a limit of `2^32` (`i32::MIN.unsigned_abs()`), but this
/// is one too high. A signed 32-bit value has one less unit of positive
/// headroom than negative headroom, so the maximum positive value is
/// `2^31 - 1` (`i32::MAX`).
const MAX_GAIN_SERIAL: u128 = i32::MAX.unsigned_abs() as u128;

/// Gain ceiling for a given input width.
///
/// Each extra input bit halves the safe filter-gain headroom relative to
/// the 1-bit serial case. This linear model is an approximation for the
/// parallel (16-bit) case - verify against TRM before
/// trusting it in a headroom-critical design; use
/// [`FilterParameters::new_ignore_gain_ceiling`] if you've verified your
/// own headroom instead.
const fn max_gain(width: InputWidth) -> u128 {
    MAX_GAIN_SERIAL >> (width.bits() - 1)
}

/// Filter order and filter oversampling ratio (FOSR).
///
/// `fosr` is the actual oversampling ratio. The value written to the
/// hardware register is `FOSR - 1`.
#[allow(missing_docs)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum FilterOrder {
    /// Filter disabled.
    Disabled,

    /// Fast Sinc filter, with a gain of `2 * FOSR^2`.
    FastSinc { fosr: u16 },

    /// First-order Sinc filter, with a gain of `FOSR`.
    Sinc1 { fosr: u16 },

    /// Second-order Sinc filter, with a gain of `FOSR^2`.
    Sinc2 { fosr: u16 },

    /// Third-order Sinc filter, with a gain of `FOSR^3`.
    Sinc3 { fosr: u16 },

    /// Fourth-order Sinc filter, with a gain of `FOSR^4`.
    Sinc4 { fosr: u16 },

    /// Fifth-order Sinc filter, with a gain of `FOSR^5`.
    Sinc5 { fosr: u16 },
}

impl FilterOrder {
    fn fosr(&self) -> u16 {
        match self {
            Self::Disabled => 1,
            Self::FastSinc { fosr }
            | Self::Sinc1 { fosr }
            | Self::Sinc2 { fosr }
            | Self::Sinc3 { fosr }
            | Self::Sinc4 { fosr }
            | Self::Sinc5 { fosr } => *fosr,
        }
    }

    /// Returns Some(gain) for filter order + OSR combinations that are at
    /// least arithmetically representable. Computed in `u128` so that
    /// representability is checked independently of any gain *ceiling* -
    /// even filter orders whose gain vastly exceeds the accumulator's
    /// headroom are `Some` here, as long as the exponentiation itself
    /// doesn't overflow `u128`. Ceiling enforcement happens separately in
    /// [`FilterParameters::total_gain_checked`].
    ///
    /// Returns `None` only for combinations where `FOSR^order` overflows
    /// `u128` (e.g. `Sinc5` near `fosr = u16::MAX`) - a case that indicates
    /// a nonsensical configuration, not merely a large one.
    fn gain(&self) -> Option<u128> {
        let gain: u128 = match *self {
            Self::Disabled => 1,
            Self::FastSinc { fosr } => 2u128.checked_mul((fosr as u128).checked_pow(2)?)?,
            Self::Sinc1 { fosr } => (fosr as u128).checked_pow(1)?,
            Self::Sinc2 { fosr } => (fosr as u128).checked_pow(2)?,
            Self::Sinc3 { fosr } => (fosr as u128).checked_pow(3)?,
            Self::Sinc4 { fosr } => (fosr as u128).checked_pow(4)?,
            Self::Sinc5 { fosr } => (fosr as u128).checked_pow(5)?,
        };

        Some(gain)
    }

    /// Tests whether the filter order + OSR combination is at least
    /// arithmetically representable (the gain calculation itself doesn't
    /// overflow `u128`). Does not check against any input-width gain ceiling
    /// A filter can be `valid()` and still be far too high-gain for a
    /// given accumulator/input-width combination; see
    /// [`FilterParameters::total_gain_checked`] for that check.
    fn valid(&self) -> bool {
        self.gain().is_some()
    }
}

/// Filter parameters (order, OSR and input width) for a filter.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct FilterParameters {
    order: FilterOrder,
    iosr: u16,
    width: InputWidth,
}

impl FilterParameters {
    /// Create from FilterOrder (carrying its FOSR) and the actual IOSR value,
    /// assuming a 1-bit serial input. Panics if out of range.
    /// For a non-panicking variant, use [`FilterParameters::try_new`].
    /// For parallel (ADC/DMA) inputs use [`FilterParameters::new_for_width`].
    pub fn new(order: FilterOrder, iosr: u16) -> Self {
        Self::new_for_width(order, iosr, InputWidth::Serial)
    }

    /// Try to create from the actual OSR value, assuming a 1-bit serial input.
    /// See the TRM for valid OSR and IOSR.
    /// Filter gain and total gain must each <= 2^31.
    ///
    /// Returns [`Error::InvalidFilterParameters`] if `iosr` is outside
    /// `1..=256`, the filter order's FOSR is invalid, or the resulting gain
    /// exceeds the ceiling.
    pub fn try_new(order: FilterOrder, iosr: u16) -> Result<Self, Error> {
        Self::try_new_for_width(order, iosr, InputWidth::Serial)
    }

    /// Create from FilterOrder, IOSR, and the effective input bit-width.
    /// Use this for parallel (ADC/DMA) inputs, where the gain ceiling is
    /// lower than for serial inputs. Panics if out of range.
    pub fn new_for_width(order: FilterOrder, iosr: u16, width: InputWidth) -> Self {
        Self::try_new_for_width(order, iosr, width)
            .expect("FilterParameters: fosr or iosr out of range for input width")
    }

    /// Try to create from the actual OSR value and the effective input
    /// bit-width. Filter gain and total gain must each fit under `max_gain` for
    /// the given `width`.
    ///
    /// Returns [`Error::InvalidFilterParameters`] if `iosr` is outside
    /// `1..=256`, the filter order's FOSR is invalid, or the resulting gain
    /// exceeds the ceiling for `width`.
    pub fn try_new_for_width(order: FilterOrder, iosr: u16, width: InputWidth) -> Result<Self, Error> {
        if (1..=256).contains(&iosr) && order.fosr() > 0 {
            let params = Self { order, iosr, width };
            if params.total_gain_checked().is_some() {
                return Ok(params);
            }
        }
        Err(Error::InvalidFilterParameters)
    }

    /// Skips the input-width gain-ceiling check entirely. For callers who
    /// have verified their own headroom - e.g. known-small input amplitude,
    /// or downstream scaling/offset that keeps the accumulator in range
    /// regardless of the nominal filter gain.
    ///
    /// Still validates FOSR/IOSR register range and rejects combinations
    /// where the gain calculation itself overflows `u128` (garbage in,
    /// garbage out) - only the "does this fit under the input-width
    /// ceiling" check is bypassed.
    ///
    /// Because the ceiling is skipped, the resulting gain may exceed
    /// `u32::MAX`. [`FilterParameters::total_gain`] truncates silently in
    /// that case (`as u32`) - use [`FilterParameters::total_gain_wide`] for
    /// a lossless `u128` reading, and prefer it when constructing via this
    /// method.
    ///
    /// Returns [`Error::InvalidFilterParameters`] if `iosr` is outside
    /// `1..=256`, the filter order's FOSR is invalid, or the gain
    /// computation itself overflows `u128`.
    pub fn new_ignore_gain_ceiling(order: FilterOrder, iosr: u16) -> Result<Self, Error> {
        if (1..=256).contains(&iosr) && order.fosr() > 0 && order.valid() {
            Ok(Self {
                order,
                iosr,
                width: InputWidth::Serial,
            })
        } else {
            Err(Error::InvalidFilterParameters)
        }
    }

    pub(crate) fn register_values(self) -> (u8, u16, u8) {
        let discriminant = match self.order {
            FilterOrder::Disabled => 0,
            FilterOrder::FastSinc { .. } => 0,
            FilterOrder::Sinc1 { .. } => 1,
            FilterOrder::Sinc2 { .. } => 2,
            FilterOrder::Sinc3 { .. } => 3,
            FilterOrder::Sinc4 { .. } => 4,
            FilterOrder::Sinc5 { .. } => 5,
        };
        (discriminant, self.order.fosr() - 1, (self.iosr - 1) as u8)
    }

    /// Returns the total gain of this filter parametrization, checked
    /// against the ceiling for this instance's input width. `None` if the
    /// ceiling is exceeded - cannot happen for instances built via
    /// `new`/`try_new`/`new_for_width`/`try_new_for_width`, since they
    /// already require this to succeed at construction. May be `None`'s
    /// logical inverse (i.e. always computable) for
    /// `new_ignore_gain_ceiling` instances, since those skip the ceiling -
    /// this method still reports the ceiling-checked view for them, which
    /// is why [`total_gain`]/[`total_gain_wide`] exist as the ceiling-free
    /// accessors.
    fn total_gain_checked(&self) -> Option<u128> {
        self.order
            .gain()
            .and_then(|filter_gain| filter_gain.checked_mul(self.iosr as u128))
            .filter(|&gain| gain <= max_gain(self.width))
    }

    /// Returns the total gain of this filter parametrization, truncated to
    /// `u32`.
    ///
    /// For instances built via `new`/`try_new`/`new_for_width`/
    /// `try_new_for_width`, the gain is guaranteed `<= i32::MAX`-derived
    /// ceiling and this never truncates.
    ///
    /// For instances built via [`FilterParameters::new_ignore_gain_ceiling`],
    /// the true gain may exceed `u32::MAX` and this value silently
    /// truncates (`as u32`) - use [`FilterParameters::total_gain_wide`]
    /// instead in that case.
    pub fn total_gain(&self) -> u32 {
        self.total_gain_wide() as u32
    }

    /// Returns the total gain of this filter parametrization as a lossless
    /// `u128`, regardless of how the instance was constructed. Prefer this
    /// over [`FilterParameters::total_gain`] for instances built via
    /// [`FilterParameters::new_ignore_gain_ceiling`].
    pub fn total_gain_wide(&self) -> u128 {
        // Safe to unwrap: `order.gain()` only returns `None` on arithmetic
        // overflow, which both constructor paths already reject at
        // construction time (`order.valid()` / `total_gain_checked`).
        self.order
            .gain()
            .and_then(|filter_gain| filter_gain.checked_mul(self.iosr as u128))
            .expect("FilterParameters: gain computation overflowed u128 for a validated instance")
    }

    /// Recommended right-shift to achieve i24-fullscale results
    pub fn recommended_shift(&self) -> u8 {
        let gain = self.total_gain_wide();

        gain.next_power_of_two().ilog2().saturating_sub(23) as u8
    }
}

/// Trigger edge selection for injected conversions.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum TriggerEdge {
    /// Detect only rising edges (low to high transitions)
    Rising = 0b01,
    /// Detect only falling edges (high to low transitions)
    Falling = 0b10,
    /// Detect both rising and falling edges
    Any = 0b11,
}
