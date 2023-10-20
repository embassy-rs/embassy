#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use core::fmt::{self, Write};

use embassy_executor::Spawner;
use embassy_stm32::dma::NoDma;
use embassy_stm32::i2c::{Error, I2c};
use embassy_stm32::time::Hertz;
use embassy_stm32::usart::UartTx;
use embassy_stm32::{bind_interrupts, i2c, peripherals, usart};
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    I2C1 => i2c::InterruptHandler<peripherals::I2C1>;
    USART1 => usart::InterruptHandler<peripherals::USART1>;
});

pub struct SerialWriter {
    tx: UartTx<'static, peripherals::USART1, peripherals::DMA1_CH1>,
}
impl SerialWriter {
    pub fn new(tx: UartTx<'static, peripherals::USART1, peripherals::DMA1_CH1>) -> Self {
        SerialWriter { tx }
    }
}
impl fmt::Write for SerialWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        _ = self.tx.blocking_write(s.as_bytes());
        Ok(())
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    let uart = usart::Uart::new(
        p.USART1,
        p.PB7,
        p.PB6,
        Irqs,
        p.DMA1_CH1,
        p.DMA1_CH2,
        usart::Config::default(),
    )
    .unwrap();
    let (tx, _rx) = uart.split();

    let mut writer = SerialWriter::new(tx);

    writeln!(
        &mut writer,
        "I2c test.  Master role. Connect another device in slave role to the bus."
    )
    .unwrap();

    let mut i2c = I2c::new(
        p.I2C1,
        p.PB8,
        p.PB9,
        Irqs,
        NoDma,
        NoDma,
        Hertz(100_000),
        Default::default(),
    );
    Timer::after(Duration::from_millis(1000)).await;

    // start of the acutal test
    let mut counter = 0;
    loop {
        counter += 1;
        writeln!(&mut writer, "Loop counter: {:?}", counter).unwrap();

        let mut buf_20 = [0_u8; 20];
        let mut buf_64 = [0_u8; 64];
        let mut buf_65 = [0_u8; 65];

        writeln!(&mut writer, "Start of test").unwrap();

        for i in 0..buf_20.len() {
            buf_20[i] = 0x20 + (i as u8)
        }
        for i in 0..buf_64.len() {
            buf_64[i] = 0x40 + (i as u8)
        }
        for i in 0..buf_65.len() {
            buf_65[i] = 0x60 + (i as u8)
        }
        // test 1: slave address 0x61 should not be addressable
        match i2c.blocking_write(0x61, &buf_20) {
            Ok(_) => writeln!(&mut writer, "Test 1 Error: would expect nack").unwrap(),
            Err(Error::Timeout) => writeln!(&mut writer, "Operation timed out").unwrap(),
            Err(err) => writeln!(&mut writer, "Test 1 OK: expected NACK error: {:?}", err).unwrap(),
        };
        // 0x41 good case master write slave read: master does send 20 bytes slave receives 20 bytes
        match i2c.blocking_write(0x41, &buf_20) {
            Ok(_) => writeln!(&mut writer, "Test 0x41 Ok").unwrap(),
            Err(Error::Timeout) => writeln!(&mut writer, "Operation timed out").unwrap(),
            Err(err) => writeln!(&mut writer, "Test 0x41 Error: {:?}", err).unwrap(),
        };
        // 0x42 edge case master write exact 64 bytes: must succeed on master and slave
        match i2c.blocking_write(0x42, &buf_64) {
            Ok(_) => writeln!(&mut writer, "Test 0x42 Ok. Master write exact 64 bytes").unwrap(),
            Err(Error::Timeout) => writeln!(&mut writer, "Operation timed out").unwrap(),
            Err(err) => writeln!(&mut writer, "Test 0x42 error IncorrectFramesize: {:?}", err).unwrap(),
        };
        // 0x43 edge case master write exact 65 bytes: 1 too manyu must fail  on master and slave
        match i2c.blocking_write(0x43, &buf_64) {
            Ok(_) => writeln!(&mut writer, "Test 0x42 Failed. Expected a Nack").unwrap(),
            Err(Error::Timeout) => writeln!(&mut writer, "Operation timed out").unwrap(),
            Err(err) => writeln!(
                &mut writer,
                "Test 0x43 Ok: Got error NACK du to buffer of 1 too big {:?}",
                err
            )
            .unwrap(),
        };

        match i2c.blocking_read(0x48, &mut buf_20) {
            Ok(_) => {
                writeln!(&mut writer, "Test 0x48 Read expected to fail!").unwrap();
            }
            Err(Error::Timeout) => writeln!(&mut writer, "Operation timed out").unwrap(),
            Err(err) => writeln!(
                &mut writer,
                "Test 0x48 Ok. First time, slave did not yet prepare a buffer Error: {:?}",
                err
            )
            .unwrap(),
        };
        match i2c.blocking_read(0x49, &mut buf_20) {
            Ok(_) => {
                writeln!(&mut writer, "Test 0x49 Read Ok").unwrap();
                print_buffer(&mut writer, &buf_20);
            }
            Err(Error::Timeout) => writeln!(&mut writer, "Operation timed out").unwrap(),
            Err(err) => writeln!(&mut writer, "Test 0x49 Error: {:?}", err).unwrap(),
        };
        match i2c.blocking_read(0x4A, &mut buf_64) {
            Ok(_) => {
                writeln!(&mut writer, "Test 0x4A failed. Expected was a NACK error").unwrap();
                print_buffer(&mut writer, &buf_64);
            }
            Err(Error::Timeout) => writeln!(&mut writer, "Operation timed out").unwrap(),
            Err(err) => writeln!(&mut writer, "Test 0x4A Ok. Expected to fail. Error: {:?}", err).unwrap(),
        };
        match i2c.blocking_read(0x4B, &mut buf_64) {
            Ok(_) => {
                writeln!(&mut writer, "Test 0x4B succeeded: edge case read exact the buffer size").unwrap();
                print_buffer(&mut writer, &buf_64);
            }
            Err(Error::Timeout) => writeln!(&mut writer, "Operation timed out").unwrap(),
            Err(err) => writeln!(&mut writer, "Test 0x4B failed. Error: {:?}", err).unwrap(),
        };
        /*
        match i2c.blocking_write_read(0x44, &buf_20, &mut buf_64) {
            Ok(_) => {
                writeln!(&mut writer, "Test 0x44 Ok ").unwrap();
                writeln!(
                    &mut writer,
                    "Uppercase input should be transformed to lowercase, A -> b "
                )
                .unwrap();

                for i in 0..buf_rcv.len() {
                    writeln!(&mut writer, "{}", buf_rcv[i]).unwrap();
                }
                writeln!(&mut writer, "").unwrap()
            }
            Err(Error::Timeout) => writeln!(&mut writer, "Operation timed out").unwrap(),
            Err(err) => writeln!(&mut writer, "Test 0x44 error: {:?}", err).unwrap(),
        };
        */
        // 0x4F test end and slave will present results
        let mut result: [u8; 2] = [0, 0];
        match i2c.blocking_read(0x4F, &mut result) {
            Ok(_) => writeln!(
                &mut writer,
                "Result of the whole test as reported by the slave count/errors: {}/{}\r",
                result[0], result[1]
            )
            .unwrap(),
            Err(Error::Timeout) => writeln!(&mut writer, "Operation timed out").unwrap(),
            Err(err) => writeln!(&mut writer, "Test 0x4F unexpected error: {:?}", err).unwrap(),
        };
        writeln!(&mut writer, "").unwrap();
        Timer::after(Duration::from_millis(10_000)).await;
    }
    fn print_buffer(writer: &mut SerialWriter, buf: &[u8]) {
        for i in 0..buf.len() {
            write!(writer, " {:2x} ", buf[i]).unwrap();
        }
        writeln!(writer, "\n\r").unwrap()
    }
}
