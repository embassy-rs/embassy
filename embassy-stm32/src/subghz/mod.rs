//! Sub-GHz Radio (SUBGHZ) peripheral driver for STM32WL / STM32WLE5 series.
//!
//! The STM32WLEx family integrates a Semtech SX1262-class sub-GHz radio transceiver
//! hardwired to the internal `SUBGHZSPI` interface, power management (`PWR`), and reset (`RCC`).
//!
//! This module provides:
//! - [`SubghzSpi`]: `SpiDevice` driver managing `SUBGHZSPI` and hardware slave-select line (`PWR_SUBGHZSPICR.NSS`).
//! - [`SubghzControl`]: Control of radio reset (`RCC_CSR.RFRST`), BUSY status polling / awaiting, and event interrupts (`SUBGHZ_RADIO`).

pub mod spi;

use core::future::poll_fn;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicBool, Ordering};
use core::task::Poll;

use embassy_sync::waitqueue::AtomicWaker;
pub use spi::SubghzSpi;

use crate::interrupt::typelevel::Interrupt;
use crate::pac;

static RADIO_WAKER: AtomicWaker = AtomicWaker::new();
static RADIO_PENDING: AtomicBool = AtomicBool::new(false);
static BUSY_WAKER: AtomicWaker = AtomicWaker::new();
/// Set when a radio interrupt has switched the `SUBGHZ_RADIO` vector off, until `await_irq` switches it on again.
///
/// The radio keeps its interrupt line asserted until the caller clears it in the radio. Switching the vector on
/// while that is the case raises the interrupt again straight away, which would leave a stale event for the next
/// `await_irq`. So only `await_irq` switches the vector back on, never `wait_on_busy`.
static RADIO_IRQ_OFF: AtomicBool = AtomicBool::new(false);

/// `RFBUSY` is EXTI line 45, a configurable line: bit 13 of the second EXTI register.
const BUSY_EXTI_LINE: usize = 13;

/// EXTI register block holding the interrupt mask registers: `EXTI` on STM32WLE, `EXTI.cpu(0)` on STM32WL5x.
#[cfg(exti_w)]
fn exti_mask_regs() -> pac::exti::Cpu {
    pac::EXTI.cpu(0)
}
#[cfg(not(exti_w))]
fn exti_mask_regs() -> pac::exti::Exti {
    pac::EXTI
}

/// Interrupt handler for `SUBGHZ_RADIO` interrupts.
///
/// The `SUBGHZ_RADIO` vector is shared by two sources: the radio interrupt lines (EXTI line 44, a
/// direct line that the radio flags) and the `RFBUSY` falling edge (EXTI line 45).
pub struct RadioInterruptHandler {
    _marker: PhantomData<()>,
}

impl crate::interrupt::typelevel::Handler<crate::interrupt::typelevel::SUBGHZ_RADIO> for RadioInterruptHandler {
    unsafe fn on_interrupt() {
        // RFBUSY went low. Clear the edge and mask the line again: `wait_on_busy` re-arms it per wait.
        if pac::EXTI.pr(1).read().line(BUSY_EXTI_LINE) {
            pac::EXTI.pr(1).write_value(pac::exti::regs::Lines(1 << BUSY_EXTI_LINE));
            exti_mask_regs().imr(1).modify(|w| w.set_line(BUSY_EXTI_LINE, false));
            BUSY_WAKER.wake();
            return;
        }

        // A radio interrupt. It stays asserted until the caller clears it in the radio, so stop the
        // vector here and let `await_irq` re-enable it. A pending `wait_on_busy` is woken as well,
        // because it shares the vector and would otherwise not see the `RFBUSY` edge while the vector is off.
        RADIO_IRQ_OFF.store(true, Ordering::Release);
        crate::interrupt::typelevel::SUBGHZ_RADIO::disable();
        RADIO_PENDING.store(true, Ordering::Release);
        RADIO_WAKER.wake();
        BUSY_WAKER.wake();
    }
}

/// Source selection for the `SUBGHZSPI_NSS` line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum NssSource {
    /// Sub-GHz SPI NSS signal driven from software via `PWR_SUBGHZSPICR.NSS`.
    Software,
    /// Sub-GHz SPI NSS signal driven autonomously from `LPTIM3_OUT`.
    Lptim3,
}

/// Configure the hardware driving source for the `SUBGHZSPI_NSS` signal.
pub fn set_nss_source(source: NssSource) {
    match source {
        NssSource::Software => {
            pac::PWR
                .cr1()
                .modify(|w| w.set_subghzspinsssel(pac::pwr::vals::Subghzspinsssel::Subghzspicr));
        }
        NssSource::Lptim3 => {
            pac::PWR
                .cr1()
                .modify(|w| w.set_subghzspinsssel(pac::pwr::vals::Subghzspinsssel::Lptim3));
        }
    }
}

