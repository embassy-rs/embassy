#![no_std]
#![no_main]

use core::sync::atomic::{AtomicBool, AtomicU8, Ordering};

use defmt::*;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_futures::select::{Either, select};
use embassy_nrf::config::{ClockSpeed, Config as NrfConfig, HfclkSource};
use embassy_nrf::gpio::{Input, Pull};
use embassy_nrf::usb::vbus_detect::HardwareVbusDetect;
use embassy_nrf::usb::{self, Driver};
use embassy_nrf::{bind_interrupts, peripherals};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embassy_usb::class::hid::{
    HidBootProtocol, HidProtocolMode, HidReaderWriter, HidSubclass, ReportId, RequestHandler, State,
};
use embassy_usb::control::OutResponse;
use embassy_usb::{Builder, Config, Handler, UsbDeviceSpeed};
use static_cell::StaticCell;
use usbd_hid::descriptor::{KeyboardReport, SerializedDescriptor};
use {defmt_rtt as _, panic_probe as _};

static SUSPENDED: AtomicBool = AtomicBool::new(false);
static HID_PROTOCOL_MODE: AtomicU8 = AtomicU8::new(HidProtocolMode::Boot as u8);
static EP_OUT_BUFFER: StaticCell<[u8; 2048]> = StaticCell::new();

bind_interrupts!(pub struct Irqs {
    USBHS => usb::InterruptHandler<peripherals::USBHS>;
    VREGUSB => usb::vbus_detect::InterruptHandler;
});

pub type UsbDriver<'d> = Driver<'d, HardwareVbusDetect>;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = NrfConfig::default();
    config.hfclk_source = HfclkSource::ExternalXtal;
    config.clock_speed = ClockSpeed::CK64;
    let p = embassy_nrf::init(config);

    let driver = Driver::new(
        p.USBHS,
        Irqs,
        HardwareVbusDetect::new(Irqs),
        EP_OUT_BUFFER.init([0; 2048]),
        usb::Config::default(),
    );

    let mut config = Config::new(0xc0de, 0xcafe);
    config.manufacturer = Some("Embassy");
    config.product = Some("HID keyboard example");
    config.serial_number = Some("12345678");
    config.max_power = 100;
    config.max_packet_size_0 = 64;
    config.max_speed = UsbDeviceSpeed::High;
    config.supports_remote_wakeup = true;
    config.composite_with_iads = false;
    config.device_class = 0;
    config.device_sub_class = 0;
    config.device_protocol = 0;

    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    let mut msos_descriptor = [0; 256];
    let mut control_buf = [0; 256];
    let mut request_handler = MyRequestHandler {};
    let mut device_handler = MyDeviceHandler::new();
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
        report_descriptor: KeyboardReport::desc(),
        request_handler: None,
        poll_ms: 60,
        max_packet_size: 64,
        hid_subclass: HidSubclass::Boot,
        hid_boot_protocol: HidBootProtocol::Keyboard,
    };
    let hid = HidReaderWriter::<_, 1, 8>::new(&mut builder, &mut state, config);
    let mut usb = builder.build();

    let remote_wakeup: Signal<CriticalSectionRawMutex, _> = Signal::new();

    let usb_fut = async {
        loop {
            usb.run_until_suspend().await;
            match select(usb.wait_resume(), remote_wakeup.wait()).await {
                Either::First(_) => {}
                Either::Second(_) => {
                    if let Err(e) = usb.remote_wakeup().await {
                        warn!("remote wakeup failed: {:?}", e);
                    }
                }
            }
        }
    };

    let _button = Input::new(p.P1_26, Pull::Up);
    let (reader, mut writer) = hid.split();

    let in_fut = async {
        loop {
            // button.wait_for_low().await;
            embassy_time::Timer::after_secs(3).await;
            info!("PRESSED");

            if SUSPENDED.load(Ordering::Acquire) {
                info!("Triggering remote wakeup");
                remote_wakeup.signal(());
            } else if HID_PROTOCOL_MODE.load(Ordering::Relaxed) == HidProtocolMode::Boot as u8 {
                match writer.write(&[0, 0, 4, 0, 0, 0, 0, 0]).await {
                    Ok(()) => {}
                    Err(e) => warn!("Failed to send boot report: {:?}", e),
                }
            } else {
                let report = KeyboardReport {
                    keycodes: [4, 0, 0, 0, 0, 0],
                    leds: 0,
                    modifier: 0,
                    reserved: 0,
                };
                match writer.write_serialize(&report).await {
                    Ok(()) => {}
                    Err(e) => warn!("Failed to send report: {:?}", e),
                }
            }

            // button.wait_for_high().await;
            embassy_time::Timer::after_secs(1).await;
            info!("RELEASED");
            if HID_PROTOCOL_MODE.load(Ordering::Relaxed) == HidProtocolMode::Boot as u8 {
                match writer.write(&[0, 0, 0, 0, 0, 0, 0, 0]).await {
                    Ok(()) => {}
                    Err(e) => warn!("Failed to send boot report: {:?}", e),
                }
            } else {
                let report = KeyboardReport {
                    keycodes: [0, 0, 0, 0, 0, 0],
                    leds: 0,
                    modifier: 0,
                    reserved: 0,
                };
                match writer.write_serialize(&report).await {
                    Ok(()) => {}
                    Err(e) => warn!("Failed to send report: {:?}", e),
                }
            }
        }
    };

    let out_fut = async {
        reader.run(false, &mut request_handler).await;
    };

    join(usb_fut, join(in_fut, out_fut)).await;
}

