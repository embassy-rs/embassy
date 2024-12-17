#![no_std]
#![no_main]

use core::cell::{Cell, RefCell};

use defmt::{panic, *};
use embassy_executor::Spawner;
use embassy_stm32::time::Hertz;
use embassy_stm32::{bind_interrupts, interrupt, peripherals, timer, usb, Config};
use embassy_sync::blocking_mutex::raw::{CriticalSectionRawMutex, NoopRawMutex};
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::signal::Signal;
use embassy_sync::zerocopy_channel;
use embassy_usb::class::uac1;
use embassy_usb::class::uac1::speaker::{self, Speaker};
use embassy_usb::driver::EndpointError;
use heapless::Vec;
use micromath::F32Ext;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    OTG_FS => usb::InterruptHandler<peripherals::USB_OTG_FS>;
});

static TIMER: Mutex<CriticalSectionRawMutex, RefCell<Option<timer::low_level::Timer<peripherals::TIM2>>>> =
    Mutex::new(RefCell::new(None));

// A counter signal that is written by the feedback timer, once every `FEEDBACK_REFRESH_PERIOD`.
// At that point, a feedback value is sent to the host.
pub static FEEDBACK_SIGNAL: Signal<CriticalSectionRawMutex, u32> = Signal::new();

// Stereo input
pub const INPUT_CHANNEL_COUNT: usize = 2;

// This example uses a fixed sample rate of 48 kHz.
pub const SAMPLE_RATE_HZ: u32 = 48_000;
pub const FEEDBACK_COUNTER_TICK_RATE: u32 = 42_000_000;

// Use 32 bit samples, which allow for a lot of (software) volume adjustment without degradation of quality.
pub const SAMPLE_WIDTH: uac1::SampleWidth = uac1::SampleWidth::Width4Byte;
pub const SAMPLE_WIDTH_BIT: usize = SAMPLE_WIDTH.in_bit();
pub const SAMPLE_SIZE: usize = SAMPLE_WIDTH as usize;
pub const SAMPLE_SIZE_PER_S: usize = (SAMPLE_RATE_HZ as usize) * INPUT_CHANNEL_COUNT * SAMPLE_SIZE;

// Size of audio samples per 1 ms - for the full-speed USB frame period of 1 ms.
pub const USB_FRAME_SIZE: usize = SAMPLE_SIZE_PER_S.div_ceil(1000);

// Select front left and right audio channels.
pub const AUDIO_CHANNELS: [uac1::Channel; INPUT_CHANNEL_COUNT] = [uac1::Channel::LeftFront, uac1::Channel::RightFront];

// Factor of two as a margin for feedback (this is an excessive amount)
pub const USB_MAX_PACKET_SIZE: usize = 2 * USB_FRAME_SIZE;
pub const USB_MAX_SAMPLE_COUNT: usize = USB_MAX_PACKET_SIZE / SAMPLE_SIZE;

// The data type that is exchanged via the zero-copy channel (a sample vector).
pub type SampleBlock = Vec<u32, USB_MAX_SAMPLE_COUNT>;

// Feedback is provided in 10.14 format for full-speed endpoints.
pub const FEEDBACK_REFRESH_PERIOD: uac1::FeedbackRefresh = uac1::FeedbackRefresh::Period8Frames;
const FEEDBACK_SHIFT: usize = 14;

const TICKS_PER_SAMPLE: f32 = (FEEDBACK_COUNTER_TICK_RATE as f32) / (SAMPLE_RATE_HZ as f32);

struct Disconnected {}

impl From<EndpointError> for Disconnected {
    fn from(val: EndpointError) -> Self {
        match val {
            EndpointError::BufferOverflow => panic!("Buffer overflow"),
            EndpointError::Disabled => Disconnected {},
        }
    }
}

/// Sends feedback messages to the host.
async fn feedback_handler<'d, T: usb::Instance + 'd>(
    feedback: &mut speaker::Feedback<'d, usb::Driver<'d, T>>,
    feedback_factor: f32,
) -> Result<(), Disconnected> {
    let mut packet: Vec<u8, 4> = Vec::new();

    // Collects the fractional component of the feedback value that is lost by rounding.
    let mut rest = 0.0_f32;

    loop {
        let counter = FEEDBACK_SIGNAL.wait().await;

        packet.clear();

        let raw_value = counter as f32 * feedback_factor + rest;
        let value = raw_value.round();
        rest = raw_value - value;

        let value = value as u32;
        packet.push(value as u8).unwrap();
        packet.push((value >> 8) as u8).unwrap();
        packet.push((value >> 16) as u8).unwrap();

        feedback.write_packet(&packet).await?;
    }
}

