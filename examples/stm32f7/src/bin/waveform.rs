#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(int_roundings)]

use defmt::*;
use embassy_executor::{Spawner, task};
use embassy_stm32::{
    gpio::{Output, Pin, Speed}, peripherals::TIM3, time::{khz, Hertz}, timer::{simple_pwm::{Ch3, Ch4, PwmPin}, Channel} 
};

use embassy_stm32::timer::simple_pwm::SimplePwm;

use embassy_stm32::gpio::OutputType;
use embassy_time::{Duration, Instant, Timer};

use {defmt_rtt as _, panic_probe as _};

const DSHOT_FRAME_SIZE: usize = 16; // 16 bits per frame

static mut DSHOT_FRAMES: [u16; DSHOT_FRAME_SIZE * 4 + 8] = [0; DSHOT_FRAME_SIZE * 4 + 8];

// Create DShot data packet
fn create_dshot_packet(throttle: u16, telemetry: bool) -> u16 {
    let mut packet = (throttle & 0x07FF) << 1; // 11-bit throttle
    if telemetry {
        packet |= 1; // Set telemetry bit if needed
    }
    let crc = ((packet ^ (packet >> 4) ^ (packet >> 8)) & 0x0F) as u16; // 4-bit CRC
    (packet << 4) | crc
}

// Prepare DShot frame as array of pulse widths for each bit
fn prepare_dshot_frame(packet: u16, one_third: u16, two_third: u16, ch: usize) {
    for i in 0..(DSHOT_FRAME_SIZE) {
        // Calculate each bit’s pulse width
        let is_one = (packet & (1 << (15 - i))) != 0;
        unsafe {
            DSHOT_FRAMES[i * 4 + ch] = if is_one {
                two_third // Set for ~62.5% high pulse
            } else {
                one_third // Set for ~37.5% high pulse
            };
        }
    }
}

#[task]
async fn blink(mut led: Output<'static>) {
    loop {
        led.set_high();
        info!("*blink*");
        Timer::after(Duration::from_millis(800)).await;
        led.set_low();
        Timer::after(Duration::from_millis(800)).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {

    let mut config = embassy_stm32::Config::default();

    {
        use embassy_stm32::rcc::*;
        config.rcc.hse = Some(Hse {
            freq: Hertz(8_000_000),
            mode: HseMode::Bypass,
        });
        config.rcc.pll_src = PllSource::HSE;
        config.rcc.pll = Some(Pll {
            prediv: PllPreDiv::DIV4,
            mul: PllMul::MUL216,
            divp: Some(PllPDiv::DIV2), // 8mhz / 4 * 216 / 2 = 216Mhz
            divq: Some(PllQDiv::DIV9), // 8mhz / 4 * 216 / 9 = 48Mhz
            divr: None,
        });
        config.rcc.ahb_pre = AHBPrescaler::DIV1;
        config.rcc.apb1_pre = APBPrescaler::DIV4;
        config.rcc.apb2_pre = APBPrescaler::DIV2;
        config.rcc.sys = Sysclk::PLL1_P;
    }

    let mut p = embassy_stm32::init(config);
    println!("hello");

    let blue = Output::new(p.PB7, Level::Low, Speed::Low);

    spawner.spawn(blink(blue)).unwrap();

    let pwm_pin1 = PwmPin::new_ch1(p.PA6, OutputType::PushPull);
    let pwm_pin2 = PwmPin::new_ch2(p.PC7, OutputType::PushPull);
    let pwm_pin3: PwmPin<'_, TIM3, Ch3> = PwmPin::new_ch3(p.PC8, OutputType::PushPull);
    let pwm_pin4: PwmPin<'_, TIM3, Ch4> = PwmPin::new_ch4(p.PC9, OutputType::PushPull);

    let mut pwm = SimplePwm::new(p.TIM3, Some(pwm_pin1), Some(pwm_pin2), Some(pwm_pin3), Some(pwm_pin4), khz(600), Default::default());

    let max = pwm.get_max_duty() as u16;
    let one_third = max / 3;
    let two_third = one_third * 2;
    pwm.enable_all_channels();

    let packet0 = create_dshot_packet(0, false);
    
    prepare_dshot_frame(packet0, one_third, two_third, 0);
    prepare_dshot_frame(packet0, one_third, two_third, 1);
    prepare_dshot_frame(packet0, one_third, two_third, 2);
    prepare_dshot_frame(packet0, one_third, two_third, 3);

    let inst = Instant::now();

    let start_ch = Channel::Ch1;
    let end_ch = Channel::Ch4; 

    unsafe {
        while (Instant::now() - inst) < Duration::from_secs(3) {
            pwm.waveform_up(&mut p.DMA1_CH2, start_ch, end_ch, &DSHOT_FRAMES).await;
            Timer::after(Duration::from_micros(50)).await;
        }
    }

    unsafe {
        for thr in (100..500).chain((100..500).rev()).cycle() {
            let packet = create_dshot_packet(thr, false);

            println!("{}", thr);

            prepare_dshot_frame(packet, one_third, two_third, 0);
            prepare_dshot_frame(packet, one_third, two_third, 1);
            prepare_dshot_frame(packet, one_third, two_third, 2);
            prepare_dshot_frame(packet, one_third, two_third, 3);

            for _ in 0..100 {
                pwm.waveform_up(&mut p.DMA1_CH2, start_ch, end_ch, &DSHOT_FRAMES).await;
                Timer::after(Duration::from_micros(50)).await;
            }
        }
    }
    
}
