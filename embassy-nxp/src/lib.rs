#![no_std]
#![allow(unsafe_op_in_unsafe_fn)]

// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;

#[cfg(lpc55)]
pub mod adc;
#[cfg(lpc55)]
pub mod dma;
pub mod gpio;
#[cfg(lpc55)]
pub mod pint;
#[cfg(lpc55)]
pub mod pwm;
#[cfg(lpc55)]
pub mod sct;
#[cfg(lpc55)]
pub mod usart;
#[cfg(lpc55)]
pub mod usb;

#[cfg(lpc55)]
mod power;

#[cfg(rt1xxx)]
mod iomuxc;

#[cfg(feature = "_time_driver")]
#[cfg_attr(feature = "time-driver-pit", path = "time_driver/pit.rs")]
#[cfg_attr(feature = "time-driver-rtc", path = "time_driver/rtc.rs")]
mod time_driver;

// This mod MUST go last, so that it sees all the `impl_foo!` macros
#[cfg_attr(lpc55, path = "chips/lpc55.rs")]
#[cfg_attr(feature = "mimxrt1011", path = "chips/mimxrt1011.rs")]
#[cfg_attr(feature = "mimxrt1062", path = "chips/mimxrt1062.rs")]
mod chip;

pub use chip::{Peripherals, interrupt, peripherals};
pub use embassy_hal_internal::{Peri, PeripheralType};
#[cfg(feature = "unstable-pac")]
pub use nxp_pac as pac;
#[cfg(not(feature = "unstable-pac"))]
pub(crate) use nxp_pac as pac;

