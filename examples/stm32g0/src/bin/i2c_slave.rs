#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

// test is targeted for nucleo-g070RB board

use core::fmt::{self, Write};

use embassy_executor::Spawner;
use embassy_stm32::dma::NoDma;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::i2c::{Address2Mask, Dir, I2c};
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
            Dir::WRITE => (),
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
            Dir::READ => (),
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
    config.slave_address_2(0x41, Address2Mask::MASK4);

    let i2c = I2c::new(p.I2C1, p.PB8, p.PB9, Irqs, NoDma, NoDma, Hertz(100_000), config);

    let mut buf_64 = [0; 64]; // buffer is longer than master will send: wait for STOP condition
    let mut buf_20 = [0; 20]; // buffer is shorter than master will send: wait for STOP condition
    let mut buf_20a = [0; 20];
    let mut buf_2 = [0; 2];
    let mut errors = 0;
    let mut tcount = 0;
    let mut counter = 0;

    spawner.spawn(system_ticker(led)).unwrap();

    // start of the actual test
    i2c.slave_start_listen().unwrap();
    loop {
        counter += 1;
        writeln!(&mut writer, "Loop: {}\r", counter).unwrap();

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
        _ = i2c.slave_prepare_read(&mut buf_2, i2c::AddressIndex::Address1);

        writeln!(&mut writer, "Waiting for master activity\r").unwrap();

        let t = i2c.slave_transaction().await;
        let dir = t.dir();
        tcount += 1;
        // preparations for the next round
        match t.address() {
            0x43 => {
                /*
                // prepare for test 0x44: master write read. 20a is send to the master
                for i in 0..buf_20a.len() {
                    buf_20a[i] = 0x44 + (i as u8)
                }
                _ = i2c.slave_prepare_read(&mut buf_20a, i2c::AddressIndex::Address2);
                i2c.slave_sbc(false);
                */
            }
            0x48 => {
                // prepare buffer for test 0x49
                for i in 0..buf_20.len() {
                    buf_20[i] = 0x49 + (i as u8)
                }
                _ = i2c.slave_prepare_read(&buf_20, i2c::AddressIndex::Address2);
            }
            0x49 => {
                // prepare buffer for test 0x4A
                for i in 0..buf_20.len() {
                    buf_20[i] = 0x4A + (i as u8)
                }
                _ = i2c.slave_prepare_read(&buf_20, i2c::AddressIndex::Address2);
            }
            0x4A => {
                // prepare buffer for test 0x4B
                for i in 0..buf_64.len() {
                    buf_64[i] = 0x4B + (i as u8)
                }
                match i2c.slave_prepare_read(&buf_64, i2c::AddressIndex::Address2) {
                    Err(_) => writeln!(&mut writer, "Buffer error\r").unwrap(),
                    _ => (),
                }
            }
            0x4B => {
                // prepare for test 0x4F
                let err = i2c.slave_error_count() as u8;
                let result: [u8; 3] = [tcount, errors, err];
                _ = i2c.slave_prepare_read(&result, i2c::AddressIndex::Address2);
                writeln!(
                    &mut writer,
                    "Count: {} test errors {}   i2  errors: {}\r",
                    tcount, errors, err
                )
                .unwrap();
            }
            _ => (),
        }
        // printing does cost a lot of time, and can interfere with the test if too verbose
        writeln!(
            &mut writer,
            "A:x{:2x} d:{:?} s:{:2x}, act:{:2x} r:{:?}\r",
            t.address(),
            dir as u8,
            t.size(),
            t.index(),
            t.result()
        )
        .unwrap();

        match t.address() {
            0x41 => {
                //Evaluate test 0x41: Good case master write 20 bytes
                checkIsWrite!(writer, dir);
                match t.result() {
                    None => {
                        writeln!(&mut writer, "Test 0x41 passed\r").unwrap();
                        print_buffer(&mut writer, t.buffer());
                    }
                    Some(err) => {
                        errors += 1;
                        writeln!(&mut writer, "Test 0x41 failed. Error: {:?}\r", err).unwrap()
                    }
                };
            }
            0x42 => {
                //Evaluate test 0x42: edge case master write exact 64 bytes: must succeed on master and slave
                checkIsWrite!(writer, dir);
                match t.result() {
                    None => {
                        writeln!(&mut writer, "Test 0x42 passed. send 64 bytes\r").unwrap();
                        print_buffer(&mut writer, t.buffer());
                    }
                    Some(err) => {
                        errors += 1;
                        writeln!(&mut writer, "Test 0x42 failed. Error: {:?}\r", err).unwrap()
                    }
                };
            }
            0x43 => {
                //"Evaluate test 0x43.edge case master write exact 65 bytes: 1 too many must fail on master and slave
                checkIsWrite!(writer, dir);
                match t.result() {
                    None => {
                        errors += 1;
                        writeln!(&mut writer, "Test 0x43 failed. Expected Overrun error Got Ok\r").unwrap()
                    }
                    Some(err) => writeln!(
                        &mut writer,
                        "Test 0x43 passed. Expected error: Overrun. Error: {:?}\r",
                        err
                    )
                    .unwrap(),
                };
            }
            0x44 => {
                // Evaluate test 0x44: master write read combined transaction write 20 bytes, then read 20 bytes
                checkIsRead!(writer, dir);
                match t.result() {
                    None => {
                        writeln!(&mut writer, "Test 0x44 passed\r").unwrap();
                        print_buffer(&mut writer, t.buffer())
                    }
                    Some(err) => {
                        errors += 1;
                        writeln!(&mut writer, "Test 0x44 failed. Error:{:?}\r", err).unwrap()
                    }
                };
            }
            0x48 => {
                // 0x48 master read slave write slave did not yet prepare a buffer, master will fail
                checkIsRead!(writer, dir);
                match t.result() {
                    None => {
                        writeln!(&mut writer, "Test 0x48 failed. Expected to fail: \r").unwrap();
                        errors += 1;
                    }
                    Some(err) => writeln!(&mut writer, "Test 0x48 passed. Got expected error: {:?}\r", err).unwrap(),
                };
            }
            0x49 => {
                //"Evaluate test 0x49.  master read 20 bytes good case.
                checkIsRead!(writer, dir);
                match t.result() {
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
                checkIsRead!(writer, dir);
                match t.result() {
                    None => {
                        writeln!(&mut writer, "Test 0x4A Passed.").unwrap();
                    }
                    Some(err) => {
                        errors += 1;
                        writeln!(&mut writer, "Test failed. Error: {:?}\r", err).unwrap()
                    }
                }
            }
            0x4B => {
                // Master-read-slave-write Master expects 64 bytes, Should be ok
                checkIsRead!(writer, dir);
                match t.result() {
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
                match t.result() {
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
                writeln!(&mut writer, "------------------\r").unwrap();
                tcount = 0;
                errors = 0;
            }
            0x4 => {
                // Arbitration lost test Master does read 2 bytes on t.address() 0x10
                // this slave will send 0xFF04, the other slave will send 0xFF03
                // This slave should generate a arbitration lost if the other slave is online
                writeln!(&mut writer, "Evaluate test 0x10: slave arbitration lost.\r").unwrap();
                checkIsRead!(writer, dir);
                match t.result() {
                    None => {
                        writeln!(&mut writer, "Test 0x10 should fail if a second slave with testcase i2c_salve_arbitration.rs is connected.\r").unwrap();
                        errors += 1;
                    }
                    Some(err) => writeln!(&mut writer, "Test 0x10 passed. Error: {:?}\r", err).unwrap(),
                };
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
