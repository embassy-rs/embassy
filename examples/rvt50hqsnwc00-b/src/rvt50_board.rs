//! Board support for Riverdi STM32 embedded 5" displays (STM32U5A9NJH6Q).
//!
//! Default configuration matches [RVT50HQSNWN00](https://download.riverdi.com/RVT50HQSNWN00/DS_RVT50HQSNWN00_Rev.1.1.pdf):
//! 800×480 RGB565 panel, no touch panel.
//!
//! Pin assignments follow the
//! [BD_50STM32U5 Rev.1.1](https://download.riverdi.com/RVT50HQSNWN00/BD_50STM32U5_Rev.1.1.pdf)
//! schematic and the official
//! [LVGL Riverdi STM32U5 port](https://github.com/lvgl/lv_port_riverdi_stm32u5).

use defmt::info;
use embassy_stm32::gpio::{Level, Output, Pin, Speed};
use embassy_stm32::ltdc::{self, Ltdc, LtdcConfiguration, PolarityActive, PolarityEdge};
use embassy_stm32::time::Hertz;
use embassy_stm32::{bind_interrupts, can, peripherals, Config, Peri, Peripherals, rcc};
use embassy_time::{Duration, Timer};

#[cfg(feature = "touch")]
use embassy_stm32::i2c::{self, Config as I2cConfig, I2c};
#[cfg(feature = "touch")]
use embassy_stm32::mode::Blocking;

pub const DISPLAY_WIDTH: usize = 800;
pub const DISPLAY_HEIGHT: usize = 480;

/// Nominal CAN bitrate for the on-board TJA1441 transceiver (P5).
pub const CAN_BITRATE: u32 = 500_000;

/// Capacitive touch controller on I2C1 (7-bit address), touch-panel variants only.
#[cfg(feature = "touch")]
pub const TOUCH_I2C_ADDR: u8 = 0x41;

/// Board pins from BD_50STM32U5 Rev.1.1.
pub mod pins {
    //! Signal names as routed on the Riverdi 5" STM32U5 module.

    // LTDC RGB565 (RK050HR18)
    pub const LTDC_CLK: &str = "PD3";
    pub const LTDC_HSYNC: &str = "PE0";
    pub const LTDC_VSYNC: &str = "PD13";
    pub const LTDC_DE: &str = "PF11";

    pub const LCD_DISP_RESET: &str = "PH7";
    pub const BACKLIGHT_PWM: &str = "PB14"; // TIM15_CH1

    // CAN-FD on P5 (TJA1441AT/0Z)
    pub const CAN_RX: &str = "PB8"; // FDCAN1_RX
    pub const CAN_TX: &str = "PB9"; // FDCAN1_TX
    pub const CAN_STB: &str = "PI6"; // FDCAN_STB, active low

    #[cfg(feature = "touch")]
    pub const CTP_RST: &str = "PE3";
    #[cfg(feature = "touch")]
    pub const CTP_INT: &str = "PE6";
    #[cfg(feature = "touch")]
    pub const TOUCH_I2C_SCL: &str = "PG13";
    #[cfg(feature = "touch")]
    pub const TOUCH_I2C_SDA: &str = "PG14";
}

bind_interrupts!(pub struct Irqs {
    LTDC => ltdc::InterruptHandler<peripherals::LTDC>;
});

bind_interrupts!(pub struct CanIrqs {
    FDCAN1_IT0 => can::IT0InterruptHandler<peripherals::FDCAN1>;
    FDCAN1_IT1 => can::IT1InterruptHandler<peripherals::FDCAN1>;
});

