#![no_std]
#![no_main]

//! STM32N6570-DK NeoChrom (GPU2D) + embedded-3dgfx + embedded-gui Graphics Engine.
//!
//! Drives the on-board 5" 800x480 RK050HR18C panel via LTDC with:
//! - Rock-solid 25.0 MHz pixel clock via PLL4 oscillator (no snow noise, no stripes!)
//! - NeoChrom GPU2D hardware-accelerated glassmorphism panels, cyber-grid, & 20-band audio spectrum
//! - `embedded-3dgfx` real-time 3D dual-polyhedron rotation (Cube + Octahedron) with Z-buffering in AXISRAM1
//! - `embedded-gui` live animated progress bar with smooth cubic easing
//! - Double-buffered tear-free rendering with VBlank synchronization

#[path = "../gfx_framebuffer.rs"]
mod framebuffer;
#[path = "../nema_sink.rs"]
mod nema_sink;
#[path = "../rk050hr18c.rs"]
mod rk050hr18c;

use core::fmt::Write as _;

use cortex_m::peripheral::{CPUID, MPU, SCB};
use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
#[cfg(not(feature = "stub-gpu2d"))]
use embassy_stm32::gpu2d;
use embassy_stm32::ltdc::{self, Ltdc, LtdcLayer, LtdcLayerConfig, PixelFormat};
use embassy_stm32::rcc::mux::Ltdcsel;
use embassy_stm32::rcc::{CpuClk, IcConfig, Icint, Icsel, Pll, Plldivm, Pllpdiv, Pllsel, SupplyConfig, SysClk};
use embassy_stm32::rif::{RifMaster, RifMasterAttributes, RifPeripheral, RifPeripheralAttributes};
use embassy_stm32::{Config, bind_interrupts, pac, peripherals};
use embassy_stm32_neochrom::{NeoChrom, nema_gfx_hal};
use embassy_time::{Instant, Timer};
use embedded_3dgfx::pipeline::vertex::mesh::{Geometry, K3dMesh, RenderMode};
use embedded_3dgfx::prelude::{CommandBuffer, FrameCtx, K3dengine};
use embedded_graphics::mono_font::MonoTextStyleBuilder;
use embedded_graphics::mono_font::ascii::{FONT_7X13, FONT_9X15, FONT_10X20};
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::pixelcolor::raw::RawU16;
use embedded_graphics::prelude::*;
use embedded_graphics::text::Text;
use embedded_gui::prelude::{Easing, GuiContext, Rect, Tween};
use heapless::String;
use micromath::F32Ext;
use nalgebra::Point3;
use panic_probe as _;
use stm32_bindings::nema_gfx::*;

use crate::framebuffer::{FastMonoFont, Framebuffer};
use crate::rk050hr18c::{HEIGHT, LTDC_CONFIG, Rk050Hr18c, WIDTH};

#[cfg(not(feature = "stub-gpu2d"))]
bind_interrupts!(struct Irqs {
    LTDC_LO => ltdc::InterruptHandler<peripherals::LTDC>;
    GPU2D_ER => gpu2d::InterruptHandler<peripherals::GPU2D>;
});

#[cfg(feature = "stub-gpu2d")]
bind_interrupts!(struct Irqs {
    LTDC_LO => ltdc::InterruptHandler<peripherals::LTDC>;
});

// Z-buffer located in AXISRAM1 (0x3400_0000..0x3410_0000, 1MB available)
const ZBUFFER_BASE: usize = 0x3400_0000;
const VP_WIDTH: u32 = 260;
const VP_HEIGHT: u32 = 200;
const ZBUFFER_LEN: usize = (VP_WIDTH * VP_HEIGHT) as usize;

// Framebuffers located in AXISRAM3..6 (0x3420_0000..0x343C_0000)
// Code and data reside safely in AXISRAM2 (0x3410_0000..0x3420_0000)
const FB0_BASE: usize = 0x3420_0000;
const FB1_BASE: usize = 0x342E_0000;
const FB_PIXELS: usize = WIDTH as usize * HEIGHT as usize;

// 3D Cube Model Definition
static CUBE_VERTS: [[f32; 3]; 8] = [
    [-1.0, -1.0, -1.0],
    [1.0, -1.0, -1.0],
    [1.0, 1.0, -1.0],
    [-1.0, 1.0, -1.0],
    [-1.0, -1.0, 1.0],
    [1.0, -1.0, 1.0],
    [1.0, 1.0, 1.0],
    [-1.0, 1.0, 1.0],
];

static CUBE_LINES: [[usize; 2]; 12] = [
    [0, 1],
    [1, 2],
    [2, 3],
    [3, 0],
    [4, 5],
    [5, 6],
    [6, 7],
    [7, 4],
    [0, 4],
    [1, 5],
    [2, 6],
    [3, 7],
];

// 3D Octahedron / Diamond Definition
static OCTA_VERTS: [[f32; 3]; 6] = [
    [0.0, 1.3, 0.0],
    [0.0, -1.3, 0.0],
    [1.0, 0.0, 0.0],
    [-1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0],
    [0.0, 0.0, -1.0],
];

static OCTA_LINES: [[usize; 2]; 12] = [
    [0, 2],
    [0, 3],
    [0, 4],
    [0, 5],
    [1, 2],
    [1, 3],
    [1, 4],
    [1, 5],
    [2, 4],
    [4, 3],
    [3, 5],
    [5, 2],
];

/// Wrapper that translates 3D viewport draws onto the target Framebuffer
struct ViewportTarget<'a, 'b> {
    fb: &'a mut Framebuffer<'b>,
    offset: Point,
    width: u32,
    height: u32,
}

impl OriginDimensions for ViewportTarget<'_, '_> {
    fn size(&self) -> Size {
        Size::new(self.width, self.height)
    }
}

impl DrawTarget for ViewportTarget<'_, '_> {
    type Color = Rgb565;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        let offset = self.offset;
        self.fb.draw_iter(
            pixels
                .into_iter()
                .map(|Pixel(p, c)| Pixel(Point::new(p.x + offset.x, p.y + offset.y), c)),
        )
    }
}

/// Framebuffer window: AXISRAM3 + AXISRAM4.
///
/// `FB0_BASE` (0x3420_0000) and `FB1_BASE` (0x342E_0000) each hold an 800x480
/// RGB565 buffer (0xBB800 bytes), so the pair ends at 0x3439_B800. The window is
/// rounded up to 2 MiB: the smallest power-of-two MPU region that both covers
/// them and is naturally aligned to its own size.
const FB_WINDOW_BASE: u32 = 0x3420_0000;
const FB_WINDOW_LEN: usize = 0x0020_0000;

