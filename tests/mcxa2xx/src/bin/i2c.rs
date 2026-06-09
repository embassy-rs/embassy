//! I2c controller + target (Async and DMA variants).
//!
//! Runs the same controller-side script twice against the same target
//! LPI2C instance: first with an Async-mode target, then with a DMA-mode
//! target. Reusing peripherals across phases via `reborrow()` keeps the
//! board wiring constant.

#![no_std]
#![no_main]

teleprobe_meta::target!(b"frdm-mcx-a266");

use embassy_executor::Spawner;
use embassy_futures::select::{Either, select};
use hal::bind_interrupts;
use hal::clocks::config::Div8;
use hal::config::Config;
use hal::i2c::{controller, target};
use hal::peripherals::{LPI2C1, LPI2C2};
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

bind_interrupts!(
    struct Irqs {
        LPI2C1 => target::InterruptHandler<LPI2C1>;
        LPI2C2 => controller::InterruptHandler<LPI2C2>;
    }
);

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    config.clock_cfg.sirc.fro_lf_div = Div8::from_divisor(1);

    let mut p = hal::init(config);

    defmt::info!("I2C controller + target test");

    // ---- Phase 1: Async target ----
    defmt::info!("---- Phase 1: Async target ----");
    {
        let mut tcfg = target::Config::default();
        tcfg.address = target::Address::Dual(0x13, 0x37);
        let mut target =
            target::I2c::new_async(p.LPI2C1.reborrow(), p.P1_1.reborrow(), p.P1_0.reborrow(), Irqs, tcfg).unwrap();

        let mut i2c = controller::I2c::new_async(
            p.LPI2C2.reborrow(),
            p.P1_9.reborrow(),
            p.P1_8.reborrow(),
            Irqs,
            controller::Config::default(),
        )
        .unwrap();

        let mut addr0_value = [0u8];
        let mut addr1_value = [0u8];
        let target_fut = async {
            loop {
                defmt::debug!("Target listen");
                let request = target.async_listen().await.unwrap();
                defmt::info!("Received event {}", request);

                match request {
                    target::Request::Read(0x13) => {
                        defmt::dbg!(target.async_respond_to_read(&addr0_value).await.unwrap());
                    }
                    target::Request::Read(0x37) => {
                        target.async_respond_to_read(&addr1_value).await.unwrap();
                    }
                    target::Request::Write(0x13) => {
                        target.async_respond_to_write(&mut addr0_value).await.unwrap();
                    }
                    target::Request::Write(0x37) => {
                        target.async_respond_to_write(&mut addr1_value).await.unwrap();
                    }
                    _ => {}
                }
            }
        };

        match select(target_fut, controller_script(&mut i2c)).await {
            Either::First(_) => defmt::panic!("target loop exited unexpectedly"),
            Either::Second(_) => {}
        }
    }

    // ---- Phase 2: DMA target ----
    defmt::info!("---- Phase 2: DMA target ----");
    {
        let mut tcfg = target::Config::default();
        tcfg.address = target::Address::Dual(0x13, 0x37);
        let mut target = target::I2c::new_async_with_dma(
            p.LPI2C1.reborrow(),
            p.P1_1.reborrow(),
            p.P1_0.reborrow(),
            p.DMA0_CH0.reborrow(),
            p.DMA0_CH1.reborrow(),
            Irqs,
            tcfg,
        )
        .unwrap();

        let mut i2c = controller::I2c::new_async(
            p.LPI2C2.reborrow(),
            p.P1_9.reborrow(),
            p.P1_8.reborrow(),
            Irqs,
            controller::Config::default(),
        )
        .unwrap();

        let mut addr0_value = [0u8];
        let mut addr1_value = [0u8];
        let target_fut = async {
            loop {
                defmt::debug!("Target listen");
                let request = target.async_listen().await.unwrap();
                defmt::info!("Received event {}", request);

                match request {
                    target::Request::Read(0x13) => {
                        defmt::dbg!(target.async_respond_to_read(&addr0_value).await.unwrap());
                    }
                    target::Request::Read(0x37) => {
                        target.async_respond_to_read(&addr1_value).await.unwrap();
                    }
                    target::Request::Write(0x13) => {
                        target.async_respond_to_write(&mut addr0_value).await.unwrap();
                    }
                    target::Request::Write(0x37) => {
                        target.async_respond_to_write(&mut addr1_value).await.unwrap();
                    }
                    _ => {}
                }
            }
        };

        match select(target_fut, controller_script(&mut i2c)).await {
            Either::First(_) => defmt::panic!("target loop exited unexpectedly"),
            Either::Second(_) => {}
        }
    }

    defmt::info!("Test OK");
    cortex_m::asm::bkpt();
}

async fn controller_script(i2c: &mut controller::I2c<'_, hal::i2c::Async>) {
    // Give the target side time to enter `async_listen` and arm its
    // interrupts before the first START. Without this the target can
    // miss the very first address match of a phase.
    embassy_time::Timer::after_millis(250).await;

    let mut buf = [0u8];

    defmt::info!("Write read 0x13");
    i2c.async_write_read(0x13, &[13], &mut buf).await.unwrap();
    assert_eq!(buf[0], 13);
    defmt::info!("Write read 0x37");
    i2c.async_write_read(0x37, &[37], &mut buf).await.unwrap();
    assert_eq!(buf[0], 37);

    defmt::info!("Read 0x13");
    i2c.async_read(0x13, &mut buf).await.unwrap();
    assert_eq!(buf[0], 13);
    defmt::info!("Read 0x37");
    i2c.async_read(0x37, &mut buf).await.unwrap();
    assert_eq!(buf[0], 37);

    defmt::info!("Write 0x01");
    let error = i2c.async_write(0x01, &[0]).await.unwrap_err();
    assert_eq!(error, controller::IOError::AddressNack);
}
