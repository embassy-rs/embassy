use crate::pac::{PWR, RCC, SYSCFG};
use crate::peripherals;

/// Voltage Scale
///
/// Represents the voltage range feeding the CPU core. The maximum core
/// clock frequency depends on this value.
#[derive(Copy, Clone, PartialEq)]
pub enum VoltageScale {
    /// VOS 0 range VCORE 1.26V - 1.40V
    Scale0,
    /// VOS 1 range VCORE 1.15V - 1.26V
    Scale1,
    /// VOS 2 range VCORE 1.05V - 1.15V
    Scale2,
    /// VOS 3 range VCORE 0.95V - 1.05V
    Scale3,
}

/// Power Configuration
///
/// Generated when the PWR peripheral is frozen. The existence of this
/// value indicates that the voltage scaling configuration can no
/// longer be changed.
pub struct Power {
    pub(crate) vos: VoltageScale,
}

impl Power {
    pub fn new(_peri: peripherals::PWR, enable_overdrive: bool) -> Self {
        // NOTE(unsafe) we have the PWR singleton
        unsafe {
            // NB. The lower bytes of CR3 can only be written once after
            // POR, and must be written with a valid combination. Refer to
            // RM0433 Rev 7 6.8.4. This is partially enforced by dropping
            // `self` at the end of this method, but of course we cannot
            // know what happened between the previous POR and here.
            PWR.cr3().modify(|w| {
                w.set_scuen(true);
                w.set_ldoen(true);
                w.set_bypass(false);
            });
            // Validate the supply configuration. If you are stuck here, it is
            // because the voltages on your board do not match those specified
            // in the D3CR.VOS and CR3.SDLEVEL fields. By default after reset
            // VOS = Scale 3, so check that the voltage on the VCAP pins =
            // 1.0V.
            while !PWR.csr1().read().actvosrdy() {}

            // Go to Scale 1
            PWR.d3cr().modify(|w| w.set_vos(0b11));
            while !PWR.d3cr().read().vosrdy() {}

            let vos = if !enable_overdrive {
                VoltageScale::Scale1
            } else {
                critical_section::with(|_| {
                    RCC.apb4enr().modify(|w| w.set_syscfgen(true));

                    SYSCFG.pwrcr().modify(|w| w.set_oden(1));
                });
                while !PWR.d3cr().read().vosrdy() {}
                VoltageScale::Scale0
            };
            Self { vos }
        }
    }
}