/// The radio did not release `RFBUSY` within the allowed time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct BusyTimeout;

/// Control of the sub-GHz radio: reset, `RFBUSY` monitoring, wakeup and interrupts.
///
/// This is not a radio driver. It does not know the radio's command set; send commands through [`SubghzSpi`].
pub struct SubghzControl<'d> {
    _phantom: PhantomData<&'d ()>,
}

impl<'d> SubghzControl<'d> {
    /// Create a new `SubghzControl` driver instance.
    pub fn new(
        _irq: impl crate::interrupt::typelevel::Binding<crate::interrupt::typelevel::SUBGHZ_RADIO, RadioInterruptHandler>
        + 'd,
    ) -> Self {
        unsafe {
            cortex_m::peripheral::NVIC::unpend(pac::Interrupt::SUBGHZ_RADIO);
            crate::interrupt::typelevel::SUBGHZ_RADIO::enable();
        }
        Self { _phantom: PhantomData }
    }

    /// Check if the radio is currently asserting the `RFBUSY` signal.
    #[inline(always)]
    pub fn is_busy(&self) -> bool {
        pac::PWR.sr2().read().rfbusys()
    }

    /// Check if the masked radio busy signal (`RFBUSYMS`) is currently asserted.
    ///
    /// The reference manual describes `RFBUSYMS` as set when `NSS` goes low and held for a minimum
    /// time after the command, so that it covers the gap before the radio raises `RFBUSY`. On an
    /// STM32WLE5CC it was still set 400 ms after `RFBUSY` had gone low, following either a command
    /// or a plain `NSS` pulse, so [`wait_on_busy`](Self::wait_on_busy) uses `RFBUSY` and not this.
    #[inline(always)]
    pub fn is_busy_masked(&self) -> bool {
        pac::PWR.sr2().read().rfbusyms()
    }

    /// Asynchronously wait for the radio to clear the `RFBUSY` signal.
    ///
    /// Most commands release `RFBUSY` within a few microseconds and are waited for by polling. A
    /// longer wait, such as calibration or waking from sleep (about 1.5 ms), sleeps until the `RFBUSY`
    /// falling edge interrupts through EXTI line 45, so the executor can idle.
    ///
    /// This shares the `SUBGHZ_RADIO` vector with [`await_irq`](Self::await_irq), so the interrupt
    /// must be bound with [`RadioInterruptHandler`].
    ///
    /// After a radio interrupt the vector stays off until [`await_irq`](Self::await_irq) switches it on again.
    /// In that time this polls instead of sleeping, because it must not switch the vector on itself.
    pub async fn wait_on_busy(&mut self) {
        // Short spin for fast commands (~1-2 µs)
        for _ in 0..50 {
            if !self.is_busy() {
                return;
            }
            core::hint::spin_loop();
        }

        while self.is_busy() {
            if RADIO_IRQ_OFF.load(Ordering::Acquire) {
                embassy_futures::yield_now().await;
            } else {
                self.wait_for_busy_edge().await;
            }
        }
    }

    /// Sleep until `RFBUSY` falls, or until a radio interrupt switches the vector off.
    async fn wait_for_busy_edge(&mut self) {
        poll_fn(|cx| {
            BUSY_WAKER.register(cx.waker());

            if !self.is_busy() || RADIO_IRQ_OFF.load(Ordering::Acquire) {
                Self::disarm_busy_irq();
                return Poll::Ready(());
            }

            // Arm after registering the waker, then check again: an edge between the check above and
            // arming would otherwise be lost.
            Self::arm_busy_irq();
            if !self.is_busy() || RADIO_IRQ_OFF.load(Ordering::Acquire) {
                Self::disarm_busy_irq();
                return Poll::Ready(());
            }

            Poll::Pending
        })
        .await;
    }

    fn arm_busy_irq() {
        critical_section::with(|_| {
            pac::EXTI.pr(1).write_value(pac::exti::regs::Lines(1 << BUSY_EXTI_LINE));
            pac::EXTI.ftsr(1).modify(|w| w.set_line(BUSY_EXTI_LINE, true));
            exti_mask_regs().imr(1).modify(|w| w.set_line(BUSY_EXTI_LINE, true));
            // The vector is only switched on if no radio interrupt has switched it off (see `RADIO_IRQ_OFF`).
            // Checked in the critical section so the handler cannot switch it off in between.
            if !RADIO_IRQ_OFF.load(Ordering::Acquire) {
                unsafe { crate::interrupt::typelevel::SUBGHZ_RADIO::enable() };
            }
        });
    }

