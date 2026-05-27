//! JPEG hardware codec.
//!
//! Async encoder + decoder for the STM32 hardware JPEG peripheral (jpeg_v1).
//!
//! Encode emits the JFIF/baseline JPEG header in software (using the standard
//! Annex K quantization and Huffman tables), then runs the codec for the
//! entropy-coded segment. Decode uses hardware header parsing (`HDR=1`) so any
//! standard baseline JPEG with a JFIF header is handled.
//!
//! Supported color spaces: `Grayscale` and `YCbCr` (4:4:4 / 4:2:2 / 4:2:0).
//! RGB and CMYK source are not yet supported. Decode of progressive or 12-bit
//! JPEGs returns [`Error::Unsupported`].
//!
//! Encode runs end-to-end via DMA on both DMA channels passed to [`Jpeg::new`]:
//! input is fed via `write_raw` while output is drained via `read_raw`,
//! awaiting the codec's end-of-conversion (`EOC`) interrupt to release the
//! task. After EOC the driver tail-drains any words still in the codec's
//! output FIFO via CPU before returning. Decode is CPU-driven for now (the
//! per-MCU plane scatter is naive — see the limitation below).
//!
//! For setups where DMA channels are scarce, [`Jpeg::new_blocking`] constructs
//! the driver without DMA channels (and without the JPEG interrupt). Pair it
//! with [`Jpeg::encode_blocking`] / [`Jpeg::decode_blocking`], which busy-wait
//! on the codec FIFO flags instead of `await`ing on `EOC`.
//!
//! v1 limitations:
//! - The decode plane scatter is naive: bytes are written sequentially into
//!   Y, then Cb, then Cr without per-MCU geometric remapping. Round-trip
//!   dimensions and grayscale are correct; YCbCr planes are MCU-ordered.

use core::marker::PhantomData;

use embassy_hal_internal::{Peri, PeripheralType};
use embassy_sync::waitqueue::AtomicWaker;

use crate::dma::{ChannelAndRequest, TransferOptions};
use crate::interrupt::typelevel::Interrupt;
use crate::mode::{Async, Blocking, Mode};
use crate::{interrupt, pac, peripherals, rcc};

const DC_HUFF_TABLE_SIZE: usize = 12;
const AC_HUFF_TABLE_SIZE: usize = 162;

// Register offsets within the JPEG peripheral block. We access the HUFFENC and
// QMEM banks via raw pointer arithmetic because the metapac exposes one method
// per register and the indexing pattern (4 banks × N registers each) is much
// cleaner with offsets.
const QMEM0_OFFSET: usize = 0x50;
const QMEM_BANK_STRIDE: usize = 0x40;
const HUFFENC_AC0_OFFSET: usize = 0x500;
const HUFFENC_AC_BANK_STRIDE: usize = 0x160;
const HUFFENC_DC0_OFFSET: usize = 0x7C0;
const HUFFENC_DC_BANK_STRIDE: usize = 0x20;

static JPEG_WAKER: AtomicWaker = AtomicWaker::new();

/// JPEG codec interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let regs = T::regs();
        let sr = regs.jpeg_sr().read();
        // Disable any of the events that fired so we re-arm only after the task
        // observes the flag. The flag itself stays asserted until the task
        // clears it via CFR.
        if sr.eocf() || sr.hpdf() {
            regs.jpeg_cr().modify(|w| {
                if sr.eocf() {
                    w.set_eocie(false);
                }
                if sr.hpdf() {
                    w.set_hpdie(false);
                }
            });
            JPEG_WAKER.wake();
        }
    }
}

/// Color space.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ColorSpace {
    /// Single-component luminance.
    Grayscale,
    /// Three-component YCbCr (Y, Cb, Cr planes).
    YCbCr,
}

/// Chroma subsampling for YCbCr color space. Ignored for grayscale.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ChromaSubsampling {
    /// 4:4:4 — no subsampling, MCU is 8×8.
    S444,
    /// 4:2:2 — chroma horizontally subsampled by 2, MCU is 16×8.
    S422,
    /// 4:2:0 — chroma horizontally and vertically subsampled by 2, MCU is 16×16.
    S420,
}

/// Encode parameters.
pub struct EncodeConfig {
    /// Image width in pixels. Must be a multiple of the MCU width.
    pub width: u16,
    /// Image height in pixels. Must be a multiple of the MCU height.
    pub height: u16,
    /// Color space.
    pub color_space: ColorSpace,
    /// Chroma subsampling. Ignored for `Grayscale`.
    pub subsampling: ChromaSubsampling,
    /// Quality 1..=100. 50 corresponds to the unmodified Annex K tables.
    pub quality: u8,
}

/// Decode result describing the decoded image.
#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct DecodeInfo {
    /// Image width in pixels.
    pub width: u16,
    /// Image height in pixels.
    pub height: u16,
    /// Color space found in the JPEG header.
    pub color_space: ColorSpace,
    /// Chroma subsampling found in the JPEG header.
    pub subsampling: ChromaSubsampling,
    /// Bytes written to the Y plane.
    pub y_bytes: usize,
    /// Bytes written to the Cb plane (0 for grayscale).
    pub cb_bytes: usize,
    /// Bytes written to the Cr plane (0 for grayscale).
    pub cr_bytes: usize,
}

/// Planar YCbCr input for [`Jpeg::encode_planar`].
pub struct PlanarYCbCr<'a> {
    /// Luma plane, `width × height` bytes.
    pub y: &'a [u8],
    /// Cb plane, sized per the chroma subsampling.
    pub cb: &'a [u8],
    /// Cr plane, sized per the chroma subsampling.
    pub cr: &'a [u8],
}

/// Planar YCbCr output for [`Jpeg::decode`].
pub struct PlanarYCbCrMut<'a> {
    /// Luma plane.
    pub y: &'a mut [u8],
    /// Cb plane (may be empty for grayscale).
    pub cb: &'a mut [u8],
    /// Cr plane (may be empty for grayscale).
    pub cr: &'a mut [u8],
}

/// JPEG driver errors.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// Output buffer is smaller than the produced JPEG / decoded image.
    OutputTooSmall,
    /// Input ended before the codec consumed enough data to finish.
    InputTruncated,
    /// JPEG features that aren't supported (progressive, 12-bit, RGB/CMYK source, etc).
    Unsupported,
    /// Encode parameters are invalid (zero dimensions, out-of-range quality, dimensions
    /// not a multiple of the MCU size, mismatched buffer sizes, etc).
    InvalidConfig,
}

/// JPEG codec driver.
pub struct Jpeg<'d, T: Instance, M: Mode> {
    _peripheral: Peri<'d, T>,
    indma: Option<ChannelAndRequest<'d>>,
    outdma: Option<ChannelAndRequest<'d>>,
    _phantom: PhantomData<M>,
}

impl<'d, T: Instance> Jpeg<'d, T, Async> {
    /// Create a new JPEG driver with DMA-driven async encode/decode.
    pub fn new<DIn: DmaIn<T>, DOut: DmaOut<T>>(
        peri: Peri<'d, T>,
        indma: Peri<'d, DIn>,
        outdma: Peri<'d, DOut>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>>
        + interrupt::typelevel::Binding<DIn::Interrupt, crate::dma::InterruptHandler<DIn>>
        + interrupt::typelevel::Binding<DOut::Interrupt, crate::dma::InterruptHandler<DOut>>
        + 'd,
    ) -> Self {
        Self::init_peripheral();

        let indma = new_dma_nonopt!(indma, _irq);
        let outdma = new_dma_nonopt!(outdma, _irq);

        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        Self {
            _peripheral: peri,
            indma: Some(indma),
            outdma: Some(outdma),
            _phantom: PhantomData,
        }
    }
}

