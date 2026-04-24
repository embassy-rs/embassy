//! CSI-2 Host controller.
//!
//! Receives MIPI CSI-2 packets from an external image sensor and forwards the
//! stream to the DCMIPP pixel pipeline. This driver covers the host / lane
//! merger / D-PHY power control and timing: enable/disable, lane selection,
//! D-PHY frequency configuration, per-virtual-channel start/stop, and error
//! reporting. Pixel-format and memory-destination configuration lives on the
//! DCMIPP side (see the `dcmipp` module).
//!
//! CSI-2 data and clock lanes are fixed differential MIPI D-PHY pads on the
//! chip and are not GPIO-muxable, so this driver has no pin traits or
//! `new_with_pins` constructor.
//!
//! ## Virtual channels
//! Up to four virtual channels (VC0..VC3) can be enabled simultaneously via
//! the [`Config::virtual_channels`] bitmask. Each enabled VC latches all
//! data types (`ALLDT = 1`) by default; callers that need finer filtering can
//! program `VCxCFGR1..4` directly through `pac::CSI`.
//!
//! ## D-PHY init
//! The RM0486 §40.6 init sequence is applied in [`Csi::new`]:
//!   1. PRCR.PEN = 1 (release the D-PHY digital section from reset).
//!   2. PFCR.DLD = 0 (force Rx mode on data lane 0);
//!      PFCR.CCFR = round((F<sub>cfg</sub> MHz − 17) × 4);
//!      PFCR.HSFR = high-speed frequency band from `data_rate_mbps`.
//!   3. PTCR0.TRSEN = 0, PTCR0.TCKEN = 0 (disable the D-PHY test interface).
//!   4. LMCFGR lane count + D0/D1 mapping.
//!   5. VCxCFGR1.ALLDT = 1 for every enabled VC.
//!   6. PCR lane enables.
//!
//! HSFR is derived from `data_rate_mbps` using the linear approximation
//! `((rate − 80) / 40).clamp(0, 0x7F)`. This is the same form used by the
//! published ST D-PHY drivers and is a reasonable band estimate for most
//! rates between 80 Mbps and 2.5 Gbps; callers that need a specific band
//! (e.g. to match an exact PLL output) can pin it with
//! [`Config::hs_freq_range_override`].

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
    /// Nominal data rate per lane in Mbps. Used to compute the D-PHY
    /// high-speed frequency band (`PFCR.HSFR`), and recorded for downstream
    /// bandwidth / watchdog calculations.
    pub data_rate_mbps: u32,
    /// Virtual channels to start together as one stream. Bit `n` enables VC
    /// `n` (0..=3). Defaults to `0b0001` (VC0 only).
    pub virtual_channels: u8,
    /// Configuration-clock frequency in MHz feeding the D-PHY test/control
    /// interface. Drives `PFCR.CCFR = round((F − 17) × 4)`. Defaults to
    /// 24 MHz (the HSI_DIV default on the N6).
    pub config_clock_mhz: u32,
    /// Override for the D-PHY high-speed frequency-range band
    /// (`PFCR.HSFR`, 7 bits). `None` uses the linear approximation
    /// documented at the module level.
    pub hs_freq_range_override: Option<u8>,
}

impl Config {
    /// Create a configuration with the given lane count and per-lane data
    /// rate. Defaults: VC0 only, 24 MHz config clock, HSFR auto-derived.
    pub const fn new(lanes: LaneCount, data_rate_mbps: u32) -> Self {
        Self {
            lanes,
            data_rate_mbps,
            virtual_channels: 0b0001,
            config_clock_mhz: 24,
            hs_freq_range_override: None,
        }
    }
}

/// CSI-2 Host driver.
pub struct Csi<'d, T: Instance> {
    _peri: Peri<'d, T>,
    virtual_channels: u8,
}

