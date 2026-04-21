// USB Host HID example for NUCLEO-WBA65RI.
// Hardware: fit jumper JP1 (5V_USB) to supply VBUS to the downstream device.

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::usb::HostDriver;
use embassy_stm32::{Config, bind_interrupts, peripherals, usb};
use embassy_usb_host::class::hid::HidHost;
use embassy_usb_host::{BusRoute, UsbHost};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    USB_OTG_HS => usb::HostInterruptHandler<peripherals::USB_OTG_HS>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("USB Host HID example");

    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.pll1 = Some(Pll {
            source: PllSource::Hsi,
            prediv: PllPreDiv::Div1,   // PLLM = 1 → HSI / 1 = 16 MHz
            mul: PllMul::Mul30,        // PLLN = 30 → 16 MHz * 30 = 480 MHz VCO
            divr: Some(PllDiv::Div5),  // PLLR = 5 → 96 MHz (Sysclk)
            divq: Some(PllDiv::Div10), // PLLQ = 10 → 48 MHz
            divp: Some(PllDiv::Div30), // PLLP = 30 → 16 MHz (USB_OTG_HS)
            frac: Some(0),
        });

        config.rcc.ahb_pre = AHBPrescaler::Div1;
        config.rcc.apb1_pre = APBPrescaler::Div1;
        config.rcc.apb2_pre = APBPrescaler::Div1;
        config.rcc.apb7_pre = APBPrescaler::Div1;
        config.rcc.ahb5_pre = AHB5Prescaler::Div4;

        config.rcc.voltage_scale = VoltageScale::Range1;
        config.rcc.mux.otghssel = mux::Otghssel::Pll1P;
        config.rcc.sys = Sysclk::Pll1R;
    }
    let p = embassy_stm32::init(config);

    info!("Initializing USB host...");

    // Create the host driver (HS mode, internal PHY)
    let driver = HostDriver::new_hs_host(p.USB_OTG_HS, Irqs, p.PD6, p.PD7);
    let mut host = UsbHost::new(driver);
    info!("USB host initialized, waiting for device...");

    loop {
        // Wait for a device to connect
        let speed = host.wait_for_connection().await;
        info!("Device connected at speed {:?}", speed);

        // Enumerate the device
        let mut config_buf = [0u8; 256];
        let result = host.enumerate(BusRoute::Direct(speed), &mut config_buf).await;

        let (enum_info, config_len) = match result {
            Ok(r) => r,
            Err(e) => {
                error!("Enumeration failed: {:?}", e);
                continue;
            }
        };

        info!(
            "Enumerated: VID={:04x} PID={:04x} addr={}",
            enum_info.device_desc.vendor_id, enum_info.device_desc.product_id, enum_info.device_address
        );

        // Try to create a HID host driver
        let mut hid = match HidHost::new(host.driver(), &config_buf[..config_len], &enum_info) {
            Ok(h) => h,
            Err(e) => {
                error!("HID init failed: {:?}", e);
                continue;
            }
        };

        // Disable idle repeat (STALL-tolerant: some devices don't support SET_IDLE)
        if let Err(e) = hid.set_idle(0, 0).await {
            error!("SET_IDLE failed: {:?}", e);
            continue;
        }

        info!("HID device ready, reading reports...");

        // Read loop: log raw HID input reports
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
