//! Type-system: sealed and capability markers, channel markers, pin-presence
//! (`PinSet`) and channel-config tokens, indexed channel types and trigger
//! types.

use super::*;

// =============================================================================
// Sealed
// =============================================================================

pub(crate) mod sealed {
    pub trait Sealed {}
}

/// Impls [`sealed::Sealed`] only.
macro_rules! impl_sealed {
    ($($m:path),* $(,)?) => {
        $( impl sealed::Sealed for $m {} )*
    };
}

/// Impls [`sealed::Sealed`] + a target trait.
macro_rules! impl_sealed_and {
    ($trait_:path => $($m:path),* $(,)?) => {
        $(
            impl sealed::Sealed for $m {}
            impl $trait_ for $m {}
        )*
    };
}

/// Impls [`sealed::Sealed`] + a target trait.
macro_rules! impl_trait {
    ($trait_:path => $($m:path),* $(,)?) => {
        $(
            impl $trait_ for $m {}
        )*
    };
}

// =============================================================================
// Channel- and instance-specific pin traits
// =============================================================================

pin_trait!(CkoutPin, Instance, @A);
pin_trait!(Datin0Pin, Instance, @A);
pin_trait!(Ckin0Pin, Instance, @A);
pin_trait!(Ckin1Pin, Instance, @A);
pin_trait!(Datin1Pin, Instance, @A);
pin_trait!(Datin2Pin, Instance, @A);
pin_trait!(Ckin2Pin, Instance, @A);
pin_trait!(Datin3Pin, Instance, @A);
pin_trait!(Ckin3Pin, Instance, @A);
pin_trait!(Datin4Pin, Instance, @A);
pin_trait!(Ckin4Pin, Instance, @A);
pin_trait!(Datin5Pin, Instance, @A);
pin_trait!(Ckin5Pin, Instance, @A);
pin_trait!(Datin6Pin, Instance, @A);
pin_trait!(Ckin6Pin, Instance, @A);
pin_trait!(Datin7Pin, Instance, @A);
pin_trait!(Ckin7Pin, Instance, @A);

// =============================================================================
// Instance + Instance-level markers
// =============================================================================

pub(crate) type Registers = crate::pac::dfsdm::DfsdmSuperset;

pub(crate) trait SealedInstance: crate::rcc::RccPeripheral {
    fn regs() -> Registers;
}

/// A DFSDM peripheral instance.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + 'static {
    /// Number of transceivers on this instance.
    type Transceivers: capability::TransceiverCount;
    /// Number of filters on this instance.
    type Filters: capability::FilterCount;

    /// Shared instance-level state.
    fn instance_state() -> &'static InstanceState;
    // type Split<C: ClockOutputMode>;

    // fn split<C: ClockOutputMode>(dfsdm: Dfsdm<Self, C>) -> Self::Split<C>
    // where
    //     Self: Sized;
}

/// Type-level capability tags for a DFSDM instance shape.
pub(crate) mod capability {
    /// Two transceivers.
    pub struct Tcv2;
    /// Four transceivers.
    pub struct Tcv4;
    /// Eight transceivers.
    pub struct Tcv8;

    /// One filter.
    pub struct Flt1;
    /// Two filters.
    pub struct Flt2;
    /// Four filters.
    pub struct Flt4;
    /// Six filters.
    pub struct Flt6;
    /// Eight filters.
    pub struct Flt8;

    /// Has a per-transceiver pulse-skipper block (DLY).
    pub trait HasDelay {}

    /// Has the HWID hardware-information-register block.
    pub trait HasHwid {}

    /// Accepts a parallel ADC input path (DATMPX = 1).
    ///
    /// The ADC must also be configured to route its results to the DFSDM; use
    /// [`crate::adc::Adc::start_dfsdm`].
    pub trait AdcInput {}

