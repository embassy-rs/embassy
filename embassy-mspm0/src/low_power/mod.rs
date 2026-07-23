//! Low-power (deep-sleep) support.
//!
//! Deep-sleep depth is gated by [`WakeGuard`](crate::sysctl::WakeGuard), which drivers hold to keep
//! the chip shallower than a given [`SleepLevel`]; the low-power executor then idles into the deepest
//! mode no guard blocks.
//!
//! Each family implements the SYSCTL power-mode sequence from the device TRM (chapter
//! "System Control (SYSCTL)" -> "Operating Modes"), cross-checked against TI driverlib's
//! `DL_SYSCTL_setPowerPolicy*`. The families are split into three behaviors:
//! - `full.rs` — STOP0/1/2 + STANDBY0/1: all G families and the supported L families.
//! - `c110x.rs` — STOP0/2 + STANDBY0/1 (no STOP1); STOP0 also clears `USELFCLK`: C-series.
//! - `h321x.rs` — STOP0/2 + STANDBY0/1 (no STOP1): H321x.

use core::sync::atomic::Ordering;

use critical_section::CriticalSection;
use pac::sysctl::vals::Dsleep;
use portable_atomic::AtomicU32;

use crate::pac;

#[cfg(any(
    mspm0l110x, mspm0l130x, mspm0l134x, mspm0l122x, mspm0l222x, mspm0g110x, mspm0g150x, mspm0g310x, mspm0g350x,
    mspm0g151x, mspm0g351x, mspm0g518x
))]
#[path = "full.rs"]
mod inner;

#[cfg(any(mspm0c110x, mspm0c1105_c1106))]
#[path = "c110x.rs"]
mod inner;

#[cfg(mspm0h321x)]
#[path = "h321x.rs"]
mod inner;

#[cfg(any(
    mspm0l110x,
    mspm0l130x,
    mspm0l134x,
    mspm0l122x,
    mspm0l222x,
    mspm0g110x,
    mspm0g150x,
    mspm0g310x,
    mspm0g350x,
    mspm0g151x,
    mspm0g351x,
    mspm0g518x,
    mspm0c110x,
    mspm0c1105_c1106,
    mspm0h321x
))]
pub use inner::{SleepMode, enter_sleep};

#[cfg(not(any(
    mspm0l110x,
    mspm0l130x,
    mspm0l134x,
    mspm0l122x,
    mspm0l222x,
    mspm0g110x,
    mspm0g150x,
    mspm0g310x,
    mspm0g350x,
    mspm0g151x,
    mspm0g351x,
    mspm0g518x,
    mspm0c110x,
    mspm0c1105_c1106,
    mspm0h321x
)))]
compile_error!("the `low-power` feature is not implemented for this chip family");

/// Deep-sleep modes. Defined in [`sysctl`](crate::sysctl) alongside the always-available
/// [`WakeGuard`](crate::sysctl::WakeGuard) that takes one; re-exported here for convenience.
pub use crate::sysctl::SleepLevel;

const LEVELS: [SleepLevel; 5] = [
    SleepLevel::Stop0,
    SleepLevel::Stop1,
    SleepLevel::Stop2,
    SleepLevel::Standby0,
    SleepLevel::Standby1,
];

/// `SLEEP_BLOCKS[i]` counts the guards forbidding `LEVELS[i]` and every deeper mode.
static SLEEP_BLOCKS: [AtomicU32; 5] = [
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
];

/// Add a block at `level` (and every deeper mode). Paired with [`unblock`] by
/// [`WakeGuard`](crate::sysctl::WakeGuard).
pub(crate) fn block(level: SleepLevel) {
    SLEEP_BLOCKS[level as usize].fetch_add(1, Ordering::Relaxed);
}

/// Remove a block previously added at `level`.
pub(crate) fn unblock(level: SleepLevel) {
    SLEEP_BLOCKS[level as usize].fetch_sub(1, Ordering::Relaxed);
}

/// Deepest mode currently permitted, or `None` (plain `WFI`) if all deep sleep is blocked.
fn deepest_allowed() -> Option<SleepLevel> {
    for (i, blocks) in SLEEP_BLOCKS.iter().enumerate() {
        if blocks.load(Ordering::Relaxed) > 0 {
            // LEVELS[i] and everything deeper is blocked, so the deepest permitted is one shallower.
            return i.checked_sub(1).map(|j| LEVELS[j]);
        }
    }
    Some(SleepLevel::Standby1)
}

/// Enter the deepest sleep permitted by the active [`WakeGuard`](crate::sysctl::WakeGuard)s, waiting
/// for an interrupt.
///
/// Called by the low-power executor on idle. With no guards held it enters the deepest mode the
/// chip supports; a held guard caps the depth, and a guard on [`SleepLevel::Stop0`] keeps it a
/// plain `WFI`.
///
/// # Safety
/// Deep sleep powers down PD1 (and, in STANDBY, most of PD0). Any peripheral transaction that must
/// survive has to be protected by a [`WakeGuard`](crate::sysctl::WakeGuard) shallow enough to keep it
/// clocked. Until the drivers hold their own guards, the caller is responsible for this.
pub unsafe fn sleep(cs: CriticalSection) {
    match deepest_allowed() {
        None => {
            cortex_m::asm::dsb();
            cortex_m::asm::wfi();
            cortex_m::asm::isb();
        }
        Some(level) => enter_sleep(cs, inner::level_to_mode(level)),
    }
}

/// Enter SHUTDOWN, the lowest-power state. Does not return.
///
/// SHUTDOWN powers down VCORE: all SRAM is lost except the `SHUTDNSTORE` bytes, and the only wake
/// sources are a wake-capable IO event, NRST, or SWD activity. Does not return as the wake results
/// in a reset.
/// You can respond to the reset on boot using [`ResetCause::BorWakeFromShutdown`](crate::ResetCause).
///
/// # Safety
/// This is irreversible in place: CPU and peripheral state are lost and this call never returns.
//
// From the TRM: SYSCTL "Operating Modes": set `PMODECFG.DSLEEP = SHUTDOWN`, arm `SLEEPDEEP`,
// then `WFI`. This is identical across every MSPM0 family.
pub unsafe fn shutdown(_cs: CriticalSection) -> ! {
    let sysctl = pac::SYSCTL;
    sysctl.pmodecfg().modify(|w| w.set_dsleep(Dsleep::SHUTDOWN));

    let mut scb = unsafe { cortex_m::Peripherals::steal() }.SCB;
    scb.set_sleepdeep();
    cortex_m::asm::dsb();

    loop {
        cortex_m::asm::wfi();
    }
}

/// Arm ARM deep-sleep (`SLEEPDEEP`), wait for an interrupt, then clear it.
///
/// The mode-specific SYSCTL programming must already be done by the caller. `WFI` wakes on a
/// pending enabled interrupt even with PRIMASK set; `SLEEPDEEP` is cleared on wake so a later plain
/// executor idle does not deep-sleep.
pub(crate) unsafe fn arm_and_wait() {
    let mut scb = unsafe { cortex_m::Peripherals::steal() }.SCB;
    scb.set_sleepdeep();
    cortex_m::asm::dsb();
    cortex_m::asm::wfi();
    cortex_m::asm::isb();
    scb.clear_sleepdeep();
}
