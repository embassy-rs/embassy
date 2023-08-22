#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
teleprobe_meta::target!(b"rpi-pico");

use defmt::{assert, assert_eq, assert_ne, *};
use embassy_executor::Spawner;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::pwm::{Config, InputMode, Pwm};
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut p = embassy_rp::init(Default::default());
    info!("Hello World!");

    // Connections on CI device: 6 -> 9, 7 -> 11
    let (mut p6, mut p7, mut p9, mut p11) = (p.PIN_6, p.PIN_7, p.PIN_9, p.PIN_11);

    let cfg = {
        let mut c = Config::default();
        c.divider = 125.into();
        c.top = 10000;
        c.compare_a = 5000;
        c.compare_b = 5000;
        c
    };

    // Test free-running clock
    {
        let pwm = Pwm::new_free(&mut p.PWM_CH3, cfg.clone());
        cortex_m::asm::delay(125);
        let ctr = pwm.counter();
        assert!(ctr > 0);
        assert!(ctr < 100);
        cortex_m::asm::delay(125);
        assert!(ctr < pwm.counter());
    }

    for invert_a in [false, true] {
        info!("free-running, invert A: {}", invert_a);
        let mut cfg = cfg.clone();
        cfg.invert_a = invert_a;
        cfg.invert_b = !invert_a;

        // Test output from A
        {
            let pin1 = Input::new(&mut p9, Pull::None);
            let _pwm = Pwm::new_output_a(&mut p.PWM_CH3, &mut p6, cfg.clone());
            Timer::after(Duration::from_millis(1)).await;
            assert_eq!(pin1.is_low(), invert_a);
            Timer::after(Duration::from_millis(5)).await;
            assert_eq!(pin1.is_high(), invert_a);
            Timer::after(Duration::from_millis(5)).await;
            assert_eq!(pin1.is_low(), invert_a);
            Timer::after(Duration::from_millis(5)).await;
            assert_eq!(pin1.is_high(), invert_a);
        }

        // Test output from B
        {
            let pin2 = Input::new(&mut p11, Pull::None);
            let _pwm = Pwm::new_output_b(&mut p.PWM_CH3, &mut p7, cfg.clone());
            Timer::after(Duration::from_millis(1)).await;
            assert_ne!(pin2.is_low(), invert_a);
            Timer::after(Duration::from_millis(5)).await;
            assert_ne!(pin2.is_high(), invert_a);
            Timer::after(Duration::from_millis(5)).await;
            assert_ne!(pin2.is_low(), invert_a);
            Timer::after(Duration::from_millis(5)).await;
            assert_ne!(pin2.is_high(), invert_a);
        }

        // Test output from A+B
        {
            let pin1 = Input::new(&mut p9, Pull::None);
            let pin2 = Input::new(&mut p11, Pull::None);
            let _pwm = Pwm::new_output_ab(&mut p.PWM_CH3, &mut p6, &mut p7, cfg.clone());
            Timer::after(Duration::from_millis(1)).await;
            assert_eq!(pin1.is_low(), invert_a);
            assert_ne!(pin2.is_low(), invert_a);
            Timer::after(Duration::from_millis(5)).await;
            assert_eq!(pin1.is_high(), invert_a);
            assert_ne!(pin2.is_high(), invert_a);
            Timer::after(Duration::from_millis(5)).await;
            assert_eq!(pin1.is_low(), invert_a);
            assert_ne!(pin2.is_low(), invert_a);
            Timer::after(Duration::from_millis(5)).await;
            assert_eq!(pin1.is_high(), invert_a);
            assert_ne!(pin2.is_high(), invert_a);
        }
    }

    // Test level-gated
    {
        let mut pin2 = Output::new(&mut p11, Level::Low);
        let pwm = Pwm::new_input(&mut p.PWM_CH3, &mut p7, InputMode::Level, cfg.clone());
        assert_eq!(pwm.counter(), 0);
        Timer::after(Duration::from_millis(5)).await;
        assert_eq!(pwm.counter(), 0);
        pin2.set_high();
        Timer::after(Duration::from_millis(1)).await;
        pin2.set_low();
        let ctr = pwm.counter();
        assert!(ctr >= 1000);
        Timer::after(Duration::from_millis(1)).await;
        assert_eq!(pwm.counter(), ctr);
    }

    // Test rising-gated
    {
        let mut pin2 = Output::new(&mut p11, Level::Low);
        let pwm = Pwm::new_input(&mut p.PWM_CH3, &mut p7, InputMode::RisingEdge, cfg.clone());
        assert_eq!(pwm.counter(), 0);
        Timer::after(Duration::from_millis(5)).await;
        assert_eq!(pwm.counter(), 0);
        pin2.set_high();
        Timer::after(Duration::from_millis(1)).await;
        pin2.set_low();
        assert_eq!(pwm.counter(), 1);
        Timer::after(Duration::from_millis(1)).await;
        assert_eq!(pwm.counter(), 1);
    }

    // Test falling-gated
    {
        let mut pin2 = Output::new(&mut p11, Level::High);
        let pwm = Pwm::new_input(&mut p.PWM_CH3, &mut p7, InputMode::FallingEdge, cfg.clone());
        assert_eq!(pwm.counter(), 0);
        Timer::after(Duration::from_millis(5)).await;
        assert_eq!(pwm.counter(), 0);
        pin2.set_low();
        Timer::after(Duration::from_millis(1)).await;
        pin2.set_high();
        assert_eq!(pwm.counter(), 1);
        Timer::after(Duration::from_millis(1)).await;
        assert_eq!(pwm.counter(), 1);
    }

    info!("Test OK");
    cortex_m::asm::bkpt();
}
