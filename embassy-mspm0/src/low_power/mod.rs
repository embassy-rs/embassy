//! Low-power (deep-sleep) support.
//!
//! Deep-sleep depth is gated by [`WakeGuard`](crate::sysctl::WakeGuard), which drivers hold to keep
//! the chip shallower than a given [`SleepLevel`]; the low-power executor then idles into the deepest
//! mode no guard blocks.
//!
//! # Wake source caveats
//! - `GPIO_ERR_01` (L, G) — a wake edge can be missed. Only the STANDBY1 half is handled: if the pin
//!   is still asserted when the chip goes back to sleep no further edge is detected
//! - `GPIO_ERR_08` (H) — in low-power mode a GPIO can trigger a fast wake regardless of the `FASTWAKE`
//!   register and the pin configuration.
//! - `UART_ERR_01` (L, G, H) — a start bit arriving while the chip is on its way back into STANDBY1
//!   is not received.
use core::sync::atomic::Ordering;

use critical_section::CriticalSection;
use pac::cpuss::vals::Prefetch;
use pac::sysctl::vals::Dsleep;
use portable_atomic::AtomicU8;

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

pub use crate::sysctl::SleepLevel;

static SLEEP_BLOCKS: [AtomicU8; 5] = [const { AtomicU8::new(0) }; 5];

/// Block sleep at `level` and every deeper mode. Paired with [`unblock`] by
/// [`WakeGuard`](crate::sysctl::WakeGuard).
pub(crate) fn block(level: SleepLevel) {
    trace!("Blocking sleep at level {:?}", level);
    if SLEEP_BLOCKS[level as usize].fetch_add(1, Ordering::Relaxed) == (u8::MAX - 1) {
        panic!("Blocking at SleepLevel {:?} would overflow", level)
    };
}

/// Remove a block previously added at `level`.
pub(crate) fn unblock(level: SleepLevel) {
    trace!("Unblocking sleep at level {:?}", level);
    SLEEP_BLOCKS[level as usize].fetch_sub(1, Ordering::Relaxed);
}

/// Deepest mode currently permitted, or `None` if all deep sleep is blocked.
fn deepest_allowed() -> Option<SleepLevel> {
    for (i, blocks) in SLEEP_BLOCKS.iter().enumerate() {
        if blocks.load(Ordering::Relaxed) > 0 {
            return i.checked_sub(1).map(|j| SleepLevel::LEVELS[j]);
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
    trace!("Attempting to enter low-power sleep");

    // Some of the prefetcher errata applies even for a plain WFI
    // FIXME: This could be a problem for embassy-executor's default executor.
    let _prefetch = PrefetchSuspend::new();

    match deepest_allowed() {
        None => {
            trace!("Low-power sleep blocked");
            cortex_m::asm::dsb();
            cortex_m::asm::wfi();
            cortex_m::asm::isb();
        }
        Some(level) => {
            trace!("Low-power sleep allowed, mode: {:?}", level);
            enter_sleep(cs, inner::level_to_mode(level));
        }
    }
}

/// Enter SHUTDOWN, the lowest-power state. Does not return.
///
/// SHUTDOWN powers down VCORE: all SRAM is lost except the `SHUTDNSTORE` bytes, and the only wake
/// sources are a wake-capable IO event, NRST, or SWD activity. Does not return as the wake results
/// in a reset.
/// You can respond to the reset on boot using [`ResetCause::BorWakeFromShutdown`](crate::ResetCause).
//
// From the TRM: SYSCTL "Operating Modes": set `PMODECFG.DSLEEP = SHUTDOWN`, arm `SLEEPDEEP`,
// then `WFI`. This is identical across every MSPM0 family.
pub fn shutdown(_cs: CriticalSection) -> ! {
    let sysctl = pac::SYSCTL;
    sysctl.pmodecfg().modify(|w| w.set_dsleep(Dsleep::SHUTDOWN));

    let _prefetch = PrefetchSuspend::new();

    let mut scb = unsafe { cortex_m::Peripherals::steal() }.SCB;
    scb.set_sleepdeep();
    cortex_m::asm::dsb();

    cortex_m::asm::wfi();

    unsafe { core::hint::unreachable_unchecked() }
}

/// Workaround for CPU_ERR_02, CPU_ERR_03, PMCU_ERR_13 - the prefetcher has at least one errata in
/// sleep for every currently supported MCU.
struct PrefetchSuspend(pac::cpuss::regs::Ctl);

impl PrefetchSuspend {
    fn new() -> Self {
        let saved = pac::CPUSS.ctl().read();
        let mut disabled = saved;
        disabled.set_prefetch(Prefetch::DISABLE);
        pac::CPUSS.ctl().write_value(disabled);

        // CPU_ERR_02 means the prefetcher will not be disabled until pending flash access is finished.
        // Reading any SYSCTL register after disabling prefetch will complete the pending flash access.
        #[cfg(not(mspm0h321x))]
        let _ = pac::SYSCTL.shutdnstore(0).read();
        #[cfg(mspm0h321x)]
        let _ = pac::SYSCTL.clkstatus().read();

        cortex_m::asm::dsb();
        cortex_m::asm::isb();

        Self(saved)
    }
}

impl Drop for PrefetchSuspend {
    fn drop(&mut self) {
        pac::CPUSS.ctl().write_value(self.0);
    }
}

/// Arm ARM deep-sleep (`SLEEPDEEP`), wait for an interrupt, then clear it.
///
/// The mode-specific SYSCTL programming must already be done by the caller, and the prefetcher must
/// already be suspended by [`PrefetchSuspend`]. `WFI` wakes on a pending enabled interrupt even with
/// PRIMASK set; `SLEEPDEEP` is cleared on wake so a later plain executor idle does not deep-sleep.
pub(crate) unsafe fn arm_and_wait() {
    let mut scb = unsafe { cortex_m::Peripherals::steal() }.SCB;
    scb.set_sleepdeep();
    cortex_m::asm::dsb();
    cortex_m::asm::wfi();
    cortex_m::asm::isb();
    scb.clear_sleepdeep();
}
