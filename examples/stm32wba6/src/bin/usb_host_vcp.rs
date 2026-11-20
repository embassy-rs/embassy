//! USB Host VCP (CH34x & FTDI USB-to-UART bridges) example for NUCLEO-WBA65RI.
//!
//! Demonstrates using `embassy-usb` host drivers to enumerate and communicate
//! with CH340/CH341 and FTDI FT232R/FT2232/FT-X USB-to-serial converter ICs.
//!
//! Hardware: fit jumper JP1 (5V_USB) to supply VBUS to downstream devices.

#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::usb::HostDriver;
use embassy_stm32::{Config, bind_interrupts, peripherals, usb};
use embassy_usb::class::cdc_acm::{LineCoding, ParityType, StopBits};
use embassy_usb::class::vcp::host::ch34x::{self, Ch34xDevice};
use embassy_usb::class::vcp::host::ftdi::{self, FtdiDevice};
use embassy_usb::host::{BusRoute, BusState};
use panic_probe as _;

bind_interrupts!(struct Irqs {
    USB_OTG_HS => usb::HostInterruptHandler<peripherals::USB_OTG_HS>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("USB Host VCP (CH34x / FTDI) example starting...");

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
        config.rcc.mux.otghssel = mux::Otghssel::Pll1P;
    }
    let p = embassy_stm32::init(config);

    info!("Initializing USB host controller...");

    // Create the host driver (HS mode, internal PHY)
    let driver = HostDriver::new_hs_host(p.USB_OTG_HS, p.PD6, p.PD7, Irqs);

    static BUS_STATE: BusState = BusState::new();
    let (mut bus_ctrl, bus) = embassy_usb::host::bus(driver, &BUS_STATE);
    info!("USB host initialized, waiting for device connection...");

    loop {
        // Wait for a device to connect
        let speed = bus_ctrl.wait_for_connection().await;
        info!("Device connected at speed {:?}", speed);

        // Enumerate the device
        let mut config_buf = [0u8; 256];
        let result = bus.enumerate(BusRoute::Direct(speed), &mut config_buf).await;

        let (enum_info, config_len) = match result {
            Ok(r) => r,
            Err(e) => {
                error!("Enumeration failed: {:?}", e);
                continue;
            }
        };

        let vid = enum_info.device_desc.vendor_id;
        let pid = enum_info.device_desc.product_id;
        info!(
            "Enumerated device: VID={:04x} PID={:04x} addr={}",
            vid, pid, enum_info.device_address
        );

        let config_desc = &config_buf[..config_len];

        if vid == ch34x::id::VID_WCH {
            info!("Detected WCH CH34x device! Initializing driver...");
            let ch34x = match Ch34xDevice::new(&bus, &enum_info) {
                Ok(dev) => dev,
                Err(e) => {
                    error!("Ch34xDevice init failed: {:?}", e);
                    continue;
                }
            };

            let mut port = match ch34x.port(config_desc, 0) {
                Ok(p) => p,
                Err(e) => {
                    error!("Ch34xPort open failed: {:?}", e);
                    continue;
                }
            };

            if let Err(e) = port.enable().await {
                error!("Ch34x enable failed: {:?}", e);
                continue;
            }

            if let Err(e) = port.set_baud_rate(115_200).await {
                error!("Ch34x set_baud_rate failed: {:?}", e);
                continue;
            }

            let coding = LineCoding::new(115_200, StopBits::One, ParityType::None, 8);
            if let Err(e) = port.set_line_coding(&coding).await {
                error!("Ch34x set_line_coding failed: {:?}", e);
                continue;
            }

            info!("CH34x port ready (115200 8N1), starting echo loop...");
            let mut buf = [0u8; 64];
            loop {
                match port.read(&mut buf).await {
                    Ok(n) if n > 0 => {
                        info!("CH34x RX: {:x}", &buf[..n]);
                        if let Err(e) = port.write(&buf[..n]).await {
                            error!("CH34x TX failed: {:?}", e);
                            break;
                        }
                    }
                    Ok(_) => {}
                    Err(e) => {
                        error!("CH34x RX error: {:?}", e);
                        break;
                    }
                }
            }
        } else if vid == ftdi::id::VID_FTDI {
            info!("Detected FTDI device! Initializing driver...");
            let ftdi = match FtdiDevice::new(&bus, &enum_info) {
                Ok(dev) => dev,
                Err(e) => {
                    error!("FtdiDevice init failed: {:?}", e);
                    continue;
                }
            };

            let mut port = match ftdi.port(config_desc, 0) {
                Ok(p) => p,
                Err(e) => {
                    error!("FtdiPort open failed: {:?}", e);
                    continue;
                }
            };

            if let Err(e) = port.reset().await {
                error!("Ftdi reset failed: {:?}", e);
                continue;
            }

            if let Err(e) = port.set_baudrate(115_200).await {
                error!("Ftdi set_baudrate failed: {:?}", e);
                continue;
            }

            info!("FTDI port ready (115200 8N1), starting echo loop...");
            let mut buf = [0u8; 64];
            loop {
                match port.read(&mut buf).await {
                    Ok(n) if n > 0 => {
                        info!("FTDI RX: {:x}", &buf[..n]);
                        if let Err(e) = port.write(&buf[..n]).await {
                            error!("FTDI TX failed: {:?}", e);
                            break;
                        }
                    }
                    Ok(_) => {}
                    Err(e) => {
                        error!("FTDI RX error: {:?}", e);
                        break;
                    }
                }
            }
        } else {
            warn!("Unsupported VID/PID: {:04x}:{:04x}, expected CH34x or FTDI", vid, pid);
        }

        info!("Device disconnected, waiting for next...");
    }
}
