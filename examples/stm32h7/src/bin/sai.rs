//! Daisy Seed rev.7(with PCM3060 codec)
//! https://electro-smith.com/products/daisy-seed
#![no_std]
#![no_main]

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32 as hal;
use grounded::uninit::GroundedArrayCell;
use hal::rcc::*;
use hal::sai::*;
use hal::time::Hertz;
use panic_probe as _;

const BLOCK_LENGTH: usize = 32; // 32 samples
const HALF_DMA_BUFFER_LENGTH: usize = BLOCK_LENGTH * 2; //  2 channels
const DMA_BUFFER_LENGTH: usize = HALF_DMA_BUFFER_LENGTH * 2; //  2 half-blocks
const SAMPLE_RATE: u32 = 48000;

//DMA buffer must be in special region. Refer https://embassy.dev/book/#_stm32_bdma_only_working_out_of_some_ram_regions
#[link_section = ".sram1_bss"]
static mut TX_BUFFER: GroundedArrayCell<u32, DMA_BUFFER_LENGTH> = GroundedArrayCell::uninit();
#[link_section = ".sram1_bss"]
static mut RX_BUFFER: GroundedArrayCell<u32, DMA_BUFFER_LENGTH> = GroundedArrayCell::uninit();

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = hal::Config::default();
    config.rcc.pll1 = Some(Pll {
        source: PllSource::HSE,
        prediv: PllPreDiv::DIV4,
        mul: PllMul::MUL200,
        divp: Some(PllDiv::DIV2),
        divq: Some(PllDiv::DIV5),
        divr: Some(PllDiv::DIV2),
    });
    config.rcc.pll3 = Some(Pll {
        source: PllSource::HSE,
        prediv: PllPreDiv::DIV6,
        mul: PllMul::MUL295,
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
        TX_BUFFER.initialize_all_copied(0);
        let (ptr, len) = TX_BUFFER.get_ptr_len();
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
        tx_config,
    );

    let rx_buffer: &mut [u32] = unsafe {
        RX_BUFFER.initialize_all_copied(0);
        let (ptr, len) = RX_BUFFER.get_ptr_len();
        core::slice::from_raw_parts_mut(ptr, len)
    };

    let mut sai_receiver = Sai::new_synchronous(sub_block_rx, p.PE3, p.DMA1_CH1, rx_buffer, rx_config);

    sai_receiver.start();
    sai_transmitter.start();

    let mut buf = [0u32; HALF_DMA_BUFFER_LENGTH];

    loop {
        sai_receiver.read(&mut buf).await.unwrap();
        sai_transmitter.write(&buf).await.unwrap();
    }
}

const fn mclk_div_from_u8(v: u8) -> MasterClockDivider {
    match v {
        1 => MasterClockDivider::Div1,
        2 => MasterClockDivider::Div2,
        3 => MasterClockDivider::Div3,
        4 => MasterClockDivider::Div4,
        5 => MasterClockDivider::Div5,
        6 => MasterClockDivider::Div6,
        7 => MasterClockDivider::Div7,
        8 => MasterClockDivider::Div8,
        9 => MasterClockDivider::Div9,
        10 => MasterClockDivider::Div10,
        11 => MasterClockDivider::Div11,
        12 => MasterClockDivider::Div12,
        13 => MasterClockDivider::Div13,
        14 => MasterClockDivider::Div14,
        15 => MasterClockDivider::Div15,
        16 => MasterClockDivider::Div16,
        17 => MasterClockDivider::Div17,
        18 => MasterClockDivider::Div18,
        19 => MasterClockDivider::Div19,
        20 => MasterClockDivider::Div20,
        21 => MasterClockDivider::Div21,
        22 => MasterClockDivider::Div22,
        23 => MasterClockDivider::Div23,
        24 => MasterClockDivider::Div24,
        25 => MasterClockDivider::Div25,
        26 => MasterClockDivider::Div26,
        27 => MasterClockDivider::Div27,
        28 => MasterClockDivider::Div28,
        29 => MasterClockDivider::Div29,
        30 => MasterClockDivider::Div30,
        31 => MasterClockDivider::Div31,
        32 => MasterClockDivider::Div32,
        33 => MasterClockDivider::Div33,
        34 => MasterClockDivider::Div34,
        35 => MasterClockDivider::Div35,
        36 => MasterClockDivider::Div36,
        37 => MasterClockDivider::Div37,
        38 => MasterClockDivider::Div38,
        39 => MasterClockDivider::Div39,
        40 => MasterClockDivider::Div40,
        41 => MasterClockDivider::Div41,
        42 => MasterClockDivider::Div42,
        43 => MasterClockDivider::Div43,
        44 => MasterClockDivider::Div44,
        45 => MasterClockDivider::Div45,
        46 => MasterClockDivider::Div46,
        47 => MasterClockDivider::Div47,
        48 => MasterClockDivider::Div48,
        49 => MasterClockDivider::Div49,
        50 => MasterClockDivider::Div50,
        51 => MasterClockDivider::Div51,
        52 => MasterClockDivider::Div52,
        53 => MasterClockDivider::Div53,
        54 => MasterClockDivider::Div54,
        55 => MasterClockDivider::Div55,
        56 => MasterClockDivider::Div56,
        57 => MasterClockDivider::Div57,
        58 => MasterClockDivider::Div58,
        59 => MasterClockDivider::Div59,
        60 => MasterClockDivider::Div60,
        61 => MasterClockDivider::Div61,
        62 => MasterClockDivider::Div62,
        63 => MasterClockDivider::Div63,
        _ => panic!(),
    }
}
