#![no_std]
#![no_main]

//! LCD example for the STM32N6570-DK.
//!
//! Drives the on-board 5" RK050HR18C-B01 panel (800x480, 24-bit parallel RGB) via LTDC
//! and renders through an `embedded_graphics` `DrawTarget`-backed framebuffer.
//!
//! Uses double buffering in AXISRAM3..6 to avoid tearing: the CPU draws into the back
//! buffer while LTDC is scanning out the front buffer, then they swap at vblank.
//!
//! Notes:
//! - `LCD_R3` (PB4, NJTRST) and `LCD_R5` (PA15, JTDI) are consumed by LTDC. SWD and RTT
//!   continue to work, so flashing + `defmt-rtt` are unaffected.
//! - Frame time is dominated by the software fill + `embedded-graphics` drawing at
//!   800 MHz CPU. For higher framerates the N6's Chrom-ART (DMA2D) accelerator would be
//!   the next step — it can do SRAM fills and blits in parallel with the CPU.

#[path = "../framebuffer.rs"]
mod framebuffer;
#[path = "../rk050hr18c.rs"]
mod rk050hr18c;

use core::fmt::Write as _;

use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::dma2d::{self, Dma2d};
use embassy_stm32::ltdc::{self, Ltdc, LtdcLayer, LtdcLayerConfig, PixelFormat};
use embassy_stm32::rcc::mux::Ltdcsel;
use embassy_stm32::rcc::{CpuClk, IcConfig, Icint, Icsel, Pll, Plldivm, Pllpdiv, Pllsel, SysClk};
use embassy_stm32::{Config, bind_interrupts, pac, peripherals};
use embassy_time::Instant;
use embedded_graphics::mono_font::ascii::{FONT_6X10, FONT_10X20};
use embedded_graphics::mono_font::{MonoTextStyle, MonoTextStyleBuilder};
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::pixelcolor::raw::RawU16;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{Circle, PrimitiveStyle, PrimitiveStyleBuilder, Rectangle, Triangle};
use embedded_graphics::text::{Alignment, Baseline, Text, TextStyle, TextStyleBuilder};
use embedded_graphics_core::geometry::Dimensions;
use heapless::String;
use {defmt_rtt as _, panic_probe as _};

use crate::framebuffer::Framebuffer;
use crate::rk050hr18c::{HEIGHT, LTDC_CONFIG, Rk050Hr18c, WIDTH};

bind_interrupts!(struct Irqs {
    LTDC_LO => ltdc::InterruptHandler<peripherals::LTDC>;
    DMA2D => dma2d::InterruptHandler<peripherals::DMA2D>;
});

// Two framebuffers, each 800*480*2 = 750 KB.
//   fb0 lives in the lower 768 KB of AXISRAM2 — single bank, no DMA2D cross-bank
//        straddle.
//   fb1 lives at AXISRAM3 start; 750 KB straddles one bank boundary into AXISRAM4
//        (unavoidable — no single bank except AXISRAM2 is big enough).
// AXISRAM5..6 stays free; that's where a third buffer would go for triple buffering.
const FB0_BASE: usize = 0x3410_0000;
const FB1_BASE: usize = 0x3420_0000;
const FB_PIXELS: usize = WIDTH as usize * HEIGHT as usize;

const BG: Rgb565 = Rgb565::new(2, 4, 6);
const TITLE_BG: Rgb565 = Rgb565::new(24, 6, 10);
const FG: Rgb565 = Rgb565::WHITE;

/// Rectangles smaller than this go through the CPU fast path — DMA2D setup overhead
/// (~20 register writes + completion poll) beats a software fill only for larger areas.
const DMA2D_MIN_PIXELS: u32 = 256;

/// Thin `DrawTarget` wrapper over a `Framebuffer` that offloads `fill_solid` / `clear`
/// to DMA2D (register-to-memory mode) in blocking mode. `draw_iter` and
/// `fill_contiguous` still go to the CPU path — DMA2D can't help with per-pixel or
/// arbitrary-stream work (text glyphs, circle scanlines, etc. fall under these).
struct Dma2dFb<'a, 'fb>(&'fb mut Framebuffer<'a>);

impl<'a, 'fb> Dma2dFb<'a, 'fb> {
    /// Raw slice + stride passthrough, for direct per-pixel writes (e.g. the gradient).
    fn pixels_mut(&mut self) -> (&mut [u16], u16) {
        self.0.pixels_mut()
    }
}

impl OriginDimensions for Dma2dFb<'_, '_> {
    fn size(&self) -> Size {
        self.0.size()
    }
}

