//! Board support for Riverdi STM32 embedded 5" displays (STM32U5A9NJH6Q).
//!
//! Default configuration matches [RVT50HQSNWN00](https://download.riverdi.com/RVT50HQSNWN00/DS_RVT50HQSNWN00_Rev.1.1.pdf):
//! 800×480 RGB565 panel, no touch panel.
//!
//! Pin assignments follow the
//! [BD_50STM32U5 Rev.1.1](https://download.riverdi.com/RVT50HQSNWN00/BD_50STM32U5_Rev.1.1.pdf)
//! schematic and the Riverdi STM32U5 reference design.

use defmt::info;
use embassy_stm32::exti::{self, ExtiInput};
use embassy_stm32::gpio::{Input, Level, Output, Pin, Pull, Speed};
use embassy_stm32::interrupt;
use embassy_stm32::ltdc::{self, Ltdc, LtdcConfiguration, PolarityActive, PolarityEdge};
use embassy_stm32::mode::Async;
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

/// Standard CAN ID for the periodic demo pattern transmitted by `can_raw`.
pub const CAN_PATTERN_TX_ID: u16 = 0x123;

/// Standard CAN ID for LED state frames received by `can_raw`.
///
/// Payload: byte 0 bit 0 — `1` = LED on, `0` = LED off.
pub const CAN_LED_STATE_RX_ID: u16 = 0x124;

/// Interval between pattern frames in `can_raw`.
pub const CAN_PATTERN_INTERVAL_MS: u64 = 500;

/// Build the 8-byte demo pattern: signature bytes plus a rotating walk bit.
pub fn can_pattern_payload(seq: u8) -> [u8; 8] {
    [seq, 0xDE, 0xAD, 0xBE, 0xEF, 0x55, 0xAA, 1 << (seq % 8)]
}

/// Extract the standard CAN ID from a received frame, if present.
pub fn can_frame_standard_id(frame: &can::frame::Frame) -> Option<u16> {
    match frame.header().id() {
        embedded_can::Id::Standard(id) => Some(id.as_raw()),
        _ => None,
    }
}

/// Parse LED on/off from a `CAN_LED_STATE_RX_ID` payload.
pub fn can_led_state_from_payload(data: &[u8]) -> Option<bool> {
    data.first().map(|byte| byte & 1 != 0)
}

/// Capacitive touch controller on I2C1 (7-bit address), touch-panel variants only.
#[cfg(feature = "touch")]
pub const TOUCH_I2C_ADDR: u8 = 0x41;

/// Board layout and pin assignments from BD_50STM32U5 Rev.1.1.
pub mod pins {
    //! MCU pin map for the Riverdi 5" STM32U5 embedded display module.
    //!
    //! ```text
    //!                    +-------- MCU (STM32U5A9NJH6Q) --------+
    //!                    |                                      |
    //!   LCD + CTP -------+ LTDC / I2C1 (touch variants)         |
    //!   BACKLIGHT -------+ PB14 (TIM15 PWM)                     |
    //!   CAN (P5) --------+ PB8/PB9 + PI6 (TJA1441)              |
    //!   RS485 (P7) ------+ USART6 + PE4 (ST3485)                |
    //!   USB (P6) --------+ PA11/PA12 (AP2265)                   |
    //!   SWD (P3) --------+ PA13/PA14/PB3                        |
    //!   microSD ---------+ SDMMC1                                |
    //!   RiBUS -----------+ SPI (PI1..PI3, PG1)                  |
    //!   Expansion (P4) --+ (see datasheet)                       |
    //!   User button -----+ PH3 (BOOT0)                           |
    //!   User LED --------+ PE5                                     |
    //!                    +--------------------------------------+
    //! ```

    // --- Display (LTDC RGB565, RK050HR18) ---
    pub const LTDC_CLK: &str = "PD3";
    pub const LTDC_HSYNC: &str = "PE0";
    pub const LTDC_VSYNC: &str = "PD13";
    pub const LTDC_DE: &str = "PF11";
    pub const LCD_DISP_RESET: &str = "PH7";
    pub const BACKLIGHT_PWM: &str = "PB14"; // TIM15_CH1

