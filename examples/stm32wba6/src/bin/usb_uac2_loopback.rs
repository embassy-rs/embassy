//! USB Audio Class 2.0 I/O box: what the host plays on line out comes back on line in.
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

use defmt::{panic, *};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::join::join5;
use embassy_stm32::usb::Driver;
use embassy_stm32::{Config, bind_interrupts, peripherals, usb};
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_usb::Builder;
use embassy_usb::class::uac::device::SampleWidth;
use embassy_usb::class::uac::device::uac2::io_box::{self, Channels, FeatureUnitControls, IoBox, State};
use embassy_usb::driver::EndpointError;
use heapless::Deque;
use panic_probe as _;

bind_interrupts!(struct Irqs {
    USB_OTG_HS => usb::InterruptHandler<peripherals::USB_OTG_HS>;
});

const RATES: [u32; 2] = [44_100, 48_000];
const WIDTH: SampleWidth = SampleWidth::Width3Byte;
const CHANNELS: u8 = 2;
const FRAME: usize = WIDTH as usize * CHANNELS as usize;
const MAX_PACKET: usize = io_box::max_packet_size(48_000, WIDTH, CHANNELS as usize);
/// Feature Unit entries: the master and both channels.
const FU: usize = 3;

/// Line out's samples on their way back: up to 4 ms.
type Fifo = Mutex<NoopRawMutex, RefCell<Deque<u8, { 4 * 49 * FRAME }>>>;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Hello World!");

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

    let mut ep_out_buffer = [0u8; 512];
    let mut usb_config = embassy_stm32::usb::Config::default();
    // See usb_hs_serial.rs about VBUS detection.
    usb_config.vbus_detection = false;
    let driver = Driver::new_hs(p.USB_OTG_HS, p.PD6, p.PD7, Irqs, &mut ep_out_buffer, usb_config);

    let mut config = embassy_usb::Config::new(0xc0de, 0xcafe);
    config.manufacturer = Some("Embassy");
    config.product = Some("UAC2 loopback");
    config.serial_number = Some("12345678");

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
            // frames per microframe: rate / 8000, the remainder carried
            let mut acc = 0u32;
            loop {
                let rate = control_monitor.sample_rate_hz();
                acc += rate;
                let frames = (acc / 8000) as usize;
                acc %= 8000;
                let len = frames * FRAME;
                fifo.lock(|f| {
                    let mut f = f.borrow_mut();
                    for b in &mut packet[..len] {
                        // nothing played (yet): silence
                        *b = f.pop_front().unwrap_or(0);
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
            // samples per microframe, 16.16
            loop {
                let rate = control_monitor.sample_rate_hz();
                let value = ((rate as u64) << 16) / 8000;
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