    fn disarm_busy_irq() {
        critical_section::with(|_| {
            exti_mask_regs().imr(1).modify(|w| w.set_line(BUSY_EXTI_LINE, false));
        });
    }

    /// Like [`wait_on_busy`](Self::wait_on_busy), but gives up after `timeout`.
    ///
    /// A radio that never releases `RFBUSY` (for example after a bad command sequence)
    /// makes `wait_on_busy` wait forever. Reset the radio with [`reset`](Self::reset) on error.
    pub async fn wait_on_busy_timeout(&mut self, timeout: embassy_time::Duration) -> Result<(), BusyTimeout> {
        embassy_time::with_timeout(timeout, self.wait_on_busy())
            .await
            .map_err(|_| BusyTimeout)
    }

    /// Block the current thread until the radio clears the `RFBUSY` signal.
    pub fn blocking_wait_on_busy(&mut self) {
        while self.is_busy() {
            core::hint::spin_loop();
        }
    }

    /// Wake the radio from sleep.
    ///
    /// The radio leaves sleep on a falling edge of `NSS`. This holds `NSS` low until the radio
    /// releases `RFBUSY`, so the next SPI command can be sent immediately. It is harmless if the
    /// radio is already awake. Releasing `NSS` is cancel-safe.
    pub async fn wakeup(&mut self) {
        let _nss = spi::NssGuard::assert();
        // A sleeping radio already holds RFBUSY high, so there is no window in which this can
        // return before the radio has woken.
        self.wait_on_busy().await;
    }

    /// Choose which radio events wake the CPU from **Standby** (`PWR_CR3.EWRFIRQ` and `EWRFBUSY`).
    ///
    /// - `irq`: a radio interrupt (IRQ[2:0]) wakes the CPU.
    /// - `busy`: an edge on `RFBUSY` wakes the CPU. The active edge is `PWR_CR4.WRFBUSYP` and is
    ///   not changed here; it defaults to the rising edge.
    ///
    /// Both are off by default. Waking from Standby restarts the program as after a reset.
    ///
    /// This does not affect Run, Sleep or Stop. There the radio interrupt and the `RFBUSY` edge
    /// reach the CPU through EXTI lines 44 and 45 and the `SUBGHZ_RADIO` vector. Line 45 is armed by
    /// [`wait_on_busy`](Self::wait_on_busy). Waking from Stop on a radio interrupt is not configured by this driver.
    pub fn set_wakeup_sources(&mut self, irq: bool, busy: bool) {
        pac::PWR.cr3().modify(|w| {
            w.set_ewrfirq(irq);
            w.set_ewrfbusy(busy);
        });
    }

    /// Asynchronously perform a hardware reset of the sub-GHz radio via `RCC_CSR.RFRST`.
    pub async fn reset(&mut self) {
        pac::RCC.csr().modify(|w| w.set_rfrst(true));
        embassy_time::Timer::after_micros(100).await;
        pac::RCC.csr().modify(|w| w.set_rfrst(false));

        while pac::RCC.csr().read().rfrstf() {
            embassy_futures::yield_now().await;
        }
    }

    /// Blockingly perform a hardware reset of the sub-GHz radio via `RCC_CSR.RFRST`.
    pub fn blocking_reset(&mut self) {
        pac::RCC.csr().modify(|w| w.set_rfrst(true));
        embassy_time::block_for(embassy_time::Duration::from_micros(100));
        pac::RCC.csr().modify(|w| w.set_rfrst(false));

        while pac::RCC.csr().read().rfrstf() {
            core::hint::spin_loop();
        }
    }

    /// Asynchronously wait for a radio interrupt event (`SUBGHZ_RADIO`).
    ///
    /// The radio keeps the interrupt asserted until it is cleared in the radio, for example with
    /// `ClrIrqStatus`. Clear it before waiting again, or this returns straight away.
    pub async fn await_irq(&mut self) {
        poll_fn(|cx| {
            if RADIO_PENDING.swap(false, Ordering::AcqRel) {
                return Poll::Ready(());
            }

            RADIO_WAKER.register(cx.waker());

            if RADIO_PENDING.swap(false, Ordering::AcqRel) {
                return Poll::Ready(());
            }

            critical_section::with(|_| {
                RADIO_IRQ_OFF.store(false, Ordering::Release);
                unsafe {
                    cortex_m::peripheral::NVIC::unpend(pac::Interrupt::SUBGHZ_RADIO);
                    crate::interrupt::typelevel::SUBGHZ_RADIO::enable();
                }
            });

            Poll::Pending
        })
        .await;
    }
}
