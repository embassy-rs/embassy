//! Trait for the Delay Block

#![macro_use]

use embassy_hal_internal::PeripheralType;

pub(crate) trait DlybRccPeripheral {
    #[cfg(rcc_n6)]
    #[allow(dead_code)]
    fn reset_and_enable();
}

#[cfg(rcc_n6)]
foreach_peripheral!(
    (dlybsd, DLYB_SDMMC1) => {
        impl DlybRccPeripheral for crate::peripherals::DLYB_SDMMC1 {
            fn reset_and_enable() {
                crate::pac::RCC.miscrstr().modify(|w| w.set_sdmmc1dllrst(false));
            }
        }
    };
    (dlybsd, DLYB_SDMMC2) => {
        impl DlybRccPeripheral for crate::peripherals::DLYB_SDMMC2 {
            fn reset_and_enable() {
                crate::pac::RCC.miscrstr().modify(|w| w.set_sdmmc2dllrst(false));
            }
        }
    };
);

#[cfg(not(rcc_n6))]
impl<T> DlybRccPeripheral for T {}

pub(crate) trait SealedDlybInstance<T>: DlybRccPeripheral + PeripheralType + 'static {
    #[allow(dead_code)]
    fn regs() -> crate::pac::dlybsd::Dlybsd;
}

/// Instance With Delay Block
#[allow(private_bounds)]
pub trait DlybInstance<T>: SealedDlybInstance<T> {}

impl<T, I: SealedDlybInstance<T>> DlybInstance<T> for I {}

#[allow(unused_macros)]
macro_rules! impl_dlyb_instance {
    ($peri:ident, $dlyb:ident) => {
        impl crate::dlyb::SealedDlybInstance<crate::peripherals::$peri> for crate::peripherals::$dlyb {
            fn regs() -> crate::pac::dlybsd::Dlybsd {
                crate::pac::$dlyb
            }
        }
    };
}
