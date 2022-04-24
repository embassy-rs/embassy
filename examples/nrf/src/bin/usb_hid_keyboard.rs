#![no_std]
#![no_main]
#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]

use core::mem;
use core::sync::atomic::{AtomicBool, Ordering};
use defmt::*;
use embassy::channel::Signal;
use embassy::executor::Spawner;
use embassy::interrupt::InterruptExt;
use embassy::time::Duration;
use embassy::util::{select, select3, Either, Either3};
use embassy_nrf::gpio::{Input, Pin, Pull};
use embassy_nrf::interrupt;
use embassy_nrf::pac;
use embassy_nrf::usb::Driver;
use embassy_nrf::Peripherals;
use embassy_usb::control::OutResponse;
use embassy_usb::{Builder, Config, DeviceStateHandler};
use embassy_usb_hid::{HidReaderWriter, ReportId, RequestHandler, State};
use futures::future::join;
use usbd_hid::descriptor::{KeyboardReport, SerializedDescriptor};

use defmt_rtt as _; // global logger
use panic_probe as _;

static ENABLE_USB: Signal<bool> = Signal::new();
static SUSPENDED: AtomicBool = AtomicBool::new(false);

fn on_power_interrupt(_: *mut ()) {
    let regs = unsafe { &*pac::POWER::ptr() };

    if regs.events_usbdetected.read().bits() != 0 {
        regs.events_usbdetected.reset();
        info!("Vbus detected, enabling USB...");
        ENABLE_USB.signal(true);
    }

    if regs.events_usbremoved.read().bits() != 0 {
        regs.events_usbremoved.reset();
        info!("Vbus removed, disabling USB...");
        ENABLE_USB.signal(false);
    }
}

#[embassy::main]
async fn main(_spawner: Spawner, p: Peripherals) {
    let clock: pac::CLOCK = unsafe { mem::transmute(()) };
    let power: pac::POWER = unsafe { mem::transmute(()) };

    info!("Enabling ext hfosc...");
    clock.tasks_hfclkstart.write(|w| unsafe { w.bits(1) });
    while clock.events_hfclkstarted.read().bits() != 1 {}

    // Create the driver, from the HAL.
    let irq = interrupt::take!(USBD);
    let driver = Driver::new(p.USBD, irq);

    // Create embassy-usb Config
    let mut config = Config::new(0xc0de, 0xcafe);
    config.manufacturer = Some("Embassy");
    config.product = Some("HID keyboard example");
    config.serial_number = Some("12345678");
    config.max_power = 100;
    config.max_packet_size_0 = 64;
    config.supports_remote_wakeup = true;

    // Create embassy-usb DeviceBuilder using the driver and config.
    // It needs some buffers for building the descriptors.
    let mut device_descriptor = [0; 256];
    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    let mut control_buf = [0; 16];
    let request_handler = MyRequestHandler {};
    let device_state_handler = MyDeviceStateHandler::new();

    let mut state = State::new();

    let mut builder = Builder::new(
        driver,
        config,
        &mut device_descriptor,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut control_buf,
        Some(&device_state_handler),
    );

    // Create classes on the builder.
    let config = embassy_usb_hid::Config {
        report_descriptor: KeyboardReport::desc(),
        request_handler: Some(&request_handler),
        poll_ms: 60,
        max_packet_size: 64,
    };
    let hid = HidReaderWriter::<_, 1, 8>::new(&mut builder, &mut state, config);

    // Build the builder.
    let mut usb = builder.build();

    let remote_wakeup = Signal::new();

    // Run the USB device.
    let usb_fut = async {
        enable_command().await;
        loop {
            match select(usb.run_until_suspend(), ENABLE_USB.wait()).await {
                Either::First(_) => {}
                Either::Second(enable) => {
                    if enable {
                        warn!("Enable when already enabled!");
                    } else {
                        usb.disable().await;
                        enable_command().await;
                    }
                }
            }

            match select3(usb.wait_resume(), ENABLE_USB.wait(), remote_wakeup.wait()).await {
                Either3::First(_) => (),
                Either3::Second(enable) => {
                    if enable {
                        warn!("Enable when already enabled!");
                    } else {
                        usb.disable().await;
                        enable_command().await;
                    }
                }
                Either3::Third(_) => unwrap!(usb.remote_wakeup().await),
            }
        }
    };

    let mut button = Input::new(p.P0_11.degrade(), Pull::Up);

    let (reader, mut writer) = hid.split();

    // Do stuff with the class!
    let in_fut = async {
        loop {
            button.wait_for_low().await;
            info!("PRESSED");

            if SUSPENDED.load(Ordering::Acquire) {
                info!("Triggering remote wakeup");
                remote_wakeup.signal(());
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
                };
            }

            button.wait_for_high().await;
            info!("RELEASED");
            let report = KeyboardReport {
                keycodes: [0, 0, 0, 0, 0, 0],
                leds: 0,
                modifier: 0,
                reserved: 0,
            };
            match writer.write_serialize(&report).await {
                Ok(()) => {}
                Err(e) => warn!("Failed to send report: {:?}", e),
            };
        }
    };

    let out_fut = async {
        reader.run(false, &request_handler).await;
    };

    let power_irq = interrupt::take!(POWER_CLOCK);
    power_irq.set_handler(on_power_interrupt);
    power_irq.unpend();
    power_irq.enable();

    power
        .intenset
        .write(|w| w.usbdetected().set().usbremoved().set());

    // Run everything concurrently.
    // If we had made everything `'static` above instead, we could do this using separate tasks instead.
    join(usb_fut, join(in_fut, out_fut)).await;
}

