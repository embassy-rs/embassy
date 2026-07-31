//! USB1 high-speed CDC-ACM throughput test firmware for the LPC55S6x.
//!
//! Command-driven benchmark peer for a host-side throughput script.
//! The host sends a 5-byte header: a command byte followed by a u32 LE
//! byte count.
//!
//! - `b'I'`: device sources (writes) exactly `count` bytes to the host.
//! - `b'O'`: device sinks (reads) exactly `count` bytes, then replies
//!   with the received count as a u32 LE ack.
//!
//! Any payload bytes that arrive in the same packet as an `b'O'` header
//! count toward the sink total. No per-packet logging on the hot path.

#![no_std]
#![no_main]

#[path = "../throughput.rs"]
mod throughput;

use defmt::{info, panic};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_nxp::config::MainClock;
use embassy_nxp::usb::{Driver, InterruptHandler, Memory};
use embassy_nxp::{bind_interrupts, peripherals};
use embassy_usb::UsbDeviceSpeed;
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use embassy_usb::driver::EndpointError;
use panic_probe as _;

use throughput::{Event, Parser};

bind_interrupts!(struct Irqs {
    USB1 => InterruptHandler<peripherals::USBHSD>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = embassy_nxp::config::Config::default();
    // USB-HS requires a system clock of at least 96 MHz.
    config.main_clock = MainClock::FroHf96;
    let p = embassy_nxp::init(config);

    info!("Initialization complete");

    let driver = Driver::<peripherals::USBHSD>::new(p.USBHSD, Irqs, Memory::usb1_sram());

    let mut config = embassy_usb::Config::new(0xc0de, 0xcafe);
    config.manufacturer = Some("Embassy");
    config.product = Some("USB-HS throughput test");
    config.serial_number = Some("12345678");
    config.max_power = 100;
    config.max_packet_size_0 = 64;
    config.max_speed = UsbDeviceSpeed::High;

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

    // 512-byte bulk max packet size for HS.
    let mut class = CdcAcmClass::new(&mut builder, &mut state, 512);

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

async fn bench<'d>(class: &mut CdcAcmClass<'d, Driver<'d, peripherals::USBHSD>>) -> Result<(), Disconnected> {
    const IN_CHUNK: usize = 3584;
    const OUT_CHUNK: usize = 512;

    let mut tx = [0u8; IN_CHUNK];
    for (i, byte) in tx.iter_mut().enumerate() {
        *byte = (i % 512) as u8;
    }
    let mut buf = [0u8; OUT_CHUNK];
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
                    while remaining > 0 {
                        let chunk = remaining.min(IN_CHUNK as u32) as usize;
                        class.write_packet(&tx[..chunk]).await?;
                        remaining -= chunk as u32;
                    }
                    if total % OUT_CHUNK as u32 == 0 {
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