    // --- User I/O ---
    //
    // `PH3` is shared with the MCU boot strap `BOOT0` (user button S1 on the board).
    // By default the STM32U5 samples the boot mode from the PH3 pin at reset.
    //
    // To use PH3 reliably as GPIO or EXTI (instead of as a live boot strap), program
    // the user option byte `nSWBOOT0` in `FLASH->OPTR` to **0** once per device:
    //
    // 1. **Set `nSWBOOT0 = 0`** — BOOT0 is taken from the `nBOOT0` option bit, not PH3.
    // 2. **Set `nBOOT0` as needed** — typically `0` to keep booting from internal flash.
    // 3. **Launch option bytes** — e.g. via STM32CubeProgrammer (*OB* tab) or
    //    `HAL_FLASH_OB_Launch()` after programming.
    //
    // After that, the PH3 pad is no longer wired to the boot loader path and can be
    // configured as a normal GPIO input (as in `init_user_button_input()` /
    // `init_user_button()`). See the README section *User button (PH3 / BOOT0)*.
    pub const USER_BUTTON: &str = "PH3"; // BOOT0
    pub const USER_LED: &str = "PE5";

    // --- CAN on P5 (TJA1441AT/0Z) ---
    pub const CAN_RX: &str = "PB8";  // FDCAN1_RX
    pub const CAN_TX: &str = "PB9";  // FDCAN1_TX
    pub const CAN_STB: &str = "PI6"; // FDCAN_STB, active low

    // --- RS485 on P7 (ST3485E, USART6) ---
    pub const RS485_TX: &str = "PE1";  // USART6_TX
    pub const RS485_RX: &str = "PJ4";  // USART6_RX
    pub const RS485_DE: &str = "PE4";  // USART6_DE

    // --- USB on P6 (AP2265) ---
    pub const USB_DM: &str = "PA11";
    pub const USB_DP: &str = "PA12";

    // --- SWD on P3 ---
    pub const SWDIO: &str = "PA13";
    pub const SWCLK: &str = "PA14";
    pub const SWO: &str = "PB3";

    // --- RiBUS (SPI) ---
    pub const RIBUS_SCK: &str = "PI1";
    pub const RIBUS_MISO: &str = "PI2";
    pub const RIBUS_MOSI: &str = "PI3";
    pub const RIBUS_CS: &str = "PG1";

    // --- microSD (SDMMC1) ---
    pub const SD_D0: &str = "PC8";
    pub const SD_D1: &str = "PC9";
    pub const SD_D2: &str = "PC10";
    pub const SD_D3: &str = "PC11";
    pub const SD_CK: &str = "PC12";
    pub const SD_CMD: &str = "PD2";

    pub const CTP_RST: &str = "PE3";
    pub const CTP_INT: &str = "PE6";
    pub const TOUCH_I2C_SCL: &str = "PG13";
    pub const TOUCH_I2C_SDA: &str = "PG14";
}

bind_interrupts!(pub struct Irqs {
    LTDC => ltdc::InterruptHandler<peripherals::LTDC>;
});

bind_interrupts!(pub struct CanIrqs {
    FDCAN1_IT0 => can::IT0InterruptHandler<peripherals::FDCAN1>;
    FDCAN1_IT1 => can::IT1InterruptHandler<peripherals::FDCAN1>;
});

bind_interrupts!(pub struct ButtonIrqs {
    EXTI3 => exti::InterruptHandler<interrupt::typelevel::EXTI3>;
});