/// Mark the framebuffer window Normal **non-cacheable**, then enable the D-cache.
///
/// The framebuffers are written by the CPU, written by the GPU2D over AXI, and
/// read by the LTDC scan-out. Neither the GPU nor the LTDC snoops the CPU's
/// D-cache, so a cacheable window would need clean/invalidate at every one of
/// those hand-offs; making it non-cacheable removes the hazard outright.
/// Everything else — code, stacks, and the NemaGFX command-list pool — keeps
/// its default write-back attribute through `PRIVDEFENA`, so only framebuffer
/// traffic pays the cost, and the GPU2D buffers stay cacheable for speed with
/// `nema_buffer_flush()` keeping them coherent.
///
/// The `MAIR`/`RBAR`/`RLAR` encoding matches `embassy/examples/stm32n6`.
fn configure_framebuffer_cache(mpu: &mut MPU, scb: &mut SCB, cpuid: &mut CPUID) {
    const MAIR_NORMAL_NC: u32 = 0x44; // outer + inner non-cacheable
    let limit = FB_WINDOW_BASE + FB_WINDOW_LEN as u32 - 1;

    unsafe {
        mpu.ctrl.write(0); // disable the MPU while reconfiguring
        cortex_m::asm::dsb();
        cortex_m::asm::isb();

        // Attribute index 0 = Normal, non-cacheable.
        let mair0 = mpu.mair[0].read();
        mpu.mair[0].write((mair0 & !0xFF) | MAIR_NORMAL_NC);

        mpu.rnr.write(0);
        mpu.rbar.write((FB_WINDOW_BASE & !0x1F) | (0b01 << 1) | 1);
        // RLAR: LIMIT[31:5] | AttrIndx=0 | EN=1.
        mpu.rlar.write((limit & !0x1F) | 1);

        // Enable the MPU, keeping the architectural default map as background.
        mpu.ctrl.write((1 << 2) | (1 << 0)); // PRIVDEFENA | ENABLE
        cortex_m::asm::dsb();
        cortex_m::asm::isb();
    }

    // Only bring the D-cache up once the framebuffer window is non-cacheable.
    scb.enable_dcache(cpuid);
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut cp = cortex_m::peripheral::Peripherals::take().unwrap();

    // VTOR is set up in the reset handler by cortex-m-rt's `set-vtor` feature
    // (see Cargo.toml): this image runs from AXISRAM2, not from address 0.

    // Power on the cache RAMs. On Cortex-M55 the caches are only backed by RAM
    // once MEMSYSCTL.MSCR.ICACTIVE/DCACTIVE are set, and ST's STM32N6 GPU2D
    // examples set them before enabling either cache. MEMSYSCTL (0xE001_E000) is
    // an ARM core register that neither the PAC nor embassy models, hence the
    // raw access.
    const MEMSYSCTL_MSCR: *mut u32 = 0xE001_E000 as *mut u32;
    const MSCR_ICACTIVE: u32 = 1 << 13;
    const MSCR_DCACTIVE: u32 = 1 << 12;
    unsafe {
        let mscr = core::ptr::read_volatile(MEMSYSCTL_MSCR);
        core::ptr::write_volatile(MEMSYSCTL_MSCR, mscr | MSCR_ICACTIVE | MSCR_DCACTIVE);
    }

    // Order matters: the framebuffer window must stop being cacheable before the
    // D-cache is enabled. The core I-cache is independent and always safe.
    configure_framebuffer_cache(&mut cp.MPU, &mut cp.SCB, &mut cp.CPUID);

    // Enable Cortex-M55 Instruction Cache for maximum execution speed
    cp.SCB.enable_icache();

    let mut config = Config::default();
    config.rcc.supply_config = SupplyConfig::External;

    // PLL1: 800 MHz CPU, 200 MHz system bus
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

    // PLL4: 25.0 MHz pixel clock for LTDC
    // Input HSI 64MHz / divm(4) = 16MHz * divn(25) = 400MHz VCO
    // 400MHz / divp1(4) / divp2(2) = 50MHz PLL4 output
    // 50MHz / ic16 divider(2) = 25.0 MHz pixel clock
    config.rcc.pll4 = Some(Pll::Oscillator {
        source: Pllsel::Hsi,
        divm: Plldivm::Div4,
        fractional: 0,
        divn: 25,
        divp1: Pllpdiv::Div4,
        divp2: Pllpdiv::Div2,
    });
    config.rcc.ic16 = Some(IcConfig {
        source: Icsel::Pll4,
        divider: Icint::Div2,
    });
    config.rcc.mux.ltdcsel = Ltdcsel::Ic16;

    let p = embassy_stm32::init(config);

    // The application starts with interrupts GLOBALLY DISABLED: PRIMASK is already 1 at
    // the first byte of main, before any of our setup. Nothing in our code, cortex-m-rt
    // (which never touches PRIMASK) or the embassy_executor main macro (which emits a
    // bare fn main) sets it, so it is inherited from the reset state -- the STM32N6 boot
    // ROM jumps to the application with interrupts masked.
    //
    // Until this call, every interrupt-driven path is dead and fails SILENTLY rather
    // than loudly: the embassy time driver never ticks (Timer::after never wakes, so
    // even the panel power-on sequence hangs), the LTDC line interrupt never fires, and
    // the GPU2D error / cache-hold handshake never runs.
    unsafe { cortex_m::interrupt::enable() };

    info!("STM32N6 NeoChrom + 3D Graphics Engine Demo Starting...");

    info!("Step 1: Enabling all SRAM...");
    enable_all_sram();
    info!("Step 2: Promoting RIF masters...");
    // Required for the display. Without this the LTDC cannot read the framebuffer
    // and the panel stays dark -- while the demo still reports healthy frame
    // timings, because the CPU and GPU2D can write the framebuffer fine. So a
    // dark panel with a clean log points here, not at the GPU.
    //
    // This does NOT lock the debugger out: with this call in place the demo runs
    // and CubeProgrammer can still read SRAM afterwards (verified). An earlier
    // note here claimed the opposite, from a bisect that was confounded --
    // src/bin/gpu2d_probe wedges the target when *run* (download is fine), and
    // that tool was responsible for the debug-access failures that previously
    // required a power cycle to clear.
    promote_display_and_gpu_masters();

    let cnt1 = pac::TIM5.cnt().read();
    cortex_m::asm::delay(5_000_000);
    let cnt2 = pac::TIM5.cnt().read();
    let dier = pac::TIM5.dier().read();
    let cr1 = pac::TIM5.cr1().read();
    info!(
        "TIM5 diag: cnt1={}, cnt2={}, cen={}, dier=0x{:x}",
        cnt1,
        cnt2,
        cr1.cen(),
        dier.0
    );

    info!("Step 3: Initializing LCD panel GPIOs...");
    let mut panel = Rk050Hr18c::new(p.PE1, p.PQ3, p.PQ6);
    info!("Step 4: Powering on panel...");
    panel.power_on().await;
    info!("Step 5: Panel powered on!");

    info!("Step 6: Initializing LTDC with pins...");
    let mut ltdc = Ltdc::<_, ltdc::Rgb888>::new_with_pins(
        p.LTDC, p.PB13, p.PB14, p.PE11, p.PG13, p.PG15, p.PA7, p.PB2, p.PG6, p.PH3, p.PH6, p.PA8, p.PA2, p.PG12, p.PG1,
        p.PA1, p.PA0, p.PB15, p.PB12, p.PB11, p.PG8, p.PG0, p.PD9, p.PD15, p.PB4, p.PH4, p.PA15, p.PG11, p.PD8, Irqs,
    );
    info!("Step 7: Calling ltdc.init()...");
    ltdc.init(&LTDC_CONFIG);
    info!("Step 8: LTDC initialized!");

    info!("Step 9: Initializing NeoChrom GPU2D...");
    #[cfg(feature = "stub-gpu2d")]
    let _gpu = NeoChrom::new().expect("NeoChrom GPU2D init failed");
    #[cfg(not(feature = "stub-gpu2d"))]
    let _gpu = NeoChrom::new(p.GPU2D, Irqs).expect("NeoChrom GPU2D init failed");
    info!("NeoChrom GPU2D hardware driver initialized successfully!");

    let layer_config = LtdcLayerConfig {
        pixel_format: PixelFormat::RGB565,
        layer: LtdcLayer::Layer1,
        window_x0: 0,
        window_x1: WIDTH,
        window_y0: 0,
        window_y1: HEIGHT,
    };

    let fb0_slice: &'static mut [u16] = unsafe { core::slice::from_raw_parts_mut(FB0_BASE as *mut u16, FB_PIXELS) };
    let fb1_slice: &'static mut [u16] = unsafe { core::slice::from_raw_parts_mut(FB1_BASE as *mut u16, FB_PIXELS) };
    let bg_clean = RawU16::from(Rgb565::new(2, 4, 8)).into_inner();
    fb0_slice.fill(bg_clean);
    fb1_slice.fill(bg_clean);
    let mut fb0 = Framebuffer::new(fb0_slice, WIDTH, HEIGHT);
    let mut fb1 = Framebuffer::new(fb1_slice, WIDTH, HEIGHT);
    // Opaque fills go to the GPU as one batch instead of being written row by row.
    // Off during the static pre-render, which uses its own framebuffer and its own
    // command list.
    fb0.gpu_fills = true;
    fb1.gpu_fills = true;

    let zbuffer_slice: &'static mut [u32] =
        unsafe { core::slice::from_raw_parts_mut(ZBUFFER_BASE as *mut u32, ZBUFFER_LEN) };

    ltdc.init_layer(&layer_config, None);
    ltdc.init_buffer(LtdcLayer::Layer1, fb0.as_ptr() as *const ());
    pac::LTDC.srcr().write(|w| w.set_imr(pac::ltdc::vals::Imr::Reload));

    let fbs_ptr = [fb0.as_ptr() as *const (), fb1.as_ptr() as *const ()];
    let mut back_idx = 1;

    // --- embedded-3dgfx Setup ---
    let cube_geo = Geometry {
        vertices: &CUBE_VERTS,
        faces: &[],
        colors: &[],
        lines: &CUBE_LINES,
        normals: &[],
        vertex_normals: &[],
        uvs: &[],
        texture_id: None,
    };
    let mut cube_mesh = K3dMesh::new(cube_geo);
    cube_mesh.set_render_mode(RenderMode::Lines);
    cube_mesh.set_color(Rgb565::new(0, 63, 31)); // Radiant Electric Cyan

    let octa_geo = Geometry {
        vertices: &OCTA_VERTS,
        faces: &[],
        colors: &[],
        lines: &OCTA_LINES,
        normals: &[],
        vertex_normals: &[],
        uvs: &[],
        texture_id: None,
    };
    let mut octa_mesh = K3dMesh::new(octa_geo);
    octa_mesh.set_render_mode(RenderMode::Lines);
    octa_mesh.set_color(Rgb565::new(31, 46, 0)); // Radiant Gold / Amber

    let mut engine_3d = K3dengine::new(VP_WIDTH as u16, VP_HEIGHT as u16);
    engine_3d.camera.set_position(Point3::new(0.0, 0.0, 3.4));
    // Both meshes are wireframe (RenderMode::Lines with faces: &[]), so nothing in this
    // scene emits a depth-carrying primitive and the z-buffer is never read. Opting out
    // of the clear skips a full pass over it -- 52,000 words, ~183us measured -- every
    // frame.
    //
    // This was withdrawn once on suspicion of causing a text-background regression and
    // then exonerated: that symptom appears with the clear both enabled and disabled, so
    // it is not this. If either mesh ever goes RenderMode::Solid, this line must come
    // out or occlusion will be wrong -- see set_depth_clear_enabled in embedded-3dgfx.
    engine_3d.set_depth_clear_enabled(false);
    // Hand the viewport's triangles to the GPU2D instead of the CPU rasterizer.
    // See `nema_sink`: iteration 1 covers the geometry path only, depth is not
    // wired up yet.
    engine_3d.set_raster_sink(Some(&nema_sink::NEMA_RASTER_SINK));

    // --- embedded-gui Setup ---
    let mut progress_tween = Tween::new(0.05, 1.0, 1800, Easing::InOutCubic);

    // Text styles for embedded-graphics HUD overlays with background_color for fast contiguous blitting
    let title_style = MonoTextStyleBuilder::new()
        .font(&FONT_10X20)
        .text_color(Rgb565::WHITE)
        .background_color(Rgb565::new(2, 7, 7))
        .build();
    let sub_style = MonoTextStyleBuilder::new()
        .font(&FONT_7X13)
        .text_color(Rgb565::new(0, 58, 31)) // Bright Neon Cyan
        .background_color(Rgb565::new(2, 7, 7))
        .build();
    let telemetry_title_style = MonoTextStyleBuilder::new()
        .font(&FONT_9X15)
        .text_color(Rgb565::new(31, 56, 0)) // Vivid Amber
        .background_color(Rgb565::new(3, 11, 10))
        .build();
    let vp_title_style = MonoTextStyleBuilder::new()
        .font(&FONT_9X15)
        .text_color(Rgb565::new(0, 63, 31)) // Electric Cyan
        .background_color(Rgb565::new(2, 12, 10))
        .build();
    let stack_title_style = MonoTextStyleBuilder::new()
        .font(&FONT_9X15)
        .text_color(Rgb565::new(31, 20, 24)) // Radiant Neon Rose
        .background_color(Rgb565::new(7, 6, 8))
        .build();
    let spectrum_title_style = MonoTextStyleBuilder::new()
        .font(&FONT_7X13)
        .text_color(Rgb565::new(0, 63, 31)) // Electric Cyan
        .background_color(Rgb565::new(3, 10, 9))
        .build();
    let telemetry_label_style = MonoTextStyleBuilder::new()
        .font(&FONT_7X13)
        .text_color(Rgb565::WHITE)
        .background_color(Rgb565::new(2, 6, 6))
        .build();
    let badge_styles = [
        MonoTextStyleBuilder::new()
            .font(&FONT_7X13)
            .text_color(Rgb565::WHITE)
            .background_color(Rgb565::new(0, 35, 31))
            .build(),
        MonoTextStyleBuilder::new()
            .font(&FONT_7X13)
            .text_color(Rgb565::WHITE)
            .background_color(Rgb565::new(31, 30, 0))
            .build(),
        MonoTextStyleBuilder::new()
            .font(&FONT_7X13)
            .text_color(Rgb565::WHITE)
            .background_color(Rgb565::new(0, 50, 11))
            .build(),
        MonoTextStyleBuilder::new()
            .font(&FONT_7X13)
            .text_color(Rgb565::WHITE)
            .background_color(Rgb565::new(21, 12, 31))
            .build(),
        MonoTextStyleBuilder::new()
            .font(&FONT_7X13)
            .text_color(Rgb565::WHITE)
            .background_color(Rgb565::new(31, 7, 13))
            .build(),
    ];

    info!("Initializing ultra-fast pre-sampled font cache...");
    let fast_font = FastMonoFont::new();

    // Pre-render static dashboard onto both framebuffers
    info!("Pre-rendering static glassmorphism dashboard onto fb0 and fb1...");
    let static_fbs = [(fbs_ptr[0], &mut fb0), (fbs_ptr[1], &mut fb1)];
    for (ptr, fb) in static_fbs {
        unsafe {
            // ST's canonical GPU2D usage (Resize_GPU/resize_gpu.c): a fixed-size,
            // non-expandable command list bound *circular*, never re-bound.
            //
            // Small, and flushed between card groups below -- which is the other half of
            // ST's pattern: their lists are small because they submit frequently. This
            // one has to be both, because it draws the whole dashboard and the AA
            // rounded rects tessellate heavily: at 8 KB with a single submit at the end,
            // the list overflowed partway through and every element after that point was
            // truncated -- the 5 badges at x=565 come last, so they were the visible
            // casualty. The submit+wait pairs after each group bound the usage instead.
            //
            // It stays small because the NemaGFX bump allocator's free() is a no-op, so
            // nema_cl_destroy() does not return this buffer and this program's
            // command-list sizes are a fixed budget out of a 128 KB pool -- shared with
            // the steady-state list below, which is the one that has to be big.
            let mut cl = nema_cl_create_sized(8 * 1024);
            nema_cl_bind_circular(&mut cl);
            nema_bind_dst_tex(
                ptr as usize,
                WIDTH as u32,
                HEIGHT as u32,
                NEMA_RGB565,
                (WIDTH * 2) as i32,
            );
            nema_set_clip(0, 0, WIDTH as u32, HEIGHT as u32);

            // Clear background to deep sci-fi navy blue
            nema_set_blend(
                NEMA_BF_ONE,
                nema_tex_t_NEMA_TEX0,
                nema_tex_t_NEMA_NOTEX,
                nema_tex_t_NEMA_NOTEX,
            );
            // Cards rendered with solid opaque blend (NEMA_BF_ONE) to eliminate blending quantization noise
            nema_set_blend(
                NEMA_BF_ONE,
                nema_tex_t_NEMA_TEX0,
                nema_tex_t_NEMA_NOTEX,
                nema_tex_t_NEMA_NOTEX,
            );

            // Top Header Card
            nema_fill_rounded_rect_aa(20.0, 12.0, 760.0, 60.0, 14.0, nema_rgba(18, 30, 56, 255));
            nema_fill_rounded_rect_aa(20.0, 70.0, 760.0, 3.0, 1.5, nema_rgba(0, 220, 255, 255));
            nema_cl_submit(&mut cl);
            nema_cl_wait(&mut cl);

            // Left HUD Stats Card
            nema_fill_rounded_rect_aa(20.0, 85.0, 230.0, 235.0, 12.0, nema_rgba(16, 26, 50, 255));
            nema_fill_rounded_rect_aa(20.0, 85.0, 230.0, 28.0, 12.0, nema_rgba(26, 45, 85, 255));
            nema_draw_line(25, 113, 245, 113, nema_rgba(0, 220, 255, 255));
            nema_cl_submit(&mut cl);
            nema_cl_wait(&mut cl);

            // Central 3D Viewport Card
            nema_fill_rounded_rect_aa(265.0, 85.0, 270.0, 235.0, 12.0, nema_rgba(14, 22, 44, 255));
            nema_fill_rounded_rect_aa(265.0, 85.0, 270.0, 28.0, 12.0, nema_rgba(20, 50, 80, 255));
            nema_draw_line(270, 113, 530, 113, nema_rgba(0, 240, 160, 255));
            nema_cl_submit(&mut cl);
            nema_cl_wait(&mut cl);

            // Right Features Badges Card
            nema_fill_rounded_rect_aa(550.0, 85.0, 230.0, 235.0, 12.0, nema_rgba(16, 26, 50, 255));
            nema_fill_rounded_rect_aa(550.0, 85.0, 230.0, 28.0, 12.0, nema_rgba(60, 25, 70, 255));
            nema_draw_line(555, 113, 775, 113, nema_rgba(255, 40, 160, 255));
            nema_cl_submit(&mut cl);
            nema_cl_wait(&mut cl);

            // Feature Badges with rounded pills and vibrant opaque colors
            let badges = [
                (122.0, nema_rgba(0, 140, 255, 255)),  // Azure
                (160.0, nema_rgba(255, 120, 0, 255)),  // Orange
                (198.0, nema_rgba(0, 200, 90, 255)),   // Emerald
                (236.0, nema_rgba(170, 50, 255, 255)), // Purple
                (274.0, nema_rgba(255, 30, 110, 255)), // Hot Rose
            ];
            for (by, color) in badges {
                nema_fill_rounded_rect_aa(565.0, by, 200.0, 28.0, 8.0, color);
            }
            nema_cl_submit(&mut cl);
            nema_cl_wait(&mut cl);

            // Bottom Spectrum Analyzer Card
            nema_fill_rounded_rect_aa(20.0, 330.0, 760.0, 138.0, 12.0, nema_rgba(16, 26, 50, 255));
            nema_fill_rounded_rect_aa(20.0, 330.0, 760.0, 24.0, 12.0, nema_rgba(24, 40, 76, 255));
            nema_draw_line(25, 354, 775, 354, nema_rgba(0, 220, 255, 255));

            nema_cl_submit(&mut cl);
            nema_cl_wait(&mut cl);
            nema_cl_destroy(&mut cl);
        }

        // Draw static text
        Text::new(
            "STM32N6570-DK  |  NEOCHROM GPU2D + 3DGFX + GUI DEMO",
            Point::new(34, 38),
            title_style,
        )
        .draw(fb)
        .unwrap();
        Text::new(
            "CORTEX-M55 @ 800MHz  *  RK050HR18C 800x480 RGB888  *  25MHz LTDC  *  EMBEDDED-3DGFX",
            Point::new(35, 56),
            sub_style,
        )
        .draw(fb)
        .unwrap();
        Text::new("SYSTEM TELEMETRY", Point::new(32, 103), telemetry_title_style)
            .draw(fb)
            .unwrap();
        Text::new("CORE:       M55 @ 800M", Point::new(32, 230), telemetry_label_style)
            .draw(fb)
            .unwrap();
        Text::new("PCLK:       25.0 MHz", Point::new(32, 250), telemetry_label_style)
            .draw(fb)
            .unwrap();
        Text::new("FORMAT:     RGB565 DBL", Point::new(32, 270), telemetry_label_style)
            .draw(fb)
            .unwrap();

        Text::new("3D PERSPECTIVE", Point::new(280, 103), vp_title_style)
            .draw(fb)
            .unwrap();
        Text::new("STACK LAYERS", Point::new(562, 103), stack_title_style)
            .draw(fb)
            .unwrap();

        let badge_labels = [
            (141, "NEOCHROM 2D GPU CORE", badge_styles[0]),
            (179, "EMBEDDED-3DGFX ENGINE", badge_styles[1]),
            (217, "EMBEDDED-GUI WIDGETS", badge_styles[2]),
            (255, "CORTEX-M55 HARD-FLOAT", badge_styles[3]),
            (293, "TEAR-FREE DOUBLE BUFFER", badge_styles[4]),
        ];
        for (ty, label, bstyle) in badge_labels {
            Text::new(label, Point::new(574, ty), bstyle).draw(fb).unwrap();
        }

        Text::new(
            "AUDIO FREQUENCY SPECTRUM ANALYZER (20-BAND GPU EQUALIZER)",
            Point::new(34, 347),
            spectrum_title_style,
        )
        .draw(fb)
        .unwrap();

        // Draw static telemetry HUD labels
        fast_font.draw_str(
            fb,
            32,
            122,
            "FPS:      ",
            Rgb565::new(20, 245, 120),
            Rgb565::new(2, 6, 6),
        );
        fast_font.draw_str(
            fb,
            32,
            142,
            "GPU TIME: ",
            Rgb565::new(0, 220, 255),
            Rgb565::new(2, 6, 6),
        );
        fast_font.draw_str(
            fb,
            32,
            162,
            "3D TIME:  ",
            Rgb565::new(255, 200, 0),
            Rgb565::new(2, 6, 6),
        );
        fast_font.draw_str(
            fb,
            32,
            182,
            "HUD TIME: ",
            Rgb565::new(255, 40, 120),
            Rgb565::new(2, 6, 6),
        );
        fast_font.draw_str(fb, 32, 202, "FRAME NO: ", Rgb565::WHITE, Rgb565::new(2, 6, 6));
    }
    info!("Static dashboards ready!");

    // NemaGFX reports failures through `nema_get_error()` rather than return
    // values, and a rejected command list fails *silently* otherwise: this demo
    // ran at 62 FPS while rendering nothing because the command list buffer was
    // only 4-byte aligned, so `nema_cl_bind_circular()` set
    // NEMA_ERR_INVALID_CL_ALIGMENT (0x4000) and every drawing call then failed
    // with NEMA_ERR_NO_BOUND_CL (0x80). Cheap insurance -- see nema_error.h.
    let nema_err = unsafe { nema_get_error() };
    if nema_err != 0 {
        defmt::error!("NemaGFX error after static dashboard: 0x{:x}", nema_err);
    }

    let mut frame_count: u32 = 0;
    let mut fps_timer = Instant::now();
    let mut fps_frames: u32 = 0;
    let mut current_fps: u32 = 60;
    // The HUD reports its own draw time one frame late: the measurement only closes
    // once the text has been blitted, which is after the value has been written into
    // it. So this is the one phase timer that has to survive across iterations -- the
    // others are measured and consumed within the frame that produced them.
    let mut last_hud_us: u64 = 0;

    // One reusable command list for the spectrum bars, bound circular once -- ST's
    // pattern. The per-frame path must therefore NOT re-bind or rewind it.
    //
    // 64 KB rather than 8 KB, and the size is load-bearing: this list is non-expandable
    // and circular, so when a frame's commands do not fit it is implicitly submitted
    // and the CPU *blocks* until the GPU drains. The spectrum's 20 AA rounded rects are
    // tessellated into enough commands to overflow 8 KB several times per frame, and
    // that blocking measured as ~1310us per frame -- reported as "GPU time" by the
    // spectrum-phase timer, which sits on the CPU side of the submission, when it was
    // never GPU time at all. At 64 KB it drops to ~152us.
    let mut gpu_cl = unsafe { nema_cl_create_sized(64 * 1024) };
    unsafe { nema_cl_bind_circular(&mut gpu_cl) };

    // --- GPU text: bake the 7x13 font into an RGB565 atlas and blit it ---------------
    // The background colour is baked in as the HUD's own (2,6,6), so a straight
    // NEMA_BL_SRC blit is pixel-identical to the CPU draw_str it replaces -- a 1-bpp
    // source could only ever give black and white.
    // ATLAS_BASE was 0x3430_0000, which is *inside* FB1 (0x342E_0000 + 768 KB ends at
    // 0x343A_0000) -- so FB1 rendering overwrote the atlas every frame and the blits
    // read picture data instead of glyphs. In .bss instead, where the linker guarantees
    // real, non-overlapping memory.
    const ATLAS_COLS: usize = 16;
    const ATLAS_PX: usize = ATLAS_COLS * 7 * 6 * 13;
    // UnsafeCell in a Sync wrapper: SyncUnsafeCell would be the obvious choice but is
    // still unstable (rust-lang/rust#95439).
    struct AtlasCell(core::cell::UnsafeCell<[u16; ATLAS_PX]>);
    unsafe impl Sync for AtlasCell {}
    static TEXT_ATLAS: AtlasCell = AtlasCell(core::cell::UnsafeCell::new([0; ATLAS_PX]));

    let atlas: &mut [u16] = unsafe { &mut *TEXT_ATLAS.0.get() };
    let (atlas_w, atlas_h) = fast_font.build_atlas(atlas, ATLAS_COLS, Rgb565::WHITE, Rgb565::new(2, 6, 6));
    unsafe {
        // The GPU reads raw memory, so the freshly written, dirty atlas has to be pushed
        // out to it. One clean here; the atlas is never written again.
        let atlas_addr = atlas.as_ptr() as usize;
        cortex_m::peripheral::Peripherals::steal()
            .SCB
            .clean_dcache_by_address(atlas_addr, core::mem::size_of_val(atlas));
        nema_bind_src_tex(
            atlas_addr,
            atlas_w,
            atlas_h,
            NEMA_RGB565,
            (atlas_w * 2) as i32,
            0, /* wrap: clamp */
        );
    }
    info!("text atlas: {}x{} at 0x{:x}", atlas_w, atlas_h, atlas.as_ptr() as usize);

    // Emit a string from the atlas as 7x13 blits into the bound command list. They take
    // effect at the next submit, which is the one-frame-behind behaviour the HUD values
    // already had.
    let draw_text_gpu = |x: i32, y: i32, s: &str| {
        unsafe {
            // Wrapped rather than open-coded: nema_set_blend_blit() is `static inline` in the
            // C header so bindgen cannot see it, and the blit and fill variants differ only in
            // which texture unit carries the source (NEMA_TEX1 vs NEMA_NOTEX). Getting that
            // wrong is silent: uniform-coloured output, not an error.
            nema_gfx_hal::blend::set_blit(0x1 /* NEMA_BL_SRC = NEMA_BF_ONE: straight copy */);
            nema_set_clip(x, y, (s.len() as u32) * 7, 13);
            for (i, b) in s.bytes().enumerate() {
                let idx = if (32..=127).contains(&b) { (b - 32) as usize } else { 0 };
                let sx = ((idx % ATLAS_COLS) * 7) as i32;
                let sy = ((idx / ATLAS_COLS) * 13) as i32;
                nema_blit_subrect(x + (i as i32) * 7, y, 7, 13, sx, sy);
            }
        }
    };

    // --- Depth buffer plumbing probes (item 3, step 1) --------------------------------
    // The z16 depth buffer must be sized to the *destination* coordinates, not to the
    // viewport: nema_bind_depth_buffer() takes no offset and indexes by absolute pixel.
    // The viewport occupies x=270..530, y=116..316, so 531x317 entries of 2 bytes is
    // 337 KB -- and a full-screen 800x480 one would be 750 KB, which does not fit. This
    // region is the free tail after FB1, inside the same non-cacheable MPU window.
    // First attempt was 0x343A_0000 (the tail after FB1) and it faulted on the very
    // first write, so that region is not enabled SRAM. AXISRAM1 is 1 MB and only the
    // 208 KB z-buffer is in use, so its tail is free. Cacheability does not matter here:
    // the depth buffer is written and read by the GPU only, never by the CPU.
    const DEPTH_BASE: usize = 0x3404_0000;
    const DEPTH_W: u32 = 531;
    const DEPTH_H: u32 = 317;
    // Probe 2 settled the container: 4 bytes per pixel, big-endian, holding the 24-bit
    // value (depth16 << 8) -- i.e. the depth lands in bytes 1..2 of each word. So the
    // buffer is W*H*4, not *2. Endianness is irrelevant functionally: the GPU both writes
    // and compares it and the CPU never reads it in normal operation.
    const DEPTH_BYTES: usize = (DEPTH_W * DEPTH_H * 4) as usize;
    const DEPTH_WORDS: usize = (DEPTH_W * DEPTH_H) as usize;

    // Probe 1: is that region real, enabled SRAM? The window is non-cacheable, so a
    // write/read-back tests memory rather than cache.
    let probe: &mut [u8] = unsafe { core::slice::from_raw_parts_mut(DEPTH_BASE as *mut u8, DEPTH_BYTES) };
    probe.fill(0x00);
    // Push the fill out to memory before the GPU writes, or probe 2 reads either the
    // dirty cache or SRAM retained from a previous boot. Both confounded this twice.
    unsafe {
        cortex_m::peripheral::Peripherals::steal()
            .SCB
            .clean_dcache_by_address(DEPTH_BASE, DEPTH_BYTES);
    }
    let writable = probe.iter().all(|&b| b == 0x00);
    info!(
        "depth probe 1: region 0x{:x} ({} bytes) is {}",
        DEPTH_BASE,
        DEPTH_BYTES,
        if writable { "writable" } else { "NOT USABLE" }
    );

    // Probe 2: does the GPU actually write it? Bind the buffer, clear it to a known value,
    // submit and read back. nema_clear_depth keeps bits 23:8, so 0xFFFFFF00 should leave
    // 0xFFFF in every z16 entry -- which also settles whether the buffer is bound at all.
    unsafe {
        nema_bind_depth_buffer(DEPTH_BASE, DEPTH_W, DEPTH_H);
        nema_clear_depth(0x1234_5600); // bits 23:8 = 0x3456 -- a distinctive value so the storage format is unambiguous
        nema_cl_submit(&mut gpu_cl);
        nema_cl_wait(&mut gpu_cl);
    }
    // Drop the CPU line for the range WITHOUT writing it back: probe 1 dirtied the cache,
    // and this region is cacheable. A clean would push our 0x5A pattern back over whatever
    // the GPU wrote, so a clean+invalidate would read back our own data and prove nothing.
    unsafe {
        cortex_m::peripheral::Peripherals::steal()
            .SCB
            .invalidate_dcache_by_address(DEPTH_BASE, DEPTH_BYTES);
    }

    let words_all: &[u32] = unsafe { core::slice::from_raw_parts(DEPTH_BASE as *const u32, DEPTH_WORDS) };
    // defmt cannot format a slice with hex, so read the first four words as u32.
    let words: &[u32] = unsafe { core::slice::from_raw_parts(DEPTH_BASE as *const u32, 4) };
    let in_hi = words_all.iter().filter(|&&v| (v >> 8) & 0xFFFF == 0x3456).count();
    let in_lo = 0usize;
    info!(
        "depth probe 2: words {:08x} {:08x} {:08x} {:08x} | of {} entries, {} have 0x34 high, {} low",
        words[0], words[1], words[2], words[3], DEPTH_WORDS, in_hi, in_lo
    );

    info!("Entering GPU + 3D accelerated rendering loop!");

    loop {
        let frame_start = Instant::now();
        let target_fb_ptr = fbs_ptr[back_idx];

        // Sleep until the LTDC reaches the last visible line, i.e. the start of vblank. This
        // was a `while ... { nop(); }` spin on SRCR.VBR -- the same wait, but with the CPU
        // awake for it. The LTDC_LO handler bound in `Irqs` wakes the task instead and the
        // executor sleeps in between. (`ltdc.wait_line` blocked forever while interrupts
        // were globally disabled, PRIMASK=1, which `interrupt::enable` fixed.)
        ltdc.wait_line(479).await;

        // Update animations
        if progress_tween.tick(16) {
            progress_tween.reset();
        }
        let progress_val = progress_tween.value();

        // --- 1. NeoChrom GPU2D: Update Equalizer Spectrum Bars ---
        let gpu_start = Instant::now();
        unsafe {
            // The list is bound circular once before the loop; re-binding and
            // rewinding it each frame is what ST's examples never do.
            nema_bind_dst_tex(
                target_fb_ptr as usize,
                WIDTH as u32,
                HEIGHT as u32,
                NEMA_RGB565,
                (WIDTH * 2) as i32,
            );
            nema_set_clip(25, 355, 750, 110);

            // Clear only the spectrum interior
            nema_set_blend(
                NEMA_BF_ONE,
                nema_tex_t_NEMA_TEX0,
                nema_tex_t_NEMA_NOTEX,
                nema_tex_t_NEMA_NOTEX,
            );
            nema_fill_rect(25, 355, 750, 110, nema_rgba(16, 26, 50, 255));

            // Draw 20 dynamic equalizer bars
            nema_set_blend(
                0x0504,
                nema_tex_t_NEMA_TEX0,
                nema_tex_t_NEMA_NOTEX,
                nema_tex_t_NEMA_NOTEX,
            );
            let bar_w = 26.0;
            let bar_gap = 10.5;
            let start_x = 35.0;
            let base_y = 458.0;

            for i in 0..20 {
                let fi = i as f32;
                let t = (frame_count as f32) * 0.06 + fi * 0.45;
                let s1 = (t * 1.3).sin().abs();
                let s2 = (t * 0.7 + 1.2).cos().abs();
                let bar_h = 14.0 + (s1 * 0.6 + s2 * 0.4) * 82.0;

                let bx = start_x + fi * (bar_w + bar_gap);
                let by = base_y - bar_h;

                let (r, g, b) = if i < 5 {
                    (0, 220, 255)
                } else if i < 10 {
                    (20, 245, 120)
                } else if i < 15 {
                    (255, 200, 0)
                } else {
                    (255, 40, 120)
                };

                nema_fill_rounded_rect_aa(bx, by, bar_w, bar_h, 5.0, nema_rgba(r, g, b, 240));
                let peak_y = (by - 6.0).max(362.0);
                // Plain rect, not a rounded one. This marker was
                // `nema_fill_rounded_rect_aa(..., 3.0, 1.5)` -- radius exactly half
                // the height -- so the two corner caps consumed the entire height
                // and the straight section was zero-length: a degenerate capsule,
                // which produced a stray white triangle sliver for a few frames as
                // the marker animated through particular sub-pixel positions. The
                // markers are 3 px tall; there is nothing to round.
                //
                // It also removes 20 AA rounded rects from every frame, halving the
                // primitive count in the spectrum pass.
                nema_fill_rect_f(bx + 2.0, peak_y, bar_w - 4.0, 3.0, nema_rgba(255, 255, 255, 255));
            }

            // No submit here any more: the whole frame's GPU work goes into one
            // command list and is submitted once, below.
        }
        let last_gpu_us = gpu_start.elapsed().as_micros();

        let current_fb = if back_idx == 0 { &mut fb0 } else { &mut fb1 };

        // --- 2. embedded-3dgfx Accelerated 3D Rotation ---
        let d3_start = Instant::now();
        // Clear 3D Viewport background (entire interior from y=114 to y=318, x=268 to x=532)
        current_fb
            .fill_solid(
                &embedded_graphics::primitives::Rectangle::new(Point::new(268, 114), Size::new(264, 204)),
                Rgb565::new(1, 2, 5),
            )
            .unwrap();

        // Emit that fill now so it precedes the wireframe in the command list -- both
        // go into the same list, so their order is what settles who wins.
        let vp_filled = nema_sink::emit_fills(
            current_fb.fills(),
            target_fb_ptr as usize,
            WIDTH as u32,
            HEIGHT as u32,
            (WIDTH * 2) as i32,
        );
        current_fb.clear_fills();

        let rot_x = (frame_count as f32) * 0.022;
        let rot_y = (frame_count as f32) * 0.034;
        let rot_z = (frame_count as f32) * 0.014;

        cube_mesh.set_attitude(rot_x, rot_y, rot_z);
        octa_mesh.set_attitude(-rot_y, rot_x * 1.4, -rot_z);

        let mut commands = CommandBuffer::<256>::new();
        engine_3d.record([&cube_mesh, &octa_mesh], &mut commands, None).ok();

        // No z-buffer clear here. `engine_3d.record()` already pushes
        // RenderCommand::ClearDepth and `execute` performs it (see
        // engine/recording/execute.rs), so the `zmask.fill(u32::MAX)` that used to
        // sit here was clearing 208 KB a second time -- 52,000 redundant stores
        // every frame.
        //
        // The remaining clear is itself dead work for this scene, because the
        // engine's line primitive carries no depth and nothing else here emits a
        // depth-carrying primitive, so nothing reads the buffer. Removing *that*
        // needs the clear to become lazy in the engine -- emitted on the first
        // depth-carrying draw rather than unconditionally in record() -- which is
        // the correct general fix. A front-end heuristic ("does this scene look
        // like it uses depth?") would risk stale depth and wrong occlusion.
        let mut frame_ctx = FrameCtx {
            zbuffer: zbuffer_slice,
            width: VP_WIDTH as usize,
            height: VP_HEIGHT as usize,
        };
        let mut vp_target = ViewportTarget {
            fb: current_fb,
            offset: Point::new(270, 116),
            width: VP_WIDTH,
            height: VP_HEIGHT,
        };
        engine_3d.execute(&mut vp_target, &mut frame_ctx, &commands, None).ok();

        // The sink accepted the viewport's triangles, so the engine's CPU
        // rasterizer drew none of them. Emit them to the GPU2D now -- after
        // `execute` returns, which is the only point at which the whole frame's
        // triangle set is known -- and submit through the same command list the
        // spectrum bars use.
        // The sink accepted the viewport's primitives, so the engine's CPU
        // rasterizer drew none of them. Emit them to the GPU2D now -- after
        // `execute` returns, which is the only point at which the whole frame's
        // primitives are known -- and submit through the same command list the
        // spectrum bars use.
        let (vp_tris, vp_lines) = nema_sink::NEMA_RASTER_SINK.flush(
            target_fb_ptr as usize,
            WIDTH as u32,
            HEIGHT as u32,
            (WIDTH * 2) as i32,
            270,
            116,
        );
        if frame_count == 0 {
            info!("3D sink: {} triangles, {} lines -> GPU2D", vp_tris, vp_lines);
        }
        let last_3d_us = d3_start.elapsed().as_micros();

        // --- 3. embedded-gui Dashboard Widgets ---
        // Timed, because this was the only phase without a measurement. It renders
        // on the CPU -- embedded-graphics into the framebuffer -- so it is the
        // largest unaccounted block in the frame, and the first candidate for
        // moving to the GPU.
        //
        // The phase is split, because 213 us is unexplained: its pixels are accounted
        // for (fills offloaded, ~49 px/frame of per-pixel work) yet the time did not
        // move when the fills left the CPU. Splitting setup from `render` says which
        // half it is instead of guessing a fourth time.
        let gui_start = Instant::now();
        let mut gui = GuiContext::<16, 8, 8>::new(Rect::new(20, 85, 230, 235));
        let _bar = gui
            .add_themed_progress_bar(Rect::new(30, 292, 210, 14), progress_val)
            .unwrap();
        let last_gui_setup_us = gui_start.elapsed().as_micros();
        let gui_render_start = Instant::now();
        gui.render(current_fb).ok();
        let last_gui_render_us = gui_render_start.elapsed().as_micros();
        let gui_filled = nema_sink::emit_fills(
            current_fb.fills(),
            target_fb_ptr as usize,
            WIDTH as u32,
            HEIGHT as u32,
            (WIDTH * 2) as i32,
        );
        current_fb.clear_fills();
        let last_gui_us = gui_start.elapsed().as_micros();

        // --- 3b. One submit for the whole frame ---
        // The command list now holds everything the GPU owes this frame: the spectrum
        // bars, the viewport fill, the wireframe, and the GUI's fills. Submit and wait
        // once here, because the HUD text below writes those same pixels on the CPU --
        // a fill still queued would be drawn after it and erase it.
        unsafe {
            nema_cl_submit(&mut gpu_cl);
            nema_cl_wait(&mut gpu_cl);
        }
        if frame_count == 0 {
            info!(
                "frame 0: {} viewport fills, {} gui fills -> GPU2D",
                vp_filled, gui_filled
            );
        }

        // --- 4. HUD Telemetry Overlays (Fast Direct Blit ~15 us) ---
        let hud_start = Instant::now();
        let mut buf: String<16> = String::new();

        buf.clear();
        let _ = write!(buf, "{:<6}", current_fps);
        draw_text_gpu(108, 122, &buf);

        buf.clear();
        let _ = write!(buf, "{:<5}us", last_gpu_us);
        draw_text_gpu(108, 142, &buf);

        buf.clear();
        let _ = write!(buf, "{}.{:01} ms", last_3d_us / 1000, (last_3d_us % 1000) / 100);
        draw_text_gpu(108, 162, &buf);

        buf.clear();
        let _ = write!(buf, "{:<5}us", last_hud_us);
        draw_text_gpu(108, 182, &buf);

        buf.clear();
        let _ = write!(buf, "{:<7}", frame_count);
        draw_text_gpu(108, 202, &buf);
        last_hud_us = hud_start.elapsed().as_micros();

        // --- 5. Tear-free VBlank Double Buffer Flip ---
        cortex_m::asm::dsb();
        let flip_start = Instant::now();
        let layer = pac::LTDC.layer(0);
        layer.cfbar().modify(|w| w.set_cfbadd(target_fb_ptr as u32));
        pac::LTDC.srcr().write(|w| w.set_vbr(pac::ltdc::vals::Vbr::Reload));
        let last_flip_us = flip_start.elapsed().as_micros();

        back_idx = 1 - back_idx;
        frame_count += 1;
        fps_frames += 1;

        // One-shot probe (frame 60): is the ~620-cycle per-call cost of small fills a
        // property of *this memory* (non-cacheable framebuffer writes draining the
        // write buffer per call) or of the software path? Both models predict the same
        // numbers on the framebuffer, so the control matters: run the identical
        // experiment -- 100 fills of 1x14 versus one fill of 100x14, same pixel count --
        // on a scratch buffer in .bss, which our non-cacheable MPU window does not
        // cover and is therefore cacheable.
        if frame_count == 60 {
            const PROBE_W: usize = 100;
            const PROBE_H: usize = 14;
            let mut scratch = [0u16; PROBE_W * PROBE_H];
            let mut sfb = Framebuffer::new(&mut scratch, PROBE_W as u16, PROBE_H as u16);
            let probe_color = Rgb565::new(31, 63, 31);

            let t = Instant::now();
            for i in 0..PROBE_W {
                let r = embedded_graphics::primitives::Rectangle::new(
                    Point::new(i as i32, 0),
                    Size::new(1, PROBE_H as u32),
                );
                sfb.fill_solid(&r, probe_color).unwrap();
            }
            let cache_small = t.elapsed().as_micros();

            let t = Instant::now();
            let r = embedded_graphics::primitives::Rectangle::new(
                Point::new(0, 0),
                Size::new(PROBE_W as u32, PROBE_H as u32),
            );
            sfb.fill_solid(&r, probe_color).unwrap();
            let cache_big = t.elapsed().as_micros();

            // Same pixel count, drawn in the viewport background colour so it cannot be
            // seen, and that area is redrawn every frame regardless.
            let saved_gpu_fills = current_fb.gpu_fills;
            current_fb.gpu_fills = false;
            let vp_bg = Rgb565::new(1, 2, 5);

            let t = Instant::now();
            for i in 0..PROBE_W {
                let r = embedded_graphics::primitives::Rectangle::new(
                    Point::new(270 + i as i32, 130),
                    Size::new(1, PROBE_H as u32),
                );
                current_fb.fill_solid(&r, vp_bg).unwrap();
            }
            let fb_small = t.elapsed().as_micros();

            let t = Instant::now();
            let r = embedded_graphics::primitives::Rectangle::new(
                Point::new(270, 130),
                Size::new(PROBE_W as u32, PROBE_H as u32),
            );
            current_fb.fill_solid(&r, vp_bg).unwrap();
            let fb_big = t.elapsed().as_micros();

            current_fb.gpu_fills = saved_gpu_fills;

            info!(
                "fill probe: cacheable 100x(1x14)={}us 1x(100x14)={}us || framebuffer 100x(1x14)={}us 1x(100x14)={}us",
                cache_small, cache_big, fb_small, fb_big
            );
        }

        // FPS Calculation every second
        let fps_elapsed = fps_timer.elapsed().as_millis();
        if fps_elapsed >= 1000 {
            current_fps = (fps_frames * 1000) / (fps_elapsed as u32);
            info!(
                "FPS: {} | GPU: {} us | GUI: {} us (setup {} / render {}) | 3D: {} us | HUD: {} us | Flip: {} us | Frames: {}",
                current_fps,
                last_gpu_us,
                last_gui_us,
                last_gui_setup_us,
                last_gui_render_us,
                last_3d_us,
                last_hud_us,
                last_flip_us,
                frame_count
            );
            // Which DrawTarget write path accounts for the pixels, so the GUI phase
            // can be attacked where its time actually goes rather than guessed at.
            // The counters live on the framebuffer and the two buffers alternate, so
            // this covers one buffer's frames -- the proportions are the point.
            let (fc, fp, cc, cp, ic, ip) = current_fb.take_stats();
            info!(
                "writes: fill {} calls/{} px | contig {} calls/{} px | iter {} calls/{} px",
                fc, fp, cc, cp, ic, ip
            );
            // And how much of the fill work that *looks* offloaded actually executed
            // on the CPU instead, because a CPU write arrived before the queue could
            // be handed over.
            let (gfc, gfp, gfd) = current_fb.take_gpu_fill_stats();
            info!(
                "gpu fills: {} calls/{} px | drained back to CPU {} times",
                gfc, gfp, gfd
            );
            fps_timer = Instant::now();
            fps_frames = 0;
        }

        // Optional frame timing limiter (~60 FPS)
        let total_frame_ms = frame_start.elapsed().as_millis();
        if total_frame_ms < 16 {
            Timer::after(embassy_time::Duration::from_millis(16 - total_frame_ms)).await;
        }
    }
}

/// Enable run-mode clocks for every AXISRAM and AHBSRAM bank
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

/// Promote display and GPU bus masters in the Resource Isolation Framework (RIF)
fn promote_display_and_gpu_masters() {
    for rif_master in [RifMaster::Gpu2d, RifMaster::Dma2d, RifMaster::LtdcL1, RifMaster::LtdcL2] {
        rif_master.set_attributes(&RifMasterAttributes::new(1, true, true));
    }
    for rif_periph in [
        RifPeripheral::Gpu2d,
        RifPeripheral::Dma2d,
        RifPeripheral::Ltdc,
        RifPeripheral::LtdcL1,
        RifPeripheral::LtdcL2,
        RifPeripheral::Tim5,
    ] {
        rif_periph.set_attributes(&RifPeripheralAttributes::new(true, true));
    }
    unsafe {
        cortex_m::peripheral::NVIC::unmask(pac::Interrupt::TIM5);
    }
}
