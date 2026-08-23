//! Digital Filter and Sigma-Delta Modulator (DFSDM)

#![macro_use]

use core::marker::PhantomData;

use embassy_hal_internal::PeripheralType;
use embassy_sync::waitqueue::AtomicWaker;

use crate::gpio::{AfType, Flex, OutputType, Pull, Speed};
use crate::timer::low_level::InputCaptureSelection;
use crate::{Peri, interrupt, rcc};

/// TODO
#[allow(missing_docs)]
#[derive(Clone, Copy, PartialEq)]
pub enum GenericConfigEnum {
    Low,
    High,
}

/// DFSDM error.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    //TODO
    /// Overrun error: the hardware generated data faster than we could read it.
    Overrun,
    /// Internal peripheral error.
    PeripheralError,
}

/// DFSDM configuration.
#[non_exhaustive]
pub struct Config {
    //TODO
}

impl Default for Config {
    fn default() -> Self {
        Self {}
    }
}

/// Marker trait for DFSDM clockmodes
mod sealed_clockmode {
    pub trait ClockOutputMode {}
}
use sealed_clockmode::*;

/// No clock output
pub struct OutputEnabled;

/// Clock output enabled
pub struct OutputDisabled;

impl ClockOutputMode for OutputEnabled {}
impl ClockOutputMode for OutputDisabled {}

/// DFSDM driver.
pub struct Dfsdm<'d, T: Instance, C: ClockOutputMode> {
    _instance_marker: PhantomData<T>,
    _clock_mode: PhantomData<C>,
    _ckout: Option<Flex<'d>>,
}

/// Just a wrapper to make this horde of Channels more convenient
pub struct TransceiverChannelBuilders8<T: Instance, C: ClockOutputMode> {
    /// Builder for channel 0
    pub ch0: TransceiverBuilder<T, Tcv0, C>,
    /// Builder for channel 1
    pub ch1: TransceiverBuilder<T, Tcv1, C>,
    /// Builder for channel 2
    pub ch2: TransceiverBuilder<T, Tcv2, C>,
    /// Builder for channel 3
    pub ch3: TransceiverBuilder<T, Tcv3, C>,
    /// Builder for channel 4
    pub ch4: TransceiverBuilder<T, Tcv4, C>,
    /// Builder for channel 5
    pub ch5: TransceiverBuilder<T, Tcv5, C>,
    /// Builder for channel 6
    pub ch6: TransceiverBuilder<T, Tcv6, C>,
    /// Builder for channel 7
    pub ch7: TransceiverBuilder<T, Tcv7, C>,
}

/// Just a wrapper to make this horde of Channels more convenient
pub struct TransceiverChannelBuilders2<T: Instance, C: ClockOutputMode> {
    /// Builder for channel 0
    pub ch0: TransceiverBuilder<T, Tcv0, C>,
    /// Builder for channel 1
    pub ch1: TransceiverBuilder<T, Tcv1, C>,
}

impl<T: Instance, C: ClockOutputMode> TransceiverChannelBuilders8<T, C> {
    pub(crate) fn new(dfsdm: &Dfsdm<T, C>) -> Self {
        Self {
            ch0: TransceiverBuilder::new(dfsdm),
            ch1: TransceiverBuilder::new(dfsdm),
            ch2: TransceiverBuilder::new(dfsdm),
            ch3: TransceiverBuilder::new(dfsdm),
            ch4: TransceiverBuilder::new(dfsdm),
            ch5: TransceiverBuilder::new(dfsdm),
            ch6: TransceiverBuilder::new(dfsdm),
            ch7: TransceiverBuilder::new(dfsdm),
        }
    }
}

impl<T: Instance, C: ClockOutputMode> TransceiverChannelBuilders2<T, C> {
    pub(crate) fn new(dfsdm: &Dfsdm<T, C>) -> Self {
        Self {
            ch0: TransceiverBuilder::new(dfsdm),
            ch1: TransceiverBuilder::new(dfsdm),
        }
    }
}

impl<'d, T, C> Dfsdm<'d, T, C>
where
    T: Instance,
    C: ClockOutputMode,
{
}

impl<'d, T: Instance, C: ClockOutputMode> Dfsdm<'d, T, C> {
    pub fn split(self) -> T::Split<C> {
        T::split(self)
    }
}

