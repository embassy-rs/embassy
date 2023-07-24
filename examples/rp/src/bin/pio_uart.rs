//! This example shows how to use USB (Universal Serial Bus) in the RP2040 chip.
//!
//! This creates a USB serial port that echos.

#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::{info, panic};
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::{PIO0, USB};
use embassy_rp::usb::{Driver, Instance, InterruptHandler};
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use embassy_usb::driver::EndpointError;
use embassy_usb::{Builder, Config};
use embassy_rp::pio::{InterruptHandler as PioInterruptHandler};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct UsbIrqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});

bind_interrupts!(struct PioIrqs {
    PIO0_IRQ_0 => PioInterruptHandler<PIO0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Hello there!");

    let p = embassy_rp::init(Default::default());

    // Create the driver, from the HAL.
    let driver = Driver::new(p.USB, UsbIrqs);

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
    let mut class = CdcAcmClass::new(&mut builder, &mut state, 64);

    // Build the builder.
    let mut usb = builder.build();

    // Run the USB device.
    let usb_fut = usb.run();

    // Do stuff with the class!
    let echo_fut = async {
        loop {
            class.wait_connection().await;
            info!("Connected");
            let _ = echo(&mut class).await;
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

async fn echo<'d, T: Instance + 'd>(class: &mut CdcAcmClass<'d, Driver<'d, T>>) -> Result<(), Disconnected> {
    let mut buf = [0; 64];
    loop {
        let n = class.read_packet(&mut buf).await?;
        let data = &buf[..n];
        info!("data: {:x}", data);
        class.write_packet(data).await?;
    }
}

mod uart {
    use embassy_rp::peripherals::PIO0;
    use embassy_rp::pio::{Common, Pio, PioPin, StateMachine};
    use embassy_rp::Peripheral;

    use crate::PioIrqs;

    pub struct PioUart<'a> {
        baud: u64,
        pio: Common<'a, PIO0>,
        sm0: StateMachine<'a, PIO0, 0>,
        sm1: StateMachine<'a, PIO0, 1>,
    }

    impl<'a> PioUart<'a> {
        pub async fn new(
            baud: u64,
            pio: impl Peripheral<P = PIO0> + 'a,
            tx_pin: impl PioPin,
            rx_pin: impl PioPin,
        ) -> PioUart<'a> {
            let Pio {
                mut common,
                mut sm0,
                mut sm1,
                ..
            } = Pio::new(pio, PioIrqs);

            crate::uart_tx::setup_uart_tx_on_sm0(&mut common, &mut sm0, tx_pin, baud);
            crate::uart_rx::setup_uart_rx_on_sm1(&mut common, &mut sm1, rx_pin, baud);

            PioUart {
                baud,
                pio: common,
                sm0,
                sm1,
            }
        }
    }
}

mod uart_tx {
    use embassy_rp::gpio::Level;
    use embassy_rp::peripherals::PIO0;
    use embassy_rp::pio::{Common, Config, Direction, FifoJoin, PioPin, ShiftDirection, StateMachine};
    use embassy_rp::relocate::RelocatedProgram;
    use fixed::traits::ToFixed;
    use fixed_macro::types::U56F8;

    pub fn setup_uart_tx_on_sm0<'a>(
        common: &mut Common<'a, PIO0>,
        sm_tx: &mut StateMachine<'a, PIO0, 0>,
        tx_pin: impl PioPin,
        baud: u64,
    ) {
        let prg = pio_proc::pio_asm!(
            r#"
                ;.program uart_tx
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

        let relocated = RelocatedProgram::new(&prg.program);
        let mut cfg = Config::default();

        cfg.use_program(&common.load_program(&relocated), &[&tx_pin]);
        cfg.clock_divider = (U56F8!(125_000_000) / (8 * baud)).to_fixed();
        cfg.shift_out.auto_fill = false;
        cfg.shift_out.direction = ShiftDirection::Right;
        cfg.fifo_join = FifoJoin::TxOnly;
        cfg.set_out_pins(&[&tx_pin]);
        cfg.set_set_pins(&[&tx_pin]);
        sm_tx.set_config(&cfg);
        sm_tx.set_enable(true)
    }
}

mod uart_rx {
    use embassy_rp::gpio::Level;
    use embassy_rp::peripherals::PIO0;
    use embassy_rp::pio::{Common, Config, Direction, FifoJoin, PioPin, ShiftDirection, StateMachine};
    use embassy_rp::relocate::RelocatedProgram;
    use fixed::traits::ToFixed;
    use fixed_macro::types::U56F8;

    pub fn setup_uart_rx_on_sm1<'a>(
        common: &mut Common<'a, PIO0>,
        sm_rx: &mut StateMachine<'a, PIO0, 1>,
        rx_pin: impl PioPin,
        baud: u64,
    ) {
        let prg = pio_proc::pio_asm!(
            r#"
                ;.program uart_rx

                ; Slightly more fleshed-out 8n1 UART receiver which handles framing errors and
                ; break conditions more gracefully.
                ; IN pin 0 and JMP pin are both mapped to the GPIO used as UART RX.

                start:
                    wait 0 pin 0        ; Stall until start bit is asserted
                    set x, 7    [10]    ; Preload bit counter, then delay until halfway through
                bitloop:                ; the first data bit (12 cycles incl wait, set).
                    in pins, 1          ; Shift data bit into ISR
                    jmp x-- bitloop [6] ; Loop 8 times, each loop iteration is 8 cycles
                    jmp pin good_stop   ; Check stop bit (should be high)

                    irq 4 rel           ; Either a framing error or a break. Set a sticky flag,
                    wait 1 pin 0        ; and wait for line to return to idle state.
                    jmp start           ; Don't push data if we didn't see good framing.

                good_stop:              ; No delay before returning to start; a little slack is
                    push                ; important in case the TX clock is slightly too fast.
            "#
        );

        let rx_pin = common.make_pio_pin(rx_pin);
        sm_rx.set_pins(Level::High, &[&rx_pin]);
        sm_rx.set_pin_dirs(Direction::In, &[&rx_pin]);

        let relocated = RelocatedProgram::new(&prg.program);
        let mut cfg = Config::default();

        cfg.use_program(&common.load_program(&relocated), &[&rx_pin]);
        cfg.clock_divider = (U56F8!(125_000_000) / (8 * baud)).to_fixed();
        cfg.shift_out.auto_fill = false;
        cfg.shift_out.direction = ShiftDirection::Right;
        cfg.fifo_join = FifoJoin::RxOnly;
        cfg.set_in_pins(&[&rx_pin]);
        cfg.set_jmp_pin(&rx_pin);
        // cfg.set_set_pins(&[&rx_pin]);
        sm_rx.set_config(&cfg);
        sm_rx.set_enable(true)
    }
}