/// Interrupt binding for the capacitive touch panel INT line (`CTP_INT` / PE6).
#[cfg(feature = "touch")]
bind_interrupts!(pub struct TouchIrqs {
    EXTI6 => exti::InterruptHandler<interrupt::typelevel::EXTI6>;
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

    // 16 MHz / 4 * 75 / 12 = 25 MHz LTDC clock (matches Riverdi CubeMX reference)
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

/// Riverdi RK050HR18 panel timing from the Riverdi CubeMX reference.
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

/// User push button on `PH3` / `BOOT0` as plain GPIO input (active high).
///
/// Uses `Pull::None` because the board already provides a BOOT0 pull-down.
/// Until `nSWBOOT0` is programmed to `0` (see [`pins::USER_BUTTON`] docs), PH3
/// remains a boot strap sampled at reset; `Input::new` still reconfigures the pad
/// as a digital input at runtime.
pub fn init_user_button_input(pin: Peri<'static, peripherals::PH3>) -> Input<'static> {
    Input::new(pin, Pull::None)
}

/// User push button on `PH3` / `BOOT0` (active high, EXTI on rising edge).
pub fn init_user_button(
    pin: Peri<'static, peripherals::PH3>,
    exti: Peri<'static, peripherals::EXTI3>,
) -> ExtiInput<'static, Async> {
    ExtiInput::new(pin, exti, Pull::Down, ButtonIrqs)
}

/// User LED on `PE5` (active high).
pub fn init_user_led(pin: Peri<'static, peripherals::PE5>) -> Output<'static> {
    Output::new(pin, Level::Low, Speed::Low)
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

pub struct DisplayResources {
    pub ltdc: Ltdc<'static, peripherals::LTDC, ltdc::Rgb565>,
    #[cfg(feature = "touch")]
    pub i2c: I2c<'static, Blocking, i2c::Master>,
    /// EXTI input on `CTP_INT` (PE6, active-low). Asserts on touch events.
    #[cfg(feature = "touch")]
    pub touch_int: ExtiInput<'static, Async>,
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

/// Initialize LTDC, panel reset, backlight, and optionally touch I2C.
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
        PE6,
        EXTI6,
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
        let mut touch_reset = Output::new(PE3, Level::Low, Speed::Low);
        touch_reset.set_high();
        Timer::after(Duration::from_millis(10)).await;
        touch_reset.set_low();
        Timer::after(Duration::from_millis(10)).await;
        touch_reset.set_high();
        Timer::after(Duration::from_millis(10)).await;

        let i2c = I2c::new_blocking(I2C1, PG14, PG13, I2cConfig::default());
        // CTP_INT (PE6) is active-low: controller pulls it down on each touch event.
        let touch_int = ExtiInput::new(PE6, EXTI6, Pull::Up, TouchIrqs);
        info!("LTDC, touch I2C and CTP_INT interrupt initialized");
        return DisplayResources { ltdc, i2c, touch_int };
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
    /// `true` when the last I2C read succeeded.
    pub i2c_ok: bool,
    /// First byte returned by the touch read (register 0x10), for debug logging.
    pub raw_status: u8,
}

/// Read touch coordinates (Riverdi reference port / `lvgl_touch` demo protocol).
///
/// Register `0x10` is read over I2C; contact is inferred from the status byte
/// and coordinates (not from I2C success alone). Idle reads return `raw` `0x00`
/// or `0xFE` and coordinates parked at the panel edge.
#[cfg(feature = "touch")]
pub fn read_touch(i2c: &mut I2c<'static, Blocking, i2c::Master>) -> TouchPoint {
    let mut data = [0u8; 16];
    match i2c.blocking_write_read(TOUCH_I2C_ADDR, &[0x10], &mut data) {
        Ok(()) => {
            let raw_status = data[0];
            let x = u16::from(data[3] & 0x0F) << 8 | u16::from(data[2]);
            let y = u16::from(data[5] & 0x0F) << 8 | u16::from(data[4]);
            let x = x.min(DISPLAY_WIDTH as u16 - 1);
            let y = y.min(DISPLAY_HEIGHT as u16 - 1);
            let pressed = touch_is_active(raw_status, x, y);
            TouchPoint {
                x,
                y,
                pressed,
                i2c_ok: true,
                raw_status,
            }
        }
        Err(_) => TouchPoint::default(),
    }
}

#[cfg(feature = "touch")]
fn touch_is_active(raw_status: u8, x: u16, y: u16) -> bool {
    if raw_status == 0x00 || raw_status == 0xFE {
        return false;
    }
    // Idle reads park at panel edges (observed: top-right (799,0), corners).
    if x == 0 && y == 0 {
        return false;
    }
    if x == DISPLAY_WIDTH as u16 - 1 && y == 0 {
        return false;
    }
    if x == DISPLAY_WIDTH as u16 - 1 && y == DISPLAY_HEIGHT as u16 - 1 {
        return false;
    }
    true
}