impl<'d, T: Instance> Jpeg<'d, T, Blocking> {
    /// Create a new JPEG driver without DMA channels for blocking-only use.
    ///
    /// Pair with [`encode_blocking`](Self::encode_blocking) /
    /// [`decode_blocking`](Self::decode_blocking). Saves two DMA channels and
    /// the JPEG interrupt slot at the cost of busy-waiting on the FIFOs.
    pub fn new_blocking(peri: Peri<'d, T>) -> Self {
        Self::init_peripheral();
        Self {
            _peripheral: peri,
            indma: None,
            outdma: None,
            _phantom: PhantomData,
        }
    }
}

impl<'d, T: Instance, M: Mode> Jpeg<'d, T, M> {
    fn init_peripheral() {
        debug!("jpeg init: rcc enable_and_reset");
        rcc::enable_and_reset::<T>();

        // Per ST HAL: turn on JCEN, stop any in-flight conversion, mask all
        // IRQs/DMA, flush both FIFOs, and clear all flags before touching the
        // table memory. The HUFFENC/QMEM RAMs are only writable while JCEN=1.
        debug!("jpeg init: register setup");
        let regs = T::regs();
        regs.jpeg_cr().write(|w| w.set_jcen(true));
        regs.jpeg_confr0().write(|w| w.set_start(false));
        regs.jpeg_cr().modify(|w| {
            w.set_iftie(false);
            w.set_ifnfie(false);
            w.set_oftie(false);
            w.set_ofneie(false);
            w.set_eocie(false);
            w.set_hpdie(false);
            w.set_idmaen(false);
            w.set_odmaen(false);
            w.set_iff(true);
            w.set_off(true);
        });
        regs.jpeg_cfr().write(|w| {
            w.set_ceocf(true);
            w.set_chpdf(true);
        });

        // Load the standard Annex K Huffman encoding tables once. They are
        // constant across all encodes, so we don't reload per call.
        debug!("jpeg init: load_huffenc");
        load_huffenc::<T>();
        debug!("jpeg init: done");
    }
}

impl<'d, T: Instance> Jpeg<'d, T, Async> {
    /// Encode a raw, MCU-ordered pixel buffer into a JPEG (with full JFIF header) in `dst`.
    ///
    /// `src` layout depends on `cfg`:
    /// - `Grayscale`: tightly packed Y bytes, row-major, exactly `width*height` bytes.
    /// - `YCbCr`: per-MCU, all 8×8 luma blocks in raster order followed by the
    ///   Cb 8×8 block then the Cr 8×8 block. Total size:
    ///   - 4:2:0 → `width*height*3/2`
    ///   - 4:2:2 → `width*height*2`
    ///   - 4:4:4 → `width*height*3`
    ///
    /// Returns the number of bytes written to `dst`.
    pub async fn encode(&mut self, src: &[u8], cfg: &EncodeConfig, dst: &mut [u8]) -> Result<usize, Error> {
        validate_encode_config(cfg)?;

        let expected_input = mcu_ordered_input_size(cfg);
        if src.len() != expected_input {
            return Err(Error::InvalidConfig);
        }

        // Emit JFIF header into the front of dst.
        let header_len = emit_jfif_header(cfg, dst)?;

        // EOI is two bytes; the entropy data goes between header and EOI.
        if dst.len() < header_len + 2 {
            return Err(Error::OutputTooSmall);
        }
        let payload_room = dst.len() - header_len - 2;

        // Configure peripheral for encode.
        configure_encode::<T>(cfg);

        // Match the ST HAL `JPEG_Init_Process` sequence: stop the codec, mask
        // all interrupts and DMA, flush both FIFOs, clear flags. Then START
        // the codec, then arm DMA + IRQ. JCEN was already turned on in `new`.
        let regs = T::regs();
        regs.jpeg_confr0().write(|w| w.set_start(false));
        regs.jpeg_cr().modify(|w| {
            w.set_iftie(false);
            w.set_ifnfie(false);
            w.set_oftie(false);
            w.set_ofneie(false);
            w.set_eocie(false);
            w.set_hpdie(false);
            w.set_idmaen(false);
            w.set_odmaen(false);
            w.set_iff(true);
            w.set_off(true);
        });
        regs.jpeg_cfr().write(|w| {
            w.set_ceocf(true);
            w.set_chpdf(true);
        });

        // Run the codec. unwrap: Async-mode constructor always populates DMA.
        debug!("jpeg encode: entering run_codec");
        let dst_payload = &mut dst[header_len..header_len + payload_room];
        let indma = self.indma.as_mut().unwrap();
        let outdma = self.outdma.as_mut().unwrap();
        let payload_bytes = run_codec::<T>(indma, outdma, src, dst_payload).await?;
        debug!("jpeg encode: run_codec done, {} bytes", payload_bytes);

        // Stop the codec and tear down DMA / IRQ enables.
        regs.jpeg_confr0().write(|w| w.set_start(false));
        regs.jpeg_cr().modify(|w| {
            w.set_idmaen(false);
            w.set_odmaen(false);
            w.set_eocie(false);
        });

        // Append EOI marker.
        let eoi_at = header_len + payload_bytes;
        if dst.len() < eoi_at + 2 {
            return Err(Error::OutputTooSmall);
        }
        dst[eoi_at] = 0xFF;
        dst[eoi_at + 1] = 0xD9;

        Ok(eoi_at + 2)
    }

    /// Encode planar Y/Cb/Cr into a JPEG. The driver shuffles `planes` into MCU
    /// order in the caller-supplied `scratch` buffer (which must be exactly the
    /// MCU-ordered input size for `cfg`), then runs [`encode`](Jpeg::encode).
    pub async fn encode_planar(
        &mut self,
        planes: PlanarYCbCr<'_>,
        scratch: &mut [u8],
        cfg: &EncodeConfig,
        dst: &mut [u8],
    ) -> Result<usize, Error> {
        if cfg.color_space != ColorSpace::YCbCr {
            return Err(Error::InvalidConfig);
        }
        validate_encode_config(cfg)?;

        let expected_scratch = mcu_ordered_input_size(cfg);
        if scratch.len() != expected_scratch {
            return Err(Error::InvalidConfig);
        }
        let (y_size, c_w, c_h) = ycbcr_plane_dims(cfg);
        if planes.y.len() != y_size || planes.cb.len() != c_w * c_h || planes.cr.len() != c_w * c_h {
            return Err(Error::InvalidConfig);
        }

        planar_to_mcu(&planes, cfg, scratch);
        self.encode(scratch, cfg, dst).await
    }

