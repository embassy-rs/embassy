#![no_std]
#![no_main]
#[cfg(feature = "rp2040")]
teleprobe_meta::target!(b"rpi-pico");
#[cfg(feature = "rp235xb")]
teleprobe_meta::target!(b"pimoroni-pico-plus-2");

use core::future::Future;

use defmt::{assert_eq, error, info, panic};
use defmt_rtt as _;
use embassy_embedded_hal::SetConfig;
use embassy_executor::Spawner;
use embassy_rp::clocks::{PllConfig, XoscConfig};
use embassy_rp::config::Config as rpConfig;
use embassy_rp::peripherals::{I2C0, I2C1};
use embassy_rp::time::Hertz;
use embassy_rp::{bind_interrupts, i2c, i2c_slave, mode};
use embedded_hal_1::i2c::Operation;
use panic_probe as _;

use crate::i2c::AbortReason;

bind_interrupts!(struct Irqs {
    I2C0_IRQ => i2c::InterruptHandler<I2C0>;
    I2C1_IRQ => i2c::InterruptHandler<I2C1>;
});

const DEV_ADDR: u8 = 0x42;

#[embassy_executor::task]
async fn device_task(mut dev: i2c_slave::I2cSlave<'static, mode::Async>) -> ! {
    info!("Device start");

    let mut count = 0xD0;

    loop {
        let mut buf = [0u8; 128];
        match dev.listen(&mut buf).await {
            Ok(i2c_slave::Command::GeneralCall(len)) => {
                assert_eq!(buf[..len], [0xCA, 0x11], "recieving the general call failed");
                info!("General Call - OK");
            }
            Ok(i2c_slave::Command::Read) => {
                loop {
                    match dev.respond_to_read(&[count]).await {
                        Ok(x) => match x {
                            i2c_slave::ReadStatus::Done => break,
                            i2c_slave::ReadStatus::NeedMoreBytes => count += 1,
                            i2c_slave::ReadStatus::LeftoverBytes(x) => panic!("tried to write {} extra bytes", x),
                        },
                        Err(e) => match e {
                            embassy_rp::i2c_slave::Error::Abort(AbortReason::Other(n)) => panic!("Other {:b}", n),
                            _ => panic!("{}", e),
                        },
                    }
                }
                count += 1;
            }
            Ok(i2c_slave::Command::Write(len)) => match len {
                1 => {
                    assert_eq!(buf[..len], [0xAA], "recieving a single byte failed");
                    info!("Single Byte Write - OK")
                }
                4 => {
                    assert_eq!(buf[..len], [0xAA, 0xBB, 0xCC, 0xDD], "recieving 4 bytes failed");
                    info!("4 Byte Write - OK")
                }
                32 => {
                    assert_eq!(
                        buf[..len],
                        [
                            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
                            25, 26, 27, 28, 29, 30, 31
                        ],
                        "recieving 32 bytes failed"
                    );
                    info!("32 Byte Write - OK")
                }
                _ => panic!("Invalid write length {}", len),
            },
            Ok(i2c_slave::Command::WriteRead(len)) => {
                info!("device received write read: {:x}", buf[..len]);
                match buf[0] {
                    0xC2 => {
                        let resp_buff = [0xD1, 0xD2, 0xD3, 0xD4];
                        dev.respond_to_read(&resp_buff).await.unwrap();
                    }
                    0xC8 => {
                        let mut resp_buff = [0u8; 32];
                        for i in 0..32 {
                            resp_buff[i] = i as u8;
                        }
                        dev.respond_to_read(&resp_buff).await.unwrap();
                        // reset count for next round of tests
                        count = 0xD0;
                    }
                    x => panic!("Invalid Write Read {:x}", x),
                }
            }
            Err(e) => match e {
                embassy_rp::i2c_slave::Error::Abort(AbortReason::Other(n)) => panic!("Other {:b}", n),
                _ => panic!("{}", e),
            },
        }
    }
}

/// Bound an operation so a hang is reported: `Ok(true)` if it hung.
async fn bounded(op: impl Future<Output = Result<(), i2c::Error>>) -> Result<bool, i2c::Error> {
    match embassy_time::with_timeout(embassy_time::Duration::from_millis(500), op).await {
        Ok(r) => r.map(|()| false),
        Err(_) => Ok(true),
    }
}

fn nack() -> Result<bool, i2c::Error> {
    Err(i2c::Error::Abort(AbortReason::NoAcknowledge))
}

async fn set_freq(con: &mut i2c::I2c<'static, mode::Async>, hz: u32) {
    let mut cfg = i2c::Config::default();
    cfg.frequency = Hertz(hz);
    con.set_config(&cfg).unwrap();
    embassy_time::Timer::after_millis(30).await;
}

/// Reports every check, then fails the test if any did not hold.
struct ErrorReporter {
    failed: u32,
}

impl ErrorReporter {
    fn check(&mut self, name: &str, got: Result<bool, i2c::Error>, want: Result<bool, i2c::Error>) {
        if got == want {
            info!("PASS {}", name);
        } else {
            error!("FAIL {}: got {:?}", name, got);
            self.failed += 1;
        }
    }

    fn finish(self) {
        if self.failed > 0 {
            panic!("{} i2c error checks failed", self.failed);
        }
    }
}

