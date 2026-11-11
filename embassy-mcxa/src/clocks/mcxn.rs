//! Initial MCXN clock support: CPU0 on FRO12, without voltage scaling or deep sleep.
//!
//! The fixed profile uses SIRC at 12 MHz and clk_1m at 1 MHz. Keep inherited
//! supply, SRAM ECC and flash wait states. Identical register maps do not imply
//! identical electrical limits, ROM/security behavior, or silicon errata.

use core::cell::RefCell;

use cortex_m::peripheral::SCB;

use crate::{pac, peripherals};

/// Configuration of the fixed MCXN clock profile.
pub mod config {
    /// Fixed FRO12 profile. No unsupported clock choices are accepted or ignored.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    #[non_exhaustive]
    pub struct ClocksConfig {}
}

/// A nominal clock frequency established by source/divider readback.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Clock {
    /// Nominal frequency in hertz, following the MCXA clock API's `u32` convention.
    ///
    /// This is calculated from source/divider readback, not independently measured.
    pub frequency: u32,
}

/// Clocks available in the initial, non-deep-sleep MCXN profile.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Clocks {
    /// Main clock before the AHB divider; nominally 12 MHz in this profile.
    pub main_clk: Clock,
    /// Primary CPU/system/AHB clock after the AHB divider; nominally 12 MHz.
    pub cpu_system_clk: Clock,
    /// Slow peripheral-bus clock, fixed at one quarter of the system clock.
    pub slow_clk: Clock,
    /// Independent 1 MHz FRO12 output used by the OSTIMER time driver.
    pub clk_1m: Clock,
}

/// Startup phase retained in diagnostics when hardware does not settle.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum ClockPhase {
    /// Enable and validate the slow internal oscillator (FRO12).
    Sirc,
    /// Select and verify the source of the main system clock.
    MainSource,
    /// Configure and stabilize the AHB clock divider.
    AhbDivider,
    /// Release and stabilize the fixed slow-clock divider.
    SlowDivider,
    /// Enable or disable a peripheral's register-interface clock.
    PeripheralGate,
    /// Assert, release, or verify a peripheral reset.
    PeripheralReset,
}

impl core::fmt::Display for ClockPhase {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::Sirc => "SIRC setup",
            Self::MainSource => "main clock source selection",
            Self::AhbDivider => "AHB divider setup",
            Self::SlowDivider => "slow clock divider setup",
            Self::PeripheralGate => "peripheral clock gating",
            Self::PeripheralReset => "peripheral reset",
        })
    }
}

/// A failure while establishing the MCXN clock profile or peripheral clocks.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum ClockError {
    /// A clock snapshot was already published by an earlier initialization.
    AlreadyInitialized,
    /// The numeric device-type readback does not match the selected chip feature.
    /// Contains the observed `DEVICE_TYPE_NUM` value.
    WrongChip(u16),
    /// CPU1 is not held in reset on a dual-core chip.
    OtherCoreActive,
    /// Hardware did not reach the required state within the startup polling budget.
    Timeout(ClockPhase),
    /// A register readback did not match the supported profile or required state.
    InvalidReadback(ClockPhase),
}

impl core::fmt::Display for ClockError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::AlreadyInitialized => f.write_str("clocks are already initialized"),
            Self::WrongChip(part) => write!(f, "unexpected MCXN device type {part:#06x}"),
            Self::OtherCoreActive => f.write_str("CPU1 must be held in reset"),
            Self::Timeout(phase) => write!(f, "timed out during {phase}"),
            Self::InvalidReadback(phase) => write!(f, "invalid readback during {phase}"),
        }
    }
}

impl core::error::Error for ClockError {}

static CLOCKS: critical_section::Mutex<RefCell<Option<Clocks>>> = critical_section::Mutex::new(RefCell::new(None));

// A finite polling budget, not a wall-clock timeout or an Embassy timer. No
// initialized timer is available while switching the clocks that feed it.
pub(crate) fn wait(phase: ClockPhase, mut ready: impl FnMut() -> bool) -> Result<(), ClockError> {
    for _ in 0..1_000_000 {
        if ready() {
            return Ok(());
        }
        core::hint::spin_loop();
    }
    Err(ClockError::Timeout(phase))
}

