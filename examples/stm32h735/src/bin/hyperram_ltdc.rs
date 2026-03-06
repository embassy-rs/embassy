#![no_std]
#![no_main]
#![macro_use]

/// Combines the OCTOSPI2 HyperRAM bring-up from `hyperram.rs` with the LTDC display
/// pipeline from `ltdc.rs`: a full-width **RGB565** double-buffered framebuffer that
/// lives in external HyperRAM instead of the 320 KiB internal AXI SRAM, animated the
/// same way as `ltdc.rs` (bouncing ferris crab).
///
/// No CLUT is needed this time: RGB565 is a first-class LTDC pixel format, so each
/// pixel is written directly as a 16-bit value. The LTDC layer's pixel format and its
/// framebuffer address are entirely orthogonal (`docs/embassy/understanding.md` §4):
/// `CFBAR` is cast straight to a `u32` with no region restriction, so pointing it at
/// the OCTOSPI2 window at `0x7000_0000` works exactly like pointing it at SRAM.
///
/// Panel enable pins (see `ltdc.rs` / `docs/embassy/ltdc-regression-diagnosis.md`):
/// LCD_DISP (PD10) and LCD_BL_CTRL (PG15) must be driven high or the panel shows a
/// blank white screen regardless of valid video output.
use bouncy_box::BouncyBox;
use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::ltdc::{self, Ltdc, LtdcConfiguration, LtdcLayer, LtdcLayerConfig, PolarityActive, PolarityEdge};
use embassy_stm32::ospi::{
    AddressSize, ChipSelectHighTime, Config as OspiConfig, FIFOThresholdLevel, HyperbusConfig, HyperbusLatencyMode,
    MemorySize, MemoryType, Ospi, OspiWidth, TransferConfig, WrapSize,
};
use embassy_stm32::{bind_interrupts, peripherals};
use embassy_time::{Duration, Timer};
use embedded_graphics::Pixel;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::{OriginDimensions, Point, Size};
use embedded_graphics::image::Image;
use embedded_graphics::pixelcolor::{Rgb565, Rgb888};
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::Rectangle;
use tinybmp::Bmp;
use {defmt_rtt as _, panic_probe as _};

const DISPLAY_WIDTH: usize = 480;
const DISPLAY_HEIGHT: usize = 272;
const MY_TASK_POOL_SIZE: usize = 2;

/// HyperRAM (OCTOSPI2) memory-mapped window base (silicon-fixed, RM0468 memory map).
const HYPERRAM_BASE: u32 = 0x7000_0000;
/// One RGB565 frame: 480*272*2 = 261,120 bytes (~255 KiB).
const FB_SIZE_PIXELS: usize = DISPLAY_WIDTH * DISPLAY_HEIGHT;
/// 256 KiB spacing keeps both buffers far from the S70KL1281's 8 MiB die boundary
/// (`0x7080_0000`) and leaves each buffer trivially bus-aligned - CFBAR must be
/// bus-aligned, see the FifoUnderrun regression fixed in `ltdc.rs`.
const FB1_ADDR: u32 = HYPERRAM_BASE;
const FB2_ADDR: u32 = HYPERRAM_BASE + 0x4_0000;