impl DrawTarget for Dma2dFb<'_, '_> {
    type Color = Rgb565;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        self.0.draw_iter(pixels)
    }

    fn fill_contiguous<I>(&mut self, area: &Rectangle, colors: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Self::Color>,
    {
        self.0.fill_contiguous(area, colors)
    }

    fn fill_solid(&mut self, area: &Rectangle, color: Self::Color) -> Result<(), Self::Error> {
        let rect = area.intersection(&self.0.bounding_box());
        if rect.size.width == 0 || rect.size.height == 0 {
            return Ok(());
        }
        let pixels = rect.size.width * rect.size.height;
        if pixels < DMA2D_MIN_PIXELS {
            return self.0.fill_solid(area, color);
        }

        let fb_ptr = self.0.as_ptr() as *mut u16;
        let stride = self.size().width as u16;
        let raw = RawU16::from(color).into_inner();
        dma2d_r2m_fill_rgb565(
            fb_ptr,
            stride,
            rect.top_left.x as u16,
            rect.top_left.y as u16,
            rect.size.width as u16,
            rect.size.height as u16,
            raw,
        );
        Ok(())
    }

    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
        let fb_ptr = self.0.as_ptr() as *mut u16;
        let stride = self.size().width as u16;
        let w = self.size().width as u16;
        let h = self.size().height as u16;
        let raw = RawU16::from(color).into_inner();
        dma2d_r2m_fill_rgb565(fb_ptr, stride, 0, 0, w, h, raw);
        Ok(())
    }
}