/// Establish the fixed FRO12 profile exactly once.
pub fn init(_config: config::ClocksConfig) -> Result<(), ClockError> {
    critical_section::with(|cs| {
        if CLOCKS.borrow_ref(cs).is_some() {
            return Err(ClockError::AlreadyInitialized);
        }
        let syscon = pac::SYSCON0;
        let part = (syscon.device_type().read().0 & 0xffff) as u16;
        if part != crate::_generated::MCXN_DEVICE_TYPE {
            return Err(ClockError::WrongChip(part));
        }
        #[cfg(not(feature = "_mcxn2xx"))]
        if syscon.cpuctrl().read().cpu1rsten().to_bits() != 1 {
            return Err(ClockError::OtherCoreActive);
        }

        // Enable the register interface only. Never reset a live clock block.
        syscon.ahbclkctrl2().modify(|w| w.set_scg(1u8.into()));
        cortex_m::asm::dsb();
        cortex_m::asm::isb();
        let scg = pac::SCG0;
        if scg.sirccsr().read().sircerr().to_bits() != 0 {
            return Err(ClockError::InvalidReadback(ClockPhase::Sirc));
        }
        scg.sirccsr().modify(|w| {
            // SIRCERR is W1C: do not clear a newly latched error in this RMW.
            w.set_sircerr(0u8.into());
            w.set_lk(0u8.into());
        });
        scg.sirccsr().modify(|w| {
            w.set_sircerr(0u8.into());
            w.set_sirc_clk_periph_en(1u8.into());
            w.set_sircsten(0u8.into());
        });
        wait(ClockPhase::Sirc, || scg.sirccsr().read().sircvld().to_bits() == 1)?;
        if scg.sirccsr().read().sircerr().to_bits() != 0 {
            return Err(ClockError::InvalidReadback(ClockPhase::Sirc));
        }

        // No regulator, SRAM-voltage or flash-wait-state write is needed for
        // this conservative profile. Retain adequate inherited flash timing
        // across the source/divider transition, including after downclocking.
        scg.rccr().modify(|w| w.set_scs(2u8.into()));
        wait(ClockPhase::MainSource, || scg.csr().read().scs().to_bits() == 2)?;
        syscon.ahbclkdiv().modify(|w| w.set_div(0));
        wait(ClockPhase::AhbDivider, || {
            syscon.ahbclkdiv().read().unstab().to_bits() == 0
        })?;
        syscon.slowclkdiv().modify(|w| {
            w.set_reset(0u8.into());
            w.set_halt(0u8.into());
        });
        wait(ClockPhase::SlowDivider, || {
            syscon.slowclkdiv().read().unstab().to_bits() == 0
        })?;

        // Establish ordinary WFE idle, not A's deep-sleep/FLASHWAKE policy.
        // As in the A clock driver, do not call Peripherals::steal(): it marks
        // the core singleton as taken even if we only use its SCB field.
        // SAFETY: SCB is a zero-sized handle containing only PhantomData.
        // This temporary handle is used on the primary core inside a critical
        // section solely for these two read-modify-write accessors. It does not
        // escape initialization or reserve the application's core peripherals.
        let mut scb: SCB = unsafe { core::mem::transmute(()) };
        scb.clear_sleeponexit();
        scb.clear_sleepdeep();
        pac::CMC0.ckctrl().modify(|w| w.set_ckmode(0u8.into()));
        pac::CMC0.flashcr().modify(|w| {
            w.set_flashdis(0u8.into());
            w.set_flashdoze(0u8.into());
        });
        cortex_m::asm::dsb();
        cortex_m::asm::isb();

        let divider = syscon.ahbclkdiv().read();
        let slow = syscon.slowclkdiv().read();
        if scg.csr().read().scs().to_bits() != 2 || divider.div() != 0 {
            return Err(ClockError::InvalidReadback(ClockPhase::MainSource));
        }
        if slow.halt().to_bits() != 0 || slow.reset().to_bits() != 0 {
            return Err(ClockError::InvalidReadback(ClockPhase::SlowDivider));
        }
        let system = 12_000_000 / (u32::from(divider.div()) + 1);
        *CLOCKS.borrow_ref_mut(cs) = Some(Clocks {
            main_clk: Clock { frequency: 12_000_000 },
            cpu_system_clk: Clock { frequency: system },
            slow_clk: Clock { frequency: system / 4 },
            clk_1m: Clock { frequency: 1_000_000 },
        });
        Ok(())
    })
}

