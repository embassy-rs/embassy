//! Tamper detection (TAMP)
//!
//! The TAMP peripheral monitors external tamper pins and a set of chip-internal
//! tamper conditions, latching a sticky status flag and (optionally) an
//! interrupt when one fires. It also owns a set of backup registers and (on
//! most supported chips) a monotonic counter that live in the backup domain.
//!
//! TAMP has no independent clock gate: it is always clocked whenever the
//! backup domain is powered, and the backup domain write protection is
//! unlocked unconditionally during [`crate::init()`]. So, unlike most other
//! peripherals, [`Tamp::new()`] does not call `rcc::enable_and_reset`.
//!
//! Supported on STM32G0, G4, H5, L5, U5, WBA, WL and N6. The underlying register
//! layout differs meaningfully across these (channel/tamper counts, whether
//! there's a monotonic counter, and — on H5 — internal tampers are exposed as
//! individually named bit fields rather than an indexed array); those
//! differences are handled internally and don't show up in the public API.

use core::future::poll_fn;
use core::sync::atomic::{Ordering, compiler_fence};
use core::task::Poll;

use embassy_hal_internal::Peri;
use embassy_sync::waitqueue::AtomicWaker;

use crate::gpio::{self, Input, Pull};
use crate::interrupt::typelevel::Interrupt as _;
use crate::pac::tamp::regs::{Cr1, Fltcr, Ier, Misr, Scr, Sr};
use crate::peripherals::TAMP;
use crate::{interrupt, pac};

#[cfg(tamp_h5)]
mod h5;
#[cfg(tamp_n6)]
mod n6;

/// Generates the seven index-mapping functions used by chips that expose
/// internal tampers as individually named bit fields (`itamp1e()`, `itamp2e()`,
/// ...) instead of an indexed array accessor.
///
/// Indices not listed in the table fall through to the `_` arm: reads return
/// `false`/no-op and writes are a no-op, per the `InternalTamper` contract
/// ("indices that aren't physically present are simply inert").
macro_rules! itamp_fields {
    ($( $idx:literal => ($e:ident, $set_e:ident, $ie:ident, $set_ie:ident, $f:ident, $mf:ident, $set_cf:ident) ),* $(,)?) => {
        pub fn cr1_itampe(r: crate::pac::tamp::regs::Cr1, n: usize) -> bool {
            match n {
                $( $idx => r.$e(), )*
                _ => false,
            }
        }

        pub fn cr1_set_itampe(w: &mut crate::pac::tamp::regs::Cr1, n: usize, val: bool) {
            match n {
                $( $idx => w.$set_e(val), )*
                _ => {}
            }
        }

        pub fn ier_itampie(r: crate::pac::tamp::regs::Ier, n: usize) -> bool {
            match n {
                $( $idx => r.$ie(), )*
                _ => false,
            }
        }

        pub fn ier_set_itampie(w: &mut crate::pac::tamp::regs::Ier, n: usize, val: bool) {
            match n {
                $( $idx => w.$set_ie(val), )*
                _ => {}
            }
        }

        pub fn sr_itampf(r: crate::pac::tamp::regs::Sr, n: usize) -> bool {
            match n {
                $( $idx => r.$f(), )*
                _ => false,
            }
        }

        pub fn misr_itampmf(r: crate::pac::tamp::regs::Misr, n: usize) -> bool {
            match n {
                $( $idx => r.$mf(), )*
                _ => false,
            }
        }

        pub fn scr_set_citampf(w: &mut crate::pac::tamp::regs::Scr, n: usize, val: bool) {
            match n {
                $( $idx => w.$set_cf(val), )*
                _ => {}
            }
        }
    };
}
pub(crate) use itamp_fields;

// On several chips TAMP doesn't have its own NVIC line — its interrupt is
// combined with RTC's (alarm/wakeup/LSE-CSS) on a shared vector. `bind_interrupts!`
// supports binding more than one handler to the same interrupt name, so users
// combining TAMP with the RTC low-power wakeup feature on these chips just
// need to bind both handlers to that shared name.
#[cfg(tamp_g0)]
type TampInterrupt = interrupt::typelevel::RTC_TAMP;
#[cfg(tamp_g4)]
type TampInterrupt = interrupt::typelevel::RTC_TAMP_LSECSS;
#[cfg(all(tamp_wl, not(feature = "_core-cm0p")))]
type TampInterrupt = interrupt::typelevel::TAMP_STAMP_LSECSS_SSRU;
#[cfg(all(tamp_wl, feature = "_core-cm0p"))]
type TampInterrupt = interrupt::typelevel::RTC_LSECSS;
#[cfg(any(tamp_h5, tamp_l5, tamp_u5, tamp_wba, tamp_n6))]
type TampInterrupt = interrupt::typelevel::TAMP;

