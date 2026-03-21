#![no_std]
#![no_main]

use core::sync::atomic::{AtomicBool, AtomicU8, Ordering};

use defmt::*;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_nrf::config::{ClockSpeed, Config as NrfConfig, HfclkSource};
use embassy_nrf::usb::vbus_detect::HardwareVbusDetect;
use embassy_nrf::usb::{self, Driver};
use embassy_nrf::{Peri, bind_interrupts, peripherals};
use embassy_time::Timer;
use embassy_usb::class::hid::{
    HidBootProtocol, HidProtocolMode, HidReaderWriter, HidSubclass, ReportId, RequestHandler, State,
};
use embassy_usb::control::OutResponse;
use embassy_usb::{Builder, Config, Handler, UsbDeviceSpeed};
use static_cell::StaticCell;
use usbd_hid::descriptor::{MouseReport, SerializedDescriptor};
use {defmt_rtt as _, panic_probe as _};

static CONFIGURED: AtomicBool = AtomicBool::new(false);
static HID_PROTOCOL_MODE: AtomicU8 = AtomicU8::new(HidProtocolMode::Report as u8);
static EP_OUT_BUFFER: StaticCell<[u8; 2048]> = StaticCell::new();

bind_interrupts!(pub struct Irqs {
    USBHS => usb::InterruptHandler<peripherals::USBHS>;
    VREGUSB => usb::vbus_detect::InterruptHandler;
});

type UsbDriver<'d> = Driver<'d, HardwareVbusDetect>;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = init_board();

    let driver = usb_driver(p.USBHS, &mut EP_OUT_BUFFER.init([0; 2048])[..]);

    let mut config = Config::new(0xc0de, 0xcaf1);
    config.manufacturer = Some("Embassy");
    config.product = Some("HID mouse example");
    config.serial_number = Some("mouse-demo");
    config.max_power = 100;
    config.max_packet_size_0 = 64;
    config.max_speed = UsbDeviceSpeed::High;
    config.composite_with_iads = false;
    config.device_class = 0;
    config.device_sub_class = 0;
    config.device_protocol = 0;

    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    let mut msos_descriptor = [0; 256];
    let mut control_buf = [0; 256];
    let mut control_handler = MyRequestHandler {};
    let mut out_handler = MyRequestHandler {};
    let mut device_handler = MyDeviceHandler;
    let mut state = State::new();

    let mut builder = Builder::new(
        driver,
        config,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut msos_descriptor,
        &mut control_buf,
    );
    builder.handler(&mut device_handler);

    let config = embassy_usb::class::hid::Config {
        report_descriptor: MouseReport::desc(),
        request_handler: Some(&mut control_handler),
        poll_ms: 60,
        max_packet_size: 64,
        hid_subclass: HidSubclass::Boot,
        hid_boot_protocol: HidBootProtocol::Mouse,
    };

    let hid = HidReaderWriter::<_, 1, 8>::new(&mut builder, &mut state, config);
    let mut usb = builder.build();

    let usb_fut = usb.run();
    let (reader, mut writer) = hid.split();
    let hid_fut = async {
        let mut y: i8 = 5;
        loop {
            Timer::after_millis(500).await;
            if !CONFIGURED.load(Ordering::Acquire) {
                continue;
            }

            y = -y;

            if HID_PROTOCOL_MODE.load(Ordering::Relaxed) == HidProtocolMode::Boot as u8 {
                match writer.write(&[0, 0, y as u8]).await {
                    Ok(()) => {}
                    Err(e) => warn!("Failed to send boot report: {:?}", e),
                }
            } else {
                let report = MouseReport {
                    buttons: 0,
                    x: 0,
                    y,
                    wheel: 0,
                    pan: 0,
                };
                match writer.write_serialize(&report).await {
                    Ok(()) => {}
                    Err(e) => warn!("Failed to send report: {:?}", e),
                }
            }
        }
    };

    let out_fut = async {
        reader.run(false, &mut out_handler).await;
    };

    join(usb_fut, join(hid_fut, out_fut)).await;
}

struct MyDeviceHandler;

impl Handler for MyDeviceHandler {
    fn enabled(&mut self, enabled: bool) {
        if !enabled {
            CONFIGURED.store(false, Ordering::Release);
        }
    }

    fn reset(&mut self) {
        CONFIGURED.store(false, Ordering::Release);
    }

    fn configured(&mut self, configured: bool) {
        CONFIGURED.store(configured, Ordering::Release);
    }
}

struct MyRequestHandler;

impl RequestHandler for MyRequestHandler {
    fn get_report(&mut self, id: ReportId, _buf: &mut [u8]) -> Option<usize> {
        let _ = id;
        None
    }

    fn set_report(&mut self, id: ReportId, data: &[u8]) -> OutResponse {
        let _ = (id, data);
        OutResponse::Accepted
    }

    fn get_protocol(&self) -> HidProtocolMode {
        HidProtocolMode::from(HID_PROTOCOL_MODE.load(Ordering::Relaxed))
    }

    fn set_protocol(&mut self, protocol: HidProtocolMode) -> OutResponse {
        HID_PROTOCOL_MODE.store(protocol as u8, Ordering::Relaxed);
        OutResponse::Accepted
    }

    fn set_idle_ms(&mut self, id: Option<ReportId>, dur: u32) {
        let _ = (id, dur);
    }

    fn get_idle_ms(&mut self, id: Option<ReportId>) -> Option<u32> {
        let _ = id;
        None
    }
}

fn init_board() -> embassy_nrf::Peripherals {
    let mut config = NrfConfig::default();
    config.hfclk_source = HfclkSource::ExternalXtal;
    config.clock_speed = ClockSpeed::CK64;
    embassy_nrf::init(config)
}

fn usb_driver(usb: Peri<'static, peripherals::USBHS>, ep_out_buffer: &'static mut [u8]) -> UsbDriver<'static> {
    Driver::new(
        usb,
        Irqs,
        HardwareVbusDetect::new(Irqs),
        ep_out_buffer,
        usb::Config::default(),
    )
}
