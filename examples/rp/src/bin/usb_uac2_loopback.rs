//! USB Audio Class 2.0 I/O box at full speed: what the host plays on line out comes back on
//! line in.
//!
//! Stereo, 24-bit, at 44.1 or 48 kHz, with mute and volume on line out (logged, not applied).
//! There's no audio clock on the board, so the device runs at USB's clock: the feedback
//! endpoint reports the nominal rate.
//!
//! On Linux, find the card's number with `aplay -l` (here 2), then play a file and record it back:
//!
//! ```text
//! arecord -D hw:2,0 -f S24_3LE -r 48000 -c 2 back.wav &
//! aplay -D hw:2,0 -f S24_3LE -r 48000 -c 2 played.wav
//! ```

#![no_std]
#![no_main]

use core::cell::RefCell;

use defmt::{info, panic};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::join::join5;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::USB;
use embassy_rp::uid::uid_hex;
use embassy_rp::usb::{Driver, InterruptHandler};
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_usb::class::uac::device::SampleWidth;
use embassy_usb::class::uac::device::uac2::io_box::{self, Channels, FeatureUnitControls, IoBox, State};
use embassy_usb::driver::EndpointError;
use embassy_usb::{Builder, Config, UsbDeviceSpeed};
use heapless::Deque;
use panic_probe as _;

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});

const RATES: [u32; 2] = [44_100, 48_000];
const WIDTH: SampleWidth = SampleWidth::Width3Byte;
const CHANNELS: u8 = 2;
const FRAME: usize = WIDTH as usize * CHANNELS as usize;
const SPEED: UsbDeviceSpeed = UsbDeviceSpeed::Full;
const PACKET_RATE: u32 = io_box::packet_rate_hz(SPEED);
const MAX_PACKET: usize = io_box::max_packet_size(SPEED, 48_000, WIDTH, CHANNELS as usize);
/// Feature Unit entries: the master and both channels.
const FU: usize = 3;

/// Line out's samples on their way back: up to 4 ms.
type Fifo = Mutex<NoopRawMutex, RefCell<Deque<u8, { 4 * 49 * FRAME }>>>;

#[embassy_executor::main(executor = "embassy_rp::executor::Executor", entry = "cortex_m_rt::entry")]
async fn main(_spawner: Spawner) {
    info!("Hello world!");

    let p = embassy_rp::init(Default::default());

    let driver = Driver::new(p.USB, Irqs);

    let mut config = Config::new(0xc0de, 0xcafe);
    config.manufacturer = Some("Embassy");
    config.product = Some("UAC2 loopback");
    config.serial_number = Some(uid_hex());
    config.max_power = 100;
    config.max_packet_size_0 = 64;
    config.max_speed = SPEED;

    let mut config_descriptor = [0; 512];
    let mut bos_descriptor = [0; 256];
    // The clock source's sample rate RANGE reply: 2 + 12 bytes per rate.
    let mut control_buf = [0; 64];
    let mut state = State::<FU>::new();

    let mut builder = Builder::new(
        driver,
        config,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut [], // no msos descriptors
        &mut control_buf,
    );

    let mut uac = io_box::Config::<FU>::new(
        &RATES,
        WIDTH,
        Channels::Numbered(CHANNELS),
        Channels::Numbered(CHANNELS),
    );
    let both = FeatureUnitControls::MUTE | FeatureUnitControls::VOLUME;
    uac.line_out_feature_unit = [both, both, both].into_iter().collect();
    let IoBox {
        mut line_out,
        mut line_in,
        mut feedback,
        control_monitor,
        ..
    } = IoBox::new(&mut builder, &mut state, uac);

    let mut usb = builder.build();
    let fifo: Fifo = Mutex::new(RefCell::new(Deque::new()));

    let line_out_fut = async {
        let mut packet = [0u8; MAX_PACKET];
        loop {
            line_out.wait_connection().await;
            info!("line out: streaming");
            loop {
                let n = match line_out.read_packet(&mut packet).await {
                    Ok(n) => n,
                    Err(EndpointError::Disabled) => break,
                    Err(EndpointError::BufferOverflow) => panic!("line out packet too long"),
                };
                fifo.lock(|f| {
                    let mut f = f.borrow_mut();
                    for &b in &packet[..n] {
                        // full: the oldest goes
                        if f.is_full() {
                            f.pop_front();
                        }
                        let _ = f.push_back(b);
                    }
                });
            }
            info!("line out: stopped");
        }
    };

    let line_in_fut = async {
        let mut packet = [0u8; MAX_PACKET];
        loop {
            line_in.wait_connection().await;
            info!("line in: streaming");
            // audio frames per USB frame: rate / 1000, the remainder carried
            let mut acc = 0u32;
            // silence until 2 packets are buffered, so that packet size jitter doesn't run the FIFO dry
            let mut primed = false;
            loop {
                let rate = control_monitor.sample_rate_hz();
                acc += rate;
                let frames = (acc / PACKET_RATE) as usize;
                acc %= PACKET_RATE;
                let len = frames * FRAME;
                fifo.lock(|f| {
                    let mut f = f.borrow_mut();
                    primed = primed && f.len() >= len || f.len() >= 2 * MAX_PACKET;
                    for b in &mut packet[..len] {
                        *b = if primed { f.pop_front().unwrap_or(0) } else { 0 };
                    }
                });
                if line_in.write_packet(&packet[..len]).await.is_err() {
                    break;
                }
            }
            info!("line in: stopped");
        }
    };

    let feedback_fut = async {
        loop {
            feedback.wait_connection().await;
            // samples per frame, 16.16 (sent as 10.14)
            loop {
                let rate = control_monitor.sample_rate_hz();
                let value = ((rate as u64) << 16) / PACKET_RATE as u64;
                if feedback.write_packet(value as u32).await.is_err() {
                    break;
                }
            }
        }
    };

    let control_fut = async {
        loop {
            control_monitor.changed().await;
            info!(
                "{} Hz, line out volume: left {}, right {}",
                control_monitor.sample_rate_hz(),
                control_monitor.line_out_volume(0),
                control_monitor.line_out_volume(1)
            );
        }
    };

    join5(usb.run(), line_out_fut, line_in_fut, feedback_fut, control_fut).await;
}
