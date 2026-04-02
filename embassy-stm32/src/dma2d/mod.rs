//! DMA2D Chrom-ART Graphics Accelerator

use core::future::poll_fn;
use core::marker::PhantomData;
use core::ptr::NonNull;
use core::task::Poll;

use embassy_hal_internal::PeripheralType;
use embassy_sync::waitqueue::AtomicWaker;
use stm32_metapac::dma2d::vals;

use crate::interrupt::typelevel::Interrupt;
use crate::peripherals::{self};
use crate::rcc::{self, RccPeripheral};
use crate::{Peri, interrupt};

static DMA2D_WAKER: AtomicWaker = AtomicWaker::new();

/// DMA2D Error
#[non_exhaustive]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// Transfer Error
    TransferError,

    /// Configuration Error
    ConfigError,
}

/// DMA2D Buffer kind
#[derive(Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum BufferKind {
    /// Foreground buffer
    Foreground,
    /// Background buffer
    Background,
    /// Output buffer
    Output,
}

/// DMA2D Region of a [`Buffer2D`]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[allow(unused)]
pub struct Region2D {
    /// Pointer to the first pixel of the region
    pub ptr: NonNull<u8>,
    /// Pixel color format
    format: PixelFormat,
    /// Number of pixels to skip after each line
    pub line_offset: u16,
    /// Number of pixels per line
    pub pixels_per_line: u16,
    /// Number of lines
    pub lines: u16,
}

/// DMA2D Buffer
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Buffer2D {
    /// Buffer base pointer
    pub ptr: NonNull<u8>,
    /// Pixel format of the buffer
    pub format: PixelFormat,
    /// Line stride of the buffer
    pub stride: u16,
    /// Width of the buffer
    pub width: u16,
    /// Height of the buffer
    pub height: u16,
}

impl Buffer2D {
    /// Create a new buffer descriptor over an existing pointer
    pub fn new(ptr: *mut u8, format: PixelFormat, stride: u16, width: u16, height: u16) -> Self {
        Self {
            ptr: unsafe { NonNull::new_unchecked(ptr) },
            format,
            stride,
            width,
            height,
        }
    }

    /// Get a region of a 2D Buffer specified by origin x, y and width, height
    pub fn region(&self, x: u16, y: u16, w: u16, h: u16) -> Region2D {
        assert!(x + w <= self.width);
        assert!(y + h <= self.height);

        let bytes_per_pixel = self.format.bytes_per_pixel().expect("bytes per pixel");
        let stride_bytes = self.stride as usize * bytes_per_pixel;
        let offset_bytes = y as usize * stride_bytes + x as usize * bytes_per_pixel;

        let ptr = unsafe { NonNull::new_unchecked(self.ptr.as_ptr().add(offset_bytes)) };

        Region2D {
            ptr,
            format: self.format,
            line_offset: self.stride - w,
            pixels_per_line: w,
            lines: h,
        }
    }
}

/// DMA2D Pixel Format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum PixelFormat {
    /// ARGB8888 24-bit color with alpha
    Argb8888 = 0b0000,
    /// RGB888 24-bit color
    Rgb888 = 0b0001,
    /// RGB565 16-bit color
    Rgb565 = 0b0010,
    /// ARGB1555
    Argb1555 = 0b0011,
    /// ARGB4444
    Argb4444 = 0b0100,
    /// 8 bit luminance
    L8 = 0b0101,
    /// 4-bit alpha, 4 bit luminance
    AL44 = 0b0110,
    /// 8-bit alpha, 8 bit luminance
    AL88 = 0b0111,
    /// 4-bit luminance
    L4 = 0b1000,
    /// 8-bit alpha
    A8 = 0b1001,
    /// 4-bit alpha
    A4 = 0b1010,
    /// YCbCr YUV
    #[cfg(dma2d_v2)]
    YCbCr = 0b1011,
}

impl PixelFormat {
    /// Number of 4 bit nibbles per pixel
    pub const fn nibbles_per_pixel(&self) -> usize {
        match self {
            PixelFormat::Argb8888 => 8,
            PixelFormat::Rgb888 => 8,
            PixelFormat::Rgb565 => 4,
            PixelFormat::Argb1555 => 4,
            PixelFormat::Argb4444 => 4,
            PixelFormat::L8 => 2,
            PixelFormat::AL44 => 2,
            PixelFormat::AL88 => 4,
            PixelFormat::L4 => 1,
            PixelFormat::A8 => 2,
            PixelFormat::A4 => 1,
            #[cfg(dma2d_v2)]
            PixelFormat::YCbCr => 4,
        }
    }

