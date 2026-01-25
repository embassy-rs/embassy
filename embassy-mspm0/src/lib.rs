#![no_std]
#![allow(unsafe_op_in_unsafe_fn)]
// Doc feature labels can be tested locally by running RUSTDOCFLAGS="--cfg=docsrs" cargo +nightly doc
#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg_hide), doc(cfg_hide(doc, docsrs)))]
#![cfg_attr(
    docsrs,
    doc = "<div style='padding:30px;background:#810;color:#fff;text-align:center;'><p>You might want to <a href='https://docs.embassy.dev/embassy-mspm0'>browse the `embassy-mspm0` documentation on the Embassy website</a> instead.</p><p>The documentation here on `docs.rs` is built for a single chip only, while on the Embassy website you can pick your exact chip from the top menu. Available peripherals and their APIs change depending on the chip.</p></div>\n\n"
)]
#![doc = include_str!("../README.md")]

// These mods MUST go first, so that the others see the macros.
pub(crate) mod fmt;
mod macros;

pub mod adc;
pub mod dma;
pub mod gpio;
// TODO: I2C unicomm
#[cfg(not(unicomm))]
pub mod i2c;
#[cfg(not(unicomm))]
pub mod i2c_target;
#[cfg(any(mspm0g150x, mspm0g151x, mspm0g350x, mspm0g351x))]
pub mod mathacl;
pub mod timer;
#[cfg(any(mspm0g150x, mspm0g151x, mspm0g350x, mspm0g351x, mspm0l122x, mspm0l222x))]
pub mod trng;
// TODO: UART unicomm
#[cfg(not(unicomm))]
pub mod uart;
pub mod wwdt;

#[cfg(canfd)]
pub mod can;

/// Operating modes for peripherals.
pub mod mode {
    trait SealedMode {}

    /// Operating mode for a peripheral.
    #[allow(private_bounds)]
    pub trait Mode: SealedMode {}

    /// Blocking mode.
    pub struct Blocking;
    impl SealedMode for Blocking {}
    impl Mode for Blocking {}

    /// Async mode.
    pub struct Async;
    impl SealedMode for Async {}
    impl Mode for Async {}
}

#[cfg(feature = "_time-driver")]
mod time_driver;

pub(crate) mod _generated {
    #![allow(dead_code)]
    #![allow(unused_imports)]
    #![allow(non_snake_case)]
    #![allow(missing_docs)]

    include!(concat!(env!("OUT_DIR"), "/_generated.rs"));
}

// Reexports
pub(crate) use _generated::gpio_pincm;
pub use _generated::{Peripherals, peripherals};
pub use embassy_hal_internal::Peri;
#[cfg(feature = "unstable-pac")]
pub use mspm0_metapac as pac;
#[cfg(not(feature = "unstable-pac"))]
pub(crate) use mspm0_metapac as pac;

pub use crate::_generated::interrupt;

/// Macro to bind interrupts to handlers.
///
/// This defines the right interrupt handlers, and creates a unit struct (like `struct Irqs;`)
/// and implements the right [`Binding`]s for it. You can pass this struct to drivers to
/// prove at compile-time that the right interrupts have been bound.
///
/// Example of how to bind one interrupt:
///
/// ```rust,ignore
/// use embassy_nrf::{bind_interrupts, spim, peripherals};
///
/// bind_interrupts!(
///     /// Binds the SPIM3 interrupt.
///     struct Irqs {
///         SPIM3 => spim::InterruptHandler<peripherals::SPI3>;
///     }
/// );
/// ```
///
/// Example of how to bind multiple interrupts in a single macro invocation:
///
/// ```rust,ignore
/// use embassy_nrf::{bind_interrupts, spim, twim, peripherals};
///
/// bind_interrupts!(struct Irqs {
///     SPIM3 => spim::InterruptHandler<peripherals::SPI3>;
///     TWISPI0 => twim::InterruptHandler<peripherals::TWISPI0>;
/// });
/// ```

// developer note: this macro can't be in `embassy-hal-internal` due to the use of `$crate`.
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

/// `embassy-mspm0` global configuration.
#[non_exhaustive]
#[derive(Clone, Copy)]
pub struct Config {
    // TODO: OSC configuration.
    /// The size of DMA block transfer burst.
    ///
    /// If this is set to a value
    pub dma_burst_size: dma::BurstSize,

