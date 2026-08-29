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

/// DFSDM instance.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + 'static {
    /// PAC representation
    type Repr;
    /// Amount of transceivers in this instance
    type Transceivers: capability::TransceiverCount;
    /// Amount of filters in this instance
    type Filters: capability::FilterCount;

    // type Split<C: ClockOutputMode>;

    // fn split<C: ClockOutputMode>(dfsdm: Dfsdm<Self, C>) -> Self::Split<C>
    // where
    //     Self: Sized;
}

/// Type-level capability tags for DFSDM instance variants.
pub(crate) mod capability {
    /// 2 transceiver channels.
    pub struct Tcv2;
    /// 4 transceiver channels.
    pub struct Tcv4;
    /// 8 transceiver channels.
    pub struct Tcv8;

    /// 1 filter.
    pub struct Flt1;
    /// 4 filters.
    pub struct Flt4;
    /// 8 filters.
    pub struct Flt8;

    /// Has Delay block in channel
    pub trait HasDelay {}

    /// Has a HWID block in the instance
    pub trait HasHwid {}

    /// Accepts data directly from ADC
    pub trait AdcInput {}

    pub trait TransceiverCount: super::Shape {
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
    pub trait FilterCount {
        const COUNT: u8;
    }
    impl FilterCount for Flt1 {
        const COUNT: u8 = 1;
    }
    impl FilterCount for Flt4 {
        const COUNT: u8 = 4;
    }
    impl FilterCount for Flt8 {
        const COUNT: u8 = 8;
    }
}

/// Marker trait for configuration shape
#[allow(private_bounds)]
pub trait Shape: sealed::Sealed {
    /// Selector bundle `configure_pins` hands to its closure.
    type Selectors<T: Instance>;
    /// Fresh selector bundle.
    fn selectors<T: Instance>() -> Self::Selectors<T>;
}

impl_sealed! {
    capability::Tcv2,
    capability::Tcv4,
    capability::Tcv8,
}

impl Shape for capability::Tcv2 {
    type Selectors<T: Instance> = ChannelSelectors2<T>;
    fn selectors<T: Instance>() -> Self::Selectors<T> {
        ChannelSelectors2::new()
    }
}

impl Shape for capability::Tcv4 {
    type Selectors<T: Instance> = ChannelSelectors4<T>;
    fn selectors<T: Instance>() -> Self::Selectors<T> {
        ChannelSelectors4::new()
    }
}

impl Shape for capability::Tcv8 {
    type Selectors<T: Instance> = ChannelSelectors8<T>;
    fn selectors<T: Instance>() -> Self::Selectors<T> {
        ChannelSelectors8::new()
    }
}

/// Marker trait for DFSDM clockmodes
pub trait ClockOutputMode: sealed::Sealed {}
/// Clock output enabled
pub struct OutputEnabled;
/// No clock output
pub struct OutputDisabled;

impl_sealed_and! {
    ClockOutputMode =>
    OutputEnabled,
    OutputDisabled,
}

// =============================================================================
// Interrupt-Marking
// =============================================================================

/// Filter-marked interrupt state trait
pub trait FilterInterrupt<F: FilterMarker> {
    /// Interrupt type
    type Interrupt: interrupt::typelevel::Interrupt;

