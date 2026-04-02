//! LCD example on STM32F429I-DISCO
//!
//! Uses LTDC to drive IL9341 panel, and DMA2D to fill SRAM framebuffer

#![no_std]
#![no_main]

use defmt::{error, info};
use embassy_executor::Spawner;
use embassy_stm32::dma2d::{self, Buffer2D, Dma2d};
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::ltdc::{self, Ltdc, LtdcConfiguration, LtdcLayerConfig};
use embassy_stm32::mode::Blocking;
use embassy_stm32::spi::mode::Master;
use embassy_stm32::spi::{self};
use embassy_stm32::time::Hertz;
use embassy_stm32::{Config, bind_interrupts, peripherals};
use embassy_time::Timer;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(
    struct Irqs {
        LTDC => ltdc::InterruptHandler<peripherals::LTDC>;
        DMA2D => dma2d::InterruptHandler<peripherals::DMA2D>;
    }
);

const WIDTH: usize = 240;
const HEIGHT: usize = 320;

static FB: StaticCell<[u16; WIDTH * HEIGHT]> = StaticCell::new();

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hse = Some(Hse {
            freq: Hertz(8_000_000),
            mode: HseMode::Oscillator,
        });
        config.rcc.pll_src = PllSource::HSE;
        config.rcc.pll = Some(Pll {
            prediv: PllPreDiv::DIV8,
            mul: PllMul::MUL360,
            divp: Some(PllPDiv::DIV2), // 8mhz / 8 * 360 / 2 = 180Mhz.
            divq: Some(PllQDiv::DIV7),
            divr: None,
        });
        config.rcc.ahb_pre = AHBPrescaler::DIV1;
        config.rcc.apb1_pre = APBPrescaler::DIV4;
        config.rcc.apb2_pre = APBPrescaler::DIV2;
        config.rcc.sys = Sysclk::PLL1_P;

        // Configure SAI PLL for LTDC
        config.rcc.pllsai = Some(Pll {
            prediv: PllPreDiv::DIV8,
            mul: PllMul::MUL192,
            divp: None,
            divq: None,
            divr: Some(PllRDiv::DIV4), // 8mhz / 8 * 192 / 4 = 48MHz
        });

        // Set LCD dot clock divisor to 8
        // F = PLLSAI.R / 8 = 48 / 8 = 6MHz
        config.rcc.lcd_div = Some(ltdc::LcdClockDiv::Div8);
    }
    let p = embassy_stm32::init(config);

    let mut lcd_cs = Output::new(p.PC2, Level::High, Speed::VeryHigh);
    let mut lcd_dc = Output::new(p.PD13, Level::High, Speed::VeryHigh);

    let mut spi_cfg = spi::Config::default();
    spi_cfg.frequency = Hertz(10_000_000);
    spi_cfg.mode = spi::MODE_0;

    let mut lcd_spi = spi::Spi::new_blocking_txonly(
        p.SPI5, p.PF7, // SCK
        p.PF9, // MOSI
        spi_cfg,
    );

    ili9341_init(&mut lcd_spi, &mut lcd_cs, &mut lcd_dc).await;
    drop(lcd_spi);

    let b2 = p.PD6;
    let b3 = p.PG11;
    let b4 = p.PG12;
    let b5 = p.PA3;
    let b6 = p.PB8;
    let b7 = p.PB9;

    let g2 = p.PA6;
    let g3 = p.PG10;
    let g4 = p.PB10;
    let g5 = p.PB11;
    let g6 = p.PC7;
    let g7 = p.PD3;

    let r2 = p.PC10;
    let r3 = p.PB0;
    let r4 = p.PA11;
    let r5 = p.PA12;
    let r6 = p.PB1;
    let r7 = p.PG6;

    let mut ltdc = Ltdc::<_, ltdc::Rgb666>::new_with_pins(
        p.LTDC, Irqs, p.PG7, p.PC6, p.PA4, p.PF10, b2, b3, b4, b5, b6, b7, g2, g3, g4, g5, g6, g7, r2, r3, r4, r5, r6,
        r7,
    );

    info!("LTDC Init");
    ltdc.init(&LtdcConfiguration {
        active_width: WIDTH as u16,
        active_height: HEIGHT as u16,
        h_back_porch: 20,
        h_front_porch: 10,
        v_back_porch: 2,
        v_front_porch: 4,
        h_sync: 10,
        v_sync: 2,
        h_sync_polarity: ltdc::PolarityActive::ActiveLow,
        v_sync_polarity: ltdc::PolarityActive::ActiveLow,
        data_enable_polarity: ltdc::PolarityActive::ActiveLow,
        pixel_clock_polarity: ltdc::PolarityEdge::FallingEdge,
    });

    info!("LTDC Layer Init");
    ltdc.init_layer(
        &LtdcLayerConfig {
            layer: ltdc::LtdcLayer::Layer1,
            pixel_format: ltdc::PixelFormat::RGB565,
            window_x0: 0,
            window_x1: WIDTH as u16,
            window_y0: 0,
            window_y1: HEIGHT as u16,
        },
        None,
    );

    let fb = FB.init_with(|| [0; WIDTH * HEIGHT]);

    ltdc.set_buffer(ltdc::LtdcLayer::Layer1, fb.as_ptr() as *const _)
        .await
        .unwrap();

    let mut dma2d = Dma2d::new(p.DMA2D, Irqs);
    let buf = Buffer2D::new(
        fb.as_ptr() as *mut u8,
        dma2d::PixelFormat::Rgb565,
        WIDTH as u16,
        WIDTH as u16,
        HEIGHT as u16,
    );

    if let Err(e) = dma2d.fill(&buf.region(0, 0, WIDTH as u16, HEIGHT as u16), 0xf800).await {
        error!("DMA2D {}", e);
    }

    let mut led = Output::new(p.PG13, Level::High, Speed::Low);

    loop {
        dma2d
            .fill(&buf.region(0, 0, WIDTH as u16, HEIGHT as u16), 0xf800)
            .await
            .unwrap();

        led.set_high();
        Timer::after_millis(1000).await;

        dma2d
            .fill(&buf.region(0, 0, WIDTH as u16, HEIGHT as u16), 0x008f)
            .await
            .unwrap();

        led.set_low();
        Timer::after_millis(1000).await;
    }
}