bind_interrupts!(struct Irqs {
    LTDC => ltdc::InterruptHandler<peripherals::LTDC>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = rcc_setup::stm32h735g_init();

    // blink the led on another task
    let led = Output::new(p.PC3, Level::High, Speed::Low);
    spawner.spawn(unwrap!(led_task(led)));

    // Panel control pins, outside the LTDC signal interface (see ltdc.rs).
    let _lcd_disp = Output::new(p.PD10, Level::High, Speed::Low);
    let _lcd_backlight = Output::new(p.PG15, Level::High, Speed::Low);

    info!("init HyperRAM (OCTOSPI2)");

    // HyperBus device config for the S70KL1281 - identical to hyperram.rs; field
    // derivation in docs/hal-oracle/octospi-hyperram.md §2.
    let ospi_config = OspiConfig {
        fifo_threshold: FIFOThresholdLevel::_4Bytes,
        memory_type: MemoryType::HyperBusMemory,
        device_size: MemorySize::_16MiB,
        chip_select_high_time: ChipSelectHighTime::_4Cycle,
        free_running_clock: false,
        clock_mode: false,
        wrap_size: WrapSize::None,
        clock_prescaler: 1, // 200 MHz kernel / 2 = 100 MHz HyperBus clock
        sample_shifting: false,
        delay_hold_quarter_cycle: true,
        chip_select_boundary: 23,
        delay_block_bypass: false,
        max_transfer: 0,
        refresh: 400,
    };

    // Pin map: docs/HARDWARE.md "OCTOSPI2 <-> HyperRAM pin map".
    // Kept alive for the program's lifetime: `Ospi::drop` disables the peripheral
    // clock and its owned pins disconnect on drop, which would tear down the live
    // memory-mapped window the LTDC layer below reads from.
    let mut ospi = Ospi::new_blocking_octospi_with_dqs(
        p.OCTOSPI2,
        p.PF4,  // CLK
        p.PF0,  // DQ0
        p.PF1,  // DQ1
        p.PF2,  // DQ2
        p.PF3,  // DQ3
        p.PG0,  // DQ4
        p.PG1,  // DQ5
        p.PG10, // DQ6
        p.PG11, // DQ7
        p.PG12, // NCS
        p.PF12, // DQS
        ospi_config,
    );

    // HyperBus latency (HLCR): fixed, initial latency 6 — see hyperram.rs.
    ospi.configure_hyperbus(HyperbusConfig {
        latency_mode: HyperbusLatencyMode::Fixed,
        access_time: 6,
        rw_recovery_time: 3,
        write_zero_latency: false,
    });

    // Same 8-lane DTR address-only command as hyperram.rs; see its comment for why
    // `enable_memory_mapped_mode` (written for NOR-flash commands) works unmodified.
    let hyperbus_command = TransferConfig {
        adwidth: OspiWidth::OCTO,
        address: Some(0),
        adsize: AddressSize::_32bit,
        addtr: true,
        dwidth: OspiWidth::OCTO,
        ddtr: true,
        dqse: true,
        ..Default::default()
    };
    ospi.enable_memory_mapped_mode(hyperbus_command, hyperbus_command)
        .expect("failed to enable HyperRAM memory-mapped mode");

    info!(
        "HyperRAM memory-mapped; FB1 @ 0x{:08x}, FB2 @ 0x{:08x}",
        FB1_ADDR, FB2_ADDR
    );

    // numbers from STMicroelectronics/STM32CubeH7 STM32H735G-DK C-based example
    const RK043FN48H_HSYNC: u16 = 41; // Horizontal synchronization
    const RK043FN48H_HBP: u16 = 13; // Horizontal back porch
    const RK043FN48H_HFP: u16 = 32; // Horizontal front porch
    const RK043FN48H_VSYNC: u16 = 10; // Vertical synchronization
    const RK043FN48H_VBP: u16 = 2; // Vertical back porch
    const RK043FN48H_VFP: u16 = 2; // Vertical front porch

    let ltdc_config = LtdcConfiguration {
        active_width: DISPLAY_WIDTH as _,
        active_height: DISPLAY_HEIGHT as _,
        h_back_porch: RK043FN48H_HBP - 11, // -11 from MX_LTDC_Init
        h_front_porch: RK043FN48H_HFP,
        v_back_porch: RK043FN48H_VBP,
        v_front_porch: RK043FN48H_VFP,
        h_sync: RK043FN48H_HSYNC,
        v_sync: RK043FN48H_VSYNC,
        h_sync_polarity: PolarityActive::ActiveLow,
        v_sync_polarity: PolarityActive::ActiveLow,
        data_enable_polarity: PolarityActive::ActiveHigh,
        pixel_clock_polarity: PolarityEdge::FallingEdge,
    };

    info!("init ltdc");
    let mut ltdc = Ltdc::<_, ltdc::Rgb888>::new_with_pins(
        p.LTDC, Irqs, p.PG7, p.PC6, p.PA4, p.PE13, p.PG14, p.PD0, p.PD6, p.PA8, p.PE12, p.PA3, p.PB8, p.PB9, p.PB1,
        p.PB0, p.PA6, p.PE11, p.PH15, p.PH4, p.PC7, p.PD3, p.PE0, p.PH3, p.PH8, p.PH9, p.PH10, p.PH11, p.PE1, p.PE15,
    );
    ltdc.init(&ltdc_config);

    // we only need to draw on one layer for this example (not to be confused with the double buffer)
    info!("enable bottom layer (RGB565, no CLUT)");
    let layer_config = LtdcLayerConfig {
        pixel_format: ltdc::PixelFormat::Rgb565, // 2 bytes per pixel, no CLUT
        layer: LtdcLayer::Layer1,
        window_x0: 0,
        window_x1: DISPLAY_WIDTH as _,
        window_y0: 0,
        window_y1: DISPLAY_HEIGHT as _,
    };
    ltdc.init_layer(&layer_config, None);

    let ferris_bmp: Bmp<Rgb888> = Bmp::from_slice(include_bytes!("./ferris.bmp")).unwrap();

    // Safety: FB1/FB2 are two disjoint regions of the OCTOSPI2 memory-mapped window
    // (see FB1_ADDR/FB2_ADDR above); DoubleBuffer is the only thing that mutates them.
    let fb1 = unsafe { core::slice::from_raw_parts_mut(FB1_ADDR as *mut TargetPixelType, FB_SIZE_PIXELS) };
    let fb2 = unsafe { core::slice::from_raw_parts_mut(FB2_ADDR as *mut TargetPixelType, FB_SIZE_PIXELS) };
    let mut double_buffer = DoubleBuffer::new(fb1, fb2, layer_config);

    // this allows us to perform some simple animation for every frame
    let mut bouncy_box = BouncyBox::new(
        ferris_bmp.bounding_box(),
        Rectangle::new(Point::zero(), Size::new(DISPLAY_WIDTH as u32, DISPLAY_HEIGHT as u32)),
        2,
    );

    loop {
        // cpu intensive drawing to the buffer that is NOT currently being copied to the LCD screen
        double_buffer.clear();
        let position = bouncy_box.next_point();
        let ferris = Image::new(&ferris_bmp, position);
        unwrap!(ferris.draw(&mut double_buffer));

        // perform async dma data transfer to the lcd screen
        unwrap!(double_buffer.swap(&mut ltdc).await);
    }
}

#[embassy_executor::task(pool_size = MY_TASK_POOL_SIZE)]
async fn led_task(mut led: Output<'static>) {
    let mut counter = 0;
    loop {
        info!("blink: {}", counter);
        counter += 1;

        // on
        led.set_low();
        Timer::after(Duration::from_millis(50)).await;

        // off
        led.set_high();
        Timer::after(Duration::from_millis(450)).await;
    }
}

pub type TargetPixelType = u16;

/// Truncating 8-bit -> 5/6/5-bit channel reduction (no dithering); embedded-graphics
/// has no built-in Rgb888->Rgb565 conversion.
fn rgb888_to_rgb565(c: Rgb888) -> Rgb565 {
    Rgb565::new(c.r() >> 3, c.g() >> 2, c.b() >> 3)
}

// A simple double buffer, RGB565 pixels, no CLUT
pub struct DoubleBuffer {
    buf0: &'static mut [TargetPixelType],
    buf1: &'static mut [TargetPixelType],
    is_buf0: bool,
    layer_config: LtdcLayerConfig,
}

impl DoubleBuffer {
    pub fn new(
        buf0: &'static mut [TargetPixelType],
        buf1: &'static mut [TargetPixelType],
        layer_config: LtdcLayerConfig,
    ) -> Self {
        Self {
            buf0,
            buf1,
            is_buf0: true,
            layer_config,
        }
    }

    pub fn current(&mut self) -> &mut [TargetPixelType] {
        if self.is_buf0 { self.buf0 } else { self.buf1 }
    }

    pub async fn swap<T: ltdc::Instance>(&mut self, ltdc: &mut Ltdc<'_, T>) -> Result<(), ltdc::Error> {
        let frame_buffer = self.current().as_ptr();
        self.is_buf0 = !self.is_buf0;
        ltdc.set_buffer(self.layer_config.layer, frame_buffer as *const _).await
    }

    /// Clears the buffer
    pub fn clear(&mut self) {
        for pixel in self.current().iter_mut() {
            *pixel = 0x0000; // solid black
        }
    }
}

// Implement DrawTarget for
impl DrawTarget for DoubleBuffer {
    type Color = Rgb888;
    type Error = ();

    /// Draw a pixel
    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        let size = self.size();
        let width = size.width as i32;
        let height = size.height as i32;
        let buf = self.current();

        for pixel in pixels {
            let Pixel(point, color) = pixel;

            if point.x >= 0 && point.y >= 0 && point.x < width && point.y < height {
                let index = point.y * width + point.x;
                buf[index as usize] = rgb888_to_rgb565(color).into_storage();
            } else {
                // Ignore invalid points
            }
        }

        Ok(())
    }
}

