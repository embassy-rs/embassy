#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::time::Hertz;
use embassy_stm32::usb::Driver;
use embassy_stm32::{Config, bind_interrupts, peripherals};
use embassy_usb::class::uac1::source::{AudioSource, AudioSourceControlHandler, AudioSourceEpIn};
use embassy_usb::class::uac1::terminal_type::TerminalType;
use embassy_usb::class::uac1::{self};
use embassy_usb::{Builder, UsbVersion};
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    USB_LP_CAN1_RX0 => embassy_stm32::usb::InterruptHandler<peripherals::USB>;
});

// Use 32 bit samples, which allow for a lot of (software) volume adjustment without degradation of quality.
pub const SAMPLE_WIDTH: uac1::SampleWidth = uac1::SampleWidth::Width2Byte;

// Feedback is provided in 10.14 format for full-speed endpoints.
pub const FEEDBACK_REFRESH_PERIOD: u8 = 8; // 8 frames = 8ms

/// Device supported sample rates
static SUPPORTED_SAMPLE_RATES: [u32; 1] = [16_000];

// Sine wave: 1000 Hz, 1 ms, 16-bit, 2 channels
// Sample rate: 16000 Hz
// Total bytes: 64
static SINE_16000HZ_1MS_16BIT_2CH: [u8; 64] = [
    0xFB, 0x30, 0xFB, 0x30, 0x82, 0x5A, 0x82, 0x5A, 0x41, 0x76, 0x41, 0x76, 0xFF, 0x7F, 0xFF, 0x7F, 0x41, 0x76, 0x41,
    0x76, 0x82, 0x5A, 0x82, 0x5A, 0xFB, 0x30, 0xFB, 0x30, 0x00, 0x00, 0x00, 0x00, 0x05, 0xCF, 0x05, 0xCF, 0x7E, 0xA5,
    0x7E, 0xA5, 0xBF, 0x89, 0xBF, 0x89, 0x01, 0x80, 0x01, 0x80, 0xBF, 0x89, 0xBF, 0x89, 0x7E, 0xA5, 0x7E, 0xA5, 0x05,
    0xCF, 0x05, 0xCF, 0x00, 0x00, 0x00, 0x00,
];

#[embassy_executor::task()]
async fn usb_bus(
    mut usb_device: embassy_usb::UsbDevice<'static, embassy_stm32::usb::Driver<'static, peripherals::USB>>,
) {
    usb_device.run().await;
}

#[embassy_executor::task]
async fn audio_feedback(mut ep_in: AudioSourceEpIn<'static, embassy_stm32::usb::Driver<'static, peripherals::USB>>) {
    let feedback_buf = [0x00, 0x00, 0x04]; //sample rate in 3 bytes for 10.14 format

    loop {
        // Wait for the endpoint to be enabled (blocks if disabled)
        ep_in.wait_enabled().await;

        // Now the endpoint is enabled, try to write
        match ep_in.write(&feedback_buf).await {
            Ok(_) => {
                // Wait exactly 8 ms as required by bRefresh
                embassy_time::Timer::after_millis(8).await;
            }
            Err(e) => {
                error!("audio_feedback: write error {:?}", e);
                // Short delay before retrying (in case of transient error)
                embassy_time::Timer::after_micros(100).await;
            }
        }
    }
}

#[embassy_executor::task]
async fn sine_wave_gen(
    mut audio_ep_in: AudioSourceEpIn<'static, embassy_stm32::usb::Driver<'static, peripherals::USB>>,
) {
    loop {
        audio_ep_in.wait_enabled().await;
        let _ = audio_ep_in.write(&SINE_16000HZ_1MS_16BIT_2CH).await;
    }
}

#[embassy_executor::task]
async fn status_led(mut led: Output<'static>) {
    loop {
        led.set_high();
        embassy_time::Timer::after_millis(500).await;
        led.set_low();
        embassy_time::Timer::after_millis(500).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    debug!("Deep in main()");
    //RCC
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hse = Some(Hse {
            freq: Hertz(8_000_000),
            mode: HseMode::Oscillator,
        });
        config.rcc.pll = Some(Pll {
            src: PllSource::HSE,
            prediv: PllPreDiv::Div1,
            mul: PllMul::Mul9,
        });
        config.rcc.sys = Sysclk::Pll1P;
        config.rcc.ahb_pre = AHBPrescaler::Div1;
        config.rcc.apb1_pre = APBPrescaler::Div2;
        config.rcc.apb2_pre = APBPrescaler::Div1;
    }

    //Init peripherials
    let p = embassy_stm32::init(config);
    debug!("Init was done");

    //Inint status LED
    let led = Output::new(p.PC13, Level::High, Speed::Low);

    // USB driver
    debug!("{}", "Create USB driver");
    let driver = Driver::new(p.USB, Irqs, p.PA12, p.PA11);

    // USB config for composite audio device
    let mut usb_cfg = embassy_usb::Config::new(0xc0de, 0xcafe);
    usb_cfg.manufacturer = Some("Embassy");
    usb_cfg.product = Some("USB Audio Source");

    // Standard USB 2.0 full-speed settings
    usb_cfg.max_packet_size_0 = 64;
    usb_cfg.supports_remote_wakeup = false;
    usb_cfg.bcd_usb = UsbVersion::Two;

    // USB buffers
    debug!("{}", "Allocate USB buffers");
    static CFG_DESCR_BUF_LEN: usize = 512;
    static BOS_DESCR_BUF_LEN: usize = 64;
    static CFG_DESCRIPTOR: StaticCell<[u8; CFG_DESCR_BUF_LEN]> = StaticCell::new();
    static BOS_DESCRIPTOR: StaticCell<[u8; BOS_DESCR_BUF_LEN]> = StaticCell::new();
    static CTRL_BUF: StaticCell<[u8; 64]> = StaticCell::new();
    let cfg_descr = CFG_DESCRIPTOR.init([0; CFG_DESCR_BUF_LEN]);
    let bos_descr = BOS_DESCRIPTOR.init([0; BOS_DESCR_BUF_LEN]);
    let ctrl_buf = CTRL_BUF.init([0; 64]);

    debug!("Create USB builder");
    let mut builder = Builder::new(driver, usb_cfg, cfg_descr, bos_descr, &mut [], ctrl_buf);

    debug!("Create audio stream, feedback endpoints and handler");
    let (audio_ep_in, feedback_ep_in, handler) = AudioSource::new(
        &mut builder,
        &SUPPORTED_SAMPLE_RATES,
        SAMPLE_WIDTH,
        FEEDBACK_REFRESH_PERIOD,
        Some(TerminalType::MiniDisk),
    );

    static AUDIO_CONTROL_HANDLER: StaticCell<AudioSourceControlHandler> = StaticCell::new();
    let audio_control_handler = AUDIO_CONTROL_HANDLER.init(handler);

    builder.handler(audio_control_handler);

    debug!("Create UsbDevice instance in the builder!");
    let usb = builder.build();

    debug!("Run \"usb_task\"!");
    spawner.spawn(unwrap!(usb_bus(usb)));

    debug!("Run \"status_led_task\"!");
    spawner.spawn(status_led(led).unwrap());

    debug!("Run \"sine_wave_gen\" task!");
    spawner.spawn(sine_wave_gen(audio_ep_in).unwrap());

    debug!("Run \"feedback_task\" task!");
    spawner.spawn(audio_feedback(feedback_ep_in).unwrap());

    debug!("All tasks were started!");
}