/// Read the clock snapshot only after successful hardware initialization.
pub fn with_clocks<R: 'static>(f: impl FnOnce(&Clocks) -> R) -> Option<R> {
    critical_section::with(|cs| CLOCKS.borrow_ref(cs).as_ref().map(f))
}

pub(crate) trait Gate {
    /// Enable the register clock without disturbing inherited peripheral state.
    /// An already asserted reset is incompatible with non-destructive enabling.
    fn enable() -> Result<(), ClockError>;

    /// No active user may own this peripheral while its reset is pulsed.
    #[cfg(time_driver_ostimer)]
    unsafe fn initialize() -> Result<(), ClockError>;
}

/// N resets are active-high, with clocks disabled for the reset pulse.
#[doc(hidden)]
#[macro_export]
macro_rules! impl_mcxn_gate {
    ($name:ident, $clk_reg:ident, $rst_reg:ident, $clk_bit:ident, $rst_bit:ident, $group:expr) => {
        impl $crate::clocks::Gate for $crate::peripherals::$name {
            fn enable() -> Result<(), $crate::clocks::ClockError> {
                ::paste::paste! {
                    let syscon = $crate::pac::SYSCON0;
                    if syscon.$rst_reg().read().$rst_bit().to_bits() != 0 {
                        return Err($crate::clocks::ClockError::InvalidReadback(
                            $crate::clocks::ClockPhase::PeripheralReset,
                        ));
                    }
                    let clock = syscon.$clk_reg();
                    let mut mask = clock.read();
                    mask.0 = 0;
                    mask.[<set_ $clk_bit>](1u8.into());
                    syscon.ahbclkctrlset($group).write(|w| w.set_data(mask.0));
                    $crate::clocks::wait($crate::clocks::ClockPhase::PeripheralGate,
                        || clock.read().$clk_bit().to_bits() == 1)?;
                    cortex_m::asm::dsb();
                    cortex_m::asm::isb();
                    Ok(())
                }
            }

            #[cfg(time_driver_ostimer)]
            unsafe fn initialize() -> Result<(), $crate::clocks::ClockError> {
                ::paste::paste! {
                    let syscon = $crate::pac::SYSCON0;
                    let clock = syscon.$clk_reg();
                    let reset = syscon.$rst_reg();
                    let mut clock_mask = clock.read();
                    clock_mask.0 = 0;
                    clock_mask.[<set_ $clk_bit>](1u8.into());
                    let mut reset_mask = reset.read();
                    reset_mask.0 = 0;
                    reset_mask.[<set_ $rst_bit>](1u8.into());
                    syscon.ahbclkctrlclr($group).write(|w| w.set_data(clock_mask.0));
                    $crate::clocks::wait($crate::clocks::ClockPhase::PeripheralGate,
                        || clock.read().$clk_bit().to_bits() == 0)?;
                    syscon.presetctrlset($group).write(|w| w.set_data(reset_mask.0));
                    $crate::clocks::wait($crate::clocks::ClockPhase::PeripheralReset,
                        || reset.read().$rst_bit().to_bits() == 1)?;
                    cortex_m::asm::dsb();
                    syscon.presetctrlclr($group).write(|w| w.set_data(reset_mask.0));
                    $crate::clocks::wait($crate::clocks::ClockPhase::PeripheralReset,
                        || reset.read().$rst_bit().to_bits() == 0)?;
                    <Self as $crate::clocks::Gate>::enable()
                }
            }
        }
    };
}

pub(crate) fn init_gpio_clocks() -> Result<(), ClockError> {
    // Resetting whole GPIO/PORT blocks would discard inherited access policy,
    // protection locks, and pad state beyond the three owned RGB pins.
    <peripherals::PORT0 as Gate>::enable()?;
    <peripherals::GPIO0 as Gate>::enable()?;
    <peripherals::PORT1 as Gate>::enable()?;
    <peripherals::GPIO1 as Gate>::enable()?;
    Ok(())
}

#[cfg(time_driver_ostimer)]
pub(crate) fn enable_ostimer_clock() -> Result<(), ClockError> {
    // SYSCON mux reset selects no clock. clk_1m is an independent FRO12 output.
    pac::SYSCON0.ostimerclksel().modify(|w| w.set_sel(2u8.into()));
    if pac::SYSCON0.ostimerclksel().read().sel().to_bits() != 2 {
        return Err(ClockError::InvalidReadback(ClockPhase::PeripheralGate));
    }
    unsafe { <peripherals::OSTIMER0 as Gate>::initialize() }
}