impl OriginDimensions for DoubleBuffer {
    /// Return the size of the display
    fn size(&self) -> Size {
        Size::new(
            (self.layer_config.window_x1 - self.layer_config.window_x0) as _,
            (self.layer_config.window_y1 - self.layer_config.window_y0) as _,
        )
    }
}

mod rcc_setup {

    use embassy_stm32::rcc::mux::Fmcsel;
    use embassy_stm32::rcc::{Hse, HseMode, *};
    use embassy_stm32::time::Hertz;
    use embassy_stm32::{Config, Peripherals};

    /// Clocks for the STM32H735G-DK: SYSCLK 520 MHz (PLL1), OCTOSPI kernel clock
    /// 200 MHz (PLL2_R, routed via `mux.octospisel`), LTDC pixel clock ~9.64 MHz
    /// (PLL3_R, hardwired - no mux). Numbers match `docs/hal-oracle/octospi-hyperram.md`
    /// §1 and `docs/hal-oracle/ltdc-cmsis.md` §1 / `docs/HARDWARE.md`.
    pub fn stm32h735g_init() -> Peripherals {
        let mut config = Config::default();
        config.rcc.hse = Some(Hse {
            freq: Hertz::mhz(25),
            mode: HseMode::Oscillator,
        });
        config.rcc.hsi = None;
        config.rcc.csi = false;
        config.rcc.pll1 = Some(Pll {
            source: PllSource::Hse,
            prediv: PllPreDiv::Div5, // PLL_M
            mul: PllMul::Mul104,     // PLL_N
            divp: Some(PllDiv::Div1),
            divq: Some(PllDiv::Div4),
            divr: Some(PllDiv::Div2),
        });
        // numbers adapted from Drivers/BSP/STM32H735G-DK/stm32h735g_discovery_ospi.c
        // MX_OSPI_ClockConfig
        config.rcc.pll2 = Some(Pll {
            source: PllSource::Hse,
            prediv: PllPreDiv::Div5, // PLL_M
            mul: PllMul::Mul80,      // PLL_N
            divp: Some(PllDiv::Div5),
            divq: Some(PllDiv::Div2),
            divr: Some(PllDiv::Div2), // pll2_r = 200 MHz
        });
        // numbers adapted from Drivers/BSP/STM32H735G-DK/stm32h735g_discovery_lcd.c
        // MX_LTDC_ClockConfig
        config.rcc.pll3 = Some(Pll {
            source: PllSource::Hse,
            prediv: PllPreDiv::Div5, // PLL_M
            mul: PllMul::Mul160,     // PLL_N
            divp: Some(PllDiv::Div2),
            divq: Some(PllDiv::Div2),
            divr: Some(PllDiv::Div83),
        });
        config.rcc.voltage_scale = VoltageScale::Scale0;
        config.rcc.supply_config = SupplyConfig::DirectSMPS;
        config.rcc.sys = Sysclk::Pll1P;
        config.rcc.ahb_pre = AHBPrescaler::Div2;
        config.rcc.apb1_pre = APBPrescaler::Div2;
        config.rcc.apb2_pre = APBPrescaler::Div2;
        config.rcc.apb3_pre = APBPrescaler::Div2;
        config.rcc.apb4_pre = APBPrescaler::Div2;
        config.rcc.mux.octospisel = Fmcsel::Pll2R;
        embassy_stm32::init(config)
    }
}

