//! CSI-2 Host controller.
//!
//! Receives MIPI CSI-2 packets from an external image sensor and forwards the
//! stream to the DCMIPP pixel pipeline. This driver covers the host / lane
//! merger / D-PHY power control: enable/disable, lane selection, and error
//! reporting. Pixel-format and memory-destination configuration lives on the
//! DCMIPP side (see the `dcmipp` module).
//!
//! CSI-2 data and clock lanes are fixed differential MIPI D-PHY pads on the
//! chip and are not GPIO-muxable, so this driver has no pin traits or
//! `new_with_pins` constructor.

use core::future::poll_fn;
use core::marker::PhantomData;
use core::task::Poll;

use embassy_hal_internal::PeripheralType;
use embassy_sync::waitqueue::AtomicWaker;

use crate::interrupt::typelevel::Interrupt;
use crate::{Peri, interrupt, rcc};

/// Interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        // Mask any pending error interrupts so the handler doesn't refire while
        // the task drains flags. The task clears the flags via SR0 write-1-to-clear.
        T::regs().ier0().modify(|w| {
            w.set_eccerrie(false);
            w.set_crcerrie(false);
            w.set_iderrie(false);
            w.set_spkterrie(false);
            w.set_wderrie(false);
        });
        STATE.waker.wake();
    }
}

struct State {
    waker: AtomicWaker,
}

impl State {
    const fn new() -> State {
        State {
            waker: AtomicWaker::new(),
        }
    }
}

static STATE: State = State::new();

/// Number of CSI-2 data lanes in use.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum LaneCount {
    /// Single-lane (D0).
    One,
    /// Dual-lane (D0 + D1).
    Two,
}

impl LaneCount {
    const fn as_u8(self) -> u8 {
        match self {
            LaneCount::One => 1,
            LaneCount::Two => 2,
        }
    }
}

/// CSI-2 Host error.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    /// Uncorrectable ECC error on a CSI-2 short packet header.
    Ecc,
    /// CRC mismatch on a long packet payload.
    Crc,
    /// Short packet with an invalid data type or format error.
    ShortPacket,
    /// Unknown / invalid data type identifier.
    DataId,
    /// No packet received within the configured watchdog window.
    Watchdog,
    /// D-PHY / lane-level synchronisation failure on either data lane.
    LaneSync,
}

/// CSI-2 Host configuration.
#[non_exhaustive]
pub struct Config {
    /// Number of data lanes in use.
    pub lanes: LaneCount,
    /// Nominal data rate per lane in Mbps. Recorded for software use / reference;
    /// the on-chip D-PHY calibration handles actual timing setup automatically
    /// on this IP, so this field does not drive register values today. Populate
    /// it anyway so downstream code (watchdog timeouts, bandwidth checks) can
    /// see what the sensor is configured for.
    pub data_rate_mbps: u32,
}

impl Config {
    /// Create a configuration with the given lane count and per-lane data rate.
    pub const fn new(lanes: LaneCount, data_rate_mbps: u32) -> Self {
        Self { lanes, data_rate_mbps }
    }
}

/// CSI-2 Host driver.
pub struct Csi<'d, T: Instance> {
    _peri: Peri<'d, T>,
}

impl<'d, T: Instance> Csi<'d, T> {
    /// Create a new CSI-2 Host driver.
    ///
    /// Enables the peripheral clock, performs a reset, configures the lane
    /// merger + D-PHY, and leaves the controller disabled. Call [`start`]
    /// after the sensor has started streaming.
    ///
    /// [`start`]: Self::start
    pub fn new(
        peri: Peri<'d, T>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        config: Config,
    ) -> Self {
        rcc::enable_and_reset::<T>();

        let r = T::regs();

        // Lane merger: 1 or 2 lanes, straight mapping (D0→0, D1→1).
        r.lmcfgr().modify(|w| {
            w.set_lanenb(config.lanes.as_u8());
            w.set_dl0map(1);
            w.set_dl1map(2);
        });

        // Power up the PHY and enable clock + data lanes.
        r.pcr().modify(|w| {
            w.set_pwrdown(false);
            w.set_clen(true);
            w.set_dl0en(true);
            w.set_dl1en(matches!(config.lanes, LaneCount::Two));
        });

        unsafe { T::Interrupt::enable() };

        Self { _peri: peri }
    }

