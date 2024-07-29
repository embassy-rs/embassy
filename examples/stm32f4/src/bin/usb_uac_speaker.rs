#![no_std]
#![no_main]

use defmt::{panic, *};
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_stm32::time::Hertz;
use embassy_stm32::usb::{Driver, Instance};
use embassy_stm32::{bind_interrupts, peripherals, usb, Config};
use embassy_usb::class::uac1;
use embassy_usb::class::uac1::speaker::{self, Speaker};
use embassy_usb::driver::EndpointError;
use embassy_usb::Builder;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    OTG_FS => usb::InterruptHandler<peripherals::USB_OTG_FS>;
});

const USB_PACKET_SIZE: usize = 384 * 2; // Double the size of a regular packet
const SAMPLE_RATE_HZ: u32 = 48000;
const SAMPLE_SIZE: usize = uac1::SampleWidth::Width4Byte as usize;
const SAMPLE_COUNT: usize = USB_PACKET_SIZE / SAMPLE_SIZE;
const SAMPLE_BLOCK_COUNT: usize = 2;

type SampleBlock = ([f32; SAMPLE_COUNT], usize);

// If you are trying this and your USB device doesn't connect, the most
// common issues are the RCC config and vbus_detection
//
// See https://embassy.dev/book/#_the_usb_examples_are_not_working_on_my_board_is_there_anything_else_i_need_to_configure
// for more information.
#[embassy_executor::main]
async fn main(_spawner: Spawner) {
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
            divp: Some(PllPDiv::DIV2), // 8mhz / 4 * 168 / 2 = 168Mhz.
            divq: Some(PllQDiv::DIV7), // 8mhz / 4 * 168 / 7 = 48Mhz.
            divr: None,
        });
        config.rcc.ahb_pre = AHBPrescaler::DIV1;
        config.rcc.apb1_pre = APBPrescaler::DIV4;
        config.rcc.apb2_pre = APBPrescaler::DIV2;
        config.rcc.sys = Sysclk::PLL1_P;
        config.rcc.mux.clk48sel = mux::Clk48sel::PLL1_Q;
    }
    let p = embassy_stm32::init(config);

    static EP_OUT_BUFFER: StaticCell<[u8; 2 * USB_PACKET_SIZE]> = StaticCell::new();
    let ep_out_buffer = EP_OUT_BUFFER.init([0u8; 2 * USB_PACKET_SIZE]);

    static CONFIG_DESCRIPTOR: StaticCell<[u8; 128]> = StaticCell::new();
    let config_descriptor = CONFIG_DESCRIPTOR.init([0; 128]);

    static BOS_DESCRIPTOR: StaticCell<[u8; 16]> = StaticCell::new();
    let bos_descriptor = BOS_DESCRIPTOR.init([0; 16]);

    static CONTROL_BUF: StaticCell<[u8; 64]> = StaticCell::new();
    let control_buf = CONTROL_BUF.init([0; 64]);

    static STATE: StaticCell<speaker::State> = StaticCell::new();
    let state = STATE.init(speaker::State::new());

    // Create the driver, from the HAL.
    let mut usb_config = usb::Config::default();

    // Do not enable vbus_detection with an external HS PHY.
    usb_config.vbus_detection = false;

    // Using a Microchip PHY requires a delay during setup.
    usb_config.xcvrdly = true;

    // Initialize driver for high-speed external PHY.
    let usb_driver = usb::Driver::new_fw(
        p.USB_OTG_FS,
        Irqs,
        p.PA5,
        p.PC2,
        p.PC3,
        p.PC0,
        p.PA3,
        p.PB0,
        p.PB1,
        p.PB10,
        p.PB11,
        p.PB12,
        p.PB13,
        p.PB5,
        ep_out_buffer,
        usb_config,
    );

    // Basic USB device configuration
    let mut config = embassy_usb::Config::new(0x1209, 0xaf02);
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
    let (mut stream, control_changed) = Speaker::new(
        &mut builder,
        state,
        USB_PACKET_SIZE as u16,
        uac1::SampleResolution::Resolution32Bit,
        &[SAMPLE_RATE_HZ],
        &[uac1::ChannelConfig::LeftFront, uac1::ChannelConfig::RightFront],
        uac1::FeedbackRefreshPeriod::Period8ms,
    );

    // Build and run the USB device
    let mut usb_device = builder.build();
    let usb_fut = usb_device.run();

    // Establish a zero-copy channel for transferring received audio samples between tasks
    static SAMPLE_BLOCKS: StaticCell<[SampleBlock; SAMPLE_BLOCK_COUNT]> = StaticCell::new();
    let sample_blocks = SAMPLE_BLOCKS.init([([0.0; SAMPLE_COUNT], 0); SAMPLE_BLOCK_COUNT]);

    static CHANNEL: StaticCell<Channel<'_, NoopRawMutex, SampleBlock>> = StaticCell::new();
    let channel = CHANNEL.init(Channel::new(sample_blocks));
    let (mut sender, receiver) = channel.split();

    let receive_fut = async {
        loop {
            stream.wait_connection().await;
            info!("Connected");
            let _ = receive(&mut stream, &mut sender).await;
            info!("Disconnected");
        }
    };

    // Run everything concurrently.
    join(usb_fut, receive_fut).await;
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

async fn receive<'d, T: usb::Instance + 'd>(
    stream: &mut speaker::Stream<'d, usb::Driver<'d, T>>,
    sender: &mut Sender<'static, NoopRawMutex, SampleBlock>,
) -> Result<(), Disconnected> {
    loop {
        let mut usb_data = [0u8; USB_PACKET_SIZE];
        let data_size = stream.read_packet(&mut usb_data).await?;
        let word_count = data_size / SAMPLE_SIZE;

        if word_count * SAMPLE_SIZE == data_size {
            // Obtain a free buffer from the channel
            let (samples, sample_count) = sender.send().await;

            for w in 0..word_count {
                let byte_offset = w * SAMPLE_SIZE;
                let sample = sample_as_f32(u32::from_le_bytes(
                    usb_data[byte_offset..byte_offset + SAMPLE_SIZE].try_into().unwrap(),
                ));

                // Fill the sample buffer with data.
                samples[w] = sample;
            }

            *sample_count = word_count;

            sender.send_done();
        } else {
            info!("Invalid USB buffer size of {}, skipped.", data_size);
        }
    }
}
