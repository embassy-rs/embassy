//! DCMIPP (Digital Camera Interface Pixel Pipeline).
//!
//! Sits downstream of the CSI-2 Host (or the legacy parallel DVP input) and
//! writes pixel data into SRAM through its own AXI master. The peripheral
//! exposes three independent pixel pipes:
//!
//! - **Pipe0** — raw dump. No ISP, no colour conversion: bytes from the CSI
//!   packet payload land in memory in 32-bit words. Good for DMA-ing Bayer
//!   straight to CV/NN workloads, or for dumping ancillary data (embedded
//!   lines, CSI headers).
//! - **Pipe1** — the main pipe. ISP passthrough, optional Bayer demosaic,
//!   optional crop, optional downsize, and a pixel packer with full format
//!   support including multi-planar YUV.
//! - **Pipe2** — ancillary pipe, sharing Pipe1's ISP front end. Supports
//!   crop + downsize and coplanar output formats only (single memory plane);
//!   typical use is a low-resolution preview running alongside a Pipe1 full
//!   frame.
//!
//! The three pipes run concurrently. [`Dcmipp::new`] enables the peripheral
//! clock and NVIC line once; [`Dcmipp::split`] hands out the three pipe
//! handles, which are independent `Send`able objects with their own async
//! wakers. The single interrupt line is shared: on fire, the handler reads
//! the common status register and wakes whichever pipe's frame/overrun
//! flags are asserted.

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
        let r = T::regs();
        let sr = r.cmsr2().read();

        if sr.p0framef() || sr.p0ovrf() {
            r.p0ier().modify(|w| {
                w.set_frameie(false);
                w.set_ovrie(false);
            });
            PIPE0_STATE.waker.wake();
        }
        if sr.p1framef() || sr.p1ovrf() {
            r.p1ier().modify(|w| {
                w.set_frameie(false);
                w.set_ovrie(false);
            });
            PIPE1_STATE.waker.wake();
        }
        if sr.p2framef() || sr.p2ovrf() {
            r.p2ier().modify(|w| {
                w.set_frameie(false);
                w.set_ovrie(false);
            });
            PIPE2_STATE.waker.wake();
        }
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

static PIPE0_STATE: State = State::new();
static PIPE1_STATE: State = State::new();
static PIPE2_STATE: State = State::new();

/// DCMIPP error.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    /// Output FIFO overrun while streaming to memory.
    Overrun,
}

/// Pixel format on the output side of Pipe1 / Pipe2.
///
/// Value encoding matches the `PnPPCR.FORMAT` field in RM0486 §39.14.85. See
/// [`PixelFormat::is_coplanar`] for the subset accepted by Pipe2.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum PixelFormat {
    /// RGB888 (24 bpp, packed 4-byte aligned).
    Rgb888 = 0x0,
    /// RGB565 (16 bpp).
    Rgb565 = 0x1,
    /// ARGB8888 (alpha = 0xFF).
    Argb8888 = 0x2,
    /// RGBA8888 (alpha = 0xFF).
    Rgba8888 = 0x3,
    /// Monochrome 8-bit luminance.
    Y8 = 0x4,
    /// YUV444 interleaved (32 bpp, AYUV).
    Yuv444 = 0x5,
    /// YUV422 interleaved, YUYV byte order.
    Yuv422Yuyv = 0x6,
    /// YUV422 semi-planar (2-buffer).
    Yuv422SemiPlanar = 0x7,
    /// YUV420 semi-planar (2-buffer, NV21; set `swap_rb` for NV12).
    Yuv420SemiPlanar = 0x8,
    /// YUV420 planar (3-buffer, YV12).
    Yuv420Planar = 0x9,
    /// YUV422 interleaved, UYVY byte order.
    Yuv422Uyvy = 0xA,
}

impl PixelFormat {
    /// Whether this format fits in a single memory plane. Pipe2 requires it.
    pub const fn is_coplanar(self) -> bool {
        !matches!(
            self,
            PixelFormat::Yuv422SemiPlanar | PixelFormat::Yuv420SemiPlanar | PixelFormat::Yuv420Planar
        )
    }
}

/// Which interface feeds the pipeline with pixels.
///
/// This is a DCMIPP-wide setting (one mux for all three pipes). Whichever
/// pipe is configured last wins; in practice all three pipes share a single
/// physical sensor so their configs all name the same source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum InputSource {
    /// Legacy parallel DVP bus.
    Parallel,
    /// CSI-2 Host (MIPI).
    Csi,
}

/// Raw Bayer pattern of the sensor. Encoding matches `P1DMCR.TYPE`
/// (RM0486 §39.14.56).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum BayerPattern {
    /// Red‑Green‑Green‑Blue top-left 2×2 pattern.
    Rggb = 0x0,
    /// Green‑Red‑Blue‑Green top-left 2×2 pattern.
    Grbg = 0x1,
    /// Green‑Blue‑Red‑Green top-left 2×2 pattern.
    Gbrg = 0x2,
    /// Blue‑Green‑Green‑Red top-left 2×2 pattern.
    Bggr = 0x3,
}

/// Crop window. Coordinates and sizes are in pixels for Pipe1/Pipe2. On
/// Pipe0 (raw word-aligned dump) the same struct is reused but values are
/// interpreted as 32-bit words horizontally.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CropConfig {
    /// Top-left origin `(x, y)`.
    pub origin: (u16, u16),
    /// Window `(width, height)`.
    pub size: (u16, u16),
}

