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

use defmt::{info, panic};
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_nxp::config::MainClock;
use embassy_nxp::usb::{Driver, InterruptHandler};
use embassy_nxp::{bind_interrupts, peripherals};
use embassy_usb::UsbDeviceSpeed;
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use embassy_usb::driver::EndpointError;
use {defmt_rtt as _, panic_probe as _};

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

    let driver = Driver::new(p.USBHSD, Irqs);

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
    // Chunk sizes match the driver's multi-packet bulk slots: IN slots are
    // 3584 B (7 packets per command entry), OUT slots 4096 B (8 packets).
    // Both are multiples of 512 so wire packets stay max-packet-sized and
    // the host-visible 512-byte pattern ramp stays aligned.
    const IN_CHUNK: usize = 3584;
    const OUT_CHUNK: usize = 4096;

    // Recognizable pattern so the host can spot-check payload integrity:
    // a 0..512 ramp repeating per 512-byte packet.
    let mut tx = [0u8; IN_CHUNK];
    for (i, b) in tx.iter_mut().enumerate() {
        *b = (i % 512) as u8;
    }
    let mut buf = [0u8; OUT_CHUNK];

    loop {
        let n = class.read_packet(&mut buf).await?;
        if n < 5 {
            continue;
        }
        let cmd = buf[0];
        let total = u32::from_le_bytes([buf[1], buf[2], buf[3], buf[4]]) as usize;

        match cmd {
            b'I' => {
                info!("IN test: {} bytes", total);
                let mut remaining = total;
                while remaining > 0 {
                    let chunk = remaining.min(IN_CHUNK);
                    class.write_packet(&tx[..chunk]).await?;
                    remaining -= chunk;
                }
            }
            b'O' => {
                info!("OUT test: {} bytes", total);
                // Payload may share the header packet.
                let mut received = n - 5;
                while received < total {
                    received += class.read_packet(&mut buf).await?;
                }
                class.write_packet(&(received as u32).to_le_bytes()).await?;
            }
            _ => info!("unknown command {}", cmd),
        }
    }
}
