//! Boot Select button
//!
//! The RP2040 and RP2350 ROMs supports a BOOTSEL button that is used to enter the USB bootloader
//! if held during reset. To avoid wasting GPIO pins, the button is multiplexed onto
//! the CS pin of the QSPI flash, but that makes it somewhat expensive and complicated
//! to utilize outside of the rom's bootloader.
//!
//! This module provides functionality to poll BOOTSEL from an embassy application.

use core::mem;

use rp_pac::IO_QSPI;
use rp_pac::io::regs::{GpioCtrl, GpioStatus};
use rp_pac::io::vals::Oeover;

use crate::Peri;
use crate::flash::in_ram;

/// Reads the BOOTSEL button. Returns true if the button is pressed.
///
/// Reading isn't cheap, as this function waits for core 1 to finish it's current
/// task and for any DMAs from flash to complete
pub fn is_bootsel_pressed(_p: Peri<'_, crate::peripherals::BOOTSEL>) -> bool {
    unsafe {
        // Compute the base address for the GPIO_QSPI_SS_STATUS/GPIO_QSPI_SS_CTRL registers.
        let cs_gpio: *mut ();
        if cfg!(feature = "rp2040") {
            cs_gpio = IO_QSPI.gpio(1).as_ptr();
        } else if cfg!(feature = "_rp235x") {
            cs_gpio = IO_QSPI.gpio(3).as_ptr();
        } else {
            unimplemented!()
        };

        let mut cs_ctrl = GpioCtrl::default();
        cs_ctrl.set_oeover(Oeover::Disable);
        let cs_ctrl: u32 = mem::transmute(cs_ctrl);

        let mut cs_status = 0;
        in_ram(|| cs_status = ram_helpers::read_cs_status(cs_gpio, cs_ctrl)).expect("Must be called from Core 0");

        // bootsel is active low, so invert
        !mem::transmute::<u32, GpioStatus>(cs_status).infrompad()
    }
}

mod ram_helpers {

    /// Temporally reconfigures the CS gpio and returns the GpioStatus.

    /// This function runs from RAM so it can disable flash XIP.
    ///
    /// # Safety
    ///
    /// The caller must ensure flash is idle and will remain idle.
    /// This function must live in ram. It uses inline asm to avoid any
    /// potential calls to ABI functions that might be in flash.
    #[inline(never)]
    #[unsafe(link_section = ".data.ram_func")]
    #[cfg(target_arch = "arm")]
    pub unsafe fn read_cs_status(cs_gpio: *mut (), cs_ctrl: u32) -> u32 {
        let cs_status: u32;
        core::arch::asm!(
            ".equiv GPIO_STATUS, 0x0",
            ".equiv GPIO_CTRL,   0x4",

            "ldr {orig_ctrl}, [{cs_gpio}, $GPIO_CTRL]",

            // The BOOTSEL pulls the flash's CS line low though a 1K resistor.
            // this is weak enough to avoid disrupting normal operation.
            // But, if we disable CS's output drive and allow it to float...
            "str {val}, [{cs_gpio}, $GPIO_CTRL]",

            // ...then wait for the state to settle...
            "2:", // ~4000 cycle delay loop
            "subs {delay}, #8",
            "bne 2b",

            // ...we can read the current state of bootsel
            "ldr {val}, [{cs_gpio}, $GPIO_STATUS]",

            // Finally, restore CS to normal operation so XIP can continue
            "str {orig_ctrl}, [{cs_gpio}, $GPIO_CTRL]",

            cs_gpio = in(reg) cs_gpio,
            orig_ctrl = out(reg) _,
            val = inout(reg) cs_ctrl => cs_status,
            delay = in(reg) 8192,
            options(nostack),
        );

        cs_status
    }

    #[cfg(not(target_arch = "arm"))]
    pub unsafe fn read_cs_status(cs_gpio: *mut (), cs_ctrl: u32) -> u32 {
        unimplemented!()
    }
}
