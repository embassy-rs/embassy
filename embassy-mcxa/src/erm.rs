//! Error recording for SRAM ECC and flash uncorrectable errors.
//!
//! Constructors reset the peripheral and discard records from any previous epoch.
//! Status and reports are nonconsuming; acknowledge flags explicitly. The hardware
//! retains flags and the latest SRAM attributes, not a lossless event queue. A
//! report samples multiple registers and is not an atomic snapshot.

use core::marker::PhantomData;

use embassy_hal_internal::Peri;
use embassy_hal_internal::drop::OnDrop;
use maitake_sync::WaitCell;

use crate::clocks::periph_helpers::NoConfig;
use crate::clocks::{ClockError, Gate, WakeGuard, enable_and_reset, enable_ecc_ram_a_clock};
use crate::interrupt::typelevel::{Binding, Interrupt};
#[cfg(feature = "_mcxa5xx")]
use crate::pac::erm::Erm as Regs;
#[cfg(not(feature = "_mcxa5xx"))]
use crate::pac::erm0::Erm0 as Regs;
use crate::peripherals::ERM0;
use crate::{interrupt, pac};

const SRAM_BASE: u32 = 0x2000_0000;

/// A nonempty selection of supported error events.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Events(u8);

impl Events {
    /// Corrected single-bit SRAM error.
    pub const SRAM_CORRECTED: Self = Self(1);
    /// Uncorrectable SRAM error.
    pub const SRAM_UNCORRECTABLE: Self = Self(2);
    /// Uncorrectable flash error.
    pub const FLASH_UNCORRECTABLE: Self = Self(4);
    /// All supported events.
    pub const ALL: Self = Self(7);

    /// Selects every event in either selection.
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    const fn intersects(self, other: Self) -> bool {
        self.0 & other.0 != 0
    }
}

/// Latched SRAM error flags.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SramEvents {
    /// Corrected single-bit error.
    Corrected,
    /// Uncorrectable error.
    Uncorrectable,
    /// Both SRAM flags are set.
    Both,
}

impl SramEvents {
    const fn events(self) -> Events {
        match self {
            Self::Corrected => Events::SRAM_CORRECTED,
            Self::Uncorrectable => Events::SRAM_UNCORRECTABLE,
            Self::Both => Events::SRAM_CORRECTED.union(Events::SRAM_UNCORRECTABLE),
        }
    }
}

/// Currently latched supported error flags.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Status {
    /// Latched SRAM flags, if any.
    pub sram: Option<SramEvents>,
    /// Whether an uncorrectable flash error is latched.
    pub flash_uncorrectable: bool,
}

impl Status {
    /// Returns the latched events, or `None` when no supported flag is set.
    pub const fn events(self) -> Option<Events> {
        let sram = match self.sram {
            Some(events) => events.events().0,
            None => 0,
        };
        let bits = sram
            | if self.flash_uncorrectable {
                Events::FLASH_UNCORRECTABLE.0
            } else {
                0
            };
        if bits == 0 { None } else { Some(Events(bits)) }
    }

    /// Whether any latched event matches the selection.
    pub const fn intersects(self, events: Events) -> bool {
        match self.events() {
            Some(pending) => pending.intersects(events),
            None => false,
        }
    }
}

/// A nonconsuming report of latched error evidence.
///
/// SRAM attributes describe the latest recorded SRAM error, not necessarily every
/// flag in `events`. Concurrent errors can replace attributes while they are read.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Report {
    /// SRAM error flags and the latest recorded SRAM attributes.
    Sram {
        /// All SRAM flags sampled, including flags outside the requested filter.
        events: SramEvents,
        /// Recorded SRAM error address in the canonical `0x2000_0000` alias.
        ///
        /// EAR0 contains a byte offset on MCXA; this adds the SRAM base as in
        /// NXP's [board configuration] and [error-recording example], despite the
        /// system-address wording in A2 RM 9.5.4 and A5 RM 16.5.4. This does not
        /// identify the original access's security alias.
        ///
        /// [board configuration]: https://github.com/nxp-mcuxpresso/mcuxsdk-examples/blob/3151bc057b8924db54c7395d39b213c05ab94dc3/_boards/frdmmcxa287/driver_examples/erm/error_recording/app.h#L28-L30
        /// [error-recording example]: https://github.com/nxp-mcuxpresso/mcuxsdk-examples/blob/3151bc057b8924db54c7395d39b213c05ab94dc3/driver_examples/erm/error_recording/error_recording.c#L102-L107
        address: u32,
        /// Recorded ECC syndrome.
        syndrome: u8,
    },
    /// Uncorrectable flash error; no flash address or syndrome is available.
    FlashUncorrectable,
}

