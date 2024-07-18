#![no_std]
#![no_main]

use core::fmt;
use core::fmt::Write;
use core::str;
use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::i2c::{Error, I2c};
use embassy_stm32::time::Hertz;
use embassy_stm32::{bind_interrupts, i2c, peripherals};
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    I2C1 => i2c::EventInterruptHandler<peripherals::I2C1>, i2c::ErrorInterruptHandler<peripherals::I2C1>;
});

const INTERTEST_WAIT_MS: u64 = 100; // long time in case the slave is build in debug mode.

#[embassy_executor::task]
pub async fn system_ticker(mut led: Output<'static>) {
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
    let led = Output::new(p.PA5, Level::High, Speed::Low); // nucleog070rb board

    let mut sw = StringWriter::new();

    info!("I2c test.  Master role. Connect another device in slave role to the bus.");

    let mut i2c = I2c::new(
        p.I2C1,
        p.PB8,
        p.PB9,
        Irqs,
        p.DMA1_CH1,
        p.DMA1_CH2,
        Hertz(100_000),
        Default::default(),
    );

    Timer::after(Duration::from_millis(1000)).await;
    _ = spawner.spawn(system_ticker(led));

    // start of the acutal test
    let mut counter = 0;
    loop {
        counter += 1;
        info!("Loop counter: {:?}\r", counter);

        let mut buf_20 = [0_u8; 20];
        let mut buf_64 = [0_u8; 64];
        let mut buf_65 = [0_u8; 65];
        let mut buf_2 = [0_u8; 2];

        info!("Start of test\r");

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
            Ok(_) => info!("Test 0x41 passed\r"),
            Err(Error::Timeout) => info!("Operation timed out\r"),
            Err(err) => info!("Test 0x41 failed. Error: {:?}\r", err),
        };
        Timer::after(Duration::from_millis(INTERTEST_WAIT_MS)).await;

        // 0x42 edge case master write exact 64 bytes: must succeed on master and slave
        match i2c.blocking_write(0x42, &buf_64) {
            Ok(_) => info!("Test 0x42 passed. Master write exact 64 bytes\r"),
            Err(Error::Timeout) => info!("Operation timed out\r"),
            Err(err) => info!("Test 0x42 failed. Got error: {:?}\r", err),
        };
        Timer::after(Duration::from_millis(INTERTEST_WAIT_MS)).await;

        // 0x43 edge case master write exact 65 bytes: 1 too many must fail  on master and slave
        match i2c.blocking_write(0x43, &buf_65) {
            Ok(_) => info!("Test passed.\r"),
            Err(Error::Timeout) => info!("Operation timed out\r"),
            Err(err) => info!("Test 0x43 failed: Error: {:?}\r", err),
        };
        /* skip test for now
        // 0x44 master write read combined transaction write 20 bytes, then read 20 bytes
        match i2c.blocking_write_read(0x44, &buf_20, &mut buf_20a) {
            Ok(_) => {
                info!( "Test 0x44 Ok \r");
                info!("Uppercase input should be transformed to lowercase, A -> b \r");
                sw.print_data( &buf_20a);
            }
            Err(Error::Timeout) => info!( "Operation timed out\r"),
            Err(err) => info!( "Test 0x44 error: {:?}\r", err),
        };
        */
        // master read. Slave did not prepare a buffer (buffer empty) and will NACK
        Timer::after(Duration::from_millis(INTERTEST_WAIT_MS)).await;
        match i2c.blocking_read(0x48, &mut buf_20) {
            Ok(_) => {
                info!("Test 0x48 passed.Master cannot detect this is an error case!\r");
            }
            Err(Error::Timeout) => info!("Operation timed out\r"),
            Err(err) => info!("Test 0x48 failed. Error:{:?}\r", err),
        };
        Timer::after(Duration::from_millis(INTERTEST_WAIT_MS)).await;

        // master read 20 bytes good case.
        match i2c.blocking_read(0x49, &mut buf_20) {
            Ok(_) => {
                info!("Test 0x49 Read Ok\r");
                sw.print_data(&buf_20);
            }
            Err(Error::Timeout) => info!("Operation timed out\r"),
            Err(err) => info!("Test 0x49 Error: {:?}\r", err),
        };
        Timer::after(Duration::from_millis(INTERTEST_WAIT_MS)).await;

        // master read 64 bytes, but the slave did prepair only 20 Should fail with NACK
        match i2c.blocking_read(0x4A, &mut buf_64) {
            Ok(_) => {
                info!("Test 0x4A passed. Master cannot detect this error case\r");
                sw.print_data(&buf_64);
            }
            Err(Error::Timeout) => info!("Operation timed out\r"),
            Err(err) => info!("Test 0x4A failed.  Error: {:?}\r", err),
        };
        Timer::after(Duration::from_millis(INTERTEST_WAIT_MS)).await;
        // master read 64 bytes, slave did prepare 64 bytes. good case
        match i2c.blocking_read(0x4B, &mut buf_64) {
            Ok(_) => {
                info!("Test 0x4B passed\r");
                sw.print_data(&buf_64);
            }
            Err(Error::Timeout) => info!("Operation timed out\r"),
            Err(err) => info!("Test 0x4B failed. Error: {:?}\r", err),
        };
        Timer::after(Duration::from_millis(INTERTEST_WAIT_MS)).await;

        // test for arbitration lost. 2 slaves respond on address 10 one with 0xFF03, on with 0xFF04
        // if both are online the one with 0xFF04 should report arbitration loss, here we read 0xFF03
        match i2c.blocking_read(0x10, &mut buf_2) {
            Ok(_) => {
                info!("Test 0x10 Received {:x}:{:x}\r", buf_2[0], buf_2[1]);
                info!("Look in the log of the slaves to evaluate the result\r");
            }
            Err(Error::Timeout) => info!("Operation timed out\r"),
            Err(err) => info!("Test 0x10 Failed: Error: {:?}", err),
        };

        // 0x4F Master does read 2 bytes with the result of the slave
        let mut result: [u8; 3] = [0, 0, 0];
        match i2c.blocking_read(0x4F, &mut result) {
            Ok(_) => info!(
                "Test result: count {} errors {} i2c errors:{}\r",
                result[0], result[1], result[2]
            ),
            Err(Error::Timeout) => info!("Operation timed out\r"),
            Err(err) => info!("Test 0x4F unexpected error: {:?}\r", err),
        };
        info!("\r");

        Timer::after(Duration::from_millis(10_000)).await;
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
            write!(self, " {:x} ", data[i]).unwrap();
        }
        writeln!(self, "").unwrap();
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
