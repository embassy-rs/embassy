use super::*;

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
// Instance
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

pub(crate) mod sealed_shape {
    pub trait Sealed {}
}

#[allow(private_bounds)]
pub trait Shape: sealed_shape::Sealed {
    /// Selector bundle `configure_pins` hands to its closure.
    type Selectors<T: Instance>;
    /// Fresh selector bundle.
    fn selectors<T: Instance>() -> Self::Selectors<T>;
}

impl sealed_shape::Sealed for capability::Tcv2 {}
impl sealed_shape::Sealed for capability::Tcv4 {}
impl sealed_shape::Sealed for capability::Tcv8 {}

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
        $sealed_trait:path,
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
        pub trait $marker_trait: $sealed_trait {
            const CHANNEL: $enum;
        }

        $(
            #[doc = concat!($channel_string, " channel marker ", stringify!($index), ".")]
            pub struct $channel;

            #[allow(missing_docs)]
            impl $sealed_trait for $channel {}


            #[allow(missing_docs)]
            impl $marker_trait for $channel {
                const CHANNEL: $enum = $enum::$channel;
            }
        )+
    };
}

pub(crate) mod sealed {
    pub trait SealedTransceiverTrait {}
    pub trait SealedFilterTrait {}
    pub trait SealedFilterChannelTrait<F: super::FilterMarker> {}
    pub trait SealedTransceiverChannelTrait<F: super::TransceiverMarker> {}
}

define_indexed_channels!(
    TransceiverChannel,
    TransceiverMarker,
    TransceiverIndex,
    sealed::SealedTransceiverTrait,
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
    sealed::SealedFilterTrait,
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
// Datapacking markertrait
// =============================================================================

/// Marks a Transceiverchannel as being allowed to set Datapacking dual-mode.
pub trait DualPackingAllowed {}

impl DualPackingAllowed for Tcv0 {}
impl DualPackingAllowed for Tcv2 {}
impl DualPackingAllowed for Tcv4 {}
impl DualPackingAllowed for Tcv6 {}

// =============================================================================
// Trigger types
// =============================================================================

trigger_trait!(InjectedTrigger, Instance);

pub struct InjectedDfsdmTrigger<T: Instance> {
    _trigger: u8,
    _marker: PhantomData<T>,
}

impl<T: Instance> InjectedDfsdmTrigger<T> {
    pub fn from(trigger: impl InjectedTrigger<T>) -> Self {
        Self {
            _trigger: trigger.signal(),
            _marker: PhantomData,
        }
    }
}

// =============================================================================
// PowerState (Enabled/Disabled) marker traits
// =============================================================================

mod sealed_state {
    pub trait Sealed {}
}

pub trait PowerState: sealed_state::Sealed {}

// Module States
pub struct Disabled;
pub struct Enabled;

// Channel States (could reuse the above, but sometimes hardware has separate enables)
pub struct ChannelDisabled;
pub struct ChannelEnabled;

impl sealed_state::Sealed for Disabled {}
impl sealed_state::Sealed for Enabled {}

impl PowerState for Disabled {}
impl PowerState for Enabled {}

pub mod config_types {

    #[derive(Copy, Clone)]
    pub enum OutputSerialClockSource {
        System,
        Audio,
    }

    impl OutputSerialClockSource {
        pub fn to_reg_val(self) -> bool {
            match self {
                Self::System => false,
                Self::Audio => true,
            }
        }
    }

    #[derive(Copy, Clone)]
    pub enum OutputSerialClockDivider {
        Disabled,
        Enabled(u16),
    }

    impl OutputSerialClockDivider {
        pub fn to_reg_val(self) -> u8 {
            match self {
                Self::Disabled => 0,
                Self::Enabled(val) => (val - 1).try_into().expect("Clock divider must be between 2 and 256"), // TODO Maybe replace with error propagation?
            }
        }
    }

    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    #[repr(u8)]
    pub enum DataPackingMode {
        Standard = 0,
        Interleaved = 1,
        Dual = 2,
        // 3 = Reserved
    }

    #[derive(Copy, Clone)]
    pub enum ChannelInput {
        Same,
        Neighbor,
    }

    impl ChannelInput {
        pub fn to_reg_val(self) -> bool {
            match self {
                Self::Same => false,
                Self::Neighbor => true,
            }
        }
    }

    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    #[repr(u8)]
    pub enum InputDataMux {
        ExternalSerial = 0,
        InternalAdc = 1,
        InternalRegisterWrite = 2,
        // 3 = Reserved
    }

    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    #[repr(u8)]
    pub enum SpiClockSelect {
        ExternalCkin = 0,
        InternalCkout = 1,
        InternalCkoutFallingHalved = 2,
        InternalCkoutRisingHalved = 3,
    }

    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    #[repr(u8)]
    pub enum SerialInterfaceType {
        SpiRisingEdge = 0,
        SpiFallingEdge = 1,
        ManchesterRising0 = 2,
        ManchesterRising1 = 3,
    }

    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    #[repr(u8)]
    pub enum AnalogWatchdogFilterOrder {
        FastSinc = 0,
        Sinc1 = 1,
        Sinc2 = 2,
        Sinc3 = 3,
    }

    bitflags::bitflags! {
        #[derive(Clone, Copy, PartialEq, Eq)]
        pub struct BreakSignals: u8 {
            const BREAK0 = 0b0001;
            const BREAK1 = 0b0010;
            const BREAK2 = 0b0100;
            const BREAK3 = 0b1000;
        }
    }

    /// Unsigned integer constrained to `BITS` bits, backed by a `u32`.
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct UInt<const BITS: u8>(u32);

    impl<const BITS: u8> UInt<BITS> {
        ///Create new sized uint, asserts correctness at runtime. For a runtime-stabler variant try [`UInt::try_from`]
        pub const fn new(value: u32) -> Self {
            const { core::assert!(BITS > 0 && BITS <= 32, "invalid bit width") };
            core::assert!(value < (1 << BITS), "value exceeds bit width");
            Self(value)
        }

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
}