    /// Number of bytes per pixel. Panics if pixel format size is 4 bits
    pub const fn bytes_per_pixel(&self) -> Option<usize> {
        match self {
            PixelFormat::Argb8888 => Some(4),
            PixelFormat::Rgb888 => Some(4),
            PixelFormat::Rgb565 => Some(2),
            PixelFormat::Argb1555 => Some(2),
            PixelFormat::Argb4444 => Some(2),
            PixelFormat::L8 => Some(1),
            PixelFormat::AL44 => Some(1),
            PixelFormat::AL88 => Some(2),
            PixelFormat::A8 => Some(1),
            #[cfg(dma2d_v2)]
            PixelFormat::YCbCr => Some(2),
            _ => None,
        }
    }
}

impl Into<vals::FgpfccrCm> for PixelFormat {
    fn into(self) -> vals::FgpfccrCm {
        match self {
            PixelFormat::Argb8888 => vals::FgpfccrCm::ARGB8888,
            PixelFormat::Rgb888 => vals::FgpfccrCm::RGB888,
            PixelFormat::Rgb565 => vals::FgpfccrCm::RGB565,
            PixelFormat::Argb1555 => vals::FgpfccrCm::ARGB1555,
            PixelFormat::Argb4444 => vals::FgpfccrCm::ARGB4444,
            PixelFormat::L8 => vals::FgpfccrCm::L8,
            PixelFormat::AL44 => vals::FgpfccrCm::AL44,
            PixelFormat::AL88 => vals::FgpfccrCm::AL88,
            PixelFormat::L4 => vals::FgpfccrCm::L4,
            PixelFormat::A8 => vals::FgpfccrCm::A8,
            PixelFormat::A4 => vals::FgpfccrCm::A4,
            #[cfg(dma2d_v2)]
            PixelFormat::YCbCr => vals::FgpfccrCm::YCB_CR,
        }
    }
}

impl Into<vals::BgpfccrCm> for PixelFormat {
    fn into(self) -> vals::BgpfccrCm {
        match self {
            PixelFormat::Argb8888 => vals::BgpfccrCm::ARGB8888,
            PixelFormat::Rgb888 => vals::BgpfccrCm::RGB888,
            PixelFormat::Rgb565 => vals::BgpfccrCm::RGB565,
            PixelFormat::Argb1555 => vals::BgpfccrCm::ARGB1555,
            PixelFormat::Argb4444 => vals::BgpfccrCm::ARGB4444,
            PixelFormat::L8 => vals::BgpfccrCm::L8,
            PixelFormat::AL44 => vals::BgpfccrCm::AL44,
            PixelFormat::AL88 => vals::BgpfccrCm::AL88,
            PixelFormat::L4 => vals::BgpfccrCm::L4,
            PixelFormat::A8 => vals::BgpfccrCm::A8,
            PixelFormat::A4 => vals::BgpfccrCm::A4,
            #[cfg(dma2d_v2)]
            _ => panic!("YCbCr pixel format not supported for background buffer"),
        }
    }
}

impl Into<vals::OpfccrCm> for PixelFormat {
    fn into(self) -> vals::OpfccrCm {
        match self {
            PixelFormat::Argb8888 => vals::OpfccrCm::ARGB8888,
            PixelFormat::Rgb888 => vals::OpfccrCm::RGB888,
            PixelFormat::Rgb565 => vals::OpfccrCm::RGB565,
            PixelFormat::Argb1555 => vals::OpfccrCm::ARGB1555,
            PixelFormat::Argb4444 => vals::OpfccrCm::ARGB4444,
            _ => panic!("Selected output pixel format not supported"),
        }
    }
}

/// Input buffer alpha Mode
#[derive(Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum AlphaMode {
    /// Do not modify alpha
    NoModify,
    /// Replace alpha
    Replace(u8),
    /// Multiply alpha
    Multiply(u8),
}

