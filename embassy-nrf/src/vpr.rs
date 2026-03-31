//! VPR coprocessor control.
#![macro_use]

use crate::{interrupt, pac};
use core::marker::PhantomData;
use embassy_hal_internal::{Peri, PeripheralType};

/// VPR driver.
///
/// VPR is coprocessor available on several nRF54 microcontrollers.
pub struct Vpr<'d> {
    r: pac::vpr::Vpr,
    _phantom: PhantomData<&'d ()>,
}

impl<'d> Vpr<'d> {
    /// Create a new instance initializing the program counter.
    pub fn new<T: Instance>(_vpr: Peri<'d, T>, address: u32) -> Result<Self, Error> {
        let mut this = Self {
            r: T::regs(),
            _phantom: PhantomData,
        };

        #[cfg(feature = "_s")]
        let spu = pac::SPU00_S;

        #[cfg(feature = "_ns")]
        let spu = pac::SPU00_NS;

        let flpr_index = 12;
        spu.periph(flpr_index).perm().write(|w| {
            w.set_secattr(true);
            w.set_dmasec(true);
        });

        this.reinit(address)?;

        Ok(this)
    }

    fn bound_check(address: u32) -> Result<(), Error> {
        if address % 8 != 0 {
            return Err(Error::NotAligned);
        }

        Ok(())
    }

    /// Initializes the program counter which will take effect on a
    /// core reset if the core is already running.
    pub fn reinit(&mut self, address: u32) -> Result<(), Error> {
        Self::bound_check(address)?;
        self.r.initpc().write_value(address);
        Ok(())
    }

    /// Start the coprocessor
    pub fn start(&mut self) {
        self.r.cpurun().write(|w| w.set_en(pac::vpr::vals::CpurunEn::RUNNING));
    }
}

impl<'d> Drop for Vpr<'d> {
    fn drop(&mut self) {
        //     self.r.cpurun().write(|w| w.set_en(pac::vpr::vals::CpurunEn::STOPPED))
    }
}

/// Error
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    /// Unaligned address
    NotAligned,
}

pub(crate) trait SealedInstance {
    fn regs() -> pac::vpr::Vpr;
}

/// VPR peripheral instance.
#[allow(private_bounds)]
pub trait Instance: PeripheralType + SealedInstance + 'static + Send {
    /// Interrupt for this peripheral.
    type Interrupt: interrupt::typelevel::Interrupt;
}

macro_rules! impl_vpr {
    ($type:ident, $pac_type:ident, $irq:ident) => {
        impl crate::vpr::SealedInstance for peripherals::$type {
            fn regs() -> pac::vpr::Vpr {
                pac::$pac_type
            }
        }
        impl crate::vpr::Instance for peripherals::$type {
            type Interrupt = crate::interrupt::typelevel::$irq;
        }
    };
}
