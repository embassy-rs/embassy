#![no_std]
#![no_main]

// test is targeted for nucleo-g070RB board

use core::fmt;
use core::fmt::Write;
use core::str;
use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::i2c::{Address2Mask, Dir, I2c};
use embassy_stm32::time::Hertz;
use embassy_stm32::{bind_interrupts, i2c, mode, peripherals};
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    I2C1 => i2c::EventInterruptHandler<peripherals::I2C1>, i2c::ErrorInterruptHandler<peripherals::I2C1>;
});

macro_rules! checkIsWrite {
    ($direction:ident) => {
        match $direction {
            Dir::Write => (),
            _ => {
                info!("Error incorrect direction {:?}\r", $direction as usize);
                continue;
            }
        }
    };
}
macro_rules! checkIsRead {
    ($direction:ident) => {
        match $direction {
            Dir::Read => (),
            _ => {
                info!("Error incorrect direction {:?}\r", $direction as usize);
                continue;
            }
        }
    };
}

#[embassy_executor::task]
pub async fn system_ticker(mut led: Output<'static>) {
    loop {
        info!("Alive!");
        Timer::after(Duration::from_millis(5000)).await;
        led.set_high();
        Timer::after(Duration::from_millis(100)).await;
        led.set_low();
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    let led = Output::new(p.PA5, Level::High, Speed::Low);

    let mut sw = StringWriter::new();

    info!("async i2c slave test. Should be used together with i2c_master test\r");

    let mut config = i2c::Config::default();
    config.slave_address_7bits(0x10); // for arbitration lost test
    config.slave_address_2(0x41, Address2Mask::MASK4);

    let i2c = I2c::<mode::Async>::new(
        p.I2C1,
        p.PB8,
        p.PB9,
        Irqs,
        p.DMA1_CH1,
        p.DMA1_CH2,
        Hertz(100_000),
        config,
    );

    let mut buf_64 = [0; 64]; // buffer is longer than master will send: wait for STOP condition
    let mut buf_20 = [0; 20]; // buffer is shorter than master will send: wait for STOP condition
    let mut buf_20a = [0; 20];
    let mut buf_2 = [0; 2];
    let mut errors = 0;
    let mut tcount = 0;
    let mut counter = 0;

    _ = spawner.spawn(system_ticker(led));

    // start of the actual test
    _ = i2c.slave_start_listen();
    loop {
        counter += 1;
        info!("Loop: {}\r", counter);
        info!("Loop: {}\r", counter);

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

        info!("Waiting for master activity\r");

        let t = i2c.next_transaction().await;
        let dir = t.dir();
        tcount += 1;
        // printing does cost a lot of time, and can interfere with the test if too verbose
        info!(
            "A:x{:x} d:{:?} s:{:x}, act:{:x} r:{:?}\r",
            t.address(),
            dir as u8,
            t.size(),
            t.index(),
            t.result()
        );
        match t.address() {
            0x41 => {
                //Evaluate test 0x41: Good case master write 20 bytes
                checkIsWrite!(dir);
                match t.result() {
                    Ok(_) => {
                        info!("Test 0x41 passed\r");
                        sw.print_data(t.buffer());
                    }
                    Err(err) => {
                        errors += 1;
                        info!("Test 0x41 failed. Error: {:?}\r", err)
                    }
                };
            }
            0x42 => {
                //Evaluate test 0x42: edge case master write exact 64 bytes: must succeed on master and slave
                checkIsWrite!(dir);
                match t.result() {
                    Ok(_) => {
                        info!("Test 0x42 passed. send 64 bytes\r");
                        sw.print_data(t.buffer());
                    }
                    Err(err) => {
                        errors += 1;
                        info!("Test 0x42 failed. Error: {:?}\r", err)
                    }
                };
            }
            0x43 => {
                //"Evaluate test 0x43.edge case master write exact 65 bytes: 1 too many must fail on master and slave
                checkIsWrite!(dir);
                match t.result() {
                    Ok(_) => {
                        errors += 1;
                        info!("Test 0x43 failed. Expected Overrun error Got Ok\r")
                    }
                    Err(err) => info!("Test 0x43 passed. Expected error: Overrun. Error: {:?}\r", err),
                };
            }
            0x44 => {
                // Evaluate test 0x44: master write read combined transaction write 20 bytes, then read 20 bytes
                checkIsRead!(dir);
                match t.result() {
                    Ok(_) => {
                        info!("Test 0x44 passed\r");
                        sw.print_data(t.buffer())
                    }
                    Err(err) => {
                        errors += 1;
                        info!("Test 0x44 failed. Error:{:?}\r", err)
                    }
                };
            }
            0x48 => {
                // 0x48 master read slave write slave did not yet prepare a buffer, master will fail
                checkIsRead!(dir);
                match t.result() {
                    Ok(_) => {
                        info!("Test 0x48 failed. Expected to fail: \r");
                        errors += 1;
                    }
                    Err(err) => info!("Test 0x48 passed. Got expected error: {:?}\r", err),
                };
            }
            0x49 => {
                //"Evaluate test 0x49.  master read 20 bytes good case.
                checkIsRead!(dir);
                match t.result() {
                    Ok(_) => {
                        info!("Test passed");
                    }
                    Err(err) => {
                        errors += 1;
                        info!("Test 0x49 failed. Error: {:?}\r", err)
                    }
                };
            }
            0x4A => {
                // 0x4A master read slave write bad  case: master expects 64 does slave does prepare 20 characters
                checkIsRead!(dir);
                match t.result() {
                    Ok(_) => {
                        info!("Test 0x4A Passed.");
                    }
                    Err(err) => {
                        errors += 1;
                        info!("Test failed. Error: {:?}\r", err)
                    }
                }
            }
            0x4B => {
                // Master-read-slave-write Master expects 64 bytes, Should be ok
                checkIsRead!(dir);
                match t.result() {
                    Ok(_) => {
                        info!("Test 0x4B passed\r");
                    }
                    Err(err) => {
                        errors += 1;
                        info!("Test 0x4B failed. Error: {:?}\r", err)
                    }
                };
            }
            0x4F => {
                // Master-read-slave-write 2 bytes with test summary Should be ok.
                match t.result() {
                    Ok(_) => {
                        info!("Test 0x4F Result send to master\r");
                    }
                    Err(err) => {
                        errors += 1;
                        info!("Test 0x4F failed. Error: {:?}\r", err)
                    }
                };
                info!("Test finished. nr tests/nr errors: {}/{}!\r", tcount, errors);
                info!("------------------\r");
                tcount = 0;
                errors = 0;
            }
            0x4 => {
                // Arbitration lost test Master does read 2 bytes on t.address() 0x10
                // this slave will send 0xFF04, the other slave will send 0xFF03
                // This slave should generate a arbitration lost if the other slave is online
                info!("Evaluate test 0x10: slave arbitration lost.\r");
                checkIsRead!(dir);
                match t.result() {
                    Ok(_) => {
                        info!( "Test 0x10 should fail if a second slave with testcase i2c_salve_arbitration.rs is connected.\r");
                        errors += 1;
                    }
                    Err(err) => info!("Test 0x10 passed. Error: {:?}\r", err),
                };
            }
            _ => (),
        }
        info!("Prepare for test {:x}", t.address());
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
                    Err(_) => info!("Buffer error\r"),
                    _ => (),
                }
            }
            0x4B => {
                // prepare for test 0x4F
                let err = i2c.slave_error_count() as u8;
                let result: [u8; 3] = [tcount, errors, err];
                _ = i2c.slave_prepare_read(&result, i2c::AddressIndex::Address2);
                info!("Count: {} test errors {}   i2  errors: {}\r", tcount, errors, err);
            }
            _ => (),
        }
    }
}

pub struct StringWriter {
    buf: [u8; 256],
    cursor: usize,
}

impl StringWriter {
    pub fn new() -> Self {
        StringWriter {
            buf: [0; 256],
            cursor: 0,
        }
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.buf.len()
    }
    pub fn as_str(&self) -> &str {
        str::from_utf8(&self.buf[0..self.cursor]).unwrap()
    }

    pub fn clear(&mut self) {
        self.cursor = 0;
    }

    fn print_data(&mut self, data: &[u8]) {
        self.cursor = 0;
        for i in 0..data.len() {
            write!(self, " {:x} ", data[i]);
        }
        write!(self, "\n");
        info!("{}", self.as_str());
    }
}

impl Write for StringWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let cap = self.capacity();
        for (i, &b) in self.buf[self.cursor..cap].iter_mut().zip(s.as_bytes().iter()) {
            *i = b;
        }
        self.cursor = usize::min(cap, self.cursor + s.as_bytes().len());
        Ok(())
    }
}
