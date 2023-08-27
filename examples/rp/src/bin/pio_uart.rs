//! This example shows how to use the PIO module in the RP2040 chip to implement a duplex UART.
//! The PIO module is a very powerful peripheral that can be used to implement many different
//! protocols. It is a very flexible state machine that can be programmed to do almost anything.
//!
//! This example opens up a USB device that implements a CDC ACM serial port. It then uses the
//! PIO module to implement a UART that is connected to the USB serial port. This allows you to
//! communicate with a device connected to the RP2040 over USB serial.

#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(async_fn_in_trait)]

use defmt::{info, panic, trace};
use embassy_executor::Spawner;
use embassy_futures::join::{join, join3};
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::{PIO0, USB};
use embassy_rp::pio::InterruptHandler as PioInterruptHandler;
use embassy_rp::usb::{Driver, Instance, InterruptHandler};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::pipe::Pipe;
use embassy_usb::class::cdc_acm::{CdcAcmClass, Receiver, Sender, State};
use embassy_usb::driver::EndpointError;
use embassy_usb::{Builder, Config};
use embedded_io_async::{Read, Write};
use {defmt_rtt as _, panic_probe as _};

use crate::uart::PioUart;
use crate::uart_rx::PioUartRx;
use crate::uart_tx::PioUartTx;

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
    PIO0_IRQ_0 => PioInterruptHandler<PIO0>;
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

    // Required for windows compatibility.
    // https://developer.nordicsemi.com/nRF_Connect_SDK/doc/1.9.1/kconfig/CONFIG_CDC_ACM_IAD.html#help
    config.device_class = 0xEF;
    config.device_sub_class = 0x02;
    config.device_protocol = 0x01;
    config.composite_with_iads = true;

    // Create embassy-usb DeviceBuilder using the driver and config.
    // It needs some buffers for building the descriptors.
    let mut device_descriptor = [0; 256];
    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    let mut control_buf = [0; 64];

    let mut state = State::new();

    let mut builder = Builder::new(
        driver,
        config,
        &mut device_descriptor,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut control_buf,
    );

    // Create classes on the builder.
    let class = CdcAcmClass::new(&mut builder, &mut state, 64);

    // Build the builder.
    let mut usb = builder.build();

    // Run the USB device.
    let usb_fut = usb.run();

    // PIO UART setup
    let uart = PioUart::new(9600, p.PIO0, p.PIN_4, p.PIN_5);
    let (mut uart_tx, mut uart_rx) = uart.split();

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
        uart_pipe_writer.write(data).await;
    }
}

/// Read from the USB TX pipe and write it to the USB
async fn usb_write<'d, T: Instance + 'd>(
    usb_tx: &mut Sender<'d, Driver<'d, T>>,
    usb_pipe_reader: &mut embassy_sync::pipe::Reader<'_, NoopRawMutex, 20>,
) -> Result<(), Disconnected> {
    let mut buf = [0; 64];
    loop {
        let n = usb_pipe_reader.read(&mut buf).await;
        let data = &buf[..n];
        trace!("USB OUT: {:x}", data);
        usb_tx.write_packet(&data).await?;
    }
}

/// Read from the UART and write it to the USB TX pipe
async fn uart_read(
    uart_rx: &mut PioUartRx<'_>,
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
        usb_pipe_writer.write(data).await;
    }
}

/// Read from the UART TX pipe and write it to the UART
async fn uart_write(
    uart_tx: &mut PioUartTx<'_>,
    uart_pipe_reader: &mut embassy_sync::pipe::Reader<'_, NoopRawMutex, 20>,
) -> ! {
    let mut buf = [0; 64];
    loop {
        let n = uart_pipe_reader.read(&mut buf).await;
        let data = &buf[..n];
        trace!("UART OUT: {:x}", data);
        let _ = uart_tx.write(&data).await;
    }
}

mod uart {
    use embassy_rp::peripherals::PIO0;
    use embassy_rp::pio::{Pio, PioPin};
    use embassy_rp::Peripheral;

    use crate::uart_rx::PioUartRx;
    use crate::uart_tx::PioUartTx;
    use crate::Irqs;

    pub struct PioUart<'a> {
        tx: PioUartTx<'a>,
        rx: PioUartRx<'a>,
    }

    impl<'a> PioUart<'a> {
        pub fn new(
            baud: u64,
            pio: impl Peripheral<P = PIO0> + 'a,
            tx_pin: impl PioPin,
            rx_pin: impl PioPin,
        ) -> PioUart<'a> {
            let Pio {
                mut common, sm0, sm1, ..
            } = Pio::new(pio, Irqs);

            let tx = PioUartTx::new(&mut common, sm0, tx_pin, baud);
            let rx = PioUartRx::new(&mut common, sm1, rx_pin, baud);

            PioUart { tx, rx }
        }

        pub fn split(self) -> (PioUartTx<'a>, PioUartRx<'a>) {
            (self.tx, self.rx)
        }
    }
}

mod uart_tx {
    use core::convert::Infallible;

    use embassy_rp::gpio::Level;
    use embassy_rp::peripherals::PIO0;
    use embassy_rp::pio::{Common, Config, Direction, FifoJoin, PioPin, ShiftDirection, StateMachine};
    use embedded_io_async::{ErrorType, Write};
    use fixed::traits::ToFixed;
    use fixed_macro::types::U56F8;