    /// Enable the CSI-2 receiver. After this call incoming packets are
    /// dispatched to DCMIPP and per-virtual-channel outputs latch data.
    pub fn start(&mut self) {
        T::regs().cr().modify(|w| {
            w.set_csien(true);
            // Start virtual channel 0 — the only VC the DCMIPP pipes listen on
            // by default. Callers that use other VCs can configure them via
            // the DCMIPP driver and set the remaining start bits themselves.
            w.set_vc0start(true);
        });
    }

    /// Disable the CSI-2 receiver.
    pub fn stop(&mut self) {
        T::regs().cr().modify(|w| {
            w.set_vc0stop(true);
            w.set_csien(false);
        });
    }

    /// Wait for an error interrupt. Re-arms the same error interrupts before
    /// returning so the next error surfaces on the following call.
    ///
    /// Returns the highest-priority error flag currently set in SR0/SR1. If
    /// several flags are set at the same time, the returned `Error` variant is
    /// chosen in an implementation-defined order; call again to drain the rest.
    pub async fn wait_error(&mut self) -> Error {
        let r = T::regs();

        // Clear any stale flags so we only wake on fresh errors.
        r.fcr0().write(|w| {
            w.set_ceccerrf(true);
            w.set_ccrcerrf(true);
            w.set_ciderrf(true);
            w.set_cspkterrf(true);
            w.set_cwderrf(true);
        });

        // Arm the interrupts we care about.
        r.ier0().modify(|w| {
            w.set_eccerrie(true);
            w.set_crcerrie(true);
            w.set_iderrie(true);
            w.set_spkterrie(true);
            w.set_wderrie(true);
        });

        poll_fn(|cx| {
            STATE.waker.register(cx.waker());

            let sr0 = r.sr0().read();
            if sr0.eccerrf() {
                r.fcr0().write(|w| w.set_ceccerrf(true));
                return Poll::Ready(Error::Ecc);
            }
            if sr0.crcerrf() {
                r.fcr0().write(|w| w.set_ccrcerrf(true));
                return Poll::Ready(Error::Crc);
            }
            if sr0.spkterrf() {
                r.fcr0().write(|w| w.set_cspkterrf(true));
                return Poll::Ready(Error::ShortPacket);
            }
            if sr0.iderrf() {
                r.fcr0().write(|w| w.set_ciderrf(true));
                return Poll::Ready(Error::DataId);
            }
            if sr0.wderrf() {
                r.fcr0().write(|w| w.set_cwderrf(true));
                return Poll::Ready(Error::Watchdog);
            }
            if sr0.syncerrf() {
                // `syncerrf` has no direct clear bit on this IP version; it is
                // cleared implicitly when PHY lane synchronisation is
                // re-established. Surface it for the caller regardless.
                return Poll::Ready(Error::LaneSync);
            }
            Poll::Pending
        })
        .await
    }
}

impl<'d, T: Instance> Drop for Csi<'d, T> {
    fn drop(&mut self) {
        let r = T::regs();
        r.cr().modify(|w| {
            w.set_vc0stop(true);
            w.set_csien(false);
        });
        r.pcr().modify(|w| {
            w.set_pwrdown(true);
            w.set_clen(false);
            w.set_dl0en(false);
            w.set_dl1en(false);
        });
        T::Interrupt::disable();
    }
}

trait SealedInstance: crate::rcc::RccPeripheral {
    fn regs() -> crate::pac::csi::Csi;
}

/// CSI-2 Host instance.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + 'static + Send {
    /// Interrupt for this instance.
    type Interrupt: interrupt::typelevel::Interrupt;
}

foreach_interrupt! {
    ($inst:ident, csi, CSI, GLOBAL, $irq:ident) => {
        impl SealedInstance for crate::peripherals::$inst {
            fn regs() -> crate::pac::csi::Csi {
                crate::pac::$inst
            }
        }
        impl Instance for crate::peripherals::$inst {
            type Interrupt = crate::interrupt::typelevel::$irq;
        }
    };
}
