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

use defmt::{info, panic};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_nxp::usb::{Driver, InterruptHandler, Memory};
use embassy_nxp::{bind_interrupts, peripherals};
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use embassy_usb::driver::EndpointError;
use panic_probe as _;

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
    // One packet per transfer: unlike the high-speed controller, the
    // full-speed block does not packetize a command entry, so an endpoint is
    // armed one max-packet-size packet at a time and `write_packet` rejects
    // anything larger.
    const CHUNK: usize = 64;

    // Recognizable pattern so the host can spot-check payload integrity:
    // a 0..512 ramp repeating per 512-byte packet, identical to the HS bench.
    // The ramp period spans 8 full-speed packets, so index by stream offset.
    let mut tx = [0u8; 512];
    for (i, b) in tx.iter_mut().enumerate() {
        *b = (i % 512) as u8;
    }
    let mut buf = [0u8; CHUNK];

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
                let mut off = 0usize;
                while remaining > 0 {
                    let chunk = remaining.min(CHUNK);
                    // Walk the 512-byte ramp so the host sees one continuous
                    // stream rather than the first packet repeated.
                    class.write_packet(&tx[off..off + chunk]).await?;
                    off = (off + chunk) % tx.len();
                    remaining -= chunk;
                }
                // A stream ending exactly on a max-packet-size boundary needs
                // a zero-length packet to tell the host the transfer is over;
                // without it the host waits for more and the last packet is
                // never delivered.
                if total % CHUNK == 0 {
                    class.write_packet(&[]).await?;
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
