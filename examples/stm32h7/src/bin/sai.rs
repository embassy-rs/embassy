//! Daisy Seed rev.7(with PCM3060 codec)
//! https://electro-smith.com/products/daisy-seed
#![no_std]
#![no_main]

use embassy_executor::Spawner;
use grounded::uninit::GroundedArrayCell;
use hal::rcc::*;
use hal::sai::*;
use hal::time::Hertz;
use hal::{bind_interrupts, dma, peripherals};
use {defmt_rtt as _, embassy_stm32 as hal, panic_probe as _};

const BLOCK_LENGTH: usize = 32; // 32 samples
const HALF_DMA_BUFFER_LENGTH: usize = BLOCK_LENGTH * 2; //  2 channels
const DMA_BUFFER_LENGTH: usize = HALF_DMA_BUFFER_LENGTH * 2; //  2 half-blocks
const SAMPLE_RATE: u32 = 48000;

//DMA buffer must be in special region. Refer https://embassy.dev/book/#_stm32_bdma_only_working_out_of_some_ram_regions
#[unsafe(link_section = ".sram1_bss")]
static mut TX_BUFFER: GroundedArrayCell<u32, DMA_BUFFER_LENGTH> = GroundedArrayCell::uninit();
#[unsafe(link_section = ".sram1_bss")]
static mut RX_BUFFER: GroundedArrayCell<u32, DMA_BUFFER_LENGTH> = GroundedArrayCell::uninit();

bind_interrupts!(struct Irqs {
    DMA1_STREAM0 => dma::InterruptHandler<peripherals::DMA1_CH0>;
    DMA1_STREAM1 => dma::InterruptHandler<peripherals::DMA1_CH1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = hal::Config::default();
    config.rcc.pll1 = Some(Pll {
        source: PllSource::HSE,
        prediv: PllPreDiv::DIV4,
        mul: PllMul::MUL200,
        fracn: None,
        divp: Some(PllDiv::DIV2),
        divq: Some(PllDiv::DIV5),
        divr: Some(PllDiv::DIV2),
    });
    config.rcc.pll3 = Some(Pll {
        source: PllSource::HSE,
        prediv: PllPreDiv::DIV6,
        mul: PllMul::MUL295,
        fracn: None,
        divp: Some(PllDiv::DIV16),
        divq: Some(PllDiv::DIV4),
        divr: Some(PllDiv::DIV32),
    });
    config.rcc.sys = Sysclk::PLL1_P;
    config.rcc.mux.sai1sel = hal::pac::rcc::vals::Saisel::PLL3_P;
    config.rcc.ahb_pre = AHBPrescaler::DIV2; // 200 Mhz
    config.rcc.apb1_pre = APBPrescaler::DIV2; // 100 Mhz
    config.rcc.apb2_pre = APBPrescaler::DIV2; // 100 Mhz
    config.rcc.apb3_pre = APBPrescaler::DIV2; // 100 Mhz
    config.rcc.apb4_pre = APBPrescaler::DIV2; // 100 Mhz
    config.rcc.hse = Some(Hse {
        freq: Hertz::mhz(16),
        mode: HseMode::Oscillator,
    });

    let p = hal::init(config);

    let (sub_block_tx, sub_block_rx) = hal::sai::split_subblocks(p.SAI1);
    let kernel_clock = hal::rcc::frequency::<hal::peripherals::SAI1>().0;
    let mclk_div = mclk_div_from_u8((kernel_clock / (SAMPLE_RATE * 256)) as u8);

    let mut tx_config = hal::sai::Config::default();
    tx_config.mode = Mode::Master;
    tx_config.tx_rx = TxRx::Transmitter;
    tx_config.sync_output = true;
    tx_config.clock_strobe = ClockStrobe::Falling;
    tx_config.master_clock_divider = mclk_div;
    tx_config.stereo_mono = StereoMono::Stereo;
    tx_config.data_size = DataSize::Data24;
    tx_config.bit_order = BitOrder::MsbFirst;
    tx_config.frame_sync_polarity = FrameSyncPolarity::ActiveHigh;
    tx_config.frame_sync_offset = FrameSyncOffset::OnFirstBit;
    tx_config.frame_length = 64;
    tx_config.frame_sync_active_level_length = embassy_stm32::sai::word::U7(32);
    tx_config.fifo_threshold = FifoThreshold::Quarter;

    let mut rx_config = tx_config.clone();
    rx_config.mode = Mode::Slave;
    rx_config.tx_rx = TxRx::Receiver;
    rx_config.sync_input = SyncInput::Internal;
    rx_config.clock_strobe = ClockStrobe::Rising;
    rx_config.sync_output = false;

    let tx_buffer: &mut [u32] = unsafe {
        let buf = &mut *core::ptr::addr_of_mut!(TX_BUFFER);
        buf.initialize_all_copied(0);
        let (ptr, len) = buf.get_ptr_len();
        core::slice::from_raw_parts_mut(ptr, len)
    };

    let mut sai_transmitter = Sai::new_asynchronous_with_mclk(
        sub_block_tx,
        p.PE5,
        p.PE6,
        p.PE4,
        p.PE2,
        p.DMA1_CH0,
        tx_buffer,
        Irqs,
        tx_config,
    );

    let rx_buffer: &mut [u32] = unsafe {
        let buf = &mut *core::ptr::addr_of_mut!(RX_BUFFER);
        buf.initialize_all_copied(0);
        let (ptr, len) = buf.get_ptr_len();
        core::slice::from_raw_parts_mut(ptr, len)
    };

    let mut sai_receiver = Sai::new_synchronous(sub_block_rx, p.PE3, p.DMA1_CH1, rx_buffer, Irqs, rx_config);

    sai_receiver.start().unwrap();

    let mut buf = [0u32; HALF_DMA_BUFFER_LENGTH];

    loop {
        // write() must be called before read() to start the master (transmitter)
        // clock used by the receiver
        sai_transmitter.write(&buf).await.unwrap();
        sai_receiver.read(&mut buf).await.unwrap();
    }
}

fn mclk_div_from_u8(v: u8) -> MasterClockDivider {
    assert!((1..=63).contains(&v));
    MasterClockDivider::from_bits(v)
}