#[allow(private_bounds)]
impl<'d, T, C> Dfsdm<'d, T, C>
where
    C: ClockOutputMode,
    T: Instance<Transceivers = capability::Tcv8, Filters = capability::Flt8>,
{
    /// Split up the peripheral after first initialization
    // pub fn split_8ch_8flt(self) -> TransceiverChannelBuilders8<T, C> {
    //     TransceiverChannelBuilders8 {
    //         ch0: TransceiverBuilder::new(&self),
    //         ch1: TransceiverBuilder::new(&self),
    //         ch2: TransceiverBuilder::new(&self),
    //         ch3: TransceiverBuilder::new(&self),
    //         ch4: TransceiverBuilder::new(&self),
    //         ch5: TransceiverBuilder::new(&self),
    //         ch6: TransceiverBuilder::new(&self),
    //         ch7: TransceiverBuilder::new(&self),
    //     }
    // }

    pub fn get_filter_test<F>(&self) -> Filter<T, F>
    where
        T: FilterInterrupt<F>,
        F: FilterMarker,
    {
        Filter {
            _instance_marker: PhantomData,
            _filter_marker: PhantomData,
        }
    }
}

// #[allow(private_bounds)]
// impl<'d, T, C> Dfsdm<'d, T, C>
// where
//     C: ClockOutputMode,
//     T: Instance<Transceivers = capability::Tcv2, Filters = capability::Flt1>,
// {
//     /// Split up the peripheral after first initialization
//     pub fn split_2ch_1flt(self) -> TransceiverChannelBuilders2<T, C> {
//         TransceiverChannelBuilders2 {
//             ch0: TransceiverBuilder::new(&self),
//             ch1: TransceiverBuilder::new(&self),
//         }
//     }
// }

impl<'d, T> Dfsdm<'d, T, OutputEnabled>
where
    T: Instance,
{
    /// Configure DFSDM module with a clock output
    pub fn new_ckout(peri: Peri<'d, T>, ckout: Peri<'d, if_afio!(impl CkoutPin<T, A>)>, config: Config) -> Self {
        let ckout = new_pin!(ckout, AfType::output(OutputType::PushPull, Speed::VeryHigh));

        Self::new_inner(peri, ckout, config)
    }
}

impl<'d, T> Dfsdm<'d, T, OutputDisabled>
where
    T: Instance,
{
    /// Configure DFSDM module without a clock output
    pub fn new(peri: Peri<'d, T>, config: Config) -> Self {
        Self::new_inner(peri, None, config)
    }
}

impl<'d, T, C> Dfsdm<'d, T, C>
where
    T: Instance,
    C: ClockOutputMode,
{
    fn new_inner(peri: Peri<'d, T>, ckout: Option<Flex<'d>>, config: Config) -> Self {
        let _ = config;
        let _ = peri;

        rcc::enable_and_reset::<T>();

        //         macro_rules! config_pins {
        //     ($($pin:ident),*) => {
        //                 critical_section::with(|_| {
        //             $(
        //                 set_as_af!($pin, AfType::input(Pull::None));
        //             )*
        //         })
        //     };
        // }
        // TODO MAYBE USE CRITICAL SECTION FOR AFS?!
        Self {
            _instance_marker: PhantomData,
            _clock_mode: PhantomData,
            _ckout: ckout,
        }
    }
}

type Registers = crate::pac::dfsdm::Dfsdm8ch8fltDlyVer;

trait SealedInstance: crate::rcc::RccPeripheral {
    fn regs() -> Registers;
}

/// Filter-marked interrupt state trait
pub trait FilterInterrupt<F: FilterMarker> {
    /// Interrupt type
    type Interrupt: interrupt::typelevel::Interrupt;

    /// Filter-interrupt state
    fn state() -> &'static State;
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
    /// Presence of delay blocks in this instances' filters
    type Delay: capability::ConfDelay;
    /// Presence of version block in this instance
    type Version: capability::ConfVersion;

    type Split<C: ClockOutputMode>;

    fn split<C: ClockOutputMode>(dfsdm: Dfsdm<Self, C>) -> Self::Split<C>
    where
        Self: Sized;
}

/// Type-level capability tags for DFSDM instance variants.
mod capability {
    /// 2 transceiver channels.
    pub struct Tcv2;
    /// 8 transceiver channels.
    pub struct Tcv8;

    /// 1 filter.
    pub struct Flt1;
    /// 8 filters.
    pub struct Flt8;