/// Downsize configuration for Pipe1 / Pipe2.
///
/// The resizer runs after crop. RM0486 §39.7.4 gives the math used here:
/// fixed-point downsize ratio + inverse-ratio divisor + final output
/// dimensions. Ratios are clamped to the hardware's 1×..8× range.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct DownsizeConfig {
    /// Pixel dimensions entering the resizer (typically the sensor's active
    /// frame size, or the crop window size if crop is enabled).
    pub input: (u16, u16),
    /// Pixel dimensions written to memory.
    pub output: (u16, u16),
}

/// Per-channel multiplicative gain applied through the Pipe1 colour-
/// conversion matrix as a diagonal. `1.0` is pass-through. The hardware
/// encodes each coefficient as 11-bit signed two's complement with a
/// scale of `256 = 1.0` (matches the ST ISP middleware's `To_CConv_Reg`).
/// The effective positive range is therefore ~`0.0..3.99`; values
/// outside saturate.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ChannelGains {
    /// Red gain.
    pub r: f32,
    /// Green gain.
    pub g: f32,
    /// Blue gain.
    pub b: f32,
}

impl ChannelGains {
    /// Identity (1.0, 1.0, 1.0).
    pub const IDENTITY: Self = Self { r: 1.0, g: 1.0, b: 1.0 };
}

/// 3×3 colour-conversion matrix with per-row offsets, programmed into
/// the DCMIPP Pipe1 ISP via [`Pipe1::set_color_matrix`].
///
/// Hardware applies `out = M · in + offset`, optionally clamped to
/// 8-bit unsigned. Coefficients are encoded as 11-bit signed Q2.8
/// (effective range `-4.0 ..= +3.996`, identity coefficient `1.0`);
/// offsets are 10-bit signed integers (range `-512 ..= 511`) added to
/// the matrix product before the optional clamp.
///
/// Typical use: write a colour-correction matrix derived from sensor
/// characterisation against a Macbeth chart, plus zero offsets, plus
/// `clamp = true` for 8-bit display output. For pass-through use
/// [`ColorMatrix::IDENTITY`].
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ColorMatrix {
    /// Row-major 3×3 coefficients. Element `[i][j]` multiplies input
    /// channel `j` to contribute to output channel `i`, with rows in
    /// R/G/B order.
    pub coeffs: [[f32; 3]; 3],
    /// Per-row offsets in 8-bit-output units, i.e. an offset of `+16.0`
    /// shifts the corresponding output channel up by 16/255.
    pub offsets: [f32; 3],
    /// Clamp the output to `[0, 255]`. Recommended `true` for 8-bit
    /// display / capture pipelines. Applies to the hardware's unsigned
    /// 8-bit output mode (`P1CCCR.TYPE = 0`); the signed-output mode is
    /// not exposed by this struct.
    pub clamp: bool,
}

impl ColorMatrix {
    /// Identity matrix with zero offsets and clamping enabled. Passing
    /// this to [`Pipe1::set_color_matrix`] should produce a frame
    /// pixel-identical to "no CCM applied" — a useful sanity check
    /// that the encoding into the hardware Q-format is correct.
    pub const IDENTITY: Self = Self {
        coeffs: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        offsets: [0.0, 0.0, 0.0],
        clamp: true,
    };
}

/// Round half away from zero. `f32::round` isn't in `core` and a libm
/// dependency for this single use isn't worth it.
fn round_half_away_from_zero(v: f32) -> i32 {
    if v >= 0.0 { (v + 0.5) as i32 } else { (v - 0.5) as i32 }
}

/// Encode a Q2.8-format coefficient (range `-4.0 ..= +3.996`) as an
/// 11-bit signed two's-complement field for the CCM coefficient registers.
fn q28_to_reg11(v: f32) -> u16 {
    let scaled = round_half_away_from_zero((v * 256.0).clamp(-1024.0, 1023.0));
    (scaled as u16) & 0x07FF
}

/// Encode a CCM offset (range `-512.0 ..= +511.0`, units of 8-bit output
/// channel) as a 10-bit signed two's-complement field.
fn offset_to_reg10(v: f32) -> u16 {
    let scaled = round_half_away_from_zero(v.clamp(-512.0, 511.0));
    (scaled as u16) & 0x03FF
}

/// Pipe0 (raw dump) configuration.
///
/// Pipe0 bypasses the ISP and writes CSI packet payload bytes to memory in
/// 32-bit words. Use this for Bayer dumping, embedded-line capture, or any
/// pass-through scenario where the ISP is not wanted.
#[non_exhaustive]
pub struct Pipe0Config {
    /// Upstream source (shared across pipes, see [`InputSource`]).
    pub source: InputSource,
    /// CSI-2 virtual channel (0..=3) to latch pixels from.
    pub virtual_channel: u8,
    /// Line pitch in bytes. Must be a multiple of 16.
    pub pitch_bytes: u16,
    /// Optional crop window (expressed in 32-bit words horizontally, pixels
    /// vertically — Pipe0 uses the word-granular stat/crop register).
    pub crop: Option<CropConfig>,
}

impl Pipe0Config {
    /// Create a Pipe0 configuration with VC0 and no crop.
    pub const fn new(source: InputSource, pitch_bytes: u16) -> Self {
        Self {
            source,
            virtual_channel: 0,
            pitch_bytes,
            crop: None,
        }
    }
}

