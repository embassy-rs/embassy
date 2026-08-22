//! Digital Filter and Sigma-Delta Modulator (DFSDM)

#![macro_use]

use core::marker::PhantomData;

use embassy_hal_internal::PeripheralType;
use embassy_sync::waitqueue::AtomicWaker;

use crate::gpio::{AfType, Flex, OutputType, Pull, Speed};
use crate::{Peri, interrupt, rcc};

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

/// TODO
#[allow(missing_docs)]
#[derive(Clone, Copy, PartialEq)]
pub enum GenericConfigEnum {
    Low,
    High,
}

struct State {
    waker: AtomicWaker,
}

impl State {
    const fn new() -> State {
        State {
            waker: AtomicWaker::new(),
        }
    }
}

static STATE: State = State::new();

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

impl<'d, T, C> Dfsdm<'d, T, C>
where
    T: Instance,
    C: ClockOutputMode,
{
}

#[allow(private_bounds)]
impl<'d, T, C> Dfsdm<'d, T, C>
where
    C: ClockOutputMode,
    T: Instance<Transceivers = capability::Tcv8, Filters = capability::Flt8>,
{
    /// Split up the peripheral after first initialization
    pub fn split_8ch_8flt(self) -> TransceiverChannelBuilders8<T, C> {
        TransceiverChannelBuilders8 {
            ch0: TransceiverBuilder::new(&self),
            ch1: TransceiverBuilder::new(&self),
            ch2: TransceiverBuilder::new(&self),
            ch3: TransceiverBuilder::new(&self),
            ch4: TransceiverBuilder::new(&self),
            ch5: TransceiverBuilder::new(&self),
            ch6: TransceiverBuilder::new(&self),
            ch7: TransceiverBuilder::new(&self),
        }
    }
}

#[allow(private_bounds)]
impl<'d, T, C> Dfsdm<'d, T, C>
where
    C: ClockOutputMode,
    T: Instance<Transceivers = capability::Tcv2, Filters = capability::Flt1>,
{
    /// Split up the peripheral after first initialization
    pub fn split_2ch_1flt(self) -> TransceiverChannelBuilders2<T, C> {
        TransceiverChannelBuilders2 {
            ch0: TransceiverBuilder::new(&self),
            ch1: TransceiverBuilder::new(&self),
        }
    }
}

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

trait SealedInstance: crate::rcc::RccPeripheral {
    // fn regs(&self) -> crate::pac::dfsdm::Dfsdm;
}

pub trait FilterInterrupt<F: FilterMarker> {
    type Interrupt: interrupt::typelevel::Interrupt;
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

/// Associates a DFSDM clock-input pin with a specific transceiver channel.
///
/// The channel type `C` and marker `M` tie the pin to a concrete
/// [`TransceiverChannel`]. This allows the compiler to reject pin
/// combinations that do not belong to the requested channel.
///
/// `T` identifies the DFSDM peripheral.
/// `A` identifies the alternate-function configuration.
#[cfg(afio)]
pub trait CkinPin<T, M, A>: crate::gpio::Pin
where
    T: Instance,
    M: TransceiverMarker,
{
    /// Forwards the alternate-function number provided by the underlying pin.
    fn af_num(&self) -> u8;
}

/// Associates a DFSDM clock-input pin with a specific transceiver channel.
///
/// The channel type `C` and marker `M` tie the pin to a concrete
/// [`TransceiverChannel`]. This allows the compiler to reject pin
/// combinations that do not belong to the requested channel.
///
/// `T` identifies the DFSDM peripheral.
#[cfg(not(afio))]
pub trait CkinPin<T, M>: crate::gpio::Pin
where
    T: Instance,
    M: TransceiverMarker,
{
    /// Forwards the alternate-function number provided by the underlying pin.
    fn af_num(&self) -> u8;
}
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

/// Associates a DFSDM data-input pin with a specific transceiver channel.
///
/// The channel type `C` and marker `M` tie the pin to a concrete
/// [`TransceiverChannel`]. This allows the compiler to reject pin
/// combinations that do not belong to the requested channel.
///
/// `T` identifies the DFSDM peripheral.
/// `A` identifies the alternate-function configuration.
#[cfg(afio)]
pub trait DatinPin<T, M, A>: crate::gpio::Pin
where
    T: Instance,
    M: TransceiverMarker,
{
    /// Forwards the alternate-function number provided by the underlying pin.
    fn af_num(&self) -> u8;
}

/// Associates a DFSDM data-input pin with a specific transceiver channel.
///
/// The channel type `C` and marker `M` tie the pin to a concrete
/// [`Transceiver`]. This allows the compiler to reject pin
/// combinations that do not belong to the requested channel.
///
/// `T` identifies the DFSDM peripheral.
#[cfg(not(afio))]
pub trait DatinPin<T, M>: crate::gpio::Pin
where
    T: Instance,
    M: TransceiverMarker,
{
    /// Forwards the alternate-function number provided by the underlying pin.
    fn af_num(&self) -> u8;
}
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
        version: $version:ty $(,)?
    ) => {
        impl SealedInstance for crate::peripherals::$inst {
            // fn regs() -> crate::pac::dfsdm::Dfsdm2ch1flt {
            //     // SAFETY: every DFSDM shape shares the base ch()/flt()
            //     // register layout with Dfsdm2ch1flt at identical offsets;
            //     // richer shapes only append registers, never relocate these.
            //     unsafe { crate::pac::dfsdm::Dfsdm2ch1flt::from_ptr(crate::pac::$inst.as_ptr()) }
            // }
        }

        impl Instance for crate::peripherals::$inst {
            type Repr = $repr;
            type Transceivers = $tcv;
            type Filters = $flt;
            type Delay = $delay;
            type Version = $version;
        }
    };
}