mod bouncy_box {
    use embedded_graphics::geometry::Point;
    use embedded_graphics::primitives::Rectangle;

    enum Direction {
        DownLeft,
        DownRight,
        UpLeft,
        UpRight,
    }

    pub struct BouncyBox {
        direction: Direction,
        child_rect: Rectangle,
        parent_rect: Rectangle,
        current_point: Point,
        move_by: usize,
    }

    // This calculates the coordinates of a chile rectangle bounced around inside a parent bounded box
    impl BouncyBox {
        pub fn new(child_rect: Rectangle, parent_rect: Rectangle, move_by: usize) -> Self {
            let center_box = parent_rect.center();
            let center_img = child_rect.center();
            let current_point = Point::new(center_box.x - center_img.x / 2, center_box.y - center_img.y / 2);
            Self {
                direction: Direction::DownRight,
                child_rect,
                parent_rect,
                current_point,
                move_by,
            }
        }

        pub fn next_point(&mut self) -> Point {
            let direction = &self.direction;
            let img_height = self.child_rect.size.height as i32;
            let box_height = self.parent_rect.size.height as i32;
            let img_width = self.child_rect.size.width as i32;
            let box_width = self.parent_rect.size.width as i32;
            let move_by = self.move_by as i32;

            match direction {
                Direction::DownLeft => {
                    self.current_point.x -= move_by;
                    self.current_point.y += move_by;

                    let x_out_of_bounds = self.current_point.x < 0;
                    let y_out_of_bounds = (self.current_point.y + img_height) > box_height;

                    if x_out_of_bounds && y_out_of_bounds {
                        self.direction = Direction::UpRight
                    } else if x_out_of_bounds && !y_out_of_bounds {
                        self.direction = Direction::DownRight
                    } else if !x_out_of_bounds && y_out_of_bounds {
                        self.direction = Direction::UpLeft
                    }
                }
                Direction::DownRight => {
                    self.current_point.x += move_by;
                    self.current_point.y += move_by;

                    let x_out_of_bounds = (self.current_point.x + img_width) > box_width;
                    let y_out_of_bounds = (self.current_point.y + img_height) > box_height;

                    if x_out_of_bounds && y_out_of_bounds {
                        self.direction = Direction::UpLeft
                    } else if x_out_of_bounds && !y_out_of_bounds {
                        self.direction = Direction::DownLeft
                    } else if !x_out_of_bounds && y_out_of_bounds {
                        self.direction = Direction::UpRight
                    }
                }
                Direction::UpLeft => {
                    self.current_point.x -= move_by;
                    self.current_point.y -= move_by;

                    let x_out_of_bounds = self.current_point.x < 0;
                    let y_out_of_bounds = self.current_point.y < 0;

                    if x_out_of_bounds && y_out_of_bounds {
                        self.direction = Direction::DownRight
                    } else if x_out_of_bounds && !y_out_of_bounds {
                        self.direction = Direction::UpRight
                    } else if !x_out_of_bounds && y_out_of_bounds {
                        self.direction = Direction::DownLeft
                    }
                }
                Direction::UpRight => {
                    self.current_point.x += move_by;
                    self.current_point.y -= move_by;

                    let x_out_of_bounds = (self.current_point.x + img_width) > box_width;
                    let y_out_of_bounds = self.current_point.y < 0;

                    if x_out_of_bounds && y_out_of_bounds {
                        self.direction = Direction::DownLeft
                    } else if x_out_of_bounds && !y_out_of_bounds {
                        self.direction = Direction::UpLeft
                    } else if !x_out_of_bounds && y_out_of_bounds {
                        self.direction = Direction::DownRight
                    }
                }
            }

            self.current_point
        }
    }
}