    /// No delay unit present.
    pub struct NoDelay;
    /// Delay unit present.
    pub struct WithDelay;

    /// No version register present.
    pub struct NoVersion;
    /// Version register present.
    pub struct WithVersion;

    pub trait ConfDelay {}
    impl ConfDelay for NoDelay {}
    impl ConfDelay for WithDelay {}
    pub trait ConfVersion {}
    impl ConfVersion for NoVersion {}
    impl ConfVersion for WithVersion {}
    pub trait TransceiverCount {
        const COUNT: u8;
    }
    impl TransceiverCount for Tcv2 {
        const COUNT: u8 = 2;
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
    impl FilterCount for Flt8 {
        const COUNT: u8 = 8;
    }
}

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

macro_rules! impl_ckin_bridge {
    ($marker:ty, $existing:ident) => {
        #[cfg(afio)]
        impl<T: Instance, A, P> CkinPin<T, $marker, A> for P
        where
            P: $existing<T, A>,
        {
            fn af_num(&self) -> u8 {
                $existing::af_num(self)
            }
        }

        #[cfg(not(afio))]
        impl<T: Instance, P> CkinPin<T, $marker> for P
        where
            P: $existing<T>,
        {
            fn af_num(&self) -> u8 {
                $existing::af_num(self)
            }
        }
    };
}

impl_ckin_bridge!(Tcv0, Ckin0Pin);
impl_ckin_bridge!(Tcv1, Ckin1Pin);
impl_ckin_bridge!(Tcv2, Ckin2Pin);
impl_ckin_bridge!(Tcv3, Ckin3Pin);
impl_ckin_bridge!(Tcv4, Ckin4Pin);
impl_ckin_bridge!(Tcv5, Ckin5Pin);
impl_ckin_bridge!(Tcv6, Ckin6Pin);
impl_ckin_bridge!(Tcv7, Ckin7Pin);

macro_rules! impl_datin_bridge {
    ($marker:ty, $existing:ident) => {
        #[cfg(afio)]
        impl<T: Instance, A, P> DatinPin<T, $marker, A> for P
        where
            P: $existing<T, A>,
        {
            fn af_num(&self) -> u8 {
                $existing::af_num(self)
            }
        }

        #[cfg(not(afio))]
        impl<T: Instance, P> DatinPin<T, $marker> for P
        where
            P: $existing<T>,
        {
            fn af_num(&self) -> u8 {
                $existing::af_num(self)
            }
        }
    };
}

impl_datin_bridge!(Tcv0, Datin0Pin);
impl_datin_bridge!(Tcv1, Datin1Pin);
impl_datin_bridge!(Tcv2, Datin2Pin);
impl_datin_bridge!(Tcv3, Datin3Pin);
impl_datin_bridge!(Tcv4, Datin4Pin);
impl_datin_bridge!(Tcv5, Datin5Pin);
impl_datin_bridge!(Tcv6, Datin6Pin);
impl_datin_bridge!(Tcv7, Datin7Pin);

/// Implements `SealedInstance` + `Instance` for one DFSDM instance.
///
/// One template, so channel/filter/delay/version capability for a given
/// shape is only ever written once, next to the `Repr` it belongs to.
macro_rules! impl_dfsdm_instance {
    (
        $inst:ident,
        repr: $repr:ty,
        transceivers: $tcv:ty,
        filters: $flt:ty,
        delay: $delay:ty,
        version: $version:ty,
        split: $split_ty:ident $(,)?
    ) => {
        impl SealedInstance for crate::peripherals::$inst {
            fn regs() -> Registers {
                // SAFETY: siehe Trait-Doku — $inst zeigt auf einen realen DFSDM-Block,
                // dessen Basislayout mit Dfsdm8ch8fltDlyVer übereinstimmt. Aufrufer
                // dürfen nur die laut Instance::{Delay,Version,Transceivers,Filters}
                // tatsächlich vorhandenen Register lesen/schreiben.
                unsafe { Registers::from_ptr(crate::pac::$inst.as_ptr()) }
            }
        }

        impl Instance for crate::peripherals::$inst {
            type Repr = $repr;
            type Transceivers = $tcv;
            type Filters = $flt;
            type Delay = $delay;
            type Version = $version;

            type Split<C: ClockOutputMode> = $split_ty<Self, C>;

            fn split<C: ClockOutputMode>(dfsdm: Dfsdm<'_, Self, C>) -> Self::Split<C> {
                $split_ty::new(&dfsdm)
            }
        }
    };
}

foreach_interrupt! {
    ($inst:ident, dfsdm, DFSDM_8CH_8FLT_DLY, FLT0, $irq:ident) => {
        impl_dfsdm_instance!($inst,
            repr: crate::pac::dfsdm::Dfsdm8ch8fltDly,
            transceivers: capability::Tcv8,
            filters: capability::Flt8,
            delay: capability::WithDelay,
            version: capability::NoVersion,
            split: TransceiverChannelBuilders8,
        );
    };
    ($inst:ident, dfsdm, DFSDM_8CH_8FLT, FLT0, $irq:ident) => {
        impl_dfsdm_instance!($inst,
            repr: crate::pac::dfsdm::Dfsdm8ch8flt,
            transceivers: capability::Tcv8,
            filters: capability::Flt8,
            delay: capability::NoDelay,
            version: capability::NoVersion,
            split: TransceiverChannelBuilders8,
        );
    };
    ($inst:ident, dfsdm, DFSDM_2CH_1FLT, FLT0, $irq:ident) => {
        impl_dfsdm_instance!($inst,
            repr: crate::pac::dfsdm::Dfsdm2ch1flt,
            transceivers: capability::Tcv2,
            filters: capability::Flt1,
            delay: capability::NoDelay,
            version: capability::NoVersion,
            split: TransceiverChannelBuilders2,
        );
    };
}

macro_rules! impl_dfsdm_filter_irq {
    ($inst:ident, $filter:ty, $irq:ident) => {
        impl FilterInterrupt<$filter> for crate::peripherals::$inst {
            type Interrupt = crate::interrupt::typelevel::$irq;

            fn state() -> &'static State {
                static STATE: State = State::new();
                &STATE
            }
        }
    };
}