    /// Filter-interrupt state
    fn state() -> &'static State;
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

define_dfsdm_pin_trait!(
    CkinPin,
    "Associates a DFSDM clock-input pin with a transceiver channel."
);

define_dfsdm_pin_trait!(
    DatinPin,
    "Associates a DFSDM data-input pin with a transceiver channel."
);

// ============================================================
// Pin presence markers (PinSet)
// ============================================================

/// Type-level pin presence of one channel. Exactly three states exist;
/// "clock without data" has no representative and is therefore inexpressible.
#[allow(private_bounds)]
pub trait PinSet: sealed::Sealed {
    /// Channel owns a DATIN pin.
    const HAS_DATA: bool;
    /// Channel owns a CKIN pin.
    const HAS_CLK: bool;
    /// Storage for the DATIN pin: `Flex<'d>` if present, `()` if absent.
    type Datin<'d>;
    /// Storage for the CKIN pin: `Flex<'d>` if present, `()` if absent.
    type Ckin<'d>;
}

/// Channel has no pins (unused, parallel input, or PDM-right reading the
/// next channel's line).
pub struct NoPins;
/// Channel has a DATIN pin only (Manchester, CKOUT-clocked SPI, PDM left).
pub struct DataOnly;
/// Channel has DATIN + CKIN (externally clocked SPI).

pub struct DataClk;

impl_sealed! {
    NoPins,
    DataOnly,
    DataClk,
}

impl PinSet for NoPins {
    const HAS_DATA: bool = false;
    const HAS_CLK: bool = false;
    type Datin<'d> = ();
    type Ckin<'d> = ();
}

impl PinSet for DataOnly {
    const HAS_DATA: bool = true;
    const HAS_CLK: bool = false;
    type Datin<'d> = Flex<'d>;
    type Ckin<'d> = ();
}

impl PinSet for DataClk {
    const HAS_DATA: bool = true;
    const HAS_CLK: bool = true;
    type Datin<'d> = Flex<'d>;
    type Ckin<'d> = Flex<'d>;
}

/// Pin sets that include a DATIN pin.
pub trait HasData: PinSet {}
impl HasData for DataOnly {}
impl HasData for DataClk {}

/// Pin sets that include a DataClk pin.
pub trait HasDataAndClk: PinSet {}
impl HasDataAndClk for DataClk {}

// ============================================================
// Channel config tokens
// ============================================================

/// Token: channel gets no pins. Valid in any slot.
pub struct NoPinsCfg;

/// Token: channel gets a DATIN pin (AF already configured). Only accepted by
/// the `configure_pins` slot belonging to `M`'s channel.
pub struct DatinCfg<'d, T: Instance, M: TransceiverMarker> {
    pub(crate) datin: Flex<'d>,
    pub(crate) _m: PhantomData<(T, M)>,
}

/// Token: channel gets DATIN + CKIN.
pub struct DckCfg<'d, T: Instance, M: TransceiverMarker> {
    pub(crate) datin: Flex<'d>,
    pub(crate) ckin: Flex<'d>,
    pub(crate) _m: PhantomData<(T, M)>,
}

/// Accepted by one `configure_pins` slot. `Presence` is the type-level pin
/// state that flows into the split.
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a valid pin token for this DFSDM channel",
    label = "this token doesn't belong to channel `{M}`",
    note = "return the token from the matching `creator.chN` selector - a token for one channel can't be reused on another"
)]
pub trait ChannelCfg<'d, T: Instance, M: TransceiverMarker> {
    /// Pin presence of the declaring channel.
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
// Interrupthandler
// =============================================================================

/// Trait for interrupt handlers
pub struct InterruptHandler<T: Instance, F: FilterMarker> {
    _marker: PhantomData<(T, F)>,
}

impl<T, F> interrupt::typelevel::Handler<<T as FilterInterrupt<F>>::Interrupt> for InterruptHandler<T, F>
where
    T: Instance + FilterInterrupt<F>,
    F: FilterMarker,
{
    unsafe fn on_interrupt() {
        <T as FilterInterrupt<F>>::state().waker.wake();
        // T::disable_exti_interrupt();
        // T::state().waker.wake();
        // T::clear_exti_pending();
        // handle this filter's status/ovr/regular-conversion bits only
    }
}

// impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
//     unsafe fn on_interrupt() {
//         //TODO THIS IS DCMI STUFF
//         crate::pac::DFSDM1.ch(0);
//         crate::pac::DFSDM2.ch(0);
//         let ris = crate::pac::DCMI.ris().read();
//         if ris.err_ris() {
//             trace!("DCMI IRQ: Error.");
//             crate::pac::DCMI.ier().modify(|ier| ier.set_err_ie(false));
//         }
//         if ris.ovr_ris() {
//             trace!("DCMI IRQ: Overrun.");
//             crate::pac::DCMI.ier().modify(|ier| ier.set_ovr_ie(false));
//         }
//         if ris.frame_ris() {
//             trace!("DCMI IRQ: Frame captured.");
//             crate::pac::DCMI.ier().modify(|ier| ier.set_frame_ie(false));
//         }
//         STATE.waker.wake();
//     }
// }

// =============================================================================
// Interrupt/FilterChannel state
// =============================================================================

/// State shared between interrupt routine and filter object
pub struct State {
    /// Waker for the state
    pub waker: AtomicWaker,
    // Maybe we need some atomics? pub injected_done: core::sync::atomic::AtomicBool,
}

