#![no_std]
#![no_main]
#![feature(min_type_alias_impl_trait)]
#![feature(impl_trait_in_bindings)]
#![feature(type_alias_impl_trait)]
#![allow(incomplete_features)]

#[path = "../example_common.rs"]
mod example_common;

use defmt::{assert_eq, panic};
use embassy::executor::Spawner;
use embassy::traits::flash::Flash;
use embassy_nrf::Peripherals;
use embassy_nrf::{interrupt, qspi};
use example_common::*;
use futures::pin_mut;

const PAGE_SIZE: usize = 4096;

// Workaround for alignment requirements.
// Nicer API will probably come in the future.
#[repr(C, align(4))]
struct AlignedBuf([u8; 4096]);

#[embassy::main]
async fn main(spawner: Spawner) {
    let p = Peripherals::take().unwrap();

    let csn = p.P0_17;
    let sck = p.P0_19;
    let io0 = p.P0_20;
    let io1 = p.P0_21;
    let io2 = p.P0_22;
    let io3 = p.P0_23;

    let config = qspi::Config::default();
    let irq = interrupt::take!(QSPI);
    let q = qspi::Qspi::new(p.QSPI, irq, sck, csn, io0, io1, io2, io3, config);
    pin_mut!(q);

    let mut id = [1; 3];
    q.as_mut()
        .custom_instruction(0x9F, &[], &mut id)
        .await
        .unwrap();
    info!("id: {}", id);

    // Read status register
    let mut status = [4; 1];
    q.as_mut()
        .custom_instruction(0x05, &[], &mut status)
        .await
        .unwrap();

    info!("status: {:?}", status[0]);

    if status[0] & 0x40 == 0 {
        status[0] |= 0x40;

        q.as_mut()
            .custom_instruction(0x01, &status, &mut [])
            .await
            .unwrap();

        info!("enabled quad in status");
    }

    let mut buf = AlignedBuf([0u8; PAGE_SIZE]);

    let pattern = |a: u32| (a ^ (a >> 8) ^ (a >> 16) ^ (a >> 24)) as u8;

    for i in 0..8 {
        info!("page {:?}: erasing... ", i);
        q.as_mut().erase(i * PAGE_SIZE).await.unwrap();

        for j in 0..PAGE_SIZE {
            buf.0[j] = pattern((j + i * PAGE_SIZE) as u32);
        }

        info!("programming...");
        q.as_mut().write(i * PAGE_SIZE, &buf.0).await.unwrap();
    }

    for i in 0..8 {
        info!("page {:?}: reading... ", i);
        q.as_mut().read(i * PAGE_SIZE, &mut buf.0).await.unwrap();

        info!("verifying...");
        for j in 0..PAGE_SIZE {
            assert_eq!(buf.0[j], pattern((j + i * PAGE_SIZE) as u32));
        }
    }

    info!("done!")
}
