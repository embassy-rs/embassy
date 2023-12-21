//! This example shows how to use the PIO module in the RP2040 to implement a stepper motor driver
//! for a 5-wire stepper such as the 28BYJ-48. You can halt an ongoing rotation by dropping the future.

#![no_std]
#![no_main]
use core::mem::{self, MaybeUninit};

use defmt::info;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::PIO0;
use embassy_rp::pio::{Common, Config, Direction, Instance, InterruptHandler, Irq, Pio, PioPin, StateMachine};
use embassy_time::{with_timeout, Duration, Timer};
use fixed::traits::ToFixed;
use fixed::types::extra::U8;
use fixed::FixedU32;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

pub struct PioStepper<'d, T: Instance, const SM: usize> {
    irq: Irq<'d, T, SM>,
    sm: StateMachine<'d, T, SM>,
}

impl<'d, T: Instance, const SM: usize> PioStepper<'d, T, SM> {
    pub fn new(
        pio: &mut Common<'d, T>,
        mut sm: StateMachine<'d, T, SM>,
        irq: Irq<'d, T, SM>,
        pin0: impl PioPin,
        pin1: impl PioPin,
        pin2: impl PioPin,
        pin3: impl PioPin,
    ) -> Self {
        let prg = pio_proc::pio_asm!(
            "pull block",
            "mov x, osr",
            "pull block",
            "mov y, osr",
            "jmp !x end",
            "loop:",
            "jmp !osre step",
            "mov osr, y",
            "step:",
            "out pins, 4 [31]"
            "jmp x-- loop",
            "end:",
            "irq 0 rel"
        );
        let pin0 = pio.make_pio_pin(pin0);
        let pin1 = pio.make_pio_pin(pin1);
        let pin2 = pio.make_pio_pin(pin2);
        let pin3 = pio.make_pio_pin(pin3);
        sm.set_pin_dirs(Direction::Out, &[&pin0, &pin1, &pin2, &pin3]);
        let mut cfg = Config::default();
        cfg.set_out_pins(&[&pin0, &pin1, &pin2, &pin3]);
        cfg.clock_divider = (125_000_000 / (100 * 136)).to_fixed();
        cfg.use_program(&pio.load_program(&prg.program), &[]);
        sm.set_config(&cfg);
        sm.set_enable(true);
        Self { irq, sm }
    }

    // Set pulse frequency
    pub fn set_frequency(&mut self, freq: u32) {
        let clock_divider: FixedU32<U8> = (125_000_000 / (freq * 136)).to_fixed();
        assert!(clock_divider <= 65536, "clkdiv must be <= 65536");
        assert!(clock_divider >= 1, "clkdiv must be >= 1");
        T::PIO.sm(SM).clkdiv().write(|w| w.0 = clock_divider.to_bits() << 8);
        self.sm.clkdiv_restart();
    }

    // Full step, one phase
    pub async fn step(&mut self, steps: i32) {
        if steps > 0 {
            self.run(steps, 0b1000_0100_0010_0001_1000_0100_0010_0001).await
        } else {
            self.run(-steps, 0b0001_0010_0100_1000_0001_0010_0100_1000).await
        }
    }

    // Full step, two phase
    pub async fn step2(&mut self, steps: i32) {
        if steps > 0 {
            self.run(steps, 0b1001_1100_0110_0011_1001_1100_0110_0011).await
        } else {
            self.run(-steps, 0b0011_0110_1100_1001_0011_0110_1100_1001).await
        }
    }

    // Half step
    pub async fn step_half(&mut self, steps: i32) {
        if steps > 0 {
            self.run(steps, 0b1001_1000_1100_0100_0110_0010_0011_0001).await
        } else {
            self.run(-steps, 0b0001_0011_0010_0110_0100_1100_1000_1001).await
        }
    }

    async fn run(&mut self, steps: i32, pattern: u32) {
        self.sm.tx().wait_push(steps as u32).await;
        self.sm.tx().wait_push(pattern).await;
        let drop = OnDrop::new(|| {
            self.sm.clear_fifos();
            unsafe {
                self.sm.exec_instr(
                    pio::InstructionOperands::JMP {
                        address: 0,
                        condition: pio::JmpCondition::Always,
                    }
                    .encode(),
                );
            }
        });
        self.irq.wait().await;
        drop.defuse();
    }
}

struct OnDrop<F: FnOnce()> {
    f: MaybeUninit<F>,
}

impl<F: FnOnce()> OnDrop<F> {
    pub fn new(f: F) -> Self {
        Self { f: MaybeUninit::new(f) }
    }

    pub fn defuse(self) {
        mem::forget(self)
    }
}

impl<F: FnOnce()> Drop for OnDrop<F> {
    fn drop(&mut self) {
        unsafe { self.f.as_ptr().read()() }
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let Pio {
        mut common, irq0, sm0, ..
    } = Pio::new(p.PIO0, Irqs);

    let mut stepper = PioStepper::new(&mut common, sm0, irq0, p.PIN_4, p.PIN_5, p.PIN_6, p.PIN_7);
    stepper.set_frequency(120);
    loop {
        info!("CW full steps");
        stepper.step(1000).await;

        info!("CCW full steps, drop after 1 sec");
        if let Err(_) = with_timeout(Duration::from_secs(1), stepper.step(i32::MIN)).await {
            info!("Time's up!");
            Timer::after(Duration::from_secs(1)).await;
        }

        info!("CW half steps");
        stepper.step_half(1000).await;

        info!("CCW half steps");
        stepper.step_half(-1000).await;
    }
}