    pub struct PioUartTx<'a> {
        sm_tx: StateMachine<'a, PIO0, 0>,
    }

    impl<'a> PioUartTx<'a> {
        pub fn new(
            common: &mut Common<'a, PIO0>,
            mut sm_tx: StateMachine<'a, PIO0, 0>,
            tx_pin: impl PioPin,
            baud: u64,
        ) -> Self {
            let prg = pio_proc::pio_asm!(
                r#"
                .side_set 1 opt

                ; An 8n1 UART transmit program.
                ; OUT pin 0 and side-set pin 0 are both mapped to UART TX pin.

                    pull       side 1 [7]  ; Assert stop bit, or stall with line in idle state
                    set x, 7   side 0 [7]  ; Preload bit counter, assert start bit for 8 clocks
                bitloop:                   ; This loop will run 8 times (8n1 UART)
                    out pins, 1            ; Shift 1 bit from OSR to the first OUT pin
                    jmp x-- bitloop   [6]  ; Each loop iteration is 8 cycles.
            "#
            );
            let tx_pin = common.make_pio_pin(tx_pin);
            sm_tx.set_pins(Level::High, &[&tx_pin]);
            sm_tx.set_pin_dirs(Direction::Out, &[&tx_pin]);

            let mut cfg = Config::default();

            cfg.set_out_pins(&[&tx_pin]);
            cfg.use_program(&common.load_program(&prg.program), &[&tx_pin]);
            cfg.shift_out.auto_fill = false;
            cfg.shift_out.direction = ShiftDirection::Right;
            cfg.fifo_join = FifoJoin::TxOnly;
            cfg.clock_divider = (U56F8!(125_000_000) / (8 * baud)).to_fixed();
            sm_tx.set_config(&cfg);
            sm_tx.set_enable(true);

            Self { sm_tx }
        }

        pub async fn write_u8(&mut self, data: u8) {
            self.sm_tx.tx().wait_push(data as u32).await;
        }
    }

    impl ErrorType for PioUartTx<'_> {
        type Error = Infallible;
    }

    impl Write for PioUartTx<'_> {
        async fn write(&mut self, buf: &[u8]) -> Result<usize, Infallible> {
            for byte in buf {
                self.write_u8(*byte).await;
            }
            Ok(buf.len())
        }
    }
}

mod uart_rx {
    use core::convert::Infallible;

    use embassy_rp::gpio::Level;
    use embassy_rp::peripherals::PIO0;
    use embassy_rp::pio::{Common, Config, Direction, FifoJoin, PioPin, ShiftDirection, StateMachine};
    use embedded_io_async::{ErrorType, Read};
    use fixed::traits::ToFixed;
    use fixed_macro::types::U56F8;

    pub struct PioUartRx<'a> {
        sm_rx: StateMachine<'a, PIO0, 1>,
    }

    impl<'a> PioUartRx<'a> {
        pub fn new(
            common: &mut Common<'a, PIO0>,
            mut sm_rx: StateMachine<'a, PIO0, 1>,
            rx_pin: impl PioPin,
            baud: u64,
        ) -> Self {
            let prg = pio_proc::pio_asm!(
                r#"
                ; Slightly more fleshed-out 8n1 UART receiver which handles framing errors and
                ; break conditions more gracefully.
                ; IN pin 0 and JMP pin are both mapped to the GPIO used as UART RX.

                start:
                    wait 0 pin 0        ; Stall until start bit is asserted
                    set x, 7    [10]    ; Preload bit counter, then delay until halfway through
                rx_bitloop:             ; the first data bit (12 cycles incl wait, set).
                    in pins, 1          ; Shift data bit into ISR
                    jmp x-- rx_bitloop [6] ; Loop 8 times, each loop iteration is 8 cycles
                    jmp pin good_rx_stop   ; Check stop bit (should be high)

                    irq 4 rel           ; Either a framing error or a break. Set a sticky flag,
                    wait 1 pin 0        ; and wait for line to return to idle state.
                    jmp start           ; Don't push data if we didn't see good framing.

                good_rx_stop:           ; No delay before returning to start; a little slack is
                    in null 24
                    push                ; important in case the TX clock is slightly too fast.
            "#
            );
            let mut cfg = Config::default();
            cfg.use_program(&common.load_program(&prg.program), &[]);

            let rx_pin = common.make_pio_pin(rx_pin);
            sm_rx.set_pins(Level::High, &[&rx_pin]);
            cfg.set_in_pins(&[&rx_pin]);
            cfg.set_jmp_pin(&rx_pin);
            sm_rx.set_pin_dirs(Direction::In, &[&rx_pin]);

            cfg.clock_divider = (U56F8!(125_000_000) / (8 * baud)).to_fixed();
            cfg.shift_in.auto_fill = false;
            cfg.shift_in.direction = ShiftDirection::Right;
            cfg.shift_in.threshold = 32;
            cfg.fifo_join = FifoJoin::RxOnly;
            sm_rx.set_config(&cfg);
            sm_rx.set_enable(true);

            Self { sm_rx }
        }

        pub async fn read_u8(&mut self) -> u8 {
            self.sm_rx.rx().wait_pull().await as u8
        }
    }

    impl ErrorType for PioUartRx<'_> {
        type Error = Infallible;
    }

    impl Read for PioUartRx<'_> {
        async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Infallible> {
            let mut i = 0;
            while i < buf.len() {
                buf[i] = self.read_u8().await;
                i += 1;
            }
            Ok(i)
        }
    }
}