/// Pipe1 (main) configuration.
#[non_exhaustive]
pub struct Pipe1Config {
    /// Upstream source.
    pub source: InputSource,
    /// CSI-2 virtual channel (0..=3) to latch pixels from.
    pub virtual_channel: u8,
    /// CSI-2 data type ID to filter the incoming stream (e.g. `0x2B` for
    /// RAW10, `0x24` for RGB888). Ignored when `source` is
    /// [`InputSource::Parallel`]. Defaults to `0x2B` (RAW10) — the most
    /// common Bayer-sensor format.
    pub csi_data_type: u8,
    /// Output pixel format as written to memory.
    pub output: PixelFormat,
    /// Line pitch in bytes. Must be a multiple of 16 per RM0486 §39.14.88.
    pub pitch_bytes: u16,
    /// Swap R/B components (or U/V for YUV). Useful e.g. to flip NV21 ↔ NV12.
    pub swap_rb: bool,
    /// Demosaic configuration. `None` bypasses the demosaic block (input must
    /// already be RGB/YUV). `Some(pattern)` enables it for the given Bayer
    /// layout — required when the sensor feeds 8/10/12/14-bit raw Bayer.
    pub demosaic: Option<BayerPattern>,
    /// Crop window (in pixels). Applied before downsize.
    pub crop: Option<CropConfig>,
    /// Downsize configuration. Ratios outside 1×..8× are clamped.
    pub downsize: Option<DownsizeConfig>,
}

impl Pipe1Config {
    /// Create a Pipe1 configuration with default R/B order, virtual channel 0,
    /// RAW10 CSI data type, no demosaic, no crop, and no downsize.
    /// `pitch_bytes` must be a multiple of 16 (DCMIPP output alignment).
    pub const fn new(source: InputSource, output: PixelFormat, pitch_bytes: u16) -> Self {
        Self {
            source,
            virtual_channel: 0,
            csi_data_type: 0x2B,
            output,
            pitch_bytes,
            swap_rb: false,
            demosaic: None,
            crop: None,
            downsize: None,
        }
    }
}

/// Pipe2 (ancillary) configuration.
///
/// Pipe2 shares Pipe1's ISP front end and only supports coplanar output
/// formats. Passing a multi-planar [`PixelFormat`] to [`Pipe2::configure`]
/// panics.
#[non_exhaustive]
pub struct Pipe2Config {
    /// Upstream source.
    pub source: InputSource,
    /// CSI-2 virtual channel (0..=3) to latch pixels from.
    pub virtual_channel: u8,
    /// Output pixel format. Must satisfy [`PixelFormat::is_coplanar`].
    pub output: PixelFormat,
    /// Line pitch in bytes. Must be a multiple of 16.
    pub pitch_bytes: u16,
    /// Swap R/B components (or U/V for YUV).
    pub swap_rb: bool,
    /// Crop window. Applied before downsize.
    pub crop: Option<CropConfig>,
    /// Downsize configuration.
    pub downsize: Option<DownsizeConfig>,
}

impl Pipe2Config {
    /// Create a Pipe2 configuration with VC0, no crop, no downsize.
    pub const fn new(source: InputSource, output: PixelFormat, pitch_bytes: u16) -> Self {
        Self {
            source,
            virtual_channel: 0,
            output,
            pitch_bytes,
            swap_rb: false,
            crop: None,
            downsize: None,
        }
    }
}

/// Given an input and output dimension, compute the fixed-point
/// `(ratio, div)` values required by the DCMIPP resizer for one axis.
/// Per RM0486 §39.7.4:
///   ratio = floor(8192 * input / output), clamped to \[8192, 65535\]
///   div   = floor((1024 * 8192 - 1) / ratio), clamped to \[128, 1023\]
fn downsize_coeffs(input: u16, output: u16) -> (u16, u16) {
    let ratio_num = (input as u32) * 8192;
    let ratio_den = output.max(1) as u32;
    let ratio = (ratio_num / ratio_den).clamp(8192, 65535) as u16;
    let div = ((1024 * 8192 - 1) / ratio as u32).clamp(128, 1023) as u16;
    (ratio, div)
}

/// DCMIPP driver — hands out three independent pipe handles via [`split`].
///
/// [`split`]: Self::split
pub struct Dcmipp<'d, T: Instance> {
    peri: Peri<'d, T>,
}

impl<'d, T: Instance> Dcmipp<'d, T> {
    /// Create a new DCMIPP driver. Enables the peripheral clock, performs a
    /// reset, and unmasks the NVIC line. Pipes start disabled; configure
    /// each one after [`split`].
    ///
    /// [`split`]: Self::split
    pub fn new(
        peri: Peri<'d, T>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
    ) -> Self {
        rcc::enable_and_reset::<T>();
        unsafe { T::Interrupt::enable() };
        Self { peri }
    }

    /// Split into three independent pipe handles. Each can be configured,
    /// started, awaited, and stopped independently; they share only the
    /// peripheral clock, the NVIC line, and the `CMCR.insel` input mux.
    pub fn split(self) -> (Pipe0<'d, T>, Pipe1<'d, T>, Pipe2<'d, T>) {
        (
            Pipe0 {
                _peri: self.peri,
                frame_count: 0,
                dbm_enabled: false,
            },
            Pipe1 {
                _marker: PhantomData,
                frame_count: 0,
                dbm_enabled: false,
            },
            Pipe2 {
                _marker: PhantomData,
                frame_count: 0,
                dbm_enabled: false,
            },
        )
    }
}

