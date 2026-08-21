//! Digital Filter and Sigma-Delta Modulator (DFSDM)
use core::marker::PhantomData;

use embassy_hal_internal::PeripheralType;
use embassy_sync::waitqueue::AtomicWaker;

use crate::gpio::{AfType, Flex, OutputType, Pull, Speed};
use crate::{Peri, interrupt, rcc};

/// Interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _marker: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        //TODO THIS IS DCMI STUFF
        crate::pac::DFSDM1.ch(0);
        crate::pac::DFSDM2.ch(0);
        let ris = crate::pac::DCMI.ris().read();
        if ris.err_ris() {
            trace!("DCMI IRQ: Error.");
            crate::pac::DCMI.ier().modify(|ier| ier.set_err_ie(false));
        }
        if ris.ovr_ris() {
            trace!("DCMI IRQ: Overrun.");
            crate::pac::DCMI.ier().modify(|ier| ier.set_ovr_ie(false));
        }
        if ris.frame_ris() {
            trace!("DCMI IRQ: Frame captured.");
            crate::pac::DCMI.ier().modify(|ier| ier.set_frame_ie(false));
        }
        STATE.waker.wake();
    }
}

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

macro_rules! config_pins {
    ($($pin:ident),*) => {
                critical_section::with(|_| {
            $(
                set_as_af!($pin, AfType::input(Pull::None));
            )*
        })
    };
}

pub trait ClockMode {}

pub struct InternalClock;
pub struct ExternalClock;

impl ClockMode for InternalClock {}
impl ClockMode for ExternalClock {}

/// DFSDM driver.
pub struct Dfsdm<'d, T: Instance, C: ClockMode> {
    inner: Peri<'d, T>,
    _clock_mode: PhantomData<C>,
    _ckout: Option<Flex<'d>>,
}

/// Just a wrapper to make this horde of Channels more convenient
pub struct TransceiverChannelBuilders8<T: Instance, C: ClockMode> {
    pub ch0: TransceiverChannelBuilder<T, Tcv0, C>,
    pub ch1: TransceiverChannelBuilder<T, Tcv1, C>,
    pub ch2: TransceiverChannelBuilder<T, Tcv2, C>,
    pub ch3: TransceiverChannelBuilder<T, Tcv3, C>,
    pub ch4: TransceiverChannelBuilder<T, Tcv4, C>,
    pub ch5: TransceiverChannelBuilder<T, Tcv5, C>,
    pub ch6: TransceiverChannelBuilder<T, Tcv6, C>,
    pub ch7: TransceiverChannelBuilder<T, Tcv7, C>,
}

/// Just a wrapper to make this horde of Channels more convenient
pub struct TransceiverChannelBuilders2<T: Instance, C: ClockMode> {
    pub ch0: TransceiverChannelBuilder<T, Tcv0, C>,
    pub ch1: TransceiverChannelBuilder<T, Tcv1, C>,
}

impl<'d, T, C> Dfsdm<'d, T, C>
where
    T: Instance,
    C: ClockMode,
{
}
trait TransChNr2 {}
trait TransChNr8 {}
trait FltChNr8 {}
trait FltChNr1 {}

impl TransChNr2 for crate::pac::dfsdm::Dfsdm2ch1flt {}
impl TransChNr8 for crate::pac::dfsdm::Dfsdm8ch8fltDly {}
impl TransChNr8 for crate::pac::dfsdm::Dfsdm8ch8fltDlyVer {}
impl FltChNr1 for crate::pac::dfsdm::Dfsdm2ch1flt {}
impl FltChNr8 for crate::pac::dfsdm::Dfsdm8ch8fltDly {}
impl FltChNr8 for crate::pac::dfsdm::Dfsdm8ch8fltDlyVer {}

trait HasTransChNr8: Instance {}
trait HasTransChNr2: Instance {}
trait HasFltChNr1 {}
trait HasFltChNr8 {}

impl<T> HasTransChNr8 for T
where
    T: Instance,
    T::Repr: TransChNr8,
{
}
impl<T> HasTransChNr2 for T
where
    T: Instance,
    T::Repr: TransChNr2,
{
}
impl<T> HasFltChNr1 for T
where
    T: Instance,
    T::Repr: FltChNr1,
{
}
impl<T> HasFltChNr8 for T
where
    T: Instance,
    T::Repr: FltChNr8,
{
}

