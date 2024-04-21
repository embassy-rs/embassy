//! This example shows how to create a pwm using the PIO module in the RP2040 chip.

#![no_std]
#![no_main]
use core::time::Duration;

use embassy_executor::Spawner;
use embassy_rp::gpio::Level;
use embassy_rp::peripherals::PIO0;
use embassy_rp::pio::{Common, Config, Direction, Instance, InterruptHandler, Pio, PioPin, StateMachine};
use embassy_rp::{bind_interrupts, clocks};
use embassy_time::Timer;
use pio::InstructionOperands;
use {defmt_rtt as _, panic_probe as _};

const REFRESH_INTERVAL: u64 = 20000;

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

pub fn to_pio_cycles(duration: Duration) -> u32 {
    (clocks::clk_sys_freq() / 1_000_000) / 3 * duration.as_micros() as u32 // parentheses are required to prevent overflow
}

pub struct PwmPio<'d, T: Instance, const SM: usize> {
    sm: StateMachine<'d, T, SM>,
}

impl<'d, T: Instance, const SM: usize> PwmPio<'d, T, SM> {
    pub fn new(pio: &mut Common<'d, T>, mut sm: StateMachine<'d, T, SM>, pin: impl PioPin) -> Self {
        let prg = pio_proc::pio_asm!(
            ".side_set 1 opt"
                "pull noblock    side 0"
                "mov x, osr"
                "mov y, isr"
            "countloop:"
                "jmp x!=y noset"
                "jmp skip        side 1"
            "noset:"
                "nop"
            "skip:"
                "jmp y-- countloop"
        );

        pio.load_program(&prg.program);
        let pin = pio.make_pio_pin(pin);
        sm.set_pins(Level::High, &[&pin]);
        sm.set_pin_dirs(Direction::Out, &[&pin]);

        let mut cfg = Config::default();
        cfg.use_program(&pio.load_program(&prg.program), &[&pin]);

        sm.set_config(&cfg);

        Self { sm }
    }

    pub fn start(&mut self) {
        self.sm.set_enable(true);
    }

    pub fn stop(&mut self) {
        self.sm.set_enable(false);
    }

    pub fn set_period(&mut self, duration: Duration) {
        let is_enabled = self.sm.is_enabled();
        while !self.sm.tx().empty() {} // Make sure that the queue is empty
        self.sm.set_enable(false);
        self.sm.tx().push(to_pio_cycles(duration));
        unsafe {
            self.sm.exec_instr(
                InstructionOperands::PULL {
                    if_empty: false,
                    block: false,
                }
                .encode(),
            );
            self.sm.exec_instr(
                InstructionOperands::OUT {
                    destination: ::pio::OutDestination::ISR,
                    bit_count: 32,
                }
                .encode(),
            );
        };
        if is_enabled {
            self.sm.set_enable(true) // Enable if previously enabled
        }
    }

    pub fn set_level(&mut self, level: u32) {
        self.sm.tx().push(level);
    }

    pub fn write(&mut self, duration: Duration) {
        self.set_level(to_pio_cycles(duration));
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let Pio { mut common, sm0, .. } = Pio::new(p.PIO0, Irqs);

    // Note that PIN_25 is the led pin on the Pico
    let mut pwm_pio = PwmPio::new(&mut common, sm0, p.PIN_25);
    pwm_pio.set_period(Duration::from_micros(REFRESH_INTERVAL));
    pwm_pio.start();

    let mut duration = 0;
    loop {
        duration = (duration + 1) % 1000;
        pwm_pio.write(Duration::from_micros(duration));
        Timer::after_millis(1).await;
    }
}
