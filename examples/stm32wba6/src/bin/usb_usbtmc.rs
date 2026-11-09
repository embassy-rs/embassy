//! This example demonstrates a USB Test and Measurement Class (USBTMC / USB488) instrument
//! running on an STM32WBA65 microcontroller.
//!
//! You can interact with this device using PyVISA from a host computer:
//! ```python
//! import pyvisa
//! rm = pyvisa.ResourceManager()
//! inst = rm.open_resource("USB0::0x1209::0x0001::12345678::INSTR")
//! print(inst.query("*IDN?"))
//! print(inst.query(":MEAS:VOLT?"))
//! ```

#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_stm32::usb::Driver;
use embassy_stm32::{Config, bind_interrupts, peripherals, usb};
use embassy_usb::Builder;
use embassy_usb::class::usbtmc::{Config as TmcConfig, State, UsbtmcClass, UsbtmcError};
use panic_probe as _;

bind_interrupts!(struct Irqs {
    USB_OTG_HS => usb::InterruptHandler<peripherals::USB_OTG_HS>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Starting USBTMC instrument on STM32WBA65");

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

    // Create the USB OTG HS driver.
    let mut ep_out_buffer = [0u8; 1024];
    let mut usb_config = embassy_stm32::usb::Config::default();
    usb_config.vbus_detection = false;
    let driver = Driver::new_hs(p.USB_OTG_HS, p.PD6, p.PD7, Irqs, &mut ep_out_buffer, usb_config);

    // Create embassy-usb Config
    let mut usb_dev_config = embassy_usb::Config::new(0x1209, 0x0001);
    usb_dev_config.manufacturer = Some("Embassy");
    usb_dev_config.product = Some("STM32WBA65 USBTMC Instrument");
    usb_dev_config.serial_number = Some("12345678");
    usb_dev_config.max_power = 100;
    usb_dev_config.max_packet_size_0 = 64;

    // Descriptors and buffers
    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    let mut control_buf = [0; 64];

    let mut state = State::new();

    let mut builder = Builder::new(
        driver,
        usb_dev_config,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut [],
        &mut control_buf,
    );

    let mut tmc_config = TmcConfig::new_usb488();
    tmc_config.max_packet_size = 512;
    let mut tmc = UsbtmcClass::new(&mut builder, &mut state, tmc_config);

    let mut usb = builder.build();

    let usb_fut = usb.run();

    let tmc_fut = async {
        loop {
            tmc.wait_connection().await;
            info!("USBTMC device connected");

            let mut buf = [0u8; 256];
            loop {
                let msg = match tmc.read_message(&mut buf).await {
                    Ok(msg) => msg,
                    Err(UsbtmcError::Endpoint(_)) => break,
                    Err(e) => {
                        info!("Read error: {:?}", defmt::Debug2Format(&e));
                        // A message that could not be parsed stalls Bulk-OUT, so the host
                        // notices instead of having us misread the rest of the stream.
                        // Clear it to keep accepting commands (the host can also clear it
                        // itself with CLEAR_FEATURE(ENDPOINT_HALT), and viClear() does too).
                        tmc.clear_stall();
                        continue;
                    }
                };

                let cmd = core::str::from_utf8(msg.payload).unwrap_or("").trim_end();
                info!("Received command: {}", cmd);

                if cmd.eq_ignore_ascii_case("*IDN?") {
                    let _ = tmc
                        .write_message(b"STMicroelectronics,STM32WBA65-USBTMC,SN12345678,v0.1.0\n", true)
                        .await;
                } else if cmd.eq_ignore_ascii_case("*STB?") {
                    let _ = tmc.write_message(b"0\n", true).await;
                } else if cmd.eq_ignore_ascii_case(":MEAS:VOLT?") {
                    let _ = tmc.write_message(b"+3.3000E+00\n", true).await;
                } else if cmd.eq_ignore_ascii_case("*RST") {
                    info!("Instrument reset");
                } else {
                    info!("Unknown command: {}", cmd);
                }
            }

            info!("USBTMC device disconnected");
        }
    };

    join(usb_fut, tmc_fut).await;
}
