#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::time::Hertz;
use embassy_stm32::usb::HostDriver;
use embassy_stm32::{Config, bind_interrupts, peripherals, usb};
use embassy_usb_host::UsbHost;
use embassy_usb_host::class::cdc_acm::{CdcAcmHost, LineCoding};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    OTG_FS => usb::HostInterruptHandler<peripherals::USB_OTG_FS>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("USB Host Serial example");

    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hse = Some(Hse {
            freq: Hertz(8_000_000),
            mode: HseMode::Bypass,
        });
        config.rcc.pll_src = PllSource::HSE;
        config.rcc.pll = Some(Pll {
            prediv: PllPreDiv::DIV4,
            mul: PllMul::MUL168,
            divp: Some(PllPDiv::DIV2), // 168 MHz sysclk
            divq: Some(PllQDiv::DIV7), // 48 MHz USB clock
            divr: None,
        });
        config.rcc.ahb_pre = AHBPrescaler::DIV1;
        config.rcc.apb1_pre = APBPrescaler::DIV4;
        config.rcc.apb2_pre = APBPrescaler::DIV2;
        config.rcc.sys = Sysclk::PLL1_P;
        config.rcc.mux.clk48sel = mux::Clk48sel::PLL1_Q;
    }
    let p = embassy_stm32::init(config);

    info!("Initializing USB host...");

    // Create the host driver (FS mode)
    let driver = HostDriver::new_fs_host(p.USB_OTG_FS, Irqs, p.PA12, p.PA11);

    let mut host = UsbHost::new(driver);
    info!("USB host initialized, waiting for device...");

    loop {
        // Wait for a device to connect
        let speed = host.wait_for_connection().await;
        info!("Device connected at speed {:?}", speed);

        // Enumerate the device
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

        // Try to create a CDC ACM host driver
        let mut cdc = match CdcAcmHost::new(
            host.driver(),
            &config_buf[..config_len],
            addr,
            dev_desc.max_packet_size0 as u16,
        ) {
            Ok(c) => c,
            Err(e) => {
                error!("CDC ACM init failed: {:?}", e);
                continue;
            }
        };

        // Configure serial: 115200 8N1
        let coding = LineCoding::default();
        if let Err(e) = cdc.set_line_coding(&coding).await {
            error!("SET_LINE_CODING failed: {:?}", e);
            continue;
        }

        // Assert DTR
        if let Err(e) = cdc.set_control_line_state(true, false).await {
            error!("SET_CONTROL_LINE_STATE failed: {:?}", e);
            continue;
        }

        info!("CDC ACM ready, starting echo loop...");

        // Echo loop: read from USB serial, log, write back
        let mut buf = [0u8; 64];
        loop {
            match cdc.read(&mut buf).await {
                Ok(n) if n > 0 => {
                    info!("RX: {:x}", &buf[..n]);
                    if let Err(e) = cdc.write(&buf[..n]).await {
                        error!("TX failed: {:?}", e);
                        break;
                    }
                }
                Ok(_) => {}
                Err(e) => {
                    error!("RX failed: {:?}", e);
                    break;
                }
            }
        }

        info!("Device disconnected, waiting for next...");
    }
}