    /// Decode a JPEG bitstream into planar Y/Cb/Cr.
    ///
    /// The peripheral parses the JFIF header in hardware (`HDR=1`); after the
    /// HPDF interrupt the driver reads back the image dimensions and color
    /// space, then runs the codec into an MCU-row staging buffer that gets
    /// scattered into the caller's plane buffers.
    pub async fn decode(&mut self, src: &[u8], mut dst: PlanarYCbCrMut<'_>) -> Result<DecodeInfo, Error> {
        if src.as_ptr() as usize % 4 != 0 {
            return Err(Error::InvalidConfig);
        }
        // Feed the leading full words via aligned u32 access; the trailing
        // 0..3 bytes are packed into a final word with zero padding (the codec
        // signals EOC once it has consumed all the JPEG data, so trailing
        // zeros after EOI are harmless).
        let full_words = src.len() / 4;
        let tail_bytes = src.len() % 4;
        let src_words = unsafe { core::slice::from_raw_parts(src.as_ptr() as *const u32, full_words) };
        let tail_word: u32 = if tail_bytes > 0 {
            let mut t = [0u8; 4];
            t[..tail_bytes].copy_from_slice(&src[full_words * 4..]);
            u32::from_le_bytes(t)
        } else {
            0
        };
        let total_words = full_words + if tail_bytes > 0 { 1 } else { 0 };
        let word_at = |i: usize| -> u32 { if i < full_words { src_words[i] } else { tail_word } };

        let regs = T::regs();

        // Reset peripheral state. JCEN was set in `new` and stays on.
        regs.jpeg_confr0().write(|w| w.set_start(false));
        regs.jpeg_cr().modify(|w| {
            w.set_iftie(false);
            w.set_ifnfie(false);
            w.set_oftie(false);
            w.set_ofneie(false);
            w.set_eocie(false);
            w.set_hpdie(false);
            w.set_idmaen(false);
            w.set_odmaen(false);
            w.set_iff(true);
            w.set_off(true);
        });
        regs.jpeg_cfr().write(|w| {
            w.set_ceocf(true);
            w.set_chpdf(true);
        });

        // Configure: decode + HW header parsing.
        regs.jpeg_confr1().write(|w| {
            w.set_de(true);
            w.set_hdr(true);
        });

        // Start the codec.
        regs.jpeg_confr0().write(|w| w.set_start(true));

        // CPU-driven feed of the bitstream until HPDF (header parsed) or EOC.
        let mut in_idx = 0usize;
        loop {
            let sr = regs.jpeg_sr().read();
            if sr.hpdf() || sr.eocf() {
                break;
            }
            if sr.ifnff() && in_idx < total_words {
                regs.jpeg_dir().write(|w| w.set_datain(word_at(in_idx)));
                in_idx += 1;
            } else {
                if in_idx >= total_words {
                    return Err(Error::InputTruncated);
                }
                embassy_futures::yield_now().await;
            }
        }

        // Read back image params from CONFRx.
        let info = read_decode_info::<T>()?;

        // Validate output sizing.
        let (y_size, c_w, c_h) = match info.color_space {
            ColorSpace::Grayscale => (info.width as usize * info.height as usize, 0, 0),
            ColorSpace::YCbCr => {
                let (y, cw, ch) = ycbcr_plane_dims_for(info.width, info.height, info.subsampling);
                (y, cw, ch)
            }
        };
        let c_size = c_w * c_h;
        if dst.y.len() < y_size
            || (matches!(info.color_space, ColorSpace::YCbCr) && (dst.cb.len() < c_size || dst.cr.len() < c_size))
        {
            // Abort the codec.
            regs.jpeg_confr0().write(|w| w.set_start(false));
            return Err(Error::OutputTooSmall);
        }

        // Clear HPDF; continue feeding input + draining output until EOC.
        regs.jpeg_cfr().write(|w| w.set_chpdf(true));
        let y_cap = dst.y.len();
        let cb_cap = dst.cb.len();
        let cr_cap = dst.cr.len();
        let (y_pos, c_pos) = {
            let mut sink = PlanarSink {
                info,
                y_w: info.width as usize,
                c_w,
                y_pos: 0,
                c_pos: 0,
                dst: &mut dst,
            };
            loop {
                let sr = regs.jpeg_sr().read();
                let mut did_io = false;
                if sr.ifnff() && in_idx < src_words.len() {
                    regs.jpeg_dir().write(|w| w.set_datain(src_words[in_idx]));
                    in_idx += 1;
                    did_io = true;
                }
                if sr.ofnef() {
                    let word = regs.jpeg_dor().read().dataout();
                    sink.push(&word.to_le_bytes());
                    did_io = true;
                }
                if sr.eocf() {
                    while regs.jpeg_sr().read().ofnef() {
                        let word = regs.jpeg_dor().read().dataout();
                        sink.push(&word.to_le_bytes());
                    }
                    regs.jpeg_cfr().write(|w| w.set_ceocf(true));
                    break;
                }
                if !did_io {
                    embassy_futures::yield_now().await;
                }
            }
            (sink.y_pos, sink.c_pos)
        };

        regs.jpeg_confr0().write(|w| w.set_start(false));

        Ok(DecodeInfo {
            y_bytes: y_pos.min(y_cap),
            cb_bytes: c_pos.min(cb_cap),
            cr_bytes: c_pos.min(cr_cap),
            ..info
        })
    }
}

impl<'d, T: Instance, M: Mode> Jpeg<'d, T, M> {
    /// Blocking variant of encode — drives the codec FIFOs with the CPU.
    /// Available in both [`Async`] and [`Blocking`] modes; useful when no DMA
    /// channel is available.
    pub fn encode_blocking(&mut self, src: &[u8], cfg: &EncodeConfig, dst: &mut [u8]) -> Result<usize, Error> {
        validate_encode_config(cfg)?;

        let expected_input = mcu_ordered_input_size(cfg);
        if src.len() != expected_input {
            return Err(Error::InvalidConfig);
        }
        if src.as_ptr() as usize % 4 != 0 || src.len() % 4 != 0 {
            return Err(Error::InvalidConfig);
        }

        let header_len = emit_jfif_header(cfg, dst)?;
        if dst.len() < header_len + 2 {
            return Err(Error::OutputTooSmall);
        }
        let payload_room = dst.len() - header_len - 2;
        let payload_word_cap = payload_room / 4;

        configure_encode::<T>(cfg);

        let regs = T::regs();
        regs.jpeg_confr0().write(|w| w.set_start(false));
        regs.jpeg_cr().modify(|w| {
            w.set_iftie(false);
            w.set_ifnfie(false);
            w.set_oftie(false);
            w.set_ofneie(false);
            w.set_eocie(false);
            w.set_hpdie(false);
            w.set_idmaen(false);
            w.set_odmaen(false);
            w.set_iff(true);
            w.set_off(true);
        });
        regs.jpeg_cfr().write(|w| {
            w.set_ceocf(true);
            w.set_chpdf(true);
        });

        let src_words = unsafe { core::slice::from_raw_parts(src.as_ptr() as *const u32, src.len() / 4) };
        let payload = &mut dst[header_len..header_len + payload_room];

        regs.jpeg_confr0().write(|w| w.set_start(true));

        let mut in_idx = 0usize;
        let mut out_idx = 0usize;
        loop {
            let sr = regs.jpeg_sr().read();
            let mut did_io = false;
            if sr.ifnff() && in_idx < src_words.len() {
                regs.jpeg_dir().write(|w| w.set_datain(src_words[in_idx]));
                in_idx += 1;
                did_io = true;
            }
            if sr.ofnef() && out_idx < payload_word_cap {
                let word = regs.jpeg_dor().read().dataout();
                let off = out_idx * 4;
                payload[off..off + 4].copy_from_slice(&word.to_le_bytes());
                out_idx += 1;
                did_io = true;
            }
            if sr.eocf() {
                while regs.jpeg_sr().read().ofnef() && out_idx < payload_word_cap {
                    let word = regs.jpeg_dor().read().dataout();
                    let off = out_idx * 4;
                    payload[off..off + 4].copy_from_slice(&word.to_le_bytes());
                    out_idx += 1;
                }
                regs.jpeg_cfr().write(|w| w.set_ceocf(true));
                break;
            }
            if !did_io && out_idx >= payload_word_cap {
                regs.jpeg_confr0().write(|w| w.set_start(false));
                return Err(Error::OutputTooSmall);
            }
        }

        regs.jpeg_confr0().write(|w| w.set_start(false));

        let payload_bytes = out_idx * 4;
        let eoi_at = header_len + payload_bytes;
        if dst.len() < eoi_at + 2 {
            return Err(Error::OutputTooSmall);
        }
        dst[eoi_at] = 0xFF;
        dst[eoi_at + 1] = 0xD9;
        Ok(eoi_at + 2)
    }

