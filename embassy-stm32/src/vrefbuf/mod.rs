//! Voltage Reference Buffer (VREFBUF)
// use core::ptr::{read_volatile, write_volatile};
use core::marker::PhantomData;

use embassy_hal_internal::PeripheralType;
use stm32_metapac::vrefbuf::vals::*;

use crate::Peri;

/// Voltage Reference (VREFBUF) driver.
pub struct VoltageReferenceBuffer<'d, T: Instance> {
    vrefbuf: PhantomData<&'d mut T>,
}

impl<'d, T: Instance> VoltageReferenceBuffer<'d, T> {
    /// Creates an VREFBUF (Voltage Reference) instance with a voltage scale and impedance mode.
    ///
    /// [Self] has to be started with [Self::new()].
    pub fn new(_instance: Peri<'d, T>, voltage_scale: Vrs, impedance_mode: Hiz) -> Self {
        #[cfg(rcc_wba)]
        {
            use crate::pac::RCC;
            RCC.apb7enr().modify(|w| w.set_vrefen(true));
        }
        #[cfg(any(rcc_u5, rcc_h50, rcc_h5))]
        {
            use crate::pac::RCC;
            RCC.apb3enr().modify(|w| w.set_vrefen(true));
        }
        #[cfg(any(rcc_h7rs, rcc_h7rm0433, rcc_h7ab, rcc_h7))]
        {
            use crate::pac::RCC;
            RCC.apb4enr().modify(|w| w.set_vrefen(true));
        }
        let vrefbuf = T::regs();
        vrefbuf.csr().modify(|w| {
            w.set_hiz(impedance_mode);
            w.set_envr(true);
            w.set_vrs(voltage_scale);
        });
        while vrefbuf.csr().read().vrr() != false {
            // wait...
        }
        trace!(
            "Vrefbuf configured with voltage scale {} and impedance mode {}",
            voltage_scale,
            impedance_mode
        );
        VoltageReferenceBuffer { vrefbuf: PhantomData }
    }
}

trait SealedInstance {
    fn regs() -> crate::pac::vrefbuf::Vrefbuf;
}

/// VREFBUF instance trait.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType {}

foreach_peripheral!(
    (vrefbuf, $inst:ident) => {
        impl SealedInstance for crate::peripherals::$inst {
            fn regs() -> crate::pac::vrefbuf::Vrefbuf {
                crate::pac::$inst
            }
        }

        impl Instance for crate::peripherals::$inst {}
    };
);
