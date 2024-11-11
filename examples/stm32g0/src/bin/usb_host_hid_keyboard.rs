#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{AfType, Level, Output, OutputType, Speed};
use embassy_stm32::i2c::{self, I2c};
use embassy_stm32::time::{mhz, Hertz};
use embassy_stm32::usb::UsbHost;
use embassy_stm32::{bind_interrupts, pac, peripherals, usb, Config};
use embassy_time::Timer;
use embassy_usb::handlers::kbd::KbdHandler;
use embassy_usb::handlers::UsbHostHandler;
use embassy_usb::host::UsbHostBusExt;
use embassy_usb_driver::host::DeviceEvent::Connected;
use embassy_usb_driver::host::UsbHostDriver;
use {defmt_rtt as _, panic_probe as _};

pub use crate::pac::rcc::vals::Mcosel;

bind_interrupts!(struct Irqs {
    USB_UCPD1_2 => usb::USBHostInterruptHandler<peripherals::USB>;
    I2C1 => i2c::EventInterruptHandler<peripherals::I2C1>, i2c::ErrorInterruptHandler<peripherals::I2C1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hse = Some(Hse {
            freq: mhz(8),
            mode: HseMode::Bypass,
        });

        config.rcc.pll = Some(
            // fVCO = fPLLIN × (N / M) = 8 MHz × (16 / 1) = 128 MHz
            // • fPLLP = fVCO / P
            // • fPLLQ = fVCO / Q
            // • fPLLR = fVCO / R
            // N = mul
            // M = prediv
            // PLLRCLK => system clock
            // PLLQCLK => USB
            // PLLPCLK => unused
            // Maximum VCO frequency is 344 MHz. For Range 1 (default)
            // 2.66 < PLL / M < 16
            // M = 2 => 8 / 2 = 4
            // N = 30 => 4 * 30 = 120
            // fVCO = 8Mhz / 2 * 60 = 240MHz
            // PLLR = 240MHz / 4 = 60MHz
            // PLLQ = 240MHz / 5 = 48MHz
            Pll {
                source: PllSource::HSE, // 8 Mhz
                prediv: PllPreDiv::DIV2,
                mul: PllMul::MUL60,
                divp: None,
                divq: Some(PllQDiv::DIV5), // PLLQ should be 48MHz
                divr: Some(PllRDiv::DIV4),
            },
        );
        config.rcc.sys = Sysclk::PLL1_R;
        config.rcc.hsi48 = Some(Hsi48Config { sync_from_usb: true });
        config.rcc.mux.usbsel = mux::Usbsel::PLL1_Q;
    }

    let p = embassy_stm32::init(config);

    // configure clock out
    pac::RCC
        .cfgr()
        .modify(|w: &mut pac::rcc::regs::Cfgr| w.set_mco1sel(Mcosel::PLL1_Q));
    // configure pin for clock out
    let mut mco = embassy_stm32::gpio::Flex::new(p.PA9);
    mco.set_as_af_unchecked(0, AfType::output(OutputType::PushPull, Speed::High));

    let mut led = Output::new(p.PA5, Level::High, Speed::Low);

    // This example assumes we're using the NUCLE0-G0B1RE together with X-NUCLEO-DRP1M1 USB-C expansion board.
    // We need to turn on USB C power delivery by interfacing with the TCPP03-M20 IC.
    // This is done setting enable high and by sending a command over I2C.
    // See the TCPP03-M20 datasheet for more details.

    let mut enable = Output::new(p.PC8, Level::High, Speed::Low);
    enable.set_high();

    Timer::after_millis(1000).await;

    // i2c
    // SCL: PB8
    // SDA: PB9
    let mut i2c = I2c::new(
        p.I2C1,
        p.PB8,
        p.PB9,
        Irqs,
        p.DMA1_CH1,
        p.DMA1_CH2,
        Hertz(100_000),
        Default::default(),
    );

    let i2c_address: u8 = 0x68 >> 1; //0b00110_100; // 7 bits address 0110 10x

    // We have to turn on the GDP switches to enable power from SOURCE
    let reg = 0;
    let value: u8 = 0b00011100;

    i2c.write(i2c_address, &[reg, value]).await.unwrap();

    let reg = 0x1;
    let mut read_buf = [255u8; 1];
    i2c.write_read(i2c_address, &[reg], &mut read_buf).await.unwrap();

    debug!("TCPP03-M20 Value 1: {:02X}", read_buf[0]);

    // Create the driver, from the HAL.
    let mut usbhost = UsbHost::new(p.USB, Irqs, p.PA12, p.PA11);

    // info!("Start USB driver");
    usbhost.start();
    // let mut host = UsbHost::new(driver);

    debug!("Detecting device");
    // Wait for root-port to detect device
    let speed = loop {
        match usbhost.wait_for_device_event().await {
            Connected(speed) => break speed,
            _ => {}
        }
    };

    println!("Found device with speed = {:?}", speed);

    let enum_info = usbhost.enumerate_root(speed, 1).await.unwrap();
    let mut kbd = KbdHandler::try_register(&usbhost, enum_info)
        .await
        .expect("Couldn't register keyboard");

    loop {
        let result = kbd.wait_for_event().await;
        debug!("{}", result);
    }
}
