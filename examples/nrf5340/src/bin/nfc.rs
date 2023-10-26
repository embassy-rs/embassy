#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_nrf::gpio::{Level, Output, OutputDrive};
use embassy_nrf::nfc::NfcT;
use embassy_nrf::peripherals::NFCT;
use embassy_nrf::{bind_interrupts, nfc};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    NFCT => nfc::InterruptHandler;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    let mut led = Output::new(p.P0_28, Level::Low, OutputDrive::Standard);
    let mut nfc = NfcT::new(p.NFCT, Irqs);

    // let mut buf: [u8; 1024] = [0; 1024];
    loop {
        nfc.wait_for_field().await;
        // let field = nfc.is_field_present();
        // info!("Field present: {=bool}", field);

        // dbg!("Starting recv_frame");
        // let frame = nfc.recv_frame(&mut buf).await;
        // if let Ok(frame_data) = frame {
        //     dbg!("Got frame: {}", frame_data);
        // } else if let Err(e) = frame {
        //     dbg!("Got frame err: {}", e);
        // }
        led.set_high();
        Timer::after_millis(300).await;
        led.set_low();
        Timer::after_millis(300).await;
    }
}
