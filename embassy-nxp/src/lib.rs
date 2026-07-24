#![no_std]
#![allow(unsafe_op_in_unsafe_fn)]

// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;

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
            w.set_periph_clk_sel(pac::ccm::vals::PeriphClkSel::PERIPH_CLK_SEL_1);
        });

        while matches!(
            pac::CCM.cdhipr().read().periph_clk_sel_busy(),
            pac::ccm::vals::PeriphClkSelBusy::PERIPH_CLK_SEL_BUSY_1
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
            .modify(|x| x.set_periph_clk_sel(pac::ccm::vals::PeriphClkSel::PERIPH_CLK_SEL_0)); // false

        while matches!(
            pac::CCM.cdhipr().read().periph_clk_sel_busy(),
            pac::ccm::vals::PeriphClkSelBusy::PERIPH_CLK_SEL_BUSY_1
        ) {}

        pac::CCM
            .cbcmr()
            .write(|v| v.set_pre_periph_clk_sel(pac::ccm::vals::PrePeriphClkSel::PRE_PERIPH_CLK_SEL_0));

        // TODO: Some for USB PLLs

        // DCDC clock?
        pac::CCM.ccgr6().modify(|v| v.set_cg0(1));
    }

    #[cfg(any(lpc55, rt1xxx))]
    gpio::init();

    #[cfg(lpc55)]
    {
        if config.main_clock == config::MainClock::FroHf96 {
            // Flash wait states first, so timing is safe before the frequency rises.
            // 9 system-clock access time covers rates up to 100 MHz.
            pac::SYSCON
                .fmccr()
                .modify(|w| w.set_flashtim(pac::syscon::vals::Flashtim::FLASHTIM8));
            // Ensure the FRO-HF 96 MHz output is enabled (ROM default leaves it on;
            // set it anyway).
            pac::ANACTRL.fro192m_ctrl().modify(|w| w.set_ena_96mhzclk(true));
            // AHB divider = 1, then switch main clock to FRO-HF (glitch-free per UM).
            pac::SYSCON.ahbclkdiv().modify(|w| w.set_div(0));
            pac::SYSCON
                .mainclksela()
                .modify(|w| w.set_sel(pac::syscon::vals::MainclkselaSel::ENUM_0X3));
            pac::SYSCON
                .mainclkselb()
                .modify(|w| w.set_sel(pac::syscon::vals::MainclkselbSel::ENUM_0X0));
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

/// LPC55 clock tree setup beyond the simple FRO paths handled inline in `init`.
#[cfg(lpc55)]
mod clocks {
    use crate::pac;

    /// Switch the main (system) clock to PLL0 at 150 MHz, fed from the 16 MHz
    /// external crystal (clk_in).
    ///
    /// PLL parameters (UM11126 §4.6.6): N = 8 (ref 2 MHz), M = 150
    /// (CCO = 300 MHz, within the 275–550 MHz range), P = 1 (post-divide by
    /// 2·P = 2) → 150 MHz. SELI/SELP follow the UM11126 bandwidth formulas
    /// for M = 150 (SELI = 8000/M = 53, SELP = min(M/4 + 1, 31) = 31).
    ///
    /// NOTE: NXP's closed-source power library additionally raises the core
    /// voltage for operation above 100 MHz (`POWER_SetVoltageForFreq`). The
    /// registers involved are not documented in UM11126, so this runs on the
    /// reset-default regulator settings; long-duration stress testing is the
    /// acceptance gate for this configuration.
    pub(crate) fn setup_pll0_150m_main_clock() {
        const XTAL_HZ: u32 = 16_000_000;
        const NDIV: u8 = 8;
        const MDIV: u16 = 150;
        const PDIV: u8 = 1;
        const SELI: u8 = 53; // 8000 / M, for 122 <= M < 8000
        const SELP: u8 = 31; // min(M/4 + 1, 31)
        const _: () = core::assert!(XTAL_HZ / NDIV as u32 * MDIV as u32 / (2 * PDIV as u32) == 150_000_000);

        // Flash wait states first: 12 system-clock access time covers 150 MHz
        // (UM11126 FMCCR.FLASHTIM = 11).
        pac::SYSCON
            .fmccr()
            .modify(|w| w.set_flashtim(pac::syscon::vals::Flashtim::FLASHTIM11));

        // Power up the 16 MHz crystal oscillator + its LDO, and route it to
        // the system clock tree (clk_in). PDRUNCFG0: bit 8 = XTAL32M,
        // bit 20 = LDOXO32M; writing 1 to the CLR register powers ON.
        pac::PMC
            .pdruncfgclr0()
            .write(|w| w.set_pdruncfgclr0((1 << 8) | (1 << 20)));
        pac::ANACTRL.xo32m_ctrl().modify(|w| w.set_enable_system_clk_out(true));
        pac::SYSCON.clock_ctrl().modify(|w| w.set_clkin_ena(true));
        // Bounded wait for the crystal to be ready (typ. ~350 us).
        for _ in 0..1_000_000 {
            if pac::ANACTRL.xo32m_status().read().xo_ready() {
                break;
            }
        }

        // Power up PLL0 and its SSCG block (the M-divider lives in the SSCG
        // wrapper even with spread spectrum disabled). Bits 9 / 23.
        pac::PMC
            .pdruncfgclr0()
            .write(|w| w.set_pdruncfgclr0((1 << 9) | (1 << 23)));

        // PLL0 input = clk_in (16 MHz crystal).
        pac::SYSCON
            .pll0clksel()
            .write(|w| w.set_sel(pac::syscon::vals::Pll0clkselSel::ENUM_0X1));

        // Bandwidth + enable the post-divider output clock.
        pac::SYSCON.pll0ctrl().write(|w| {
            w.set_selr(0);
            w.set_seli(SELI);
            w.set_selp(SELP);
            w.set_clken(true);
        });
        // Dividers: write the ratio, then pulse the change request bit.
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
        // Integer M via the external M-divider (SEL_EXT = 1), no spread
        // spectrum (MF/MR/MC = 0).
        pac::SYSCON.pll0sscg1().write(|w| {
            w.set_mdiv_ext(MDIV);
            w.set_sel_ext(true);
        });
        pac::SYSCON.pll0sscg1().write(|w| {
            w.set_mdiv_ext(MDIV);
            w.set_sel_ext(true);
            w.set_mreq(true);
        });

        // Wait for lock (ref = 2 MHz > 100 kHz, so the lock detector is
        // usable per UM11126), with a generous bound; the UM quotes a
        // worst-case lock time of ~500 us.
        let mut locked = false;
        for _ in 0..1_000_000 {
            if pac::SYSCON.pll0stat().read().lock() {
                locked = true;
                break;
            }
        }
        if !locked {
            warn!("PLL0 failed to lock, main clock left unchanged");
            return;
        }

        // AHB divider = 1, then glitch-free switch: main clock B = PLL0.
        pac::SYSCON.ahbclkdiv().modify(|w| w.set_div(0));
        pac::SYSCON
            .mainclkselb()
            .modify(|w| w.set_sel(pac::syscon::vals::MainclkselbSel::ENUM_0X1));
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
        /// Also satisfies the USB-HS >= 96 MHz system clock requirement.
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