/// Blocking DMA2D register-to-memory fill of an RGB565 rectangle. Polls `CR.START`.
/// Assumes the DMA2D peripheral has already been enabled (`Dma2d::new` does this) and
/// its master attributes have been promoted for SRAM writes.
fn dma2d_r2m_fill_rgb565(fb_ptr: *mut u16, stride: u16, x: u16, y: u16, width: u16, height: u16, color_rgb565: u16) {
    use pac::dma2d::vals;

    let r = pac::DMA2D;
    while r.cr().read().start() == vals::CrStart::Start {}

    let offset_bytes = (y as usize * stride as usize + x as usize) * 2;
    let dst = (fb_ptr as usize + offset_bytes) as u32;

    // Output pixel format = RGB565 (variant value 2 per RM0486 §21).
    r.opfccr().write(|reg| reg.set_cm(vals::OpfccrCm::from_bits(0b0010)));
    // Output color (low 16 bits carry the RGB565 word on dma2d_v1).
    r.ocolr().write(|reg| reg.0 = color_rgb565 as u32);
    // Output address + line offset in pixels.
    r.omar().write(|reg| reg.set_ma(dst));
    r.oor().write(|reg| reg.set_lo(stride - width));
    // Number of lines + pixels per line.
    r.nlr().write(|reg| {
        reg.set_pl(width);
        reg.set_nl(height);
    });
    // Kick off in register-to-memory mode.
    r.cr().write(|reg| {
        reg.set_mode(vals::Mode::RegisterToMemory);
        reg.set_start(vals::CrStart::Start);
    });
    // Spin until START self-clears (hardware drops it on completion).
    while r.cr().read().start() == vals::CrStart::Start {}
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    // PLL1 oscillator: HSI 64 MHz / DIVM=4 = 16 MHz ref, × DIVN=50 → 800 MHz VCO.
    //   IC1 div1 → 800 MHz CPU (M55 max).
    //   IC2/6/11 div4 → 200 MHz system bus. The embassy N6 defaults of AHB=Div2 and
    //   APB=Div1 give HCLK=100 MHz and PCLK=100 MHz — safe with the stock embassy-time
    //   TIM driver configuration. Cranking APB higher breaks the time driver's tick
    //   rate assumption on this chip.
    config.rcc.pll1 = Some(Pll::Oscillator {
        source: Pllsel::Hsi,
        divm: Plldivm::Div4,
        fractional: 0,
        divn: 50,
        divp1: Pllpdiv::Div1,
        divp2: Pllpdiv::Div1,
    });
    config.rcc.ic1 = Some(IcConfig {
        source: Icsel::Pll1,
        divider: Icint::Div1,
    });
    let sys_ic = IcConfig {
        source: Icsel::Pll1,
        divider: Icint::Div4,
    };
    config.rcc.ic2 = Some(sys_ic);
    config.rcc.ic6 = Some(sys_ic);
    config.rcc.ic11 = Some(sys_ic);
    config.rcc.cpu = CpuClk::Ic1;
    config.rcc.sys = SysClk::Ic2;

    // PLL4 bypass → HSI 64 MHz. IC16 = 32 MHz drives the LTDC pixel clock.
    config.rcc.pll4 = Some(Pll::Bypass { source: Pllsel::Hsi });
    config.rcc.ic16 = Some(IcConfig {
        source: Icsel::Pll4,
        divider: Icint::Div2,
    });
    config.rcc.mux.ltdcsel = Ltdcsel::Ic16;

    let p = embassy_stm32::init(config);
    info!("stm32n6 lcd example starting");

    enable_all_sram();
    promote_ltdc_and_dma2d_masters_to_secure();

    let mut panel = Rk050Hr18c::new(p.PE1, p.PQ3, p.PQ6);
    panel.power_on().await;

    // Full 24-bit RGB888 LTDC pin mapping from UM3300 §8.3.
    let mut ltdc = Ltdc::<_, ltdc::Rgb888>::new_with_pins(
        p.LTDC, Irqs, p.PB13, // CLK
        p.PB14, // HSYNC
        p.PE11, // VSYNC
        p.PG13, // DE
        p.PG15, p.PA7, p.PB2, p.PG6, p.PH3, p.PH6, p.PA8, p.PA2, // B0..B7
        p.PG12, p.PG1, p.PA1, p.PA0, p.PB15, p.PB12, p.PB11, p.PG8, // G0..G7
        p.PG0, p.PD9, p.PD15, p.PB4, p.PH4, p.PA15, p.PG11, p.PD8, // R0..R7
    );
    ltdc.init(&LTDC_CONFIG);

    // Construct Dma2d just for the side effects — clock enable + reset + NVIC wiring.
    // We drive DMA2D registers directly from the Framebuffer wrapper for sync fills.
    let _dma2d = Dma2d::new(p.DMA2D, Irqs);

    let layer_config = LtdcLayerConfig {
        pixel_format: PixelFormat::RGB565,
        layer: LtdcLayer::Layer1,
        window_x0: 0,
        window_x1: WIDTH,
        window_y0: 0,
        window_y1: HEIGHT,
    };

    // Safety: FB0 / FB1 live in AXISRAM3..6 (non-secure alias). `memory.x` doesn't
    // reference those banks, and each slice is only accessed through its own Framebuffer.
    let fb0_slice: &'static mut [u16] = unsafe { core::slice::from_raw_parts_mut(FB0_BASE as *mut u16, FB_PIXELS) };
    let fb1_slice: &'static mut [u16] = unsafe { core::slice::from_raw_parts_mut(FB1_BASE as *mut u16, FB_PIXELS) };
    let mut fb0 = Framebuffer::new(fb0_slice, WIDTH, HEIGHT);
    let mut fb1 = Framebuffer::new(fb1_slice, WIDTH, HEIGHT);
    fb0.fill(BG);
    fb1.fill(BG);

    ltdc.init_layer(&layer_config, None);
    ltdc.init_buffer(LtdcLayer::Layer1, fb0.as_ptr());
    pac::LTDC.srcr().write(|w| w.set_imr(pac::ltdc::vals::Imr::Reload));

    // LTDC is showing fb0 after the IMR reload; we always draw into fbs[back_idx].
    let mut fbs = [fb0, fb1];
    let mut back_idx = 1;

    // Give text styles an opaque background color so embedded-graphics draws glyphs via
    // `fill_contiguous` (row-major, batched) instead of per-pixel `draw_iter`. The
    // background colors match what the text sits on visually — so they cost us a redraw
    // of the glyph cell but no visible change vs transparent backgrounds.
    let title_font = MonoTextStyleBuilder::new()
        .font(&FONT_10X20)
        .text_color(FG)
        .background_color(TITLE_BG)
        .build();
    let small_font = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(Rgb565::new(28, 56, 28))
        .background_color(BG)
        .build();
    let right_align = TextStyleBuilder::new()
        .alignment(Alignment::Right)
        .baseline(Baseline::Middle)
        .build();
    let left_mid = TextStyleBuilder::new()
        .alignment(Alignment::Left)
        .baseline(Baseline::Middle)
        .build();

    let boot = Instant::now();
    let mut ball_x: i32 = 120;
    let mut ball_y: i32 = 260;
    let mut dx: i32 = 3;
    let mut dy: i32 = 2;
    const BALL_R: u32 = 18;
    let mut frame: u32 = 0;
    let mut fps_window_start = Instant::now();
    let mut fps_window_frames: u32 = 0;

    loop {
        let frame_start = Instant::now();

        let mut back = Dma2dFb(&mut fbs[back_idx]);
        let t_clear = Instant::now();
        back.clear(BG).unwrap();
        let clear_us = t_clear.elapsed().as_micros();

        let t_render = Instant::now();
        render_frame(
            &mut back,
            &title_font,
            &small_font,
            left_mid,
            right_align,
            boot,
            ball_x,
            ball_y,
            BALL_R,
            frame,
        );
        let render_us = t_render.elapsed().as_micros();

        // Flip via the HAL: writes CFBAR, requests a vblank reload, and .awaits the
        // LTDC reload interrupt. Precise vblank sync and no wasted sleep — the executor
        // is free to run other tasks while we wait.
        cortex_m::asm::dsb();
        let new_front_ptr = fbs[back_idx].as_ptr();
        let t_flip = Instant::now();
        ltdc.set_buffer(LtdcLayer::Layer1, new_front_ptr).await.unwrap();
        let flip_us = t_flip.elapsed().as_micros();
        back_idx = 1 - back_idx;

        let frame_us = frame_start.elapsed().as_micros();
        fps_window_frames += 1;
        let elapsed_ms = fps_window_start.elapsed().as_millis();
        if elapsed_ms >= 1000 {
            let fps_x100 = (fps_window_frames as u64 * 100_000) / elapsed_ms;
            info!(
                "fps={}.{:02}  clear={}us render={}us flip={}us total={}us",
                fps_x100 / 100,
                fps_x100 % 100,
                clear_us,
                render_us,
                flip_us,
                frame_us,
            );
            fps_window_start = Instant::now();
            fps_window_frames = 0;
        }

        ball_x += dx;
        ball_y += dy;
        let lo_y: i32 = 200;
        let hi_y: i32 = HEIGHT as i32 - 64 - (BALL_R as i32) - 8;
        if ball_x - (BALL_R as i32) <= 0 || ball_x + (BALL_R as i32) >= WIDTH as i32 {
            dx = -dx;
        }
        if ball_y <= lo_y || ball_y >= hi_y {
            dy = -dy;
        }
        frame = frame.wrapping_add(1);
    }
}

