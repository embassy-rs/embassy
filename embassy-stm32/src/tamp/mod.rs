//! Tamper detection (TAMP)
//!
//! The TAMP peripheral monitors external tamper pins and a set of chip-internal
//! tamper conditions, latching a sticky status flag and (optionally) an
//! interrupt when one fires. It also owns a set of backup registers and a
//! monotonic counter that live in the backup domain.
//!
//! TAMP has no independent clock gate: it is always clocked whenever the
//! backup domain is powered, and the backup domain write protection is
//! unlocked unconditionally during [`crate::init()`]. So, unlike most other
//! peripherals, [`Tamp::new()`] does not call `rcc::enable_and_reset`.

use core::future::poll_fn;
use core::sync::atomic::{Ordering, compiler_fence};
use core::task::Poll;

use embassy_hal_internal::Peri;
use embassy_sync::waitqueue::AtomicWaker;

use crate::gpio::{self, Input, Pull};
use crate::interrupt::InterruptExt;
/// Tamper input debounce filter, shared by all external tamper channels (`FLTCR.TAMPFLT`).
pub use crate::pac::tamp::vals::Tampflt as Filter;
use crate::pac::tamp::vals::Tamptrg;
use crate::peripherals::TAMP;
use crate::{interrupt, pac};

#[cfg(tamp_wba)]
const EXTERNAL_CHANNELS: u8 = 6;
#[cfg(tamp_u5)]
const EXTERNAL_CHANNELS: u8 = 8;
const INTERNAL_TAMPERS: u8 = 13;
const BACKUP_REGISTER_COUNT: usize = 32;

static WAKER: AtomicWaker = AtomicWaker::new();

fn regs() -> pac::tamp::Tamp {
    pac::TAMP
}

fn read_status() -> TamperStatus {
    let sr = regs().sr().read();

    let mut external = 0u8;
    for n in 0..EXTERNAL_CHANNELS {
        if sr.tampf(n as usize) {
            external |= 1 << n;
        }
    }

    let mut internal = 0u16;
    for n in 0..INTERNAL_TAMPERS {
        if sr.itampf(n as usize) {
            internal |= 1 << n;
        }
    }

    TamperStatus { external, internal }
}

/// Active level that triggers a tamper detection event on an external channel.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Trigger {
    /// The pin being sampled (or transitioning to) a low level triggers detection.
    ActiveLow,
    /// The pin being sampled (or transitioning to) a high level triggers detection.
    ActiveHigh,
}

impl Trigger {
    fn to_pac(self) -> Tamptrg {
        match self {
            Trigger::ActiveLow => Tamptrg::FilteredLowOrUnfilteredHigh,
            Trigger::ActiveHigh => Tamptrg::FilteredHighOrUnfilteredLow,
        }
    }
}

/// Configuration for an external tamper channel.
#[derive(Clone, Copy, Debug)]
pub struct ExternalTamperConfig {
    /// Active level that triggers detection on this channel.
    pub trigger: Trigger,
    /// Debounce filter. This setting is shared by all external channels (`FLTCR`
    /// is a single, chip-wide register), so the last call to
    /// [`Tamp::configure_external_channel()`] wins for every channel.
    pub filter: Filter,
    /// GPIO pull configuration for the tamper pin.
    pub pull: Pull,
}

impl Default for ExternalTamperConfig {
    fn default() -> Self {
        Self {
            trigger: Trigger::ActiveHigh,
            filter: Filter::NoFilter,
            pull: Pull::None,
        }
    }
}

/// A GPIO pin bound to an external tamper channel.
///
/// Dropping this disables the channel (and its interrupt), releasing the pin.
pub struct ExternalTamperPin<'d> {
    _pin: Input<'d>,
    channel: u8,
}

impl<'d> Drop for ExternalTamperPin<'d> {
    fn drop(&mut self) {
        let r = regs();
        let n = self.channel as usize;
        r.ier().modify(|w| w.set_tampie(n, false));
        r.cr1().modify(|w| w.set_tampe(n, false));
    }
}

/// One of the chip's internal tamper detection conditions (ITAMPx).
///
/// Which physical condition each index corresponds to is chip-specific;
/// consult the reference manual for your part.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct InternalTamper(pub u8);

/// Snapshot of which tamper channels/conditions have a latched detection flag.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct TamperStatus {
    external: u8,
    internal: u16,
}

impl TamperStatus {
    /// Whether external channel `channel` has a latched detection flag.
    pub fn is_external(&self, channel: u8) -> bool {
        self.external & (1 << channel) != 0
    }

    /// Whether internal tamper `tamper` has a latched detection flag.
    pub fn is_internal(&self, tamper: u8) -> bool {
        self.internal & (1 << tamper) != 0
    }

    /// Whether any channel or internal tamper has a latched detection flag.
    pub fn any(&self) -> bool {
        self.external != 0 || self.internal != 0
    }
}

/// Tamper detection driver.
pub struct Tamp<'d> {
    _peri: Peri<'d, TAMP>,
}

impl<'d> Tamp<'d> {
    /// Create a new tamper detection driver.
    pub fn new(
        _peri: Peri<'d, TAMP>,
        _irq: impl interrupt::typelevel::Binding<interrupt::typelevel::TAMP, InterruptHandler> + 'd,
    ) -> Self {
        interrupt::TAMP.unpend();
        unsafe { interrupt::TAMP.enable() };

        Self { _peri }
    }

