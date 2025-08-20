#![no_std]
#![no_main]

use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_nrf::gpio::{Level, Output, OutputDrive};
use embassy_nrf::{bind_interrupts, peripherals, spim, spis};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    SERIAL20 => spim::InterruptHandler<peripherals::TWISPI20>;
    SERIAL21 => spis::InterruptHandler<peripherals::TWISPI21>;
});

/*
The following is designed for the nrf54l15 PDK. The peripherals and gpio pins used are in the
peripheral power domain (PERI PD). On Port P1, jumper the following Spim pins to the Spis pins.

TWISPI20 Spim                     TWISPI21 Spis
---------------------------------------------------
P1.04  SCK   --------------->     P1.05  SCK
P1.06  MISO  <---------------     P1.07  MISO
P1.08  MOSI  --------------->     P1.09  MOSI
P1.10  CSN   --------------->     P1.11  CSN
*/

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    info!("Running main!");

    // SPI Slave
    let sck = p.P1_05;
    let miso = p.P1_07;
    let mosi = p.P1_09;
    let cs = p.P1_11;
    let config = spis::Config::default();
    let slave = spis::Spis::new(p.TWISPI21, Irqs, cs, sck, miso, mosi, config);

    // Start the slave before running the master
    unwrap!(spawner.spawn(echo_task(slave)));
    Timer::after_millis(100).await;

    // SPI Master
    let sck = p.P1_04;
    let miso = p.P1_06;
    let mosi = p.P1_08;
    let cs = p.P1_10;
    let mut config = spim::Config::default();
    // The core clock in the peripheral domain is 16 MHz. A prescaler (divisor) of 16 will make the
    // SPI clock 1 MHz.
    config.prescaler = 16;
    let mut master = spim::Spim::new(p.TWISPI20, Irqs, sck, miso, mosi, config);
    let mut cs = Output::new(cs, Level::High, OutputDrive::Standard);

    // First write with no read.
    let tx_buf = [0xC0, 0xDE, 0xCA, 0xFE];
    cs.set_low();
    master.write(&tx_buf).await.unwrap();
    cs.set_high();

    Timer::after_millis(250).await;

    // First transfer (read/write). Will read back the first write.
    let mut rx_buf = [0xFF; 4];
    let tx_buf = [0xDE, 0xAD, 0xC0, 0xDE];
    cs.set_low();
    master.transfer(&mut rx_buf, &tx_buf).await.unwrap();
    cs.set_high();
    info!("master read: {:X}", rx_buf);

    Timer::after_millis(250).await;

    // Second transfer (read/write). Will read back the second write (first transfer).
    let mut rx_buf = [0xFF; 4];
    let tx_buf = [0xBA, 0x5E, 0xBA, 0x11];
    cs.set_low();
    master.transfer(&mut rx_buf, &tx_buf).await.unwrap();
    cs.set_high();
    info!("master read: {:X}", rx_buf);

    Timer::after_millis(250).await;

    // Final read. Will read back the third write (second transfer).
    let mut rx_buf = [0xFF; 4];
    cs.set_low();
    master.read(&mut rx_buf).await.unwrap();
    cs.set_high();
    info!("master read: {:X}", rx_buf);
}

#[embassy_executor::task]
async fn echo_task(mut slave: spis::Spis<'static, peripherals::TWISPI21>) {
    info!("Running echo_task!");
    let mut rx = [0u8; 4];
    let mut tx_next = [0u8; 4];

    loop {
        let (rx_len, _tx_len) = slave.transfer(&mut rx, &tx_next).await.unwrap();
        tx_next.fill(0);
        tx_next[..rx_len].copy_from_slice(&rx[..rx_len]);
    }
}
