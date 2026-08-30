#![no_std]
#![no_main]

use core::mem::MaybeUninit;
use core::ops::Div;

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::dfsdm::config_types::{CkoutDivider, FilterOrder, FilterParameters, InternalSpiMode};
use embassy_stm32::dfsdm::{
    Dfsdm, FilterConfig, Flt0, Flt1, InjectedTrigger, TransceiverConfig, TransceiverConfigOnline, TransceiverTrait,
};
use embassy_stm32::dma::{self, Channel, Request, Transfer, TransferOptions};
use embassy_stm32::gpio::{Level, Output, Speed};
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
        .new_parallel_dma(dfsdm::config_types::DataPackingModeReduced::Standard)
        .configure(&tcv_cfg, &tcv_cfg_online)
        .enable();

    let flt_cfg = FilterConfig {
        // filter_cfg: FilterParameters::try_new(FilterOrder::Sinc3 { fosr: 5 }, 4).expect("This is inside the bounds"),
        filter_params: FilterParameters::try_new(FilterOrder::Disabled, 32).expect("This is inside the bounds"),
    };

    let mut flt0 = split
        .flt0
        .configure(&flt_cfg)
        .assign_regular_transceiver(&ch_test)
        .enable();

    flt0.start_regular_conversion(); // Waiting for data now

    // Generate a 32-element array with a distinct pattern for each index
    // This ensures we aren't accidentally transferring the same word 32 times
    // or skipping/offsetting any bytes.
    let source: [u32; 32] = core::array::from_fn(|i| (i as u32).wrapping_mul(0x01010101) ^ 0xDEADBEEF);

    let mut dma_ch = Channel::new(p.MDMA_CH0, Irqs);
    let tfer_opts = TransferOptions::default();

    println!("DMA starting...");

    // No request number needed as MEM2MEM transfers on MDMA are software-controlled. Defaulting to 0.
    let tfer = unsafe { dma_ch.write_mem2mem::<u32, u32>(0, &source, ch_test.get_datinr_as_ptr(), tfer_opts) };
    tfer.await; // theoretically unnecessary

    println!("DMA finished.");

    let dfsdm_data: [i32; 32] = source.map(|x| {
        let signed_32 = x as i32; // 1. Reinterpret as i32
        (signed_32 << 16) >> 16 // 2. Shift left to drop bottom 8 bits, then arithmetic shift right to sign-extend the 24th bit
    });
    let integral: i64 = dfsdm_data.iter().map(|&x| x as i64).sum();
    println!("Manual integration: {}", integral);
    loop {
        // ch_test.write_sample_standard(10);
        if let Some((data, channel, rpend)) = flt0.try_get_regular_result() {
            println!("New regular 0: ");
            println!("Channel: {}", channel);
            println!("Value: {}", data);
            println!("Delayed: {}", rpend);

            return;
        }
    }
}