macro_rules! impl_dfsdm_filter_irqs {
    (
        $variant:ident,
        $( $flt:ident => $filter:ty ),+ $(,)?
    ) => {
        foreach_interrupt! {
            $(
                ($inst:ident, dfsdm, $variant, $flt, $irq:ident) => {
                    impl_dfsdm_filter_irq!($inst, $filter, $irq);
                };
            )+
        }
    };
}

impl_dfsdm_filter_irqs! {
    DFSDM_2CH_1FLT,
    FLT0 => Flt0,
}

impl_dfsdm_filter_irqs! {
    DFSDM_8CH_8FLT,
    FLT0 => Flt0,
    FLT1 => Flt1,
    FLT2 => Flt2,
    FLT3 => Flt3,
    FLT4 => Flt4,
    FLT5 => Flt5,
    FLT6 => Flt6,
    FLT7 => Flt7,
}

impl_dfsdm_filter_irqs! {
    DFSDM_8CH_8FLT_DLY,
    FLT0 => Flt0,
    FLT1 => Flt1,
    FLT2 => Flt2,
    FLT3 => Flt3,
    FLT4 => Flt4,
    FLT5 => Flt5,
    FLT6 => Flt6,
    FLT7 => Flt7,
}

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

/// Instance State struct
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

mod sealed {
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

dma_trait!(Dma, Instance, FilterMarker); //TODO

/// Trait for filters to generify them for TODO?
pub trait FilterTrait<M: FilterMarker>: sealed::SealedFilterChannelTrait<M> {
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
pub trait TransceiverTrait<M: TransceiverMarker>: sealed::SealedTransceiverChannelTrait<M> {
    /// Get transceiver identifier
    fn transceiver(&self) -> TransceiverChannel {
        <M as TransceiverMarker>::CHANNEL
    }

