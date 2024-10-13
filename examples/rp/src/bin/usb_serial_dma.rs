#![no_std]
#![no_main]

use core::fmt::Write;
use heapless::String;
use defmt::{info, panic, unwrap};
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::USB;
use embassy_rp::adc::{Adc, Channel, Config, InterruptHandler as AdcInterruptHandler};
use embassy_rp::usb::{Driver, Instance, InterruptHandler as UsbInterruptHandler};
use embassy_time::{Duration, Ticker, Instant};
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use embassy_usb::driver::EndpointError;
use embassy_usb::UsbDevice;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => UsbInterruptHandler<USB>;
    ADC_IRQ_FIFO => AdcInterruptHandler;
});


#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Hello there!");

    let p = embassy_rp::init(Default::default());

    let mut adc = Adc::new(p.ADC, Irqs, Config::default());
    let mut dma = p.DMA_CH0;
    let mut pins = [
      Channel::new_pin(p.PIN_26, embassy_rp::gpio::Pull::Up),
      Channel::new_pin(p.PIN_27, embassy_rp::gpio::Pull::Up),
    ];

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
    let mut class = {
        static STATE: StaticCell<State> = StaticCell::new();
        let state = STATE.init(State::new());
        CdcAcmClass::new(&mut builder, state, 64)
    };

    // Build the builder.
    let usb = builder.build();

    // Run the USB device.
    unwrap!(spawner.spawn(usb_task(usb)));

    const BLOCK_SIZE: usize = 4000;
  
    const ALPHA: f64 = 0.68; // Define smoothing factor for exponential averaging
    let mut ema1: f64 = 0.0; // Initialize exponential moving average for channel 1
    let mut ema2: f64 = 0.0; // Initialize exponential moving average for channel 2

    let mut ticker = Ticker::every(Duration::from_millis(50));

    loop {
        class.wait_connection().await;

        let now = Instant::now().as_micros();

        // Read 100 samples from the ADC
        let mut buf = [0_u16; { BLOCK_SIZE * 2 }];
        let div = 96; // 500kHz sample rate (48Mhz / 100kHz - 1)
        adc.read_many_multichannel(&mut pins, &mut buf, div, &mut dma).await.unwrap();

        // Separate and process data for each channel
        let mut sum1: u64 = 0;
        let mut sum2: u64 = 0;

        // Interleaved data: even indices are channel 1, odd indices are channel 2
        for i in 0..BLOCK_SIZE {
            sum1 += buf[i * 2] as u64;       // Channel 1 (even indices)
            sum2 += buf[i * 2 + 1] as u64;   // Channel 2 (odd indices)
        }

        let avg1: f64 = (sum1 as f64) / (BLOCK_SIZE as f64);
        let avg2: f64 = (sum2 as f64) / (BLOCK_SIZE as f64);

        // Apply the exponential moving average (EMA) to the averaged data for both channels
        ema1 = ALPHA * avg1 + (1.0 - ALPHA) * ema1;
        ema2 = ALPHA * avg2 + (1.0 - ALPHA) * ema2;

        // Scale the EMA values to include 3 decimal places of precision (multiplied by 1000)
        let ema1_scaled = (ema1 * 1000.0) as u64;
        let ema2_scaled = (ema2 * 1000.0) as u64;

        // Separate integer and fractional parts for both channels
        let integer_part1 = ema1_scaled / 1000;
        let fractional_part1 = ema1_scaled % 1000;

        let integer_part2 = ema2_scaled / 1000;
        let fractional_part2 = ema2_scaled % 1000;

        let delta = Instant::now().as_micros() - now;

        // Convert the averages for both channels to strings with 3 decimal places of precision
        let mut avg_buf = String::<64>::new(); // Buffer for channel 1
        let _ = write!(avg_buf, "CH1: {}.{:03}, CH2:  {}.{:03}, us: {}\r\n", integer_part1, fractional_part1, integer_part2, fractional_part2, delta); // 3 decimal places for channel 1

        // Send the average over USB
        class.write_packet(avg_buf.as_bytes()).await.unwrap();

        ticker.next().await;
    }
}

type MyUsbDriver = Driver<'static, USB>;
type MyUsbDevice = UsbDevice<'static, MyUsbDriver>;

#[embassy_executor::task]
async fn usb_task(mut usb: MyUsbDevice) -> ! {
    usb.run().await
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