impl Into<vals::FgpfccrAm> for AlphaMode {
    fn into(self) -> vals::FgpfccrAm {
        match self {
            AlphaMode::NoModify => vals::FgpfccrAm::NO_MODIFY,
            AlphaMode::Replace(_) => vals::FgpfccrAm::REPLACE,
            AlphaMode::Multiply(_) => vals::FgpfccrAm::MULTIPLY,
        }
    }
}

impl Into<vals::BgpfccrAm> for AlphaMode {
    fn into(self) -> vals::BgpfccrAm {
        match self {
            AlphaMode::NoModify => vals::BgpfccrAm::NO_MODIFY,
            AlphaMode::Replace(_) => vals::BgpfccrAm::REPLACE,
            AlphaMode::Multiply(_) => vals::BgpfccrAm::MULTIPLY,
        }
    }
}

/// Color Configuration
pub struct ColorConfig {
    /// Pixel format
    pub pixel_format: PixelFormat,
    /// Alpha mode
    pub alpha_mode: AlphaMode,
    /// Invert alpha
    #[cfg(dma2d_v2)]
    pub alpha_invert: bool,
    /// Swap red and blue channels
    #[cfg(dma2d_v2)]
    pub swap_red_blue: bool,
    /// Swap output bytes
    #[cfg(dma2d_v2)]
    pub swap_bytes: bool,
}

impl Default for ColorConfig {
    fn default() -> Self {
        Self {
            pixel_format: PixelFormat::Argb8888,
            alpha_mode: AlphaMode::NoModify,
            #[cfg(dma2d_v2)]
            alpha_invert: false,
            #[cfg(dma2d_v2)]
            swap_red_blue: false,
            #[cfg(dma2d_v2)]
            swap_bytes: false,
        }
    }
}

/// DMA2D Peripheral
pub struct Dma2d<'d, T: Instance> {
    _peri: PhantomData<&'d mut T>,
}

impl<'d, T: Instance> Dma2d<'d, T> {
    /// Create a Dma2d peripheral
    pub fn new(
        _peri: Peri<'d, T>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
    ) -> Self {
        rcc::enable_and_reset::<T>();

        Self { _peri: PhantomData }
    }

    /// Configure a Buffer2D
    pub fn set_buffer(&self, kind: BufferKind, buffer: &Buffer2D) {
        match kind {
            BufferKind::Foreground => {
                T::regs().fgmar().write(|w| w.set_ma(buffer.ptr.as_ptr() as u32));
                T::regs().fgor().write(|w| w.set_lo(buffer.stride - buffer.width));
            }
            BufferKind::Background => {
                T::regs().bgmar().write(|w| w.set_ma(buffer.ptr.as_ptr() as u32));
                T::regs().bgor().write(|w| w.set_lo(buffer.stride - buffer.width));
            }
            BufferKind::Output => {
                T::regs().omar().write(|w| w.set_ma(buffer.ptr.as_ptr() as u32));
                T::regs().oor().write(|w| w.set_lo(buffer.stride - buffer.width));
                T::regs().nlr().write(|w| {
                    w.set_pl(buffer.width);
                    w.set_nl(buffer.height);
                });
            }
        }
    }

    /// Configure a Region2D
    pub fn set_region(&mut self, kind: BufferKind, region: &Region2D) {
        match kind {
            BufferKind::Foreground => {
                T::regs().fgmar().write(|w| w.set_ma(region.ptr.as_ptr() as u32));
                T::regs().fgor().write(|w| w.set_lo(region.line_offset));
            }
            BufferKind::Background => {
                T::regs().bgmar().write(|w| w.set_ma(region.ptr.as_ptr() as u32));
                T::regs().bgor().write(|w| w.set_lo(region.line_offset));
            }
            BufferKind::Output => {
                T::regs().omar().write(|w| w.set_ma(region.ptr.as_ptr() as u32));
                T::regs().oor().write(|w| w.set_lo(region.line_offset));
                T::regs().nlr().write(|w| {
                    w.set_pl(region.pixels_per_line);
                    w.set_nl(region.lines);
                });
            }
        }
    }