fn apply_input_source(source: InputSource, _swap_rb: bool) {
    // CMCR.swaprb is the *common* R/B swap that runs before each pipe.
    // The per-pipe `Pipe1Config::swap_rb` flag drives the output packer's
    // own swap (P1PPCR.swaprb). Setting both swaps cancels them out, so
    // the common one is left cleared and only the packer-side swap is
    // user-controlled.
    crate::pac::DCMIPP.cmcr().modify(|w| {
        w.set_insel(matches!(source, InputSource::Csi));
        w.set_swaprb(false);
    });
}

// ----------------------------------------------------------------------------
// Pipe0
// ----------------------------------------------------------------------------

/// Handle to DCMIPP Pipe0 (raw dump). Obtained from [`Dcmipp::split`].
pub struct Pipe0<'d, T: Instance> {
    _peri: Peri<'d, T>,
    frame_count: u32,
    dbm_enabled: bool,
}

impl<'d, T: Instance> Pipe0<'d, T> {
    /// Configure Pipe0: source, VC, pitch, optional crop. Leaves the pipe
    /// disabled.
    pub fn configure(&mut self, cfg: &Pipe0Config) {
        let r = T::regs();

        apply_input_source(cfg.source, false);

        r.p0fscr().modify(|w| {
            w.set_vc(cfg.virtual_channel & 0x3);
            w.set_pipen(false);
        });

        match cfg.crop {
            Some(c) => {
                r.p0scstr().write(|w| {
                    w.set_hstart(c.origin.0 & 0x0FFF);
                    w.set_vstart(c.origin.1 & 0x0FFF);
                });
                r.p0scszr().write(|w| {
                    w.set_hsize(c.size.0 & 0x0FFF);
                    w.set_vsize(c.size.1 & 0x0FFF);
                    w.set_enable(true);
                });
            }
            None => r.p0scszr().modify(|w| w.set_enable(false)),
        }

        // No DBM yet. Pitch is programmed here but the pitch register is on
        // the common pixel-packer block for Pipe0; see pac::DCMIPP.p0stm0ar
        // usage with pitch-like semantics? Actually RM0486 uses scszr for
        // size and the DMA handles pitch implicitly for Pipe0 word writes.
        // Drop double-buffer mode until a start_continuous call sets it.
        r.p0ppcr().modify(|w| w.set_dbm(false));

        self.dbm_enabled = false;
        let _ = cfg.pitch_bytes; // reserved for future packer changes
    }

    /// Arm a single-shot capture. `buffer` must be 16-byte aligned and at
    /// least the configured frame size.
    pub async fn capture(&mut self, buffer: *mut u8) -> Result<(), Error> {
        let r = T::regs();

        r.p0ppcr().modify(|w| w.set_dbm(false));
        r.p0ppm0ar1().write(|w| w.set_m0a(buffer as u32));

        r.p0fctcr().modify(|w| {
            w.set_cptmode(true);
            w.set_cptreq(true);
        });

        r.p0fcr().write(|w| {
            w.set_cframef(true);
            w.set_covrf(true);
        });
        r.p0ier().modify(|w| {
            w.set_frameie(true);
            w.set_ovrie(true);
        });
        r.p0fscr().modify(|w| w.set_pipen(true));

        let result = poll_fn(|cx| {
            PIPE0_STATE.waker.register(cx.waker());
            let sr = T::regs().p0sr().read();
            if sr.ovrf() {
                T::regs().p0fcr().write(|w| w.set_covrf(true));
                return Poll::Ready(Err(Error::Overrun));
            }
            if sr.framef() {
                T::regs().p0fcr().write(|w| w.set_cframef(true));
                return Poll::Ready(Ok(()));
            }
            Poll::Pending
        })
        .await;

        r.p0fscr().modify(|w| w.set_pipen(false));
        r.p0fctcr().modify(|w| w.set_cptreq(false));

        result
    }

    /// Start continuous double-buffered capture.
    ///
    /// # Safety
    ///
    /// Both buffers must remain valid and uniquely-owned until [`stop`] is
    /// called.
    ///
    /// [`stop`]: Self::stop
    pub unsafe fn start_continuous(&mut self, buf_a: *mut u8, buf_b: *mut u8) {
        let r = T::regs();
        r.p0ppm0ar1().write(|w| w.set_m0a(buf_a as u32));
        r.p0ppm0ar2().write(|w| w.set_m0a(buf_b as u32));
        r.p0ppcr().modify(|w| w.set_dbm(true));
        r.p0fctcr().modify(|w| {
            w.set_cptmode(false);
            w.set_cptreq(true);
        });
        r.p0fcr().write(|w| {
            w.set_cframef(true);
            w.set_covrf(true);
        });
        r.p0ier().modify(|w| {
            w.set_frameie(true);
            w.set_ovrie(true);
        });
        r.p0fscr().modify(|w| w.set_pipen(true));
        self.frame_count = 0;
        self.dbm_enabled = true;
    }

