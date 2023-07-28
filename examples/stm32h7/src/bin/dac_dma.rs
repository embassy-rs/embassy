#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::dac::{DacChannel, ValueArray};
use embassy_stm32::pac::timer::vals::{Mms, Opm};
use embassy_stm32::peripherals::{TIM6, TIM7};
use embassy_stm32::rcc::low_level::RccPeripheral;
use embassy_stm32::time::{mhz, Hertz};
use embassy_stm32::timer::low_level::Basic16bitInstance;
use micromath::F32Ext;
use {defmt_rtt as _, panic_probe as _};

pub type Dac1Type =
    embassy_stm32::dac::DacCh1<'static, embassy_stm32::peripherals::DAC1, embassy_stm32::peripherals::DMA1_CH3>;

pub type Dac2Type =
    embassy_stm32::dac::DacCh2<'static, embassy_stm32::peripherals::DAC1, embassy_stm32::peripherals::DMA1_CH4>;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut config = embassy_stm32::Config::default();
    config.rcc.sys_ck = Some(mhz(400));
    config.rcc.hclk = Some(mhz(100));
    config.rcc.pll1.q_ck = Some(mhz(100));

    // Initialize the board and obtain a Peripherals instance
    let p: embassy_stm32::Peripherals = embassy_stm32::init(config);

    // Obtain two independent channels (p.DAC1 can only be consumed once, though!)
    let (dac_ch1, dac_ch2) = embassy_stm32::dac::Dac::new(p.DAC1, p.DMA1_CH3, p.DMA1_CH4, p.PA4, p.PA5).split();

    spawner.spawn(dac_task1(dac_ch1)).ok();
    spawner.spawn(dac_task2(dac_ch2)).ok();
}

#[embassy_executor::task]
async fn dac_task1(mut dac: Dac1Type) {
    let data: &[u8; 256] = &calculate_array::<256>();

    info!("TIM6 frequency is {}", TIM6::frequency());
    const FREQUENCY: Hertz = Hertz::hz(200);

    // Compute the reload value such that we obtain the FREQUENCY for the sine
    let reload: u32 = (TIM6::frequency().0 / FREQUENCY.0) / data.len() as u32;

    // Depends on your clock and on the specific chip used, you may need higher or lower values here
    if reload < 10 {
        error!("Reload value {} below threshold!", reload);
    }

    dac.select_trigger(embassy_stm32::dac::Ch1Trigger::Tim6).unwrap();
    dac.enable_channel().unwrap();

    TIM6::enable();
    TIM6::regs().arr().modify(|w| w.set_arr(reload as u16 - 1));
    TIM6::regs().cr2().modify(|w| w.set_mms(Mms::UPDATE));
    TIM6::regs().cr1().modify(|w| {
        w.set_opm(Opm::DISABLED);
        w.set_cen(true);
    });

    debug!(
        "TIM6 Frequency {}, Target Frequency {}, Reload {}, Reload as u16 {}, Samples {}",
        TIM6::frequency(),
        FREQUENCY,
        reload,
        reload as u16,
        data.len()
    );

    // Loop technically not necessary if DMA circular mode is enabled
    loop {
        info!("Loop DAC1");
        if let Err(e) = dac.write(ValueArray::Bit8(data), true).await {
            error!("Could not write to dac: {}", e);
        }
    }
}

#[embassy_executor::task]
async fn dac_task2(mut dac: Dac2Type) {
    let data: &[u8; 256] = &calculate_array::<256>();

    info!("TIM7 frequency is {}", TIM7::frequency());

    const FREQUENCY: Hertz = Hertz::hz(600);
    let reload: u32 = (TIM7::frequency().0 / FREQUENCY.0) / data.len() as u32;

    if reload < 10 {
        error!("Reload value {} below threshold!", reload);
    }

    TIM7::enable();
    TIM7::regs().arr().modify(|w| w.set_arr(reload as u16 - 1));
    TIM7::regs().cr2().modify(|w| w.set_mms(Mms::UPDATE));
    TIM7::regs().cr1().modify(|w| {
        w.set_opm(Opm::DISABLED);
        w.set_cen(true);
    });

    dac.select_trigger(embassy_stm32::dac::Ch2Trigger::Tim7).unwrap();

    debug!(
        "TIM7 Frequency {}, Target Frequency {}, Reload {}, Reload as u16 {}, Samples {}",
        TIM7::frequency(),
        FREQUENCY,
        reload,
        reload as u16,
        data.len()
    );

    if let Err(e) = dac.write(ValueArray::Bit8(data), true).await {
        error!("Could not write to dac: {}", e);
    }
}

fn to_sine_wave(v: u8) -> u8 {
    if v >= 128 {
        // top half
        let r = 3.14 * ((v - 128) as f32 / 128.0);
        (r.sin() * 128.0 + 127.0) as u8
    } else {
        // bottom half
        let r = 3.14 + 3.14 * (v as f32 / 128.0);
        (r.sin() * 128.0 + 127.0) as u8
    }
}

fn calculate_array<const N: usize>() -> [u8; N] {
    let mut res = [0; N];
    let mut i = 0;
    while i < N {
        res[i] = to_sine_wave(i as u8);
        i += 1;
    }
    res
}
