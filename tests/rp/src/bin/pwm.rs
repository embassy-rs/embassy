#![no_std]
#![no_main]
#[cfg(feature = "rp2040")]
teleprobe_meta::target!(b"rpi-pico");
#[cfg(feature = "rp235xb")]
teleprobe_meta::target!(b"pimoroni-pico-plus-2");

use defmt::{assert, assert_eq, assert_ne, *};
use embassy_executor::Spawner;
use embassy_rp::gpio::{Input, Pull};
#[cfg(feature = "rp2040")]
use embassy_rp::gpio::{Level, Output};
use embassy_rp::pwm::{Config, InputMode, Pwm};
use embassy_time::Timer;
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
        let pwm = Pwm::new_free(p.PWM_SLICE3.reborrow(), cfg.clone());
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
            let pin1 = Input::new(p9.reborrow(), Pull::None);
            let _pwm = Pwm::new_output_a(p.PWM_SLICE3.reborrow(), p6.reborrow(), cfg.clone());
            Timer::after_millis(1).await;
            assert_eq!(pin1.is_low(), invert_a);
            Timer::after_millis(5).await;
            assert_eq!(pin1.is_high(), invert_a);
            Timer::after_millis(5).await;
            assert_eq!(pin1.is_low(), invert_a);
            Timer::after_millis(5).await;
            assert_eq!(pin1.is_high(), invert_a);
        }

        // Test output from B
        {
            let pin2 = Input::new(p11.reborrow(), Pull::None);
            let _pwm = Pwm::new_output_b(p.PWM_SLICE3.reborrow(), p7.reborrow(), cfg.clone());
            Timer::after_millis(1).await;
            assert_ne!(pin2.is_low(), invert_a);
            Timer::after_millis(5).await;
            assert_ne!(pin2.is_high(), invert_a);
            Timer::after_millis(5).await;
            assert_ne!(pin2.is_low(), invert_a);
            Timer::after_millis(5).await;
            assert_ne!(pin2.is_high(), invert_a);
        }

        // Test output from A+B
        {
            let pin1 = Input::new(p9.reborrow(), Pull::None);
            let pin2 = Input::new(p11.reborrow(), Pull::None);
            let _pwm = Pwm::new_output_ab(p.PWM_SLICE3.reborrow(), p6.reborrow(), p7.reborrow(), cfg.clone());
            Timer::after_millis(1).await;
            assert_eq!(pin1.is_low(), invert_a);
            assert_ne!(pin2.is_low(), invert_a);
            Timer::after_millis(5).await;
            assert_eq!(pin1.is_high(), invert_a);
            assert_ne!(pin2.is_high(), invert_a);
            Timer::after_millis(5).await;
            assert_eq!(pin1.is_low(), invert_a);
            assert_ne!(pin2.is_low(), invert_a);
            Timer::after_millis(5).await;
            assert_eq!(pin1.is_high(), invert_a);
            assert_ne!(pin2.is_high(), invert_a);
        }
    }

    // Test level-gated
    #[cfg(feature = "rp2040")]
    {
        let mut pin2 = Output::new(p11.reborrow(), Level::Low);
        let pwm = Pwm::new_input(
            p.PWM_SLICE3.reborrow(),
            p7.reborrow(),
            Pull::None,
            InputMode::Level,
            cfg.clone(),
        );
        assert_eq!(pwm.counter(), 0);
        Timer::after_millis(5).await;
        assert_eq!(pwm.counter(), 0);
        pin2.set_high();
        Timer::after_millis(1).await;
        pin2.set_low();
        let ctr = pwm.counter();
        info!("ctr: {}", ctr);
        assert!(ctr >= 1000);
        Timer::after_millis(1).await;
        assert_eq!(pwm.counter(), ctr);
    }

    // Test rising-gated
    #[cfg(feature = "rp2040")]
    {
        let mut pin2 = Output::new(p11.reborrow(), Level::Low);
        let pwm = Pwm::new_input(
            p.PWM_SLICE3.reborrow(),
            p7.reborrow(),
            Pull::None,
            InputMode::RisingEdge,
            cfg.clone(),
        );
        assert_eq!(pwm.counter(), 0);
        Timer::after_millis(5).await;
        assert_eq!(pwm.counter(), 0);
        pin2.set_high();
        Timer::after_millis(1).await;
        pin2.set_low();
        assert_eq!(pwm.counter(), 1);
        Timer::after_millis(1).await;
        assert_eq!(pwm.counter(), 1);
    }

    // Test falling-gated
    #[cfg(feature = "rp2040")]
    {
        let mut pin2 = Output::new(p11.reborrow(), Level::High);
        let pwm = Pwm::new_input(
            p.PWM_SLICE3.reborrow(),
            p7.reborrow(),
            Pull::None,
            InputMode::FallingEdge,
            cfg.clone(),
        );
        assert_eq!(pwm.counter(), 0);
        Timer::after_millis(5).await;
        assert_eq!(pwm.counter(), 0);
        pin2.set_low();
        Timer::after_millis(1).await;
        pin2.set_high();
        assert_eq!(pwm.counter(), 1);
        Timer::after_millis(1).await;
        assert_eq!(pwm.counter(), 1);
    }

    // pull-down
    {
        let pin2 = Input::new(p11.reborrow(), Pull::None);
        Pwm::new_input(
            p.PWM_SLICE3.reborrow(),
            p7.reborrow(),
            Pull::Down,
            InputMode::FallingEdge,
            cfg.clone(),
        );
        Timer::after_millis(1).await;
        assert!(pin2.is_low());
    }

    // pull-up
    {
        let pin2 = Input::new(p11.reborrow(), Pull::None);
        Pwm::new_input(
            p.PWM_SLICE3.reborrow(),
            p7.reborrow(),
            Pull::Up,
            InputMode::FallingEdge,
            cfg.clone(),
        );
        Timer::after_millis(1).await;
        assert!(pin2.is_high());
    }

    info!("Test OK");
    cortex_m::asm::bkpt();
}