    /// Wait for the next frame. Returns the buffer index (0 or 1) that was
    /// just filled.
    pub async fn wait_frame(&mut self) -> Result<u8, Error> {
        let r = T::regs();
        r.p0fcr().write(|w| {
            w.set_cframef(true);
            w.set_covrf(true);
        });
        r.p0ier().modify(|w| {
            w.set_frameie(true);
            w.set_ovrie(true);
        });

        poll_fn(|cx| {
            PIPE0_STATE.waker.register(cx.waker());
            let sr = T::regs().p0sr().read();
            if sr.ovrf() {
                T::regs().p0fcr().write(|w| w.set_covrf(true));
                return Poll::Ready(Err(Error::Overrun));
            }
            if sr.framef() {
                T::regs().p0fcr().write(|w| w.set_cframef(true));
                let idx = if self.dbm_enabled {
                    (self.frame_count & 1) as u8
                } else {
                    0
                };
                self.frame_count = self.frame_count.wrapping_add(1);
                return Poll::Ready(Ok(idx));
            }
            Poll::Pending
        })
        .await
    }

    /// Stop whatever capture is running on Pipe0.
    pub fn stop(&mut self) {
        let r = T::regs();
        r.p0fctcr().modify(|w| w.set_cptreq(false));
        r.p0fscr().modify(|w| w.set_pipen(false));
        r.p0ier().modify(|w| {
            w.set_frameie(false);
            w.set_ovrie(false);
        });
        self.dbm_enabled = false;
    }
}

impl<'d, T: Instance> Drop for Pipe0<'d, T> {
    fn drop(&mut self) {
        self.stop();
    }
}

// ----------------------------------------------------------------------------
// Pipe1
// ----------------------------------------------------------------------------

/// Handle to DCMIPP Pipe1 (main ISP + format conversion). Obtained from
/// [`Dcmipp::split`].
pub struct Pipe1<'d, T: Instance> {
    _marker: PhantomData<&'d mut T>,
    frame_count: u32,
    dbm_enabled: bool,
}

impl<'d, T: Instance> Pipe1<'d, T> {
    /// Configure Pipe1: input, VC, output format, pitch, optional demosaic,
    /// optional crop, optional downsize. Leaves the pipe disabled.
    pub fn configure(&mut self, cfg: &Pipe1Config) {
        let r = T::regs();

        apply_input_source(cfg.source, cfg.swap_rb);

        // P1FSCR drives the CSI ingress filter for this pipe: pick the VC
        // and the data type to latch. DTMODE=0 = "match DTIDA only" (BSP
        // `DCMIPP_DTMODE_DTIDA`).
        r.p1fscr().modify(|w| {
            w.set_vc(cfg.virtual_channel & 0x3);
            w.set_dtmode(0);
            w.set_dtida(cfg.csi_data_type & 0x3F);
            w.set_pipen(false);
        });

        match cfg.demosaic {
            Some(pattern) => r.p1dmcr().modify(|w| {
                w.set_type_(pattern as u8);
                w.set_enable(true);
            }),
            None => r.p1dmcr().modify(|w| w.set_enable(false)),
        }

        match cfg.crop {
            Some(c) => {
                r.p1crstr().write(|w| {
                    w.set_hstart(c.origin.0 & 0x0FFF);
                    w.set_vstart(c.origin.1 & 0x0FFF);
                });
                r.p1crszr().write(|w| {
                    w.set_hsize(c.size.0 & 0x0FFF);
                    w.set_vsize(c.size.1 & 0x0FFF);
                    w.set_enable(true);
                });
            }
            None => r.p1crszr().modify(|w| w.set_enable(false)),
        }

        match cfg.downsize {
            Some(ds) => {
                let (hratio, hdiv) = downsize_coeffs(ds.input.0, ds.output.0);
                let (vratio, vdiv) = downsize_coeffs(ds.input.1, ds.output.1);
                r.p1dsrtior().modify(|w| {
                    w.set_hratio(hratio);
                    w.set_vratio(vratio);
                });
                r.p1dscr().modify(|w| {
                    w.set_hdiv(hdiv);
                    w.set_vdiv(vdiv);
                    w.set_enable(true);
                });
                r.p1dsszr().modify(|w| {
                    w.set_hsize(ds.output.0);
                    w.set_vsize(ds.output.1);
                });
            }
            None => r.p1dscr().modify(|w| w.set_enable(false)),
        }

        r.p1ppcr().modify(|w| {
            w.set_format(cfg.output as u8);
            w.set_swaprb(cfg.swap_rb);
            w.set_dbm(false);
        });
        r.p1ppm0pr().modify(|w| w.set_pitch(cfg.pitch_bytes));

        self.dbm_enabled = false;
    }

    /// Arm a single-shot capture. `buffer` must be 16-byte aligned.
    pub async fn capture(&mut self, buffer: *mut u8) -> Result<(), Error> {
        let r = T::regs();

        r.p1ppcr().modify(|w| w.set_dbm(false));
        r.p1ppm0ar1().write(|w| w.set_m0a(buffer as u32));
        r.p1fctcr().modify(|w| {
            w.set_cptmode(true);
            w.set_cptreq(true);
        });
        r.p1fcr().write(|w| {
            w.set_cframef(true);
            w.set_covrf(true);
        });
        r.p1ier().modify(|w| {
            w.set_frameie(true);
            w.set_ovrie(true);
        });
        r.p1fscr().modify(|w| w.set_pipen(true));

        let result = poll_fn(|cx| {
            PIPE1_STATE.waker.register(cx.waker());
            let sr = T::regs().p1sr().read();
            if sr.ovrf() {
                T::regs().p1fcr().write(|w| w.set_covrf(true));
                return Poll::Ready(Err(Error::Overrun));
            }
            if sr.framef() {
                T::regs().p1fcr().write(|w| w.set_cframef(true));
                return Poll::Ready(Ok(()));
            }
            Poll::Pending
        })
        .await;

        r.p1fscr().modify(|w| w.set_pipen(false));
        r.p1fctcr().modify(|w| w.set_cptreq(false));

        result
    }

