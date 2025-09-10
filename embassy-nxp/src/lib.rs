#![no_std]

// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;

pub mod gpio;
#[cfg(feature = "lpc55-core0")]
pub mod pint;
#[cfg(feature = "lpc55-core0")]
pub mod usart;

#[cfg(feature = "_time_driver")]
#[cfg_attr(feature = "time-driver-pit", path = "time_driver/pit.rs")]
#[cfg_attr(feature = "time-driver-rtc", path = "time_driver/rtc.rs")]
mod time_driver;

// This mod MUST go last, so that it sees all the `impl_foo!` macros
#[cfg_attr(feature = "lpc55-core0", path = "chips/lpc55.rs")]
#[cfg_attr(feature = "mimxrt1011", path = "chips/mimxrt1011.rs")]
#[cfg_attr(feature = "mimxrt1062", path = "chips/mimxrt1062.rs")]
mod chip;

#[cfg(feature = "unstable-pac")]
pub use chip::pac;
#[cfg(not(feature = "unstable-pac"))]
pub(crate) use chip::pac;
pub use chip::{peripherals, Peripherals};
pub use embassy_hal_internal::{Peri, PeripheralType};

/// Initialize the `embassy-nxp` HAL with the provided configuration.
///
/// This returns the peripheral singletons that can be used for creating drivers.
///
/// This should only be called once and at startup, otherwise it panics.
pub fn init(_config: config::Config) -> Peripherals {
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

    #[cfg(any(feature = "lpc55-core0", rt1xxx))]
    gpio::init();

    #[cfg(feature = "lpc55-core0")]
    pint::init();

    #[cfg(feature = "_time_driver")]
    time_driver::init();

    peripherals
}

/// HAL configuration for the NXP board.
pub mod config {
    #[derive(Default)]
    pub struct Config {}
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

impl_mode!(Blocking);
