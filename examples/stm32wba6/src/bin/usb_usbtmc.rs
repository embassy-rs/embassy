//! This example demonstrates a USB Test and Measurement Class (USBTMC / USB488) instrument
//! running on an STM32WBA65 microcontroller.
//!
//! # Running it
//!
//! ```sh
//! cd examples/stm32wba6
//! cargo run --bin usb_usbtmc     # flashes and streams the defmt log over RTT
//! ```
//!
//! The board enumerates as `1209:0001` and the kernel binds its `usbtmc` driver, which
//! creates `/dev/usbtmcN`. That node is normally root-only, so install a udev rule (or run
//! the host tool as root) before using PyVISA:
//!
//! ```sh
//! echo 'SUBSYSTEM=="usbmisc", KERNEL=="usbtmc*", MODE="0666"' \
//!   | sudo tee /etc/udev/rules.d/99-embassy-usbtmc.rules
//! sudo udevadm control --reload && sudo udevadm trigger
//! ```
//!
//! To talk raw USB instead (to send malformed messages or clear halts by hand), give
//! libusb access to the device node. The host tool then has to detach the kernel driver
//! (`dev.detach_kernel_driver(0)`) before it can claim the interface:
//!
//! ```sh
//! echo 'SUBSYSTEM=="usb", ATTR{idVendor}=="1209", ATTR{idProduct}=="0001", MODE="0666"' \
//!   | sudo tee /etc/udev/rules.d/99-embassy-usbtmc-raw.rules
//! sudo udevadm control --reload && sudo udevadm trigger
//! python3 -m venv /tmp/usbtmc-venv && /tmp/usbtmc-venv/bin/pip install pyusb
//! ```
//!
//! # Talking to it
//!
//! With PyVISA:
//!
//! ```python
//! import pyvisa
//! rm = pyvisa.ResourceManager()
//! inst = rm.open_resource("USB0::0x1209::0x0001::12345678::INSTR")
//! print(inst.query("*IDN?"))
//! print(inst.query(":MEAS:VOLT?"))
//! ```
//!
//! The instrument answers these SCPI commands:
//!
//! - `*IDN?` - identification string.
//! - `*STB?` - status byte (always `0`).
//! - `:MEAS:VOLT?` - a fixed measurement.
//! - `*RST` - logged, state unchanged.
//! - `:TEST:STALL` - halts both bulk endpoints instead of replying, to exercise the
//!   endpoint-halt path. The host's next transfer fails with `EPIPE`; clear it with
//!   `usb_clear_halt()`/`CLEAR_FEATURE(ENDPOINT_HALT)` or `viClear()`, and the instrument
//!   starts answering again.
//!
//! Any other command is logged and ignored. A Bulk-OUT message that cannot be parsed
//! halts Bulk-OUT on its own; see the `usbtmc` module docs.

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

            // Endpoint-halt self-test: stall and un-stall the bulk endpoints once, and log
            // what the driver reads back. This exercises the real halt feature (registers,
            // not just class bookkeeping) before the host starts a transfer, and leaves the
            // endpoints usable.
            tmc.stall();
            info!("self-test: stalled = {}", tmc.is_stalled());
            tmc.clear_stall();
            info!("self-test: stalled after clear = {}", tmc.is_stalled());

            let mut buf = [0u8; 256];
            loop {
                let msg = match tmc.read_message(&mut buf).await {
                    Ok(msg) => msg,
                    Err(UsbtmcError::Endpoint(_)) => break,
                    Err(e) => {
                        info!("Read error: {:?}", defmt::Debug2Format(&e));
                        // The class has halted Bulk-OUT, so the host sees a failed
                        // transfer instead of us misreading the rest of the stream. It now
                        // clears the halt with CLEAR_FEATURE(ENDPOINT_HALT) or `viClear()`,
                        // and this read resumes. Clearing the stall here would reset the
                        // data toggle behind the host's back, so don't.
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
                } else if cmd.eq_ignore_ascii_case(":TEST:STALL") {
                    // Reject the transfer the way an instrument that cannot serve a
                    // command would: halt the bulk endpoints instead of replying. The
                    // host sees the failed transfer and clears the halt with
                    // CLEAR_FEATURE(ENDPOINT_HALT) or `viClear()`; the device recovers
                    // on its own once that happens.
                    info!("Stalling the bulk endpoints on request");
                    tmc.stall();
                    info!("Stalled = {}", tmc.is_stalled());
                    continue;
                } else {
                    info!("Unknown command: {}", cmd);
                }
            }

            info!("USBTMC device disconnected");
        }
    };

    join(usb_fut, tmc_fut).await;
}