    /// Start continuous double-buffered capture.
    ///
    /// # Safety
    ///
    /// Both buffers must remain valid and uniquely-owned until [`stop`] is
    /// called.
    ///
    /// [`stop`]: Self::stop
    pub unsafe fn start_continuous(&mut self, buf_a: *mut u8, buf_b: *mut u8) {
        let r = T::regs();
        r.p1ppm0ar1().write(|w| w.set_m0a(buf_a as u32));
        r.p1ppm0ar2().write(|w| w.set_m0a(buf_b as u32));
        r.p1ppcr().modify(|w| w.set_dbm(true));
        r.p1fctcr().modify(|w| {
            w.set_cptmode(false);
            w.set_cptreq(true);
        });
        r.p1fcr().write(|w| {
            w.set_cframef(true);
            w.set_covrf(true);
        });
        r.p1ier().modify(|w| {
            w.set_frameie(true);
            w.set_ovrie(true);
        });
        r.p1fscr().modify(|w| w.set_pipen(true));
        self.frame_count = 0;
        self.dbm_enabled = true;
    }

    /// Wait for the next frame. Returns the buffer index (0 or 1).
    pub async fn wait_frame(&mut self) -> Result<u8, Error> {
        let r = T::regs();
        r.p1fcr().write(|w| {
            w.set_cframef(true);
            w.set_covrf(true);
        });
        r.p1ier().modify(|w| {
            w.set_frameie(true);
            w.set_ovrie(true);
        });

        poll_fn(|cx| {
            PIPE1_STATE.waker.register(cx.waker());
            let sr = T::regs().p1sr().read();
            if sr.ovrf() {
                T::regs().p1fcr().write(|w| w.set_covrf(true));
                return Poll::Ready(Err(Error::Overrun));
            }
            if sr.framef() {
                T::regs().p1fcr().write(|w| w.set_cframef(true));
                let idx = if self.dbm_enabled {
                    (self.frame_count & 1) as u8
                } else {
                    0
                };
                self.frame_count = self.frame_count.wrapping_add(1);
                return Poll::Ready(Ok(idx));
            }
            Poll::Pending
        })
        .await
    }

    /// Stop whatever capture is running on Pipe1.
    pub fn stop(&mut self) {
        let r = T::regs();
        r.p1fctcr().modify(|w| w.set_cptreq(false));
        r.p1fscr().modify(|w| w.set_pipen(false));
        r.p1ier().modify(|w| {
            w.set_frameie(false);
            w.set_ovrie(false);
        });
        self.dbm_enabled = false;
    }

    /// Program the colour-conversion matrix as a diagonal gain matrix and
    /// enable it. Off-diagonal coefficients and the offset column are
    /// zeroed. Re-callable while streaming — the new matrix latches at
    /// the next frame boundary.
    ///
    /// This shares hardware with [`set_color_matrix`]; calling either
    /// after the other replaces the previous CCM. Use [`set_color_matrix`]
    /// for full 3×3 colour correction (and to keep WB+CCM separate by
    /// applying WB at the sensor / via the demosaic stage instead).
    ///
    /// [`set_color_matrix`]: Self::set_color_matrix
    pub fn set_color_gains(&mut self, gains: ChannelGains) {
        let r = T::regs();
        // White-balance gains are non-negative; saturate before reusing the
        // signed CCM encoder. Diagonal-only: rr→P1CCRR1, gg→P1CCGR1,
        // bb→P1CCBR2; the other registers are zeroed via `write`.
        let rr = q28_to_reg11(gains.r.max(0.0));
        let gg = q28_to_reg11(gains.g.max(0.0));
        let bb = q28_to_reg11(gains.b.max(0.0));
        r.p1ccrr1().write(|w| w.set_rr(rr));
        r.p1ccrr2().write(|_| {});
        r.p1ccgr1().write(|w| w.set_gg(gg));
        r.p1ccgr2().write(|_| {});
        r.p1ccbr1().write(|_| {});
        r.p1ccbr2().write(|w| w.set_bb(bb));
        r.p1cccr().write(|w| w.set_enable(true));
    }

    /// Program the full 3×3 colour-conversion matrix and enable it.
    ///
    /// Hardware applies `out = M · in + offset`, optionally clamped to
    /// 8-bit unsigned. Coefficients are encoded in 11-bit signed Q2.8
    /// (range `-4.0 ..= +3.996`); offsets are 10-bit signed integers
    /// (range `-512 ..= 511`) added to the matrix product.
    ///
    /// Re-callable while streaming — the new matrix latches at the next
    /// frame boundary. Shares hardware with [`set_color_gains`]; the
    /// last call wins.
    ///
    /// [`set_color_gains`]: Self::set_color_gains
    pub fn set_color_matrix(&mut self, m: ColorMatrix) {
        let r = T::regs();
        let c = m.coeffs.map(|row| row.map(q28_to_reg11));
        let o = m.offsets.map(offset_to_reg10);
        r.p1ccrr1().write(|w| {
            w.set_rr(c[0][0]);
            w.set_rg(c[0][1]);
        });
        r.p1ccrr2().write(|w| {
            w.set_rb(c[0][2]);
            w.set_ra(o[0]);
        });
        r.p1ccgr1().write(|w| {
            w.set_gr(c[1][0]);
            w.set_gg(c[1][1]);
        });
        r.p1ccgr2().write(|w| {
            w.set_gb(c[1][2]);
            w.set_ga(o[1]);
        });
        r.p1ccbr1().write(|w| {
            w.set_br(c[2][0]);
            w.set_bg(c[2][1]);
        });
        r.p1ccbr2().write(|w| {
            w.set_bb(c[2][2]);
            w.set_ba(o[2]);
        });
        r.p1cccr().write(|w| {
            w.set_enable(true);
            w.set_clamp(m.clamp);
            // type_ = false → unsigned 8-bit clip [0, 255] when CLAMP is on.
            w.set_type_(false);
        });
    }