/// Macro to bind interrupts to handlers.
/// (Copied from `embassy-rp`)
/// This defines the right interrupt handlers, and creates a unit struct (like `struct Irqs;`)
/// and implements the right [`Binding`]s for it. You can pass this struct to drivers to
/// prove at compile-time that the right interrupts have been bound.
///
/// Example of how to bind one interrupt:
///
/// ```rust,ignore
/// use embassy_nxp::{bind_interrupts, usart, peripherals};
///
/// bind_interrupts!(
///     /// Binds the USART Interrupts.
///     struct Irqs {
///         FLEXCOMM0 => usart::InterruptHandler<peripherals::USART0>;
///     }
/// );
/// ```
#[macro_export]
macro_rules! bind_interrupts {
    ($(#[$attr:meta])* $vis:vis struct $name:ident {
        $(
            $(#[cfg($cond_irq:meta)])?
            $irq:ident => $(
                $(#[cfg($cond_handler:meta)])?
                $handler:ty
            ),*;
        )*
    }) => {
        #[derive(Copy, Clone)]
        $(#[$attr])*
        $vis struct $name;

        $(
            #[allow(non_snake_case)]
            #[unsafe(no_mangle)]
            $(#[cfg($cond_irq)])?
            unsafe extern "C" fn $irq() {
                unsafe {
                    $(
                        $(#[cfg($cond_handler)])?
                        <$handler as $crate::interrupt::typelevel::Handler<$crate::interrupt::typelevel::$irq>>::on_interrupt();

                    )*
                }
            }

            $(#[cfg($cond_irq)])?
            $crate::bind_interrupts!(@inner
                $(
                    $(#[cfg($cond_handler)])?
                    unsafe impl $crate::interrupt::typelevel::Binding<$crate::interrupt::typelevel::$irq, $handler> for $name {}
                )*
            );
        )*
    };
    (@inner $($t:tt)*) => {
        $($t)*
    }
}

/// Initialize the `embassy-nxp` HAL with the provided configuration.
///
/// This returns the peripheral singletons that can be used for creating drivers.
///
/// This should only be called once and at startup, otherwise it panics.
pub fn init(config: config::Config) -> Peripherals {
    #[cfg(not(lpc55))]
    let _ = &config;
    // Do this first, so that it panics if user is calling `init` a second time
    // before doing anything important.
    let peripherals = Peripherals::take();

    #[cfg(feature = "mimxrt1011")]
    {
        // The RT1010 Reference manual states that core clock root must be switched before
        // reprogramming PLL2.
        pac::CCM.cbcdr().modify(|w| {
            w.set_periph_clk_sel(pac::ccm::vals::PeriphClkSel::PeriphClkSel1);
        });

        while matches!(
            pac::CCM.cdhipr().read().periph_clk_sel_busy(),
            pac::ccm::vals::PeriphClkSelBusy::PeriphClkSelBusy1
        ) {}

        info!("Core clock root switched");

        // 480 * 18 / 24 = 360
        pac::CCM_ANALOG.pfd_480().modify(|x| x.set_pfd2_frac(12));

        //480*18/24(pfd0)/4
        pac::CCM_ANALOG.pfd_480().modify(|x| x.set_pfd0_frac(24));
        pac::CCM.cscmr1().modify(|x| x.set_flexspi_podf(3.into()));

        // CPU Core
        pac::CCM_ANALOG.pfd_528().modify(|x| x.set_pfd3_frac(18));
        cortex_m::asm::delay(500_000);

        // Clock core clock with PLL 2.
        pac::CCM
            .cbcdr()
            .modify(|x| x.set_periph_clk_sel(pac::ccm::vals::PeriphClkSel::PeriphClkSel0)); // false

        while matches!(
            pac::CCM.cdhipr().read().periph_clk_sel_busy(),
            pac::ccm::vals::PeriphClkSelBusy::PeriphClkSelBusy1
        ) {}

        pac::CCM
            .cbcmr()
            .write(|v| v.set_pre_periph_clk_sel(pac::ccm::vals::PrePeriphClkSel::PrePeriphClkSel0));

        // TODO: Some for USB PLLs

        // DCDC clock?
        pac::CCM.ccgr6().modify(|v| v.set_cg0(1));
    }

    #[cfg(any(lpc55, rt1xxx))]
    gpio::init();

    #[cfg(lpc55)]
    {
        if config.main_clock == config::MainClock::FroHf96 {
            power::set_voltage_for_freq(96_000_000);
            clocks::set_flash_access_cycles(8);
            pac::ANACTRL.fro192m_ctrl().modify(|w| w.set_ena_96mhzclk(true));
            pac::SYSCON.ahbclkdiv().modify(|w| w.set_div(0));
            pac::SYSCON
                .mainclksela()
                .modify(|w| w.set_sel(pac::syscon::vals::MainclkselaSel::Enum0x3));
            pac::SYSCON
                .mainclkselb()
                .modify(|w| w.set_sel(pac::syscon::vals::MainclkselbSel::Enum0x0));
        }

        if config.main_clock == config::MainClock::Pll0_150M {
            clocks::setup_pll0_150m_main_clock();
        }

        pint::init();
        pwm::Pwm::reset();
    }

    #[cfg(feature = "_time_driver")]
    time_driver::init();

    #[cfg(lpc55)]
    dma::init();

    peripherals
}

/// LPC55 clock tree setup.
#[cfg(lpc55)]
mod clocks {
    use crate::{pac, power};

    const CMD_SET_READ_MODE: u32 = 2;
    const FLASH_COMMAND_TIMEOUT_POLLS: u32 = 1_000_000;
    const XO_READY_TIMEOUT_POLLS: u32 = 1_000_000;
    const PLL_LOCK_TIMEOUT_POLLS: u32 = 1_000_000;

    const fn supports_pll0_150m(revision: u8) -> bool {
        revision == 1
    }

    const fn pll0_output_frequency_hz(input_hz: u32, ndiv: u8, mdiv: u16, pdiv: u8) -> u32 {
        input_hz / ndiv as u32 * mdiv as u32 / (2 * pdiv as u32)
    }

    /*
     * Copyright 2017 - 2021 , NXP
     * All rights reserved.
     *
     * SPDX-License-Identifier: BSD-3-Clause
     */
    pub(super) fn set_flash_access_cycles(wait_states: u8) {
        assert!(
            wait_states <= 0x0f,
            "LPC55 flash wait-state value {} exceeds the FMCCR field",
            wait_states
        );

        let prefetch_enabled = pac::SYSCON.fmccr().read().prefen();
        pac::SYSCON.fmccr().modify(|w| w.set_prefen(false));

        pac::FLASH
            .int_clr_status()
            .write_value(pac::flash::regs::IntClrStatus(0x1f));
        pac::FLASH.dataw(0).modify(|w| {
            w.set_dataw((w.dataw() & !0x0f) | u32::from(wait_states));
        });
        pac::FLASH.cmd().write_value(pac::flash::regs::Cmd(CMD_SET_READ_MODE));

        let mut command_done = false;
        for _ in 0..FLASH_COMMAND_TIMEOUT_POLLS {
            let status = pac::FLASH.int_status().read();
            if status.fail() || status.err() {
                panic!(
                    "LPC55 FLASH CMD_SET_READ_MODE failed: FAIL={}, ERR={}",
                    status.fail(),
                    status.err()
                );
            }
            if status.done() {
                command_done = true;
                break;
            }
        }
        if !command_done {
            panic!(
                "LPC55 FLASH CMD_SET_READ_MODE timed out after {} polls",
                FLASH_COMMAND_TIMEOUT_POLLS
            );
        }

        pac::SYSCON.fmccr().modify(|w| {
            w.set_flashtim(pac::syscon::vals::Flashtim::from_bits(wait_states));
            w.set_prefen(prefetch_enabled);
        });
    }

    pub(crate) fn setup_pll0_150m_main_clock() {
        const XTAL_HZ: u32 = 16_000_000;
        const NDIV: u8 = 8;
        const MDIV: u16 = 150;
        const PDIV: u8 = 1;
        const SELI: u8 = 53;
        const SELP: u8 = 31;
        const _: () = core::assert!(pll0_output_frequency_hz(XTAL_HZ, NDIV, MDIV, PDIV) == 150_000_000);

        let revision = pac::SYSCON.dieid().read().rev_id();
        if !supports_pll0_150m(revision) {
            panic!(
                "LPC55 PLL0 150 MHz requires die revision 1B (REV_ID 1); observed REV_ID {}",
                revision
            );
        }

        pac::SYSCON
            .mainclksela()
            .write(|w| w.set_sel(pac::syscon::vals::MainclkselaSel::Enum0x0));
        pac::SYSCON
            .mainclkselb()
            .write(|w| w.set_sel(pac::syscon::vals::MainclkselbSel::Enum0x0));

        power::set_voltage_for_freq(150_000_000);
        set_flash_access_cycles(11);

        pac::PMC
            .pdruncfgclr0()
            .write(|w| w.set_pdruncfgclr0((1 << 8) | (1 << 20)));
        pac::ANACTRL.xo32m_ctrl().modify(|w| w.set_enable_system_clk_out(true));
        pac::SYSCON.clock_ctrl().modify(|w| w.set_clkin_ena(true));

        let mut crystal_ready = false;
        for _ in 0..XO_READY_TIMEOUT_POLLS {
            if pac::ANACTRL.xo32m_status().read().xo_ready() {
                crystal_ready = true;
                break;
            }
        }
        if !crystal_ready {
            panic!(
                "LPC55 16 MHz crystal did not report XO_READY within {} polls",
                XO_READY_TIMEOUT_POLLS
            );
        }

        pac::PMC
            .pdruncfgset0()
            .write(|w| w.set_pdruncfgset0((1 << 9) | (1 << 23)));
        pac::SYSCON
            .pll0clksel()
            .write(|w| w.set_sel(pac::syscon::vals::Pll0clkselSel::Enum0x1));

        pac::SYSCON.pll0ctrl().write(|w| {
            w.set_selr(0);
            w.set_seli(SELI);
            w.set_selp(SELP);
            w.set_clken(true);
        });
        pac::SYSCON.pll0ndec().write(|w| w.set_ndiv(NDIV));
        pac::SYSCON.pll0ndec().write(|w| {
            w.set_ndiv(NDIV);
            w.set_nreq(true);
        });
        pac::SYSCON.pll0pdec().write(|w| w.set_pdiv(PDIV));
        pac::SYSCON.pll0pdec().write(|w| {
            w.set_pdiv(PDIV);
            w.set_preq(true);
        });
        pac::SYSCON.pll0sscg0().modify(|w| w.set_md_lbs(0));
        pac::SYSCON.pll0sscg1().write(|w| {
            w.set_mdiv_ext(MDIV);
            w.set_sel_ext(true);
        });
        pac::SYSCON.pll0sscg1().write(|w| {
            w.set_mdiv_ext(MDIV);
            w.set_md_req(true);
            w.set_mreq(true);
            w.set_sel_ext(true);
        });

        pac::PMC
            .pdruncfgclr0()
            .write(|w| w.set_pdruncfgclr0((1 << 9) | (1 << 23)));

        let mut pll_locked = false;
        for _ in 0..PLL_LOCK_TIMEOUT_POLLS {
            if pac::SYSCON.pll0stat().read().lock() {
                pll_locked = true;
                break;
            }
        }
        if !pll_locked {
            panic!(
                "LPC55 PLL0 did not lock at 150 MHz within {} polls",
                PLL_LOCK_TIMEOUT_POLLS
            );
        }

        pac::SYSCON.ahbclkdiv().modify(|w| w.set_div(0));
        pac::SYSCON
            .mainclkselb()
            .write(|w| w.set_sel(pac::syscon::vals::MainclkselbSel::Enum0x1));
    }

    #[cfg(test)]
    mod tests {
        use super::{pll0_output_frequency_hz, supports_pll0_150m};

        #[test]
        fn pll0_150m_rejects_revision_0a() {
            assert!(!supports_pll0_150m(0));
        }

        #[test]
        fn pll0_150m_accepts_revision_1b() {
            assert!(supports_pll0_150m(1));
        }

        #[test]
        fn pll0_150m_rejects_unknown_revision() {
            assert!(!supports_pll0_150m(2));
        }

        #[test]
        fn pll0_equation_yields_150mhz() {
            assert_eq!(pll0_output_frequency_hz(16_000_000, 8, 150, 1), 150_000_000);
        }
    }
}

/// HAL configuration for the NXP board.
pub mod config {
    /// Main (system) clock selection for LPC55.
    #[cfg(lpc55)]
    #[derive(Default, Clone, Copy, PartialEq, Eq)]
    pub enum MainClock {
        /// Leave the ROM boot default untouched.
        #[default]
        Untouched,
        /// FRO HF 96 MHz as main clock (required for USB-HS).
        FroHf96,
        /// PLL0 at 150 MHz (from the 16 MHz crystal) as main clock.
        ///
        /// This mode is available only on die revision 1B. It uses the NXP
        /// power-profile algorithm before raising the system clock.
        /// It also satisfies the USB-HS >= 96 MHz system clock requirement.
        Pll0_150M,
    }

    /// HAL configuration.
    #[derive(Default)]
    pub struct Config {
        /// Main (system) clock selection.
        #[cfg(lpc55)]
        pub main_clock: MainClock,
    }
}

#[allow(unused)]
struct BitIter(u32);

impl Iterator for BitIter {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0.trailing_zeros() {
            32 => None,
            b => {
                self.0 &= !(1 << b);
                Some(b)
            }
        }
    }
}

trait SealedMode {}

/// UART mode.
#[allow(private_bounds)]
pub trait Mode: SealedMode {}

macro_rules! impl_mode {
    ($name:ident) => {
        impl SealedMode for $name {}
        impl Mode for $name {}
    };
}

/// Blocking mode.
pub struct Blocking;
/// Asynchronous mode.
pub struct Async;

impl_mode!(Blocking);
impl_mode!(Async);