#[cfg(tamp_wba)]
const EXTERNAL_CHANNELS: u8 = 6;
#[cfg(tamp_u5)]
const EXTERNAL_CHANNELS: u8 = 8;
#[cfg(tamp_g0)]
const EXTERNAL_CHANNELS: u8 = 2;
#[cfg(tamp_g4)]
const EXTERNAL_CHANNELS: u8 = 3;
#[cfg(tamp_h5)]
const EXTERNAL_CHANNELS: u8 = 8;
#[cfg(tamp_n6)]
const EXTERNAL_CHANNELS: u8 = 7;
#[cfg(tamp_l5)]
const EXTERNAL_CHANNELS: u8 = 8;
#[cfg(tamp_wl)]
const EXTERNAL_CHANNELS: u8 = 3;

#[cfg(any(tamp_u5, tamp_wba))]
const INTERNAL_TAMPERS: u8 = 13;
#[cfg(any(tamp_g0, tamp_g4))]
const INTERNAL_TAMPERS: u8 = 6;
#[cfg(any(tamp_l5, tamp_wl))]
const INTERNAL_TAMPERS: u8 = 8;
#[cfg(tamp_h5)]
const INTERNAL_TAMPERS: u8 = h5::INTERNAL_TAMPERS;
#[cfg(tamp_n6)]
const INTERNAL_TAMPERS: u8 = n6::INTERNAL_TAMPERS;

#[cfg(tamp_g0)]
const BACKUP_REGISTER_COUNT: usize = 5;
#[cfg(tamp_wl)]
const BACKUP_REGISTER_COUNT: usize = 20;
#[cfg(any(tamp_u5, tamp_wba, tamp_g4, tamp_h5, tamp_l5, tamp_n6))]
const BACKUP_REGISTER_COUNT: usize = 32;

static WAKER: AtomicWaker = AtomicWaker::new();

fn regs() -> pac::tamp::Tamp {
    pac::TAMP
}

fn cr1_itampe(r: Cr1, n: usize) -> bool {
    #[cfg(tamp_h5)]
    {
        h5::cr1_itampe(r, n)
    }
    #[cfg(tamp_n6)]
    {
        n6::cr1_itampe(r, n)
    }
    #[cfg(not(any(tamp_h5, tamp_n6)))]
    {
        r.itampe(n)
    }
}

fn cr1_set_itampe(w: &mut Cr1, n: usize, val: bool) {
    #[cfg(tamp_h5)]
    {
        h5::cr1_set_itampe(w, n, val);
    }
    #[cfg(tamp_n6)]
    {
        n6::cr1_set_itampe(w, n, val);
    }
    #[cfg(not(any(tamp_h5, tamp_n6)))]
    {
        w.set_itampe(n, val);
    }
}

fn ier_itampie(r: Ier, n: usize) -> bool {
    #[cfg(tamp_h5)]
    {
        h5::ier_itampie(r, n)
    }
    #[cfg(tamp_n6)]
    {
        n6::ier_itampie(r, n)
    }
    #[cfg(not(any(tamp_h5, tamp_n6)))]
    {
        r.itampie(n)
    }
}

fn ier_set_itampie(w: &mut Ier, n: usize, val: bool) {
    #[cfg(tamp_h5)]
    {
        h5::ier_set_itampie(w, n, val);
    }
    #[cfg(tamp_n6)]
    {
        n6::ier_set_itampie(w, n, val);
    }
    #[cfg(not(any(tamp_h5, tamp_n6)))]
    {
        w.set_itampie(n, val);
    }
}

fn sr_itampf(r: Sr, n: usize) -> bool {
    #[cfg(tamp_h5)]
    {
        h5::sr_itampf(r, n)
    }
    #[cfg(tamp_n6)]
    {
        n6::sr_itampf(r, n)
    }
    #[cfg(not(any(tamp_h5, tamp_n6)))]
    {
        r.itampf(n)
    }
}

fn misr_itampmf(r: Misr, n: usize) -> bool {
    #[cfg(tamp_h5)]
    {
        h5::misr_itampmf(r, n)
    }
    #[cfg(tamp_n6)]
    {
        n6::misr_itampmf(r, n)
    }
    #[cfg(not(any(tamp_h5, tamp_n6)))]
    {
        r.itampmf(n)
    }
}

fn scr_set_citampf(w: &mut Scr, n: usize, val: bool) {
    #[cfg(tamp_h5)]
    {
        h5::scr_set_citampf(w, n, val);
    }
    #[cfg(tamp_n6)]
    {
        n6::scr_set_citampf(w, n, val);
    }
    #[cfg(not(any(tamp_h5, tamp_n6)))]
    {
        w.set_citampf(n, val);
    }
}

fn set_tampflt(w: &mut Fltcr, filter: Filter) {
    #[cfg(any(tamp_u5, tamp_wba, tamp_wl))]
    {
        w.set_tampflt(pac::tamp::vals::Tampflt::from_bits(filter as u8));
    }
    #[cfg(any(tamp_g0, tamp_g4, tamp_h5, tamp_l5, tamp_n6))]
    {
        w.set_tampflt(filter as u8);
    }
}

fn set_tamptrg(w: &mut pac::tamp::regs::Cr2, n: usize, trigger: Trigger) {
    #[cfg(any(tamp_u5, tamp_wba, tamp_wl))]
    {
        w.set_tamptrg(n, pac::tamp::vals::Tamptrg::from_bits(trigger as u8));
    }
    #[cfg(any(tamp_g0, tamp_g4, tamp_h5, tamp_l5, tamp_n6))]
    {
        w.set_tamptrg(n, trigger as u8 != 0);
    }
}

