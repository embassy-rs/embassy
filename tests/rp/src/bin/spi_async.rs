//! Make sure to connect GPIO pins 3 (`PIN_3`) and 4 (`PIN_4`) together
//! to run this test.
//!
#![no_std]
#![no_main]
teleprobe_meta::target!(b"rpi-pico");

use defmt::{assert_eq, *};
use embassy_executor::Spawner;
use embassy_rp::spi::{Config, Spi};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    info!("Hello World!");

    let clk = p.PIN_2;
    let mosi = p.PIN_3;
    let miso = p.PIN_4;

    let mut spi = Spi::new(p.SPI0, clk, mosi, miso, p.DMA_CH0, p.DMA_CH1, Config::default());

    // equal rx & tx buffers
    {
        let tx_buf = [1_u8, 2, 3, 4, 5, 6];
        let mut rx_buf = [0_u8; 6];
        spi.transfer(&mut rx_buf, &tx_buf).await.unwrap();
        assert_eq!(rx_buf, tx_buf);
    }

    // tx > rx buffer
    {
        let tx_buf = [7_u8, 8, 9, 10, 11, 12];

        let mut rx_buf = [0_u8; 3];
        spi.transfer(&mut rx_buf, &tx_buf).await.unwrap();
        assert_eq!(rx_buf, tx_buf[..3]);

        defmt::info!("tx > rx buffer - OK");
    }

    // we make sure to that clearing FIFO works after the uneven buffers

    // equal rx & tx buffers
    {
        let tx_buf = [13_u8, 14, 15, 16, 17, 18];
        let mut rx_buf = [0_u8; 6];
        spi.transfer(&mut rx_buf, &tx_buf).await.unwrap();
        assert_eq!(rx_buf, tx_buf);

        defmt::info!("buffer rx length == tx length - OK");
    }

    // rx > tx buffer
    {
        let tx_buf = [19_u8, 20, 21];
        let mut rx_buf = [0_u8; 6];

        // we should have written dummy data to tx buffer to sync clock.
        spi.transfer(&mut rx_buf, &tx_buf).await.unwrap();

        assert_eq!(
            rx_buf[..3],
            tx_buf,
            "only the first 3 TX bytes should have been received in the RX buffer"
        );
        assert_eq!(rx_buf[3..], [0, 0, 0], "the rest of the RX bytes should be empty");
        defmt::info!("buffer rx length > tx length - OK");
    }

    // equal rx & tx buffers
    {
        let tx_buf = [22_u8, 23, 24, 25, 26, 27];
        let mut rx_buf = [0_u8; 6];
        spi.transfer(&mut rx_buf, &tx_buf).await.unwrap();

        assert_eq!(rx_buf, tx_buf);
        defmt::info!("buffer rx length = tx length - OK");
    }

    info!("Test OK");
    cortex_m::asm::bkpt();
}
