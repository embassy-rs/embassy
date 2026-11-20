//! This example demonstrates a Composite USB HID device running on an STM32WBA65 microcontroller.
//!
//! Using a single USB interface and interrupt endpoint, this device exposes:
//! - Keyboard (Report ID 1)
//! - Mouse (Report ID 2)
//! - Consumer Control / Multimedia keys (Report ID 3)
//! - Gamepad (Report ID 4)

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
use embassy_usb::class::hid::{
    COMPOSITE_DESCRIPTOR, ConsumerReport, GamepadReport, HidBootProtocol, HidSubclass, IdConsumerReport,
    IdGamepadReport, IdKeyboardReport, IdMouseReport, KeyboardReport, MouseReport, ReportId,
};
use embassy_usb::control::OutResponse;
use embassy_usb::{Builder, Handler};
use panic_probe as _;

bind_interrupts!(struct Irqs {
    USB_OTG_HS => usb::InterruptHandler<peripherals::USB_OTG_HS>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Starting Composite USB HID Device on STM32WBA65");

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
    let mut usb_dev_config = embassy_usb::Config::new(0x1209, 0x0002);
    usb_dev_config.manufacturer = Some("Embassy");
    usb_dev_config.product = Some("STM32WBA65 Composite HID");
    usb_dev_config.serial_number = Some("12345678");
    usb_dev_config.max_power = 100;
    usb_dev_config.max_packet_size_0 = 64;

    let mut config_descriptor = [0; 512];
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
        report_descriptor: COMPOSITE_DESCRIPTOR,
        request_handler: None,
        poll_ms: 10,
        max_packet_size: 64,
        hid_subclass: HidSubclass::No,
        hid_boot_protocol: HidBootProtocol::None,
    };
    let hid = HidReaderWriter::<_, 1, 16>::new(&mut builder, &mut state, hid_config);

    let mut usb = builder.build();
    let usb_fut = usb.run();

    let (reader, mut writer) = hid.split();

    let in_fut = async {
        let mut step = 0;
        loop {
            Timer::after_secs(1).await;

            match step % 4 {
                0 => {
                    // Send a Consumer Control report: Volume Up then release
                    info!("Sending Consumer Volume Increment");
                    let vol_up = IdConsumerReport::new(ConsumerReport::from_key(ConsumerReport::VOLUME_INCREMENT));
                    _ = writer.write(&vol_up.to_bytes()).await;
                    Timer::after_millis(50).await;
                    let release = IdConsumerReport::new(ConsumerReport::release());
                    _ = writer.write(&release.to_bytes()).await;
                }
                1 => {
                    // Send a Mouse movement report (small right/down wiggle)
                    info!("Sending Mouse Report");
                    let mouse = IdMouseReport::new(MouseReport {
                        buttons: 0,
                        x: 10,
                        y: 10,
                        wheel: 0,
                    });
                    _ = writer.write(&mouse.to_bytes()).await;
                }
                2 => {
                    // Send a Gamepad button press
                    info!("Sending Gamepad Report");
                    let gamepad = IdGamepadReport::new(GamepadReport {
                        buttons: GamepadReport::BUTTON_A,
                        hat: GamepadReport::HAT_UP,
                        ..Default::default()
                    });
                    _ = writer.write(&gamepad.to_bytes()).await;
                    Timer::after_millis(50).await;
                    let release = IdGamepadReport::new(GamepadReport::default());
                    _ = writer.write(&release.to_bytes()).await;
                }
                _ => {
                    // Send a Keyboard key press ('a' = 0x04)
                    info!("Sending Keyboard Report");
                    let kbd = IdKeyboardReport::new(KeyboardReport {
                        modifiers: 0,
                        keycodes: [0x04, 0, 0, 0, 0, 0],
                    });
                    _ = writer.write(&kbd.to_bytes()).await;
                    Timer::after_millis(50).await;
                    let release = IdKeyboardReport::new(KeyboardReport::default());
                    _ = writer.write(&release.to_bytes()).await;
                }
            }

            step += 1;
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