    /// Blocking variant of decode. The async decode is already CPU-driven;
    /// this version simply doesn't yield to the executor. Available in both
    /// [`Async`] and [`Blocking`] modes.
    pub fn decode_blocking(&mut self, src: &[u8], mut dst: PlanarYCbCrMut<'_>) -> Result<DecodeInfo, Error> {
        if src.as_ptr() as usize % 4 != 0 {
            return Err(Error::InvalidConfig);
        }
        let full_words = src.len() / 4;
        let tail_bytes = src.len() % 4;
        let src_words = unsafe { core::slice::from_raw_parts(src.as_ptr() as *const u32, full_words) };
        let tail_word: u32 = if tail_bytes > 0 {
            let mut t = [0u8; 4];
            t[..tail_bytes].copy_from_slice(&src[full_words * 4..]);
            u32::from_le_bytes(t)
        } else {
            0
        };
        let total_words = full_words + if tail_bytes > 0 { 1 } else { 0 };
        let word_at = |i: usize| -> u32 { if i < full_words { src_words[i] } else { tail_word } };

        let regs = T::regs();
        regs.jpeg_confr0().write(|w| w.set_start(false));
        regs.jpeg_cr().modify(|w| {
            w.set_iftie(false);
            w.set_ifnfie(false);
            w.set_oftie(false);
            w.set_ofneie(false);
            w.set_eocie(false);
            w.set_hpdie(false);
            w.set_idmaen(false);
            w.set_odmaen(false);
            w.set_iff(true);
            w.set_off(true);
        });
        regs.jpeg_cfr().write(|w| {
            w.set_ceocf(true);
            w.set_chpdf(true);
        });

        regs.jpeg_confr1().write(|w| {
            w.set_de(true);
            w.set_hdr(true);
        });
        regs.jpeg_confr0().write(|w| w.set_start(true));

        let mut in_idx = 0usize;
        loop {
            let sr = regs.jpeg_sr().read();
            if sr.hpdf() || sr.eocf() {
                break;
            }
            if sr.ifnff() && in_idx < total_words {
                regs.jpeg_dir().write(|w| w.set_datain(word_at(in_idx)));
                in_idx += 1;
            } else if in_idx >= total_words {
                return Err(Error::InputTruncated);
            }
        }

        let info = read_decode_info::<T>()?;

        let (y_size, c_w, c_h) = match info.color_space {
            ColorSpace::Grayscale => (info.width as usize * info.height as usize, 0, 0),
            ColorSpace::YCbCr => ycbcr_plane_dims_for(info.width, info.height, info.subsampling),
        };
        let c_size = c_w * c_h;
        if dst.y.len() < y_size
            || (matches!(info.color_space, ColorSpace::YCbCr) && (dst.cb.len() < c_size || dst.cr.len() < c_size))
        {
            regs.jpeg_confr0().write(|w| w.set_start(false));
            return Err(Error::OutputTooSmall);
        }

        regs.jpeg_cfr().write(|w| w.set_chpdf(true));
        let y_cap = dst.y.len();
        let cb_cap = dst.cb.len();
        let cr_cap = dst.cr.len();
        let (y_pos, c_pos) = {
            let mut sink = PlanarSink {
                info,
                y_w: info.width as usize,
                c_w,
                y_pos: 0,
                c_pos: 0,
                dst: &mut dst,
            };
            loop {
                let sr = regs.jpeg_sr().read();
                if sr.ifnff() && in_idx < total_words {
                    regs.jpeg_dir().write(|w| w.set_datain(word_at(in_idx)));
                    in_idx += 1;
                }
                if sr.ofnef() {
                    let word = regs.jpeg_dor().read().dataout();
                    sink.push(&word.to_le_bytes());
                }
                if sr.eocf() {
                    while regs.jpeg_sr().read().ofnef() {
                        let word = regs.jpeg_dor().read().dataout();
                        sink.push(&word.to_le_bytes());
                    }
                    regs.jpeg_cfr().write(|w| w.set_ceocf(true));
                    break;
                }
            }
            (sink.y_pos, sink.c_pos)
        };

        regs.jpeg_confr0().write(|w| w.set_start(false));

        Ok(DecodeInfo {
            y_bytes: y_pos.min(y_cap),
            cb_bytes: c_pos.min(cb_cap),
            cr_bytes: c_pos.min(cr_cap),
            ..info
        })
    }
}

// ===== Codec orchestration =====

async fn run_codec<'d, T: Instance>(
    indma: &mut ChannelAndRequest<'d>,
    outdma: &mut ChannelAndRequest<'d>,
    src: &[u8],
    dst: &mut [u8],
) -> Result<usize, Error> {
    // The JPEG_DIR / JPEG_DOR registers are 32-bit. Buffers must be 4-byte
    // aligned and a multiple of 4 bytes long; we walk them as &[u32].
    if src.as_ptr() as usize % 4 != 0 || src.len() % 4 != 0 {
        return Err(Error::InvalidConfig);
    }
    if dst.as_ptr() as usize % 4 != 0 || dst.len() < 4 {
        return Err(Error::OutputTooSmall);
    }
    let src_words = unsafe { core::slice::from_raw_parts(src.as_ptr() as *const u32, src.len() / 4) };
    let dst_word_len = dst.len() / 4;
    let dst_words = unsafe { core::slice::from_raw_parts_mut(dst.as_mut_ptr() as *mut u32, dst_word_len) };

    let regs = T::regs();
    let dir_ptr = regs.jpeg_dir().as_ptr() as *mut u32;
    let dor_ptr = regs.jpeg_dor().as_ptr() as *mut u32;

    // The JPEG peripheral on N6 sits behind RISAF (so DMA transactions need
    // SSEC=DSEC=1) and only asserts its DMA request line for bursts above
    // ~2 beats. ST CubeN6 uses burst-of-8 with the secure attribute set;
    // we mirror that. The `secure` and `burst_length` fields only exist on
    // gpdma-stm32n6 builds; other backends use their native burst encoding.
    #[cfg(gpdma)]
    let opts = TransferOptions {
        priority: crate::dma::Priority::High,
        #[cfg(stm32n6)]
        secure: true,
        #[cfg(stm32n6)]
        burst_length: crate::dma::Burst::_8Beats,
        ..Default::default()
    };
    #[cfg(not(gpdma))]
    let opts = TransferOptions {
        priority: crate::dma::Priority::High,
        pburst: crate::dma::Burst::Incr8,
        mburst: crate::dma::Burst::Incr8,
        ..Default::default()
    };

    debug!("run_codec: arming DMAs");
    let in_xfer = unsafe { indma.write_raw(src_words, dir_ptr, opts) };
    let out_xfer = unsafe { outdma.read_raw(dor_ptr, dst_words, opts) };

    debug!("run_codec: enabling JPEG IDMA/ODMA/EOCIE");
    regs.jpeg_cr().modify(|w| {
        w.set_idmaen(true);
        w.set_odmaen(true);
        w.set_eocie(true);
    });
    debug!("run_codec: starting codec");
    regs.jpeg_confr0().write(|w| w.set_start(true));
    debug!("run_codec: awaiting EOC vs out_xfer");

    // Mirror the usart.rs::inner_read pattern: keep the unfinished out_xfer
    // alive in the second arm so we can query get_remaining_transfers().
    let eoc = core::future::poll_fn(|cx| {
        JPEG_WAKER.register(cx.waker());
        let sr = regs.jpeg_sr().read();
        if sr.eocf() {
            core::task::Poll::Ready(())
        } else {
            regs.jpeg_cr().modify(|w| w.set_eocie(true));
            core::task::Poll::Pending
        }
    });

    use futures_util::future::{Either, select};
    let bytes = match select(out_xfer, core::pin::pin!(eoc)).await {
        Either::Left(((), _eoc_unfinished)) => {
            // Output buffer filled before EOC — caller's dst is too small.
            regs.jpeg_cfr().write(|w| w.set_ceocf(true));
            regs.jpeg_cr().modify(|w| {
                w.set_idmaen(false);
                w.set_odmaen(false);
                w.set_eocie(false);
            });
            drop(in_xfer);
            return Err(Error::OutputTooSmall);
        }
        Either::Right(((), out_xfer)) => {
            // The DMA may have stopped a few words short of fully draining
            // the output FIFO after EOC. We have access to the channel's
            // remaining count from the unfinished transfer, but the codec
            // typically holds 1-4 trailing words in its output FIFO that
            // never get DMA'd. Tail-drain via CPU.
            let mut written = (dst_word_len.saturating_sub(out_xfer.get_remaining_transfers() as usize)) * 4;
            drop(out_xfer);
            while regs.jpeg_sr().read().ofnef() && written / 4 < dst_word_len {
                let word = regs.jpeg_dor().read().dataout();
                let idx = written / 4;
                dst_words[idx] = word;
                written += 4;
            }
            regs.jpeg_cfr().write(|w| w.set_ceocf(true));
            written
        }
    };

    regs.jpeg_cr().modify(|w| {
        w.set_idmaen(false);
        w.set_odmaen(false);
        w.set_eocie(false);
    });
    drop(in_xfer);

    Ok(bytes)
}