foreach_interrupt! {
    // Fires once per instance — FLT0 is used purely as "does this instance
    // exist" gate; the actual IRQ name is unused here since interrupts are
    // now per-filter via FilterInterrupt, not on Instance.
    ($inst:ident, dfsdm, DFSDM_8CH_8FLT_DLY, FLT0, $irq:ident) => {
        impl_dfsdm_instance!($inst,
            repr: crate::pac::dfsdm::Dfsdm8ch8fltDly,
            transceivers: capability::Tcv8,
            filters: capability::Flt8,
            delay: capability::WithDelay,
            version: capability::NoVersion,
        );
    };
    ($inst:ident, dfsdm, DFSDM_8CH_8FLT, FLT0, $irq:ident) => {
        impl_dfsdm_instance!($inst,
            repr: crate::pac::dfsdm::Dfsdm8ch8flt,
            transceivers: capability::Tcv8,
            filters: capability::Flt8,
            delay: capability::NoDelay,
            version: capability::NoVersion,
        );
    };
    ($inst:ident, dfsdm, DFSDM_2CH_1FLT, FLT0, $irq:ident) => {
        impl_dfsdm_instance!($inst,
            repr: crate::pac::dfsdm::Dfsdm2ch1flt,
            transceivers: capability::Tcv2,
            filters: capability::Flt1,
            delay: capability::NoDelay,
            version: capability::NoVersion,
        );
    };
}

macro_rules! impl_dfsdm_filter_irq {
    ($inst:ident, $filter:ty, $irq:ident) => {
        impl FilterInterrupt<$filter> for crate::peripherals::$inst {
            type Interrupt = crate::interrupt::typelevel::$irq;
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
        // handle this filter's status/ovr/regular-conversion bits only
    }
}

// pub struct State {
//     pub waker: AtomicWaker,
//     pub injected_done: core::sync::atomic::AtomicBool,
// }

// impl State {
//     pub const fn new() -> Self {
//         Self {
//             waker: AtomicWaker::new(),
//             injected_done: core::sync::atomic::AtomicBool::new(false),
//         }
//     }
// }

// fn common_regs() -> crate::pac::adccommon::AdcCommon {
//     return crate::pac::$common_inst
// }

// fn state() -> &'static State {
//     static STATE: State = State::new();
//     &STATE
// }

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
        #[repr(u8)]
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum $enum {
            $(

                #[doc = concat!($channel_string, " channel identifier ", stringify!($index), ".")]
                $channel = $index,
            )+
        }

        impl $enum {

            /// Index of the channel
            pub const fn index(self) -> u8 {
                self as u8
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

dma_trait!(Dma, Instance, FilterMarker);

pub trait FilterTrait<F: FilterMarker>: sealed::SealedFilterChannelTrait<F> {
    fn process(&mut self);

    fn filter(&self) -> FilterChannel {
        F::CHANNEL
    }
}
pub trait TransceiverTrait<T: TransceiverMarker>: sealed::SealedTransceiverChannelTrait<T> {
    fn process(&mut self);

    fn transceiver(&self) -> TransceiverChannel {
        T::CHANNEL
    }

    fn index(&self) -> u8 {
        T::CHANNEL.index()
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
    ) -> Transceiver<T, M, ExternalSource>
    where
        T: Instance,
        M: TransceiverMarker,
    {
        set_as_af!(ckin, AfType::input(Pull::None));
        set_as_af!(datin, AfType::input(Pull::None));

        Transceiver {
            _instance_marker: PhantomData,
            _transceiver_marker: PhantomData,
            _datasource_marker: PhantomData,
        }
    }

    /// Configure [`Transceiver`]. as receiving data from a manchester-coded signal
    pub fn new_manchester(self, datin: Peri<if_afio!(impl DatinPin<T, M, A>)>) -> Transceiver<T, M, ExternalSource>
    where
        T: Instance,
        C: ClockOutputMode,
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
    fn process(&mut self) {
        todo!()
    }

    fn transceiver(&self) -> TransceiverChannel {
        <M as TransceiverMarker>::CHANNEL
    }
}
