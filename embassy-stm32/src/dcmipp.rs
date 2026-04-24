//! DCMIPP (Digital Camera Interface Pixel Pipeline).
//!
//! Sits downstream of the CSI-2 Host (or the legacy parallel DVP input) and
//! writes pixel data into SRAM through its own AXI master. This driver wraps
//! **Pipe1** — the main pipe with ISP passthrough, pixel-format conversion,
//! and a pixel packer writing ping-pong buffers in memory. Pipe0 (raw dump)
//! and Pipe2 (ancillary coplanar) are reachable through `pac::DCMIPP` if
//! needed but aren't wrapped here.

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
        // Mask Pipe1's end-of-frame / overrun interrupts so the handler stops
        // re-firing. The awaiting task clears the flags and re-arms.
        T::regs().p1ier().modify(|w| {
            w.set_frameie(false);
            w.set_ovrie(false);
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

/// DCMIPP error.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    /// Output FIFO overrun while streaming to memory.
    Overrun,
}

/// Pixel format on the output side of Pipe1.
///
/// Value encoding matches the `P1PPCR.FORMAT` field in RM0486 §39.14.85.
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

/// Which interface feeds the pipeline with pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum InputSource {
    /// Legacy parallel DVP bus.
    Parallel,
    /// CSI-2 Host (MIPI).
    Csi,
}

/// Pipe1 configuration.
#[non_exhaustive]
pub struct Pipe1Config {
    /// Upstream source.
    pub source: InputSource,
    /// CSI-2 virtual channel (0..=3) to latch pixels from. Ignored when
    /// [`InputSource::Parallel`] is selected.
    pub virtual_channel: u8,
    /// Output pixel format as written to memory.
    pub output: PixelFormat,
    /// Line pitch in bytes between consecutive output rows. Must be a
    /// multiple of 16 per RM0486 §39.14.88.
    pub pitch_bytes: u16,
    /// Swap R/B components (or U/V for YUV). Useful e.g. to flip NV21 ↔ NV12.
    pub swap_rb: bool,
}

impl Pipe1Config {
    /// Create a Pipe1 configuration with default R/B order and virtual channel 0.
    /// `pitch_bytes` must be a multiple of 16 (DCMIPP output alignment).
    pub const fn new(source: InputSource, output: PixelFormat, pitch_bytes: u16) -> Self {
        Self {
            source,
            virtual_channel: 0,
            output,
            pitch_bytes,
            swap_rb: false,
        }
    }
}

/// DCMIPP driver.
pub struct Dcmipp<'d, T: Instance> {
    _peri: Peri<'d, T>,
    /// Frame count since `start_continuous` was called. `wait_frame` returns
    /// `count & 1`, i.e. which of the two ping-pong buffers just filled.
    frame_count: u32,
    dbm_enabled: bool,
}

impl<'d, T: Instance> Dcmipp<'d, T> {
    /// Create a new DCMIPP driver.
    ///
    /// Enables the peripheral clock + performs a reset. The driver wakes up
    /// with no pipe running; call [`configure_pipe1`] and then [`capture`]
    /// or [`start_continuous`] to begin capture.
    ///
    /// [`configure_pipe1`]: Self::configure_pipe1
    /// [`capture`]: Self::capture
    /// [`start_continuous`]: Self::start_continuous
    pub fn new(
        peri: Peri<'d, T>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
    ) -> Self {
        rcc::enable_and_reset::<T>();

        unsafe { T::Interrupt::enable() };

        Self {
            _peri: peri,
            frame_count: 0,
            dbm_enabled: false,
        }
    }

    /// Configure Pipe1: input selection, virtual channel, output format, and
    /// line pitch. Leaves the pipe disabled — call a capture method to
    /// actually begin streaming.
    pub fn configure_pipe1(&mut self, cfg: &Pipe1Config) {
        let r = T::regs();

        // Input selection and R/B swap on the pixel pipe.
        r.cmcr().modify(|w| {
            w.set_insel(matches!(cfg.source, InputSource::Csi));
            w.set_swaprb(cfg.swap_rb);
        });

        // Flow selection: virtual channel + pipe disabled until a capture
        // method arms it.
        r.p1fscr().modify(|w| {
            w.set_vc(cfg.virtual_channel & 0x3);
            w.set_pipen(false);
        });

        // Output pixel packer: format and optional R/B swap at the output.
        r.p1ppcr().modify(|w| {
            w.set_format(cfg.output as u8);
            w.set_swaprb(cfg.swap_rb);
            w.set_dbm(false);
        });

        // Set the output line pitch for memory 0.
        r.p1ppm0pr().modify(|w| w.set_pitch(cfg.pitch_bytes));

        self.dbm_enabled = false;
    }

    /// Arm a single-shot capture. Writes one frame to `buffer` and resolves.
    ///
    /// `buffer` must be at least `pitch_bytes * height` long and aligned to
    /// 16 bytes (DCMIPP hardware constraint). The exact size depends on the
    /// configured pixel format and the sensor's active frame size; the driver
    /// does not validate it.
    pub async fn capture(&mut self, buffer: *mut u8) -> Result<(), Error> {
        let r = T::regs();

        r.p1ppcr().modify(|w| w.set_dbm(false));
        r.p1ppm0ar1().write(|w| w.set_m0a(buffer as u32));

        // Snapshot capture: CPTMODE = 1 (snapshot), CPTREQ = 1 (go).
        r.p1fctcr().modify(|w| {
            w.set_cptmode(true);
            w.set_cptreq(true);
        });

        // Arm end-of-frame / overrun interrupts.
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
            STATE.waker.register(cx.waker());

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

        // Snapshot mode stops on its own; drop the pipen bit + clear CPTREQ.
        r.p1fscr().modify(|w| w.set_pipen(false));
        r.p1fctcr().modify(|w| w.set_cptreq(false));

        result
    }

    /// Start continuous double-buffered capture. The pipeline fills
    /// `buf_a` and `buf_b` alternately; use [`wait_frame`] to synchronize
    /// with frame completions.
    ///
    /// Both buffers must be pre-configured with the same pitch/size and both
    /// 16-byte aligned.
    ///
    /// # Safety
    ///
    /// The caller must ensure `buf_a` and `buf_b` remain valid and
    /// uniquely-owned for the lifetime of the capture stream (i.e. until
    /// [`stop`] is called). DCMIPP writes to these addresses asynchronously
    /// without any borrow-checker mediation.
    ///
    /// [`wait_frame`]: Self::wait_frame
    /// [`stop`]: Self::stop
    pub unsafe fn start_continuous(&mut self, buf_a: *mut u8, buf_b: *mut u8) {
        let r = T::regs();

        r.p1ppm0ar1().write(|w| w.set_m0a(buf_a as u32));
        r.p1ppm0ar2().write(|w| w.set_m0a(buf_b as u32));

        r.p1ppcr().modify(|w| w.set_dbm(true));
        r.p1fctcr().modify(|w| {
            w.set_cptmode(false); // continuous
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

    /// Wait for the next frame completion while in continuous mode.
    ///
    /// Returns the buffer index (0 for `buf_a`, 1 for `buf_b`) that was just
    /// filled by DCMIPP and is now safe to read. Overrun surfaces as
    /// `Err(Error::Overrun)`; continuous mode keeps running — the caller may
    /// choose to `stop()` or keep draining frames.
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
            STATE.waker.register(cx.waker());
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

    /// Stop whatever capture is running. Safe to call even when idle.
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
}

impl<'d, T: Instance> Drop for Dcmipp<'d, T> {
    fn drop(&mut self) {
        self.stop();
        T::Interrupt::disable();
    }
}

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
