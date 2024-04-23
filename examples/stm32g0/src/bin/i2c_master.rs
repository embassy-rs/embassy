#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

// test is targeted for a specific board. Can simple be rewritten for a nucleo-070rb board
// by setting the uart to uart2 (see i2c_slave.rs)

use core::fmt::{self, Write};

use embassy_executor::Spawner;
use embassy_stm32::dma::NoDma;
use embassy_stm32::gpio::{Level, Output, Speed};
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
#[embassy_executor::task]
pub async fn system_ticker(mut led: Output<'static, peripherals::PA6>) {
    loop {
        Timer::after(Duration::from_millis(200)).await;
        led.set_high();
        Timer::after(Duration::from_millis(100)).await;
        led.set_low();
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    // let led = Output::new(p.PA5, Level::High, Speed::Low); // nucleog070rb board
    let led = Output::new(p.PA6, Level::High, Speed::Low);

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
    spawner.spawn(system_ticker(led)).unwrap();

    // start of the acutal test
    let mut counter = 0;
    loop {
        counter += 1;
        writeln!(&mut writer, "Loop counter: {:?}\r", counter).unwrap();

        let mut buf_20 = [0_u8; 20];
        let mut buf_20a = [0_u8; 20];
        let mut buf_64 = [0_u8; 64];
        let mut buf_65 = [0_u8; 65];
        let mut buf_2 = [0_u8; 2];

        writeln!(&mut writer, "Start of test\r").unwrap();

        for i in 0..buf_20.len() {
            buf_20[i] = 0x20 + (i as u8)
        }
        for i in 0..buf_64.len() {
            buf_64[i] = 0x40 + (i as u8)
        }
        for i in 0..buf_65.len() {
            buf_65[i] = 0x60 + (i as u8)
        }
        // 0x41 good case master write 20 bytes
        match i2c.blocking_write(0x41, &buf_20) {
            Ok(_) => writeln!(&mut writer, "Test 0x41 passed\r").unwrap(),
            Err(Error::Timeout) => writeln!(&mut writer, "Operation timed out\r").unwrap(),
            Err(err) => writeln!(&mut writer, "Test 0x41 failed. Error: {:?}\r", err).unwrap(),
        };
        Timer::after(Duration::from_millis(10)).await;

        // 0x42 edge case master write exact 64 bytes: must succeed on master and slave
        match i2c.blocking_write(0x42, &buf_64) {
            Ok(_) => writeln!(&mut writer, "Test 0x42 passed. Master write exact 64 bytes\r").unwrap(),
            Err(Error::Timeout) => writeln!(&mut writer, "Operation timed out\r").unwrap(),
            Err(err) => writeln!(&mut writer, "Test 0x42 failed. Got error: {:?}\r", err).unwrap(),
        };
        Timer::after(Duration::from_millis(10)).await;

        // 0x43 edge case master write exact 65 bytes: 1 too many must fail  on master and slave
        match i2c.blocking_write(0x43, &buf_65) {
            Ok(_) => writeln!(&mut writer, "Test passed.\r").unwrap(),
            Err(Error::Timeout) => writeln!(&mut writer, "Operation timed out\r").unwrap(),
            Err(err) => writeln!(&mut writer, "Test 0x43 failed: Error: {:?}\r", err).unwrap(),
        };
        /* skip test for now
        // 0x44 master write read combined transaction write 20 bytes, then read 20 bytes
        match i2c.blocking_write_read(0x44, &buf_20, &mut buf_20a) {
            Ok(_) => {
                writeln!(&mut writer, "Test 0x44 Ok \r").unwrap();
                writeln!(
                    &mut writer,
                    "Uppercase input should be transformed to lowercase, A -> b \r"
                )
                .unwrap();
                print_buffer(&mut writer, &buf_20a);
            }
            Err(Error::Timeout) => writeln!(&mut writer, "Operation timed out\r").unwrap(),
            Err(err) => writeln!(&mut writer, "Test 0x44 error: {:?}\r", err).unwrap(),
        };
        */
        // master read. Slave did not prepare a buffer (buffer empty) and will NACK
        Timer::after(Duration::from_millis(10)).await;
        match i2c.blocking_read(0x48, &mut buf_20) {
            Ok(_) => {
                writeln!(
                    &mut writer,
                    "Test 0x48 passed.Master cannot detect this is an error case!\r"
                )
                .unwrap();
            }
            Err(Error::Timeout) => writeln!(&mut writer, "Operation timed out\r").unwrap(),
            Err(err) => writeln!(&mut writer, "Test 0x48 failed. Error:{:?}\r", err).unwrap(),
        };
        Timer::after(Duration::from_millis(10)).await;

        // master read 20 bytes good case.
        match i2c.blocking_read(0x49, &mut buf_20) {
            Ok(_) => {
                writeln!(&mut writer, "Test 0x49 Read Ok\r").unwrap();
                print_buffer(&mut writer, &buf_20);
            }
            Err(Error::Timeout) => writeln!(&mut writer, "Operation timed out\r").unwrap(),
            Err(err) => writeln!(&mut writer, "Test 0x49 Error: {:?}\r", err).unwrap(),
        };
        Timer::after(Duration::from_millis(10)).await;

        // master read 64 bytes, but the slave did prepair only 20 Should fail with NACK
        match i2c.blocking_read(0x4A, &mut buf_64) {
            Ok(_) => {
                writeln!(&mut writer, "Test 0x4A passed. Master cannot detect this error case\r").unwrap();
                print_buffer(&mut writer, &buf_64);
            }
            Err(Error::Timeout) => writeln!(&mut writer, "Operation timed out\r").unwrap(),
            Err(err) => writeln!(&mut writer, "Test 0x4A failed.  Error: {:?}\r", err).unwrap(),
        };
        Timer::after(Duration::from_millis(10)).await;
        match i2c.blocking_read(0x4B, &mut buf_64) {
            Ok(_) => {
                writeln!(&mut writer, "Test 0x4B passed\r").unwrap();
                print_buffer(&mut writer, &buf_64);
            }
            Err(Error::Timeout) => writeln!(&mut writer, "Operation timed out\r").unwrap(),
            Err(err) => writeln!(&mut writer, "Test 0x4B failed. Error: {:?}\r", err).unwrap(),
        };
        Timer::after(Duration::from_millis(10)).await;

        // test for arbitration lost. 2 slaves respond on address 10 one with 0xFF03, on with 0xFF04
        // if both are online the one with 0xFF04 should report arbitration loss, here we read 0xFF03
        match i2c.blocking_read(0x10, &mut buf_2) {
            Ok(_) => {
                writeln!(&mut writer, "Test 0x10 Received {:2x}:{:2x}\r", buf_2[0], buf_2[1]).unwrap();
                writeln!(&mut writer, "Look in the log of the slaves to evaluate the result\r").unwrap();
            }
            Err(Error::Timeout) => writeln!(&mut writer, "Operation timed out\r").unwrap(),
            Err(err) => writeln!(&mut writer, "Test 0x10 Failed: Error: {:?}", err).unwrap(),
        };

        // 0x4F Master does read 2 bytes with the result of the slave
        let mut result: [u8; 3] = [0, 0, 0];
        match i2c.blocking_read(0x4F, &mut result) {
            Ok(_) => writeln!(
                &mut writer,
                "Test result: count {} errors {} i2c errors:{}\r",
                result[0], result[1], result[2]
            )
            .unwrap(),
            Err(Error::Timeout) => writeln!(&mut writer, "Operation timed out\r").unwrap(),
            Err(err) => writeln!(&mut writer, "Test 0x4F unexpected error: {:?}\r", err).unwrap(),
        };
        writeln!(&mut writer, "\r").unwrap();

        Timer::after(Duration::from_millis(10_000)).await;
    }
    fn print_buffer(writer: &mut SerialWriter, buf: &[u8]) {
        for i in 0..buf.len() {
            write!(writer, " {:2x} ", buf[i]).unwrap();
        }
        writeln!(writer, "\r\r").unwrap()
    }
}