    /// Transceiver count of a shape.
    pub trait TransceiverCount: super::Shape {
        /// Number of transceivers.
        const COUNT: u8;
    }
    impl TransceiverCount for Tcv2 {
        const COUNT: u8 = 2;
    }
    impl TransceiverCount for Tcv4 {
        const COUNT: u8 = 4;
    }
    impl TransceiverCount for Tcv8 {
        const COUNT: u8 = 8;
    }
    /// Filter count of a shape.
    pub trait FilterCount {
        /// Number of filters.
        const COUNT: u8;
    }
    impl FilterCount for Flt1 {
        const COUNT: u8 = 1;
    }
    impl FilterCount for Flt2 {
        const COUNT: u8 = 2;
    }
    impl FilterCount for Flt4 {
        const COUNT: u8 = 4;
    }
    impl FilterCount for Flt6 {
        const COUNT: u8 = 6;
    }
    impl FilterCount for Flt8 {
        const COUNT: u8 = 8;
    }
}

/// Configuration shape: maps a transceiver count to its selector bundle.
#[allow(private_bounds)]
pub trait Shape: sealed::Sealed {
    /// Selector bundle `configure_pins` hands to its closure.
    type Selectors<T: Instance>;
    /// Fresh selector bundle.
    fn selectors<T: Instance>() -> Self::Selectors<T>;
}

impl_sealed!(capability::Tcv2, capability::Tcv4, capability::Tcv8);

/// Marker trait for clock-output modes.
pub trait ClockOutputMode: sealed::Sealed {}
/// Clock output enabled
pub struct OutputEnabled;
/// Clock output disabled
pub struct OutputDisabled;

impl_sealed_and! {
    ClockOutputMode =>
    OutputEnabled,
    OutputDisabled,
}

/// Generalized pin traits
///
///
macro_rules! define_dfsdm_pin_trait {
    ($trait:ident, $description:literal) => {
        #[doc = $description]
        #[cfg(afio)]
        pub trait $trait<T, M, A>: crate::gpio::Pin
        where
            T: Instance,
            M: TransceiverMarker,
        {
            /// Returns the alternate-function number.
            fn af_num(&self) -> u8;
        }

        #[doc = $description]
        #[cfg(not(afio))]
        pub trait $trait<T, M>: crate::gpio::Pin
        where
            T: Instance,
            M: TransceiverMarker,
        {
            /// Returns the alternate-function number.
            fn af_num(&self) -> u8;
        }
    };
}

define_dfsdm_pin_trait!(CkinPin, "Associates a DFSDM clock-input pin with a transceiver.");

define_dfsdm_pin_trait!(DatinPin, "Associates a DFSDM data-input pin with a transceiver.");

// =============================================================================
// Pin presence markers (PinSet)
// =============================================================================

/// Type-level pin presence of one transceiver. Exactly three states exist;
/// "clock without data" has no representative and is therefore inexpressible.
#[allow(private_bounds)]
pub trait PinSet: sealed::Sealed {
    /// Transceiver owns a DATIN pin.
    const HAS_DATA: bool;
    /// Transceiver owns a CKIN pin.
    const HAS_CLK: bool;
    /// Storage for the DATIN pin: `Flex<'d>` if present, `()` if absent.
    type Datin<'d>;
    /// Storage for the CKIN pin: `Flex<'d>` if present, `()` if absent.
    type Ckin<'d>;

    /// Extracts the `Flex` if present, otherwise returns `None`.
    fn extract_datin<'d>(pin: Self::Datin<'d>) -> Option<Flex<'d>>;
    /// Extracts the `Flex` if present, otherwise returns `None`.
    fn extract_ckin<'d>(pin: Self::Ckin<'d>) -> Option<Flex<'d>>;
}

/// Transceiver has no own pins (unused, parallel input, or borrowing the
/// neighbor's).
pub struct NoPins;
/// Transceiver has a DATIN pin, no CKIN (clock from CKOUT or the neighbor).
pub struct DataOnly;
/// Transceiver has a DATIN and a CKIN pin.
pub struct DataClk;

impl_sealed!(NoPins, DataOnly, DataClk);

impl PinSet for NoPins {
    const HAS_DATA: bool = false;
    const HAS_CLK: bool = false;
    type Datin<'d> = ();
    type Ckin<'d> = ();

    fn extract_datin<'d>(_: Self::Datin<'d>) -> Option<Flex<'d>> {
        None
    }
    fn extract_ckin<'d>(_: Self::Ckin<'d>) -> Option<Flex<'d>> {
        None
    }
}

impl PinSet for DataOnly {
    const HAS_DATA: bool = true;
    const HAS_CLK: bool = false;
    type Datin<'d> = Flex<'d>;
    type Ckin<'d> = ();

