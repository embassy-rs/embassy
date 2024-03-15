//! CORDIC co-processor

use crate::peripherals;
use embassy_hal_internal::{into_ref, Peripheral, PeripheralRef};

mod enums;
pub use enums::*;

pub mod utils;

pub(crate) mod sealed;

// length of pre-allocated [u32] memory for CORDIC input,
// length should be multiple of 2
const INPUT_BUF_LEN: usize = 8;

/// Low-level CORDIC access.
#[cfg(feature = "unstable-pac")]
pub mod low_level {
    pub use super::sealed::*;
}

/// CORDIC driver
pub struct Cordic<'d, T: Instance> {
    cordic: PeripheralRef<'d, T>,
    config: Config,
    state: State,
}

/// CORDIC instance trait
pub trait Instance: sealed::Instance + Peripheral<P = Self> + crate::rcc::RccPeripheral {}

/// CORDIC configuration
pub struct Config {
    mode: Mode,
    function: Function,
    precision: Precision,
    scale: Scale,
    first_result: bool,
}

// CORDIC running state
struct State {
    input_buf: [u32; INPUT_BUF_LEN],
    buf_index: usize,
}

impl Config {
    /// Create a config for Cordic driver
    pub fn new(
        mode: Mode,
        function: Function,
        precision: Option<Precision>,
        scale: Option<Scale>,
        first_result: bool,
    ) -> Self {
        Self {
            mode,
            function,
            precision: precision.unwrap_or_default(),
            scale: scale.unwrap_or_default(),
            first_result,
        }
    }

    fn check_scale(&self) -> bool {
        let scale_raw = self.scale as u8;

        match self.function {
            Function::Cos | Function::Sin | Function::Phase | Function::Modulus => 0 == scale_raw,
            Function::Arctan => (0..=7).contains(&scale_raw),
            Function::Cosh | Function::Sinh | Function::Arctanh => 1 == scale_raw,
            Function::Ln => (1..=4).contains(&scale_raw),
            Function::Sqrt => (0..=2).contains(&scale_raw),
        }
    }
}

// common method
impl<'d, T: Instance> Cordic<'d, T> {
    /// Create a Cordic driver instance
    ///
    /// Note:  
    /// If you need a periperhal -> CORDIC -> peripehral mode,  
    /// you may want to set Cordic into [Mode::ZeroOverhead] mode, and add extra arguemnts with [Self::extra_config]
    pub fn new(cordic: impl Peripheral<P = T> + 'd, config: Config) -> Self {
        T::enable_and_reset();

        into_ref!(cordic);

        if !config.check_scale() {
            panic!("Scale value is not compatible with Function")
        }

        let mut instance = Self {
            cordic,
            config,
            state: State {
                input_buf: [0u32; 8],
                buf_index: 0,
            },
        };

        instance.reconfigure();

        instance
    }

    /// Set a new config for Cordic driver  
    pub fn set_config(&mut self, config: Config) {
        self.config = config;
        self.reconfigure();
    }

    /// Set extra config for data count and data width.
    pub fn extra_config(&mut self, arg_cnt: Count, arg_width: Width, res_width: Width) {
        let peri = &self.cordic;
        peri.set_argument_count(arg_cnt);
        peri.set_data_width(arg_width, res_width);
    }

    fn reconfigure(&mut self) {
        let peri = &self.cordic;
        let config = &self.config;

        if peri.ready_to_read() {
            warn!("At least 1 result hasn't been read, reconfigure will cause DATA LOST");
        };

        peri.disable_irq();
        peri.disable_write_dma();
        peri.disable_read_dma();

        // clean RRDY flag
        while peri.ready_to_read() {
            peri.read_result();
        }

        peri.set_func(config.function);
        peri.set_precision(config.precision);
        peri.set_scale(config.scale);

        if config.first_result {
            peri.set_result_count(Count::One)
        } else {
            peri.set_result_count(Count::Two)
        }

        match config.mode {
            Mode::ZeroOverhead => (),
            Mode::Interrupt => {
                peri.enable_irq();
            }
            Mode::Dma => {
                peri.enable_write_dma();
                peri.enable_read_dma();
            }
        }

        self.state.input_buf.fill(0u32);
        self.state.buf_index = 0;
    }
}

