//! Digital Filter and Sigma-Delta Modulator (DFSDM)

#![macro_use]

use core::marker::PhantomData;

use embassy_hal_internal::PeripheralType;
use embassy_sync::waitqueue::AtomicWaker;

use crate::gpio::{AfType, Flex, OutputType, Pull, Speed};
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

    // type Split<C: ClockOutputMode>;

    // fn split<C: ClockOutputMode>(dfsdm: Dfsdm<Self, C>) -> Self::Split<C>
    // where
    //     Self: Sized;
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
    pub trait TransceiverCount: super::Shape {
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

mod sealed_shape {
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
impl sealed_shape::Sealed for capability::Tcv8 {}

impl Shape for capability::Tcv2 {
    type Selectors<T: Instance> = ChannelSelectors2<T>;
    fn selectors<T: Instance>() -> Self::Selectors<T> {
        ChannelSelectors2::new()
    }
}

impl Shape for capability::Tcv8 {
    type Selectors<T: Instance> = ChannelSelectors8<T>;
    fn selectors<T: Instance>() -> Self::Selectors<T> {
        ChannelSelectors8::new()
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
        $(,)?// split: $split_ty:ident $(,)?        
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

            // type Split<C: ClockOutputMode> = $split_ty<Self, C>;

            // fn split<C: ClockOutputMode>(dfsdm: Dfsdm<'_, Self, C>) -> Self::Split<C> {
            //     $split_ty::new(&dfsdm)
            // }
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
            // split: TransceiverChannelBuilders8,
        );
    };
    ($inst:ident, dfsdm, DFSDM_8CH_8FLT, FLT0, $irq:ident) => {
        impl_dfsdm_instance!($inst,
            repr: crate::pac::dfsdm::Dfsdm8ch8flt,
            transceivers: capability::Tcv8,
            filters: capability::Flt8,
            delay: capability::NoDelay,
            version: capability::NoVersion,
            // split: TransceiverChannelBuilders8,
        );
    };
    ($inst:ident, dfsdm, DFSDM_2CH_1FLT, FLT0, $irq:ident) => {
        impl_dfsdm_instance!($inst,
            repr: crate::pac::dfsdm::Dfsdm2ch1flt,
            transceivers: capability::Tcv2,
            filters: capability::Flt1,
            delay: capability::NoDelay,
            version: capability::NoVersion,
            // split: TransceiverChannelBuilders2,
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

/// [`Transceiver`]
/// Configured DFSDM data input transceiver.
pub struct Transceiver<'a, 'd, T, M, MODE>
where
    T: Instance,
    M: TransceiverMarker + NextChannelForInstance<T>,
    MODE: ChannelMode,
{
    _instance_marker: PhantomData<T>,
    _transceiver_marker: PhantomData<M>,
    _datasource_marker: PhantomData<MODE>,
    pub(crate) datin: Option<Flex<'d>>,
    pub(crate) ckin: Option<Flex<'d>>,
    pub(crate) common: Option<&'a DfsdmCommon<'d, T>>,
    // ckin_pin: Option<Flex<'d>>,
    // data_pin: Flex<'d>,
}

impl<'a, 'd, T, M, MODE> sealed::SealedTransceiverChannelTrait<M> for Transceiver<'a, 'd, T, M, MODE>
where
    T: Instance,
    M: TransceiverMarker + NextChannelForInstance<T>,
    MODE: ChannelMode,
{
}

impl<'a, 'd, T, M, MODE> TransceiverTrait<M> for Transceiver<'a, 'd, T, M, MODE>
where
    T: Instance,
    M: TransceiverMarker + NextChannelForInstance<T>,
    MODE: ChannelMode,
{
}

mod sealed_mode {
    pub trait Sealed {}
}

/// Operational mode of a built [`Transceiver`]. Determined by which builder
/// constructor was used; encodes the SITP/SPICKSEL/CHINSEL/DATMPX semantics.
pub trait ChannelMode: sealed_mode::Sealed {}

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

macro_rules! impl_mode {
    ($($m:ident),*) => {
        $( impl sealed_mode::Sealed for $m {} impl ChannelMode for $m {} )*
    };
}
impl_mode!(
    SpiExtMode,
    SpiCkoutMode,
    ManchesterMode,
    PdmLeftMode,
    PdmRightMode,
    ParallelDmaMode,
    ParallelAdcMode
);

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

//TODO EXAMPLE FOR NEIGHBORS USE
impl<'a, 'd, T, M, MODE> Transceiver<'a, 'd, T, M, MODE>
where
    T: Instance,
    M: TransceiverMarker + NextChannelForInstance<T>,
    MODE: ChannelMode,
{
    /// Configures the neighbor channel.
    /// Notice: No extra arguments! Rust knows `T` from `self`.
    pub fn do_something_with_neighbor(&self) {
        // The compiler knows that `M::Next` is valid for this specific instance `T`.
        let next_idx = <M::Next as TransceiverMarker>::CHANNEL.index();
    }
}

// ============================================================
// Pin presence markers (PinSet)
// ============================================================

mod sealed_pinset {
    pub trait Sealed {}
}

/// Type-level pin presence of one channel. Exactly three states exist;
/// "clock without data" has no representative and is therefore inexpressible.
#[allow(private_bounds)]
pub trait PinSet: sealed_pinset::Sealed {
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

impl sealed_pinset::Sealed for NoPins {}
impl PinSet for NoPins {
    const HAS_DATA: bool = false;
    const HAS_CLK: bool = false;
    type Datin<'d> = ();
    type Ckin<'d> = ();
}

impl sealed_pinset::Sealed for DataOnly {}
impl PinSet for DataOnly {
    const HAS_DATA: bool = true;
    const HAS_CLK: bool = false;
    type Datin<'d> = Flex<'d>;
    type Ckin<'d> = ();
}

impl sealed_pinset::Sealed for DataClk {}
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
    _m: PhantomData<(T, M)>,
}

/// Token: channel gets DATIN + CKIN.
pub struct DckCfg<'d, T: Instance, M: TransceiverMarker> {
    pub(crate) datin: Flex<'d>,
    pub(crate) ckin: Flex<'d>,
    _m: PhantomData<(T, M)>,
}

/// Accepted by one `configure_pins` slot. `Presence` is the type-level pin
/// state that flows into the split.
pub trait ChannelCfg<'d, T: Instance, M: TransceiverMarker> {
    /// Pin presence of the declaring channel.
    type Presence: PinSet;
    /// Consume the token, yielding its pins in storage form
    /// (`()` for absent, `Flex` for present — no unwrapping anywhere).
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

// ============================================================
// Single-use selectors
// ============================================================

/// Per-channel pin selector. Cannot be constructed outside this module
/// (private field); handed out only inside the `configure_pins` closure,
/// one per channel, **by value**. Every method consumes `self`, so each
/// channel's pins can be declared exactly once (E0382 otherwise).
pub struct Sel<T: Instance, M: TransceiverMarker> {
    _m: PhantomData<(T, M)>,
}

impl<'d, T, M> Sel<T, M>
where
    T: Instance,
    M: TransceiverMarker,
{
    /// Declare this channel with a DATIN pin (AF set here).
    pub fn datin(self, datin: Peri<'d, if_afio!(impl DatinPin<T, M, A>)>) -> DatinCfg<'d, T, M> {
        DatinCfg {
            datin: new_pin!(datin, AfType::input(Pull::None)).unwrap(),
            _m: PhantomData,
        }
    }

    /// Declare this channel with DATIN + CKIN.
    pub fn datin_ckin(
        self,
        datin: Peri<'d, if_afio!(impl DatinPin<T, M, A>)>,
        ckin: Peri<'d, if_afio!(impl CkinPin<T, M, A>)>,
    ) -> DckCfg<'d, T, M> {
        DckCfg {
            datin: new_pin!(datin, AfType::input(Pull::None)).unwrap(),
            ckin: new_pin!(ckin, AfType::input(Pull::None)).unwrap(),
            _m: PhantomData,
        }
    }

    /// Declare this channel as pinless (same as [`NoPinsCfg`]).
    pub fn none(self) -> NoPinsCfg {
        NoPinsCfg
    }
}

/// Selectors for all channels of an 8-channel DFSDM instance.
///
/// NOTE: must never gain a `Drop` impl — partial moves out of it
/// (`s.ch0.datin(..)`) must remain legal.
pub struct ChannelSelectors8<T: Instance> {
    /// Selector for channel 0
    pub ch0: Sel<T, Tcv0>,
    /// Selector for channel 1
    pub ch1: Sel<T, Tcv1>,
    /// Selector for channel 2
    pub ch2: Sel<T, Tcv2>,
    /// Selector for channel 3
    pub ch3: Sel<T, Tcv3>,
    /// Selector for channel 4
    pub ch4: Sel<T, Tcv4>,
    /// Selector for channel 5
    pub ch5: Sel<T, Tcv5>,
    /// Selector for channel 6
    pub ch6: Sel<T, Tcv6>,
    /// Selector for channel 7
    pub ch7: Sel<T, Tcv7>,
}

impl<T: Instance> ChannelSelectors8<T> {
    pub(crate) fn new() -> Self {
        Self {
            ch0: Sel { _m: PhantomData },
            ch1: Sel { _m: PhantomData },
            ch2: Sel { _m: PhantomData },
            ch3: Sel { _m: PhantomData },
            ch4: Sel { _m: PhantomData },
            ch5: Sel { _m: PhantomData },
            ch6: Sel { _m: PhantomData },
            ch7: Sel { _m: PhantomData },
        }
    }
}

/// Selectors for a 2-channel DFSDM instance.
///
/// NOTE: must never gain a `Drop` impl — partial moves out of it must remain legal.
pub struct ChannelSelectors2<T: Instance> {
    /// Selector for channel 0
    pub ch0: Sel<T, Tcv0>,
    /// Selector for channel 1
    pub ch1: Sel<T, Tcv1>,
}

impl<T: Instance> ChannelSelectors2<T> {
    pub(crate) fn new() -> Self {
        Self {
            ch0: Sel { _m: PhantomData },
            ch1: Sel { _m: PhantomData },
        }
    }
}

/// [`TransceiverBuilder`]
///
///
///
pub struct TransceiverBuilder<'d, T, M, C, S, SN>
where
    T: Instance,
    M: TransceiverMarker,
    C: ClockOutputMode,
    S: PinSet,  //Own pins
    SN: PinSet, //Neighbors pins
{
    pub(crate) datin: S::Datin<'d>,
    pub(crate) ckin: S::Ckin<'d>,
    _m: PhantomData<(T, M, C, SN)>,
}

///Marks an Instance as offering ADC-input
#[allow(private_bounds)]
pub trait AdcInput: Instance {}

//TODO afaik only DFSDM1 offers ADC connection
foreach_peripheral! {
    (dfsdm, DFSDM1) => {
        impl AdcInput for crate::peripherals::DFSDM1 {}
    };
}

impl<'d, T, M, C, S, SN> TransceiverBuilder<'d, T, M, C, S, SN>
where
    T: Instance,
    M: TransceiverMarker + NextChannelForInstance<T>,
    C: ClockOutputMode,
    S: PinSet,
    SN: PinSet,
{
    /// Parallel input from ADC writes to CHyDATINR (DATMPX=2).
    /// No CKOUT, no pins needed. Serial pins declared on this channel
    /// are disconnected (the builder's Flexes drop here — they're unused
    /// in this mode).
    pub fn new_parallel_adc(self) -> Transceiver<'static, 'd, T, M, ParallelAdcMode>
    where
        T: AdcInput,
    {
        todo!()
    }

    ///Same as [`Self::new_parallel_adc`] but using neighbors pins
    pub fn new_parallel_adc_neighbor(self) -> Transceiver<'static, 'd, T, M, ParallelAdcMode>
    where
        T: AdcInput,
    {
        todo!()
    }

    /// Parallel input from CPU/DMA writes to CHyDATINR (DATMPX=2).
    /// No CKOUT, no pins needed. Serial pins declared on this channel
    /// are disconnected (the builder's Flexes drop here — they're unused
    /// in this mode).
    pub fn new_parallel_dma(self) -> Transceiver<'static, 'd, T, M, ParallelDmaMode> {
        todo!()
    }

    ///Same as [`Self::new_parallel_dma`] but using neighbors pins
    pub fn new_parallel_dma_neighbor(self) -> Transceiver<'static, 'd, T, M, ParallelDmaMode> {
        todo!()
    }

    /// Parallel input from ADC writes to CHyDATINR (DATMPX=2).
    /// No CKOUT, no pins needed. Serial pins declared on this channel
    /// are disconnected (the builder's Flexes drop here — they're unused
    /// in this mode).
    pub fn new_manchester(self) -> Transceiver<'static, 'd, T, M, ManchesterMode>
    where
        S: HasData,
    {
        todo!()
    }

    ///Same as [`Self::new_manchester`] but using neighbors pins
    pub fn new_manchester_neighbor(self) -> Transceiver<'static, 'd, T, M, ManchesterMode>
    where
        SN: HasData,
    {
        todo!()
    }

    ///TODO new_synchronous_int description
    pub fn new_synchronous_int(self) -> Transceiver<'static, 'd, T, M, SpiExtMode>
    where
        S: HasDataAndClk,
    {
        todo!()
    }

    ///Same as [`Self::new_synchronous_int`] but using neighbors pins
    pub fn new_synchronous_int_neighbor(self) -> Transceiver<'static, 'd, T, M, SpiExtMode>
    where
        SN: HasDataAndClk,
    {
        todo!()
    }
}

impl<'d, T, M, S, SN> TransceiverBuilder<'d, T, M, OutputEnabled, S, SN>
where
    T: Instance,
    M: TransceiverMarker + NextChannelForInstance<T>,
    S: PinSet,
    SN: PinSet,
{
    ///TODO new_synchronous_ext description
    pub fn new_synchronous_ext(self) -> Transceiver<'static, 'd, T, M, SpiExtMode>
    where
        S: HasData,
    {
        todo!()
    }

    ///Same as [`Self::new_synchronous_ext`] but using neighbors pins
    pub fn new_synchronous_ext_neighbor(self) -> Transceiver<'static, 'd, T, M, SpiExtMode>
    where
        SN: HasData,
    {
        todo!()
    }
}

/// Used to configure a [`Transceiver`].
// pub struct TransceiverBuilder<T, M, C>
// where
//     T: Instance,
//     M: TransceiverMarker,
//     C: ClockOutputMode,
// {
//     _dfsdm_marker: PhantomData<T>,
//     _clockmode_marker: PhantomData<C>,
//     _transceiver_marker: PhantomData<M>,
// }

// impl<T, M, C> TransceiverBuilder<T, M, C>
// where
//     T: Instance,
//     M: TransceiverMarker,
//     C: ClockOutputMode,
// {
//     pub(crate) fn new(_dfsdm: &Dfsdm<T, C>) -> Self {
//         //TODO Remove dfsdm reference maybe
//         Self {
//             _dfsdm_marker: PhantomData,
//             _clockmode_marker: PhantomData,
//             _transceiver_marker: PhantomData,
//         }
//     }

//     /// Configure [`Transceiver`]. as receiving data from its internal register
//     pub fn new_parallel(self) -> Transceiver<T, M, InternalSource> {
//         // TODO
//         Transceiver {
//             _instance_marker: PhantomData,
//             _transceiver_marker: PhantomData,
//             _datasource_marker: PhantomData,
//         }
//     }

//     /// Configure [`Transceiver`]. as receiving data from an externally clocked bus
//     pub fn new_clk_ext(
//         self,
//         ckin: Peri<if_afio!(impl CkinPin<T, M, A>)>,
//         datin: Peri<if_afio!(impl DatinPin<T, M, A>)>,
//     ) -> Transceiver<T, M, ExternalSource> {
//         set_as_af!(ckin, AfType::input(Pull::None));
//         set_as_af!(datin, AfType::input(Pull::None));

//         Transceiver {
//             _instance_marker: PhantomData,
//             _transceiver_marker: PhantomData,
//             _datasource_marker: PhantomData,
//         }
//     }

//     /// Configure [`Transceiver`]. as receiving data from a manchester-coded signal
//     pub fn new_manchester(self, datin: Peri<if_afio!(impl DatinPin<T, M, A>)>) -> Transceiver<T, M, ExternalSource> {
//         set_as_af!(datin, AfType::input(Pull::None));

//         Transceiver {
//             _instance_marker: PhantomData,
//             _transceiver_marker: PhantomData,
//             _datasource_marker: PhantomData,
//         }
//     }
// }

// impl<T, M> TransceiverBuilder<T, M, OutputEnabled>
// where
//     T: Instance,
//     M: TransceiverMarker,
// {
//     /// Configure transceiver as receiving data from a synchronously clocked bus clocked by the controller
//     pub fn new_clk_int(self, datin: Peri<if_afio!(impl DatinPin<T, M, A>)>) -> Transceiver<T, M, ExternalSource>
//     where
//         T: Instance,
//         M: TransceiverMarker,
//     {
//         set_as_af!(datin, AfType::input(Pull::None));

//         Transceiver {
//             _instance_marker: PhantomData,
//             _transceiver_marker: PhantomData,
//             _datasource_marker: PhantomData,
//         }
//     }
// }

/// Splits
///
///

pub struct FilterBuilder<T: Instance, F: FilterMarker> {
    _m: PhantomData<(T, F)>,
}

/// Holds regerences to the peripheral and the optional clock-output. Disables the RCC of the peripheral when dropped.
pub struct DfsdmCommon<'d, T: Instance> {
    pub(crate) _peri: Peri<'d, T>,
    /// `Some` iff the driver was created via [`Dfsdm::new_ckout`].
    pub(crate) _ckout: Option<Flex<'d>>,
}

impl<'d, T> Drop for DfsdmCommon<'d, T>
where
    T: Instance,
{
    fn drop(&mut self) {
        T::RCC_INFO.disable();
    }
}

pub struct DfsdmSplit2<'d, T, C, S0, S1>
where
    T: Instance,
    C: ClockOutputMode,
    S0: PinSet,
    S1: PinSet,
{
    pub common: DfsdmCommon<'d, T>,
    pub ch0: TransceiverBuilder<'d, T, Tcv0, C, S0, S1>, // neighbor = ch1
    pub ch1: TransceiverBuilder<'d, T, Tcv1, C, S1, S0>, // neighbor = ch0 (wrap!)
    pub flt0: FilterBuilder<T, Flt0>,
}

pub struct DfsdmSplit8<'d, T, C, S0, S1, S2, S3, S4, S5, S6, S7>
where
    T: Instance,
    C: ClockOutputMode,
    S0: PinSet,
    S1: PinSet,
    S2: PinSet,
    S3: PinSet,
    S4: PinSet,
    S5: PinSet,
    S6: PinSet,
    S7: PinSet,
{
    pub common: DfsdmCommon<'d, T>,
    pub ch0: TransceiverBuilder<'d, T, Tcv0, C, S0, S1>,
    pub ch1: TransceiverBuilder<'d, T, Tcv1, C, S1, S2>,
    pub ch2: TransceiverBuilder<'d, T, Tcv2, C, S2, S3>,
    pub ch3: TransceiverBuilder<'d, T, Tcv3, C, S3, S4>,
    pub ch4: TransceiverBuilder<'d, T, Tcv4, C, S4, S5>,
    pub ch5: TransceiverBuilder<'d, T, Tcv5, C, S5, S6>,
    pub ch6: TransceiverBuilder<'d, T, Tcv6, C, S6, S7>,
    pub ch7: TransceiverBuilder<'d, T, Tcv7, C, S7, S0>, // neighbor = ch0 (wrap!)
    pub flt0: FilterBuilder<T, Flt0>,
    pub flt1: FilterBuilder<T, Flt1>,
    pub flt2: FilterBuilder<T, Flt2>,
    pub flt3: FilterBuilder<T, Flt3>,
    pub flt4: FilterBuilder<T, Flt4>,
    pub flt5: FilterBuilder<T, Flt5>,
    pub flt6: FilterBuilder<T, Flt6>,
    pub flt7: FilterBuilder<T, Flt7>,
}

/// Dfsdm
///
///

/// DFSDM driver.
pub struct Dfsdm<'d, T: Instance, C: ClockOutputMode> {
    _instance_marker: PhantomData<T>,
    _clock_mode: PhantomData<C>,
    peri: Option<Peri<'d, T>>,
    ckout: Option<Flex<'d>>,
}

impl<'d, T, C> Dfsdm<'d, T, C>
where
    T: Instance,
    C: ClockOutputMode,
{
}

// impl<'d, T: Instance, C: ClockOutputMode> Dfsdm<'d, T, C> {
//     pub fn split(self) -> T::Split<C> {
//         T::split(self)
//     }
// }

#[allow(private_bounds)]
impl<'d, T, C> Dfsdm<'d, T, C>
where
    C: ClockOutputMode,
    T: Instance<Transceivers = capability::Tcv8, Filters = capability::Flt8>,
{
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
            ckout: ckout,
            peri: Some(peri),
        }
    }
}