    fn extract_datin<'d>(pin: Self::Datin<'d>) -> Option<Flex<'d>> {
        Some(pin)
    }
    fn extract_ckin<'d>(_: Self::Ckin<'d>) -> Option<Flex<'d>> {
        None
    }
}

impl PinSet for DataClk {
    const HAS_DATA: bool = true;
    const HAS_CLK: bool = true;
    type Datin<'d> = Flex<'d>;
    type Ckin<'d> = Flex<'d>;

    fn extract_datin<'d>(pin: Self::Datin<'d>) -> Option<Flex<'d>> {
        Some(pin)
    }
    fn extract_ckin<'d>(pin: Self::Ckin<'d>) -> Option<Flex<'d>> {
        Some(pin)
    }
}

/// Pin sets that include a DATIN pin.
pub trait HasData: PinSet {}
impl HasData for DataOnly {}
impl HasData for DataClk {}

/// Pin sets that include a DATIN and a CKIN pin.
pub trait HasDataAndClk: PinSet {}
impl HasDataAndClk for DataClk {}

// =============================================================================
// Channel config tokens
// =============================================================================

/// Pin token: the transceiver gets no pins. Valid in any slot.
pub struct NoPinsCfg;

/// Pin token: the transceiver gets a DATIN pin (AF already configured). Only
/// accepted by the `configure_pins` slot belonging to `M`'s transceiver.
pub struct DatinCfg<'d, T: Instance, M: TransceiverMarker> {
    pub(crate) datin: Flex<'d>,
    pub(crate) _m: PhantomData<(T, M)>,
}

/// Pin token: the transceiver gets DATIN + CKIN.
pub struct DckCfg<'d, T: Instance, M: TransceiverMarker> {
    pub(crate) datin: Flex<'d>,
    pub(crate) ckin: Flex<'d>,
    pub(crate) _m: PhantomData<(T, M)>,
}

/// Accepted by one `configure_pins` slot. `Presence` is the type-level pin
/// state that flows into the split.
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a valid pin token for this DFSDM transceiver",
    label = "this token doesn't belong to transceiver `{M}`",
    note = "return the token from the matching `creator.chN` selector - a token for one transceiver can't be reused on another"
)]
pub trait ChannelCfg<'d, T: Instance, M: TransceiverMarker> {
    /// Pin presence of the declaring transceiver.
    type Presence: PinSet;
    /// Consume the token, yielding its pins in storage form
    /// (`()` for absent, `Flex` for present - no unwrapping anywhere).
    fn into_parts(
        self,
    ) -> (
        <Self::Presence as PinSet>::Datin<'d>,
        <Self::Presence as PinSet>::Ckin<'d>,
    );
}

// NoPinsCfg is valid at ANY position:
impl<'d, T: Instance, M: TransceiverMarker> ChannelCfg<'d, T, M> for NoPinsCfg {
    type Presence = NoPins;
    fn into_parts(self) -> ((), ()) {
        ((), ())
    }
}

// DatinCfg/DckCfg implement the trait ONLY for their OWN channel marker.
// A token minted from `s.ch4` passed to the ch3 slot: E0277.
impl<'d, T: Instance, M: TransceiverMarker> ChannelCfg<'d, T, M> for DatinCfg<'d, T, M> {
    type Presence = DataOnly;
    fn into_parts(self) -> (Flex<'d>, ()) {
        (self.datin, ())
    }
}

impl<'d, T: Instance, M: TransceiverMarker> ChannelCfg<'d, T, M> for DckCfg<'d, T, M> {
    type Presence = DataClk;
    fn into_parts(self) -> (Flex<'d>, Flex<'d>) {
        (self.datin, self.ckin)
    }
}

// =============================================================================
// Indexed channel types
// =============================================================================