    /// Disable the colour-conversion matrix entirely (skip the stage in
    /// the ISP pipeline). After this call, neither [`set_color_gains`]
    /// nor [`set_color_matrix`] state is applied until called again.
    ///
    /// [`set_color_gains`]: Self::set_color_gains
    /// [`set_color_matrix`]: Self::set_color_matrix
    pub fn disable_color_matrix(&mut self) {
        T::regs().p1cccr().write(|_| {});
    }

    /// Enable / disable Pipe1's gamma stage (a fixed sRGB-2.2 curve in
    /// hardware — there is no programmable LUT). Re-callable while
    /// streaming.
    pub fn set_gamma_enable(&mut self, enable: bool) {
        T::regs().p1gmcr().write(|w| w.set_enable(enable));
    }

    /// Configure the three statistics extractors to compute the per-frame
    /// mean of the post-demosaic R, G, B channels over the full Pipe1
    /// output rectangle. Call once before streaming starts; the hardware
    /// then refreshes the result registers at every end-of-frame.
    pub fn enable_rgb_stats(&mut self, width: u16, height: u16) {
        let r = T::regs();
        r.p1ststr().write(|w| {
            w.set_hstart(0);
            w.set_vstart(0);
        });
        r.p1stszr().write(|w| {
            w.set_hsize(width & 0x0FFF);
            w.set_vsize(height & 0x0FFF);
            w.set_cropen(true);
        });
        // src: 4 = post-demosaic R, 5 = G, 6 = B (BSP `IS_DCMIPP_STAT_*`).
        // mode = 0 (mean), bins = 0 (don't care for mean).
        r.p1st1cr().write(|w| {
            w.set_src(4);
            w.set_enable(true);
        });
        r.p1st2cr().write(|w| {
            w.set_src(5);
            w.set_enable(true);
        });
        r.p1st3cr().write(|w| {
            w.set_src(6);
            w.set_enable(true);
        });
    }

    /// Read the most recently latched per-channel means (`P1ST{1,2,3}SR.accu`).
    /// Returns `(mean_r, mean_g, mean_b)`. Each value is the channel sum
    /// over the configured stats window divided by 256, per the metapac
    /// register description. Call only after `enable_rgb_stats` and at
    /// least one captured frame.
    pub fn read_rgb_means(&self) -> (u32, u32, u32) {
        let r = T::regs();
        (
            r.p1st1sr().read().accu(),
            r.p1st2sr().read().accu(),
            r.p1st3sr().read().accu(),
        )
    }
}

impl<'d, T: Instance> Drop for Pipe1<'d, T> {
    fn drop(&mut self) {
        self.stop();
    }
}

// ----------------------------------------------------------------------------
// Pipe2
// ----------------------------------------------------------------------------

/// Handle to DCMIPP Pipe2 (ancillary, coplanar-only). Obtained from
/// [`Dcmipp::split`].
pub struct Pipe2<'d, T: Instance> {
    _marker: PhantomData<&'d mut T>,
    frame_count: u32,
    dbm_enabled: bool,
}

impl<'d, T: Instance> Pipe2<'d, T> {
    /// Configure Pipe2. Panics if `cfg.output` is multi-planar — Pipe2 only
    /// supports single-plane pixel formats.
    pub fn configure(&mut self, cfg: &Pipe2Config) {
        assert!(cfg.output.is_coplanar(), "Pipe2 supports only coplanar pixel formats");

        let r = T::regs();

        apply_input_source(cfg.source, cfg.swap_rb);

        r.p2fscr().modify(|w| {
            w.set_vc(cfg.virtual_channel & 0x3);
            w.set_pipen(false);
        });

        match cfg.crop {
            Some(c) => {
                r.p2crstr().write(|w| {
                    w.set_hstart(c.origin.0 & 0x0FFF);
                    w.set_vstart(c.origin.1 & 0x0FFF);
                });
                r.p2crszr().write(|w| {
                    w.set_hsize(c.size.0 & 0x0FFF);
                    w.set_vsize(c.size.1 & 0x0FFF);
                    w.set_enable(true);
                });
            }
            None => r.p2crszr().modify(|w| w.set_enable(false)),
        }

        match cfg.downsize {
            Some(ds) => {
                let (hratio, hdiv) = downsize_coeffs(ds.input.0, ds.output.0);
                let (vratio, vdiv) = downsize_coeffs(ds.input.1, ds.output.1);
                r.p2dsrtior().modify(|w| {
                    w.set_hratio(hratio);
                    w.set_vratio(vratio);
                });
                r.p2dscr().modify(|w| {
                    w.set_hdiv(hdiv);
                    w.set_vdiv(vdiv);
                    w.set_enable(true);
                });
                r.p2dsszr().modify(|w| {
                    w.set_hsize(ds.output.0);
                    w.set_vsize(ds.output.1);
                });
            }
            None => r.p2dscr().modify(|w| w.set_enable(false)),
        }

        r.p2ppcr().modify(|w| {
            w.set_format(cfg.output as u8);
            w.set_swaprb(cfg.swap_rb);
            w.set_dbm(false);
        });
        r.p2ppm0pr().modify(|w| w.set_pitch(cfg.pitch_bytes));

        self.dbm_enabled = false;
    }

