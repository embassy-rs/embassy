//! STM32H747I-DISCO Display example using DSI in video burst mode

#![no_std]
#![no_main]

use core::mem::MaybeUninit;

use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::dsihost::panel::DsiPanel;
use embassy_stm32::dsihost::{self, DsiColor, DsiHost, DsiHostMode, DsiHostPhyConfig, DsiHostPhyLanes, DsiVideoConfig};
use embassy_stm32::fmc::Fmc;
use embassy_stm32::gpio::Output;
use embassy_stm32::ltdc::{self, Ltdc, LtdcLayer, LtdcLayerConfig, PixelFormat};
use embassy_stm32::rcc::{DsiHostPllConfig, DsiPllInput, DsiPllOutput, Hse, Pll};
use embassy_stm32::time::Hertz;
use embassy_stm32::{Config, SharedData, bind_interrupts, peripherals};
use embassy_stm32h755cm7_examples::glass::Glass;
use embassy_stm32h755cm7_examples::init_sdram;
use embassy_stm32h755cm7_examples::ui::Tui;
use embassy_time::Timer;
use mousefood::embedded_graphics::prelude::{DrawTarget, RgbColor};
use mousefood::{EmbeddedBackend, EmbeddedBackendConfig};
use ratatui::Terminal;
use {defmt_rtt as _, panic_probe as _};

extern crate alloc;

bind_interrupts!(
    struct Irqs {
        LTDC => ltdc::InterruptHandler<peripherals::LTDC>;
        DSI => dsihost::InterruptHandler<peripherals::DSIHOST>;
    }
);

