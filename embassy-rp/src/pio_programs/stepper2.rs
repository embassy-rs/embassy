use core::f64::consts::PI;
use embassy_rp::{
    Peri,
    gpio::Output,
    pio::{
        Common, Config, Direction, Instance, LoadedProgram, PioPin, StateMachine,
        program::pio_asm,
    },
    pio_programs::clock_divider::calculate_pio_clock_divider,
};

pub struct PioStepper2Program<'a, PIO: Instance> {
    prg: LoadedProgram<'a, PIO>,
}

impl<'a, PIO: Instance> PioStepper2Program<'a, PIO> {
    pub fn new(common: &mut Common<'a, PIO>) -> Self {
        let prg = pio_asm!(
            ".wrap_target",
            "pull block",
            "out x, 32",
            "loop:",
            "    set pins, 1  [31]",
            "    set pins, 0  [31]",
            "    jmp x-- loop",
            ".wrap"
        );

        let prg = common.load_program(&prg.program);

        Self { prg }
    }
}

pub struct Stepper2<'d, T: Instance, const SM: usize> {
    sm: StateMachine<'d, T, SM>,
    dir: Output<'d>,
}

impl<'d, T: Instance, const SM: usize> Stepper2<'d, T, SM> {
    pub fn new(
        pio: &mut Common<'d, T>,
        mut sm: StateMachine<'d, T, SM>,
        stp: Peri<'d, impl PioPin>,
        dir: Output<'d>,
        program: &PioStepperProgram<'d, T>,
    ) -> Self {
        let stp = pio.make_pio_pin(stp);
        sm.set_pin_dirs(Direction::Out, &[&stp]);

        let mut cfg = Config::default();
        cfg.set_set_pins(&[&stp]);

        cfg.clock_divider =
            calculate_pio_clock_divider(100 * 65);

        cfg.use_program(&program.prg, &[]);
        sm.set_config(&cfg);
        sm.set_enable(true);
        Self { sm, dir }
    }

    pub fn set_frequency(&mut self, freq: u32) {
        let clock_divider = calculate_pio_clock_divider(freq * 65);
        let divider_f32 = clock_divider.to_num::<f32>();
        assert!(divider_f32 <= 65536.0, "clkdiv must be <= 65536");
        assert!(divider_f32 >= 1.0, "clkdiv must be >= 1");

        self.sm.set_clock_divider(clock_divider);
        self.sm.clkdiv_restart();
    }
}
