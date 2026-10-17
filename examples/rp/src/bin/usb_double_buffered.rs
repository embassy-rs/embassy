//! Raw USB bulk echo using double-buffered bulk endpoints.
//!
//! This is intended for a Raspberry Pi Pico or Pico 2. Connect the device,
//! claim interface 0, and use bulk OUT/IN endpoint 1. Both directions use two
//! hardware buffers: USB can receive while software reads the previous packet,
//! and software can prepare the next IN packet before the host reads it.

#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::USB;
use embassy_rp::usb::{Driver, InterruptHandler};
use embassy_usb::driver::{Endpoint, EndpointIn, EndpointOut};
use embassy_usb::{Builder, Config};
use panic_probe as _;

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});

#[embassy_executor::main(executor = "embassy_rp::executor::Executor", entry = "cortex_m_rt::entry")]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let driver = Driver::new(p.USB, Irqs);

    let mut config = Config::new(0xc0de, 0xcafe);
    config.manufacturer = Some("Embassy");
    config.product = Some("RP double-buffered bulk echo");
    config.serial_number = Some("12345678");
    config.max_power = 100;
    config.max_packet_size_0 = 64;

    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    let mut msos_descriptor = [0; 256];
    let mut control_buf = [0; 64];
    let mut builder = Builder::new(
        driver,
        config,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut msos_descriptor,
        &mut control_buf,
    );

    let mut function = builder.function(0xff, 0, 0);
    let mut interface = function.interface();
    let mut alt = interface.alt_setting(0xff, 0, 0, None);
    let mut out = alt.endpoint_bulk_out_double_buffered(None, 64);
    let mut inp = alt.endpoint_bulk_in_double_buffered(None, 64);
    drop(function);

    let mut usb = builder.build();
    let usb_fut = usb.run();

    let echo_fut = async {
        loop {
            out.wait_enabled().await;
            info!("connected");
            loop {
                let mut packet = [0; 64];
                match out.read(&mut packet).await {
                    Ok(len) => {
                        info!("received {} bytes", len);
                        // The driver alternates the two hardware buffers here.
                        let _ = inp.write(&packet[..len]).await;
                    }
                    Err(_) => break,
                }
            }
            info!("disconnected");
        }
    };

    join(usb_fut, echo_fut).await;
}
