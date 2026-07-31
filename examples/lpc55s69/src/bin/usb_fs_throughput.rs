//! USB0 full-speed bulk throughput benchmark for the LPC55S6x.
//!
//! Speaks the same wire protocol as `usb_hs_throughput`, so one host script
//! drives both: the host sends a 5-byte header on the CDC tty, a command byte
//! followed by a little-endian u32 byte count.
//!
//! - `b'I'` + N: the device streams N bytes to the host.
//! - `b'O'` + N: the host sends N bytes, then the device replies with the
//!   received count as a little-endian u32.
//!
//! Drive it with `examples/lpc55s69/scripts/usb_throughput.py`. Expect roughly
//! 1.0-1.2 MB/s, against a full-speed bulk ceiling of 1.216 MB/s.
//!
//! Uses the default HAL config: the full-speed controller brings up its own
//! 48 MHz clock. Cable on connector **P10**.

#![no_std]
#![no_main]

#[path = "../throughput.rs"]
mod throughput;

use defmt::{info, panic};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_nxp::usb::{Driver, InterruptHandler, Memory};
use embassy_nxp::{bind_interrupts, peripherals};
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use embassy_usb::driver::EndpointError;
use panic_probe as _;

use throughput::{Event, Parser};

bind_interrupts!(struct Irqs {
    USB0 => InterruptHandler<peripherals::USB0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nxp::init(embassy_nxp::config::Config::default());

    info!("Initialization complete");

    let driver = Driver::<peripherals::USB0>::new(p.USB0, Irqs, p.PIO0_22, Memory::usb1_sram());

    let mut config = embassy_usb::Config::new(0xc0de, 0xcafe);
    config.manufacturer = Some("Embassy");
    config.product = Some("USB-FS throughput test");
    config.serial_number = Some("12345678");
    config.max_power = 100;
    config.max_packet_size_0 = 64;

    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    let mut control_buf = [0; 64];

    let mut state = State::new();

    let mut builder = embassy_usb::Builder::new(
        driver,
        config,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut [], // no msos descriptors
        &mut control_buf,
    );

    // 64-byte bulk max packet size for FS.
    let mut class = CdcAcmClass::new(&mut builder, &mut state, 64);

    let mut usb = builder.build();
    let usb_fut = usb.run();

    let bench_fut = async {
        loop {
            class.wait_connection().await;
            info!("Connected");
            let _ = bench(&mut class).await;
            info!("Disconnected");
        }
    };

    join(usb_fut, bench_fut).await;
}

struct Disconnected {}

impl From<EndpointError> for Disconnected {
    fn from(val: EndpointError) -> Self {
        match val {
            EndpointError::BufferOverflow => panic!("Buffer overflow"),
            EndpointError::Disabled => Disconnected {},
        }
    }
}

async fn bench<'d>(class: &mut CdcAcmClass<'d, Driver<'d, peripherals::USB0>>) -> Result<(), Disconnected> {
    const CHUNK: usize = 64;

    let mut tx = [0u8; 512];
    for (i, byte) in tx.iter_mut().enumerate() {
        *byte = (i % 512) as u8;
    }
    let mut buf = [0u8; CHUNK];
    let mut parser = Parser::new();

    loop {
        let len = class.read_packet(&mut buf).await?;
        let mut offset = 0;

        while offset < len {
            let feed = parser.feed(&buf[offset..len]);
            offset += feed.consumed;

            match feed.event {
                Some(Event::In(total)) => {
                    info!("IN test: {} bytes", total);
                    let mut remaining = total;
                    let mut ramp_offset = 0;
                    while remaining > 0 {
                        let chunk = remaining.min(CHUNK as u32) as usize;
                        class.write_packet(&tx[ramp_offset..ramp_offset + chunk]).await?;
                        ramp_offset = (ramp_offset + chunk) % tx.len();
                        remaining -= chunk as u32;
                    }
                    if total % CHUNK as u32 == 0 {
                        class.write_packet(&[]).await?;
                    }
                }
                Some(Event::OutStarted(total)) => info!("OUT test: {} bytes", total),
                Some(Event::OutComplete(total)) => {
                    class.write_packet(&total.to_le_bytes()).await?;
                }
                Some(Event::Unknown(command)) => info!("unknown command {}", command),
                None => {}
            }
        }
    }
}