impl<'d, T: Instance> Drop for Cordic<'d, T> {
    fn drop(&mut self) {
        T::disable();
    }
}

// q1.31 related
impl<'d, T: Instance> Cordic<'d, T> {
    /// Run a CORDIC calculation
    pub fn calc_32bit(&mut self, arg1s: &[f64], arg2s: Option<&[f64]>, output: &mut [f64]) -> usize {
        let peri = &self.cordic;
        let config = &self.config;

        assert!(
            match config.first_result {
                true => output.len() >= arg1s.len(),
                false => output.len() >= 2 * arg1s.len(),
            },
            "Output buf length is not long enough"
        );

        self.check_input_f64(arg1s, arg2s);

        peri.set_result_count(if config.first_result { Count::One } else { Count::Two });
        peri.set_data_width(Width::Bits32, Width::Bits32);

        let state = &mut self.state;

        let mut output_count = 0;

        let mut consumed_input_len = 0;

        match config.mode {
            Mode::ZeroOverhead => {
                // put double input into cordic
                if arg2s.is_some() && !arg2s.unwrap().is_empty() {
                    let arg2s = arg2s.unwrap();

                    peri.set_argument_count(Count::Two);

                    let double_value = arg1s.iter().zip(arg2s);
                    consumed_input_len = double_value.len();

                    for (arg1, arg2) in double_value {
                        // if input_buf is full, send values to cordic
                        if state.buf_index == INPUT_BUF_LEN - 1 {
                            for arg in state.input_buf.chunks(2) {
                                peri.write_argument(arg[0]);
                                peri.write_argument(arg[1]);

                                output[output_count] = utils::q1_31_to_f64(peri.read_result());
                                output_count += 1;

                                if !config.first_result {
                                    output[output_count] = utils::q1_31_to_f64(peri.read_result());
                                    output_count += 1;
                                }
                            }

                            state.buf_index = 0;
                        }

                        for &&arg in [arg1, arg2].iter() {
                            state.input_buf[state.buf_index] = utils::f64_to_q1_31(arg);
                            state.buf_index += 1;
                        }
                    }

                    // put left paired args into cordic
                    if state.buf_index > 0 {
                        for arg in state.input_buf[..state.buf_index].chunks(2) {
                            peri.write_argument(arg[0]);
                            peri.write_argument(arg[1]);

                            output[output_count] = utils::q1_31_to_f64(peri.read_result());
                            output_count += 1;

                            if !config.first_result {
                                output[output_count] = utils::q1_31_to_f64(peri.read_result());
                                output_count += 1;
                            }
                        }

                        state.buf_index = 0;
                    }
                }

                // put single input into cordic
                let input_left = &arg1s[consumed_input_len..];

                if !input_left.is_empty() {
                    peri.set_argument_count(Count::One);

                    for &arg in input_left.iter() {
                        peri.write_argument(utils::f64_to_q1_31(arg));

                        output[output_count] = utils::q1_31_to_f64(peri.read_result());
                        output_count += 1;

                        if !config.first_result {
                            output[output_count] = utils::q1_31_to_f64(peri.read_result());
                            output_count += 1;
                        }
                    }
                }

                output_count
            }
            Mode::Interrupt => todo!(),
            Mode::Dma => todo!(),
        }
    }