/// Handles streaming of audio data from the host.
async fn stream_handler<'d, T: usb::Instance + 'd>(
    stream: &mut speaker::Stream<'d, usb::Driver<'d, T>>,
    sender: &mut zerocopy_channel::Sender<'static, NoopRawMutex, SampleBlock>,
) -> Result<(), Disconnected> {
    loop {
        let mut usb_data = [0u8; USB_MAX_PACKET_SIZE];
        let data_size = stream.read_packet(&mut usb_data).await?;

        let word_count = data_size / SAMPLE_SIZE;

        if word_count * SAMPLE_SIZE == data_size {
            // Obtain a buffer from the channel
            let samples = sender.send().await;
            samples.clear();

            for w in 0..word_count {
                let byte_offset = w * SAMPLE_SIZE;
                let sample = u32::from_le_bytes(usb_data[byte_offset..byte_offset + SAMPLE_SIZE].try_into().unwrap());

                // Fill the sample buffer with data.
                samples.push(sample).unwrap();
            }

            sender.send_done();
        } else {
            debug!("Invalid USB buffer size of {}, skipped.", data_size);
        }
    }
}

/// Receives audio samples from the USB streaming task and can play them back.
#[embassy_executor::task]
async fn audio_receiver_task(mut usb_audio_receiver: zerocopy_channel::Receiver<'static, NoopRawMutex, SampleBlock>) {
    loop {
        let _samples = usb_audio_receiver.receive().await;
        // Use the samples, for example play back via the SAI peripheral.

        // Notify the channel that the buffer is now ready to be reused
        usb_audio_receiver.receive_done();
    }
}

/// Receives audio samples from the host.
#[embassy_executor::task]
async fn usb_streaming_task(
    mut stream: speaker::Stream<'static, usb::Driver<'static, peripherals::USB_OTG_FS>>,
    mut sender: zerocopy_channel::Sender<'static, NoopRawMutex, SampleBlock>,
) {
    loop {
        stream.wait_connection().await;
        _ = stream_handler(&mut stream, &mut sender).await;
    }
}

/// Sends sample rate feedback to the host.
///
/// The `feedback_factor` scales the feedback timer's counter value so that the result is the number of samples that
/// this device played back or "consumed" during one SOF period (1 ms) - in 10.14 format.
///
/// Ideally, the `feedback_factor` that is calculated below would be an integer for avoiding numerical errors.
/// This is achieved by having `TICKS_PER_SAMPLE` be a power of two. For audio applications at a sample rate of 48 kHz,
/// 24.576 MHz would be one such option.
///
/// A good choice for the STM32F4, which also has to generate a 48 MHz clock from its HSE (e.g. running at 8 MHz)
/// for USB, is to clock the feedback timer from the MCLK output of the SAI peripheral. The SAI peripheral then uses an
/// external clock. In that case, wiring the MCLK output to the timer clock input is required.
///
/// This simple example just uses the internal clocks for supplying the feedback timer,
/// and does not even set up a SAI peripheral.
#[embassy_executor::task]
async fn usb_feedback_task(mut feedback: speaker::Feedback<'static, usb::Driver<'static, peripherals::USB_OTG_FS>>) {
    let feedback_factor =
        ((1 << FEEDBACK_SHIFT) as f32 / TICKS_PER_SAMPLE) / FEEDBACK_REFRESH_PERIOD.frame_count() as f32;

    // Should be 2.3405714285714287...
    info!("Using a feedback factor of {}.", feedback_factor);

    loop {
        feedback.wait_connection().await;
        _ = feedback_handler(&mut feedback, feedback_factor).await;
    }
}

#[embassy_executor::task]
async fn usb_task(mut usb_device: embassy_usb::UsbDevice<'static, usb::Driver<'static, peripherals::USB_OTG_FS>>) {
    usb_device.run().await;
}