impl Report {
    /// Events represented by this report.
    pub const fn events(&self) -> Events {
        match self {
            Self::Sram { events, .. } => events.events(),
            Self::FlashUncorrectable => Events::FLASH_UNCORRECTABLE,
        }
    }
}

mod sealed {
    pub trait Sealed {}
}

/// Driver mode.
#[allow(private_bounds)]
pub trait Mode: sealed::Sealed {}

/// Polling mode, without bound interrupts.
pub struct Blocking;
impl sealed::Sealed for Blocking {}
impl Mode for Blocking {}

/// Interrupt-driven asynchronous mode.
pub struct Async;
impl sealed::Sealed for Async {}
impl Mode for Async {}

/// Errors during peripheral initialization.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum SetupError {
    /// Clock configuration failed.
    ClockSetup(ClockError),
}

/// Errors while waiting for an error report.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum WaitError {
    /// The internal waiter was closed.
    WaitClosed,
}

struct Info {
    regs: Regs,
    wait_cell: WaitCell,
}

static INFO: Info = Info {
    regs: pac::ERM0,
    wait_cell: WaitCell::new(),
};

impl Info {
    fn mask_interrupts(&self) {
        self.regs.cr0().modify(|w| {
            w.set_escie0(false);
            w.set_encie0(false);
            w.set_encie1(false);
        });
    }

    fn arm_interrupts(&self, events: Events) {
        self.regs.cr0().modify(|w| {
            w.set_escie0(events.intersects(Events::SRAM_CORRECTED));
            w.set_encie0(events.intersects(Events::SRAM_UNCORRECTABLE));
            w.set_encie1(events.intersects(Events::FLASH_UNCORRECTABLE));
        });
    }

    fn on_interrupt(&self) {
        // A higher-priority executor must not preempt wake while WaitCell is WAKING.
        critical_section::with(|_| {
            // A stale NVIC request must not access a gated peripheral.
            if !ERM0::is_clock_enabled() {
                return;
            }
            self.mask_interrupts();
            self.wait_cell.wake();
        });
    }
}

/// Handler for corrected SRAM error notifications.
pub struct SingleBitInterruptHandler;

impl interrupt::typelevel::Handler<interrupt::typelevel::ERM0_SINGLE_BIT> for SingleBitInterruptHandler {
    unsafe fn on_interrupt() {
        INFO.on_interrupt();
    }
}

/// Handler for uncorrectable SRAM and flash error notifications.
pub struct MultiBitInterruptHandler;

impl interrupt::typelevel::Handler<interrupt::typelevel::ERM0_MULTI_BIT> for MultiBitInterruptHandler {
    unsafe fn on_interrupt() {
        INFO.on_interrupt();
    }
}

/// Error recording module driver.
///
/// Keeps the SRAM channel clock available and prevents HAL deep sleep while live.
/// Dropping the driver masks notifications without acknowledging error records or
/// disabling functional clocks.
pub struct Erm<'d, M: Mode> {
    _peri: Peri<'d, ERM0>,
    info: &'static Info,
    _wg: WakeGuard,
    _mode: PhantomData<M>,
}

impl<'d> Erm<'d, Blocking> {
    /// Creates a polling driver, discarding all previous error records and count.
    pub fn new_blocking(peri: Peri<'d, ERM0>) -> Result<Self, SetupError> {
        Self::new_inner(peri)
    }
}

impl<'d> Erm<'d, Async> {
    /// Creates an async driver, discarding all previous error records and count.
    pub fn new_async(
        peri: Peri<'d, ERM0>,
        _irq: impl Binding<interrupt::typelevel::ERM0_SINGLE_BIT, SingleBitInterruptHandler>
        + Binding<interrupt::typelevel::ERM0_MULTI_BIT, MultiBitInterruptHandler>
        + 'd,
    ) -> Result<Self, SetupError> {
        let driver = Self::new_inner(peri)?;
        interrupt::typelevel::ERM0_SINGLE_BIT::unpend();
        interrupt::typelevel::ERM0_MULTI_BIT::unpend();
        // SAFETY: both handlers are bound and peripheral initialization succeeded.
        unsafe {
            interrupt::typelevel::ERM0_SINGLE_BIT::enable();
            interrupt::typelevel::ERM0_MULTI_BIT::enable();
        }
        Ok(driver)
    }