struct PlanarSink<'a, 'd> {
    info: DecodeInfo,
    y_w: usize,
    c_w: usize,
    y_pos: usize,
    c_pos: usize,
    dst: &'a mut PlanarYCbCrMut<'d>,
}

impl<'a, 'd> PlanarSink<'a, 'd> {
    fn push(&mut self, bytes: &[u8; 4]) {
        // For grayscale the codec emits 8×8 = 64 bytes per MCU in raster order.
        // For YCbCr 4:4:4 each MCU is Y(64) | Cb(64) | Cr(64).
        // For 4:2:2 each MCU is Y(64*2) | Cb(64) | Cr(64), MCU is 16×8.
        // For 4:2:0 each MCU is Y(64*4) | Cb(64) | Cr(64), MCU is 16×16.
        //
        // Simplification: for v1, store everything sequentially into the
        // appropriate plane buffer based on a per-MCU counter. The exact
        // raster-to-plane mapping per MCU geometry is more involved than we
        // need for the encode/decode round-trip test — caller can re-shuffle.
        match self.info.color_space {
            ColorSpace::Grayscale => {
                let n = (self.dst.y.len() - self.y_pos).min(4);
                self.dst.y[self.y_pos..self.y_pos + n].copy_from_slice(&bytes[..n]);
                self.y_pos += n;
            }
            ColorSpace::YCbCr => {
                let _ = self.y_w;
                let _ = self.c_w;
                // v1 limitation: chroma scatter not implemented — stuff bytes
                // sequentially into Y first, then Cb, then Cr. The caller gets
                // raw MCU-ordered output split by plane size.
                let total = self.dst.y.len() + self.dst.cb.len() + self.dst.cr.len();
                for &b in bytes {
                    let pos = self.y_pos + self.c_pos * 2;
                    if pos >= total {
                        return;
                    }
                    if self.y_pos < self.dst.y.len() {
                        self.dst.y[self.y_pos] = b;
                        self.y_pos += 1;
                    } else if self.c_pos < self.dst.cb.len() + self.dst.cr.len() {
                        if self.c_pos < self.dst.cb.len() {
                            self.dst.cb[self.c_pos] = b;
                        } else {
                            let cr_idx = self.c_pos - self.dst.cb.len();
                            if cr_idx < self.dst.cr.len() {
                                self.dst.cr[cr_idx] = b;
                            }
                        }
                        self.c_pos += 1;
                    }
                }
            }
        }
    }
}

// ===== Configuration helpers =====

fn validate_encode_config(cfg: &EncodeConfig) -> Result<(), Error> {
    if cfg.width == 0 || cfg.height == 0 {
        return Err(Error::InvalidConfig);
    }
    if cfg.quality < 1 || cfg.quality > 100 {
        return Err(Error::InvalidConfig);
    }
    let (mcu_w, mcu_h) = mcu_size(cfg);
    if cfg.width as usize % mcu_w != 0 || cfg.height as usize % mcu_h != 0 {
        return Err(Error::InvalidConfig);
    }
    Ok(())
}

fn mcu_size(cfg: &EncodeConfig) -> (usize, usize) {
    match (cfg.color_space, cfg.subsampling) {
        (ColorSpace::Grayscale, _) => (8, 8),
        (ColorSpace::YCbCr, ChromaSubsampling::S444) => (8, 8),
        (ColorSpace::YCbCr, ChromaSubsampling::S422) => (16, 8),
        (ColorSpace::YCbCr, ChromaSubsampling::S420) => (16, 16),
    }
}

fn mcu_count(cfg: &EncodeConfig) -> u32 {
    let (w, h) = mcu_size(cfg);
    ((cfg.width as usize / w) * (cfg.height as usize / h)) as u32
}

fn mcu_ordered_input_size(cfg: &EncodeConfig) -> usize {
    let n = mcu_count(cfg) as usize;
    match (cfg.color_space, cfg.subsampling) {
        (ColorSpace::Grayscale, _) => n * 64,
        (ColorSpace::YCbCr, ChromaSubsampling::S444) => n * 64 * 3,
        (ColorSpace::YCbCr, ChromaSubsampling::S422) => n * 64 * 4,
        (ColorSpace::YCbCr, ChromaSubsampling::S420) => n * 64 * 6,
    }
}

fn ycbcr_plane_dims(cfg: &EncodeConfig) -> (usize, usize, usize) {
    ycbcr_plane_dims_for(cfg.width, cfg.height, cfg.subsampling)
}

fn ycbcr_plane_dims_for(width: u16, height: u16, sub: ChromaSubsampling) -> (usize, usize, usize) {
    let w = width as usize;
    let h = height as usize;
    match sub {
        ChromaSubsampling::S444 => (w * h, w, h),
        ChromaSubsampling::S422 => (w * h, w / 2, h),
        ChromaSubsampling::S420 => (w * h, w / 2, h / 2),
    }
}

fn configure_encode<T: Instance>(cfg: &EncodeConfig) {
    let regs = T::regs();
    let nmcu = mcu_count(cfg).saturating_sub(1);

    let (nf, ns, colspace) = match cfg.color_space {
        ColorSpace::Grayscale => (0u8, 0u8, 0u8),
        ColorSpace::YCbCr => (2u8, 2u8, 1u8),
    };

    regs.jpeg_confr1().write(|w| {
        w.set_de(false);
        w.set_hdr(false);
        w.set_nf(nf);
        w.set_ns(ns);
        w.set_colorspace(colspace);
        w.set_ysize(cfg.height);
    });
    regs.jpeg_confr2().write(|w| w.set_nmcu(nmcu));
    regs.jpeg_confr3().write(|w| w.set_xsize(cfg.width));

    // CONFR4 = component 0 (Y / Gray)
    let (hsf_y, vsf_y, nb_y) = match (cfg.color_space, cfg.subsampling) {
        (ColorSpace::Grayscale, _) | (ColorSpace::YCbCr, ChromaSubsampling::S444) => (1u8, 1u8, 0u8),
        (ColorSpace::YCbCr, ChromaSubsampling::S422) => (2, 1, 1),
        (ColorSpace::YCbCr, ChromaSubsampling::S420) => (2, 2, 3),
    };
    regs.jpeg_confr4().write(|w| {
        w.set_hsf(hsf_y);
        w.set_vsf(vsf_y);
        w.set_nb(nb_y);
        w.set_qt(0);
        w.set_ha(false);
        w.set_hd(false);
    });

    if matches!(cfg.color_space, ColorSpace::YCbCr) {
        // CONFR5/6 = chroma components, share quant table 1, Huffman tables 1.
        for confr_set in [|w: &mut pac::jpeg::regs::JpegConfr5| {
            w.set_hsf(1);
            w.set_vsf(1);
            w.set_nb(0);
            w.set_qt(1);
            w.set_ha(true);
            w.set_hd(true);
        }] {
            regs.jpeg_confr5().write(confr_set);
        }
        regs.jpeg_confr6().write(|w| {
            w.set_hsf(1);
            w.set_vsf(1);
            w.set_nb(0);
            w.set_qt(1);
            w.set_ha(true);
            w.set_hd(true);
        });
    }

    // Quantization tables.
    load_qmem::<T>(0, &lum_quant_scaled(cfg.quality));
    if matches!(cfg.color_space, ColorSpace::YCbCr) {
        load_qmem::<T>(1, &chrom_quant_scaled(cfg.quality));
    }
}