#[allow(private_bounds)]
impl<'d, T, C> Dfsdm<'d, T, C>
where
    C: ClockMode,
    T: HasTransChNr8 + HasFltChNr8,
{
    /// Split up the peripheral after first initialization
    pub fn split_8ch_8flt(self) -> TransceiverChannelBuilders8<T, C> {
        TransceiverChannelBuilders8 {
            ch0: TransceiverChannelBuilder::new(&self),
            ch1: TransceiverChannelBuilder::new(&self),
            ch2: TransceiverChannelBuilder::new(&self),
            ch3: TransceiverChannelBuilder::new(&self),
            ch4: TransceiverChannelBuilder::new(&self),
            ch5: TransceiverChannelBuilder::new(&self),
            ch6: TransceiverChannelBuilder::new(&self),
            ch7: TransceiverChannelBuilder::new(&self),
        }
    }
}

#[allow(private_bounds)]
impl<'d, T, C> Dfsdm<'d, T, C>
where
    C: ClockMode,
    T: HasTransChNr2 + HasFltChNr1,
{
    /// Split up the peripheral after first initialization
    pub fn split_2ch_1flt(self) -> TransceiverChannelBuilders2<T, C> {
        TransceiverChannelBuilders2 {
            ch0: TransceiverChannelBuilder::new(&self),
            ch1: TransceiverChannelBuilder::new(&self),
        }
    }
}

impl<'d, T> Dfsdm<'d, T, InternalClock>
where
    T: Instance,
{
    pub fn new_ckout(peri: Peri<'d, T>, ckout: Peri<'d, if_afio!(impl CkoutPin<T, A>)>, config: Config) -> Self {
        let _ = config;
        let ckout = new_pin!(ckout, AfType::output(OutputType::PushPull, Speed::VeryHigh));
        Self {
            inner: peri,
            _clock_mode: PhantomData,
            _ckout: ckout,
        }
    }
}

impl<'d, T> Dfsdm<'d, T, ExternalClock>
where
    T: Instance,
{
    pub fn new(peri: Peri<'d, T>, config: Config) -> Self {
        let _ = config;
        Self {
            inner: peri,
            _clock_mode: PhantomData,
            _ckout: None,
        }
    }
}

impl<'d, T, C> Dfsdm<'d, T, C>
where
    T: Instance,
    C: ClockMode,
{
    // /// Generic new
    // pub fn new<D, M>(
    //     peri: Peri<'d, T>,
    //     dma: Peri<'d, D>,
    //     _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>>
    //     + interrupt::typelevel::Binding<D::Interrupt, crate::dma::InterruptHandler<D>>
    //     + 'd,
    //     d0: Peri<'d, impl Ckin0Pin<T>>,
    //     config: Config,
    // ) -> Self
    // where
    //     D: Dma<T, M>,
    //     M: FilterChannel,
    // {
    //     config_pins!(d0);

    //     Self::new_inner(peri, dma, _irq, config, false, 0b00)
    // }

    // fn new_inner<D, M>(
    //     peri: Peri<'d, T>,
    //     dma: Peri<'d, D>,
    //     irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>>
    //     + interrupt::typelevel::Binding<D::Interrupt, crate::dma::InterruptHandler<D>>
    //     + 'd,
    //     config: Config,
    //     use_embedded_synchronization: bool,
    //     edm: u8,
    // ) -> Self
    // where
    //     D: Dma<T, M>,
    //     M: FilterChannel,
    // {
    //     rcc::enable_and_reset::<T>();
    //     //TODO configure and enable
    //     T::Interrupt::unpend();
    //     unsafe { T::Interrupt::enable() };

    //     Self {
    //         inner: peri,
    //         _ckout: None,
    //     }
    // }
}

trait SealedInstance: crate::rcc::RccPeripheral {
    // fn regs(&self) -> crate::pac::dfsdm::Dfsdm;
    fn transceiver_channels() -> u8;
    fn filter_channels() -> u8;
}

/// DFSDM instance.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + 'static {
    /// Interrupt for this instance.
    type Interrupt: interrupt::typelevel::Interrupt;
    type Repr;
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
/// [`TransceiverChannel`]. This allows the compiler to reject pin
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