/// Checks for changes on the control monitor of the class.
///
/// In this case, monitor changes of volume or mute state.
#[embassy_executor::task]
async fn usb_control_task(control_monitor: speaker::ControlMonitor<'static>) {
    loop {
        control_monitor.changed().await;

        for channel in AUDIO_CHANNELS {
            let volume = control_monitor.volume(channel).unwrap();
            info!("Volume changed to {} on channel {}.", volume, channel);
        }
    }
}

/// Feedback value measurement and calculation
///
/// Used for measuring/calculating the number of samples that were received from the host during the
/// `FEEDBACK_REFRESH_PERIOD`.
///
/// Configured in this example with
/// - a refresh period of 8 ms, and
/// - a tick rate of 42 MHz.
///
/// This gives an (ideal) counter value of 336.000 for every update of the `FEEDBACK_SIGNAL`.
#[interrupt]
fn TIM2() {
    static LAST_TICKS: Mutex<CriticalSectionRawMutex, Cell<u32>> = Mutex::new(Cell::new(0));
    static FRAME_COUNT: Mutex<CriticalSectionRawMutex, Cell<usize>> = Mutex::new(Cell::new(0));

    critical_section::with(|cs| {
        // Read timer counter.
        let timer = TIMER.borrow(cs).borrow().as_ref().unwrap().regs_gp32();

        let status = timer.sr().read();

        const CHANNEL_INDEX: usize = 0;
        if status.ccif(CHANNEL_INDEX) {
            let ticks = timer.ccr(CHANNEL_INDEX).read();

            let frame_count = FRAME_COUNT.borrow(cs);
            let last_ticks = LAST_TICKS.borrow(cs);

            frame_count.set(frame_count.get() + 1);
            if frame_count.get() >= FEEDBACK_REFRESH_PERIOD.frame_count() {
                frame_count.set(0);
                FEEDBACK_SIGNAL.signal(ticks.wrapping_sub(last_ticks.get()));
                last_ticks.set(ticks);
            }
        };

        // Clear trigger interrupt flag.
        timer.sr().modify(|r| r.set_tif(false));
    });
}