    fn check_input_f64(&self, arg1s: &[f64], arg2s: Option<&[f64]>) {
        let config = &self.config;

        use Function::*;

        // check SCALE value
        match config.function {
            Cos | Sin | Phase | Modulus => assert!(Scale::A1_R1 == config.scale, "SCALE should be 0"),
            Arctan => assert!(
                (0..=7).contains(&(config.scale as u8)),
                "SCALE should be: 0 <= SCALE <= 7"
            ),
            Cosh | Sinh | Arctanh => assert!(Scale::A1o2_R2 == config.scale, "SCALE should be 1"),

            Ln => assert!(
                (1..=4).contains(&(config.scale as u8)),
                "SCALE should be: 1 <= SCALE <= 4"
            ),
            Sqrt => assert!(
                (0..=2).contains(&(config.scale as u8)),
                "SCALE should be: 0 <= SCALE <= 2"
            ),
        }

        // check ARG1 value
        match config.function {
            Cos | Sin | Phase | Modulus | Arctan => {
                assert!(
                    arg1s.iter().all(|v| (-1.0..=1.0).contains(v)),
                    "ARG1 should be: -1 <= ARG1 <= 1"
                );
            }

            Cosh | Sinh => assert!(
                arg1s.iter().all(|v| (-0.559..=0.559).contains(v)),
                "ARG1 should be: -0.559 <= ARG1 <= 0.559"
            ),

            Arctanh => assert!(
                arg1s.iter().all(|v| (-0.403..=0.403).contains(v)),
                "ARG1 should be: -0.403 <= ARG1 <= 0.403"
            ),

            Ln => {
                match config.scale {
                    Scale::A1o2_R2 => assert!(
                        arg1s.iter().all(|v| (0.05354..0.5).contains(v)),
                        "When SCALE set to 1, ARG1 should be: 0.05354 <= ARG1 < 0.5"
                    ),
                    Scale::A1o4_R4 => assert!(
                        arg1s.iter().all(|v| (0.25..0.75).contains(v)),
                        "When SCALE set to 2, ARG1 should be: 0.25 <= ARG1 < 0.75"
                    ),
                    Scale::A1o8_R8 => assert!(
                        arg1s.iter().all(|v| (0.375..0.875).contains(v)),
                        "When SCALE set to 3, ARG1 should be: 0.375 <= ARG1 < 0.875"
                    ),
                    Scale::A1o16_R16 => assert!(
                        arg1s.iter().all(|v| (0.4375f64..0.584f64).contains(v)),
                        "When SCALE set to 4, ARG1 should be: 0.4375 <= ARG1 < 0.584"
                    ),
                    _ => unreachable!(),
                };
            }

            Function::Sqrt => match config.scale {
                Scale::A1_R1 => assert!(
                    arg1s.iter().all(|v| (0.027..0.75).contains(v)),
                    "When SCALE set to 0, ARG1 should be: 0.027 <= ARG1 < 0.75"
                ),
                Scale::A1o2_R2 => assert!(
                    arg1s.iter().all(|v| (0.375..0.875).contains(v)),
                    "When SCALE set to 1, ARG1 should be: 0.375 <= ARG1 < 0.875"
                ),
                Scale::A1o4_R4 => assert!(
                    arg1s.iter().all(|v| (0.4375..0.585).contains(v)),
                    "When SCALE set to 2, ARG1 should be: 0.4375  <= ARG1 < 0.585"
                ),
                _ => unreachable!(),
            },
        }

        // check ARG2 value
        if let Some(arg2s) = arg2s {
            match config.function {
                Cos | Sin => assert!(
                    arg2s.iter().all(|v| (0.0..=1.0).contains(v)),
                    "ARG2 should be: 0 <= ARG2 <= 1"
                ),

                Phase | Modulus => assert!(
                    arg2s.iter().all(|v| (-1.0..=1.0).contains(v)),
                    "ARG2 should be: -1 <= ARG2 <= 1"
                ),

                _ => (),
            }
        }
    }
}

foreach_interrupt!(
    ($inst:ident, cordic, $block:ident, GLOBAL, $irq:ident) => {
        impl Instance for peripherals::$inst {
        }

        impl sealed::Instance for peripherals::$inst {
            fn regs() -> crate::pac::cordic::Cordic {
                crate::pac::$inst
            }
        }
    };
);
