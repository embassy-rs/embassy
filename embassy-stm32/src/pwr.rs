//! Power control

/// Configuration what peripherals should be running during standby or shutdown mode.
/// It is determined by the disabled peripherals, if shutdown mode is used instead of standby
/// mode if the chip supports it.
#[derive(Clone, Copy)]
#[non_exhaustive]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct StandbyConfig {
    // empty, we might want to disable RTC here, enable WKUP pins, ...
    #[cfg(any(stm32f1, stm32f4))]
    pub enable_wkup: bool,
}

impl Default for StandbyConfig {
    fn default() -> Self {
        Self {
            #[cfg(any(stm32f1, stm32f4))]
            enable_wkup: false,
        }
    }
}

/// Enter standby or shutdown mode.
///
/// STM32 microcontrollers support the "standby" and sometimes the "shutdown" mode,
/// where almost all peripherals and the core will be shut down.
/// In most cases, this is the deepest sleep mode.
/// If standby mode is entered, the program ends as the SRAM is not kept.
/// After a wakeup the program starts from the start.
/// It can be determined if the core started after a reset or standby mode.
///
/// Content of variables will be lost as the SRAM is disabled during standby.
/// State has to be either stored inside the RTC peripheral,
/// or on the flash.
///
/// The following peripherals are not shut down during standby and can wake up the core:
///
/// ## WKUP GPIO pins: (this is currently unimplemented)
///
/// STM32 microcontrollers have some dedicated WKUP pins that can optionally be used as an input
/// to wake up during standby.
/// If a pin is configured as wakup pin, it will be converted to an input with pull-down enabled.
/// If the pin state switches to high, the core will wake up.
///
/// ## RTC event: (this is currently unimplemented)
///
/// The RTC can wake up the core at a specific time.
/// If RTC is enabled during standby, more current is used.
///
/// ## IWDG
///
/// If the Independent Watchdog is enabled before standby,
/// it will not be disabled and can still reset the core if it was not pet.
/// This can also be used to wake up after and inaccurate amount of time has passed in standby mode,
/// which has the advantage that it is in most cases less power hungry than the RTC.
#[allow(unused_variables)]
pub fn standby(config: &StandbyConfig) -> ! {
    critical_section::with(|_| {
        // Safety: We kill the core at the end, so we don't care about most users after this
        let mut scb = unsafe { cortex_m::Peripherals::steal().SCB };
        scb.set_sleepdeep();

        #[cfg(any(stm32l0, stm32c0, stm32f1))]
        let cr = crate::pac::PWR.cr();
        #[cfg(any(stm32f4))]
        let cr = crate::pac::PWR.cr1();

        #[cfg(any(stm32l0, stm32c0, stm32f1))]
        let csr = crate::pac::PWR.csr();
        #[cfg(any(stm32f4))]
        let csr = crate::pac::PWR.csr1();

        #[cfg(any(stm32l0, stm32c0, stm32f1, stm32f4))]
        cr.write(|r| r.set_pdds(crate::pac::pwr::vals::Pdds::STANDBY_MODE));
        #[cfg(any(stm32l0, stm32c0, stm32f1, stm32f4))]
        csr.write(|r| r.set_wuf(false));

        #[cfg(any(stm32f1, stm32f4))]
        csr.write(|r| r.set_ewup(config.enable_wkup));

        #[cfg(stm32h7)]
        prepare_standby_h7();

        cortex_m::asm::wfi();
    });
    unreachable!("core should be dead after this")
}

#[cfg(stm32h7)]
fn prepare_standby_h7() {
    crate::pac::PWR.cpucr().write(|r| {
        r.set_run_d3(false);
        r.set_pdds_d1(true);
        r.set_pdds_d2(true);
        r.set_pdds_d3(true);
    });
    crate::pac::PWR.wkupfr().write(|r| {
        r.set_wkupf(0, false);
        r.set_wkupf(1, false);
        r.set_wkupf(2, false);
        r.set_wkupf(3, false);
        r.set_wkupf(4, false);
        r.set_wkupf(5, false);
    });
}

// WFI (Wait for Interrupt) or WFE (Wait for Event) while:
// – SLEEPDEEP = 1 in Cortex®
// -M0+ System Control register
// – PDDS = 1 bit in Power Control register (PWR_CR)
// – No interrupt (for WFI) or event (for WFE) is pending.
// – WUF = 0 bit in Power Control/Status register (PWR_CSR)
// – the RTC flag corresponding to the chosen wakeup source (RTC Alarm A,
// RTC Alarm B, RTC wakeup, Tamper or Time-stamp flags) is cleared
// - L0 (C0) F4
// TODO: RTC Flags

// WFI (Wait for Interrupt) or WFE (Wait for Event) while:
// – Set SLEEPDEEP in Cortex® -M3 System Control register
// – Set PDDS bit in Power Control register (PWR_CR)
// – Clear WUF bit in Power Control/Status register (PWR_CSR)
// – No interrupt (for WFI) or event (for WFI) is pending
// - F1

// WFI (Wait for Interrupt) or WFE (Wait for Event) while:
// – Set SLEEPDEEP in Cortex® -M0 System Control register
// – Set PDDS bit in Power Control register (PWR_CR)
// – Clear WUF bit in Power Control/Status register (PWR_CSR)
// - F0

// WFI (Wait for Interrupt) or WFE (Wait for Event) while:
// – SLEEPDEEP bit is set in Cortex® -M4 System Control register
// – No interrupt (for WFI) or event (for WFE) is pending
// – LPMS = “011” in PWR_CR1
// – WUFx bits are cleared in power status register 1 (PWR_SR1)
//
// L4, G0, G4
// TODO: RTC Flags

// CStop:
// WFI (Wait for Interrupt) or WFE (Wait for Event) while:
// – SLEEPDEEP = 1 (Refer to the Cortex®
// - M System Control register.)
// – CPU NVIC interrupts and events cleared.
// – All CPU EXTI Wakeup sources are cleared.
//
// Standby:
// – The CPU subsystem is in CStop mode, and there is no active EXTI
// Wakeup source and RUN_D3 = 0.
// – All PDDS_Dn bits for all domains select Standby.
// – All WKUPF bits in Power Control/Status register (PWR_WKUPFR) are
// cleared
//
// H7