// If you are trying this and your USB device doesn't connect, the most
// common issues are the RCC config and vbus_detection
//
// See https://embassy.dev/book/#_the_usb_examples_are_not_working_on_my_board_is_there_anything_else_i_need_to_configure
// for more information.
#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Hello World!");

    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hse = Some(Hse {
            freq: Hertz(8_000_000),
            mode: HseMode::Bypass,
        });
        config.rcc.pll_src = PllSource::HSE;
        config.rcc.pll = Some(Pll {
            prediv: PllPreDiv::DIV4,
            mul: PllMul::MUL168,
            divp: Some(PllPDiv::DIV2), // ((8 MHz / 4) * 168) / 2 = 168 Mhz.
            divq: Some(PllQDiv::DIV7), // ((8 MHz / 4) * 168) / 7 = 48 Mhz.
            divr: None,
        });
        config.rcc.ahb_pre = AHBPrescaler::DIV1;
        config.rcc.apb1_pre = APBPrescaler::DIV4;
        config.rcc.apb2_pre = APBPrescaler::DIV2;
        config.rcc.sys = Sysclk::PLL1_P;
        config.rcc.mux.clk48sel = mux::Clk48sel::PLL1_Q;
    }
    let p = embassy_stm32::init(config);

    // Configure all required buffers in a static way.
    debug!("USB packet size is {} byte", USB_MAX_PACKET_SIZE);
    static CONFIG_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
    let config_descriptor = CONFIG_DESCRIPTOR.init([0; 256]);

    static BOS_DESCRIPTOR: StaticCell<[u8; 32]> = StaticCell::new();
    let bos_descriptor = BOS_DESCRIPTOR.init([0; 32]);

    const CONTROL_BUF_SIZE: usize = 64;
    static CONTROL_BUF: StaticCell<[u8; CONTROL_BUF_SIZE]> = StaticCell::new();
    let control_buf = CONTROL_BUF.init([0; CONTROL_BUF_SIZE]);

    const FEEDBACK_BUF_SIZE: usize = 4;
    static EP_OUT_BUFFER: StaticCell<[u8; FEEDBACK_BUF_SIZE + CONTROL_BUF_SIZE + USB_MAX_PACKET_SIZE]> =
        StaticCell::new();
    let ep_out_buffer = EP_OUT_BUFFER.init([0u8; FEEDBACK_BUF_SIZE + CONTROL_BUF_SIZE + USB_MAX_PACKET_SIZE]);

    static STATE: StaticCell<speaker::State> = StaticCell::new();
    let state = STATE.init(speaker::State::new());

    // Create the driver, from the HAL.
    let mut usb_config = usb::Config::default();

    // Do not enable vbus_detection. This is a safe default that works in all boards.
    // However, if your USB device is self-powered (can stay powered on if USB is unplugged), you need
    // to enable vbus_detection to comply with the USB spec. If you enable it, the board
    // has to support it or USB won't work at all. See docs on `vbus_detection` for details.
    usb_config.vbus_detection = false;

    let usb_driver = usb::Driver::new_fs(p.USB_OTG_FS, Irqs, p.PA12, p.PA11, ep_out_buffer, usb_config);

    // Basic USB device configuration
    let mut config = embassy_usb::Config::new(0xc0de, 0xcafe);
    config.manufacturer = Some("Embassy");
    config.product = Some("USB-audio-speaker example");
    config.serial_number = Some("12345678");

    // Required for windows compatibility.
    // https://developer.nordicsemi.com/nRF_Connect_SDK/doc/1.9.1/kconfig/CONFIG_CDC_ACM_IAD.html#help
    config.device_class = 0xEF;
    config.device_sub_class = 0x02;
    config.device_protocol = 0x01;
    config.composite_with_iads = true;

    let mut builder = embassy_usb::Builder::new(
        usb_driver,
        config,
        config_descriptor,
        bos_descriptor,
        &mut [], // no msos descriptors
        control_buf,
    );

    // Create the UAC1 Speaker class components
    let (stream, feedback, control_monitor) = Speaker::new(
        &mut builder,
        state,
        USB_MAX_PACKET_SIZE as u16,
        uac1::SampleWidth::Width4Byte,
        &[SAMPLE_RATE_HZ],
        &AUDIO_CHANNELS,
        FEEDBACK_REFRESH_PERIOD,
    );

    // Create the USB device
    let usb_device = builder.build();

    // Establish a zero-copy channel for transferring received audio samples between tasks
    static SAMPLE_BLOCKS: StaticCell<[SampleBlock; 2]> = StaticCell::new();
    let sample_blocks = SAMPLE_BLOCKS.init([Vec::new(), Vec::new()]);

    static CHANNEL: StaticCell<zerocopy_channel::Channel<'_, NoopRawMutex, SampleBlock>> = StaticCell::new();
    let channel = CHANNEL.init(zerocopy_channel::Channel::new(sample_blocks));
    let (sender, receiver) = channel.split();

    // Run a timer for counting between SOF interrupts.
    let mut tim2 = timer::low_level::Timer::new(p.TIM2);
    tim2.set_tick_freq(Hertz(FEEDBACK_COUNTER_TICK_RATE));
    tim2.set_trigger_source(timer::low_level::TriggerSource::ITR1); // The USB SOF signal.

    const TIMER_CHANNEL: timer::Channel = timer::Channel::Ch1;
    tim2.set_input_ti_selection(TIMER_CHANNEL, timer::low_level::InputTISelection::TRC);
    tim2.set_input_capture_prescaler(TIMER_CHANNEL, 0);
    tim2.set_input_capture_filter(TIMER_CHANNEL, timer::low_level::FilterValue::FCK_INT_N2);

    // Reset all interrupt flags.
    tim2.regs_gp32().sr().write(|r| r.0 = 0);

    // Enable routing of SOF to the timer.
    tim2.regs_gp32().or().write(|r| *r = 0b10 << 10);

    tim2.enable_channel(TIMER_CHANNEL, true);
    tim2.enable_input_interrupt(TIMER_CHANNEL, true);

    tim2.start();

    TIMER.lock(|p| p.borrow_mut().replace(tim2));

    // Unmask the TIM2 interrupt.
    unsafe {
        cortex_m::peripheral::NVIC::unmask(interrupt::TIM2);
    }

    // Launch USB audio tasks.
    unwrap!(spawner.spawn(usb_control_task(control_monitor)));
    unwrap!(spawner.spawn(usb_streaming_task(stream, sender)));
    unwrap!(spawner.spawn(usb_feedback_task(feedback)));
    unwrap!(spawner.spawn(usb_task(usb_device)));
    unwrap!(spawner.spawn(audio_receiver_task(receiver)));
}