/// Initialize system, LTDC, and FDCAN clocks for the Riverdi 5" panel (~25 MHz pixel clock).
pub fn init_clocks() -> Peripherals {
    let mut config = Config::default();

    config.rcc.hse = Some(rcc::Hse {
        freq: Hertz(16_000_000),
        mode: rcc::HseMode::Bypass,
    });

    // PLL1: 16 MHz * 10 = 160 MHz (sysclk and FDCAN kernel clock via PLL1Q)
    config.rcc.pll1 = Some(rcc::Pll {
        source: rcc::PllSource::Hse,
        prediv: rcc::PllPreDiv::Div1,
        mul: rcc::PllMul::Mul10,
        divp: None,
        divq: Some(rcc::PllDiv::Div1),
        divr: Some(rcc::PllDiv::Div1),
    });
    config.rcc.sys = rcc::Sysclk::Pll1R;
    config.rcc.mux.fdcan1sel = rcc::mux::Fdcansel::Pll1Q;

    // 16 MHz / 4 * 75 / 12 = 25 MHz LTDC clock (matches lv_port_riverdi_stm32u5)
    config.rcc.pll3 = Some(rcc::Pll {
        source: rcc::PllSource::Hse,
        prediv: rcc::PllPreDiv::Div4,
        mul: rcc::PllMul::Mul75,
        divp: None,
        divq: None,
        divr: Some(rcc::PllDiv::Div12),
    });
    config.rcc.mux.ltdcsel = rcc::mux::Ltdcsel::Pll3R;

    embassy_stm32::init(config)
}

pub fn enable_icache() {
    embassy_stm32::pac::ICACHE.cr().write(|w| {
        w.set_en(true);
    });
}

/// Riverdi RK050HR18 panel timing from lv_port_riverdi_stm32u5 / CubeMX.
pub fn ltdc_configuration() -> LtdcConfiguration {
    LtdcConfiguration {
        active_width: DISPLAY_WIDTH as _,
        active_height: DISPLAY_HEIGHT as _,
        h_sync: 4,
        h_back_porch: 8,
        h_front_porch: 8,
        v_sync: 4,
        v_back_porch: 8,
        v_front_porch: 8,
        h_sync_polarity: PolarityActive::ActiveLow,
        v_sync_polarity: PolarityActive::ActiveLow,
        data_enable_polarity: PolarityActive::ActiveLow,
        pixel_clock_polarity: PolarityEdge::FallingEdge,
    }
}

/// Bring the TJA1441 CAN transceiver out of standby (`FDCAN_STB` / PI6, active low).
pub fn enable_can_transceiver(stb: Peri<'static, impl Pin>) {
    let mut stb = Output::new(stb, Level::High, Speed::Low);
    stb.set_low();
}

/// Create an FDCAN1 configurator on the board CAN connector (P5).
pub fn init_can(
    fdcan: Peri<'static, peripherals::FDCAN1>,
    rx: Peri<'static, impl can::RxPin<peripherals::FDCAN1>>,
    tx: Peri<'static, impl can::TxPin<peripherals::FDCAN1>>,
    stb: Peri<'static, impl Pin>,
) -> can::CanConfigurator<'static> {
    enable_can_transceiver(stb);
    info!("FDCAN1 on {} / {} (transceiver enabled)", pins::CAN_RX, pins::CAN_TX);
    can::CanConfigurator::new(fdcan, rx, tx, CanIrqs)
}

#[cfg(feature = "touch")]
pub struct DisplayResources {
    pub ltdc: Ltdc<'static, peripherals::LTDC, ltdc::Rgb565>,
    pub i2c: I2c<'static, Blocking, i2c::Master>,
}

#[cfg(not(feature = "touch"))]
pub struct DisplayResources {
    pub ltdc: Ltdc<'static, peripherals::LTDC, ltdc::Rgb565>,
}

async fn reset_panel(reset: Peri<'static, impl Pin>) {
    let mut disp_reset = Output::new(reset, Level::Low, Speed::Low);
    disp_reset.set_high();
    Timer::after(Duration::from_millis(20)).await;
}