// allow unused as U5 sources do not contain interrupt nor dma data
#[allow(unused)]
macro_rules! impl_peripheral_8ch_8flt_dly {
    ($inst:ident, $irq:ident) => {
        impl SealedInstance for crate::peripherals::$inst {
            // fn regs(&self) -> crate::pac::dfsdm::Dfsdm8ch8fltDly {
            //     crate::pac::$inst
            // }
            fn transceiver_channels() -> u8 {
                8
            }
            fn filter_channels() -> u8 {
                8
            }
        }

        impl Instance for crate::peripherals::$inst {
            type Interrupt = crate::interrupt::typelevel::$irq;
            type Repr = crate::pac::dfsdm::Dfsdm8ch8fltDly;
        }
    };
}
#[allow(unused)]
macro_rules! impl_peripheral_8ch_8flt {
    ($inst:ident, $irq:ident) => {
        impl SealedInstance for crate::peripherals::$inst {
            // fn regs(&self) -> crate::pac::dfsdm::Dfsdm8ch8fltDly {
            //     crate::pac::$inst
            // }
            fn transceiver_channels() -> u8 {
                8
            }
            fn filter_channels() -> u8 {
                8
            }
        }

        impl Instance for crate::peripherals::$inst {
            type Interrupt = crate::interrupt::typelevel::$irq;
            type Repr = crate::pac::dfsdm::Dfsdm8ch8flt;
        }
    };
}
#[allow(unused)]
macro_rules! impl_peripheral_2ch_1flt {
    ($inst:ident, $irq:ident) => {
        impl SealedInstance for crate::peripherals::$inst {
            // fn regs(&self) -> crate::pac::dfsdm::Dfsdm2ch1flt {
            //     crate::pac::$inst
            // }

            fn transceiver_channels() -> u8 {
                2
            }
            fn filter_channels() -> u8 {
                1
            }
        }

        impl Instance for crate::peripherals::$inst {
            type Interrupt = crate::interrupt::typelevel::$irq;
            type Repr = crate::pac::dfsdm::Dfsdm2ch1flt;
        }
    };
}

// TODO
// foreach_interrupt! {
//     ($inst:ident, dfsdm, $block:ident, FLT{ Transceiver::Tcv0,  },$irq:ident) => {
//         impl_peripheral!($inst, $irq);
//     };
// }
foreach_interrupt! {
    ($inst:ident, dfsdm, DFSDM_8CH_8FLT_DLY, FLT0, $irq:ident) => {
        impl_peripheral_8ch_8flt_dly!($inst, $irq);
    };
    ($inst:ident, dfsdm, DFSDM_8CH_8FLT, FLT0, $irq:ident) => {
        impl_peripheral_8ch_8flt!($inst, $irq);
    };
    ($inst:ident, dfsdm, DFSDM_2CH_1FLT, FLT0, $irq:ident) => {
        impl_peripheral_2ch_1flt!($inst, $irq);
    };
}

macro_rules! define_channels {
    (
        $enum:ident,
        $marker_trait:ident,
        $sealed_trait:path,
        $(
            $channel:ident
        ),+ $(,)?
    ) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum $enum {
            $(
                $channel,
            )+
        }

        pub trait $marker_trait: $sealed_trait {
            const FILTER: $enum;
        }

        $(
            pub struct $channel;

            impl $sealed_trait for $channel {}

            impl $marker_trait for $channel {
                const FILTER: $enum = $enum::$channel;
            }
        )+
    };
}

define_channels!(
    Transceiver,
    TransceiverMarker,
    sealed::SealedTransceiver,
    Tcv0,
    Tcv1,
    Tcv2,
    Tcv3,
    Tcv4,
    Tcv5,
    Tcv6,
    Tcv7,
);
define_channels!(
    Filter,
    FilterMarker,
    sealed::SealedFilter,
    Flt0,
    Flt1,
    Flt2,
    Flt3,
    Flt4,
    Flt5,
    Flt6,
    Flt7,
);

dma_trait!(Dma, Instance, FilterMarker);

mod sealed {
    pub trait SealedTransceiver {}
    pub trait SealedFilter {}
    pub trait SealedFilterChannel<F: super::FilterMarker> {}
    pub trait SealedTransceiverChannel<F: super::TransceiverMarker> {}
}

pub trait FilterChannel<F: FilterMarker>: sealed::SealedFilterChannel<F> {
    fn process(&mut self);

    fn filter(&self) -> Filter {
        F::FILTER
    }
}
pub trait TransceiverChannel<T: TransceiverMarker>: sealed::SealedTransceiverChannel<T> {
    fn process(&mut self);

    fn transceiver(&self) -> Transceiver {
        T::FILTER
    }
}

/// State trait for Transceiver channels
pub trait DataSource {}

/// Transceiver get's data clock from outside
pub struct External;

/// Transceiver get's data clock from inside
pub struct Internal;

impl DataSource for External {}
impl DataSource for Internal {}

pub struct TransceiverChannelBuilder<T, M, C>
where
    T: Instance,
    M: TransceiverMarker,
    C: ClockMode,
{
    _dfsdm_marker: PhantomData<T>,
    _clockmode_marker: PhantomData<C>,
    _transceiver_marker: PhantomData<M>,
}

impl<T, M, C> TransceiverChannelBuilder<T, M, C>
where
    T: Instance,
    M: TransceiverMarker,
    C: ClockMode,
{
    pub(crate) fn new(dfsdm: &Dfsdm<T, C>) -> Self {
        Self {
            _dfsdm_marker: PhantomData,
            _clockmode_marker: PhantomData,
            _transceiver_marker: PhantomData,
        }
    }
}

