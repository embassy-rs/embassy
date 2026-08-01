//! VPR coprocessor control.
#![macro_use]

use core::marker::PhantomData;
use core::ptr::copy_nonoverlapping;

use embassy_hal_internal::{Peri, PeripheralType};

use crate::pac::spu::vals::Dmasec;
use crate::pac::vpr::vals::CpurunEn;
use crate::{interrupt, pac};

/// SPU peripheral index of the VPR (FLPR) coprocessor.
const FLPR_SPU_INDEX: usize = 12;

/// Grant the application core secure access to the VPR (FLPR).
///
/// The VPR resets to non-secure; without this, accessing its secure-alias
/// registers bus-faults.
pub(crate) fn make_secure() {
    pac::SPU00.periph(FLPR_SPU_INDEX).perm().write(|w| {
        w.set_secattr(true);
        w.set_dmasec(Dmasec::Secure);
    });
}

/// Stop the coprocessor and pulse its debug-module reset.
///
/// This clears any program still running on it (a loaded program survives a
/// system reset). Requires [`make_secure`] to have been called first.
pub(crate) fn stop_reset(regs: pac::vpr::Vpr) {
    regs.cpurun().write(|w| w.set_en(CpurunEn::Stopped));
    regs.debugif().dmcontrol().write(|w| {
        w.set_ndmreset(true);
        w.set_dmactive(true);
    });
    regs.debugif().dmcontrol().write(|w| {
        w.set_ndmreset(false);
        w.set_dmactive(false);
    });
}

/// VPR coprocessor driver.
pub struct Vpr<'d> {
    regs: pac::vpr::Vpr,
    _p: PhantomData<&'d ()>,
    address: *const u8,
}

impl<'d> Vpr<'d> {
    /// Initialize the VPR coprocessor program counter.
    ///
    /// The address must be an 8-byte aligned in RAM.
    pub fn new<T: Instance>(_peri: Peri<'d, T>, address: *const u8) -> Result<Self, Error> {
        make_secure();

        let mut this = Self {
            regs: T::regs(),
            address,
            _p: PhantomData,
        };

        this.init(address)?;
        Ok(this)
    }

    /// Load the provided program into RAM.
    ///
    /// Call `start()` to start the coprocessor.
    pub fn load(&mut self, program: &[u8]) -> Result<(), Error> {
        if self.regs.cpurun().read().en() == pac::vpr::vals::CpurunEn::Running {
            return Err(Error::Running);
        }

        unsafe {
            copy_nonoverlapping(program.as_ptr(), self.address as *mut u8, program.len());
        }

        Ok(())
    }

    /// Initialize the coprocessor program counter.
    ///
    /// If the coprocessor is already running, this will only take effect on the next reset.
    pub fn init(&mut self, address: *const u8) -> Result<(), Error> {
        if address as u32 % 8 != 0 {
            return Err(Error::NotAligned);
        }

        self.address = address;
        let address = address as u32;
        self.regs.initpc().write_value(address);
        Ok(())
    }

    /// Start the coprocessor.
    ///
    /// If the coprocessor is already running, this does nothing.
    pub fn start(&mut self) {
        self.regs
            .cpurun()
            .write(|w| w.set_en(pac::vpr::vals::CpurunEn::Running));
    }

    /// Stop the coprocessor.
    ///
    /// If the coprocessor is already running, this will only take effect
    /// on the next reset.
    pub fn stop(&mut self) {
        self.regs
            .cpurun()
            .write(|w| w.set_en(pac::vpr::vals::CpurunEn::Stopped));
    }
}

/// Error
#[derive(Debug, Copy, Clone, Eq, PartialEq, thiserror::Error)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    /// Unaligned address
    #[error("unaligned address")]
    NotAligned,
    /// Core is already running
    #[error("core is already running")]
    Running,
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