    /// Set the color configuration for a buffer
    pub fn set_color_config(&mut self, kind: BufferKind, config: &ColorConfig) {
        match kind {
            BufferKind::Foreground => {
                T::regs().fgpfccr().modify(|w| {
                    w.set_cm(config.pixel_format.into());
                    #[cfg(dma2d_v2)]
                    w.set_ai(match config.alpha_invert {
                        true => vals::FgpfccrAi::INVERTED_ALPHA,
                        false => vals::FgpfccrAi::REGULAR_ALPHA,
                    });
                    #[cfg(dma2d_v2)]
                    w.set_rbs(match config.swap_red_blue {
                        true => vals::FgpfccrRbs::SWAP,
                        false => vals::FgpfccrRbs::REGULAR,
                    });
                    w.set_am(config.alpha_mode.into());

                    w.set_alpha(match config.alpha_mode {
                        AlphaMode::Replace(alpha) | AlphaMode::Multiply(alpha) => alpha,
                        _ => 0,
                    });
                });
            }
            BufferKind::Background => {
                T::regs().bgpfccr().modify(|w| {
                    w.set_cm(config.pixel_format.into());
                    #[cfg(dma2d_v2)]
                    w.set_ai(match config.alpha_invert {
                        true => vals::BgpfccrAi::INVERTED_ALPHA,
                        false => vals::BgpfccrAi::REGULAR_ALPHA,
                    });
                    #[cfg(dma2d_v2)]
                    w.set_rbs(match config.swap_red_blue {
                        true => vals::BgpfccrRbs::SWAP,
                        false => vals::BgpfccrRbs::REGULAR,
                    });
                    w.set_am(config.alpha_mode.into());

                    w.set_alpha(match config.alpha_mode {
                        AlphaMode::Replace(alpha) | AlphaMode::Multiply(alpha) => alpha,
                        _ => 0,
                    });
                });
            }
            BufferKind::Output => {
                T::regs().opfccr().modify(|w| {
                    w.set_cm(config.pixel_format.into());
                    #[cfg(dma2d_v2)]
                    w.set_ai(match config.alpha_invert {
                        true => vals::OpfccrAi::INVERTED_ALPHA,
                        false => vals::OpfccrAi::REGULAR_ALPHA,
                    });
                    #[cfg(dma2d_v2)]
                    w.set_rbs(match config.swap_red_blue {
                        true => vals::OpfccrRbs::SWAP,
                        false => vals::OpfccrRbs::REGULAR,
                    });
                    #[cfg(dma2d_v2)]
                    w.set_sb(match config.swap_bytes {
                        true => vals::Sb::SWAP_BYTES,
                        false => vals::Sb::REGULAR,
                    });
                });
            }
        }
    }

    /// Fill the output buffer with a color into an output region
    pub async fn fill(&mut self, dest: &Region2D, color: u32) -> Result<(), Error> {
        T::regs().opfccr().modify(|w| {
            w.set_cm(dest.format.into());
        });

        self.set_region(BufferKind::Output, dest);
        #[cfg(dma2d_v2)]
        T::regs().ocolr().modify(|w| w.set_color(color));
        #[cfg(dma2d_v1)]
        T::regs().ocolr().modify(|w| w.0 = color);
        T::regs().cr().modify(|w| w.set_mode(vals::Mode::REGISTER_TO_MEMORY));
        Self::transfer().await
    }

    /// Copy a source foreground buffer to a destination output buffer
    pub async fn copy(&self, fg: &Buffer2D, output: &Buffer2D) -> Result<(), Error> {
        // Set foreground CM to set the bits per pixel to copy
        T::regs().fgpfccr().modify(|w| {
            w.set_cm(fg.format.into());
        });

        self.set_buffer(BufferKind::Foreground, fg);
        self.set_buffer(BufferKind::Output, output);
        T::regs().cr().modify(|w| w.set_mode(vals::Mode::MEMORY_TO_MEMORY));
        Self::transfer().await
    }