macro_rules! define_indexed_channels {
    (
        $enum:ident,
        $marker_trait:ident,
        $index_trait:ident,
        $channel_string:expr,
        $(
            $channel:ident => $index:expr
        ),+ $(,)?
    ) => {

        #[doc = concat!($channel_string, " identifier.")]
        #[repr(usize)]
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum $enum {
            $(

                #[doc = concat!($channel_string, " identifier ", stringify!($index), ".")]
                $channel = $index,
            )+
        }

        impl $enum {

            #[doc = concat!("Index of the ", $channel_string, ".")]
            pub const fn index(self) -> usize {
                self as usize
            }
        }

        #[allow(missing_docs)]
        pub trait $marker_trait: sealed::Sealed {
            const CHANNEL: $enum;
        }

        $(
            #[doc = concat!($channel_string, " marker ", stringify!($index), ".")]
            pub struct $channel;

            #[allow(missing_docs)]
            impl sealed::Sealed for $channel {}


            #[allow(missing_docs)]
            impl $marker_trait for $channel {
                const CHANNEL: $enum = $enum::$channel;
            }
        )+
    };
}

define_indexed_channels!(
    TransceiverChannel,
    TransceiverMarker,
    TransceiverIndex,
    "DFSDM transceiver",
    Tcv0 => 0,
    Tcv1 => 1,
    Tcv2 => 2,
    Tcv3 => 3,
    Tcv4 => 4,
    Tcv5 => 5,
    Tcv6 => 6,
    Tcv7 => 7,
);

define_indexed_channels!(
    FilterChannel,
    FilterMarker,
    FilterIndex,
    "DFSDM filter",
    Flt0 => 0,
    Flt1 => 1,
    Flt2 => 2,
    Flt3 => 3,
    Flt4 => 4,
    Flt5 => 5,
    Flt6 => 6,
    Flt7 => 7,
);

// =============================================================================
// Trigger types
// =============================================================================

/// A trigger source that can drive filter `M` on instance `T`.
///
/// Implemented by `build.rs` for each (instance, filter, source) combination the
/// TRM allows: identity (`jextsel` == signal number) on 5-bit parts, the remapped
/// value on 3-bit parts.
#[diagnostic::on_unimplemented(
    message = "trigger `{Self}` is not valid for DFSDM instance `{T}` filter `{M}`",
    label = "invalid trigger selection",
    note = "check the TRM for valid trigger signals for this variant/filter combination"
)]
pub trait TriggerSource<T: Instance, M: FilterMarker> {
    /// JEXTSEL register value for this (instance, filter, source) combination.
    fn jextsel(&self) -> u8;
}

/// Injected-conversion trigger selection for a filter.
#[derive(Clone, Copy)]
pub enum InjectedTrigger<T: Instance, M: FilterMarker> {
    /// Injected trigger disabled (JEXTEN = 0).
    Disabled,
    /// Trigger conversions on the selected source and edge.
    Enabled {
        /// Register value the trigger resolves to
        jextsel: u8,
        /// Edge to trigger on
        edge: config::TriggerEdge,
        /// Typemarker
        _m: PhantomData<fn() -> (T, M)>,
    },
}

impl<T: Instance, M: FilterMarker> InjectedTrigger<T, M> {
    /// Enable the injected trigger from `trigger` on the given edge.
    pub fn from<TR: TriggerSource<T, M>>(trigger: TR, edge: config::TriggerEdge) -> Self {
        Self::Enabled {
            jextsel: trigger.jextsel(),
            edge,
            _m: PhantomData,
        }
    }
}

// =============================================================================
// General-purpose markertraits
// =============================================================================

/// Marker trait for power-state
pub trait PowerState: sealed::Sealed {}
/// Powered down
pub struct Disabled;
/// Powered up
pub struct Enabled;

impl_sealed_and! {
    PowerState =>
    Disabled,
    Enabled,
}

// =============================================================================
// Channel-level markertraits
// =============================================================================

/// Marks transceivers allowed to use dual data-packing mode.
#[diagnostic::on_unimplemented(
    message = "Dual packing mode is only available on even transceivers (0, 2, 4, 6)",
    label = "`{Self}` is odd - dual mode requires an even transceiver",
    note = "call `new_parallel_dma_dual` on the even transceiver instead"
)]
pub trait DualPackingAllowed: sealed::Sealed {}

impl_trait! {
    DualPackingAllowed =>
    Tcv0,
    Tcv2,
    Tcv4,
    Tcv6,
}

