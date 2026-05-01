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
    /// Nominal data rate per lane in Mbps. Used to look up the D-PHY band
    /// table (`PFCR.HSFR` + the Synopsys oscillator target).
    pub data_rate_mbps: u32,
    /// Virtual channels to start together as one stream. Bit `n` enables VC
    /// `n` (0..=3). Defaults to `0b0001` (VC0 only).
    pub virtual_channels: u8,
    /// Configuration-clock frequency in MHz feeding the D-PHY test/control
    /// interface. Recorded but unused — the BSP-correct CCFR value is
    /// hard-coded to 0x28 (matches the Synopsys D-PHY reference) so this
    /// field is not driven into the register today.
    pub config_clock_mhz: u32,
    /// Override for the D-PHY high-speed frequency-range band
    /// (`PFCR.HSFR`, 7 bits). `None` looks up the band from
    /// `data_rate_mbps`.
    pub hs_freq_range_override: Option<u8>,
    /// `VCxCFGR1.CDTFT` — common data-type format for every enabled VC.
    /// Encodes the bits-per-pixel of the incoming stream:
    ///   0=6, 1=7, 2=8, 3=10, 4=12, 5=14, 6=16. Defaults to 3 (RAW10),
    ///   the most common Bayer sensor format.
    pub data_type_format: u8,
}