    /// Whether the DMA channels are used in a fixed priority or a round robin fashion.
    ///
    /// If [`false`], the DMA priorities are fixed.
    ///
    /// If [`true`], after a channel finishes a transfer it becomes the lowest priority.
    pub dma_round_robin: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            dma_burst_size: dma::BurstSize::Complete,
            dma_round_robin: false,
        }
    }
}

/// Read the PLL startup calibration values from the FACTORY region,
/// given an expected f_loopin frequency.
/// Returns SYSPLLPARAM0, SYSPLLPARAM1.
#[cfg(all(sysctl_syspll, canfd))]
fn load_pll_values(f_loopin: u32) -> Option<(u32, u32)> {
    // TODO: should these be found via the PAC instead of hard-coded?
    // From looking at the G series TRM, these addresses are constant,
    // but it still feels wrong.
    // Taken from table "Table 1-135. FACTORYREGION_TYPEG Registers"
    // (constants identical to Table 1-116. FACTORYREGION_TYPEA Registers for PLL.)
    let addrs: (u32, u32) = match f_loopin {
        4_000_000..=8_000_000 => (0x41C4_001C, 0x41C4_0020),
        8_000_001..=16_000_000 => (0x41C4_0024, 0x41C4_0028),
        16_000_001..=32_000_000 => (0x41C4_002C, 0x41C4_0030),
        32_000_001..48_000_000 => (0x41C4_0034, 0x41C4_0038),
        _ => {
            return None;
        }
    };

    let param0 = unsafe { core::ptr::read_volatile(addrs.0 as *const u32) };
    let param1 = unsafe { core::ptr::read_volatile(addrs.1 as *const u32) };

    Some((param0, param1))
}

/// This is a minimal initialization for the PLL
/// to run at 32MHz (matching MCLK), sources from SYSCLK
/// This can then be used to feed the CANFD peripheral
/// it's functional clock (fclk <= mclk).
#[cfg(all(sysctl_syspll, canfd))]
fn enable_pll() {
    if !pac::SYSCTL.clkstatus().read().sysplloff() {
        pac::SYSCTL.hsclken().modify(|w| {
            w.set_syspllen(false);
        });
        // must wait for a "stable dead state" before re-enabling.
        while !pac::SYSCTL.clkstatus().read().sysplloff() {
            cortex_m::asm::delay(16);
        }
    }

    // We will set the pre-divider to 1, and the VCO divider to 4, causing:
    // Reference clk: 32MHz.
    // Divided by pre-divider: 32MHz
    // Multiplied by VCO divider: 128MHz
    // Divided by clk1div: 32MHz
    // Produces a CANFD functional clock of 32MHz (matching MCLK sourced from SYSOSC.)
    //
    // This is a very convoluted way to route the SYSOSC to the CANFD functional clock,
    // but the only way supported with the clock tree we have.
    // This is a stopgap to get CANFD working without too many external components.

    pac::SYSCTL.syspllcfg0().modify(|w| {
        w.set_syspllref(pac::sysctl::vals::Syspllref::SYSOSC); // SYSOSC as PLL reference.
        w.set_enableclk1(true); // SYSPLLCLK1 goes to CANFD as functional clock.
        w.set_rdivclk1(pac::sysctl::vals::Rdivclk1::CLK1DIV4);
    });

    pac::SYSCTL.syspllcfg1().modify(|w| {
        w.set_pdiv(pac::sysctl::vals::Pdiv::REFDIV1); // Divide input clock by 1
        w.set_qdiv(pac::sysctl::vals::Qdiv::from(3)); // Register value 3 results in /4 (causes VCO to be 4x reference clock = 128MHZ in this case)
    });

    let params = load_pll_values(32_000_000).unwrap();
    pac::SYSCTL
        .syspllparam0()
        .write_value(pac::sysctl::regs::Syspllparam0(params.0));
    pac::SYSCTL
        .syspllparam1()
        .write_value(pac::sysctl::regs::Syspllparam1(params.1));

    pac::SYSCTL.hsclken().modify(|w| {
        w.set_syspllen(true);
    });

    while !pac::SYSCTL.clkstatus().read().syspllgood() {
        cortex_m::asm::delay(16);
    }

    // Due to SYSPLL_ERR_01, we need to check the resulting frequency with FCC to make sure the PLL locked correctly.
    // We'll measure how many ticks of the SYSPLLCLK1 output occur for each tick of the LFCLK (32.768 KHz nominally)
    // to determine the actual output frequency of the PLL.
    // If it is not approximately correct, we will disable and re-enable the PLL.
    loop {
        pac::SYSCTL.genclkcfg().modify(|w| {
            w.set_fcctrigcnt(0);
            w.set_fcctrigsrc(pac::sysctl::vals::Fcctrigsrc::LFCLK);
            w.set_fccselclk(pac::sysctl::vals::Fccselclk::SYSPLLCLK1);
            w.set_fcclvltrig(pac::sysctl::vals::Fcclvltrig::RISE2RISE);
        });
        pac::SYSCTL.fcccmd().write(|w| {
            w.set_key(pac::sysctl::vals::FcccmdKey::KEY);
            w.set_go(true);
        });

        while !pac::SYSCTL.clkstatus().read().fccdone() {
            cortex_m::asm::delay(16);
        }

        let count = pac::SYSCTL.fcc().read().data() as i32; // safety: << 32 bits, safe to convert.

        trace!("Detected PLL output frequency: {}", count * 32768);

        if count.abs_diff(32_000_000 / 32_768) < 10 {
            break;
        }

        warn!("PLL failed to lock - retrying. Actual frequency: {}", count * 32768);

        pac::SYSCTL.hsclken().modify(|w| {
            w.set_syspllen(false);
        });

        while !pac::SYSCTL.clkstatus().read().sysplloff() {
            cortex_m::asm::delay(16);
        }

        pac::SYSCTL.hsclken().modify(|w| {
            w.set_syspllen(true);
        });

        while !pac::SYSCTL.clkstatus().read().syspllgood() {
            cortex_m::asm::delay(16);
        }
    }
}