    /// Blit a source to a destination region using a fixed background color and alpha blending.
    ///
    /// Pixel format conversion is applied to convert between input and output buffer
    /// color configurations which must be set before calling blit using
    /// [`Dma2d::set_color_config`] on both [`BufferKind::Foreground'] and [`BufferKind::Output`]
    ///
    /// If the input buffer is A8, a fixed color may be provided in `fg_color` which
    /// will set the foreground RGB channels from the FGCOLR register.
    ///
    /// If None is provided as `bg_color`, the background defaults to black.
    #[cfg(dma2d_v2)]
    pub async fn blit(
        &mut self,
        fg: &Region2D,
        dest: &Region2D,
        fg_color: Option<u32>,
        bg_color: Option<u32>,
    ) -> Result<(), Error> {
        T::regs().fgpfccr().modify(|w| {
            w.set_cm(fg.format.into());
        });

        self.set_region(BufferKind::Foreground, fg);
        self.set_region(BufferKind::Output, dest);

        if let Some(color) = fg_color {
            // Set the foreground color for A8
            T::regs().fgcolr().modify(|w| w.0 = color);
        }

        T::regs().bgcolr().modify(|w| w.0 = bg_color.unwrap_or(0xff000000));

        T::regs()
            .cr()
            .modify(|w| w.set_mode(vals::Mode::MEMORY_TO_MEMORY_PFCBLENDING_FIXED_COLOR_BG));
        Self::transfer().await
    }

    /// Blit using a fg and bg source
    pub async fn blit_with(
        &mut self,
        fg: &Region2D,
        bg: &Region2D,
        output: &Region2D,
        fg_color: Option<u32>,
    ) -> Result<(), Error> {
        T::regs().fgpfccr().modify(|w| {
            w.set_cm(fg.format.into());
        });

        T::regs().bgpfccr().modify(|w| {
            w.set_cm(bg.format.into());
        });

        self.set_region(BufferKind::Foreground, fg);
        self.set_region(BufferKind::Background, bg);
        self.set_region(BufferKind::Output, output);

        if let Some(color) = fg_color {
            // Set the foreground color for A8
            T::regs().fgcolr().modify(|w| w.0 = color);
        }

        T::regs()
            .cr()
            .modify(|w| w.set_mode(vals::Mode::MEMORY_TO_MEMORY_PFCBLENDING));
        Self::transfer().await
    }

    /// Start a transfer and wait for the completion interrupt
    async fn transfer() -> Result<(), Error> {
        poll_fn(|cx| {
            let isr = T::regs().isr().read();

            if isr.teif() {
                T::regs().ifcr().modify(|w| w.set_cteif(vals::Cteif::CLEAR));
                return Poll::Ready(Err(Error::TransferError));
            }

            if isr.ceif() {
                T::regs().ifcr().modify(|w| w.set_cceif(vals::Cceif::CLEAR));
                return Poll::Ready(Err(Error::ConfigError));
            }

            if isr.tcif() {
                T::regs().cr().modify(|w| {
                    w.set_tcie(false);
                    w.set_teie(false);
                    w.set_ceie(false);
                });

                T::regs().ifcr().modify(|w| w.set_ctcif(vals::Ctcif::CLEAR));

                Poll::Ready(Ok(()))
            } else {
                DMA2D_WAKER.register(cx.waker());
                Self::enable_interrupts(true);
                T::regs().cr().modify(|w| {
                    // Enable transfer complete interrupt
                    w.set_tcie(true);
                    // Enable transfer error interrupt
                    w.set_teie(true);
                    w.set_ceie(true);
                    w.set_start(stm32_metapac::dma2d::vals::CrStart::START);
                });
                Poll::Pending
            }
        })
        .await
    }

    /// Enable interrupts
    fn enable_interrupts(enable: bool) {
        T::Interrupt::unpend();
        if enable {
            unsafe { T::Interrupt::enable() };
        } else {
            T::Interrupt::disable()
        }
    }
}

/// DMA2D interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        cortex_m::asm::dsb();
        Dma2d::<T>::enable_interrupts(false);
        DMA2D_WAKER.wake();
    }
}

trait SealedInstance: crate::rcc::SealedRccPeripheral {
    fn regs() -> crate::pac::dma2d::Dma2d;
}

/// DMA2D instance trait.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + RccPeripheral + 'static + Send {
    /// Interrupt for this DMA2D instance.
    type Interrupt: interrupt::typelevel::Interrupt;
}

foreach_interrupt!(
    ($inst:ident, dma2d, DMA2D, GLOBAL, $irq:ident) => {
        impl Instance for peripherals::$inst {
            type Interrupt = crate::interrupt::typelevel::$irq;
        }

        impl SealedInstance for peripherals::$inst {
            fn regs() -> crate::pac::dma2d::Dma2d {
                crate::pac::$inst
            }
        }
    };
);