    /// Get transceiver index
    fn index(&self) -> usize {
        M::CHANNEL.index()
    }
}

/// Marker trait for Transceiver channels data source
mod sealed_datasource {
    pub trait DataSource {}
}
use sealed_datasource::*;

/// Transceiver get's data clock from outside
pub struct ExternalSource;

/// Transceiver get's data clock from inside
pub struct InternalSource;

impl DataSource for ExternalSource {}
impl DataSource for InternalSource {}

/// Used to configure a [`Transceiver`].
pub struct TransceiverBuilder<T, M, C>
where
    T: Instance,
    M: TransceiverMarker,
    C: ClockOutputMode,
{
    _dfsdm_marker: PhantomData<T>,
    _clockmode_marker: PhantomData<C>,
    _transceiver_marker: PhantomData<M>,
}

impl<T, M, C> TransceiverBuilder<T, M, C>
where
    T: Instance,
    M: TransceiverMarker,
    C: ClockOutputMode,
{
    pub(crate) fn new(_dfsdm: &Dfsdm<T, C>) -> Self {
        //TODO Remove dfsdm reference maybe
        Self {
            _dfsdm_marker: PhantomData,
            _clockmode_marker: PhantomData,
            _transceiver_marker: PhantomData,
        }
    }

    /// Configure [`Transceiver`]. as receiving data from its internal register
    pub fn new_parallel(self) -> Transceiver<T, M, InternalSource> {
        // TODO
        Transceiver {
            _instance_marker: PhantomData,
            _transceiver_marker: PhantomData,
            _datasource_marker: PhantomData,
        }
    }

    /// Configure [`Transceiver`]. as receiving data from an externally clocked bus
    pub fn new_clk_ext(
        self,
        ckin: Peri<if_afio!(impl CkinPin<T, M, A>)>,
        datin: Peri<if_afio!(impl DatinPin<T, M, A>)>,
    ) -> Transceiver<T, M, ExternalSource> {
        set_as_af!(ckin, AfType::input(Pull::None));
        set_as_af!(datin, AfType::input(Pull::None));

        Transceiver {
            _instance_marker: PhantomData,
            _transceiver_marker: PhantomData,
            _datasource_marker: PhantomData,
        }
    }

    /// Configure [`Transceiver`]. as receiving data from a manchester-coded signal
    pub fn new_manchester(self, datin: Peri<if_afio!(impl DatinPin<T, M, A>)>) -> Transceiver<T, M, ExternalSource> {
        set_as_af!(datin, AfType::input(Pull::None));

        Transceiver {
            _instance_marker: PhantomData,
            _transceiver_marker: PhantomData,
            _datasource_marker: PhantomData,
        }
    }
}

impl<T, M> TransceiverBuilder<T, M, OutputEnabled>
where
    T: Instance,
    M: TransceiverMarker,
{
    /// Configure transceiver as receiving data from a synchronously clocked bus clocked by the controller
    pub fn new_clk_int(self, datin: Peri<if_afio!(impl DatinPin<T, M, A>)>) -> Transceiver<T, M, ExternalSource>
    where
        T: Instance,
        M: TransceiverMarker,
    {
        set_as_af!(datin, AfType::input(Pull::None));

        Transceiver {
            _instance_marker: PhantomData,
            _transceiver_marker: PhantomData,
            _datasource_marker: PhantomData,
        }
    }
}

/// Configured DFSDM data input transceiver.
pub struct Transceiver<T, M, S>
where
    T: Instance,
    M: TransceiverMarker,
    S: DataSource,
{
    _instance_marker: PhantomData<T>,
    _transceiver_marker: PhantomData<M>,
    _datasource_marker: PhantomData<S>,
    // ckin_pin: Option<Flex<'d>>,
    // data_pin: Flex<'d>,
}

impl<T, M, S> sealed::SealedTransceiverChannelTrait<M> for Transceiver<T, M, S>
where
    T: Instance,
    M: TransceiverMarker,
    S: DataSource,
{
}

impl<T, M, S> TransceiverTrait<M> for Transceiver<T, M, S>
where
    T: Instance,
    M: TransceiverMarker,
    S: DataSource,
{
}

pub struct Filter<T, M>
where
    T: Instance + FilterInterrupt<M>,
    M: FilterMarker,
{
    _instance_marker: PhantomData<T>,
    _filter_marker: PhantomData<M>,
}

impl<T, M> Filter<T, M>
where
    T: Instance + FilterInterrupt<M>,
    M: FilterMarker,
{
    pub async fn wait_for_irq_test(&mut self) {
        use core::sync::atomic::{Ordering, compiler_fence};
        use core::task::Poll;

        use futures_util::future::poll_fn;

        // T::regs().start();

        poll_fn(|cx| {
            T::state().waker.register(cx.waker());

            compiler_fence(Ordering::SeqCst);
            if !false {
                //T::regs().wait_done() {
                Poll::Pending
            } else {
                Poll::Ready(())
            }
        })
        .await;

        // unsafe { core::ptr::read_volatile(T::regs().data()) }
    }

    pub fn get_regs_test(&mut self) -> Registers {
        DfsdmInner::<T>::enable();
        DfsdmInner::<T>::disable();
        DfsdmInner::<T>::set_output_clock_source(OutputSerialClockSource::Audio);
        DfsdmInner::<T>::set_output_clock_divider(OutputSerialClockDivider::Enabled(123));
        T::regs()
    }
}

struct DfsdmInner<T>
where
    T: Instance,
{
    _instance_marker: PhantomData<T>,
}

impl<T> DfsdmInner<T>
where
    T: Instance,
{
    //////WHOLE PERIPHERAL
    //TODO MAYBE MARK CH0 STUFF EXPLICITLY IN PAC
    fn enable() {
        T::regs().ch(0).cfgr1().modify(|reg| reg.set_dfsdmen(true));
    }

    fn disable() {
        T::regs().ch(0).cfgr1().modify(|reg| reg.set_dfsdmen(false));
    }

    ///ONLY WHEN OFF
    pub fn set_output_clock_source(source: OutputSerialClockSource) {
        T::regs()
            .ch(0)
            .cfgr1()
            .modify(|reg| reg.set_ckoutsrc(source.to_reg_val()));
    }

    ///ONLY WHEN OFF
    pub fn set_output_clock_divider(source: OutputSerialClockDivider) {
        T::regs()
            .ch(0)
            .cfgr1()
            .modify(|reg| reg.set_ckoutdiv(source.to_reg_val()));
    }

    //////CHANNELS
    /// These might crash if they get a [`TransceiverChannel`] outside the capabilities of the [`Instance`]

    pub fn set_data_packing_mode(ch: TransceiverChannel, mode: DataPackingMode) {
        T::regs().ch(ch.index()).cfgr1().modify(|reg| {
            reg.set_datpack(mode as u8);
        });
    }

    pub fn set_input_data_mux(ch: TransceiverChannel, mode: InputDataMux) {
        T::regs().ch(ch.index()).cfgr1().modify(|reg| {
            reg.set_datmpx(mode as u8);
        });
    }

    pub fn set_channel_input(ch: TransceiverChannel, mode: ChannelInput) {
        /// TODO if we do neighbor we might not have to configure our own. YET ANOTHER MARKER!?
        /// Ok How does it look: Channel 0 ... can use channel 1.. pins (and so on. the last one, 7 or 3 or 1 or whatever) can use 0.
        /// Use of a neighbors pins makes the existence of the pin necessary.
        /// Nonuse of the own pin (by yourself AND neighbor) makes the existence of the pin unnecessary.
        /// So we need reference to the pin when instantiating
        ///
        /// ja ich wäre grundsätzlich auf ne viel lustigere und simplere idee gegangen, die leider embassys standardimplementierungen widerspricht, aber was willstetun:
        /// zwei stufen.
        /// dfsdim wird erstmal nciht gesplittet, sondern hat ne config_pins fuinktion, bzw bekommt die pins direkt im ersten init als optionen.
        /// und dann bekommt man channelinput tokens (mit capabilities, weil wenn man nciht genug pins vergibt gibts ohne Ckin natürlich NUR manchester
        /// https://chatgpt.com/share/6a8b4427-eb90-83ed-82e7-82c00036c750
        T::regs().ch(ch.index()).cfgr1().modify(|reg| {
            reg.set_chinsel(mode.to_reg_val());
        });
    }
}

#[derive(Copy, Clone)]
enum OutputSerialClockSource {
    System,
    Audio,
}

impl OutputSerialClockSource {
    fn to_reg_val(self) -> bool {
        match self {
            Self::System => false,
            Self::Audio => true,
        }
    }
}

#[derive(Copy, Clone)]
enum OutputSerialClockDivider {
    Disabled,
    Enabled(u16),
}

impl OutputSerialClockDivider {
    fn to_reg_val(self) -> u8 {
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
enum ChannelInput {
    Same,
    Neighbor,
}

impl ChannelInput {
    fn to_reg_val(self) -> bool {
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


//FUCKING IMPORTANT: DONT DROP, KEEP FLEX IN CHANNEL!!! TODO
// impl<'d> Drop for Flex<'d> {
//     #[inline]
//     fn drop(&mut self) {
//         trace!("gpio: dropping {}", self.pin);
//         critical_section::with(|_| {
//             self.pin.set_as_disconnected();
//         });
//     }
// }