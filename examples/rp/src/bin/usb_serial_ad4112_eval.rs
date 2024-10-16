#![no_std]
#![no_main]

use core::fmt::Write;
use defmt::{info, panic, unwrap};
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::{SPI0, USB};
use embassy_rp::spi::{Async, Config, Spi};
use embassy_rp::usb::{Driver, InterruptHandler as UsbInterruptHandler};
use embassy_time::{Duration, Instant, Ticker, Timer};
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use embassy_usb::driver::EndpointError;
use embassy_usb::UsbDevice;
use heapless::String;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

#[path = "../ad4112.rs"]
mod ad4112;

#[path = "../dac80508.rs"]
mod dac80508;

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => UsbInterruptHandler<USB>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Hello there!");

    let p = embassy_rp::init(Default::default());

    // Create the driver, from the HAL.
    let driver = Driver::new(p.USB, Irqs);

    // Create embassy-usb Config
    let config = {
        let mut config = embassy_usb::Config::new(0xc0de, 0xcafe);
        config.manufacturer = Some("Embassy");
        config.product = Some("USB-serial example");
        config.serial_number = Some("12345678");
        config.max_power = 100;
        config.max_packet_size_0 = 64;

        // Required for windows compatibility.
        config.device_class = 0xEF;
        config.device_sub_class = 0x02;
        config.device_protocol = 0x01;
        config.composite_with_iads = true;
        config
    };

    // Create embassy-usb DeviceBuilder using the driver and config.
    let mut builder = {
        static CONFIG_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
        static BOS_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
        static CONTROL_BUF: StaticCell<[u8; 64]> = StaticCell::new();

        let builder = embassy_usb::Builder::new(
            driver,
            config,
            CONFIG_DESCRIPTOR.init([0; 256]),
            BOS_DESCRIPTOR.init([0; 256]),
            &mut [], // no msos descriptors
            CONTROL_BUF.init([0; 64]),
        );
        builder
    };

    // Create classes on the builder.
    let class = {
        static STATE: StaticCell<State> = StaticCell::new();
        let state = STATE.init(State::new());
        CdcAcmClass::new(&mut builder, state, 64)
    };

    // Build the builder.
    let usb = builder.build();

    let miso = p.PIN_16;
    let mosi = p.PIN_19;
    let clk = p.PIN_18;
    let adc_cs = Output::new(p.PIN_17, Level::High);
    let dac_cs = Output::new(p.PIN_20, Level::High);

    let mut config = Config::default();
    config.polarity = embassy_rp::spi::Polarity::IdleHigh;
    config.phase = embassy_rp::spi::Phase::CaptureOnSecondTransition;
    config.frequency = 100_000;

    let spi = Spi::new(p.SPI0, clk, mosi, miso, p.DMA_CH0, p.DMA_CH1, config);

    // Run the USB device.
    unwrap!(spawner.spawn(usb_task(usb)));
    unwrap!(spawner.spawn(spi_task(class, spi, adc_cs, dac_cs)));
}

type MyUsbDriver = Driver<'static, USB>;
type MyUsbDevice = UsbDevice<'static, MyUsbDriver>;

#[embassy_executor::task]
async fn usb_task(mut usb: MyUsbDevice) -> ! {
    usb.run().await
}

