#![no_std]
#![no_main]

use core::mem::MaybeUninit;
use core::ops::Div;

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::dfsdm::config_types::{CkoutDivider, FilterOrder, FilterParameters, InternalSpiMode};
use embassy_stm32::dfsdm::{
    Dfsdm, FilterConfig, Flt0, Flt1, InjectedTrigger, RingBufferedFilter, TransceiverConfig, TransceiverConfigOnline,
    TransceiverTrait,
};
use embassy_stm32::dma::{self, Channel, Request, Transfer, TransferOptions};
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::pac::dfsdm::regs::{Datinr, Rdatar};
use embassy_stm32::peripherals::{self, DFSDM1, MDMA};
use embassy_stm32::rcc::{self, Sysclk};
use embassy_stm32::spi::Spi;
use embassy_stm32::time::Hertz;
use embassy_stm32::{SharedData, bind_interrupts, dfsdm, pac};
use embassy_time::Timer;
use panic_probe as _;

#[unsafe(link_section = ".ram_d3.shared_data")]
static SHARED_DATA: MaybeUninit<SharedData> = MaybeUninit::uninit();

bind_interrupts! (struct Irqs{
    MDMA => dma::InterruptHandler<peripherals::MDMA_CH0>;
    DMA1_STREAM0 => dma::InterruptHandler<peripherals::DMA1_CH0>;
    DMA1_STREAM1 => dma::InterruptHandler<peripherals::DMA1_CH1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = embassy_stm32::Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hsi = Some(HSIPrescaler::Div1);
        config.rcc.csi = true;
        config.rcc.pll1 = Some(Pll {
            source: PllSource::Hsi,
            prediv: PllPreDiv::Div4,
            mul: PllMul::Mul50,
            divp: Some(PllDiv::Div2),
            divq: Some(PllDiv::Div8), // 100mhz
            divr: None,
        });
        config.rcc.sys = Sysclk::Pll1P; // 400 Mhz
        config.rcc.ahb_pre = AHBPrescaler::Div2; // 200 Mhz
        config.rcc.apb1_pre = APBPrescaler::Div2; // 100 Mhz
        config.rcc.apb2_pre = APBPrescaler::Div2; // 100 Mhz
        config.rcc.apb3_pre = APBPrescaler::Div2; // 100 Mhz
        config.rcc.apb4_pre = APBPrescaler::Div2; // 100 Mhz
        config.rcc.voltage_scale = VoltageScale::Scale1;
        config.rcc.supply_config = SupplyConfig::DirectSMPS;
    }

    //==================================================
    // Goal: Write fixed sequence into DFSDM via MDMA mem2mem and read the integration result
    //==================================================
    let p = embassy_stm32::init_primary(config, &SHARED_DATA);
    info!("Hello World!");

    // Start driver instantiation using DFSDM1 with a CKOUT pin on pin C2
    let dfsdm1 = dfsdm::Dfsdm::new(p.DFSDM1);

    let split = dfsdm1.configure_pins(|creator| {
        (
            creator.ch0.none(),
            creator.ch1.none(),
            creator.ch2.none(),
            creator.ch3.none(),
            creator.ch4.none(),
            creator.ch5.none(),
            creator.ch6.none(),
            creator.ch7.none(),
        )
    });
    let tcv_cfg = TransceiverConfig::default();
    let tcv_cfg_online = TransceiverConfigOnline::default();
    let ch_test = split
        .ch0
        .new_parallel_dma(&split.common, dfsdm::config_types::DataPackingModeReduced::Standard)
        .configure(&tcv_cfg, &tcv_cfg_online)
        .enable();

    let flt_cfg = FilterConfig {
        filter_params: FilterParameters::try_new(FilterOrder::Disabled, 32).expect("This is inside the bounds"),
        enable_injected_dma: true,
        enable_regular_dma: true,
        enable_continuous_regular: true,
        enable_fast_regular: false,
    };

    let mut flt0 = split
        .flt0
        .configure(&flt_cfg)
        .assign_regular_transceiver(&ch_test)
        .assign_injected_transceivers(&[&ch_test])
        .enable();

    let mut buffer_regular = [0u32; 32];
    let mut buffer_injected = [0u32; 32];

    flt0.start_regular_conversion(); // Waiting for data now
    flt0.start_injected_conversion(); // Waiting for data now
    let mut ring_buffered_filter_regular =
        RingBufferedFilter::new_regular(&flt0, p.DMA1_CH0, Irqs, &mut buffer_regular);
    let mut ring_buffered_filter_injected =
        RingBufferedFilter::new_injected(&flt0, p.DMA1_CH1, Irqs, &mut buffer_injected);

    ring_buffered_filter_regular.start();
    ring_buffered_filter_injected.start();

    // Generate a 32-element array with a distinct pattern for each index
    // This ensures we aren't accidentally transferring the same word 32 times
    // or skipping/offsetting any bytes.
    let source: [u32; 32 * 4] = core::array::from_fn(|i| (i as u32).wrapping_mul(0x01010101) ^ 0xDEADBEEF);

    let mut dma_ch = Channel::new(p.MDMA_CH0, Irqs);
    let tfer_opts = TransferOptions::default();

    println!("DMA starting...");

    // No request number needed as MEM2MEM transfers on MDMA are software-controlled. Defaulting to 0.
    let tfer: Transfer<'_> =
        unsafe { dma_ch.write_mem2mem::<u32, u32>(0, &source, ch_test.get_datinr_as_ptr(), tfer_opts) };
    tfer.await; // theoretically unnecessary

    println!("DMA finished.");

    let dfsdm_data = source.map(|x| {
        let signed_32 = x as i32; // 1. Reinterpret as i32
        (signed_32 << 16) >> 16 // 2. Shift left to drop bottom 8 bits, then arithmetic shift right to sign-extend the 24th bit
    });
    let integral: i64 = dfsdm_data.iter().map(|&x| x as i64).sum();
    println!("Manual integration: {}", integral);

    let mut result_buffer_regular = [0u32; 32];
    let mut result_buffer_injected = [0u32; 32];

    loop {
        let amount_regular = ring_buffered_filter_regular.read_latest(&mut result_buffer_regular);
        let amount_injected = ring_buffered_filter_injected.read_latest(&mut result_buffer_injected);
        if amount_regular > 0 {
            let a: Rdatar = pac::dfsdm::regs::Rdatar(result_buffer_regular[0]);

            println!("New regular, num; {} ", amount_regular);
            println!("Channel: {}", a.rdatach());
            println!("Value: {}", a.rdata());
            println!("Delayed: {}", a.rpend());
        }
        if amount_injected > 0 {
            let a: Rdatar = pac::dfsdm::regs::Rdatar(result_buffer_injected[0]);

            println!("New injected, num; {} ", amount_injected);
            println!("Channel: {}", a.rdatach());
            println!("Value: {}", a.rdata());
            println!("Delayed: {}", a.rpend());
        }
    }
}
