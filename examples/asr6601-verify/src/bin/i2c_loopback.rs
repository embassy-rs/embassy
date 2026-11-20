//! I2C0 master <-> I2C1 slave loopback.
//!
//! Wiring:
//! * I2C0 SCL GPIO14 <-> I2C1 SCL GPIO30
//! * I2C0 SDA GPIO15 <-> I2C1 SDA GPIO31
//! * pull-ups from SCL and SDA to 3.3 V

#![no_std]
#![no_main]

use core::ptr::{addr_of_mut, write_volatile};

use cortex_m_rt::entry;
use embassy_asr::gpio::Pull;
use embassy_asr::i2c::{Config as I2cConfig, I2c, I2cSlave, InterruptHandler, SlaveOp};
use embassy_asr::{bind_interrupts, peripherals};
use embassy_asr6601_verify::enable_debug_in_low_power;
use embassy_futures::join::join;
use panic_halt as _;

const WORDS: usize = 24;
const MAGIC: u32 = 0x4932_4321; // "I2C!"
const DONE: u32 = 0x444f_4e45;
const ADDRESS: u8 = 0x42;

bind_interrupts!(struct Irqs {
    I2C0 => InterruptHandler<peripherals::I2C0>;
    I2C1 => InterruptHandler<peripherals::I2C1>;
});

#[unsafe(no_mangle)]
pub static mut I2C_RESULTS: [u32; WORDS] = [0; WORDS];

fn word(index: usize) -> *mut u32 {
    unsafe { addr_of_mut!(I2C_RESULTS).cast::<u32>().add(index) }
}

fn put(index: usize, value: u32) {
    unsafe { write_volatile(word(index), value) }
}

#[entry]
fn main() -> ! {
    put(0, MAGIC);
    put(1, 1);

    let p = embassy_asr::init(Default::default());
    enable_debug_in_low_power();

    let mut master = I2c::new(p.I2C0, p.PA14, p.PA15, Irqs, I2cConfig::default()).unwrap();
    let mut slave = I2cSlave::new(p.I2C1, p.PB14, p.PB15, ADDRESS, Pull::Up, Pull::Up, Irqs).unwrap();

    let tx = [0x11u8, 0x22];
    let mut rx = [0u8; 2];
    let mut slave_rx = [0u8; 2];

    let (master_write_ok, slave_read_ok, master_read_ok, slave_write_ok) = embassy_futures::block_on(async {
        let slave_write = async {
            let op = slave.listen().await;
            let mut status = 0u32;
            if let Ok(SlaveOp::Read) = op {
                if slave.read(&mut slave_rx).await.is_ok() {
                    status = 1;
                }
            }
            status
        };
        let master_write = async { master.write(ADDRESS, &tx).await.is_ok() };
        let (master_write_ok, slave_read_ok) = join(master_write, slave_write).await;
        put(14, 1);
        let _ = slave.wait_stop().await;

        let slave_read = async {
            let op = slave.listen().await;
            let mut status = 0u32;
            if let Ok(SlaveOp::Write) = op {
                if slave.write(&[0x33, 0x44]).await.is_ok() {
                    status = 1;
                }
            }
            status
        };
        let master_read = async { master.read(ADDRESS, &mut rx).await.is_ok() };
        let (master_read_ok, slave_write_ok) = join(master_read, slave_read).await;
        put(14, 2);
        let _ = slave.wait_stop().await;
        (master_write_ok, slave_read_ok, master_read_ok, slave_write_ok)
    });

    put(2, 0x100);
    put(3, u32::from(master_write_ok));
    put(4, slave_read_ok);
    put(5, u32::from_le_bytes([slave_rx[0], slave_rx[1], 0, 0]));
    put(6, 0x1122);
    put(7, u32::from(master_read_ok));
    put(8, slave_write_ok);
    put(9, u32::from_le_bytes([rx[0], rx[1], 0, 0]));
    put(10, 0x3344);
    put(1, DONE);

    loop {
        cortex_m::asm::wfi();
    }
}
