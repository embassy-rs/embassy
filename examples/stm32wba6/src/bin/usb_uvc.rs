//! USB Video Class camera: moving color bars, 160x120 YUY2 at 15 or 30 fps.
//!
//! There's no image sensor on the board, so the frames are generated. On Linux, view them with
//! `ffplay -f v4l2 -framerate 15 -video_size 160x120 -pixel_format yuyv422 /dev/video0`, or any
//! webcam application. The host picks the frame rate; the log shows what it selected.

#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_stm32::usb::Driver;
use embassy_stm32::{Config, bind_interrupts, peripherals, usb};
use embassy_time::{Duration, Ticker};
use embassy_usb::class::uvc::device::{Camera, Config as UvcConfig, Format, Frame, PixelFormat, State};
use embassy_usb::{Builder, UsbDeviceSpeed};
use panic_probe as _;

bind_interrupts!(struct Irqs {
    USB_OTG_HS => usb::InterruptHandler<peripherals::USB_OTG_HS>;
});

const WIDTH: usize = 160;
const HEIGHT: usize = 120;
const BARS: usize = 8;
const BAR_WIDTH: usize = WIDTH / BARS;

/// White, yellow, cyan, green, magenta, red, blue and black, as Y, U, V.
const BAR_YUV: [[u8; 3]; BARS] = [
    [235, 128, 128],
    [210, 16, 146],
    [170, 166, 16],
    [145, 54, 34],
    [106, 202, 222],
    [81, 90, 240],
    [41, 240, 110],
    [16, 128, 128],
];

static INTERVALS: [u32; 2] = [Frame::interval_for_fps(15), Frame::interval_for_fps(30)];
static FRAMES: [Frame; 1] = [Frame::new(WIDTH as u16, HEIGHT as u16, &INTERVALS)];
static FORMATS: [Format; 1] = [Format::new(PixelFormat::Yuy2, &FRAMES)];

/// Fills a YUY2 frame with color bars, shifted by `shift` bars. Every pair of pixels is `Y0 U Y1 V`.
fn draw_bars(frame: &mut [u8], shift: usize) {
    for row in frame.chunks_exact_mut(WIDTH * 2) {
        for (x, pair) in row.chunks_exact_mut(4).enumerate() {
            let [y, u, v] = BAR_YUV[(x * 2 / BAR_WIDTH + shift) % BARS];
            pair.copy_from_slice(&[y, u, y, v]);
        }
    }
}

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
    config.product = Some("UVC camera");
    config.serial_number = Some("12345678");
    config.max_speed = UsbDeviceSpeed::High;

    let mut config_descriptor = [0; 512];
    let mut bos_descriptor = [0; 256];
    let mut control_buf = [0; 64];
    let mut state = State::new();

    let mut builder = Builder::new(
        driver,
        config,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut [], // no msos descriptors
        &mut control_buf,
    );

    let mut camera = Camera::new(&mut builder, &mut state, UvcConfig::new(&FORMATS, UsbDeviceSpeed::High));

    let mut usb = builder.build();

    let camera_fut = async {
        let mut frame = [0u8; WIDTH * HEIGHT * 2];
        loop {
            let settings = camera.wait_streaming().await;
            info!(
                "streaming {}x{} at {} fps",
                settings.frame.width,
                settings.frame.height,
                10_000_000 / settings.frame_interval
            );
            let mut ticker = Ticker::every(Duration::from_micros(settings.frame_interval as u64 / 10));
            let mut shift = 0;
            loop {
                draw_bars(&mut frame, shift);
                shift += 1;
                if camera.write_frame(&frame).await.is_err() {
                    break;
                }
                ticker.next().await;
            }
            info!("stopped");
        }
    };

    join(usb.run(), camera_fut).await;
}
