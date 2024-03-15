//! CORDIC co-processor

use crate::peripherals;
use embassy_hal_internal::{into_ref, Peripheral, PeripheralRef};

mod enums;
pub use enums::*;

pub mod utils;

pub(crate) mod sealed;

/// Low-level CORDIC access.
#[cfg(feature = "unstable-pac")]
pub mod low_level {
    pub use super::sealed::*;
}

/// CORDIC driver
pub struct Cordic<'d, T: Instance> {
    cordic: PeripheralRef<'d, T>,
    config: Config,
    //state: State,
}

/// CORDIC instance trait
pub trait Instance: sealed::Instance + Peripheral<P = Self> + crate::rcc::RccPeripheral {}

/// CORDIC configuration
pub struct Config {
    function: Function,
    precision: Precision,
    scale: Scale,
    mode: Mode,
    first_result: bool,
}

// CORDIC running state
//struct State {
//    input_buf: [u32; 8],
//    buf_len: usize,
//}

impl Config {
    /// Create a config for Cordic driver
    pub fn new(function: Function, precision: Precision, scale: Option<Scale>, mode: Mode, first_result: bool) -> Self {
        Self {
            function,
            precision,
            scale: scale.unwrap_or_default(),
            mode,
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
            // state: State {
            //     input_buf: [0u32; 8],
            //     buf_len: 0,
            // },
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

        //self.state.input_buf.fill(0u32);
    }

    /// Run a CORDIC calculation
    pub fn calc_32bit(&mut self, arg1s: &[f64], arg2s: Option<&[f64]>, output: &mut [f64]) -> usize {
        match self.config.mode {
            Mode::ZeroOverhead => {
                if arg2s.is_none() {
                    self.cordic.set_argument_count(Count::One);

                    self.cordic.set_result_count(if self.config.first_result {
                        if output.len() < arg1s.len() {
                            panic!("Output buf length is not long enough")
                        }
                        Count::One
                    } else {
                        if output.len() < 2 * arg1s.len() {
                            panic!("Output buf length is not long enough")
                        }
                        Count::Two
                    });

                    let mut cnt = 0;

                    for &arg in arg1s.iter() {
                        self.cordic.write_argument(utils::f64_to_q1_31(arg));
                        output[cnt] = utils::q1_31_to_f64(self.cordic.read_result());
                        cnt += 1;
                    }

                    cnt
                } else {
                    todo!()
                }
            }
            Mode::Interrupt => todo!(),
            Mode::Dma => todo!(),
        }
    }
}

impl<'d, T: Instance> Drop for Cordic<'d, T> {
    fn drop(&mut self) {
        T::disable();
    }
}

foreach_interrupt!(
    ($inst:ident, cordic, CORDIC, GLOBAL, $irq:ident) => {
        impl Instance for peripherals::$inst {
        }

        impl sealed::Instance for peripherals::$inst {
            fn regs() -> crate::pac::cordic::Cordic {
                crate::pac::$inst
            }
        }
    };
);