pub fn init(config: Config) -> Peripherals {
    critical_section::with(|cs| {
        let peripherals = Peripherals::take_with_cs(cs);

        // TODO: Further clock configuration

        pac::SYSCTL.mclkcfg().modify(|w| {
            // Enable MFCLK
            w.set_usemftick(true);
            // MDIV must be disabled if MFCLK is enabled.
            w.set_mdiv(0);
        });

        // Enable MFCLK for peripheral use
        //
        // TODO: Optional?
        pac::SYSCTL.genclken().modify(|w| {
            w.set_mfpclken(true);
        });

        pac::SYSCTL.borthreshold().modify(|w| {
            w.set_level(0);
        });

        // On parts which have both canfd and sysctl_syspll, enable syspll at 32MHz (match MCLK)
        // This is a bit of a hack - canfd needs a functional clock from HFXT or SYSPLL,
        // and syspll can be configured on every canfd-supported part.
        // syspll is unlikely to be accurate enough for higher-bitrate CAN,
        // and more complete clock tree support will eventually be required.
        // We don't actually need to enable this unless there are any consumers,
        // but we don't yet have the infrastructure to determine that or not.
        #[cfg(all(sysctl_syspll, canfd))]
        enable_pll();

        gpio::init(pac::GPIOA);
        #[cfg(gpio_pb)]
        gpio::init(pac::GPIOB);
        #[cfg(gpio_pc)]
        gpio::init(pac::GPIOC);

        _generated::enable_group_interrupts(cs);

        #[cfg(any(mspm0c110x, mspm0l110x))]
        unsafe {
            use crate::_generated::interrupt::typelevel::Interrupt;
            crate::interrupt::typelevel::GPIOA::enable();
        }

        // SAFETY: Peripherals::take_with_cs will only be run once or panic.
        unsafe { dma::init(cs, config.dma_burst_size, config.dma_round_robin) };

        #[cfg(feature = "_time-driver")]
        time_driver::init(cs);

        peripherals
    })
}

pub(crate) mod sealed {
    #[allow(dead_code)]
    pub trait Sealed {}
}

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

