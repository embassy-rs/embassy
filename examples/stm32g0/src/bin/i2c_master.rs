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

        let mut buf_long: [u8; 50] = [0; 50];
        let mut buf_short: [u8; 20] = [0; 20];
        let mut buf_rcv: [u8; 20] = [0; 20];
        let mut buf_10: [u8; 10] = [0; 10];
        let mut buf_io = [0_u8; 1];

        writeln!(&mut writer, "Start of test").unwrap();

        for i in 0..buf_short.len() {
            buf_short[i] = 0x41 + (i as u8)
        }

        // test 1: slave address 0x61 should not be addressable
        match i2c.blocking_write(0x61, &buf_short) {
            Ok(_) => writeln!(&mut writer, "Test 1 Error: would expect nack").unwrap(),
            Err(Error::Timeout) => writeln!(&mut writer, "Operation timed out").unwrap(),
            Err(err) => writeln!(&mut writer, "Test 1 OK: expected NACK error: {:?}", err).unwrap(),
        };
        // 0x41 good case master write slave read: master does send 20 bytes slave receives 20 bytes
        match i2c.blocking_write(0x41, &buf_short) {
            Ok(_) => writeln!(&mut writer, "Test 0x41 Ok").unwrap(),
            Err(Error::Timeout) => writeln!(&mut writer, "Operation timed out").unwrap(),
            Err(err) => writeln!(&mut writer, "Test 0x41 Error: {:?}", err).unwrap(),
        };
        // 0x42 bad case master write slave read: master does send less than 20 bytes
        for i in 0..buf_10.len() {
            buf_10[i] = 0x20 + (i as u8)
        }
        match i2c.blocking_write(0x42, &buf_10) {
            Ok(_) => writeln!(
                &mut writer,
                "Test 0x42 Ok. (Master cannot detect that frame is too short)  "
            )
            .unwrap(),
            Err(Error::Timeout) => writeln!(&mut writer, "Operation timed out").unwrap(),
            Err(err) => writeln!(&mut writer, "Test 0x42 error IncorrectFramesize: {:?}", err).unwrap(),
        };
        // 0x43 bad case master write slave read: master does send more than 20 bytes, slave does NACK
        for i in 0..buf_long.len() {
            buf_long[i] = 0x61 + (i as u8)
        }
        match i2c.blocking_write(0x43, &buf_long) {
            Ok(_) => writeln!(&mut writer, "Test 0x43 not ok expected error IncorrectFramesize: ").unwrap(),
            Err(Error::Timeout) => writeln!(&mut writer, "Operation timed out").unwrap(),
            Err(err) => writeln!(&mut writer, "Test 0x43 Ok Expected IncorrectFrameSize: {:?}", err).unwrap(),
        };
        // 0x44 master write_read good case: master sends and expects 20 bytes
        for i in 0..buf_short.len() {
            buf_short[i] = 0x41 + (i as u8)
        }
        for i in 0..buf_rcv.len() {
            buf_rcv[i] = 0x30 + (i as u8)
        }
        match i2c.blocking_write_read(0x44, &buf_short, &mut buf_rcv) {
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
        // 0x48 master read slave write good case: exact 20 characters
        for i in 0..buf_short.len() {
            buf_short[i] = 0x61 + (i as u8)
        }
        match i2c.blocking_read(0x48, &mut buf_short) {
            Ok(_) => {
                writeln!(&mut writer, "Test 0x48 Ok ").unwrap();
                for i in 0..buf_short.len() {
                    writeln!(&mut writer, "{}", buf_short[i]).unwrap();
                }
                writeln!(&mut writer, "").unwrap()
            }
            Err(Error::Timeout) => writeln!(&mut writer, "Operation timed out").unwrap(),
            Err(err) => writeln!(&mut writer, "Test 0x48 unexpected error: {:?}", err).unwrap(),
        };
        // 0x49 master read slave write bad  case: master expects 50 slave does send 20 characters
        for i in 0..buf_long.len() {
            buf_long[i] = 0x61 + (i as u8)
        }
        match i2c.blocking_read(0x49, &mut buf_long) {
            Ok(_) => {
                writeln!(&mut writer, "Test 0x49 Ok ").unwrap();
                for i in 0..buf_long.len() {
                    writeln!(&mut writer, "{}", buf_long[i]).unwrap();
                }
                writeln!(&mut writer, "").unwrap()
            }
            Err(Error::Timeout) => writeln!(&mut writer, "Operation timed out").unwrap(),
            Err(err) => writeln!(&mut writer, "Test 0x49 error: {:?}", err).unwrap(),
        };
        // 0x4A master read slave write bad  case: master expects 20 does slave does send 50 characters
        for i in 0..buf_short.len() {
            buf_short[i] = 0x41 + (i as u8)
        }
        match i2c.blocking_read(0x4A, &mut buf_short) {
            Ok(_) => {
                writeln!(&mut writer, "Test 0x4A Ok ").unwrap();

                for i in 0..buf_short.len() {
                    writeln!(&mut writer, "{}", buf_short[i]).unwrap();
                }
                writeln!(&mut writer, "").unwrap()
            }
            Err(Error::Timeout) => writeln!(&mut writer, "Operation timed out").unwrap(),
            Err(err) => writeln!(&mut writer, "Test 0x4A error: {:?}", err).unwrap(),
        };
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
        let mut joyb = [0_u8; 3];
        match i2c.blocking_read(0x52, &mut joyb) {
            Ok(_) => {
                writeln!(&mut writer, "Test 0x52 Ok ").unwrap();
                for i in 0..joyb.len() {
                    writeln!(&mut writer, " {} ", joyb[i]).unwrap();
                }
                writeln!(&mut writer, "").unwrap()
            }
            Err(Error::Timeout) => writeln!(&mut writer, "Operation timed out").unwrap(),
            Err(err) => writeln!(&mut writer, "Test 0x52 unexpected error: {:?}", err).unwrap(),
        };
        match i2c.blocking_write(0x21, &mut buf_io) {
            Ok(_) => (),
            Err(Error::Timeout) => writeln!(&mut writer, "Operation timed out").unwrap(),
            Err(err) => writeln!(&mut writer, "Test 0x21 unexpected error: {:?}", err).unwrap(),
        };
        buf_io[0] += 1;
        Timer::after(Duration::from_millis(10_000)).await;
    }
}
