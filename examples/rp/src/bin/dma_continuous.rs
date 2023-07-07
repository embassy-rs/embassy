#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::info;
use embassy_executor::Spawner;
use embassy_rp::dma::{self, ContinuousTransfer};
use embassy_rp::pac::dma::vals::TreqSel;
use embassy_rp::Peripheral;
use embassy_time::{Duration, Instant, Timer};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let mut dma0 = p.DMA_CH0.into_ref();
    let mut dma1 = p.DMA_CH1.into_ref();

    loop {
        let mut from_buf = [0u32; 300];

        for (i, x) in from_buf.iter_mut().enumerate() {
            *x = (2 - i as u32 / 100) * 100 + i as u32 % 100 + 1;
        }

        let mut control_input = [0u32; 4];

        let mut to_buf0 = [0u32; 100];
        let mut to_buf1 = [0u32; 100];
        let mut to_buf2 = [0u32; 100];

        let start = Instant::now();
        let mut cdma = ContinuousTransfer::start_new(
            dma0.reborrow(),
            dma1.reborrow(),
            &mut control_input,
            &mut to_buf2,
            TreqSel::PERMANENT,
            dma::Read::Increase(&from_buf),
        );
        #[allow(unused_assignments)]
        let mut in_time = true;
        let mut n_in_time = 0;

        (cdma, in_time) = cdma.next(&mut to_buf1).await;
        if in_time {
            n_in_time += 1;
        };

        (cdma, in_time) = cdma.next(&mut to_buf0).await;
        if in_time {
            n_in_time += 1;
        };

        cdma.stop().await;
        let time = Instant::now().duration_since(start);
        info!("time: {} µs", time.as_micros());

        info!("n_in_time {} out of 2", n_in_time);
        info!("{}", to_buf0);
        info!("{}", to_buf1);
        info!("{}", to_buf2);

        Timer::after(Duration::from_secs(1)).await;
    }
}