    /// Waits for current latched evidence matching `events`, without consuming it.
    ///
    /// An error latched before the first poll is a valid completion. A wake alone
    /// is not. Completion, cancellation, and errors mask all owned notification
    /// sources and preserve flags, SRAM attributes, and the corrected count.
    pub async fn wait_report(&mut self, events: Events) -> Result<Report, WaitError> {
        let on_drop = OnDrop::new(|| self.info.mask_interrupts());
        let mut report = None;
        self.info
            .wait_cell
            .wait_for(|| {
                self.info.arm_interrupts(events);
                report = self.snapshot(events);
                report.is_some()
            })
            .await
            .map_err(|_| WaitError::WaitClosed)?;
        self.info.mask_interrupts();
        on_drop.defuse();
        // The predicate returns true only after storing a matching snapshot.
        Ok(report.unwrap())
    }
}

impl<'d, M: Mode> Erm<'d, M> {
    fn new_inner(peri: Peri<'d, ERM0>) -> Result<Self, SetupError> {
        interrupt::typelevel::ERM0_SINGLE_BIT::disable();
        interrupt::typelevel::ERM0_MULTI_BIT::disable();
        let wg = enable_ecc_ram_a_clock().map_err(SetupError::ClockSetup)?;
        // SAFETY: the token proves exclusive ownership, and both IRQs are disabled.
        let _parts = unsafe { enable_and_reset::<ERM0>(&NoConfig).map_err(SetupError::ClockSetup)? };
        let mut driver = Self {
            _peri: peri,
            info: &INFO,
            _wg: wg,
            _mode: PhantomData,
        };
        driver.info.mask_interrupts();
        driver.acknowledge(Events::ALL);
        driver.reset_corrected_count();
        Ok(driver)
    }

    /// Reads the currently latched supported flags without clearing them.
    pub fn status(&self) -> Status {
        let status = self.info.regs.sr0().read();
        let sram = match (status.sbc0(), status.nce0()) {
            (false, false) => None,
            (true, false) => Some(SramEvents::Corrected),
            (false, true) => Some(SramEvents::Uncorrectable),
            (true, true) => Some(SramEvents::Both),
        };
        Status {
            sram,
            flash_uncorrectable: status.nce1(),
        }
    }

    /// Samples matching latched evidence without clearing it.
    ///
    /// Flash takes precedence when both channels match. SRAM reports include all
    /// sampled SRAM flags, even when only one matched `events`. Register reads are
    /// not atomic; newly arriving errors may replace the latest SRAM attributes.
    pub fn snapshot(&self, events: Events) -> Option<Report> {
        let status = self.status();
        if status.flash_uncorrectable && events.intersects(Events::FLASH_UNCORRECTABLE) {
            return Some(Report::FlashUncorrectable);
        }
        let sram = status.sram?;
        if !sram.events().intersects(events) {
            return None;
        }
        Some(Report::Sram {
            events: sram,
            address: SRAM_BASE.wrapping_add(self.info.regs.ear0().read().ear()),
            syndrome: self.info.regs.syn0().read().syndrome(),
        })
    }

    /// Clears only the selected flags using write-one-to-clear semantics.
    ///
    /// Does not reset the corrected count. Errors arriving during acknowledgement
    /// are subject to the hardware's flag-latching behavior; this is not a queue.
    pub fn acknowledge(&mut self, events: Events) {
        self.info.regs.sr0().write(|w| {
            w.set_sbc0(events.intersects(Events::SRAM_CORRECTED));
            w.set_nce0(events.intersects(Events::SRAM_UNCORRECTABLE));
            w.set_nce1(events.intersects(Events::FLASH_UNCORRECTABLE));
        });
    }

    /// Reads the SRAM corrected-error count, which saturates at 255.
    pub fn corrected_count(&self) -> u8 {
        self.info.regs.corr_err_cnt0().read().count()
    }

    /// Resets the SRAM corrected-error count without acknowledging flags.
    pub fn reset_corrected_count(&mut self) {
        self.info.regs.corr_err_cnt0().write(|w| w.set_count(0));
    }
}

impl<M: Mode> Drop for Erm<'_, M> {
    fn drop(&mut self) {
        self.info.mask_interrupts();
        interrupt::typelevel::ERM0_SINGLE_BIT::disable();
        interrupt::typelevel::ERM0_MULTI_BIT::disable();
    }
}