/// Implemented for the tuple a `configure_pins` closure returns.
/// The arity *is* the channel-count check: `(C0, C1)` only impls for
/// `Tcv2` instances, the 8-tuple only for `Tcv8`.
pub trait ChannelCfgTuple<'d, T: Instance, C: ClockOutputMode> {
    /// The fully-wired split (neighbor pin-sets already correct).
    type Split;
    fn split_parts(self, common: DfsdmCommon<'d, T>) -> Self::Split;
}

impl<'d, T, C, C0, C1> ChannelCfgTuple<'d, T, C> for (C0, C1)
where
    T: Instance<Transceivers = capability::Tcv2>,
    C: ClockOutputMode,
    C0: ChannelCfg<'d, T, Tcv0>,
    C1: ChannelCfg<'d, T, Tcv1>,
{
    type Split =
        DfsdmSplit2<'d, T, C, <C0 as ChannelCfg<'d, T, Tcv0>>::Presence, <C1 as ChannelCfg<'d, T, Tcv1>>::Presence>;

    fn split_parts(self, common: DfsdmCommon<'d, T>) -> Self::Split {
        let (d0, k0) = self.0.into_parts();
        let (d1, k1) = self.1.into_parts();
        DfsdmSplit2 {
            common,
            ch0: TransceiverBuilder {
                datin: d0,
                ckin: k0,
                _m: PhantomData,
            },
            ch1: TransceiverBuilder {
                datin: d1,
                ckin: k1,
                _m: PhantomData,
            },
            flt0: FilterBuilder { _m: PhantomData },
        }
    }
}