fn read_bkpr(register: usize) -> u32 {
    #[cfg(any(tamp_h5, tamp_n6))]
    {
        regs().bkpr(register).read()
    }
    #[cfg(not(any(tamp_h5, tamp_n6)))]
    {
        regs().bkpr(register).read().bkp()
    }
}

fn write_bkpr(register: usize, value: u32) {
    #[cfg(any(tamp_h5, tamp_n6))]
    {
        regs().bkpr(register).write_value(value);
    }
    #[cfg(not(any(tamp_h5, tamp_n6)))]
    {
        regs().bkpr(register).write(|w| w.set_bkp(value));
    }
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
        if sr_itampf(sr, n as usize) {
            internal |= 1 << n;
        }
    }

    TamperStatus { external, internal }
}

/// Tamper input debounce filter, shared by all external tamper channels (`FLTCR.TAMPFLT`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum Filter {
    /// Tamper event is activated on the edge of the input transitioning to the active level (no internal pull-up).
    NoFilter = 0,
    /// Tamper event is activated after 2 consecutive samples at the active level.
    Filter2 = 1,
    /// Tamper event is activated after 4 consecutive samples at the active level.
    Filter4 = 2,
    /// Tamper event is activated after 8 consecutive samples at the active level.
    Filter8 = 3,
}

/// Active level that triggers a tamper detection event on an external channel.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum Trigger {
    /// The pin being sampled (or transitioning to) a low level triggers detection.
    ActiveLow = 0,
    /// The pin being sampled (or transitioning to) a high level triggers detection.
    ActiveHigh = 1,
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
/// consult the reference manual for your part. Note that internal tamper
/// numbering has gaps on several chips (e.g. G4 only wires ITAMP3..ITAMP6,
/// occupying indices 2..5) — indices that aren't physically present are
/// simply inert (enabling one is a no-op, and it'll never appear in a
/// [`TamperStatus`]).
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
        _irq: impl interrupt::typelevel::Binding<TampInterrupt, InterruptHandler> + 'd,
    ) -> Self {
        TampInterrupt::unpend();
        unsafe { TampInterrupt::enable() };

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

        r.fltcr().modify(|w| set_tampflt(w, config.filter));
        r.cr2().modify(|w| set_tamptrg(w, n, config.trigger));
        r.cr1().modify(|w| w.set_tampe(n, true));
        r.ier().modify(|w| w.set_tampie(n, true));

        ExternalTamperPin { _pin: pin, channel }
    }

    /// Enable an internal tamper condition.
    pub fn enable_internal_tamper(&mut self, tamper: InternalTamper) {
        let n = tamper.0 as usize;
        let r = regs();
        r.cr1().modify(|w| cr1_set_itampe(w, n, true));
        r.ier().modify(|w| ier_set_itampie(w, n, true));
    }

    /// Disable an internal tamper condition.
    pub fn disable_internal_tamper(&mut self, tamper: InternalTamper) {
        let n = tamper.0 as usize;
        let r = regs();
        r.ier().modify(|w| ier_set_itampie(w, n, false));
        r.cr1().modify(|w| cr1_set_itampe(w, n, false));
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
                    scr_set_citampf(w, n as usize, true);
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
                if status.is_internal(n) && cr1_itampe(cr1, n as usize) {
                    ier_set_itampie(w, n as usize, true);
                }
            }
        });
    }

    /// Read a backup register (`BKPRn`). Returns `None` if `register` is out of range.
    pub fn read_backup_register(&self, register: usize) -> Option<u32> {
        if register < BACKUP_REGISTER_COUNT {
            Some(read_bkpr(register))
        } else {
            None
        }
    }

    /// Write a backup register (`BKPRn`). Silently does nothing if `register` is out of range.
    pub fn write_backup_register(&mut self, register: usize, value: u32) {
        if register < BACKUP_REGISTER_COUNT {
            write_bkpr(register, value);
        }
    }

    /// Read the monotonic tamper counter. It increments by one on every
    /// access performed internally on a read, so consecutive calls return
    /// increasing values.
    ///
    /// Not available on G0/G4, which don't have this register — this method
    /// doesn't exist on those chips.
    #[cfg(any(tamp_u5, tamp_wba, tamp_h5, tamp_l5, tamp_wl, tamp_n6))]
    pub fn monotonic_counter(&self) -> u32 {
        #[cfg(any(tamp_h5, tamp_n6))]
        {
            regs().count1r().read()
        }
        #[cfg(not(any(tamp_h5, tamp_n6)))]
        {
            regs().countr().read().count()
        }
    }
}

/// Interrupt handler.
pub struct InterruptHandler {
    _private: (),
}

impl interrupt::typelevel::Handler<TampInterrupt> for InterruptHandler {
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
                if ier_itampie(ier, n) && misr_itampmf(misr, n) {
                    ier_set_itampie(w, n, false);
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
