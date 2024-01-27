//! OCTOSPI Serial Peripheral Interface
//!

#![macro_use]

use core::ptr;

use embassy_embedded_hal::SetConfig;
use embassy_futures::join::join;
use embassy_hal_internal::{into_ref, PeripheralRef};
pub use embedded_hal_02::spi::{Mode, Phase, Polarity, MODE_0, MODE_1, MODE_2, MODE_3};

use crate::dma::{slice_ptr_parts, word, Transfer};
use crate::gpio::sealed::{AFType, Pin as _};
use crate::gpio::{AnyPin, Pull};
use crate::pac::octospi::{regs, vals, Octospi as Regs};
use crate::rcc::RccPeripheral;
use crate::time::Hertz;
use crate::{peripherals, Peripheral};

pub struct Config;

pub struct Ospi<'d, T: Instance, Dma> {
    _peri: PeripheralRef<'d, T>,
    sck: Option<PeripheralRef<'d, AnyPin>>,
    d0: Option<PeripheralRef<'d, AnyPin>>,
    d1: Option<PeripheralRef<'d, AnyPin>>,
    d2: Option<PeripheralRef<'d, AnyPin>>,
    d3: Option<PeripheralRef<'d, AnyPin>>,
    d4: Option<PeripheralRef<'d, AnyPin>>,
    d5: Option<PeripheralRef<'d, AnyPin>>,
    d6: Option<PeripheralRef<'d, AnyPin>>,
    d7: Option<PeripheralRef<'d, AnyPin>>,
    nss: Option<PeripheralRef<'d, AnyPin>>,
    dqs: Option<PeripheralRef<'d, AnyPin>>,
    dma: PeripheralRef<'d, Dma>,
    config: Config,
}

impl<'d, T: Instance, Dma> Ospi<'d, T, Dma> {
    /// Create new OSPI driver for a dualspi external chip
    pub fn new_dualspi(
        peri: impl Peripheral<P = T> + 'd,
        sck: impl Peripheral<P = impl SckPin<T>> + 'd,
        d0: impl Peripheral<P = impl D0Pin<T>> + 'd,
        d1: impl Peripheral<P = impl D1Pin<T>> + 'd,
        nss: impl Peripheral<P = impl NSSPin<T>> + 'd,
        dma: impl Peripheral<P = Dma> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(peri, sck, d0, d1, nss);

        sck.set_as_af_pull(sck.af_num(), AFType::OutputPushPull, Pull::None);
        sck.set_speed(crate::gpio::Speed::VeryHigh);
        nss.set_as_af_pull(nss.af_num(), AFType::OutputPushPull, Pull::Up);
        nss.set_speed(crate::gpio::Speed::VeryHigh);
        d0.set_as_af_pull(d0.af_num(), AFType::OutputPushPull, Pull::None);
        d0.set_speed(crate::gpio::Speed::VeryHigh);
        d1.set_as_af_pull(d1.af_num(), AFType::OutputPushPull, Pull::None);
        d1.set_speed(crate::gpio::Speed::VeryHigh);

        Self::new_inner(
            peri,
            Some(d0.map_into()),
            Some(d1.map_into()),
            None,
            None,
            None,
            None,
            None,
            None,
            Some(sck.map_into()),
            Some(nss.map_into()),
            None,
            dma,
            config,
        )
    }

    fn new_inner(
        peri: impl Peripheral<P = T> + 'd,
        d0: Option<PeripheralRef<'d, AnyPin>>,
        d1: Option<PeripheralRef<'d, AnyPin>>,
        d2: Option<PeripheralRef<'d, AnyPin>>,
        d3: Option<PeripheralRef<'d, AnyPin>>,
        d4: Option<PeripheralRef<'d, AnyPin>>,
        d5: Option<PeripheralRef<'d, AnyPin>>,
        d6: Option<PeripheralRef<'d, AnyPin>>,
        d7: Option<PeripheralRef<'d, AnyPin>>,
        sck: Option<PeripheralRef<'d, AnyPin>>,
        nss: Option<PeripheralRef<'d, AnyPin>>,
        dqs: Option<PeripheralRef<'d, AnyPin>>,
        dma: impl Peripheral<P = Dma> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(peri, dma);

        T::enable_and_reset();
        T::REGS.sr().read().busy();

        T::REGS.cr().modify(|w| {
            w.set_en(true);
        });

        #[cfg(octospi_v1)]
        {
            T::REGS.ccr().modify(|w| {
                w.set_imode(vals::PhaseMode::TWOLINES);
                w.set_admode(vals::PhaseMode::TWOLINES);
                w.set_abmode(vals::PhaseMode::TWOLINES);
                w.set_dmode(vals::PhaseMode::TWOLINES);
            });
            T::REGS.wccr().modify(|w| {
                w.set_imode(vals::PhaseMode::TWOLINES);
                w.set_admode(vals::PhaseMode::TWOLINES);
                w.set_abmode(vals::PhaseMode::TWOLINES);
                w.set_dmode(vals::PhaseMode::TWOLINES);
            });
        }

        //

        // while T::REGS::sr().read().busy() {}

        Self {
            _peri: peri,
            sck,
            d0,
            d1,
            d2,
            d3,
            d4,
            d5,
            d6,
            d7,
            nss,
            dqs,
            dma,
            config,
        }
    }
}

pub(crate) mod sealed {
    use super::*;

    pub trait Instance {
        const REGS: Regs;
    }
}

/// OSPI instance trait.
pub trait Instance: Peripheral<P = Self> + sealed::Instance + RccPeripheral {}

pin_trait!(SckPin, Instance);
pin_trait!(NckPin, Instance);
pin_trait!(D0Pin, Instance);
pin_trait!(D1Pin, Instance);
pin_trait!(D2Pin, Instance);
pin_trait!(D3Pin, Instance);
pin_trait!(D4Pin, Instance);
pin_trait!(D5Pin, Instance);
pin_trait!(D6Pin, Instance);
pin_trait!(D7Pin, Instance);
pin_trait!(DQSPin, Instance);
pin_trait!(NSSPin, Instance);

dma_trait!(OctoDma, Instance);

foreach_peripheral!(
    (octospi, $inst:ident) => {
        impl sealed::Instance for peripherals::$inst {
            const REGS: Regs = crate::pac::$inst;
        }

        impl Instance for peripherals::$inst {}
    };
);
