//! Voltage Reference Buffer (VREFBUF)
use core::marker::PhantomData;

use embassy_hal_internal::PeripheralType;
use stm32_metapac::vrefbuf::vals::*;

use crate::Peri;

/// Voltage Reference (VREFBUF) driver.
pub struct VoltageReferenceBuffer<'d, T: Instance> {
    vrefbuf: PhantomData<&'d mut T>,
}

#[cfg(rcc_wba)]
#[repr(usize)]
enum VRefBufTrim {
    VRefBuf3Trim = 0x0BFA_07A8,
    VRefBuf2Trim = 0x0BFA_07A9,
    VRefBuf1Trim = 0x0BFA_07AA,
    VRefBuf0Trim = 0x0BFA_07AB,
}

#[cfg(rcc_wba)]
fn get_refbuf_trim(voltage_scale: Vrs) -> VRefBufTrim {
    match voltage_scale {
        Vrs::VREF0 => VRefBufTrim::VRefBuf0Trim,
        Vrs::VREF1 => VRefBufTrim::VRefBuf1Trim,
        Vrs::VREF2 => VRefBufTrim::VRefBuf2Trim,
        Vrs::VREF3 => VRefBufTrim::VRefBuf3Trim,
        _ => panic!("Incorrect Vrs setting!"),
    }
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
            // This is an errata for WBA6 devices. VREFBUF_TRIM value isn't set correctly
            // [Link explaining it](https://www.st.com/resource/en/errata_sheet/es0644-stm32wba6xxx-device-errata-stmicroelectronics.pdf)
            unsafe {
                use crate::pac::VREFBUF;
                let addr = get_refbuf_trim(voltage_scale) as usize;
                let buf_trim_ptr = core::ptr::with_exposed_provenance::<u32>(addr);
                let trim_val = core::ptr::read_volatile(buf_trim_ptr);
                VREFBUF.ccr().write(|w| w.set_trim((trim_val & 0xFF) as u8));
            }
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
