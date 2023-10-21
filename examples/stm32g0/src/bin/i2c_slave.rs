#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use core::fmt::{self, Write};

use embassy_executor::Spawner;
use embassy_stm32::dma::NoDma;
use embassy_stm32::i2c::{Error, I2c};
use embassy_stm32::pac::i2c::vals;
use embassy_stm32::time::Hertz;
use embassy_stm32::usart::UartTx;
use embassy_stm32::{bind_interrupts, i2c, peripherals, usart};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    I2C1 => i2c::InterruptHandler<peripherals::I2C1>;
    USART1 => usart::InterruptHandler<peripherals::USART1>;
});

macro_rules! checkIsWrite {
    ($writer:ident, $direction:ident) => {
        match $direction {
            vals::Dir::WRITE => (),
            _ => {
                write!($writer, "Error incorrect direction {:?}\r", $direction as usize).unwrap();
                continue;
            }
        }
    };
}
macro_rules! checkIsRead {
    ($writer:ident, $direction:ident) => {
        match $direction {
            vals::Dir::READ => (),
            _ => {
                write!($writer, "Error incorrect direction {:?}\r", $direction as usize).unwrap();
                continue;
            }
        }
    };
}

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
        "async i2c slave test. Should be used together with i2c_master test\r"
    )
    .unwrap();

    let mut config = i2c::Config::default();
    config.slave_address_7bits(0x4);
    config.slave_address_2(0x41, vals::Oamsk::MASK4);
    writeln!(&mut writer, "After config:\r",).unwrap();

    let mut i2c = I2c::new(p.I2C1, p.PB8, p.PB9, Irqs, NoDma, NoDma, Hertz(100_000), config);
    writeln!(&mut writer, "After i2c init\r").unwrap();

    let mut buf_64 = [0; 64]; // buffer is longer than master will send: wait for STOP condition
    let mut buf_20 = [0; 20]; // buffer is shorter than master will send: wait for STOP condition
    let mut errors = 0;
    let mut address = 0;
    let mut dir = vals::Dir::READ;
    let mut tcount = 0;
    let mut counter = 0;
    let mut result: Option<Error> = None;

    // start of the actual test
    i2c.slave_start_listen().unwrap();
    loop {
        counter += 1;
        writeln!(&mut writer, "Loop: {}\r", counter).unwrap();

        for i in 0..buf_20.len() {
            buf_20[i] = 0x61 + (i as u8)
        }
        for i in 0..buf_64.len() {
            buf_64[i] = 0x41 + (i as u8)
        }

        writeln!(&mut writer, "Waiting for master activity\r").unwrap();

        // clear write buffers for sure
        _ = i2c.slave_read_buffer(&mut buf_64, i2c::AddressType::GenericAddress);
        _ = i2c.slave_read_buffer(&mut buf_64, i2c::AddressType::MainAddress);

        let (address, dir, size, result) = i2c.slave_transaction().await;
        writeln!(
            &mut writer,
            "Address: x{:2x}  dir: {:?} size: x{:2x}, Result:{:?}\r",
            address, dir as u8, size, result
        )
        .unwrap();
        tcount += 1;

        match address {
            0x41 => {
                writeln!(
                    &mut writer,
                    "Start test 0x41. Master-write-slave-read 20 bytes. Should be ok\r\n"
                )
                .unwrap();
                checkIsWrite!(writer, dir);
                _ = i2c.slave_read_buffer(&mut buf_20, i2c::AddressType::GenericAddress);
                match result {
                    None => {
                        writeln!(&mut writer, "Test 0x41 passed\r\n").unwrap();
                        print_buffer(&mut writer, &buf_20);
                    }
                    Some(err) => {
                        errors += 1;
                        writeln!(&mut writer, "Test 0x41 failed. Error: {:?}\r\n", err).unwrap()
                    }
                };
            }
            0x42 => {
                writeln!(
                    &mut writer,
                    "Start test 0x42. Master-write-slave-read 64 bytes. Should be ok\r\n"
                )
                .unwrap();
                checkIsWrite!(writer, dir);
                _ = i2c.slave_read_buffer(&mut buf_64, i2c::AddressType::GenericAddress);
                match result {
                    None => {
                        writeln!(&mut writer, "Test 0x42 passed. send 64 bytes").unwrap();
                        print_buffer(&mut writer, &buf_64);
                    }
                    Some(err) => {
                        errors += 1;
                        writeln!(&mut writer, "Test 0x42 failed. Error: {:?}\r", err).unwrap()
                    }
                };
            }
            0x43 => {
                writeln!(
                    &mut writer,
                    "Start test 0x43. Master-write-slave-read 65 bytes. Slave should NACK at 65 byte\r\n"
                )
                .unwrap();
                checkIsWrite!(writer, dir);
                _ = i2c.slave_read_buffer(&mut buf_64, i2c::AddressType::GenericAddress);
                match result {
                    None => {
                        {
                            writeln!(&mut writer, "Test 0x43 failed. Expected to fail Got Ok\r").unwrap()
                        };
                    }
                    Some(err) => {
                        errors += 1;
                        writeln!(&mut writer, "Test 0x43 passed. Get expected error. Error: {:?}\r", err).unwrap()
                    }
                };
            }
            0x48 => {
                // 0x48 master read slave write slave did not yet prepare a buffer, master will fail
                writeln!(
                    &mut writer,
                    "Start test 0x48. Master-read-slave-write 20 bytes. Should first time (buffer empty)\r\n"
                )
                .unwrap();
                checkIsRead!(writer, dir);
                match result {
                    None => {
                        writeln!(&mut writer, "Test 0x48 failed. Expected to fail: \r").unwrap();
                        errors += 1;
                    }
                    Some(err) => writeln!(&mut writer, "Test 0x48 passed. Got expected error: {:?}\r", err).unwrap(),
                };
                // prepare buffer for next round
                for i in 0..buf_20.len() {
                    buf_20[i] = 0x61 + (i as u8)
                }
                _ = i2c.slave_write_buffer(&buf_20, i2c::AddressType::GenericAddress);
            }
            0x49 => {
                writeln!(
                    &mut writer,
                    "Start test 0x49. Master-read-slave-write 20 bytes. Should be ok\r\n"
                )
                .unwrap();
                checkIsRead!(writer, dir);
                match result {
                    None => {
                        writeln!(&mut writer, "Test passed").unwrap();
                    }
                    Some(err) => {
                        errors += 1;
                        writeln!(&mut writer, "Test 0x49 failed. Error: {:?}\r", err).unwrap()
                    }
                };
                // prepare buffer for next round
                for i in 0..buf_20.len() {
                    buf_20[i] = 0x41 + (i as u8)
                }
                _ = i2c.slave_write_buffer(&buf_20, i2c::AddressType::GenericAddress);
            }
            0x4A => {
                // 0x4A master read slave write bad  case: master expects 64 does slave does prepare 20 characters
                writeln!(
                    &mut writer,
                    "Start test 0x4A. Master-read-slave-write Master expects 64 bytes, slave sends 20.\r\n"
                )
                .unwrap();
                checkIsRead!(writer, dir);
                match result {
                    None => {
                        writeln!(&mut writer, "Test 0x4A failed . Expected an error").unwrap();
                    }
                    Some(err) => {
                        errors += 1;
                        writeln!(&mut writer, "Test 0x4A passed. Expected errore rror: {:?}\r", err).unwrap()
                    }
                }
                _ = i2c.slave_write_buffer(&buf_20, i2c::AddressType::GenericAddress);
                // prepare buffer for next round
                for i in 0..buf_64.len() {
                    buf_64[i] = 0x41 + (i as u8)
                }
                _ = i2c.slave_write_buffer(&buf_64, i2c::AddressType::GenericAddress);
            }
            0x4B => {
                writeln!(
                    &mut writer,
                    "Start test 0x4B. Master-read-slave-write Master expects 64 bytes, Should be ok.\r\n"
                )
                .unwrap();
                checkIsRead!(writer, dir);
                match result {
                    None => {
                        writeln!(&mut writer, "Test 0x4B passed\r").unwrap();
                    }
                    Some(err) => {
                        errors += 1;
                        writeln!(&mut writer, "Test 0x4B failed. Error: {:?}\r", err).unwrap()
                    }
                };
                let result: [u8; 2] = [tcount, errors];
                _ = i2c.slave_write_buffer(&result, i2c::AddressType::GenericAddress);
            }
            0x4F => {
                checkIsRead!(writer, dir);
                writeln!(
                    &mut writer,
                    "Start test 0x4B. Master-read-slave-write 2 bytes with test summary Should be ok.\r\n"
                )
                .unwrap();
                match result {
                    None => {
                        writeln!(&mut writer, "Test 0x4F Result send to master\r").unwrap();
                    }
                    Some(err) => {
                        errors += 1;
                        writeln!(&mut writer, "Test 0x4F failed. Error: {:?}\r", err).unwrap()
                    }
                };
                writeln!(
                    &mut writer,
                    "Test finished. nr tests/nr errors: {}/{}!\r",
                    tcount, errors
                )
                .unwrap();
                writeln!(&mut writer, "-----\r").unwrap();
                tcount = 0;
                errors = 0;
            }
            _ => (),
        }
    }
    fn print_buffer(writer: &mut SerialWriter, buf: &[u8]) {
        for i in 0..buf.len() {
            write!(writer, " {:2x} ", buf[i]).unwrap();
        }
        writeln!(writer, "\n\r").unwrap()
    }
}