pub struct TransceiverChannelImp<'d, T, M, S>
where
    T: Instance,
    M: TransceiverMarker,
    S: DataSource,
{
    _peri: Peri<'d, T>,
    _transceiver_marker: PhantomData<M>,
    _datasource_marker: PhantomData<S>,
    // ckin_pin: Option<Flex<'d>>,
    // data_pin: Flex<'d>,
}

impl<'d, T, M> TransceiverChannelImp<'d, T, M, Internal>
where
    T: Instance,
    M: TransceiverMarker,
{
    /// TODO stuff with dma and input register specifically
    pub fn new_parallel<C>(dfsdm: &Dfsdm<'d, T, C>) -> Self
    where
        C: ClockMode,
    {
        Self {
            _peri: unsafe { dfsdm.inner.clone_unchecked() },
            _transceiver_marker: PhantomData,
            _datasource_marker: PhantomData,
            // ckin_pin: ckin_pin,
            // data_pin,
        }
    }
}

impl<'d, T, M> TransceiverChannelImp<'d, T, M, External>
where
    T: Instance,
    M: TransceiverMarker,
{
    ///Create new TransceiverChannelImp for use with explicit external clock reference
    pub fn new_ext_clk(
        dfsdm: &Dfsdm<'d, T, ExternalClock>,
        ckin: Peri<'d, if_afio!(impl CkinPin<T, M, A>)>,
        datin: Peri<'d, if_afio!(impl DatinPin<T, M, A>)>,
    ) -> Self
    where
        T: Instance,
        M: TransceiverMarker,
    {
        set_as_af!(ckin, AfType::input(Pull::None));
        set_as_af!(datin, AfType::input(Pull::None));
        // let ckin_pin = new_pin!(ckin, AfType::output(OutputType::PushPull, Speed::VeryHigh));
        // let datin_pin = new_pin!(datin, AfType::output(OutputType::PushPull, Speed::VeryHigh));

        Self {
            _peri: unsafe { dfsdm.inner.clone_unchecked() },
            _transceiver_marker: PhantomData,
            _datasource_marker: PhantomData,
            // ckin_pin: ckin_pin,
            // data_pin,
        }
    }

    ///Create new TransceiverChannelImp for use with internal clock reference
    pub fn new_clk_int<C>(dfsdm: &Dfsdm<'d, T, C>, datin: Peri<'d, if_afio!(impl DatinPin<T, M, A>)>) -> Self
    where
        T: Instance,
        C: ClockMode,
        M: TransceiverMarker,
    {
        set_as_af!(datin, AfType::input(Pull::None));
        // let ckin_pin = new_pin!(ckin, AfType::output(OutputType::PushPull, Speed::VeryHigh));
        // let datin_pin = new_pin!(datin, AfType::output(OutputType::PushPull, Speed::VeryHigh));

        Self {
            _peri: unsafe { dfsdm.inner.clone_unchecked() },
            _transceiver_marker: PhantomData,
            _datasource_marker: PhantomData,
            // ckin_pin: ckin_pin,
            // data_pin,
        }
    }

    ///Create new TransceiverChannelImp for use with manchester coded data (clock recovery)
    pub fn new_manchester<C>(dfsdm: &Dfsdm<'d, T, C>, datin: Peri<'d, if_afio!(impl DatinPin<T, M, A>)>) -> Self
    where
        T: Instance,
        C: ClockMode,
        M: TransceiverMarker,
    {
        set_as_af!(datin, AfType::input(Pull::None));
        // let ckin_pin = new_pin!(ckin, AfType::output(OutputType::PushPull, Speed::VeryHigh));
        // let datin_pin = new_pin!(datin, AfType::output(OutputType::PushPull, Speed::VeryHigh));

        Self {
            _peri: unsafe { dfsdm.inner.clone_unchecked() },
            _transceiver_marker: PhantomData,
            _datasource_marker: PhantomData,
            // ckin_pin: ckin_pin,
            // data_pin,
        }
    }
}

impl<'d, T, M, S> sealed::SealedTransceiverChannel<M> for TransceiverChannelImp<'d, T, M, S>
where
    T: Instance,
    M: TransceiverMarker,
    S: DataSource,
{
}

impl<'d, T, M, S> TransceiverChannel<M> for TransceiverChannelImp<'d, T, M, S>
where
    T: Instance,
    M: TransceiverMarker,
    S: DataSource,
{
    fn process(&mut self) {
        todo!()
    }

    fn transceiver(&self) -> Transceiver {
        <M as TransceiverMarker>::FILTER
    }
}
