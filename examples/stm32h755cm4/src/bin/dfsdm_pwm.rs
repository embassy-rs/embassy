#![no_std]
#![no_main]

use core::mem::MaybeUninit;

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::SharedData;
use embassy_stm32::dfsdm::config_types::{CkoutDivider, InternalSpiMode};
use embassy_stm32::dfsdm::{
    self, Dfsdm, FilterConfig, Flt0, Flt1, InjectedTrigger, TransceiverConfig, TransceiverConfigOnline,
    TransceiverTrait,
};
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::peripherals::DFSDM1;
use embassy_stm32::rcc::{self};
use embassy_stm32::time::Hertz;
use panic_probe as _;

#[unsafe(link_section = ".ram_d3.shared_data")]
static SHARED_DATA: MaybeUninit<SharedData> = MaybeUninit::uninit();

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init_secondary(&SHARED_DATA);
    info!("Hello World!");
    // let mut led = Output::new(p.PB0, Level::High, Speed::Low);
    // let mut led = Output::new(p.PB3, Level::High, Speed::Low);
    // let mut led = Output::new(p.PE1, Level::High, Speed::Low);

    // Setup mic as left channel, data is valid at clock low, to rising clock edge samples signal
    let _mic_sel = Output::new(p.PA3, Level::Low, Speed::Low);

    let prescaler = rcc::frequency::<DFSDM1>() / Hertz::mhz(2);

    println!("Trying prescaler={}", prescaler);
    // Start driver instantiation using DFSDM1 with a CKOUT pin on pin C2
    let dfsdm1 = dfsdm::Dfsdm::new_ckout(
        p.DFSDM1,
        p.PC2,
        dfsdm::config_types::CkoutSource::System,
        CkoutDivider::try_from(prescaler as u16).expect("Divider wrong?"),
    );
    println!("Running with prescaler={}", prescaler);

    let split = dfsdm1.configure_pins(|creator| {
        (
            creator.ch0.none(),
            creator.ch1.datin(p.PC3),
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
    let channel_mic = split.ch1.new_spi_int(InternalSpiMode::SpiRising).enable();
    // let (mut ch2, mut ch3) = split.ch2.new_parallel_dma_dual(split.ch3);
    // let mut ch2 = ch2.configure(&tcv_cfg, &tcv_cfg_online).enable();
    // let mut ch3 = ch3.configure(&tcv_cfg, &tcv_cfg_online).enable();
    // split.ch6.new_parallel_dma_dual(split.ch7);

    //TODO the enable semantics should really also be linked to channel assignments in filters?

    // split.common.disable();  Common is already enabled from the split!

    let flt_cfg = FilterConfig::default();
    let mut flt0 = split
        .flt0
        .configure(&flt_cfg)
        .assign_regular_transceiver(&channel_mic)
        .enable();
    flt0.start_regular_conversion();

    loop {
        if let Some((data, channel, rpend)) = flt0.try_read_regular_result() {
            println!("New measurement: ");
            println!("Channel: {}", channel);
            println!("Value: {}", data);
            println!("Delayed: {}", rpend);
            flt0.start_regular_conversion();
        }

        // info!("led on!");
        // led.set_high();
        // Timer::after_millis(500).await;

        // info!("led off!");
        // led.set_low();
        // Timer::after_millis(500).await;
    }
}
