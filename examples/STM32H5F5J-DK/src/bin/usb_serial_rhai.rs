#![no_std]
#![no_main]

use core::str;

use alloc::format;
use defmt::{panic, *};
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_stm32::time::Hertz;
use embassy_stm32::usb::{Driver, Instance};
use embassy_stm32::{bind_interrupts, peripherals, usb, Config};
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use embassy_usb::driver::EndpointError;
use embassy_usb::Builder;

extern crate alloc;
use embedded_alloc::Heap;

use rhai::{packages::Package, Engine, INT};

use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    USB_DRD_FS => usb::InterruptHandler<peripherals::USB>;
});

#[global_allocator]
static HEAP: Heap = Heap::empty();

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hsi = None;
        config.rcc.hsi48 = Some(Default::default()); // needed for RNG
        config.rcc.hse = Some(Hse {
            freq: Hertz(25_000_000),
            mode: HseMode::BypassDigital,
        });
        config.rcc.pll1 = Some(Pll {
            source: PllSource::HSE,
            prediv: PllPreDiv::DIV5,
            mul: PllMul::MUL100,
            divp: Some(PllDiv::DIV2),
            divq: Some(PllDiv::DIV2),
            divr: None,
        });
        config.rcc.ahb_pre = AHBPrescaler::DIV1;
        config.rcc.apb1_pre = APBPrescaler::DIV1;
        config.rcc.apb2_pre = APBPrescaler::DIV1;
        config.rcc.apb3_pre = APBPrescaler::DIV1;
        config.rcc.sys = Sysclk::PLL1_P;
        config.rcc.voltage_scale = VoltageScale::Scale0;
        config.rcc.mux.usbsel = mux::Usbsel::HSI48;
    }
    let p = embassy_stm32::init(config);

    // Initialize the allocator BEFORE you use it
    {
        use core::mem::MaybeUninit;
        const HEAP_SIZE: usize = 16384;
        static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
        unsafe { HEAP.init(HEAP_MEM.as_ptr() as usize, HEAP_SIZE) }
    }

    info!("Hello World!");

    // Create the driver, from the HAL.
    let driver = Driver::new(p.USB, Irqs, p.PA12, p.PA11);

    // Create embassy-usb Config
    let mut config = embassy_usb::Config::new(0xc0de, 0xcafe);
    config.manufacturer = Some("Embassy");
    config.product = Some("USB-serial example");
    config.serial_number = Some("12345678");

    // Required for windows compatibility.
    // https://developer.nordicsemi.com/nRF_Connect_SDK/doc/1.9.1/kconfig/CONFIG_CDC_ACM_IAD.html#help
    config.device_class = 0xEF;
    config.device_sub_class = 0x02;
    config.device_protocol = 0x01;
    config.composite_with_iads = true;

    // Create embassy-usb DeviceBuilder using the driver and config.
    // It needs some buffers for building the descriptors.
    let mut config_descriptor = [0; 256];
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

    // Create classes on the builder.
    let mut class = CdcAcmClass::new(&mut builder, &mut state, 64);

    // Build the builder.
    let mut usb = builder.build();

    // Run the USB device.
    let usb_fut = usb.run();

    let mut engine = Engine::new_raw();

    // this bit gets commented out to test size without Core
    let std = rhai::packages::CorePackage::new();
    std.register_into_engine(&mut engine);

    // Do stuff with the class!
    let echo_fut = async {
        loop {
            class.wait_connection().await;
            info!("Connected");
            let _ = echo(&mut class, &mut engine).await;
            info!("Disconnected");
        }
    };

    // Run everything concurrently.
    // If we had made everything `'static` above instead, we could do this using separate tasks instead.
    join(usb_fut, echo_fut).await;
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

async fn echo<'d, T: Instance + 'd>(class: &mut CdcAcmClass<'d, Driver<'d, T>>, engine: &mut Engine) -> Result<(), Disconnected> {
    let mut buffer = [0u8; 64];
    let mut pos = 0;

    loop {
        class.wait_connection().await;

        match class.read_packet(&mut buffer[pos..]).await {
            Ok(size) => {
                for &byte in &buffer[pos..pos + size] {
                    // Echo each character
                    if let Err(e) = class.write_packet(&[byte]).await {
                        error!("Failed to write packet: {:?}", e);
                        continue;
                    }

                    // Check for newline characters
                    if byte == b'\n' || byte == b'\r' {
                        // Convert buffer to &str
                        if let Ok(line) = str::from_utf8(&buffer[..pos]) {
                            // Process the received line
                            info!("Received line: {}", line);
                            match engine.eval_expression::<INT>(line) {
                                Ok(res) => {
                                    write_in_chunks(class, format!("{:?},\r\n{:?}\n\r", line, res).as_bytes()).await?
                                }
                                Err(e) => {
                                    let mes = format!("Failed to process line: {:?}\n\rError: {:?}\n\r", line, e);
                                    write_in_chunks(class, mes.as_bytes()).await?
                                }
                            }
                        } else {
                            error!("Failed to convert buffer to string");
                        }
                        pos = 0; // Reset the buffer position
                    } else {
                        pos += 1;
                    }
                }
            }
            Err(e) => {
                error!("Error reading packet: {:?}", e);
            }
        }
    }
}

async fn write_in_chunks<'d, T: Instance + 'd>(
    class: &mut CdcAcmClass<'d, Driver<'d, T>>,
    data: &[u8],
) -> Result<(), Disconnected> {
    let chunk_size = 64;
    for chunk in data.chunks(chunk_size) {
        class.write_packet(chunk).await?;
    }
    Ok(())
}