impl State {
    /// Instantiate fresh State
    pub const fn new() -> Self {
        Self {
            waker: AtomicWaker::new(),
        }
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

        #[doc = concat!($channel_string, " channel identifier.")]
        #[repr(usize)]
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum $enum {
            $(

                #[doc = concat!($channel_string, " channel identifier ", stringify!($index), ".")]
                $channel = $index,
            )+
        }

        impl $enum {

            /// Index of the channel
            pub const fn index(self) -> usize {
                self as usize
            }
        }

        #[allow(missing_docs)]
        pub trait $marker_trait: sealed::Sealed {
            const CHANNEL: $enum;
        }

        $(
            #[doc = concat!($channel_string, " channel marker ", stringify!($index), ".")]
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

trigger_trait!(InjectedTrigger, Instance);

// TODO MAYBE USE??
// pub struct InjectedDfsdmTrigger<T: Instance> {
//     _trigger: u8,
//     _marker: PhantomData<T>,
// }

// impl<T: Instance> InjectedDfsdmTrigger<T> {
//     pub fn from(trigger: impl InjectedTrigger<T>) -> Self {
//         Self {
//             _trigger: trigger.signal(),
//             _marker: PhantomData,
//         }
//     }
// }

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

/// Marks a Transceiverchannel as being allowed to set Datapacking dual-mode.
#[diagnostic::on_unimplemented(
    message = "Dual packing mode is only available on even channels (0, 2, 4, 6)",
    label = "`{Self}` is odd — dual mode requires an even channel",
    note = "call `new_parallel_dma_dual` on the even channel instead"
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
/// constructor was used; encodes the SITP/SPICKSEL/CHINSEL/DATMPX semantics.
pub trait ChannelMode: sealed::Sealed {}
/// SPI input, clock from own CKIN pin (SPICKSEL = 0).
pub struct SpiExtMode;
/// SPI input, clock derived from CKOUT (SPICKSEL = 1..3).
pub struct SpiCkoutMode;
/// Manchester-coded input, clock recovered from the data line (SITP = 2/3).
pub struct ManchesterMode;
/// PDM left half: owns the mic data line, samples on rising CKOUT edge.
pub struct PdmLeftMode;
/// PDM right half: pinless, decodes the NEXT channel's DATIN on the falling
/// CKOUT edge (CHINSEL = 1, SITP = 1).
pub struct PdmRightMode;
/// 16-bit parallel input from CPU/DMA writes (DATMPX = 2).
pub struct ParallelDmaMode;
/// 16-bit parallel input from ADC writes (DATMPX = 1).
pub struct ParallelAdcMode;

impl_sealed_and! {
    ChannelMode =>
    SpiExtMode,
    SpiCkoutMode,
    ManchesterMode,
    PdmLeftMode,
    PdmRightMode,
    ParallelDmaMode,
    ParallelAdcMode
}

/// Marker trait for Transceiver channels data source
pub trait DataSource: sealed::Sealed {}
/// Transceiver get's data clock from outside
pub struct ExternalSource;
/// Transceiver get's data clock from inside
pub struct InternalSource;

impl_sealed_and! {
    DataSource =>
    ExternalSource,
    InternalSource,
}

/// Per‑instance "successor" channel.
///
/// `C` is the instance's transceiver‑capability (`<T as Instance>::Transceivers`),
/// so the modulo‑N wrap depends on the instance shape, not on the marker itself.
pub trait NextChannel<C: capability::TransceiverCount>: TransceiverMarker {
    /// Marker of the next channel, modulo the capability's max count.
    type Next: TransceiverMarker;
}

/// Convenience trait to get the next channel directly from an Instance
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

dma_trait!(Dma, Instance, FilterMarker); //TODO

// =============================================================================
// Generification traits
// =============================================================================

/// Trait for filters to generify them for TODO?
pub trait FilterTrait<M: FilterMarker>: sealed::Sealed {
    /// Get filter identifier
    fn filter(&self) -> FilterChannel {
        M::CHANNEL
    }

    /// Get filter index
    fn index(&self) -> usize {
        M::CHANNEL.index()
    }
}

/// Trait for transceivers to generify them for Filterconfiguration
pub trait TransceiverTrait<M: TransceiverMarker>: sealed::Sealed {
    /// Get transceiver identifier
    fn transceiver(&self) -> TransceiverChannel {
        <M as TransceiverMarker>::CHANNEL
    }

    /// Get transceiver index
    fn index(&self) -> usize {
        M::CHANNEL.index()
    }
}

impl<'a, 'd, T, M, S, MODE, P> sealed::Sealed for Transceiver<'a, 'd, T, M, S, MODE, P>
where
    T: Instance,
    M: TransceiverMarker + NextChannelForInstance<T>,
    S: PinSet,
    MODE: ChannelMode,
    P: PowerState,
{
}

impl<'a, 'd, T, M, S, MODE, P> TransceiverTrait<M> for Transceiver<'a, 'd, T, M, S, MODE, P>
where
    T: Instance,
    M: TransceiverMarker + NextChannelForInstance<T>,
    S: PinSet,
    MODE: ChannelMode,
    P: PowerState,
{
}

// =============================================================================
// Config types
// =============================================================================

/// Register config types
pub mod config_types {

    /// Output serial clock source selection
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
    #[derive(Copy, Clone, PartialEq, Eq)]
    pub struct CkoutDivider(u8);

    impl CkoutDivider {
        /// Create from the actual divider value (2..=256).
        /// Panics if out of range.
        /// For a runtime-stabler variant try [`CkoutDivider::try_from`]
        pub fn new(divider: u16) -> Self {
            assert!((2..=256).contains(&divider), "CKOUT divider must be 2..=256");
            Self((divider - 1) as u8)
        }

        /// CKOUT disabled (register value 0).
        pub const DISABLED: Self = Self(0);

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

    /// Data packing mode in CHyDATINR register
    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    #[repr(u8)]
    pub enum DataPackingMode {
        /// input data in DFSDM_CHyDATINR register are stored only in INDAT0[15:0]. To empty
        /// DFSDM_CHyDATINR register one sample must be read by the DFSDM filter from channel y
        Standard = 0,
        /// Interleaved: input data in DFSDM_CHyDATINR register are stored as two samples:
        /// –first sample in INDAT0[15:0] (assigned to channel y)
        /// –second sample INDAT1[15:0] (assigned to channel y)
        /// To empty DFSDM_CHyDATINR register, two samples must be read by the digital filter from
        /// channel y (INDAT0[15:0] part is read as first sample and then INDAT1[15:0] part is read as next
        /// sample).
        Interleaved = 1,
        /// Dual: input data in DFSDM_CHyDATINR register are stored as two samples:
        /// –first sample INDAT0[15:0] (assigned to channel y)
        /// –second sample INDAT1[15:0] (assigned to channel y+1)
        /// To empty DFSDM_CHyDATINR register first sample must be read by the digital filter from channel
        /// y and second sample must be read by another digital filter from channel y+1. Dual mode is
        /// available only on even channel numbers (y = 0, 2, 4, 6), for odd channel numbers (y = 1, 3, 5, 7)
        /// DFSDM_CHyDATINR is write protected. If an even channel is set to dual mode then the following
        /// odd channel must be set into standard mode (DATPACK[1:0]=0) for correct cooperation with even channel.
        Dual = 2,
        // 3 = Reserved
    }

    /// Channel inputs selection
    #[derive(Copy, Clone)]
    pub enum ChannelInput {
        /// Channel inputs are taken from pins of the same channel y
        Same,
        /// Channel inputs are taken from pins of the following channel (channel (y+1) modulo 8)
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

    /// Input data multiplexer
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

    ///SPI clock select for channel
    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    #[repr(u8)]
    pub enum SpiClockSelect {
        /// Clock coming from external CKIN pin of the channel
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

    /// Type of the serial interface
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

    /// Filter order of the analog watchdogs Filter
    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    #[repr(u8)]
    pub enum AnalogWatchdogFilterOrder {
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
        /// Maps breaksignal connections per source channel
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

    /// Unsigned integer constrained to `BITS` bits, backed by a `u32`.
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct UInt<const BITS: u8>(u32);

    impl<const BITS: u8> UInt<BITS> {
        /// Create new sized uint, asserts correctness at runtime.
        /// For a runtime-stabler variant try [`UInt::try_from`]
        pub const fn new(value: u32) -> Self {
            const { core::assert!(BITS > 0 && BITS <= 32, "invalid bit width") };
            core::assert!(value < (1 << BITS), "value exceeds bit width");
            Self(value)
        }

        /// Unterlying value
        pub const fn value(self) -> u32 {
            self.0
        }
    }

    impl<const BITS: u8> TryFrom<u32> for UInt<BITS> {
        type Error = ();

        /// Try to create new sized uint.
        fn try_from(value: u32) -> Result<Self, Self::Error> {
            if value < (1 << BITS) { Ok(Self(value)) } else { Err(()) }
        }
    }

    impl<const BITS: u8> From<UInt<BITS>> for u8 {
        fn from(value: UInt<BITS>) -> Self {
            return value.0 as u8;
        }
    }

    // =============================================================================
    // Types specifically used for pub config
    // =============================================================================

    /// [`DataPackingMode`] restricted to non-dual modes
    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    pub enum DataPackingModeReduced {
        /// See [`DataPackingMode::Standard`]
        Standard,
        /// See [`DataPackingMode::Interleaved`]
        Interleaved,
    }

    impl From<DataPackingModeReduced> for DataPackingMode {
        fn from(mode: DataPackingModeReduced) -> Self {
            match mode {
                DataPackingModeReduced::Standard => Self::Standard,
                DataPackingModeReduced::Interleaved => Self::Interleaved,
            }
        }
    }

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

    /// SPI Edge mode for internal clock use
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
}