impl Config {
    /// Create a configuration with the given lane count and per-lane data
    /// rate. Defaults: VC0 only, 24 MHz config clock, HSFR auto-derived,
    /// 10 bits-per-pixel (RAW10).
    pub const fn new(lanes: LaneCount, data_rate_mbps: u32) -> Self {
        Self {
            lanes,
            data_rate_mbps,
            virtual_channels: 0b0001,
            config_clock_mhz: 24,
            hs_freq_range_override: None,
            data_type_format: 3,
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

        // Pick the Synopsys D-PHY band entry for this data rate. The
        // hsfreqrange + osc_freq_target values come straight from the ST
        // BSP (RM0486 §40 references the Synopsys D-PHY internal table).
        let band = match config.hs_freq_range_override {
            Some(v) => SnpsBand {
                hs_freq_range: v & 0x7F,
                osc_freq_target: 460,
            },
            None => snps_band_for_mbps(config.data_rate_mbps),
        };

        // Disable CSI before reconfiguring.
        r.cr().modify(|w| w.set_csien(false));

        // Lane merger: 1 or 2 lanes, straight D0/D1 mapping.
        r.lmcfgr().modify(|w| {
            w.set_lanenb(config.lanes.as_u8());
            w.set_dl0map(1);
            w.set_dl1map(2);
        });

        // Re-enable CSI so the D-PHY config sequence below can drive
        // the test interface.
        r.cr().modify(|w| w.set_csien(true));

        // ---- D-PHY init sequence (BSP §HAL_DCMIPP_CSI_SetConfig) ----
        // Hold the D-PHY in reset.
        r.prcr().modify(|w| w.set_pen(false));

        // Disable lanes during config.
        r.pcr().write(|w| {
            w.set_pwrdown(false);
            w.set_clen(false);
            w.set_dl0en(false);
            w.set_dl1en(false);
        });

        // 15 ns testclk pulse before any test-interface write. We pulse
        // through PTCR0.TCKEN; one register write at 100 MHz APB is
        // ≥10 ns, which is sufficient.
        r.ptcr0().write(|w| w.set_tcken(true));
        r.ptcr0().write(|w| w.0 = 0);

        // Initial PFCR with HSFR but no DLD yet.
        r.pfcr().write(|w| {
            w.set_ccfr(0x28); // BSP-fixed value (matches Synopsys table)
            w.set_hsfr(band.hs_freq_range);
        });

        // Synopsys D-PHY internal-register writes via the testbench
        // interface. RM0486 §40.5.7 / Synopsys "DesignWare D-PHY" doc:
        //   reg 0x08 = 0x38      deskew_polarity
        //   reg 0xE4 = 0x11      DLL prog enable + counter_for_des_en
        //   reg 0xE3 = osc_hi    DLL target oscillator high byte
        //   reg 0xE3 = osc_lo    DLL target oscillator low  byte
        write_phy_reg::<T>(0x00, 0x08, 0x38);
        write_phy_reg::<T>(0x00, 0xE4, 0x11);
        write_phy_reg::<T>(0x00, 0xE3, (band.osc_freq_target >> 8) as u8);
        write_phy_reg::<T>(0x00, 0xE3, band.osc_freq_target as u8);

        // Final PFCR: same HSFR, hardcoded CCFR=0x28, DLD=1 (D0 RX dir).
        r.pfcr().write(|w| {
            w.set_ccfr(0x28);
            w.set_hsfr(band.hs_freq_range);
            w.set_dld(true);
        });

        // Enable lanes; PWRDOWN bit is set per BSP (the metapac field is
        // named "Power down" but on this IP it functions as the Synopsys
        // shutdownz active-high enable — leaving it cleared keeps the PHY
        // in shutdown).
        r.pcr().write(|w| {
            w.set_pwrdown(true);
            w.set_clen(true);
            w.set_dl0en(true);
            w.set_dl1en(matches!(config.lanes, LaneCount::Two));
        });

        // Per-VC: ALLDT=true and CDTFT=data_type_format for each enabled
        // VC. CDTFT is required — without it the CSI ingress can't decode
        // the bit-width of the incoming pixels and DCMIPP receives nothing.
        let cdtft = config.data_type_format & 0x1F;
        for bit in 0..4u8 {
            if config.virtual_channels & (1 << bit) != 0 {
                match bit {
                    0 => r.vc0cfgr1().write(|w| {
                        w.set_alldt(true);
                        w.set_cdtft(cdtft);
                    }),
                    1 => r.vc1cfgr1().write(|w| {
                        w.set_alldt(true);
                        w.set_cdtft(cdtft);
                    }),
                    2 => r.vc2cfgr1().write(|w| {
                        w.set_alldt(true);
                        w.set_cdtft(cdtft);
                    }),
                    3 => r.vc3cfgr1().write(|w| {
                        w.set_alldt(true);
                        w.set_cdtft(cdtft);
                    }),
                    _ => {}
                }
            }
        }

        // Release the D-PHY from reset.
        r.prcr().modify(|w| w.set_pen(true));
        // Clear PMCR (drop any forced Tx/Rx-mode requests).
        r.pmcr().write(|w| w.0 = 0);

        unsafe { T::Interrupt::enable() };

        let _ = config.config_clock_mhz; // CCFR is hardcoded per BSP

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

/// Synopsys D-PHY band parameters for a given data rate.
struct SnpsBand {
    hs_freq_range: u8,
    osc_freq_target: u16,
}

/// Synopsys D-PHY band table (RM0486 §40 / DesignWare D-PHY datasheet).
/// Entries cover 80..2500 Mbps in coarse steps; we pick the closest band
/// at or above the requested rate. Mirrors `SNPS_Freqs` in the ST BSP
/// (`stm32n6xx_hal_dcmipp.c` v1.3.0).
fn snps_band_for_mbps(mbps: u32) -> SnpsBand {
    // (rate_mbps, hs_freq_range, osc_freq_target).
    const T: &[(u32, u8, u16)] = &[
        (80, 0x00, 460),
        (90, 0x10, 460),
        (100, 0x20, 460),
        (110, 0x30, 460),
        (120, 0x01, 460),
        (130, 0x11, 460),
        (140, 0x21, 460),
        (150, 0x31, 460),
        (160, 0x02, 460),
        (170, 0x12, 460),
        (180, 0x22, 460),
        (190, 0x32, 460),
        (205, 0x03, 460),
        (220, 0x13, 460),
        (235, 0x23, 460),
        (250, 0x33, 460),
        (275, 0x04, 460),
        (300, 0x14, 460),
        (325, 0x25, 460),
        (350, 0x35, 460),
        (400, 0x05, 460),
        (450, 0x16, 460),
        (500, 0x26, 460),
        (550, 0x37, 460),
        (600, 0x07, 460),
        (650, 0x18, 460),
        (700, 0x28, 460),
        (750, 0x39, 460),
        (800, 0x09, 460),
        (850, 0x19, 460),
        (900, 0x29, 460),
        (950, 0x3A, 460),
        (1000, 0x0A, 460),
        (1050, 0x1A, 460),
        (1100, 0x2A, 460),
        (1150, 0x3B, 460),
        (1200, 0x0B, 460),
        (1250, 0x1B, 460),
        (1300, 0x2B, 460),
        (1350, 0x3C, 460),
        (1400, 0x0C, 460),
        (1450, 0x1C, 460),
        (1500, 0x2C, 460),
        (1550, 0x3D, 285),
        (1600, 0x0D, 295),
        (1650, 0x1D, 304),
        (1700, 0x2E, 313),
        (1750, 0x3E, 322),
        (1800, 0x0E, 331),
        (1850, 0x1E, 341),
        (1900, 0x2F, 350),
        (1950, 0x3F, 359),
        (2000, 0x0F, 368),
        (2050, 0x40, 377),
        (2100, 0x41, 387),
        (2150, 0x42, 396),
        (2200, 0x43, 405),
        (2250, 0x44, 414),
        (2300, 0x45, 423),
        (2350, 0x46, 432),
        (2400, 0x47, 442),
        (2450, 0x48, 451),
        (2500, 0x49, 460),
    ];
    let mut best = T[0];
    for &entry in T {
        best = entry;
        if entry.0 >= mbps {
            break;
        }
    }
    SnpsBand {
        hs_freq_range: best.1,
        osc_freq_target: best.2,
    }
}

/// Synopsys D-PHY testbench register write (RM0486 §40 §"D-PHY internal
/// registers" / DesignWare D-PHY user guide §5.2.3.2). Writes a 12-bit
/// register address (split into MSB nibble + LSB byte) and an 8-bit value
/// through the test-interface clock/data signals on PTCR0/PTCR1.
fn write_phy_reg<T: Instance>(reg_msb: u8, reg_lsb: u8, val: u8) {
    let r = T::regs();
    // Phase A: latch the 4-bit testcode MSBs.
    r.ptcr1().write(|w| w.set_twm(true));
    r.ptcr0().write(|w| w.set_tcken(true));
    r.ptcr1().write(|w| w.set_twm(true));
    r.ptcr0().write(|w| w.0 = 0);
    r.ptcr1().write(|w| w.0 = 0);

    r.ptcr1().write(|w| w.set_tdi(reg_msb));
    r.ptcr0().write(|w| w.set_tcken(true));

    // Phase B: latch the 8-bit testcode LSBs.
    r.ptcr0().write(|w| w.0 = 0);
    r.ptcr1().write(|w| w.set_twm(true));
    r.ptcr0().write(|w| w.set_tcken(true));
    r.ptcr1().write(|w| {
        w.set_twm(true);
        w.set_tdi(reg_lsb);
    });
    r.ptcr0().write(|w| w.0 = 0);
    r.ptcr1().write(|w| w.0 = 0);

    // Phase C: latch the 8-bit data value.
    r.ptcr1().write(|w| w.set_tdi(val));
    r.ptcr0().write(|w| w.set_tcken(true));
    r.ptcr0().write(|w| w.0 = 0);
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
