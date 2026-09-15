#![no_std]
#![no_main]

//! Poll regular DFSDM conversions and print a measured voltage.
//!
//! Filter 0 (Sinc3, FOSR=128, IOSR=6) converts a serial (SPI) source on PC0,
//! clocked by the CKOUT output on PD3. Each raw accumulator is scaled through a
//! resistor-divider model and printed.

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::dfsdm::config::{CkoutDivider, DataRightShift, FilterOrder, FilterParameters, InternalSpiMode};
use embassy_stm32::dfsdm::{FilterConfig, Flt0, ResultRegular};
use embassy_stm32::peripherals::DFSDM1;
use embassy_stm32::{bind_interrupts, dfsdm};
use embassy_time::Timer;
use panic_probe as _;

bind_interrupts!(struct Irqs {
    DFSDM1_FLT0 => dfsdm::InterruptHandler<DFSDM1, Flt0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = embassy_stm32::Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.supply_config = SupplyConfig::DirectSMPS;
    }
    let p = embassy_stm32::init(config);
    info!("Hello World!");

    // DFSDM1 with a CKOUT output on PD3.
    let dfsdm1 = dfsdm::Dfsdm::new_ckout(
        p.DFSDM1,
        p.PD3,
        dfsdm::config::CkoutSource::System,
        CkoutDivider::try_from(5).expect("Divider wrong?"),
    );

    let (common, split) = dfsdm1.configure_pins(|creator| {
        (
            creator.ch0.none(),
            creator.ch1.datin(p.PC3),
            creator.ch2.none(),
            creator.ch3.none(),
            creator.ch4.datin(p.PC0),
            creator.ch5.none(),
            creator.ch6.none(),
            creator.ch7.none(),
        )
    });

    let channel4 = split
        .ch4
        .build_spi_int(&common, InternalSpiMode::SpiRising)
        .set_data_right_shift(DataRightShift::new(0))
        .enable();

    let flt_cfg = FilterConfig {
        filter_params: FilterParameters::try_new(FilterOrder::Sinc3 { fosr: 128 }, 6).expect("inside bounds"),
        ..Default::default()
    };
    let mut flt0 = split
        .flt0
        .build(&common, Irqs)
        .enable_no_dma(&channel4, [&channel4], &flt_cfg);

    flt0.regular.start_conversion();

    loop {
        if let Ok(ResultRegular { data, channel, pending }) = flt0.regular.try_get_result() {
            // AMC from 10% = -1V, 90% = 1V; outputs from -1 to 1; -1 = 0%, 1 = 100%.
            let normalized = (data as f32) / ((128_i32.pow(3) * 6_i32) as f32);

            let measured_volts = (normalized * 0.5) / 0.4;

            let voltage = (400_000.0 + 5_600.0) / 5_600.0 * measured_volts;

            info!("Data: {}, Channel:{}, Rpend: {}", voltage, channel, pending);
            Timer::after_millis(500).await;
            flt0.regular.start_conversion();
        }
    }
}