async fn error_checks(con: &mut i2c::I2c<'static, mode::Async>) {
    const ABSENT: u8 = 0x43;
    let mut t = ErrorReporter { failed: 0 };

    set_freq(con, 100_000).await;
    let r = con.blocking_write(ABSENT, &[1]).map(|()| false);
    t.check("blocking write to an absent address is NACKed", r, nack());

    let r = con.blocking_write(DEV_ADDR, &[0xAA]).map(|()| false);
    t.check("blocking write after a NACK succeeds", r, Ok(false));
    // Drain RX FIFO.
    embassy_futures::yield_now().await;

    // STOP after NACK takes 1ms@1 kHz.
    set_freq(con, 1000).await;
    let _ = bounded(con.write(ABSENT, &[1])).await;
    let r = bounded(con.write(ABSENT, &[1])).await;
    t.check("1 kHz write straight after a NACK is NACKed", r, nack());

    // Absent address - device task would panic during cancel.
    set_freq(con, 10_000).await;
    let cancel = con.write(ABSENT, &[1, 2, 3, 4]);
    let _ = embassy_time::with_timeout(embassy_time::Duration::from_micros(300), cancel).await;
    let r = bounded(con.write(ABSENT, &[1])).await;
    t.check("write straight after a cancelled one is NACKed", r, nack());

    let r = bounded(con.write(DEV_ADDR, &[])).await;
    t.check(
        "async zero-length write is rejected",
        r,
        Err(i2c::Error::InvalidWriteBufferLength),
    );

    t.finish();
}

async fn controller_task(con: &mut i2c::I2c<'static, mode::Async>) {
    info!("Controller start");

    {
        let buf = [0xCA, 0x11];
        con.write(0u8, &buf).await.unwrap();
        info!("Controler general call write");
        embassy_futures::yield_now().await;
    }

    {
        let mut buf = [0u8];
        con.read(DEV_ADDR, &mut buf).await.unwrap();
        assert_eq!(buf, [0xD0], "single byte read failed");
        info!("single byte read - OK");
        embassy_futures::yield_now().await;
    }

    {
        let mut buf = [0u8; 4];
        con.read(DEV_ADDR, &mut buf).await.unwrap();
        assert_eq!(buf, [0xD1, 0xD2, 0xD3, 0xD4], "single byte read failed");
        info!("4 byte read - OK");
        embassy_futures::yield_now().await;
    }

    {
        let buf = [0xAA];
        con.write(DEV_ADDR, &buf).await.unwrap();
        info!("Controler single byte write");
        embassy_futures::yield_now().await;
    }

    {
        let buf = [0xAA, 0xBB, 0xCC, 0xDD];
        con.write(DEV_ADDR, &buf).await.unwrap();
        info!("Controler 4 byte write");
        embassy_futures::yield_now().await;
    }

    {
        let mut buf = [0u8; 32];
        for i in 0..32 {
            buf[i] = i as u8;
        }
        con.write(DEV_ADDR, &buf).await.unwrap();
        info!("Controler 32 byte write");
        embassy_futures::yield_now().await;
    }

    {
        let mut buf = [0u8; 4];
        let mut ops = [Operation::Write(&[0xC2]), Operation::Read(&mut buf)];
        con.transaction(DEV_ADDR, &mut ops).await.unwrap();
        assert_eq!(buf, [0xD1, 0xD2, 0xD3, 0xD4], "write_read failed");
        info!("write_read - OK");
        embassy_futures::yield_now().await;
    }

    {
        let mut buf = [0u8; 32];
        let mut ops = [Operation::Write(&[0xC8]), Operation::Read(&mut buf)];
        con.transaction(DEV_ADDR, &mut ops).await.unwrap();
        assert_eq!(
            buf,
            [
                0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27,
                28, 29, 30, 31
            ],
            "write_read of 32 bytes failed"
        );
        info!("large write_read - OK")
    }

    #[embassy_executor::main]
    async fn main(spawner: Spawner) {
        let mut config = rpConfig::default();
        // Configure clk_sys to 48MHz to support 1kHz scl.
        // In theory it can go lower, but we won't bother to test below 1kHz.
        config.clocks.xosc = Some(XoscConfig {
            hz: 12_000_000,
            delay_multiplier: 128,
            sys_pll: Some(PllConfig {
                refdiv: 1,
                fbdiv: 120,
                post_div1: 6,
                post_div2: 5,
            }),
            usb_pll: Some(PllConfig {
                refdiv: 1,
                fbdiv: 120,
                post_div1: 6,
                post_div2: 5,
            }),
        });

        let p = embassy_rp::init(config);
        info!("Hello World!");

        let d_sda = p.PIN_19;
        let d_scl = p.PIN_18;
        let mut config = i2c_slave::Config::default();
        config.addr = DEV_ADDR as u16;
        let device = i2c_slave::I2cSlave::new(p.I2C1, d_sda, d_scl, Irqs, config);

        spawner.spawn(device_task(device).unwrap());

        let c_sda = p.PIN_21;
        let c_scl = p.PIN_20;
        let mut controller = i2c::I2c::new(p.I2C0, c_sda, c_scl, Irqs, Default::default());

        for freq in [1000, 100_000, 400_000, 1_000_000] {
            info!("testing at {}hz", freq);
            let mut config = i2c::Config::default();
            config.frequency = Hertz(freq);
            controller.set_config(&config).unwrap();
            controller_task(&mut controller).await;
        }

        error_checks(&mut controller).await;

        info!("Test OK");
        cortex_m::asm::bkpt();
    }
}