fn read_decode_info<T: Instance>() -> Result<DecodeInfo, Error> {
    let regs = T::regs();
    let confr1 = regs.jpeg_confr1().read();
    let nf = confr1.nf();
    let color_space = match nf {
        0 => ColorSpace::Grayscale,
        2 => ColorSpace::YCbCr,
        _ => return Err(Error::Unsupported),
    };
    let height = confr1.ysize();
    let width = regs.jpeg_confr3().read().xsize();

    // Determine subsampling from CONFR4 sampling factors.
    let confr4 = regs.jpeg_confr4().read();
    let subsampling = match (confr4.hsf(), confr4.vsf()) {
        (1, 1) => ChromaSubsampling::S444,
        (2, 1) => ChromaSubsampling::S422,
        (2, 2) => ChromaSubsampling::S420,
        _ => return Err(Error::Unsupported),
    };

    Ok(DecodeInfo {
        width,
        height,
        color_space,
        subsampling,
        y_bytes: 0,
        cb_bytes: 0,
        cr_bytes: 0,
    })
}

// ===== Quantization tables =====

const ZIGZAG: [u8; 64] = [
    0, 1, 8, 16, 9, 2, 3, 10, 17, 24, 32, 25, 18, 11, 4, 5, 12, 19, 26, 33, 40, 48, 41, 34, 27, 20, 13, 6, 7, 14, 21,
    28, 35, 42, 49, 56, 57, 50, 43, 36, 29, 22, 15, 23, 30, 37, 44, 51, 58, 59, 52, 45, 38, 31, 39, 46, 53, 60, 61, 54,
    47, 55, 62, 63,
];

const LUM_QUANT: [u8; 64] = [
    16, 11, 10, 16, 24, 40, 51, 61, 12, 12, 14, 19, 26, 58, 60, 55, 14, 13, 16, 24, 40, 57, 69, 56, 14, 17, 22, 29, 51,
    87, 80, 62, 18, 22, 37, 56, 68, 109, 103, 77, 24, 35, 55, 64, 81, 104, 113, 92, 49, 64, 78, 87, 103, 121, 120, 101,
    72, 92, 95, 98, 112, 100, 103, 99,
];

const CHROM_QUANT: [u8; 64] = [
    17, 18, 24, 47, 99, 99, 99, 99, 18, 21, 26, 66, 99, 99, 99, 99, 24, 26, 56, 99, 99, 99, 99, 99, 47, 66, 99, 99, 99,
    99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99,
    99, 99, 99, 99, 99, 99,
];

fn scale_factor(quality: u8) -> u32 {
    let q = quality as u32;
    if q >= 50 { 200 - q * 2 } else { 5000 / q }
}

fn lum_quant_scaled(quality: u8) -> [u8; 64] {
    quant_scaled(&LUM_QUANT, quality)
}

fn chrom_quant_scaled(quality: u8) -> [u8; 64] {
    quant_scaled(&CHROM_QUANT, quality)
}

fn quant_scaled(table: &[u8; 64], quality: u8) -> [u8; 64] {
    let s = scale_factor(quality);
    let mut out = [0u8; 64];
    for (i, &v) in table.iter().enumerate() {
        let q = (v as u32 * s + 50) / 100;
        out[i] = q.clamp(1, 255) as u8;
    }
    out
}

fn load_qmem<T: Instance>(bank: usize, table: &[u8; 64]) {
    // The peripheral expects coefficients in zig-zag order, packed 4 bytes per
    // 32-bit register (low byte = first coefficient).
    let base = (T::regs().as_ptr() as usize + QMEM0_OFFSET + bank * QMEM_BANK_STRIDE) as *mut u32;
    for i in 0..16 {
        let mut word: u32 = 0;
        for j in 0..4 {
            let idx = ZIGZAG[i * 4 + j] as usize;
            word |= (table[idx] as u32) << (8 * j);
        }
        unsafe { base.add(i).write_volatile(word) };
    }
}

// ===== Annex K Huffman tables (BITS + HUFFVAL) =====

const DC_LUM_BITS: [u8; 16] = [0, 1, 5, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0];
const DC_LUM_HUFFVAL: [u8; 12] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0xa, 0xb];
const DC_CHROM_BITS: [u8; 16] = [0, 3, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0];
const DC_CHROM_HUFFVAL: [u8; 12] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0xa, 0xb];

const AC_LUM_BITS: [u8; 16] = [0, 2, 1, 3, 3, 2, 4, 3, 5, 5, 4, 4, 0, 0, 1, 0x7d];
#[rustfmt::skip]
const AC_LUM_HUFFVAL: [u8; 162] = [
    0x01, 0x02, 0x03, 0x00, 0x04, 0x11, 0x05, 0x12, 0x21, 0x31, 0x41, 0x06, 0x13, 0x51, 0x61, 0x07,
    0x22, 0x71, 0x14, 0x32, 0x81, 0x91, 0xa1, 0x08, 0x23, 0x42, 0xb1, 0xc1, 0x15, 0x52, 0xd1, 0xf0,
    0x24, 0x33, 0x62, 0x72, 0x82, 0x09, 0x0a, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x25, 0x26, 0x27, 0x28,
    0x29, 0x2a, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3a, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49,
    0x4a, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5a, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69,
    0x6a, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7a, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88, 0x89,
    0x8a, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97, 0x98, 0x99, 0x9a, 0xa2, 0xa3, 0xa4, 0xa5, 0xa6, 0xa7,
    0xa8, 0xa9, 0xaa, 0xb2, 0xb3, 0xb4, 0xb5, 0xb6, 0xb7, 0xb8, 0xb9, 0xba, 0xc2, 0xc3, 0xc4, 0xc5,
    0xc6, 0xc7, 0xc8, 0xc9, 0xca, 0xd2, 0xd3, 0xd4, 0xd5, 0xd6, 0xd7, 0xd8, 0xd9, 0xda, 0xe1, 0xe2,
    0xe3, 0xe4, 0xe5, 0xe6, 0xe7, 0xe8, 0xe9, 0xea, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8,
    0xf9, 0xfa,
];

