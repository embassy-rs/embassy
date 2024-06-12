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

const DEFAULT_MIN_PULSE_WIDTH: u64 = 1000; // uncalibrated default, the shortest duty cycle sent to a servo
const DEFAULT_MAX_PULSE_WIDTH: u64 = 2000; // uncalibrated default, the longest duty cycle sent to a servo
const DEFAULT_MAX_DEGREE_ROTATION: u64 = 160; // 160 degrees is typical
const REFRESH_INTERVAL: u64 = 20000; // The period of each cycle

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

pub struct ServoBuilder<'d, T: Instance, const SM: usize> {
    pwm: PwmPio<'d, T, SM>,
    period: Duration,
    min_pulse_width: Duration,
    max_pulse_width: Duration,
    max_degree_rotation: u64,
}

impl<'d, T: Instance, const SM: usize> ServoBuilder<'d, T, SM> {
    pub fn new(pwm: PwmPio<'d, T, SM>) -> Self {
        Self {
            pwm,
            period: Duration::from_micros(REFRESH_INTERVAL),
            min_pulse_width: Duration::from_micros(DEFAULT_MIN_PULSE_WIDTH),
            max_pulse_width: Duration::from_micros(DEFAULT_MAX_PULSE_WIDTH),
            max_degree_rotation: DEFAULT_MAX_DEGREE_ROTATION,
        }
    }

    pub fn set_period(mut self, duration: Duration) -> Self {
        self.period = duration;
        self
    }

    pub fn set_min_pulse_width(mut self, duration: Duration) -> Self {
        self.min_pulse_width = duration;
        self
    }

    pub fn set_max_pulse_width(mut self, duration: Duration) -> Self {
        self.max_pulse_width = duration;
        self
    }

    pub fn set_max_degree_rotation(mut self, degree: u64) -> Self {
        self.max_degree_rotation = degree;
        self
    }

    pub fn build(mut self) -> Servo<'d, T, SM> {
        self.pwm.set_period(self.period);
        Servo {
            pwm: self.pwm,
            min_pulse_width: self.min_pulse_width,
            max_pulse_width: self.max_pulse_width,
            max_degree_rotation: self.max_degree_rotation,
        }
    }
}

pub struct Servo<'d, T: Instance, const SM: usize> {
    pwm: PwmPio<'d, T, SM>,
    min_pulse_width: Duration,
    max_pulse_width: Duration,
    max_degree_rotation: u64,
}

impl<'d, T: Instance, const SM: usize> Servo<'d, T, SM> {
    pub fn start(&mut self) {
        self.pwm.start();
    }

    pub fn stop(&mut self) {
        self.pwm.stop();
    }

    pub fn write_time(&mut self, duration: Duration) {
        self.pwm.write(duration);
    }

    pub fn rotate(&mut self, degree: u64) {
        let degree_per_nano_second = (self.max_pulse_width.as_nanos() as u64 - self.min_pulse_width.as_nanos() as u64)
            / self.max_degree_rotation;
        let mut duration =
            Duration::from_nanos(degree * degree_per_nano_second + self.min_pulse_width.as_nanos() as u64);
        if self.max_pulse_width < duration {
            duration = self.max_pulse_width;
        }

        self.pwm.write(duration);
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let Pio { mut common, sm0, .. } = Pio::new(p.PIO0, Irqs);

    let pwm_pio = PwmPio::new(&mut common, sm0, p.PIN_1);
    let mut servo = ServoBuilder::new(pwm_pio)
        .set_max_degree_rotation(120) // Example of adjusting values for MG996R servo
        .set_min_pulse_width(Duration::from_micros(350)) // This value was detemined by a rough experiment.
        .set_max_pulse_width(Duration::from_micros(2600)) // Along with this value.
        .build();

    servo.start();

    let mut degree = 0;
    loop {
        degree = (degree + 1) % 120;
        servo.rotate(degree);
        Timer::after_millis(50).await;
    }
}
