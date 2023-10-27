#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

// test is targeted for nucleo-g070RB board

use core::fmt::{self, Write};

use embassy_executor::Spawner;
use embassy_stm32::dma::NoDma;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::i2c::{Error, I2c};
use embassy_stm32::pac::i2c::vals;
use embassy_stm32::time::Hertz;
use embassy_stm32::usart::UartTx;
use embassy_stm32::{bind_interrupts, i2c, peripherals, usart};
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    I2C1 => i2c::InterruptHandler<peripherals::I2C1>;
    USART2 => usart::InterruptHandler<peripherals::USART2>;
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

#[embassy_executor::task]
pub async fn system_ticker(mut led: Output<'static, peripherals::PA5>) {
    loop {
        Timer::after(Duration::from_millis(200)).await;
        led.set_high();
        Timer::after(Duration::from_millis(100)).await;
        led.set_low();
    }
}
pub struct SerialWriter {
    tx: UartTx<'static, peripherals::USART2, peripherals::DMA1_CH1>,
}
impl SerialWriter {
    pub fn new(tx: UartTx<'static, peripherals::USART2, peripherals::DMA1_CH1>) -> Self {
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
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    let led = Output::new(p.PA5, Level::High, Speed::Low);

    /*
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
    */
    let uart = usart::Uart::new(
        p.USART2,
        p.PA3,
        p.PA2,
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
    config.slave_address_7bits(0x10); // for arbitration lost test
    config.slave_address_2(0x41, vals::Oamsk::MASK4);

    let i2c = I2c::new(p.I2C1, p.PB8, p.PB9, Irqs, NoDma, NoDma, Hertz(100_000), config);

    let mut buf_64 = [0; 64]; // buffer is longer than master will send: wait for STOP condition
    let mut buf_20 = [0; 20]; // buffer is shorter than master will send: wait for STOP condition
    let mut buf_20a = [0; 20];
    let mut buf_2 = [0; 2];
    let mut errors = 0;
    let mut address = 0;
    let mut dir = vals::Dir::READ;
    let mut tcount = 0;
    let mut counter = 0;
    let mut result: Option<Error> = None;

    spawner.spawn(system_ticker(led)).unwrap();

    // start of the actual test
    i2c.slave_start_listen().unwrap();
    loop {
        counter += 1;
        writeln!(&mut writer, "Loop: {}\r", counter).unwrap();

        // clear master write buffers for sure
        _ = i2c.slave_read_buffer(&mut buf_64, i2c::AddressType::Address2);
        _ = i2c.slave_read_buffer(&mut buf_64, i2c::AddressType::Address1);

        for i in 0..buf_20.len() {
            buf_20[i] = 0x20 + (i as u8)
        }
        for i in 0..buf_20a.len() {
            buf_20a[i] = 0x40 - (i as u8)
        }
        for i in 0..buf_64.len() {
            buf_64[i] = 0x60 + (i as u8)
        }
        // content for test 0x10
        buf_2[0] = 0xFF;
        buf_2[1] = 0x04;
        _ = i2c.slave_write_buffer(&mut buf_2, i2c::AddressType::Address1);

        writeln!(&mut writer, "Waiting for master activity\r").unwrap();

        let (address, dir, size, result) = i2c.slave_transaction().await;
        writeln!(
            &mut writer,
            "Address: x{:2x}  dir: {:?} size: x{:2x}, Result:{:?}\r",
            address, dir as u8, size, result
        )
        .unwrap();
        tcount += 1;
        // preparations for the next round
        match address {
            0x42 => {
                // prepare for test 0x44: master write read. 20a is send to the master
                _ = i2c.slave_write_buffer(&mut buf_20a, i2c::AddressType::Address2);
                i2c.slave_sbc(false);
            }
            0x43 => {
                // prepare for test 0x44: master write read. 20a is send to the master
                _ = i2c.slave_write_buffer(&mut buf_20a, i2c::AddressType::Address2);
                i2c.slave_sbc(false);
            }
            0x44 => {
                // prepare for test 0x48: slave does have no buffer to read
                i2c.slave_reset_buffer(i2c::AddressType::Address2);
            }
            0x48 => {
                // 0x48 master read slave write slave did not yet prepare a buffer, master will fail
                // prepare buffer for test 0x49
                for i in 0..buf_20.len() {
                    buf_20[i] = 0x48 + (i as u8)
                }
                _ = i2c.slave_write_buffer(&buf_20, i2c::AddressType::Address2);
            }
            0x49 => {
                // prepare buffer for test 0x4A
                for i in 0..buf_20.len() {
                    buf_20[i] = 0x49 + (i as u8)
                }
                match i2c.slave_write_buffer(&buf_20, i2c::AddressType::Address2) {
                    Err(_) => writeln!(&mut writer, "Buffer error\r").unwrap(),
                    _ => (),
                }
            }
            0x4A => {
                // prepare buffer for test 0x4B
                for i in 0..buf_64.len() {
                    buf_64[i] = 0x4A + (i as u8)
                }
                match i2c.slave_write_buffer(&buf_64, i2c::AddressType::Address2) {
                    Err(_) => writeln!(&mut writer, "Buffer error\r").unwrap(),
                    _ => (),
                }
            }
            0x4B => {
                // prepare for test 0x4F
                let result: [u8; 2] = [tcount, errors];
                _ = i2c.slave_write_buffer(&result, i2c::AddressType::Address2);
            }
            _ => (),
        }

        match address {
            0x41 => {
                writeln!(&mut writer, "Evaluate test 0x41: Good case master write 20 bytes\r").unwrap();
                checkIsWrite!(writer, dir);
                _ = i2c.slave_read_buffer(&mut buf_20, i2c::AddressType::Address2);
                match result {
                    None => {
                        writeln!(&mut writer, "Test 0x41 passed\r").unwrap();
                        print_buffer(&mut writer, &buf_20);
                    }
                    Some(err) => {
                        errors += 1;
                        writeln!(&mut writer, "Test 0x41 failed. Error: {:?}\r", err).unwrap()
                    }
                };
            }
            0x42 => {
                writeln!(
                    &mut writer,
                    "Evaluate test 0x42: edge case master write exact 64 bytes: must succeed on master and slave\r"
                )
                .unwrap();
                checkIsWrite!(writer, dir);
                _ = i2c.slave_read_buffer(&mut buf_64, i2c::AddressType::Address2);
                match result {
                    None => {
                        writeln!(&mut writer, "Test 0x42 passed. send 64 bytes\r").unwrap();
                        print_buffer(&mut writer, &buf_64);
                    }
                    Some(err) => {
                        errors += 1;
                        writeln!(&mut writer, "Test 0x42 failed. Error: {:?}\r", err).unwrap()
                    }
                };
            }
            0x43 => {
                writeln!(&mut writer, "Evaluate test 0x43.edge case master write exact 65 bytes: 1 too many must fail on master and slave\r").unwrap();
                checkIsWrite!(writer, dir);
                _ = i2c.slave_read_buffer(&mut buf_64, i2c::AddressType::Address2);
                match result {
                    None => {
                        errors += 1;
                        writeln!(&mut writer, "Test 0x43 failed. Expected BufferFull error Got Ok\r").unwrap()
                    }
                    Some(err) => writeln!(
                        &mut writer,
                        "Test 0x43 passed. Expected error: BufferFull. Error: {:?}\r",
                        err
                    )
                    .unwrap(),
                };
            }
            0x44 => {
                writeln!(
                    &mut writer,
                    "Evaluate test 0x44: master write read combined transaction write 20 bytes, then read 20 bytes \r"
                )
                .unwrap();
                checkIsRead!(writer, dir);
                _ = i2c.slave_read_buffer(&mut buf_20, i2c::AddressType::Address2);
                match result {
                    None => {
                        writeln!(&mut writer, "Test 0x44 passed\r").unwrap();
                        print_buffer(&mut writer, &buf_20)
                    }
                    Some(err) => {
                        errors += 1;
                        writeln!(&mut writer, "Test 0x44 failed. Error:{:?}\r", err).unwrap()
                    }
                };
            }
            0x48 => {
                // 0x48 master read slave write slave did not yet prepare a buffer, master will fail
                writeln!(
                    &mut writer,
                    "Evaluate test 0x48. master read. Slave did not prepare a buffer (buffer empty) and will NACK\r"
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
            }
            0x49 => {
                writeln!(&mut writer, "Evaluate test 0x49.  master read 20 bytes good case.\r").unwrap();
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
            }
            0x4A => {
                // 0x4A master read slave write bad  case: master expects 64 does slave does prepare 20 characters
                writeln!(&mut writer, "Evaluate test 0x4A.master read 64 bytes, but the slave did prepair only 20 Should fail with NACK\r").unwrap();
                checkIsRead!(writer, dir);
                match result {
                    None => {
                        errors += 1;
                        writeln!(&mut writer, "Test 0x4A failed . Expected an error").unwrap();
                    }
                    Some(err) => writeln!(&mut writer, "Test 0x4A passed. Expected error: {:?}\r", err).unwrap(),
                }
            }
            0x4B => {
                // Master-read-slave-write Master expects 64 bytes, Should be ok
                writeln!(&mut writer, "Evaluate test 0x4B. Master read 64 bytes good case.\r").unwrap();
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
            }
            0x4F => {
                // Master-read-slave-write 2 bytes with test summary Should be ok.
                checkIsRead!(writer, dir);
                writeln!(&mut writer, "Evaluate test 0x4F. Send test summary\r").unwrap();
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
            0x10 => {
                // Arbitration lost test Master does read 2 bytes on address 0x10
                // this slave will send 0xFF04, the other slave will send 0xFF03
                // This slave should generate a arbitration lost if the other slave is online
                writeln!(&mut writer, "Evaluate test 0x10: slave arbitration lost.\r").unwrap();
                checkIsRead!(writer, dir);
                match result {
                    None => {
                        writeln!(&mut writer, "Test 0x10 should fail if a second slave with testcase i2c_salve_arbitration.rs is connected.\r").unwrap();
                        errors += 1;
                    }
                    Some(err) => writeln!(&mut writer, "Test 0x10 passed. Error: {:?}\r", err).unwrap(),
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
        writeln!(writer, "\r").unwrap()
    }
}
