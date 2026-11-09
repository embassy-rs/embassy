//! This example demonstrates a USB HID Gamepad running on an STM32WBA65 microcontroller.
//!
//! The device appears as a standard USB gamepad with 4 analog axes, an 8-way hat switch,
//! and 16 buttons.

#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_stm32::usb::Driver;
use embassy_stm32::{Config, bind_interrupts, peripherals, usb};
use embassy_time::Timer;
use embassy_usb::class::hid::device::{HidReaderWriter, RequestHandler, State};
use embassy_usb::class::hid::{GamepadReport, HidBootProtocol, HidSubclass, ReportId};
use embassy_usb::control::OutResponse;
use embassy_usb::{Builder, Handler};
use panic_probe as _;

bind_interrupts!(struct Irqs {
    USB_OTG_HS => usb::InterruptHandler<peripherals::USB_OTG_HS>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Starting USB HID Gamepad on STM32WBA65");

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
            frac: Some(0),             // Fractional part (disabled)
        });
        config.rcc.mux.otghssel = mux::Otghssel::Pll1P;
    }

    let p = embassy_stm32::init(config);

    // Create the USB OTG HS driver.
    let mut ep_out_buffer = [0u8; 1024];
    let mut usb_config = embassy_stm32::usb::Config::default();
    usb_config.vbus_detection = false;
    let driver = Driver::new_hs(p.USB_OTG_HS, p.PD6, p.PD7, Irqs, &mut ep_out_buffer, usb_config);

    // Create embassy-usb Config
    let mut usb_dev_config = embassy_usb::Config::new(0x1209, 0x0001);
    usb_dev_config.manufacturer = Some("Embassy");
    usb_dev_config.product = Some("STM32WBA65 USB Gamepad");
    usb_dev_config.serial_number = Some("12345678");
    usb_dev_config.max_power = 100;
    usb_dev_config.max_packet_size_0 = 64;

    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    let mut control_buf = [0; 128];
    let mut request_handler = MyRequestHandler {};
    let mut device_handler = MyDeviceHandler::new();

    let mut state = State::new();

    let mut builder = Builder::new(
        driver,
        usb_dev_config,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut [],
        &mut control_buf,
    );

    builder.handler(&mut device_handler);

    let hid_config = embassy_usb::class::hid::device::Config {
        report_descriptor: GamepadReport::DESCRIPTOR,
        request_handler: None,
        poll_ms: 10,
        max_packet_size: 64,
        hid_subclass: HidSubclass::No,
        hid_boot_protocol: HidBootProtocol::None,
    };
    let hid = HidReaderWriter::<_, 1, 8>::new(&mut builder, &mut state, hid_config);

    let mut usb = builder.build();
    let usb_fut = usb.run();

    let (reader, mut writer) = hid.split();

    let in_fut = async {
        let mut step = 0;
        loop {
            Timer::after_millis(500).await;

            let report = match step % 5 {
                0 => GamepadReport {
                    buttons: GamepadReport::BUTTON_A,
                    hat: GamepadReport::HAT_CENTER,
                    ..Default::default()
                },
                1 => GamepadReport {
                    buttons: GamepadReport::BUTTON_B,
                    hat: GamepadReport::HAT_UP,
                    ..Default::default()
                },
                2 => GamepadReport {
                    x: 100,
                    y: -100,
                    hat: GamepadReport::HAT_RIGHT,
                    buttons: GamepadReport::BUTTON_X,
                    ..Default::default()
                },
                3 => GamepadReport {
                    z: -80,
                    rz: 80,
                    hat: GamepadReport::HAT_DOWN,
                    buttons: GamepadReport::BUTTON_Y | GamepadReport::BUTTON_START,
                    ..Default::default()
                },
                _ => GamepadReport::default(),
            };

            step += 1;
            if let Err(e) = writer.write(&report.to_bytes()).await {
                warn!("Failed to send gamepad report: {:?}", e);
            }
        }
    };

    let out_fut = async {
        reader.run(false, &mut request_handler).await;
    };

    join(usb_fut, join(in_fut, out_fut)).await;
}

struct MyRequestHandler {}

impl RequestHandler for MyRequestHandler {
    fn get_report(&mut self, id: ReportId, _buf: &mut [u8]) -> Option<usize> {
        info!("Get report for {:?}", id);
        None
    }

    fn set_report(&mut self, id: ReportId, data: &[u8]) -> OutResponse {
        info!("Set report for {:?}: {=[u8]}", id, data);
        OutResponse::Accepted
    }

    fn set_idle_ms(&mut self, id: Option<ReportId>, duration_ms: u32) {
        info!("Set idle rate for {:?} to {:?}", id, duration_ms);
    }

    fn get_idle_ms(&mut self, id: Option<ReportId>) -> Option<u32> {
        info!("Get idle rate for {:?}", id);
        None
    }
}

struct MyDeviceHandler {
    configured: bool,
}

impl MyDeviceHandler {
    fn new() -> Self {
        MyDeviceHandler { configured: false }
    }
}

impl Handler for MyDeviceHandler {
    fn enabled(&mut self, enabled: bool) {
        self.configured = false;
        info!("Device enabled: {:?}", enabled);
    }

    fn reset(&mut self) {
        self.configured = false;
        info!("Bus reset");
    }

    fn addressed(&mut self, addr: u8) {
        self.configured = false;
        info!("USB addressed: {:?}", addr);
    }

    fn configured(&mut self, configured: bool) {
        self.configured = configured;
        info!("Device configured: {:?}", configured);
    }

    fn suspended(&mut self, suspended: bool) {
        info!("Device suspended: {:?}", suspended);
    }
}