fn init_backlight(pin: Peri<'static, impl Pin>) {
    // Backlight (TIM15 CH1 / PB14) — full on via GPIO until PWM is wired up
    let mut backlight = Output::new(pin, Level::Low, Speed::Low);
    backlight.set_high();
}

/// Initialize LTDC, panel reset/backlight, and optionally touch I2C.
pub async fn init_display(p: Peripherals) -> DisplayResources {
    #[cfg(feature = "touch")]
    let Peripherals {
        LTDC,
        PD3,
        PE0,
        PD13,
        PF11,
        PD15,
        PD0,
        PD1,
        PE7,
        PE8,
        PE9,
        PE10,
        PE11,
        PE12,
        PE13,
        PE14,
        PD8,
        PD9,
        PD10,
        PD11,
        PD12,
        PH7,
        PB14,
        PE3,
        PG13,
        PG14,
        I2C1,
        ..
    } = p;

    #[cfg(not(feature = "touch"))]
    let Peripherals {
        LTDC,
        PD3,
        PE0,
        PD13,
        PF11,
        PD15,
        PD0,
        PD1,
        PE7,
        PE8,
        PE9,
        PE10,
        PE11,
        PE12,
        PE13,
        PE14,
        PD8,
        PD9,
        PD10,
        PD11,
        PD12,
        PH7,
        PB14,
        ..
    } = p;

    info!("Initializing LTDC (Riverdi RVT50 timing)...");

    reset_panel(PH7).await;
    init_backlight(PB14);

    let mut ltdc = Ltdc::<_, ltdc::Rgb565>::new_with_pins(
        LTDC,
        Irqs,
        PD3,  // CLK
        PE0,  // HSYNC
        PD13, // VSYNC
        PF11, // DE
        PD15, // B3
        PD0,  // B4
        PD1,  // B5
        PE7,  // B6
        PE8,  // B7
        PE9,  // G2
        PE10, // G3
        PE11, // G4
        PE12, // G5
        PE13, // G6
        PE14, // G7
        PD8,  // R3
        PD9,  // R4
        PD10, // R5
        PD11, // R6
        PD12, // R7
    );
    ltdc.init(&ltdc_configuration());

    #[cfg(feature = "touch")]
    {
        // Touch controller reset (PE3 / CTP_RST)
        let mut touch_reset = Output::new(PE3, Level::Low, Speed::Low);
        touch_reset.set_high();
        Timer::after(Duration::from_millis(10)).await;
        touch_reset.set_low();
        Timer::after(Duration::from_millis(10)).await;
        touch_reset.set_high();
        Timer::after(Duration::from_millis(10)).await;

        let i2c = I2c::new_blocking(I2C1, PG14, PG13, I2cConfig::default());
        info!("LTDC and touch I2C initialized");
        return DisplayResources { ltdc, i2c };
    }

    #[cfg(not(feature = "touch"))]
    {
        info!("LTDC initialized");
        DisplayResources { ltdc }
    }
}

#[derive(Clone, Copy, Default)]
pub struct TouchPoint {
    pub x: u16,
    pub y: u16,
    pub pressed: bool,
}

/// Read touch coordinates (FT5x06-style protocol from lv_port_riverdi_stm32u5).
#[cfg(feature = "touch")]
pub fn read_touch(i2c: &mut I2c<'static, Blocking, i2c::Master>) -> TouchPoint {
    let mut data = [0u8; 16];
    match i2c.blocking_write_read(TOUCH_I2C_ADDR, &[0x10], &mut data) {
        Ok(()) => {
            let x = u16::from(data[3] & 0x0F) << 8 | u16::from(data[2]);
            let y = u16::from(data[5] & 0x0F) << 8 | u16::from(data[4]);
            TouchPoint {
                x: x.min(DISPLAY_WIDTH as u16 - 1),
                y: y.min(DISPLAY_HEIGHT as u16 - 1),
                pressed: true,
            }
        }
        Err(_) => TouchPoint::default(),
    }
}
