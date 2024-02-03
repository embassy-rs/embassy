#![no_std]
#![no_main]
teleprobe_meta::target!(b"rpi-pico");

use defmt::{assert_eq, info, panic, unwrap};
use embassy_executor::Executor;
use embassy_rp::multicore::{spawn_core1, Stack};
use embassy_rp::peripherals::{I2C0, I2C1};
use embassy_rp::{bind_interrupts, i2c, i2c_slave};
use embedded_hal_1::i2c::Operation;
use embedded_hal_async::i2c::I2c;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _, panic_probe as _, panic_probe as _};

static mut CORE1_STACK: Stack<1024> = Stack::new();
static EXECUTOR0: StaticCell<Executor> = StaticCell::new();
static EXECUTOR1: StaticCell<Executor> = StaticCell::new();

use crate::i2c::AbortReason;

bind_interrupts!(struct Irqs {
    I2C0_IRQ => i2c::InterruptHandler<I2C0>;
    I2C1_IRQ => i2c::InterruptHandler<I2C1>;
});

const DEV_ADDR: u8 = 0x42;

#[embassy_executor::task]
async fn device_task(mut dev: i2c_slave::I2cSlave<'static, I2C1>) -> ! {
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
                            i2c_slave::ReadStatus::LeftoverBytes(x) => {
                                info!("tried to write {} extra bytes", x);
                                break;
                            }
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

#[embassy_executor::task]
async fn controller_task(mut con: i2c::I2c<'static, I2C0, i2c::Async>) {
    info!("Device start");

    {
        let buf = [0xCA, 0x11];
        con.write(0u16, &buf).await.unwrap();
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

    info!("Test OK");
    cortex_m::asm::bkpt();
}

#[cortex_m_rt::entry]
fn main() -> ! {
    let p = embassy_rp::init(Default::default());
    info!("Hello World!");

    let d_sda = p.PIN_19;
    let d_scl = p.PIN_18;
    let mut config = i2c_slave::Config::default();
    config.addr = DEV_ADDR as u16;
    let device = i2c_slave::I2cSlave::new(p.I2C1, d_sda, d_scl, Irqs, config);

    spawn_core1(p.CORE1, unsafe { &mut CORE1_STACK }, move || {
        let executor1 = EXECUTOR1.init(Executor::new());
        executor1.run(|spawner| unwrap!(spawner.spawn(device_task(device))));
    });

    let executor0 = EXECUTOR0.init(Executor::new());

    let c_sda = p.PIN_21;
    let c_scl = p.PIN_20;
    let mut config = i2c::Config::default();
    config.frequency = 5_000;
    let controller = i2c::I2c::new_async(p.I2C0, c_sda, c_scl, Irqs, config);

    executor0.run(|spawner| unwrap!(spawner.spawn(controller_task(controller))));
}
