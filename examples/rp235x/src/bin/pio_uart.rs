//! This example shows how to use the PIO module in the RP235x chip to implement a duplex UART.
//! The PIO module is a very powerful peripheral that can be used to implement many different
//! protocols. It is a very flexible state machine that can be programmed to do almost anything.
//!
//! This example opens up a USB device that implements a CDC ACM serial port. It then uses the
//! PIO module to implement a UART that is connected to the USB serial port. This allows you to
//! communicate with a device connected to the RP235x over USB serial.

#![no_std]
#![no_main]
#![allow(async_fn_in_trait)]

use defmt::{info, panic, trace};
use embassy_executor::Spawner;
use embassy_futures::join::{join, join3};
use embassy_rp::peripherals::{PIO0, USB};
use embassy_rp::pio_programs::uart::{PioUartRx, PioUartRxProgram, PioUartTx, PioUartTxProgram};
use embassy_rp::usb::{Driver, Instance, InterruptHandler};
use embassy_rp::{bind_interrupts, pio};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::pipe::Pipe;
use embassy_usb::class::cdc_acm::{CdcAcmClass, Receiver, Sender, State};
use embassy_usb::driver::EndpointError;
use embassy_usb::{Builder, Config};
use embedded_io_async::{Read, Write};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
    PIO0_IRQ_0 => pio::InterruptHandler<PIO0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Hello there!");

    let p = embassy_rp::init(Default::default());

    // Create the driver, from the HAL.
    let driver = Driver::new(p.USB, Irqs);

    // Create embassy-usb Config
    let mut config = Config::new(0xc0de, 0xcafe);
    config.manufacturer = Some("Embassy");
    config.product = Some("PIO UART example");
    config.serial_number = Some("12345678");
    config.max_power = 100;
    config.max_packet_size_0 = 64;

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
    let class = CdcAcmClass::new(&mut builder, &mut state, 64);

    // Build the builder.
    let mut usb = builder.build();

    // Run the USB device.
    let usb_fut = usb.run();

    // PIO UART setup
    let pio::Pio {
        mut common, sm0, sm1, ..
    } = pio::Pio::new(p.PIO0, Irqs);

    let tx_program = PioUartTxProgram::new(&mut common);
    let mut uart_tx = PioUartTx::new(9600, &mut common, sm0, p.PIN_4, &tx_program);

    let rx_program = PioUartRxProgram::new(&mut common);
    let mut uart_rx = PioUartRx::new(9600, &mut common, sm1, p.PIN_5, &rx_program);

    // Pipe setup
    let mut usb_pipe: Pipe<NoopRawMutex, 20> = Pipe::new();
    let (mut usb_pipe_reader, mut usb_pipe_writer) = usb_pipe.split();

    let mut uart_pipe: Pipe<NoopRawMutex, 20> = Pipe::new();
    let (mut uart_pipe_reader, mut uart_pipe_writer) = uart_pipe.split();

    let (mut usb_tx, mut usb_rx) = class.split();

    // Read + write from USB
    let usb_future = async {
        loop {
            info!("Wait for USB connection");
            usb_rx.wait_connection().await;
            info!("Connected");
            let _ = join(
                usb_read(&mut usb_rx, &mut uart_pipe_writer),
                usb_write(&mut usb_tx, &mut usb_pipe_reader),
            )
            .await;
            info!("Disconnected");
        }
    };

    // Read + write from UART
    let uart_future = join(
        uart_read(&mut uart_rx, &mut usb_pipe_writer),
        uart_write(&mut uart_tx, &mut uart_pipe_reader),
    );

    // Run everything concurrently.
    // If we had made everything `'static` above instead, we could do this using separate tasks instead.
    join3(usb_fut, usb_future, uart_future).await;
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

/// Read from the USB and write it to the UART TX pipe
async fn usb_read<'d, T: Instance + 'd>(
    usb_rx: &mut Receiver<'d, Driver<'d, T>>,
    uart_pipe_writer: &mut embassy_sync::pipe::Writer<'_, NoopRawMutex, 20>,
) -> Result<(), Disconnected> {
    let mut buf = [0; 64];
    loop {
        let n = usb_rx.read_packet(&mut buf).await?;
        let data = &buf[..n];
        trace!("USB IN: {:x}", data);
        (*uart_pipe_writer).write(data).await;
    }
}

/// Read from the USB TX pipe and write it to the USB
async fn usb_write<'d, T: Instance + 'd>(
    usb_tx: &mut Sender<'d, Driver<'d, T>>,
    usb_pipe_reader: &mut embassy_sync::pipe::Reader<'_, NoopRawMutex, 20>,
) -> Result<(), Disconnected> {
    let mut buf = [0; 64];
    loop {
        let n = (*usb_pipe_reader).read(&mut buf).await;
        let data = &buf[..n];
        trace!("USB OUT: {:x}", data);
        usb_tx.write_packet(&data).await?;
    }
}

/// Read from the UART and write it to the USB TX pipe
async fn uart_read<PIO: pio::Instance, const SM: usize>(
    uart_rx: &mut PioUartRx<'_, PIO, SM>,
    usb_pipe_writer: &mut embassy_sync::pipe::Writer<'_, NoopRawMutex, 20>,
) -> ! {
    let mut buf = [0; 64];
    loop {
        let n = uart_rx.read(&mut buf).await.expect("UART read error");
        if n == 0 {
            continue;
        }
        let data = &buf[..n];
        trace!("UART IN: {:x}", buf);
        (*usb_pipe_writer).write(data).await;
    }
}

/// Read from the UART TX pipe and write it to the UART
async fn uart_write<PIO: pio::Instance, const SM: usize>(
    uart_tx: &mut PioUartTx<'_, PIO, SM>,
    uart_pipe_reader: &mut embassy_sync::pipe::Reader<'_, NoopRawMutex, 20>,
) -> ! {
    let mut buf = [0; 64];
    loop {
        let n = (*uart_pipe_reader).read(&mut buf).await;
        let data = &buf[..n];
        trace!("UART OUT: {:x}", data);
        let _ = uart_tx.write(&data).await;
    }
}