async fn enable_command() {
    loop {
        if ENABLE_USB.wait().await {
            break;
        } else {
            warn!("Received disable signal when already disabled!");
        }
    }
}

struct MyRequestHandler {}

impl RequestHandler for MyRequestHandler {
    fn get_report(&self, id: ReportId, _buf: &mut [u8]) -> Option<usize> {
        info!("Get report for {:?}", id);
        None
    }

    fn set_report(&self, id: ReportId, data: &[u8]) -> OutResponse {
        info!("Set report for {:?}: {=[u8]}", id, data);
        OutResponse::Accepted
    }

    fn set_idle(&self, id: Option<ReportId>, dur: Duration) {
        info!("Set idle rate for {:?} to {:?}", id, dur);
    }

    fn get_idle(&self, id: Option<ReportId>) -> Option<Duration> {
        info!("Get idle rate for {:?}", id);
        None
    }
}

struct MyDeviceStateHandler {
    configured: AtomicBool,
}

impl MyDeviceStateHandler {
    fn new() -> Self {
        MyDeviceStateHandler {
            configured: AtomicBool::new(false),
        }
    }
}

impl DeviceStateHandler for MyDeviceStateHandler {
    fn enabled(&self, enabled: bool) {
        self.configured.store(false, Ordering::Relaxed);
        SUSPENDED.store(false, Ordering::Release);
        if enabled {
            info!("Device enabled");
        } else {
            info!("Device disabled");
        }
    }

    fn reset(&self) {
        self.configured.store(false, Ordering::Relaxed);
        info!("Bus reset, the Vbus current limit is 100mA");
    }

    fn addressed(&self, addr: u8) {
        self.configured.store(false, Ordering::Relaxed);
        info!("USB address set to: {}", addr);
    }

    fn configured(&self, configured: bool) {
        self.configured.store(configured, Ordering::Relaxed);
        if configured {
            info!(
                "Device configured, it may now draw up to the configured current limit from Vbus."
            )
        } else {
            info!("Device is no longer configured, the Vbus current limit is 100mA.");
        }
    }

    fn suspended(&self, suspended: bool) {
        if suspended {
            info!("Device suspended, the Vbus current limit is 500µA (or 2.5mA for high-power devices with remote wakeup enabled).");
            SUSPENDED.store(true, Ordering::Release);
        } else {
            SUSPENDED.store(false, Ordering::Release);
            if self.configured.load(Ordering::Relaxed) {
                info!(
                    "Device resumed, it may now draw up to the configured current limit from Vbus"
                );
            } else {
                info!("Device resumed, the Vbus current limit is 100mA");
            }
        }
    }
}