struct MyDeviceHandler {
    configured: AtomicBool,
}

impl MyDeviceHandler {
    fn new() -> Self {
        MyDeviceHandler {
            configured: AtomicBool::new(false),
        }
    }
}

impl Handler for MyDeviceHandler {
    fn enabled(&mut self, enabled: bool) {
        self.configured.store(false, Ordering::Relaxed);
        SUSPENDED.store(false, Ordering::Release);
        if enabled {
            info!("Device enabled");
        } else {
            info!("Device disabled");
        }
    }

    fn reset(&mut self) {
        self.configured.store(false, Ordering::Relaxed);
        SUSPENDED.store(false, Ordering::Release);
        info!("Bus reset, the Vbus current limit is 100mA");
    }

    fn addressed(&mut self, addr: u8) {
        self.configured.store(false, Ordering::Relaxed);
        SUSPENDED.store(false, Ordering::Release);
        info!("USB address set to: {}", addr);
    }

    fn configured(&mut self, configured: bool) {
        self.configured.store(configured, Ordering::Relaxed);
        if configured {
            SUSPENDED.store(false, Ordering::Release);
        }
        if configured {
            info!("Device configured, it may now draw up to the configured current limit from Vbus.");
        } else {
            info!("Device is no longer configured, the Vbus current limit is 100mA.");
        }
    }

    fn suspended(&mut self, suspended: bool) {
        if suspended {
            info!(
                "Device suspended, the Vbus current limit is 500µA (or 2.5mA for high-power devices with remote wakeup enabled)."
            );
            SUSPENDED.store(true, Ordering::Release);
        } else {
            SUSPENDED.store(false, Ordering::Release);
            if self.configured.load(Ordering::Relaxed) {
                info!("Device resumed, it may now draw up to the configured current limit from Vbus");
            } else {
                info!("Device resumed, the Vbus current limit is 100mA");
            }
        }
    }
}

struct MyRequestHandler;

impl RequestHandler for MyRequestHandler {
    fn get_report(&mut self, id: ReportId, _buf: &mut [u8]) -> Option<usize> {
        info!("Get report for {:?}", id);
        None
    }

    fn set_report(&mut self, id: ReportId, data: &[u8]) -> OutResponse {
        info!("Set report for {:?}: {=[u8]}", id, data);
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
        info!("Set idle rate for {:?} to {:?}", id, dur);
    }

    fn get_idle_ms(&mut self, id: Option<ReportId>) -> Option<u32> {
        info!("Get idle rate for {:?}", id);
        None
    }
}