#[allow(clippy::too_many_arguments)]
fn render_frame(
    fb: &mut Dma2dFb<'_, '_>,
    title_font: &MonoTextStyle<'_, Rgb565>,
    small_font: &MonoTextStyle<'_, Rgb565>,
    left_mid: TextStyle,
    right_align: TextStyle,
    boot: Instant,
    ball_x: i32,
    ball_y: i32,
    ball_r: u32,
    frame: u32,
) {
    // NB: the full-screen BG fill is done by DMA2D before this function is called.
    Rectangle::new(Point::zero(), Size::new(WIDTH as u32, 48))
        .into_styled(PrimitiveStyle::with_fill(TITLE_BG))
        .draw(fb)
        .unwrap();
    Text::with_text_style(
        "embassy-stm32  \u{00b7}  STM32N6 LTDC demo",
        Point::new(16, 24),
        *title_font,
        left_mid,
    )
    .draw(fb)
    .unwrap();

    let mut buf: String<48> = String::new();
    let secs = boot.elapsed().as_secs();
    let _ = write!(
        buf,
        "uptime {:02}:{:02}:{:02}   frame {}",
        secs / 3600,
        (secs / 60) % 60,
        secs % 60,
        frame
    );
    Text::with_text_style(&buf, Point::new(WIDTH as i32 - 16, 24), *title_font, right_align)
        .draw(fb)
        .unwrap();

    let row_y = 96;
    Circle::new(Point::new(60, row_y), 64)
        .into_styled(PrimitiveStyle::with_fill(Rgb565::RED))
        .draw(fb)
        .unwrap();
    Triangle::new(
        Point::new(210, row_y + 64),
        Point::new(274, row_y + 64),
        Point::new(242, row_y),
    )
    .into_styled(PrimitiveStyle::with_fill(Rgb565::GREEN))
    .draw(fb)
    .unwrap();
    Rectangle::new(Point::new(360, row_y), Size::new(80, 64))
        .into_styled(PrimitiveStyle::with_fill(Rgb565::BLUE))
        .draw(fb)
        .unwrap();
    let cx = 560;
    let cy = row_y + 32;
    Triangle::new(
        Point::new(cx, cy - 32),
        Point::new(cx - 32, cy),
        Point::new(cx, cy + 32),
    )
    .into_styled(PrimitiveStyle::with_fill(Rgb565::YELLOW))
    .draw(fb)
    .unwrap();
    Triangle::new(
        Point::new(cx, cy - 32),
        Point::new(cx + 32, cy),
        Point::new(cx, cy + 32),
    )
    .into_styled(PrimitiveStyle::with_fill(Rgb565::new(31, 40, 0)))
    .draw(fb)
    .unwrap();
    Rectangle::new(Point::new(680, row_y), Size::new(80, 64))
        .into_styled(
            PrimitiveStyleBuilder::new()
                .stroke_color(Rgb565::CYAN)
                .stroke_width(4)
                .fill_color(BG)
                .build(),
        )
        .draw(fb)
        .unwrap();

    let label_y = row_y + 80;
    for (label, x) in [
        ("circle", 62),
        ("triangle", 228),
        ("rect", 380),
        ("diamond", 548),
        ("outline", 704),
    ] {
        Text::new(label, Point::new(x, label_y), *small_font).draw(fb).unwrap();
    }

    Circle::new(Point::new(ball_x, ball_y), ball_r * 2)
        .into_styled(PrimitiveStyle::with_fill(Rgb565::MAGENTA))
        .draw(fb)
        .unwrap();

    // Gradient strip drawn directly via the raw pixel slice — embedded_graphics' per-
    // Line overhead dominates on an 800-wide strip.
    let strip_top = HEIGHT as i32 - 64;
    let strip_h = 40u32;
    let (pixels, stride) = fb.pixels_mut();
    let stride = stride as usize;
    for x in 0..WIDTH as usize {
        let t = x as u32 * 192 / WIDTH as u32;
        let (r, g, b) = hue_to_rgb565(t);
        let raw = RawU16::from(Rgb565::new(r, g, b)).into_inner();
        for y in (strip_top as usize)..(strip_top as usize + strip_h as usize) {
            pixels[y * stride + x] = raw;
        }
    }
    let div_raw = RawU16::from(Rgb565::new(12, 24, 12)).into_inner();
    let y = (strip_top - 2) as usize;
    for x in 0..WIDTH as usize {
        pixels[y * stride + x] = div_raw;
    }

    Text::new(
        "RGB565 \u{00b7} 800x480 \u{00b7} RK050HR18C-B01",
        Point::new(16, HEIGHT as i32 - 12),
        *small_font,
    )
    .draw(fb)
    .unwrap();
}

