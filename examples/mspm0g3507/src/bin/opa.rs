//! Example of using the OPA as a non-inverting PGA.
//!
//! An external voltage on PB19 (OPA1_IN0+, J3.23 on the LP-MSPM0G3507) is
//! amplified 4x onto PA16 (OPA1_OUT, J3.29) and sampled through the OPA's
//! internal ADC channel.

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_mspm0::adc::{self, Adc};
use embassy_mspm0::opa::{Gain, Opa};
use embassy_mspm0::{bind_interrupts, peripherals};
use embassy_time::Timer;
use {defmt_rtt as _, panic_halt as _};

bind_interrupts!(struct Irqs {
    ADC1 => adc::InterruptHandler<peripherals::ADC1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    info!("Hello world!");
    let p = embassy_mspm0::init(Default::default());

    let mut adc = Adc::new_async(p.ADC1, Default::default(), Irqs);
    let mut opa = Opa::new(p.OPA1, Default::default());

    // 4x gain from PB19, driving PA16. Use `opa.pga_int(...)` instead if the
    // output is only needed by the ADC.
    let out = opa.pga_ext(p.PB19, p.PA16, Gain::X4);

    loop {
        let raw = adc.read_channel(&out.adc_channel()).await;
        info!("OPA1 output: {}", raw);

        Timer::after_millis(500).await;
    }
}