/// Reset cause values from SYSCTL.RSTCAUSE register.
/// Based on MSPM0 L-series Technical Reference Manual Table 2-9 and
/// MSPM0 G-series Technical Reference Manual Table 2-12.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ResetCause {
    /// No reset since last read
    NoReset,
    /// VDD < POR- violation, PMU trim parity fault, or SHUTDNSTOREx parity fault
    PorHwFailure,
    /// NRST pin reset (>1s)
    PorExternalNrst,
    /// Software-triggered POR
    PorSwTriggered,
    /// VDD < BOR- violation
    BorSupplyFailure,
    /// Wake from SHUTDOWN
    BorWakeFromShutdown,
    /// Non-PMU trim parity fault
    #[cfg(not(any(
        mspm0c110x,
        mspm0c1105_c1106,
        mspm0g110x,
        mspm0g150x,
        mspm0g151x,
        mspm0g310x,
        mspm0g350x,
        mspm0g351x
    )))]
    BootrstNonPmuParityFault,
    /// Fatal clock fault
    BootrstClockFault,
    /// Software-triggered BOOTRST
    BootrstSwTriggered,
    /// NRST pin reset (<1s)
    BootrstExternalNrst,
    /// WWDT0 violation
    BootrstWwdt0Violation,
    /// WWDT1 violation (G-series only)
    #[cfg(any(mspm0g110x, mspm0g150x, mspm0g151x, mspm0g310x, mspm0g350x, mspm0g351x, mspm0g518x))]
    SysrstWwdt1Violation,
    /// BSL exit (if present)
    SysrstBslExit,
    /// BSL entry (if present)
    SysrstBslEntry,
    /// Uncorrectable flash ECC error (if present)
    #[cfg(not(any(mspm0c110x, mspm0c1105_c1106, mspm0g351x, mspm0g151x)))]
    SysrstFlashEccError,
    /// CPU lockup violation
    SysrstCpuLockupViolation,
    /// Debug-triggered SYSRST
    SysrstDebugTriggered,
    /// Software-triggered SYSRST
    SysrstSwTriggered,
    /// Debug-triggered CPURST
    CpurstDebugTriggered,
    /// Software-triggered CPURST
    CpurstSwTriggered,
}

/// Read the reset cause from the SYSCTL.RSTCAUSE register.
///
/// This function reads the reset cause register which indicates why the last
/// system reset occurred. The register is automatically cleared after being read,
/// so this should be called only once per application startup.
///
/// If the reset cause is not recognized, an `Err` containing the raw value is returned.
#[must_use = "Reading reset cause will clear it"]
pub fn read_reset_cause() -> Result<ResetCause, u8> {
    let cause_raw = pac::SYSCTL.rstcause().read().id();

    use ResetCause::*;
    use pac::sysctl::vals::Id;

    match cause_raw {
        Id::NORST => Ok(NoReset),
        Id::PORHWFAIL => Ok(PorHwFailure),
        Id::POREXNRST => Ok(PorExternalNrst),
        Id::PORSW => Ok(PorSwTriggered),
        Id::BORSUPPLY => Ok(BorSupplyFailure),
        Id::BORWAKESHUTDN => Ok(BorWakeFromShutdown),
        #[cfg(not(any(
            mspm0c110x,
            mspm0c1105_c1106,
            mspm0g110x,
            mspm0g150x,
            mspm0g151x,
            mspm0g310x,
            mspm0g350x,
            mspm0g351x,
            mspm0g518x,
        )))]
        Id::BOOTNONPMUPARITY => Ok(BootrstNonPmuParityFault),
        Id::BOOTCLKFAIL => Ok(BootrstClockFault),
        Id::BOOTSW => Ok(BootrstSwTriggered),
        Id::BOOTEXNRST => Ok(BootrstExternalNrst),
        Id::BOOTWWDT0 => Ok(BootrstWwdt0Violation),
        Id::SYSBSLEXIT => Ok(SysrstBslExit),
        Id::SYSBSLENTRY => Ok(SysrstBslEntry),
        #[cfg(any(mspm0g110x, mspm0g150x, mspm0g151x, mspm0g310x, mspm0g350x, mspm0g351x, mspm0g518x))]
        Id::SYSWWDT1 => Ok(SysrstWwdt1Violation),
        #[cfg(not(any(mspm0c110x, mspm0c1105_c1106, mspm0g351x, mspm0g151x)))]
        Id::SYSFLASHECC => Ok(SysrstFlashEccError),
        Id::SYSCPULOCK => Ok(SysrstCpuLockupViolation),
        Id::SYSDBG => Ok(SysrstDebugTriggered),
        Id::SYSSW => Ok(SysrstSwTriggered),
        Id::CPUDBG => Ok(CpurstDebugTriggered),
        Id::CPUSW => Ok(CpurstSwTriggered),
        other => Err(other as u8),
    }
}