/// Enable run-mode clocks for every AXISRAM / AHBSRAM bank. embassy-stm32's N6 init only
/// sets the low-power gates, which leaves AXISRAM3-6 un-clocked for bus masters.
fn enable_all_sram() {
    pac::RCC.memenr().modify(|w| {
        w.set_axisram1en(true);
        w.set_axisram2en(true);
        w.set_axisram3en(true);
        w.set_axisram4en(true);
        w.set_axisram5en(true);
        w.set_axisram6en(true);
        w.set_ahbsram1en(true);
        w.set_ahbsram2en(true);
        w.set_bkpsramen(true);
    });
}

/// Promote the LTDC and DMA2D AXI master attributes so the RISAF default region
/// (secure + CID=1 + privileged — RM0486 §7.4.4) accepts their framebuffer reads /
/// writes. See RM0486 §6.3.2 Table 22 for the master-index / RISUP-index mapping:
///   - DMA2D  → RIMU master 8,  RISUP 101 → SECCFGR3 bit 5
///   - LTDC_L1 → RIMU master 10, RISUP 103 → SECCFGR3 bit 7
///   - LTDC_L2 → RIMU master 11, RISUP 104 → SECCFGR3 bit 8 (in SECCFGR3)
fn promote_ltdc_and_dma2d_masters_to_secure() {
    pac::RIFSC.risc_seccfgr(3).modify(|w| {
        w.set_cfg(5, true); // RISUP 101 — DMA2D
        w.set_cfg(7, true); // RISUP 103 — LTDC_L1
        w.set_cfg(8, true); // RISUP 104 — LTDC_L2
    });
    pac::RIFSC.risc_privcfgr(3).modify(|w| {
        w.set_cfg(5, true);
        w.set_cfg(7, true);
        w.set_cfg(8, true);
    });
    for master in [8usize, 10, 11] {
        pac::RIFSC.rimc_attr(master).modify(|w| {
            w.set_mcid(1);
            w.set_msec(true);
            w.set_mpriv(true);
        });
    }
}

/// 6-step hue sweep across 0..192, returning 5:6:5-bit channels.
fn hue_to_rgb565(t: u32) -> (u8, u8, u8) {
    let seg = (t / 32) % 6;
    let phase = (t % 32) as u8;
    let up5 = phase >> 1;
    let down5 = 31 - up5;
    let up6 = phase;
    let down6 = 63 - phase * 2;
    match seg {
        0 => (31, up6, 0),
        1 => (down5, 63, 0),
        2 => (0, 63, up5),
        3 => (0, down6.min(63), 31),
        4 => (up5, 0, 31),
        _ => (31, 0, down5),
    }
}
