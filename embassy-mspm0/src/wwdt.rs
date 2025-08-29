//! Window Watchdog Timer (WWDT) driver.
//!
//! This HAL implements a basic window watchdog timer with handles.

#![macro_use]

use embassy_hal_internal::PeripheralType;

use crate::pac::wwdt::{vals, Wwdt as Regs};
use crate::pac::{self};
use crate::Peri;

/// Possible watchdog timeout values.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Timeout {
    USec1950,
    USec3910,
    USec5860,
    USec7810,
    USec9770,
    USec11720,
    USec13670,
    USec15630,
    USec23440,
    USec31250,
    USec32250,
    USec39060,
    USec46880,
    USec54690,
    USec62500,
    USec93750,
    USec125000,
    USec156250,
    USec187500,
    USec218750,
    MSec130,
    MSec250,
    MSec380,
    MSec500,
    MSec630,
    MSec750,
    MSec880,
    Sec1,
    Sec2,
    Sec3,
    Sec4,
    Sec5,
    Sec6,
    Sec7,
    Sec8,
    Sec16,
    Sec24,
    Sec32,
    Sec40,
    Sec48,
    Sec56,
    Sec64,
    Sec128,  // 2.13 min
    Sec192,  // 3.20 min
    Sec256,  // 4.27 min
    Sec320,  // 5.33 min
    Sec384,  // 6.40 min
    Sec448,  // 7.47 min
    Sec512,  // 8.53 min
    Sec1024, // 17.07 min
    Sec2048, // 34.13 min
    Sec3072, // 51.20 min
    Sec4096, // 68.27 min
    Sec5120, // 85.33 min
    Sec6144, // 102.40 min
    Sec7168, // 119.47 min
    Sec8192, // 136.53 min
}

impl Timeout {
    fn get_period(self) -> vals::Per {
        match self {
            //  period count is 2**25
            Self::Sec1024
            | Self::Sec2048
            | Self::Sec3072
            | Self::Sec4096
            | Self::Sec5120
            | Self::Sec6144
            | Self::Sec7168
            | Self::Sec8192 => vals::Per::EN_25,
            //  period count is 2**21
            Self::Sec64
            | Self::Sec128
            | Self::Sec192
            | Self::Sec256
            | Self::Sec320
            | Self::Sec384
            | Self::Sec448
            | Self::Sec512 => vals::Per::EN_21,
            //  period count is 2**18
            Self::Sec8 | Self::Sec16 | Self::Sec24 | Self::Sec32 | Self::Sec40 | Self::Sec48 | Self::Sec56 => {
                vals::Per::EN_18
            }
            //  period count is 2**15
            Self::Sec1 | Self::Sec2 | Self::Sec3 | Self::Sec4 | Self::Sec5 | Self::Sec6 | Self::Sec7 => {
                vals::Per::EN_15
            }
            //  period count is 2**12
            Self::MSec130
            | Self::MSec250
            | Self::MSec380
            | Self::MSec500
            | Self::MSec630
            | Self::MSec750
            | Self::MSec880 => vals::Per::EN_12,
            //  period count is 2**10
            Self::USec31250
            | Self::USec62500
            | Self::USec93750
            | Self::USec125000
            | Self::USec156250
            | Self::USec187500
            | Self::USec218750 => vals::Per::EN_10,
            //  period count is 2**8
            Self::USec7810
            | Self::USec15630
            | Self::USec23440
            | Self::USec32250
            | Self::USec39060
            | Self::USec46880
            | Self::USec54690 => vals::Per::EN_8,
            //  period count is 2**6
            Self::USec1950 | Self::USec3910 | Self::USec5860 | Self::USec9770 | Self::USec11720 | Self::USec13670 => {
                vals::Per::EN_6
            }
        }
    }

    fn get_clkdiv(self) -> u8 {
        match self {
            //  divide by 1
            Self::USec1950
            | Self::USec7810
            | Self::USec31250
            | Self::MSec130
            | Self::Sec1
            | Self::Sec8
            | Self::Sec64
            | Self::Sec1024 => 0u8,
            //  divide by 2
            Self::USec3910
            | Self::USec15630
            | Self::USec62500
            | Self::MSec250
            | Self::Sec2
            | Self::Sec16
            | Self::Sec128
            | Self::Sec2048 => 1u8,
            //  divide by 3
            Self::USec5860
            | Self::USec23440
            | Self::USec93750
            | Self::MSec380
            | Self::Sec3
            | Self::Sec24
            | Self::Sec192
            | Self::Sec3072 => 2u8,
            //  divide by 4
            Self::USec32250
            | Self::USec125000
            | Self::MSec500
            | Self::Sec4
            | Self::Sec32
            | Self::Sec256
            | Self::Sec4096 => 3u8,
            //  divide by 5
            Self::USec9770
            | Self::USec39060
            | Self::USec156250
            | Self::MSec630
            | Self::Sec5
            | Self::Sec40
            | Self::Sec320
            | Self::Sec5120 => 4u8,
            //  divide by 6
            Self::USec11720
            | Self::USec46880
            | Self::USec187500
            | Self::MSec750
            | Self::Sec6
            | Self::Sec48
            | Self::Sec384
            | Self::Sec6144 => 5u8,
            //  divide by 7
            Self::USec13670
            | Self::USec54690
            | Self::USec218750
            | Self::MSec880
            | Self::Sec7
            | Self::Sec56
            | Self::Sec448
            | Self::Sec7168 => 6u8,
            //  divide by 8
            Self::Sec512 | Self::Sec8192 => 7u8,
        }
    }
}