// Initialize the IL9341. Commands are derived from https://github.com/STMicroelectronics/stm32-ili9341
async fn ili9341_init(spi: &mut spi::Spi<'_, Blocking, Master>, cs: &mut Output<'_>, dc: &mut Output<'_>) {
    lcd_cmd_data(spi, cs, dc, 0xCA, &[0xC3, 0x08, 0x50]);

    lcd_cmd_data(spi, cs, dc, 0xCF, &[0x00, 0xC1, 0x30]); // LCD_POWERB
    lcd_cmd_data(spi, cs, dc, 0xED, &[0x64, 0x03, 0x12, 0x81]); // LCD_POWER_SEQ
    lcd_cmd_data(spi, cs, dc, 0xE8, &[0x85, 0x00, 0x78]); // LCD_DTCA
    lcd_cmd_data(spi, cs, dc, 0xCB, &[0x39, 0x2C, 0x00, 0x34, 0x02]); // LCD_POWERA
    lcd_cmd_data(spi, cs, dc, 0xF7, &[0x20]); // LCD_PRC
    lcd_cmd_data(spi, cs, dc, 0xEA, &[0x00, 0x00]); // LCD_DTCB

    lcd_cmd_data(spi, cs, dc, 0xB1, &[0x00, 0x1B]); // LCD_FRMCTR1

    // First DFC write
    lcd_cmd_data(spi, cs, dc, 0xB6, &[0x0A, 0xA2]); // LCD_DFC

    lcd_cmd_data(spi, cs, dc, 0xC0, &[0x10]); // LCD_POWER1
    lcd_cmd_data(spi, cs, dc, 0xC1, &[0x10]); // LCD_POWER2
    lcd_cmd_data(spi, cs, dc, 0xC5, &[0x45, 0x15]); // LCD_VCOM1
    lcd_cmd_data(spi, cs, dc, 0xC7, &[0x90]); // LCD_VCOM2

    lcd_cmd_data(spi, cs, dc, 0x36, &[0xC8]); // LCD_MAC
    lcd_cmd_data(spi, cs, dc, 0xF2, &[0x00]); // LCD_3GAMMA_EN

    // This is one of the key RGB-interface commands
    lcd_cmd_data(spi, cs, dc, 0xB0, &[0xC2]); // LCD_RGB_INTERFACE

    // Second DFC write - also important for LTDC/RGB mode
    lcd_cmd_data(spi, cs, dc, 0xB6, &[0x0A, 0xA7, 0x27, 0x04]); // LCD_DFC

    // Column address set: 0..239
    lcd_cmd_data(spi, cs, dc, 0x2A, &[0x00, 0x00, 0x00, 0xEF]); // LCD_COLUMN_ADDR

    // Page address set: 0..319
    lcd_cmd_data(spi, cs, dc, 0x2B, &[0x00, 0x00, 0x01, 0x3F]); // LCD_PAGE_ADDR

    // Interface control - important for RGB interface
    lcd_cmd_data(spi, cs, dc, 0xF6, &[0x01, 0x00, 0x06]); // LCD_INTERFACE

    lcd_cmd(spi, cs, dc, 0x2C); // LCD_GRAM
    Timer::after_millis(200).await;

    lcd_cmd_data(spi, cs, dc, 0x26, &[0x01]); // LCD_GAMMA

    lcd_cmd_data(
        spi,
        cs,
        dc,
        0xE0, // LCD_PGAMMA
        &[
            0x0F, 0x29, 0x24, 0x0C, 0x0E, 0x09, 0x4E, 0x78, 0x3C, 0x09, 0x13, 0x05, 0x17, 0x11, 0x00,
        ],
    );

    lcd_cmd_data(
        spi,
        cs,
        dc,
        0xE1, // LCD_NGAMMA
        &[
            0x00, 0x16, 0x1B, 0x04, 0x11, 0x07, 0x31, 0x33, 0x42, 0x05, 0x0C, 0x0A, 0x28, 0x2F, 0x0F,
        ],
    );

    lcd_cmd(spi, cs, dc, 0x11); // LCD_SLEEP_OUT
    Timer::after_millis(200).await;

    lcd_cmd(spi, cs, dc, 0x29); // LCD_DISPLAY_ON

    // GRAM start writing
    lcd_cmd(spi, cs, dc, 0x2C); // LCD_GRAM
}

fn lcd_cmd(spi: &mut spi::Spi<'_, Blocking, Master>, cs: &mut Output<'_>, dc: &mut Output<'_>, cmd: u8) {
    cs.set_low();
    dc.set_low();
    spi.blocking_write(&[cmd]).unwrap();
    cs.set_high();
}

fn lcd_data(spi: &mut spi::Spi<'_, Blocking, Master>, cs: &mut Output<'_>, dc: &mut Output<'_>, data: &[u8]) {
    cs.set_low();
    dc.set_high();
    spi.blocking_write(data).unwrap();
    cs.set_high();
}

fn lcd_cmd_data(
    spi: &mut spi::Spi<'_, Blocking, Master>,
    cs: &mut Output<'_>,
    dc: &mut Output<'_>,
    cmd: u8,
    data: &[u8],
) {
    lcd_cmd(spi, cs, dc, cmd);
    lcd_data(spi, cs, dc, data);
}