const AC_CHROM_BITS: [u8; 16] = [0, 2, 1, 2, 4, 4, 3, 4, 7, 5, 4, 4, 0, 1, 2, 0x77];
#[rustfmt::skip]
const AC_CHROM_HUFFVAL: [u8; 162] = [
    0x00, 0x01, 0x02, 0x03, 0x11, 0x04, 0x05, 0x21, 0x31, 0x06, 0x12, 0x41, 0x51, 0x07, 0x61, 0x71,
    0x13, 0x22, 0x32, 0x81, 0x08, 0x14, 0x42, 0x91, 0xa1, 0xb1, 0xc1, 0x09, 0x23, 0x33, 0x52, 0xf0,
    0x15, 0x62, 0x72, 0xd1, 0x0a, 0x16, 0x24, 0x34, 0xe1, 0x25, 0xf1, 0x17, 0x18, 0x19, 0x1a, 0x26,
    0x27, 0x28, 0x29, 0x2a, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3a, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48,
    0x49, 0x4a, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5a, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68,
    0x69, 0x6a, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7a, 0x82, 0x83, 0x84, 0x85, 0x86, 0x87,
    0x88, 0x89, 0x8a, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97, 0x98, 0x99, 0x9a, 0xa2, 0xa3, 0xa4, 0xa5,
    0xa6, 0xa7, 0xa8, 0xa9, 0xaa, 0xb2, 0xb3, 0xb4, 0xb5, 0xb6, 0xb7, 0xb8, 0xb9, 0xba, 0xc2, 0xc3,
    0xc4, 0xc5, 0xc6, 0xc7, 0xc8, 0xc9, 0xca, 0xd2, 0xd3, 0xd4, 0xd5, 0xd6, 0xd7, 0xd8, 0xd9, 0xda,
    0xe2, 0xe3, 0xe4, 0xe5, 0xe6, 0xe7, 0xe8, 0xe9, 0xea, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8,
    0xf9, 0xfa,
];

// JPEG Annex C: convert BITS into per-symbol code length and code value.
fn bits_to_size_codes(bits: &[u8; 16], huffsize: &mut [u8; 257], huffcode: &mut [u32; 257]) -> usize {
    let mut p = 0;
    for l in 0..16 {
        let mut i = bits[l] as usize;
        while i > 0 {
            huffsize[p] = (l + 1) as u8;
            p += 1;
            i -= 1;
        }
    }
    huffsize[p] = 0;
    let last_k = p;

    let mut code: u32 = 0;
    let mut si = huffsize[0] as u32;
    let mut p = 0;
    while huffsize[p] != 0 {
        while huffsize[p] as u32 == si {
            huffcode[p] = code;
            p += 1;
            code += 1;
        }
        code <<= 1;
        si += 1;
    }
    last_k
}

fn dc_huffenc_table(bits: &[u8; 16], huffval: &[u8; 12]) -> ([u8; DC_HUFF_TABLE_SIZE], [u32; DC_HUFF_TABLE_SIZE]) {
    let mut huffsize = [0u8; 257];
    let mut huffcode = [0u32; 257];
    let last_k = bits_to_size_codes(bits, &mut huffsize, &mut huffcode);
    let mut code_len = [0u8; DC_HUFF_TABLE_SIZE];
    let mut code = [0u32; DC_HUFF_TABLE_SIZE];
    for k in 0..last_k {
        let l = huffval[k] as usize;
        code[l] = huffcode[k];
        code_len[l] = huffsize[k] - 1;
    }
    (code_len, code)
}

fn ac_huffenc_table(bits: &[u8; 16], huffval: &[u8; 162]) -> ([u8; AC_HUFF_TABLE_SIZE], [u32; AC_HUFF_TABLE_SIZE]) {
    let mut huffsize = [0u8; 257];
    let mut huffcode = [0u32; 257];
    let last_k = bits_to_size_codes(bits, &mut huffsize, &mut huffcode);
    let mut code_len = [0u8; AC_HUFF_TABLE_SIZE];
    let mut code = [0u32; AC_HUFF_TABLE_SIZE];
    for k in 0..last_k {
        // JPEG Annex C: AC symbols are remapped to 0..161 for the peripheral
        // table by mapping (run, size) → run*10 + size - 1. Special cases:
        // 0x00 (EOB) maps to 160, 0xF0 (ZRL) maps to 161.
        let mut l = huffval[k] as usize;
        if l == 0 {
            l = 160;
        } else if l == 0xF0 {
            l = 161;
        } else {
            let msb = (l >> 4) & 0xF;
            let lsb = l & 0xF;
            l = msb * 10 + lsb - 1;
        }
        code[l] = huffcode[k];
        code_len[l] = huffsize[k] - 1;
    }
    (code_len, code)
}

fn load_huffenc<T: Instance>() {
    // AC tables: 88 32-bit registers each, holding 162 symbol entries (with 14
    // trailing magic words used internally by the codec).
    let base = T::regs().as_ptr() as usize;
    let ac0 = (base + HUFFENC_AC0_OFFSET) as *mut u32;
    let ac1 = (base + HUFFENC_AC0_OFFSET + HUFFENC_AC_BANK_STRIDE) as *mut u32;
    let dc0 = (base + HUFFENC_DC0_OFFSET) as *mut u32;
    let dc1 = (base + HUFFENC_DC0_OFFSET + HUFFENC_DC_BANK_STRIDE) as *mut u32;

    let (acl_len, acl_code) = ac_huffenc_table(&AC_LUM_BITS, &AC_LUM_HUFFVAL);
    let (acc_len, acc_code) = ac_huffenc_table(&AC_CHROM_BITS, &AC_CHROM_HUFFVAL);
    let (dcl_len, dcl_code) = dc_huffenc_table(&DC_LUM_BITS, &DC_LUM_HUFFVAL);
    let (dcc_len, dcc_code) = dc_huffenc_table(&DC_CHROM_BITS, &DC_CHROM_HUFFVAL);

    write_ac_huffenc(ac0, &acl_len, &acl_code);
    write_ac_huffenc(ac1, &acc_len, &acc_code);
    write_dc_huffenc(dc0, &dcl_len, &dcl_code);
    write_dc_huffenc(dc1, &dcc_len, &dcc_code);
}

fn write_ac_huffenc(base: *mut u32, code_len: &[u8; AC_HUFF_TABLE_SIZE], code: &[u32; AC_HUFF_TABLE_SIZE]) {
    // First write magic-fill values for entries 162..175 (registers 81..87).
    // Per the reference manual these locations contain values used internally
    // by the codec — must match the C HAL.
    unsafe {
        base.add(81).write_volatile(0x0FFF_0FFF);
        base.add(82).write_volatile(0x0FFF_0FFF);
        base.add(83).write_volatile(0x0FFF_0FFF);
        base.add(84).write_volatile(0x0FD1_0FD0);
        base.add(85).write_volatile(0x0FD3_0FD2);
        base.add(86).write_volatile(0x0FD5_0FD4);
        base.add(87).write_volatile(0x0FD7_0FD6);
    }
    // Symbol entries 0..161: pack 2 entries per register, low half = even index.
    let mut i = AC_HUFF_TABLE_SIZE;
    while i > 1 {
        i -= 1;
        let msb = (((code_len[i] as u32) & 0xF) << 8) | (code[i] & 0xFF);
        i -= 1;
        let lsb = (((code_len[i] as u32) & 0xF) << 8) | (code[i] & 0xFF);
        let word = lsb | (msb << 16);
        let reg = i / 2;
        unsafe { base.add(reg).write_volatile(word) };
    }
}

fn write_dc_huffenc(base: *mut u32, code_len: &[u8; DC_HUFF_TABLE_SIZE], code: &[u32; DC_HUFF_TABLE_SIZE]) {
    // Entries 12..15 are placeholder fill (length 0xF, code 0xFF).
    unsafe {
        base.add(6).write_volatile(0x0FFF_0FFF);
        base.add(7).write_volatile(0x0FFF_0FFF);
    }
    let mut i = DC_HUFF_TABLE_SIZE;
    while i > 1 {
        i -= 1;
        let msb = (((code_len[i] as u32) & 0xF) << 8) | (code[i] & 0xFF);
        i -= 1;
        let lsb = (((code_len[i] as u32) & 0xF) << 8) | (code[i] & 0xFF);
        let word = lsb | (msb << 16);
        let reg = i / 2;
        unsafe { base.add(reg).write_volatile(word) };
    }
}

// ===== JFIF header emission =====

