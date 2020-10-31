#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;
use example_common::*;

use cortex_m_rt::entry;
use nrf52840_hal::gpio;

use embassy::executor::{task, Executor};
use embassy::flash::Flash;
use embassy::util::Forever;
use embassy_nrf::qspi;

const PAGE_SIZE: usize = 4096;

// Workaround for alignment requirements.
// Nicer API will probably come in the future.
#[repr(C, align(4))]
struct AlignedBuf([u8; 4096]);

#[task]
async fn run() {
    let p = embassy_nrf::pac::Peripherals::take().dewrap();

    let port0 = gpio::p0::Parts::new(p.P0);

    let pins = qspi::Pins {
        csn: port0
            .p0_17
            .into_push_pull_output(gpio::Level::High)
            .degrade(),
        sck: port0
            .p0_19
            .into_push_pull_output(gpio::Level::High)
            .degrade(),
        io0: port0
            .p0_20
            .into_push_pull_output(gpio::Level::High)
            .degrade(),
        io1: port0
            .p0_21
            .into_push_pull_output(gpio::Level::High)
            .degrade(),
        io2: Some(
            port0
                .p0_22
                .into_push_pull_output(gpio::Level::High)
                .degrade(),
        ),
        io3: Some(
            port0
                .p0_23
                .into_push_pull_output(gpio::Level::High)
                .degrade(),
        ),
    };

    let config = qspi::Config {
        pins,
        read_opcode: qspi::ReadOpcode::READ4IO,
        write_opcode: qspi::WriteOpcode::PP4IO,
        xip_offset: 0,
        write_page_size: qspi::WritePageSize::_256BYTES,
    };

    let mut q = qspi::Qspi::new(p.QSPI, config);

    let mut id = [1; 3];
    q.custom_instruction(0x9F, &[], &mut id).await.unwrap();
    info!("id: {:[u8]}", id);

    // Read status register
    let mut status = [0; 1];
    q.custom_instruction(0x05, &[], &mut status).await.unwrap();

    info!("status: {:?}", status[0]);

    if status[0] & 0x40 == 0 {
        status[0] |= 0x40;

        q.custom_instruction(0x01, &status, &mut []).await.unwrap();

        info!("enabled quad in status");
    }

    let mut buf = AlignedBuf([0u8; PAGE_SIZE]);

    let pattern = |a: u32| (a ^ (a >> 8) ^ (a >> 16) ^ (a >> 24)) as u8;

    for i in 0..8 {
        info!("page {:?}: erasing... ", i);
        q.erase(i * PAGE_SIZE).await.unwrap();

        for j in 0..PAGE_SIZE {
            buf.0[j] = pattern((j + i * PAGE_SIZE) as u32);
        }

        info!("programming...");
        q.write(i * PAGE_SIZE, &buf.0).await.unwrap();
    }

    for i in 0..8 {
        info!("page {:?}: reading... ", i);
        q.read(i * PAGE_SIZE, &mut buf.0).await.unwrap();

        info!("verifying...");
        for j in 0..PAGE_SIZE {
            assert_eq!(buf.0[j], pattern((j + i * PAGE_SIZE) as u32));
        }
    }

    info!("done!")
}

static EXECUTOR: Forever<Executor> = Forever::new();

#[entry]
fn main() -> ! {
    info!("Hello World!");

    let executor = EXECUTOR.put(Executor::new(cortex_m::asm::wfi));
    executor.spawn(run()).dewrap();

    loop {
        executor.run();
        cortex_m::asm::wfe();
    }
}