#[unsafe(link_section = ".ram_d3")]
pub static SHARED_DATA: MaybeUninit<SharedData> = MaybeUninit::uninit();

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // DSI PLL configuration for 500MHz PHY clock.
    // The PLL input is hardwired to HSE, which on the STM32G747i-DISCO is 25MHz
    // VCO = 25 / idiv * 2 * ndiv = 25 / 5 * 2 * 100 = 1GHz
    // PLL_DSI = VCO / 2 / odiv = 1GHz / 2 / 1 = 500Mhz
    let dsi_pll = DsiHostPllConfig::new(100, DsiPllInput::Div5, DsiPllOutput::Div1);

    let mut config = Config::default();
    config.rcc.supply_config = embassy_stm32::rcc::SupplyConfig::DirectSMPS;
    config.rcc.voltage_scale = embassy_stm32::rcc::VoltageScale::Scale0;
    config.rcc.sys = embassy_stm32::rcc::Sysclk::PLL1_P;
    config.rcc.hse = Some(Hse {
        freq: Hertz::mhz(25),
        mode: embassy_stm32::rcc::HseMode::Bypass,
    });
    config.rcc.pll1 = Some(Pll {
        source: embassy_stm32::rcc::PllSource::HSE,
        prediv: embassy_stm32::rcc::PllPreDiv::DIV5,
        mul: embassy_stm32::rcc::PllMul::MUL192,
        divp: Some(embassy_stm32::rcc::PllDiv::DIV2),
        divq: Some(embassy_stm32::rcc::PllDiv::DIV8),
        divr: None,
    });

    config.rcc.pll2 = Some(Pll {
        source: embassy_stm32::rcc::PllSource::HSE,
        prediv: embassy_stm32::rcc::PllPreDiv::DIV5,
        mul: embassy_stm32::rcc::PllMul::MUL120,
        divp: Some(embassy_stm32::rcc::PllDiv::DIV3),
        divq: Some(embassy_stm32::rcc::PllDiv::DIV2),
        divr: Some(embassy_stm32::rcc::PllDiv::DIV3), // 200MHz SDRAM clock
    });

    config.rcc.pll3 = Some(Pll {
        source: embassy_stm32::rcc::PllSource::HSE,
        prediv: embassy_stm32::rcc::PllPreDiv::DIV5,
        mul: embassy_stm32::rcc::PllMul::MUL132,
        divp: Some(embassy_stm32::rcc::PllDiv::DIV2),
        divq: Some(embassy_stm32::rcc::PllDiv::DIV2),

        // LTDC is clocked from PLL3R
        // 30MHz pixel clock
        divr: Some(embassy_stm32::rcc::PllDiv::DIV22),
    });
    config.rcc.d1c_pre = embassy_stm32::rcc::AHBPrescaler::DIV1;
    config.rcc.ahb_pre = embassy_stm32::rcc::AHBPrescaler::DIV2;
    config.rcc.apb1_pre = embassy_stm32::rcc::APBPrescaler::DIV2;
    config.rcc.apb2_pre = embassy_stm32::rcc::APBPrescaler::DIV2;
    config.rcc.apb3_pre = embassy_stm32::rcc::APBPrescaler::DIV2;
    config.rcc.apb4_pre = embassy_stm32::rcc::APBPrescaler::DIV2;
    config.rcc.hsi48 = Some(Default::default());
    config.rcc.csi = true;
    config.rcc.mux.fmcsel = embassy_stm32::rcc::mux::Fmcsel::PLL2_R; // Use PLL2_R for SDRAM (100MHz)
    config.rcc.mux.dsisel = embassy_stm32::rcc::mux::Dsisel::DSI_PHY_DIV_8; // Use DSI PHY / 8 for byte lane clock (62.5MHz lane byte clock)
    config.rcc.dsi = Some(dsi_pll);

    let p = embassy_stm32::init_primary(config, &SHARED_DATA);

    let sdram = Fmc::sdram_a12bits_d32bits_4banks_bank2(
        p.FMC, // A0-A11
        p.PF0,
        p.PF1,
        p.PF2,
        p.PF3,
        p.PF4,
        p.PF5,
        p.PF12,
        p.PF13,
        p.PF14,
        p.PF15,
        p.PG0,
        p.PG1,
        // BA0-BA1
        p.PG4,
        p.PG5, // D0-D31
        p.PD14,
        p.PD15,
        p.PD0,
        p.PD1,
        p.PE7,
        p.PE8,
        p.PE9,
        p.PE10,
        p.PE11,
        p.PE12,
        p.PE13,
        p.PE14,
        p.PE15,
        p.PD8,
        p.PD9,
        p.PD10,
        p.PH8,
        p.PH9,
        p.PH10,
        p.PH11,
        p.PH12,
        p.PH13,
        p.PH14,
        p.PH15,
        p.PI0,
        p.PI1,
        p.PI2,
        p.PI3,
        p.PI6,
        p.PI7,
        p.PI9,
        p.PI10, // NBL0 - NBL3
        p.PE0,
        p.PE1,
        p.PI4,
        p.PI5,  // Control signals
        p.PH7,  // SDCKE1
        p.PG8,  // SDCLK
        p.PG15, // SDNCAS
        p.PH6,  // SDNE1 (!CS)
        p.PF11, // SDRAS
        p.PH5,  // SDNWE
        stm32_fmc::devices::is42s32800g_6::Is42s32800g {},
    );

    // Initialize SDRAM, MPU, framebuffers, and heap
    let cp = cortex_m::Peripherals::take().unwrap();
    let mut buffers = init_sdram(cp, sdram);

    // Reset display
    let mut dsi_reset = Output::new(p.PG3, embassy_stm32::gpio::Level::Low, embassy_stm32::gpio::Speed::Low);
    Timer::after_millis(120).await;
    dsi_reset.set_high();

    // Create DSI host using PJ2 as tearing input
    let mut dsi = DsiHost::new(p.DSIHOST, p.PJ2);
    let mut ltdc = Ltdc::new(p.LTDC);

    // Initialze LTDC using config from Glass
    ltdc.init(&Glass::ltdc_config());

    // Configure LTDC Layer 1 in ARGB8888 color mode with a rectangle covering panel active region
    let layer = LtdcLayerConfig {
        layer: LtdcLayer::Layer1,
        pixel_format: PixelFormat::ARGB8888,
        window_x0: 0,
        window_x1: Glass::ACTIVE_WIDTH,
        window_y0: 0,
        window_y1: Glass::ACTIVE_HEIGHT,
    };
    ltdc.init_layer(&layer, None);

    buffers.fb0.clear(RgbColor::BLACK).unwrap();

    info!("Set LTDC fb: {:x}", buffers.fb0.as_ptr());

    // Configure FB0 on LTDC Layer 1
    ltdc.set_buffer(LtdcLayer::Layer1, buffers.fb0.as_ptr().cast())
        .await
        .unwrap();

    // Initialize DSI PHY configuration struct
    let dsi_phy_config = DsiHostPhyConfig {
        lanes: DsiHostPhyLanes::Two,
        stop_wait_time: 10,
        acr: false,
        crc_rx: false,
        ecc_rx: false,
        eotp_rx: false,
        eotp_tx: false,
        bta: false,
        clock_hs2lp: 20,
        clock_lp2hs: 20,
        data_hs2lp: 20,
        data_lp2hs: 18,
        data_mrd: 0,
    };

    let mut video_config = DsiVideoConfig::default();
    video_config.mode = embassy_stm32::dsihost::DsiVideoMode::Burst;
    video_config.color = DsiColor::Rgb888;
    video_config.bta = false;
    video_config.lpcmd = true;
    video_config.lphbp = true;
    video_config.lphfp = true;
    video_config.lpva = true;
    video_config.lpvbp = true;
    video_config.lpvfp = true;
    video_config.lpvsa = true;

    // Start the panel
    dsi.start_panel::<Glass>(&dsi_phy_config, &DsiHostMode::Video(video_config))
        .await
        .unwrap();

    let backend = EmbeddedBackend::new(&mut buffers.fb0, EmbeddedBackendConfig::default());
    let mut terminal = Terminal::new(backend).unwrap();
    let mut tui = Tui::new();

    loop {
        terminal.draw(|frame| tui.draw(frame)).unwrap();
        ltdc.wait_line(ltdc.total_height()).await;
    }
}