impl<'d, T, C, C0, C1, C2, C3, C4, C5, C6, C7> ChannelCfgTuple<'d, T, C> for (C0, C1, C2, C3, C4, C5, C6, C7)
where
    T: Instance<Transceivers = capability::Tcv8>,
    C: ClockOutputMode,
    C0: ChannelCfg<'d, T, Tcv0>,
    C1: ChannelCfg<'d, T, Tcv1>,
    C2: ChannelCfg<'d, T, Tcv2>,
    C3: ChannelCfg<'d, T, Tcv3>,
    C4: ChannelCfg<'d, T, Tcv4>,
    C5: ChannelCfg<'d, T, Tcv5>,
    C6: ChannelCfg<'d, T, Tcv6>,
    C7: ChannelCfg<'d, T, Tcv7>,
{
    type Split = DfsdmSplit8<
        'd,
        T,
        C,
        <C0 as ChannelCfg<'d, T, Tcv0>>::Presence,
        <C1 as ChannelCfg<'d, T, Tcv1>>::Presence,
        <C2 as ChannelCfg<'d, T, Tcv2>>::Presence,
        <C3 as ChannelCfg<'d, T, Tcv3>>::Presence,
        <C4 as ChannelCfg<'d, T, Tcv4>>::Presence,
        <C5 as ChannelCfg<'d, T, Tcv5>>::Presence,
        <C6 as ChannelCfg<'d, T, Tcv6>>::Presence,
        <C7 as ChannelCfg<'d, T, Tcv7>>::Presence,
    >;

    fn split_parts(self, common: DfsdmCommon<'d, T>) -> Self::Split {
        let (d0, k0) = self.0.into_parts();
        let (d1, k1) = self.1.into_parts();
        let (d2, k2) = self.2.into_parts();
        let (d3, k3) = self.3.into_parts();
        let (d4, k4) = self.4.into_parts();
        let (d5, k5) = self.5.into_parts();
        let (d6, k6) = self.6.into_parts();
        let (d7, k7) = self.7.into_parts();
        DfsdmSplit8 {
            common: common,
            ch0: TransceiverBuilder {
                datin: d0,
                ckin: k0,
                _m: PhantomData,
            },
            ch1: TransceiverBuilder {
                datin: d1,
                ckin: k1,
                _m: PhantomData,
            },
            ch2: TransceiverBuilder {
                datin: d2,
                ckin: k2,
                _m: PhantomData,
            },
            ch3: TransceiverBuilder {
                datin: d3,
                ckin: k3,
                _m: PhantomData,
            },
            ch4: TransceiverBuilder {
                datin: d4,
                ckin: k4,
                _m: PhantomData,
            },
            ch5: TransceiverBuilder {
                datin: d5,
                ckin: k5,
                _m: PhantomData,
            },
            ch6: TransceiverBuilder {
                datin: d6,
                ckin: k6,
                _m: PhantomData,
            },
            ch7: TransceiverBuilder {
                datin: d7,
                ckin: k7,
                _m: PhantomData,
            },
            flt0: FilterBuilder { _m: PhantomData },
            flt1: FilterBuilder { _m: PhantomData },
            flt2: FilterBuilder { _m: PhantomData },
            flt3: FilterBuilder { _m: PhantomData },
            flt4: FilterBuilder { _m: PhantomData },
            flt5: FilterBuilder { _m: PhantomData },
            flt6: FilterBuilder { _m: PhantomData },
            flt7: FilterBuilder { _m: PhantomData },
        }
    }
}

impl<'d, T, C> Dfsdm<'d, T, C>
where
    T: Instance,
    C: ClockOutputMode,
{
    /// The closure receives one selector per channel the instance
    /// actually has, and returns one token per channel, as a tuple.
    pub fn configure_pins<F, OUT>(mut self, f: F) -> <OUT as ChannelCfgTuple<'d, T, C>>::Split
    where
        F: FnOnce(<T::Transceivers as Shape>::Selectors<T>) -> OUT,
        OUT: ChannelCfgTuple<'d, T, C>,
    {
        let out = f(<T::Transceivers as Shape>::selectors::<T>());
        out.split_parts(DfsdmCommon {
            _peri: self.peri.take().expect("taken once"),
            _ckout: self.ckout.take(),
        })
    }
}

/// Playground
pub fn test<T>(f: impl FnOnce(ChannelSelectors8<T>))
where
    T: Instance,
{
    f(ChannelSelectors8::<T>::new())
}