fn emit_jfif_header(cfg: &EncodeConfig, dst: &mut [u8]) -> Result<usize, Error> {
    let mut w = Writer::new(dst);

    // SOI
    w.put(&[0xFF, 0xD8])?;

    // APP0 / JFIF
    w.put(&[0xFF, 0xE0, 0x00, 0x10])?;
    w.put(b"JFIF\0")?;
    w.put(&[
        0x01, 0x01, // version 1.1
        0x00, // units = no units (aspect ratio only)
        0x00, 0x01, 0x00, 0x01, // X density 1, Y density 1
        0x00, 0x00, // no thumbnail
    ])?;

    // DQT(s)
    let lum_q = lum_quant_scaled(cfg.quality);
    let chrom_q = chrom_quant_scaled(cfg.quality);
    emit_dqt(&mut w, 0, &lum_q)?;
    if matches!(cfg.color_space, ColorSpace::YCbCr) {
        emit_dqt(&mut w, 1, &chrom_q)?;
    }

    // SOF0 (baseline)
    let nf = match cfg.color_space {
        ColorSpace::Grayscale => 1,
        ColorSpace::YCbCr => 3,
    };
    let sof_len = 8 + 3 * nf;
    w.put(&[0xFF, 0xC0, (sof_len >> 8) as u8, sof_len as u8, 8])?;
    w.put(&[
        (cfg.height >> 8) as u8,
        cfg.height as u8,
        (cfg.width >> 8) as u8,
        cfg.width as u8,
    ])?;
    w.put(&[nf as u8])?;
    match cfg.color_space {
        ColorSpace::Grayscale => {
            w.put(&[1, 0x11, 0])?; // component 1: H=1 V=1, quant table 0
        }
        ColorSpace::YCbCr => {
            let (hy, vy) = match cfg.subsampling {
                ChromaSubsampling::S444 => (1, 1),
                ChromaSubsampling::S422 => (2, 1),
                ChromaSubsampling::S420 => (2, 2),
            };
            w.put(&[1, ((hy << 4) | vy) as u8, 0])?; // Y
            w.put(&[2, 0x11, 1])?; // Cb
            w.put(&[3, 0x11, 1])?; // Cr
        }
    }

    // DHT(s)
    emit_dht(&mut w, 0x00, &DC_LUM_BITS, &DC_LUM_HUFFVAL)?;
    emit_dht(&mut w, 0x10, &AC_LUM_BITS, &AC_LUM_HUFFVAL)?;
    if matches!(cfg.color_space, ColorSpace::YCbCr) {
        emit_dht(&mut w, 0x01, &DC_CHROM_BITS, &DC_CHROM_HUFFVAL)?;
        emit_dht(&mut w, 0x11, &AC_CHROM_BITS, &AC_CHROM_HUFFVAL)?;
    }

    // SOS
    let sos_len = 6 + 2 * nf;
    w.put(&[0xFF, 0xDA, (sos_len >> 8) as u8, sos_len as u8, nf as u8])?;
    match cfg.color_space {
        ColorSpace::Grayscale => {
            w.put(&[1, 0x00])?;
        }
        ColorSpace::YCbCr => {
            w.put(&[1, 0x00, 2, 0x11, 3, 0x11])?;
        }
    }
    w.put(&[0x00, 0x3F, 0x00])?; // Ss=0, Se=63, Ah=Al=0

    Ok(w.pos)
}

fn emit_dqt(w: &mut Writer, table_id: u8, table: &[u8; 64]) -> Result<(), Error> {
    w.put(&[0xFF, 0xDB, 0x00, 0x43, table_id])?;
    let mut zz = [0u8; 64];
    for i in 0..64 {
        zz[i] = table[ZIGZAG[i] as usize];
    }
    w.put(&zz)?;
    Ok(())
}

fn emit_dht(w: &mut Writer, table_id: u8, bits: &[u8; 16], huffval: &[u8]) -> Result<(), Error> {
    let len = 3 + 16 + huffval.len() as u16;
    w.put(&[0xFF, 0xC4, (len >> 8) as u8, len as u8, table_id])?;
    w.put(bits)?;
    w.put(huffval)?;
    Ok(())
}

struct Writer<'a> {
    buf: &'a mut [u8],
    pos: usize,
}

impl<'a> Writer<'a> {
    fn new(buf: &'a mut [u8]) -> Self {
        Self { buf, pos: 0 }
    }
    fn put(&mut self, bytes: &[u8]) -> Result<(), Error> {
        if self.pos + bytes.len() > self.buf.len() {
            return Err(Error::OutputTooSmall);
        }
        self.buf[self.pos..self.pos + bytes.len()].copy_from_slice(bytes);
        self.pos += bytes.len();
        Ok(())
    }
}

// ===== Planar → MCU shuffle =====

fn planar_to_mcu(planes: &PlanarYCbCr, cfg: &EncodeConfig, scratch: &mut [u8]) {
    let (mcu_w, mcu_h) = mcu_size(cfg);
    let w = cfg.width as usize;
    let h = cfg.height as usize;
    let (cw, ch) = match cfg.subsampling {
        ChromaSubsampling::S444 => (w, h),
        ChromaSubsampling::S422 => (w / 2, h),
        ChromaSubsampling::S420 => (w / 2, h / 2),
    };
    let _ = ch;
    let mcus_x = w / mcu_w;
    let mcus_y = h / mcu_h;
    let mut out = 0usize;

    for my in 0..mcus_y {
        for mx in 0..mcus_x {
            // Luma block(s).
            match cfg.subsampling {
                ChromaSubsampling::S444 => {
                    copy_block(planes.y, w, mx * 8, my * 8, scratch, &mut out);
                }
                ChromaSubsampling::S422 => {
                    copy_block(planes.y, w, mx * 16, my * 8, scratch, &mut out);
                    copy_block(planes.y, w, mx * 16 + 8, my * 8, scratch, &mut out);
                }
                ChromaSubsampling::S420 => {
                    copy_block(planes.y, w, mx * 16, my * 16, scratch, &mut out);
                    copy_block(planes.y, w, mx * 16 + 8, my * 16, scratch, &mut out);
                    copy_block(planes.y, w, mx * 16, my * 16 + 8, scratch, &mut out);
                    copy_block(planes.y, w, mx * 16 + 8, my * 16 + 8, scratch, &mut out);
                }
            }
            // Cb / Cr blocks.
            copy_block(planes.cb, cw, mx * 8, my * 8, scratch, &mut out);
            copy_block(planes.cr, cw, mx * 8, my * 8, scratch, &mut out);
        }
    }
}

fn copy_block(plane: &[u8], stride: usize, x0: usize, y0: usize, dst: &mut [u8], pos: &mut usize) {
    for row in 0..8 {
        let src_off = (y0 + row) * stride + x0;
        dst[*pos..*pos + 8].copy_from_slice(&plane[src_off..src_off + 8]);
        *pos += 8;
    }
}

// ===== Trait + macro plumbing =====

trait SealedInstance {
    fn regs() -> pac::jpeg::Jpeg;
}

/// JPEG instance trait.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + crate::rcc::RccPeripheral + 'static + Send {
    /// Interrupt for this JPEG instance.
    type Interrupt: interrupt::typelevel::Interrupt;
}

foreach_interrupt!(
    ($inst:ident, jpeg, JPEG, GLOBAL, $irq:ident) => {
        impl Instance for peripherals::$inst {
            type Interrupt = crate::interrupt::typelevel::$irq;
        }

        impl SealedInstance for peripherals::$inst {
            fn regs() -> crate::pac::jpeg::Jpeg {
                crate::pac::$inst
            }
        }
    };
);

dma_trait!(DmaIn, Instance);
dma_trait!(DmaOut, Instance);
