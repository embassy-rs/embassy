// required-features: mem2mem
#![no_std]
#![no_main]
#[path = "../common.rs"]
mod common;

use core::mem::forget;

use common::*;
use defmt::assert_eq;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::dma;
use panic_probe as _;
use rand_chacha::ChaCha8Rng;
use rand_chacha::rand_core::{Rng, SeedableRng};

// TODO use correct section for BDMA on H7
static mut SRC_BUF1: [u8; 8192] = [0; _];
static mut DST_BUF1: [u8; 8192] = [0; _];
static mut SRC_BUF2: [u8; 128] = [0; _];
static mut DST_BUF2: [u8; 128] = [0; _];

#[cfg_attr(
    feature = "stop",
    embassy_executor::main(executor = "embassy_stm32::executor::Executor", entry = "cortex_m_rt::entry")
)]
#[cfg_attr(not(feature = "stop"), embassy_executor::main)]
async fn main(_spawner: Spawner) {
    let p = init();
    info!("Hello World!");

	// Simulates forgetting a DMA transfer while it is running.
	// Configuring another transfer, requires stopped DMA,
	// so it should stop DMA effectively stopping previous transfer.
	// Newly configured transfer should work correctly.

	let irqs = irqs!(MEM2MEM_DMA);

	#[cfg(feature = "mem2mem-dma")]
	transfer_mem2mem(dma::Channel::new(peri!(p, MEM2MEM_DMA), irqs), "DMA").await;
	#[cfg(feature = "mem2mem-bdma")]
	transfer_mem2mem(dma::Channel::new(peri!(p, MEM2MEM_BDMA), irqs), "BDMA").await;
	#[cfg(feature = "mem2mem-mdma")]
	transfer_mem2mem(dma::Channel::new(peri!(p, MEM2MEM_MDMA), irqs), "MDMA").await;

    info!("Test OK");
    cortex_m::asm::bkpt();
}

async fn transfer_mem2mem<'d>(mut ch: dma::Channel<'d>, name: &str) {
	info!("Testing {}", name);

	let src_buf1 = unsafe {
		&mut SRC_BUF1[..]
	};
	let dst_buf1 = unsafe {
		&mut DST_BUF1[..]
	};
	let src_buf2 = unsafe {
		&mut SRC_BUF2[..]
	};
	let dst_buf2 = unsafe {
		&mut DST_BUF2[..]
	};

	dst_buf1.fill(0);
	dst_buf2.fill(0);

	let mut rng = ChaCha8Rng::seed_from_u64(2137);
	rng.fill_bytes(src_buf1);
	rng.fill_bytes(src_buf2);

	let transfer = unsafe { ch.transfer(0, src_buf1, dst_buf1.as_mut_ptr(), dma::TransferOptions::default()) };
	forget(transfer);

	// Start new transfer as fast as possible.
	let transfer = unsafe { ch.transfer(0, src_buf2, dst_buf2.as_mut_ptr(), dma::TransferOptions::default()) };
	transfer.await;

	// Verify whether DMA actually worked.
	assert_eq!(dst_buf2, src_buf2);

	if src_buf1 == dst_buf1 {
		warn!("First transfer completed before second was started.");
	}
}
