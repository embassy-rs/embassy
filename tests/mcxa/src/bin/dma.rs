#![no_std]
#![no_main]

teleprobe_meta::target!(b"frdm-mcx-a266");

use embassy_executor::Spawner;
use embassy_mcxa::clocks::config::Div8;
use embassy_mcxa::dma::{DmaChannel, TransferOptions};
use static_cell::ConstStaticCell;
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

const BUFFER_LENGTH: usize = 4;

static SRC_BUFFER: ConstStaticCell<[u32; BUFFER_LENGTH]> = ConstStaticCell::new([1, 2, 3, 4]);
static DEST_BUFFER: ConstStaticCell<[u32; BUFFER_LENGTH]> = ConstStaticCell::new([0; BUFFER_LENGTH]);
static MEMSET_BUFFER: ConstStaticCell<[u32; BUFFER_LENGTH]> = ConstStaticCell::new([0; BUFFER_LENGTH]);

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut cfg = hal::config::Config::default();
    cfg.clock_cfg.sirc.fro_12m_enabled = true;
    cfg.clock_cfg.sirc.fro_lf_div = Some(Div8::no_div());
    let p = hal::init(cfg);

    let src = SRC_BUFFER.take();
    let dst = DEST_BUFFER.take();
    let mst = MEMSET_BUFFER.take();

    let mut dma_ch0 = DmaChannel::new(p.DMA_CH0);
    let transfer = dma_ch0.mem_to_mem(src, dst, TransferOptions::default()).unwrap();
    transfer.await.unwrap();

    assert_eq!(src, dst);

    let pattern: u32 = 0xDEADBEEF;
    let transfer = dma_ch0.memset(&pattern, mst, TransferOptions::default());
    transfer.await.unwrap();

    assert!(mst.iter().all(|&v| v == pattern));

    defmt::info!("Test OK");
    cortex_m::asm::bkpt();
}