    /// Arm a single-shot capture. `buffer` must be 16-byte aligned.
    pub async fn capture(&mut self, buffer: *mut u8) -> Result<(), Error> {
        let r = T::regs();

        r.p2ppcr().modify(|w| w.set_dbm(false));
        r.p2ppm0ar1().write(|w| w.set_m0a(buffer as u32));
        r.p2fctcr().modify(|w| {
            w.set_cptmode(true);
            w.set_cptreq(true);
        });
        r.p2fcr().write(|w| {
            w.set_cframef(true);
            w.set_covrf(true);
        });
        r.p2ier().modify(|w| {
            w.set_frameie(true);
            w.set_ovrie(true);
        });
        r.p2fscr().modify(|w| w.set_pipen(true));

        let result = poll_fn(|cx| {
            PIPE2_STATE.waker.register(cx.waker());
            let sr = T::regs().p2sr().read();
            if sr.ovrf() {
                T::regs().p2fcr().write(|w| w.set_covrf(true));
                return Poll::Ready(Err(Error::Overrun));
            }
            if sr.framef() {
                T::regs().p2fcr().write(|w| w.set_cframef(true));
                return Poll::Ready(Ok(()));
            }
            Poll::Pending
        })
        .await;

        r.p2fscr().modify(|w| w.set_pipen(false));
        r.p2fctcr().modify(|w| w.set_cptreq(false));

        result
    }

    /// Start continuous double-buffered capture.
    ///
    /// # Safety
    ///
    /// Both buffers must remain valid and uniquely-owned until [`stop`] is
    /// called.
    ///
    /// [`stop`]: Self::stop
    pub unsafe fn start_continuous(&mut self, buf_a: *mut u8, buf_b: *mut u8) {
        let r = T::regs();
        r.p2ppm0ar1().write(|w| w.set_m0a(buf_a as u32));
        r.p2ppm0ar2().write(|w| w.set_m0a(buf_b as u32));
        r.p2ppcr().modify(|w| w.set_dbm(true));
        r.p2fctcr().modify(|w| {
            w.set_cptmode(false);
            w.set_cptreq(true);
        });
        r.p2fcr().write(|w| {
            w.set_cframef(true);
            w.set_covrf(true);
        });
        r.p2ier().modify(|w| {
            w.set_frameie(true);
            w.set_ovrie(true);
        });
        r.p2fscr().modify(|w| w.set_pipen(true));
        self.frame_count = 0;
        self.dbm_enabled = true;
    }

    /// Wait for the next frame. Returns the buffer index (0 or 1).
    pub async fn wait_frame(&mut self) -> Result<u8, Error> {
        let r = T::regs();
        r.p2fcr().write(|w| {
            w.set_cframef(true);
            w.set_covrf(true);
        });
        r.p2ier().modify(|w| {
            w.set_frameie(true);
            w.set_ovrie(true);
        });

        poll_fn(|cx| {
            PIPE2_STATE.waker.register(cx.waker());
            let sr = T::regs().p2sr().read();
            if sr.ovrf() {
                T::regs().p2fcr().write(|w| w.set_covrf(true));
                return Poll::Ready(Err(Error::Overrun));
            }
            if sr.framef() {
                T::regs().p2fcr().write(|w| w.set_cframef(true));
                let idx = if self.dbm_enabled {
                    (self.frame_count & 1) as u8
                } else {
                    0
                };
                self.frame_count = self.frame_count.wrapping_add(1);
                return Poll::Ready(Ok(idx));
            }
            Poll::Pending
        })
        .await
    }

    /// Stop whatever capture is running on Pipe2.
    pub fn stop(&mut self) {
        let r = T::regs();
        r.p2fctcr().modify(|w| w.set_cptreq(false));
        r.p2fscr().modify(|w| w.set_pipen(false));
        r.p2ier().modify(|w| {
            w.set_frameie(false);
            w.set_ovrie(false);
        });
        self.dbm_enabled = false;
    }
}

impl<'d, T: Instance> Drop for Pipe2<'d, T> {
    fn drop(&mut self) {
        self.stop();
    }
}

// ----------------------------------------------------------------------------
// Instance trait
// ----------------------------------------------------------------------------

trait SealedInstance: crate::rcc::RccPeripheral {
    fn regs() -> crate::pac::dcmipp::Dcmipp;
}

/// DCMIPP instance.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + 'static + Send {
    /// Interrupt for this instance.
    type Interrupt: interrupt::typelevel::Interrupt;
}

foreach_interrupt! {
    ($inst:ident, dcmipp, DCMIPP, GLOBAL, $irq:ident) => {
        impl SealedInstance for crate::peripherals::$inst {
            fn regs() -> crate::pac::dcmipp::Dcmipp {
                crate::pac::$inst
            }
        }
        impl Instance for crate::peripherals::$inst {
            type Interrupt = crate::interrupt::typelevel::$irq;
        }
    };
}