#[embassy_executor::task]
async fn spi_task(
    mut class: CdcAcmClass<'static, Driver<'static, USB>>,
    mut spi: Spi<'static, SPI0, Async>,
    mut adc_cs: Output<'static>,
    mut dac_cs: Output<'static>,
) -> ! {
    let adc = ad4112::AD4112Driver::new();

    let dac = dac80508::DAC80508Driver::new();

    let id = adc.read_device_id(&mut spi, &mut adc_cs).await.unwrap();
    // Convert the averages for both channels to strings with 3 decimal places of precision
    let mut buf = String::<64>::new(); // Buffer for channel 1
    let _ = write!(buf, "id: {:#X}\r\n", id); // For uppercase hex, use "{:#X}"

    // Send the average over USB
    class.write_packet(buf.as_bytes()).await.unwrap();

    // Set up ADC
    adc.write_adc_mode(
        &mut spi,
        &mut adc_cs,
        ad4112::RefEnable::Enabled,
        ad4112::SingleCycle::Disabled,
        ad4112::Delay::Delay0us,
        ad4112::ADCModeOperation::ContinuousConversion,
        ad4112::ClockSelect::Internal,
    )
    .await
    .unwrap();

    adc.configure_setup(
        &mut spi,
        &mut adc_cs,
        ad4112::Register::SetupCon0,
        ad4112::BipolarUnipolar::Unipolar,
        ad4112::RefBufPlus::Enabled,
        ad4112::RefBufMinus::Enabled,
        ad4112::InputBuffer::Enabled,
        ad4112::ReferenceSelect::Internal2V5,
    )
    .await
    .unwrap();

    adc.configure_filter(
        &mut spi,
        &mut adc_cs,
        ad4112::Register::FiltCon0,
        ad4112::EnhancedFilter::Disabled,
        ad4112::EnhancedFilterSelection::Rejection27sps,
        ad4112::FilterOrder::Sinc5Sinc1,
        ad4112::OutputDataRate::Sps2597,
    )
    .await
    .unwrap();

    adc.enable_channel(
        &mut spi,
        &mut adc_cs,
        ad4112::Register::CH0,
        ad4112::ChannelEnable::Enabled,
        ad4112::SetupSelection::Setup0,
        ad4112::InputPair::Vin0VinCom,
    )
    .await
    .unwrap();

    Timer::after_millis(100).await;

    adc.write_adc_mode(
        &mut spi,
        &mut adc_cs,
        ad4112::RefEnable::Enabled,
        ad4112::SingleCycle::Disabled,
        ad4112::Delay::Delay0us,
        ad4112::ADCModeOperation::InternalOffsetCalibration,
        ad4112::ClockSelect::Internal,
    )
    .await
    .unwrap();

    Timer::after_millis(100).await;

    adc.write_adc_mode(
        &mut spi,
        &mut adc_cs,
        ad4112::RefEnable::Enabled,
        ad4112::SingleCycle::Disabled,
        ad4112::Delay::Delay0us,
        ad4112::ADCModeOperation::InternalGainCalibration,
        ad4112::ClockSelect::Internal,
    )
    .await
    .unwrap();

    Timer::after_millis(10).await;

    adc.write_adc_mode(
        &mut spi,
        &mut adc_cs,
        ad4112::RefEnable::Enabled,
        ad4112::SingleCycle::Disabled,
        ad4112::Delay::Delay0us,
        ad4112::ADCModeOperation::ContinuousConversion,
        ad4112::ClockSelect::Internal,
    )
    .await
    .unwrap();

    Timer::after_millis(1000).await;

    let gain = adc
        .read_register(&mut spi, &mut adc_cs, ad4112::Register::Gain0, 3)
        .await
        .unwrap();
    let offset = adc
        .read_register(&mut spi, &mut adc_cs, ad4112::Register::Offset0, 3)
        .await
        .unwrap();

    let mut buf = String::<64>::new(); // Buffer for channel 1
    let _ = write!(buf, "gain: {:#X}, off: {:#X}\r\n", gain, offset); // For uppercase hex, use "{:#X}"

    // Send the average over USB
    class.write_packet(buf.as_bytes()).await.unwrap();

    let mut config = Config::default();
    config.polarity = embassy_rp::spi::Polarity::IdleLow;
    config.phase = embassy_rp::spi::Phase::CaptureOnSecondTransition;
    config.frequency = 100_000;

    spi.set_config(config);

    // set up DAC
    let reset = dac80508::TriggerRegisterBuilder::new().soft_reset().build();

    dac.write_register(&mut spi, &mut dac_cs, dac80508::Register::Trigger, reset)
        .await
        .ok();

    let sync = dac80508::SyncRegisterBuilder::new()
        .dac0_broadcast_en(false)
        .dac1_broadcast_en(false)
        .dac2_broadcast_en(false)
        .dac3_broadcast_en(false)
        .dac4_broadcast_en(false)
        .dac5_broadcast_en(false)
        .dac6_broadcast_en(false)
        .dac7_broadcast_en(false)
        .dac0_sync_en(true)
        .dac1_sync_en(true)
        .dac2_sync_en(true)
        .dac3_sync_en(true)
        .dac4_sync_en(true)
        .dac5_sync_en(true)
        .dac6_sync_en(true)
        .dac7_sync_en(true)
        .build();

    dac.write_register(&mut spi, &mut dac_cs, dac80508::Register::Sync, sync)
        .await
        .ok();

    let gain = dac80508::GainRegisterBuilder::new()
        .refdiv_en(false)
        .buff0_gain(true)
        .buff1_gain(true)
        .buff2_gain(true)
        .buff3_gain(true)
        .buff4_gain(true)
        .buff5_gain(true)
        .buff6_gain(true)
        .buff7_gain(true)
        .build();

    dac.write_register(&mut spi, &mut dac_cs, dac80508::Register::Gain, gain)
        .await
        .ok();

    let alarm = dac
        .read_register(&mut spi, &mut dac_cs, dac80508::Register::Status)
        .await
        .unwrap_or((0x00, 0x00));

    let id = dac
        .read_register(&mut spi, &mut dac_cs, dac80508::Register::DeviceId)
        .await
        .unwrap_or((0x00, 0x00));

    let mut buf = String::<64>::new(); // Buffer for channel 1
    let _ = write!(buf, "dac id: {:#X}, a: {:#X}\r\n", id.1, alarm.1); // For uppercase hex, use "{:#X}"

    // Send the average over USB
    class.write_packet(buf.as_bytes()).await.unwrap();

    let mut dac0_value: u16 = 0x00_00;

    let mut ticker = Ticker::every(Duration::from_millis(500));

    loop {
        let now = Instant::now().as_micros();

        let mut config = Config::default();
        config.polarity = embassy_rp::spi::Polarity::IdleHigh;
        config.phase = embassy_rp::spi::Phase::CaptureOnSecondTransition;
        config.frequency = 100_000;

        spi.set_config(config);

        for (i, (channel_register, input_pair)) in [
            (ad4112::Register::CH0, ad4112::InputPair::Vin0VinCom),
            (ad4112::Register::CH0, ad4112::InputPair::Vin1VinCom),
            (ad4112::Register::CH0, ad4112::InputPair::Vin2VinCom),
            (ad4112::Register::CH0, ad4112::InputPair::Vin3VinCom),
            (ad4112::Register::CH0, ad4112::InputPair::Vin4VinCom),
            (ad4112::Register::CH0, ad4112::InputPair::Vin5VinCom),
            (ad4112::Register::CH0, ad4112::InputPair::Vin6VinCom),
            (ad4112::Register::CH0, ad4112::InputPair::Vin7VinCom),
        ]
        .iter()
        .enumerate()
        {
            // Enable the channel with VinXVinCom pairs
            adc.enable_channel(
                &mut spi,
                &mut adc_cs,
                *channel_register,
                ad4112::ChannelEnable::Enabled,
                ad4112::SetupSelection::Setup0,
                *input_pair,
            )
            .await
            .unwrap();

            Timer::after_micros(500).await;

            // Read data from the channel
            let data = adc.read_data(&mut spi, &mut adc_cs).await.unwrap();

            // Convert the data to voltage
            let voltage = ad4112::AD4112Driver::convert_unipolar_voltage(data, 2.5_f64);

            // Buffer to hold the formatted output
            let mut buf = String::<64>::new();

            // Format the data and voltage as a string
            let _ = write!(buf, "Channel {}: {:#X}, {}\r\n", i, data, voltage);

            // Send the data over USB
            class.write_packet(buf.as_bytes()).await.ok();
        }

        let delta = Instant::now().as_micros() - now;

        let mut config = Config::default();
        config.polarity = embassy_rp::spi::Polarity::IdleLow;
        config.phase = embassy_rp::spi::Phase::CaptureOnSecondTransition;
        config.frequency = 100_000;

        spi.set_config(config);

        for (i, channel_register) in [
            dac80508::Register::Dac0,
            dac80508::Register::Dac1,
            dac80508::Register::Dac2,
            dac80508::Register::Dac3,
            dac80508::Register::Dac4,
            dac80508::Register::Dac5,
            dac80508::Register::Dac6,
            dac80508::Register::Dac7,
        ]
        .iter()
        .enumerate()
        {
            dac.write_register(
                &mut spi,
                &mut dac_cs,
                *channel_register,
                dac0_value.wrapping_add(0xFF_u16.wrapping_mul(0xFF).wrapping_mul(i as u16)),
            )
            .await
            .ok();
        }

        let sync_byte = dac80508::TriggerRegisterBuilder::new().ldac(true).build();

        dac.write_register(&mut spi, &mut dac_cs, dac80508::Register::Trigger, sync_byte)
            .await
            .ok();

        // Increment dac0_value and wrap around if it exceeds 0xFFFF
        dac0_value = dac0_value.wrapping_add(0xFF);

        // Buffer to hold the formatted output
        let mut buf = String::<64>::new();

        // Format the data and voltage as a string
        let _ = write!(buf, "us: {}\r\n", delta);

        // Send the data over USB
        class.write_packet(buf.as_bytes()).await.ok();

        ticker.next().await;
    }
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