/// Operational mode of a built [`Transceiver`]. Determined by which builder
/// constructor was used; encodes the SITP/SPICKSEL/DATMPX semantics.
pub trait ChannelMode: sealed::Sealed {}
/// SPI input, clock from own CKIN pin (SPICKSEL = 0).
pub struct SpiExtMode;
/// SPI input, clock derived from CKOUT (SPICKSEL = 1..3).
pub struct SpiCkoutMode;
/// Manchester-coded input, clock recovered from the data line (SITP = 2/3).
pub struct ManchesterMode;
/// 16-bit parallel input from CPU/DMA writes (DATMPX = 2), Standard packing
/// (DATPACK = 0).
pub struct ParallelStandard;
/// 16-bit parallel input from CPU/DMA writes (DATMPX = 2), Interleaved packing
/// (DATPACK = 1).
pub struct ParallelInterleaved;
/// 16-bit parallel input in a dual pair: the even channel uses Dual packing
/// (DATPACK = 2), the odd channel is auto-fed from it.
pub struct ParallelPaired;
/// 16-bit parallel input from ADC writes (DATMPX = 1).
pub struct ParallelAdcMode;

impl_sealed_and! {
    ChannelMode =>
    SpiExtMode,
    SpiCkoutMode,
    ManchesterMode,
    ParallelStandard,
    ParallelInterleaved,
    ParallelPaired,
    ParallelAdcMode,
}

/// Marker for modes that carry a serial stream a delay-block pulse skipper
/// can act on. Not implemented for the parallel-input modes
/// ([`ParallelAdcMode`], [`ParallelStandard`], [`ParallelInterleaved`],
/// [`ParallelPaired`]).
pub trait SerialMode: ChannelMode {}

/// Marker for serial modes relying on an external clock; gates
/// clock-absence-detection sync.
pub trait ExternalSerialMode: SerialMode {}

impl_trait! {
    SerialMode =>
    ManchesterMode,
    SpiExtMode,
    SpiCkoutMode
}

impl_trait! {
    ExternalSerialMode =>
    ManchesterMode,
    SpiExtMode
}
// ParallelAdcMode, ParallelStandard, ParallelInterleaved, ParallelPaired
// deliberately excluded

/// Which transceiver's serial pins this transceiver's interface consumes
/// (CFGR1.CHINSEL). Pins are borrowed from that transceiver's slot, so
/// acquire/release live there too (see `Drop`).
pub trait PinSource: sealed::Sealed {
    /// Consume the next transceiver's pins instead of this transceiver's own.
    const FROM_NEIGHBOR: bool;
}
/// CHINSEL = 0
pub struct OwnPins;
/// CHINSEL = 1, pins live on M::Next's slot
pub struct NeighborPins;

impl_sealed!(OwnPins, NeighborPins);

impl PinSource for OwnPins {
    const FROM_NEIGHBOR: bool = false;
}

impl PinSource for NeighborPins {
    const FROM_NEIGHBOR: bool = true;
}
/// Per-instance "successor" transceiver.
///
/// `C` is the instance's transceiver-capability (`<T as Instance>::Transceivers`),
/// so the modulo-N wrap depends on the instance shape, not on the marker itself.
pub trait NextChannel<C: capability::TransceiverCount>: TransceiverMarker {
    /// Marker of the next transceiver, modulo the capability's max count.
    type Next: TransceiverMarker;
}

/// Links a transceiver marker to the next channel's marker (modulo the
/// instance channel count).
pub trait NextChannelForInstance<T: Instance>: TransceiverMarker {
    /// Type representing the next TransceiverMarker in the sequence
    type Next: TransceiverMarker;
}

impl<T, M> NextChannelForInstance<T> for M
where
    T: Instance,
    M: TransceiverMarker + NextChannel<T::Transceivers>,
{
    type Next = <M as NextChannel<T::Transceivers>>::Next;
}

macro_rules! impl_next_channel {
    ($cap:ty, $($cur:ident => $next:ident),+ $(,)?) => {
        $(
            impl NextChannel<$cap> for $cur {
                type Next = $next;
            }
        )+
    };
}

// For 2-channel instances: wraps 1 -> 0
impl_next_channel!(capability::Tcv2,
    Tcv0 => Tcv1,
    Tcv1 => Tcv0,
);

// For 4-channel instances: wraps 3 -> 0
impl_next_channel!(capability::Tcv4,
    Tcv0 => Tcv1,
    Tcv1 => Tcv2,
    Tcv2 => Tcv3,
    Tcv3 => Tcv0,
);

