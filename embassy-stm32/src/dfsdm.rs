//! Digital Filter and Sigma-Delta Modulator (DFSDM)
use core::marker::PhantomData;

use embassy_hal_internal::PeripheralType;
use embassy_sync::waitqueue::AtomicWaker;

use crate::gpio::{AfType, Pull};
use crate::interrupt::typelevel::Interrupt;
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

/// DFSDM driver.
pub struct Dfsdm<'d, T: Instance> {
    inner: Peri<'d, T>,
}

impl<'d, T> Dfsdm<'d, T>
where
    T: Instance,
{
    /// Generic new
    pub fn new<D, M>(
        peri: Peri<'d, T>,
        dma: Peri<'d, D>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>>
        + interrupt::typelevel::Binding<D::Interrupt, crate::dma::InterruptHandler<D>>
        + 'd,
        d0: Peri<'d, impl Ckin0Pin<T>>,
        config: Config,
    ) -> Self
    where
        D: Dma<T, M>,
        M: FilterChannel,
    {
        config_pins!(d0);

        Self::new_inner(peri, dma, _irq, config, false, 0b00)
    }

    fn new_inner<D, M>(
        peri: Peri<'d, T>,
        dma: Peri<'d, D>,
        irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>>
        + interrupt::typelevel::Binding<D::Interrupt, crate::dma::InterruptHandler<D>>
        + 'd,
        config: Config,
        use_embedded_synchronization: bool,
        edm: u8,
    ) -> Self
    where
        D: Dma<T, M>,
        M: FilterChannel,
    {
        rcc::enable_and_reset::<T>();
        //TODO configure and enable
        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        Self { inner: peri }
    }
}

trait SealedInstance: crate::rcc::RccPeripheral {
    // fn regs(&self) -> crate::pac::dfsdm::Dfsdm;
}

/// DFSDM instance.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + 'static {
    /// Interrupt for this instance.
    type Interrupt: interrupt::typelevel::Interrupt;
}

pin_trait!(CkoutPin, Instance);
pin_trait!(Datin0Pin, Instance);
pin_trait!(Ckin0Pin, Instance);
pin_trait!(Ckin1Pin, Instance);
pin_trait!(Datin1Pin, Instance);
pin_trait!(Datin2Pin, Instance);
pin_trait!(Ckin2Pin, Instance);
pin_trait!(Datin3Pin, Instance);
pin_trait!(Ckin3Pin, Instance);
pin_trait!(Datin4Pin, Instance);
pin_trait!(Ckin4Pin, Instance);
pin_trait!(Datin5Pin, Instance);
pin_trait!(Ckin5Pin, Instance);
pin_trait!(Datin6Pin, Instance);
pin_trait!(Ckin6Pin, Instance);
pin_trait!(Datin7Pin, Instance);
pin_trait!(Ckin7Pin, Instance);

// allow unused as U5 sources do not contain interrupt nor dma data
#[allow(unused)]
macro_rules! impl_peripheral_8ch_8flt_dly {
    ($inst:ident, $irq:ident) => {
        impl SealedInstance for crate::peripherals::$inst {
            // fn regs(&self) -> crate::pac::dfsdm::Dfsdm8ch8fltDly {
            //     crate::pac::$inst
            // }
        }

        impl Instance for crate::peripherals::$inst {
            type Interrupt = crate::interrupt::typelevel::$irq;
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
        }

        impl Instance for crate::peripherals::$inst {
            type Interrupt = crate::interrupt::typelevel::$irq;
        }
    };
}

// foreach_interrupt! {
//     ($inst:ident, dfsdm, $block:ident, FLT0, $irq:ident) => {
//         impl_peripheral!($inst, $irq);
//     };
// }
foreach_interrupt! {
    ($inst:ident, dfsdm, DFSDM_8CH_8FLT_DLY, FLT0, $irq:ident) => {
        impl_peripheral_8ch_8flt_dly!($inst, $irq);
    };
    ($inst:ident, dfsdm, DFSDM_2CH_1FLT, FLT0, $irq:ident) => {
        impl_peripheral_2ch_1flt!($inst, $irq);
    };
}

dma_trait!(Dma, Instance, FilterChannel);

/// Filter channel.
#[derive(Clone, Copy)]
pub enum Filter {
    Flt0,
    Flt1,
    Flt2,
    Flt3,
    Flt4,
    Flt5,
    Flt6,
    Flt7,
}

/// Filter 0 marker type.
pub struct Flt0;
/// Filter 1 marker type.
pub struct Flt1;
/// Filter 2 marker type.
pub struct Flt2;
/// Filter 3 marker type.
pub struct Flt3;
/// Filter 4 marker type.
pub struct Flt4;
/// Filter 5 marker type.
pub struct Flt5;
/// Filter 6 marker type.
pub struct Flt6;
/// Filter 7 marker type.
pub struct Flt7;

/// DFSDM filter trait.
#[allow(private_bounds)]
pub trait FilterChannel: SealedFilterChannel {
    /// The runtime filter.
    const FILTER: Filter;
}

trait SealedFilterChannel {}

impl FilterChannel for Flt0 {
    const FILTER: Filter = Filter::Flt0;
}

impl FilterChannel for Flt1 {
    const FILTER: Filter = Filter::Flt1;
}

impl FilterChannel for Flt2 {
    const FILTER: Filter = Filter::Flt2;
}

impl FilterChannel for Flt3 {
    const FILTER: Filter = Filter::Flt3;
}

impl FilterChannel for Flt4 {
    const FILTER: Filter = Filter::Flt4;
}

impl FilterChannel for Flt5 {
    const FILTER: Filter = Filter::Flt5;
}

impl FilterChannel for Flt6 {
    const FILTER: Filter = Filter::Flt6;
}

impl FilterChannel for Flt7 {
    const FILTER: Filter = Filter::Flt7;
}

impl SealedFilterChannel for Flt0 {}
impl SealedFilterChannel for Flt1 {}
impl SealedFilterChannel for Flt2 {}
impl SealedFilterChannel for Flt3 {}
impl SealedFilterChannel for Flt4 {}
impl SealedFilterChannel for Flt5 {}
impl SealedFilterChannel for Flt6 {}
impl SealedFilterChannel for Flt7 {}
