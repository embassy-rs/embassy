//! XSPI I/O manager (XSPIM).
//!
//! The XSPIM maps the XSPI controllers onto the I/O ports. In the multiplexed
//! mapping two controllers drive the same port and are time-multiplexed by an
//! arbiter, which is required for two controllers to share a single external
//! memory (for example read-while-write).
//!
//! The multiplexed mapping is represented by [`Xspim`], which also owns the NCS
//! pin shared by both controllers. The controllers borrow it for as long as
//! they live, so the NCS pin cannot be released while they are in use and both
//! controllers are guaranteed to use the same NCS.
//!
//! ```ignore
//! // Normal boot: nothing has configured the XSPIM yet.
//! let xspim = Xspim::new_multiplexed_on_port1(p.PO0, xspim::Config::default());
//! let primary = Xspi::new_blocking_xspi_dqs_xspim(p.XSPI1, /* pins */, &xspim, config);
//! let secondary = Xspi::new_blocking_xspi_secondary(p.XSPI2, &xspim, config);
//!
//! // Executing in place from a bootloader-configured XSPIM: write nothing,
//! // and leave the pins alone.
//! let xspim = unsafe { Xspim::assume_configured() };
//! ```
//!
//! # XSPI3 (STM32N6): not supported yet
//!
//! On the N6, XSPI3 sits on the port that XSPI1 and XSPI2 do *not* use. It is
//! never shared, but it is only usable while the XSPIM is multiplexed
//! (`MUXEN = 1`), which XSPI1/XSPI2 must have configured first. In the
//! reference manual's terms, XSPI3 is on port 2 when XSPI1/2 are on port 1 and
//! on port 1 when XSPI1/2 are on port 2.
//!

use core::marker::PhantomData;

use crate::Peri;
use crate::gpio::{AfType, Flex, OutputType, Pull, Speed};
use crate::pac::XSPIM;
use crate::xspi::{Instance, NCSEither};

/// XSPIM configuration.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Config {
    /// Minimum number of clock cycles between the active XSPI releasing the
    /// shared bus (falling REQ) and granting it to the other (rising ACK).
    pub req2ack_time: u8,
}

impl Default for Config {
    fn default() -> Self {
        Self { req2ack_time: 1 }
    }
}

/// XSPI I/O manager in multiplexed mode: two controllers share one port.
///
/// Dropping the handle releases the NCS pin it owns, so it must outlive the
/// controllers created from it; the controllers borrow it to enforce this.
///
/// `T` is the primary controller, the one wired to the physical pins. The
/// secondary controller is the other one, see [`SecondaryOf`].
pub struct Xspim<'d, T: Instance> {
    _ncs: Option<Flex<'d>>,
    _primary: PhantomData<T>,
}

/// An XSPI controller that can share a multiplexed port with the primary
/// controller `P`.
pub trait SecondaryOf<P: Instance>: Instance {}

#[cfg(peri_xspi2)]
impl SecondaryOf<crate::peripherals::XSPI1> for crate::peripherals::XSPI2 {}
#[cfg(peri_xspi2)]
impl SecondaryOf<crate::peripherals::XSPI2> for crate::peripherals::XSPI1 {}

impl<'d, T: Instance> Xspim<'d, T> {
    fn new_inner(ncs: Peri<'d, impl NCSEither<T>>, on_port2: bool, config: Config) -> Self {
        #[cfg(pwr_h7rs)]
        crate::pac::PWR.csr2().modify(|r| {
            r.set_en_xspim1(true);
            r.set_en_xspim2(true);
        });
        enable_clock();

        // Both controllers must target the same NCS in multiplexed mode.
        let ncs_ovr = ncs.sel() != 0;
        XSPIM.cr().modify(|w| {
            w.set_muxen(true);
            w.set_mode(on_port2);
            w.set_req2ack_time(config.req2ack_time);
            // H7RS and N6: enable chip select override (required for proper NCS routing)
            #[cfg(any(stm32h7rs, stm32n6))]
            w.set_cssel_ovr_en(true);
            w.set_cssel_ovr_o1(ncs_ovr);
            w.set_cssel_ovr_o2(ncs_ovr);
        });

        Self {
            _ncs: new_pin!(
                ncs,
                AfType::output_pull(OutputType::PushPull, Speed::VeryHigh, Pull::Up)
            ),
            _primary: PhantomData,
        }
    }

    /// Configure the XSPIM with both controllers multiplexed onto port 1.
    ///
    /// `T` is inferred from the NCS pin and must be the controller later
    /// created with the pins, the primary.
    ///
    /// All XSPI controllers must be disabled while the mapping is written, so
    /// call this before creating any `Xspi` driver.
    pub fn new_multiplexed_on_port1(ncs: Peri<'d, impl NCSEither<T>>, config: Config) -> Self {
        Self::new_inner(ncs, false, config)
    }

    /// Configure the XSPIM with both controllers multiplexed onto port 2.
    ///
    /// See [`Xspim::new_multiplexed_on_port1`].
    pub fn new_multiplexed_on_port2(ncs: Peri<'d, impl NCSEither<T>>, config: Config) -> Self {
        Self::new_inner(ncs, true, config)
    }

    /// Create a handle to a multiplexed XSPIM that was already configured,
    /// without writing any register or touching any pin.
    ///
    /// Use this when the mapping was set up earlier (for example by a first
    /// stage bootloader) and an XSPI controller is already running, such as
    /// when executing in place from memory-mapped flash on the primary
    /// controller. Writing the XSPIM or reconfiguring the NCS pin then could
    /// stall instruction fetch.
    ///
    /// # Safety
    ///
    /// The XSPIM must be programmed in a multiplexed mapping, with both
    /// controllers overriding to the same NCS, and the NCS pin must stay
    /// configured for as long as the controllers are used.
    pub unsafe fn assume_configured() -> Self {
        Self {
            _ncs: None,
            _primary: PhantomData,
        }
    }

    /// The NCS (0 = NCS1, 1 = NCS2) shared by both controllers, as programmed
    /// in the hardware.
    pub(super) fn cssel(&self) -> u8 {
        XSPIM.cr().read().cssel_ovr_o1() as u8
    }
}

/// Enable the XSPIM bus clock. Idempotent.
fn enable_clock() {
    #[cfg(rcc_h7rs)]
    crate::pac::RCC.ahb5enr().modify(|w| w.set_iomngren(true));
    #[cfg(rcc_n6)]
    crate::pac::RCC.ahb5enr().modify(|w| w.set_xspimen(true));
}

/// Single-controller mapping: direct, with the NCS override for `spi_idx`.
pub(super) fn configure_default(spi_idx: u8, ncs_cssel: u8) {
    assert!(spi_idx != 3, "XSPI3 is not supported in default mode");

    XSPIM.cr().modify(|w| {
        w.set_muxen(false);
        w.set_req2ack_time(1); // Match ST HAL (was 0xff, which is only relevant when muxen=true)
        // H7RS and N6: Enable chip select override (required for proper NCS routing)
        #[cfg(any(stm32h7rs, stm32n6))]
        w.set_cssel_ovr_en(true);
        // Each XSPI has its own override field in XSPIM
        match spi_idx {
            1 => w.set_cssel_ovr_o1(ncs_cssel != 0),
            2 => w.set_cssel_ovr_o2(ncs_cssel != 0),
            _ => unreachable!(), // XSPI3 rejected above
        }
    });
}