// For 8-channel instances: wraps 7 -> 0
impl_next_channel!(capability::Tcv8,
    Tcv0 => Tcv1,
    Tcv1 => Tcv2,
    Tcv2 => Tcv3,
    Tcv3 => Tcv4,
    Tcv4 => Tcv5,
    Tcv5 => Tcv6,
    Tcv6 => Tcv7,
    Tcv7 => Tcv0,
);

// =============================================================================
// DMA Stuff
// =============================================================================

dma_trait!(Dma, Instance, FilterMarker);

/// No DMA.
pub struct NoDma;
/// Regular-conversion DMA.
pub struct RegDma;
/// Injected-conversion DMA.
pub struct InjDma;

/// DMA mode of a filter half.
pub trait DmaMode: sealed::Sealed {
    /// Whether regular conversions use DMA.
    const REG_ENABLED: bool;
    /// Whether injected conversions use DMA.
    const INJ_ENABLED: bool;
}

impl_sealed!(NoDma, RegDma, InjDma);

impl DmaMode for NoDma {
    const REG_ENABLED: bool = false;
    const INJ_ENABLED: bool = false;
}
impl DmaMode for RegDma {
    const REG_ENABLED: bool = true;
    const INJ_ENABLED: bool = false;
}
impl DmaMode for InjDma {
    const REG_ENABLED: bool = false;
    const INJ_ENABLED: bool = true;
}

/// DMA half of a filter, erased over the concrete half type.
pub trait FilterDma<T, M>
where
    T: Instance + FilterInterrupt<M>,
    M: FilterMarker + InstanceEvents<T>,
{
    /// Pointer to the filter's data register (RDATAR for the regular half,
    /// JDATAR for injected), for custom DMA.
    fn data_register(&mut self) -> *mut u32;

    /// Starts a conversion (regular or injected, depending on the half).
    fn start_conversion(&mut self);

    /// Checks and clears the overrun flag; returns whether it was set.
    fn get_and_clear_overrun(&mut self) -> bool;
}

// =============================================================================
// Generification traits
// =============================================================================

/// Erases all transceivers of an instance into one type, for filter
/// configuration.
pub trait TransceiverTrait<T, P>: sealed::Sealed
where
    T: Instance,
    P: PowerState,
{
    /// Returns this transceiver's channel index.
    fn index(&self) -> usize;
}

impl<'a, 'd, T, M, S, MODE, PS, P> sealed::Sealed for Transceiver<'a, 'd, T, M, S, MODE, PS, P>
where
    T: Instance,
    M: TransceiverMarker + NextChannelForInstance<T>,
    S: PinSet,
    MODE: ChannelMode,
    PS: PinSource,
    P: PowerState,
{
}
impl<'a, 'd, T, M, S, MODE, PS, P> TransceiverTrait<T, P> for Transceiver<'a, 'd, T, M, S, MODE, PS, P>
where
    T: Instance,
    M: TransceiverMarker + NextChannelForInstance<T>,
    S: PinSet,
    MODE: ChannelMode,
    PS: PinSource,
    P: PowerState,
{
    fn index(&self) -> usize {
        M::CHANNEL.index()
    }
}

// =============================================================================
// NonEmpty
// =============================================================================

/// Marker trait to enforce that a const generic `N` is greater than 0.
pub trait NonEmpty {}
// Only implement `NonEmpty` for arrays of unit type `()`
// with lengths 1 through 8.
macro_rules! impl_non_empty {
    ($($n:expr),+) => { $( impl NonEmpty for [(); $n] {} )+ };
}
impl_non_empty!(1, 2, 3, 4, 5, 6, 7, 8);

/// Snapshot of the DFSDM version/ID register cluster @0x7F0 (RM0475 29.9 / RM0436/RM0441/RM0442).
/// Present only on instances whose silicon carries the HWID cluster; see `capability::HasHwid`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Hwid {
    /// Number of filters, self-described by silicon (`HWCFGR.NBF`).
    pub filter_count: u8,
    /// Number of transceivers, self-described by silicon (`HWCFGR.NBT`).
    pub transceiver_count: u8,
    /// Major.minor IP revision (`VERR`).
    pub version: (u8, u8),
    /// IP identifier (`IPIDR`).
    pub ip_id: u32,
    /// Fixed silicon ID + size code (`SIDR`).
    pub silicon_id: u32,
}