    /// Enable and configure an external tamper channel, binding it to `pin`.
    ///
    /// `channel` is the 0-based tamper channel index (TAMP1 = 0, TAMP2 = 1, ...).
    /// It must match the silicon pin mapping documented in your chip's reference
    /// manual/datasheet; this is not checked against the pin passed in.
    pub fn configure_external_channel(
        &mut self,
        channel: u8,
        pin: Peri<'d, impl gpio::Pin>,
        config: ExternalTamperConfig,
    ) -> ExternalTamperPin<'d> {
        let pin = Input::new(pin, config.pull);
        let n = channel as usize;
        let r = regs();

        r.fltcr().modify(|w| w.set_tampflt(config.filter));
        r.cr2().modify(|w| w.set_tamptrg(n, config.trigger.to_pac()));
        r.cr1().modify(|w| w.set_tampe(n, true));
        r.ier().modify(|w| w.set_tampie(n, true));

        ExternalTamperPin { _pin: pin, channel }
    }

    /// Enable an internal tamper condition.
    pub fn enable_internal_tamper(&mut self, tamper: InternalTamper) {
        let n = tamper.0 as usize;
        let r = regs();
        r.cr1().modify(|w| w.set_itampe(n, true));
        r.ier().modify(|w| w.set_itampie(n, true));
    }

    /// Disable an internal tamper condition.
    pub fn disable_internal_tamper(&mut self, tamper: InternalTamper) {
        let n = tamper.0 as usize;
        let r = regs();
        r.ier().modify(|w| w.set_itampie(n, false));
        r.cr1().modify(|w| w.set_itampe(n, false));
    }

    /// Wait for a tamper detection event.
    ///
    /// The status returned reflects the raw, latched `SR` flags at the time of
    /// return. Once handled, call [`Self::clear_tamper_flags()`] with the
    /// returned status to clear it and resume monitoring those channels —
    /// until then, their interrupt is left disabled (they won't wake this
    /// future again, but the flag itself stays latched).
    pub async fn wait_for_tamper(&mut self) -> TamperStatus {
        poll_fn(|cx| {
            WAKER.register(cx.waker());
            let status = read_status();
            if status.any() {
                Poll::Ready(status)
            } else {
                Poll::Pending
            }
        })
        .await
    }

    /// Read the current, raw tamper detection status (`SR`).
    pub fn tamper_status(&self) -> TamperStatus {
        read_status()
    }

    /// Clear the given tamper flags in `SCR`, and re-enable the interrupt
    /// (`IER`) for any of them that are still enabled in `CR1`.
    pub fn clear_tamper_flags(&mut self, status: TamperStatus) {
        let r = regs();

        r.scr().write(|w| {
            for n in 0..EXTERNAL_CHANNELS {
                if status.is_external(n) {
                    w.set_ctampf(n as usize, true);
                }
            }
            for n in 0..INTERNAL_TAMPERS {
                if status.is_internal(n) {
                    w.set_citampf(n as usize, true);
                }
            }
        });

        let cr1 = r.cr1().read();
        r.ier().modify(|w| {
            for n in 0..EXTERNAL_CHANNELS {
                if status.is_external(n) && cr1.tampe(n as usize) {
                    w.set_tampie(n as usize, true);
                }
            }
            for n in 0..INTERNAL_TAMPERS {
                if status.is_internal(n) && cr1.itampe(n as usize) {
                    w.set_itampie(n as usize, true);
                }
            }
        });
    }

    /// Read a backup register (`BKPRn`). Returns `None` if `register` is out of range.
    pub fn read_backup_register(&self, register: usize) -> Option<u32> {
        if register < BACKUP_REGISTER_COUNT {
            Some(regs().bkpr(register).read().bkp())
        } else {
            None
        }
    }

    /// Write a backup register (`BKPRn`). Silently does nothing if `register` is out of range.
    pub fn write_backup_register(&mut self, register: usize, value: u32) {
        if register < BACKUP_REGISTER_COUNT {
            regs().bkpr(register).write(|w| w.set_bkp(value));
        }
    }

    /// Read the monotonic tamper counter (`COUNTR`). It increments by one on
    /// every write to the register performed internally on a tamper event.
    pub fn monotonic_counter(&self) -> u32 {
        regs().countr().read().count()
    }
}

/// Interrupt handler.
pub struct InterruptHandler {
    _private: (),
}

impl interrupt::typelevel::Handler<interrupt::typelevel::TAMP> for InterruptHandler {
    unsafe fn on_interrupt() {
        let r = pac::TAMP;
        let (ier, misr) = (r.ier().read(), r.misr().read());

        let mut fired = false;
        r.ier().modify(|w| {
            for n in 0..EXTERNAL_CHANNELS as usize {
                if ier.tampie(n) && misr.tampmf(n) {
                    w.set_tampie(n, false);
                    fired = true;
                }
            }
            for n in 0..INTERNAL_TAMPERS as usize {
                if ier.itampie(n) && misr.itampmf(n) {
                    w.set_itampie(n, false);
                    fired = true;
                }
            }
        });

        if !fired {
            return;
        }

        compiler_fence(Ordering::SeqCst);
        WAKER.wake();
    }
}