impl<'d, T: Instance> Csi<'d, T> {
    /// Create a new CSI-2 Host driver.
    ///
    /// Enables the peripheral clock, performs a reset, programs the D-PHY
    /// (config-clock range, HS frequency band, test-interface off, lane
    /// mapping), and arms each enabled virtual channel. Leaves the receiver
    /// itself disabled. Call [`start`] after the sensor has started
    /// streaming.
    ///
    /// [`start`]: Self::start
    pub fn new(
        peri: Peri<'d, T>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        config: Config,
    ) -> Self {
        rcc::enable_and_reset::<T>();

        let r = T::regs();

        // Step 1: release the D-PHY digital section from reset.
        r.prcr().modify(|w| w.set_pen(true));

        // Step 2: D-PHY frequency configuration.
        let hsfr = match config.hs_freq_range_override {
            Some(v) => v & 0x7F,
            None => hsfr_from_mbps(config.data_rate_mbps),
        };
        let ccfr = ccfr_from_mhz(config.config_clock_mhz);
        r.pfcr().modify(|w| {
            w.set_dld(false); // data lane 0 in Rx mode
            w.set_ccfr(ccfr);
            w.set_hsfr(hsfr);
        });

        // Step 3: disable the D-PHY test interface (reset-mode write).
        r.ptcr0().modify(|w| {
            w.set_trsen(false);
            w.set_tcken(false);
        });

        // Step 4: lane merger — 1 or 2 lanes, straight mapping (D0→0, D1→1).
        r.lmcfgr().modify(|w| {
            w.set_lanenb(config.lanes.as_u8());
            w.set_dl0map(1);
            w.set_dl1map(2);
        });

        // Step 5: per-VC configuration. Enable ALLDT for every VC in the
        // mask so all data types are forwarded; callers that want finer
        // filtering can clear ALLDT and set per-DT bits through pac::CSI.
        for bit in 0..4u8 {
            if config.virtual_channels & (1 << bit) != 0 {
                match bit {
                    0 => r.vc0cfgr1().modify(|w| w.set_alldt(true)),
                    1 => r.vc1cfgr1().modify(|w| w.set_alldt(true)),
                    2 => r.vc2cfgr1().modify(|w| w.set_alldt(true)),
                    3 => r.vc3cfgr1().modify(|w| w.set_alldt(true)),
                    _ => {}
                }
            }
        }

        // Step 6: power up the PHY and enable clock + data lanes.
        r.pcr().modify(|w| {
            w.set_pwrdown(false);
            w.set_clen(true);
            w.set_dl0en(true);
            w.set_dl1en(matches!(config.lanes, LaneCount::Two));
        });

        unsafe { T::Interrupt::enable() };

        Self {
            _peri: peri,
            virtual_channels: config.virtual_channels,
        }
    }

    /// Enable the CSI-2 receiver. After this call incoming packets are
    /// dispatched to DCMIPP and per-virtual-channel outputs latch data.
    /// Starts every virtual channel in [`Config::virtual_channels`].
    pub fn start(&mut self) {
        let vcs = self.virtual_channels;
        T::regs().cr().modify(|w| {
            w.set_csien(true);
            if vcs & 0b0001 != 0 {
                w.set_vc0start(true);
            }
            if vcs & 0b0010 != 0 {
                w.set_vc1start(true);
            }
            if vcs & 0b0100 != 0 {
                w.set_vc2start(true);
            }
            if vcs & 0b1000 != 0 {
                w.set_vc3start(true);
            }
        });
    }

    /// Disable the CSI-2 receiver. Stops every virtual channel in
    /// [`Config::virtual_channels`].
    pub fn stop(&mut self) {
        let vcs = self.virtual_channels;
        T::regs().cr().modify(|w| {
            if vcs & 0b0001 != 0 {
                w.set_vc0stop(true);
            }
            if vcs & 0b0010 != 0 {
                w.set_vc1stop(true);
            }
            if vcs & 0b0100 != 0 {
                w.set_vc2stop(true);
            }
            if vcs & 0b1000 != 0 {
                w.set_vc3stop(true);
            }
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

/// Config-clock frequency to `PFCR.CCFR`, 6-bit field clamped to 0..=0x3F.
/// Formula: `round((F_MHz − 17) × 4)`.
const fn ccfr_from_mhz(mhz: u32) -> u8 {
    let signed = mhz as i32 - 17;
    let scaled = signed * 4;
    if scaled < 0 {
        0
    } else if scaled > 0x3F {
        0x3F
    } else {
        scaled as u8
    }
}

/// Data-rate to `PFCR.HSFR`, 7-bit band selector clamped to 0..=0x7F. Uses
/// the linear approximation `(mbps − 80) / 40` documented at the module
/// level; override via `Config::hs_freq_range_override` if the sensor PLL
/// targets a specific published band.
const fn hsfr_from_mbps(mbps: u32) -> u8 {
    let m = if mbps < 80 { 0 } else { (mbps - 80) / 40 };
    if m > 0x7F { 0x7F } else { m as u8 }
}

impl<'d, T: Instance> Drop for Csi<'d, T> {
    fn drop(&mut self) {
        let r = T::regs();
        r.cr().modify(|w| {
            w.set_vc0stop(true);
            w.set_vc1stop(true);
            w.set_vc2stop(true);
            w.set_vc3stop(true);
            w.set_csien(false);
        });
        r.pcr().modify(|w| {
            w.set_pwrdown(true);
            w.set_clen(false);
            w.set_dl0en(false);
            w.set_dl1en(false);
        });
        r.prcr().modify(|w| w.set_pen(false));
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
