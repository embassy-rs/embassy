#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{AfType, Level, Output, OutputType, Speed};
use embassy_stm32::i2c::{self, I2c};
use embassy_stm32::time::mhz;
use embassy_stm32::{Config, bind_interrupts, dma, pac, peripherals, usb};
use embassy_time::Timer;
use embassy_usb_host::UsbHost;
use embassy_usb_host::class::hid::HidHost;
use {defmt_rtt as _, panic_probe as _};

pub use crate::pac::rcc::vals::Mcosel;

bind_interrupts!(struct Irqs {
    USB_UCPD1_2 => usb::USBHostInterruptHandler<peripherals::USB>;
    I2C1 => i2c::EventInterruptHandler<peripherals::I2C1>, i2c::ErrorInterruptHandler<peripherals::I2C1>;
    DMA1_CHANNEL1 => dma::InterruptHandler<peripherals::DMA1_CH1>;
    DMA1_CHANNEL2_3 => dma::InterruptHandler<peripherals::DMA1_CH2>;
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
            // fVCO = 8 MHz / 2 * 60 = 240 MHz
            // PLLR = 240 MHz / 4 = 60 MHz (sysclk)
            // PLLQ = 240 MHz / 5 = 48 MHz (USB)
            Pll {
                source: PllSource::HSE,
                prediv: PllPreDiv::DIV2,
                mul: PllMul::MUL60,
                divp: None,
                divq: Some(PllQDiv::DIV5),
                divr: Some(PllRDiv::DIV4),
            },
        );
        config.rcc.sys = Sysclk::PLL1_R;
        config.rcc.hsi48 = Some(Hsi48Config { sync_from_usb: true });
        config.rcc.mux.usbsel = mux::Usbsel::PLL1_Q;
    }

    let p = embassy_stm32::init(config);

    // Configure clock out (MCO = PLL1_Q)
    pac::RCC
        .cfgr()
        .modify(|w: &mut pac::rcc::regs::Cfgr| w.set_mco1sel(Mcosel::PLL1_Q));
    let mut mco = embassy_stm32::gpio::Flex::new(p.PA9);
    mco.set_as_af_unchecked(0, AfType::output(OutputType::PushPull, Speed::High));

    // NUCLEO-G0B1RE + X-NUCLEO-DRP1M1 USB-C expansion board:
    // Enable VBUS power via TCPP03-M20 IC over I2C.
    let mut enable = Output::new(p.PC8, Level::High, Speed::Low);
    enable.set_high();

    Timer::after_millis(1000).await;

    let mut i2c = I2c::new(p.I2C1, p.PB8, p.PB9, p.DMA1_CH1, p.DMA1_CH2, Irqs, Default::default());
    let i2c_address: u8 = 0x68 >> 1;

    // Turn on GDP switches to enable power from SOURCE
    i2c.write(i2c_address, &[0x00, 0b00011100]).await.unwrap();
    let mut read_buf = [0u8; 1];
    i2c.write_read(i2c_address, &[0x01], &mut read_buf).await.unwrap();
    debug!("TCPP03-M20 reg 1: {:02X}", read_buf[0]);

    // Create the USB host driver
    let mut usbhost = usb::UsbHost::new(p.USB, Irqs, p.PA12, p.PA11);
    usbhost.start();

    let mut host = UsbHost::new(usbhost);
    info!("USB host initialized, waiting for device...");

    loop {
        let speed = host.wait_for_connection().await;
        info!("Device connected at speed {:?}", speed);

        let mut config_buf = [0u8; 256];
        let result = host.enumerate(speed, &mut config_buf).await;

        let (dev_desc, addr, config_len) = match result {
            Ok(r) => r,
            Err(e) => {
                error!("Enumeration failed: {:?}", e);
                continue;
            }
        };

        info!(
            "Enumerated: VID={:04x} PID={:04x} addr={}",
            dev_desc.vendor_id, dev_desc.product_id, addr
        );

        let mut hid = match HidHost::new(
            host.driver(),
            &config_buf[..config_len],
            addr,
            dev_desc.max_packet_size0 as u16,
        ) {
            Ok(h) => h,
            Err(e) => {
                error!("HID init failed: {:?}", e);
                continue;
            }
        };

        if let Err(e) = hid.set_idle(0, 0).await {
            error!("SET_IDLE failed: {:?}", e);
            continue;
        }

        info!("HID device ready, reading reports...");

        let mut buf = [0u8; 64];
        loop {
            match hid.read(&mut buf).await {
                Ok(n) if n > 0 => {
                    info!("HID report: {:x}", &buf[..n]);
                }
                Ok(_) => {}
                Err(e) => {
                    error!("HID read failed: {:?}", e);
                    break;
                }
            }
        }

        info!("Device disconnected, waiting for next...");
    }
}
