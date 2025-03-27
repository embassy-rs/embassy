#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::dac::{DacCh1, DacCh2, ValueArray};
use embassy_stm32::mode::Async;
use embassy_stm32::pac::timer::vals::Mms;
use embassy_stm32::peripherals::{DAC1, TIM6, TIM7};
use embassy_stm32::rcc::frequency;
use embassy_stm32::time::Hertz;
use embassy_stm32::timer::low_level::Timer;
use embassy_stm32::Peri;
use micromath::F32Ext;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let config = embassy_stm32::Config::default();

    // Initialize the board and obtain a Peripherals instance
    let p: embassy_stm32::Peripherals = embassy_stm32::init(config);

    // Obtain two independent channels (p.DAC1 can only be consumed once, though!)
    let (dac_ch1, dac_ch2) = embassy_stm32::dac::Dac::new(p.DAC1, p.DMA1_CH3, p.DMA1_CH4, p.PA4, p.PA5).split();

    spawner.spawn(dac_task1(p.TIM6, dac_ch1)).ok();
    spawner.spawn(dac_task2(p.TIM7, dac_ch2)).ok();
}

#[embassy_executor::task]
async fn dac_task1(tim: Peri<'static, TIM6>, mut dac: DacCh1<'static, DAC1, Async>) {
    let data: &[u8; 256] = &calculate_array::<256>();

    info!("TIM6 frequency is {}", frequency::<TIM6>());
    const FREQUENCY: Hertz = Hertz::hz(200);

    // Compute the reload value such that we obtain the FREQUENCY for the sine
    let reload: u32 = (frequency::<TIM6>().0 / FREQUENCY.0) / data.len() as u32;

    // Depends on your clock and on the specific chip used, you may need higher or lower values here
    if reload < 10 {
        error!("Reload value {} below threshold!", reload);
    }

    dac.set_trigger(embassy_stm32::dac::TriggerSel::Tim6);
    dac.set_triggering(true);
    dac.enable();

    let tim = Timer::new(tim);
    tim.regs_basic().arr().modify(|w| w.set_arr(reload as u16 - 1));
    tim.regs_basic().cr2().modify(|w| w.set_mms(Mms::UPDATE));
    tim.regs_basic().cr1().modify(|w| {
        w.set_opm(false);
        w.set_cen(true);
    });

    debug!(
        "TIM6 Frequency {}, Target Frequency {}, Reload {}, Reload as u16 {}, Samples {}",
        frequency::<TIM6>(),
        FREQUENCY,
        reload,
        reload as u16,
        data.len()
    );

    // Loop technically not necessary if DMA circular mode is enabled
    loop {
        info!("Loop DAC1");
        dac.write(ValueArray::Bit8(data), true).await;
    }
}

#[embassy_executor::task]
async fn dac_task2(tim: Peri<'static, TIM7>, mut dac: DacCh2<'static, DAC1, Async>) {
    let data: &[u8; 256] = &calculate_array::<256>();

    info!("TIM7 frequency is {}", frequency::<TIM7>());

    const FREQUENCY: Hertz = Hertz::hz(600);
    let reload: u32 = (frequency::<TIM7>().0 / FREQUENCY.0) / data.len() as u32;

    if reload < 10 {
        error!("Reload value {} below threshold!", reload);
    }

    let tim = Timer::new(tim);
    tim.regs_basic().arr().modify(|w| w.set_arr(reload as u16 - 1));
    tim.regs_basic().cr2().modify(|w| w.set_mms(Mms::UPDATE));
    tim.regs_basic().cr1().modify(|w| {
        w.set_opm(false);
        w.set_cen(true);
    });

    dac.set_trigger(embassy_stm32::dac::TriggerSel::Tim7);
    dac.set_triggering(true);
    dac.enable();

    debug!(
        "TIM7 Frequency {}, Target Frequency {}, Reload {}, Reload as u16 {}, Samples {}",
        frequency::<TIM7>(),
        FREQUENCY,
        reload,
        reload as u16,
        data.len()
    );

    dac.write(ValueArray::Bit8(data), true).await;
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