/// Timeout percentage that is treated as "too early" and generates violation.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ClosedWindowPercentage {
    // window period is not used
    Zero,
    // 12.5% percents
    Twelve,
    // 18.75% percents
    Eighteen,
    // 25% percents
    TwentyFive,
    // 50% percents
    Fifty,
    // 75% percents
    SeventyFive,
    // 81.25% percents
    EightyOne,
    // 87.5% percents
    EightySeven,
}

impl ClosedWindowPercentage {
    fn get_native_size(self) -> vals::Window {
        match self {
            Self::Zero => vals::Window::SIZE_0,
            Self::Twelve => vals::Window::SIZE_12,
            Self::Eighteen => vals::Window::SIZE_18,
            Self::TwentyFive => vals::Window::SIZE_25,
            Self::Fifty => vals::Window::SIZE_50,
            Self::SeventyFive => vals::Window::SIZE_75,
            Self::EightyOne => vals::Window::SIZE_81,
            Self::EightySeven => vals::Window::SIZE_87,
        }
    }
}

#[non_exhaustive]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
/// Watchdog Config
pub struct Config {
    /// Watchdog timeout
    pub timeout: Timeout,

    /// closed window percentage
    pub closed_window: ClosedWindowPercentage,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            timeout: Timeout::Sec1,
            closed_window: ClosedWindowPercentage::Zero,
        }
    }
}

pub struct Watchdog {
    regs: &'static Regs,
}

impl Watchdog {
    /// Watchdog initialization.
    pub fn new<T: Instance>(_instance: Peri<T>, config: Config) -> Self {
        // Init power for watchdog
        T::regs().gprcm(0).rstctl().write(|w| {
            w.set_resetstkyclr(true);
            w.set_resetassert(true);
            w.set_key(vals::ResetKey::KEY);
        });

        // Enable power for watchdog
        T::regs().gprcm(0).pwren().write(|w| {
            w.set_enable(true);
            w.set_key(vals::PwrenKey::KEY);
        });

        // init delay, 16 cycles
        cortex_m::asm::delay(16);

        critical_section::with(|_| {
            // make sure watchdog triggers BOOTRST
            pac::SYSCTL.systemcfg().modify(|w| {
                if *T::regs() == pac::WWDT0 {
                    w.set_wwdtlp0rstdis(false);
                }

                #[cfg(any(mspm0g110x, mspm0g150x, mspm0g151x, mspm0g310x, mspm0g350x, mspm0g351x))]
                if *T::regs() == pac::WWDT1 {
                    w.set_wwdtlp1rstdis(false);
                }
            });
        });

        //init watchdog
        T::regs().wwdtctl0().write(|w| {
            w.set_clkdiv(config.timeout.get_clkdiv());
            w.set_per(config.timeout.get_period());
            w.set_mode(vals::Mode::WINDOW);
            w.set_window0(config.closed_window.get_native_size());
            w.set_window1(vals::Window::SIZE_0);
            w.set_key(vals::Wwdtctl0Key::KEY);
        });

        // Set Window0 as active window
        T::regs().wwdtctl1().write(|w| {
            w.set_winsel(vals::Winsel::WIN0);
            w.set_key(vals::Wwdtctl1Key::KEY);
        });

        Self { regs: T::regs() }
    }

    /// Pet (reload, refresh) the watchdog.
    pub fn pet(&mut self) {
        self.regs.wwdtcntrst().write(|w| {
            w.set_restart(vals::WwdtcntrstRestart::RESTART);
        });
    }
}

pub(crate) trait SealedInstance {
    fn regs() -> &'static Regs;
}

/// WWDT instance trait
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType {}

macro_rules! impl_wwdt_instance {
    ($instance: ident) => {
        impl crate::wwdt::SealedInstance for crate::peripherals::$instance {
            fn regs() -> &'static crate::pac::wwdt::Wwdt {
                &crate::pac::$instance
            }
        }

        impl crate::wwdt::Instance for crate::peripherals::$instance {}
    };
}
