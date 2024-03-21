//! coordinate rotation digital computer (CORDIC)

use embassy_hal_internal::drop::OnDrop;
use embassy_hal_internal::{into_ref, Peripheral, PeripheralRef};

use crate::{dma, peripherals};

mod enums;
pub use enums::*;

mod errors;
pub use errors::*;

pub mod utils;

pub(crate) mod sealed;

/// Low-level CORDIC access.
#[cfg(feature = "unstable-pac")]
pub mod low_level {
    pub use super::sealed::*;
}

const INPUT_BUF_MAX_LEN: usize = 16;

/// CORDIC driver
pub struct Cordic<'d, T: Instance> {
    peri: PeripheralRef<'d, T>,
    config: Config,
}

/// CORDIC instance trait
pub trait Instance: sealed::Instance + Peripheral<P = Self> + crate::rcc::RccPeripheral {}

/// CORDIC configuration
#[derive(Debug)]
pub struct Config {
    function: Function,
    precision: Precision,
    scale: Scale,
    res1_only: bool,
}

impl Config {
    /// Create a config for Cordic driver
    pub fn new(function: Function, precision: Precision, scale: Scale, res1_only: bool) -> Result<Self, CordicError> {
        let config = Self {
            function,
            precision,
            scale,
            res1_only,
        };

        config.check_scale()?;

        Ok(config)
    }

    fn check_scale(&self) -> Result<(), ConfigError> {
        use Function::*;

        let scale_raw = self.scale as u8;

        let err_range = match self.function {
            Cos | Sin | Phase | Modulus if !(0..=0).contains(&scale_raw) => Some([0, 0]),

            Arctan if !(0..=7).contains(&scale_raw) => Some([0, 7]),

            Cosh | Sinh | Arctanh if !(1..=1).contains(&scale_raw) => Some([1, 1]),

            Ln if !(1..=4).contains(&scale_raw) => Some([1, 4]),

            Sqrt if !(0..=2).contains(&scale_raw) => Some([0, 2]),

            Cos | Sin | Phase | Modulus | Arctan | Cosh | Sinh | Arctanh | Ln | Sqrt => None,
        };

        if let Some(range) = err_range {
            Err(ConfigError {
                func: self.function,
                scale_range: range,
            })
        } else {
            Ok(())
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
    pub fn new(peri: impl Peripheral<P = T> + 'd, config: Config) -> Self {
        T::enable_and_reset();

        into_ref!(peri);

        let mut instance = Self { peri, config };

        instance.reconfigure();

        instance
    }

    /// Set a new config for Cordic driver  
    pub fn set_config(&mut self, config: Config) {
        self.config = config;
        self.reconfigure();
    }

    /// Set extra config for data count and data width.
    pub fn extra_config(&mut self, arg_cnt: AccessCount, arg_width: Width, res_width: Width) {
        self.peri.set_argument_count(arg_cnt);
        self.peri.set_data_width(arg_width, res_width);
    }

    fn reconfigure(&mut self) {
        self.peri.set_func(self.config.function);
        self.peri.set_precision(self.config.precision);
        self.peri.set_scale(self.config.scale);

        // we don't set NRES in here, but to make sure NRES is set each time user call "calc"-ish functions,
        // since each "calc"-ish functions can have different ARGSIZE and RESSIZE, thus NRES should be change accrodingly.
    }

    async fn launch_a_dma_transfer(
        &mut self,
        write_dma: impl Peripheral<P = impl WriteDma<T>>,
        read_dma: impl Peripheral<P = impl ReadDma<T>>,
        input: &[u32],
        output: &mut [u32],
    ) {
        into_ref!(write_dma, read_dma);

        let write_req = write_dma.request();
        let read_req = read_dma.request();

        self.peri.enable_write_dma();
        self.peri.enable_read_dma();

        let _on_drop = OnDrop::new(|| {
            self.peri.disable_write_dma();
            self.peri.disable_read_dma();
        });

        unsafe {
            let write_transfer = dma::Transfer::new_write(
                &mut write_dma,
                write_req,
                input,
                T::regs().wdata().as_ptr() as *mut _,
                Default::default(),
            );

            let read_transfer = dma::Transfer::new_read(
                &mut read_dma,
                read_req,
                T::regs().rdata().as_ptr() as *mut _,
                output,
                Default::default(),
            );

            embassy_futures::join::join(write_transfer, read_transfer).await;
        }
    }
}

impl<'d, T: Instance> Drop for Cordic<'d, T> {
    fn drop(&mut self) {
        T::disable();
    }
}

// q1.31 related
impl<'d, T: Instance> Cordic<'d, T> {
    /// Run a blocking CORDIC calculation in q1.31 format
    pub fn blocking_calc_32bit(
        &mut self,
        arg1s: &[f64],
        arg2s: Option<&[f64]>,
        output: &mut [f64],
    ) -> Result<usize, CordicError> {
        if arg1s.is_empty() {
            return Ok(0);
        }

        let output_length_enough = match self.config.res1_only {
            true => output.len() >= arg1s.len(),
            false => output.len() >= 2 * arg1s.len(),
        };

        if !output_length_enough {
            return Err(CordicError::OutputLengthNotEnough);
        }

        self.check_input_f64(arg1s, arg2s)?;

        self.peri.set_result_count(if self.config.res1_only {
            AccessCount::One
        } else {
            AccessCount::Two
        });

        self.peri.set_data_width(Width::Bits32, Width::Bits32);

        let mut output_count = 0;

        let mut consumed_input_len = 0;

        //
        // handle 2 input args calculation
        //

        if arg2s.is_some() && !arg2s.unwrap().is_empty() {
            let arg2s = arg2s.unwrap();

            self.peri.set_argument_count(AccessCount::Two);

            // Skip 1st value from arg1s, this value will be manually "preload" to cordic, to make use of cordic preload function.
            // And we preserve last value from arg2s, since it need to manually write to cordic, and read the result out.
            let double_input = arg1s.iter().skip(1).zip(&arg2s[..arg2s.len() - 1]);
            // Since we preload 1st value from arg1s, the consumed input length is double_input length + 1.
            consumed_input_len = double_input.len() + 1;

            // preload first value from arg1 to cordic
            self.blocking_write_f64(arg1s[0])?;

            for (&arg1, &arg2) in double_input {
                // Since we manually preload a value before,
                // we will write arg2 (from the actual last pair) first, (at this moment, cordic start to calculating,)
                // and write arg1 (from the actual next pair), then read the result, to "keep preloading"

                self.blocking_write_f64(arg2)?;
                self.blocking_write_f64(arg1)?;
                self.blocking_read_f64_to_buf(output, &mut output_count);
            }

            // write last input value from arg2s, then read out the result
            self.blocking_write_f64(arg2s[arg2s.len() - 1])?;
            self.blocking_read_f64_to_buf(output, &mut output_count);
        }

        //
        // handle 1 input arg calculation
        //

        let input_left = &arg1s[consumed_input_len..];

        if !input_left.is_empty() {
            self.peri.set_argument_count(AccessCount::One);

            // "preload" value to cordic (at this moment, cordic start to calculating)
            self.blocking_write_f64(input_left[0])?;

            for &arg in input_left.iter().skip(1) {
                // this line write arg for next round caculation to cordic,
                // and read result from last round
                self.blocking_write_f64(arg)?;
                self.blocking_read_f64_to_buf(output, &mut output_count);
            }

            // read the last output
            self.blocking_read_f64_to_buf(output, &mut output_count);
        }

        Ok(output_count)
    }

    fn blocking_read_f64_to_buf(&mut self, result_buf: &mut [f64], result_index: &mut usize) {
        result_buf[*result_index] = utils::q1_31_to_f64(self.peri.read_result());
        *result_index += 1;

        // We don't care about whether the function return 1 or 2 results,
        // the only thing matter is whether user want 1 or 2 results.
        if !self.config.res1_only {
            result_buf[*result_index] = utils::q1_31_to_f64(self.peri.read_result());
            *result_index += 1;
        }
    }

    fn blocking_write_f64(&mut self, arg: f64) -> Result<(), NumberOutOfRange> {
        self.peri.write_argument(utils::f64_to_q1_31(arg)?);
        Ok(())
    }

    /// Run a async CORDIC calculation in q.1.31 format
    pub async fn async_calc_32bit(
        &mut self,
        write_dma: impl Peripheral<P = impl WriteDma<T>>,
        read_dma: impl Peripheral<P = impl ReadDma<T>>,
        arg1s: &[f64],
        arg2s: Option<&[f64]>,
        output: &mut [f64],
    ) -> Result<usize, CordicError> {
        if arg1s.is_empty() {
            return Ok(0);
        }

        let output_length_enough = match self.config.res1_only {
            true => output.len() >= arg1s.len(),
            false => output.len() >= 2 * arg1s.len(),
        };

        if !output_length_enough {
            return Err(CordicError::OutputLengthNotEnough);
        }

        self.check_input_f64(arg1s, arg2s)?;

        into_ref!(write_dma, read_dma);

        self.peri.set_result_count(if self.config.res1_only {
            AccessCount::One
        } else {
            AccessCount::Two
        });

        self.peri.set_data_width(Width::Bits32, Width::Bits32);

        let mut output_count = 0;
        let mut consumed_input_len = 0;
        let mut input_buf = [0u32; INPUT_BUF_MAX_LEN];
        let mut input_buf_len = 0;

        //
        // handle 2 input args calculation
        //

        if !arg2s.unwrap_or_default().is_empty() {
            let arg2s = arg2s.unwrap();

            self.peri.set_argument_count(AccessCount::Two);

            let double_input = arg1s.iter().zip(arg2s);

            consumed_input_len = double_input.len();

            for (&arg1, &arg2) in double_input {
                for &arg in [arg1, arg2].iter() {
                    input_buf[input_buf_len] = utils::f64_to_q1_31(arg)?;
                    input_buf_len += 1;
                }

                if input_buf_len == INPUT_BUF_MAX_LEN {
                    self.inner_dma_calc_32bit(
                        &mut write_dma,
                        &mut read_dma,
                        true,
                        &input_buf[..input_buf_len],
                        output,
                        &mut output_count,
                    )
                    .await;

                    input_buf_len = 0;
                }
            }

            if input_buf_len > 0 {
                self.inner_dma_calc_32bit(
                    &mut write_dma,
                    &mut read_dma,
                    true,
                    &input_buf[..input_buf_len],
                    output,
                    &mut output_count,
                )
                .await;

                input_buf_len = 0;
            }
        }

        //
        // handle 1 input arg calculation
        //

        if arg1s.len() > consumed_input_len {
            let input_remain = &arg1s[consumed_input_len..];

            self.peri.set_argument_count(AccessCount::One);

            for &arg in input_remain {
                input_buf[input_buf_len] = utils::f64_to_q1_31(arg)?;
                input_buf_len += 1;

                if input_buf_len == INPUT_BUF_MAX_LEN {
                    self.inner_dma_calc_32bit(
                        &mut write_dma,
                        &mut read_dma,
                        false,
                        &input_buf[..input_buf_len],
                        output,
                        &mut output_count,
                    )
                    .await;

                    input_buf_len = 0;
                }
            }

            if input_buf_len > 0 {
                self.inner_dma_calc_32bit(
                    &mut write_dma,
                    &mut read_dma,
                    false,
                    &input_buf[..input_buf_len],
                    output,
                    &mut output_count,
                )
                .await;

                // input_buf_len = 0;
            }
        }

        Ok(output_count)
    }

    // this function is highly coupled with async_calc_32bit, and is not intended to use in other place
    async fn inner_dma_calc_32bit(
        &mut self,
        write_dma: impl Peripheral<P = impl WriteDma<T>>,
        read_dma: impl Peripheral<P = impl ReadDma<T>>,
        double_input: bool,             // gether extra info to calc output_buf size
        input_buf: &[u32],              // input_buf, its content should be extact values and length for calculation
        output: &mut [f64],             // caller uses should this as a final output array
        output_start_index: &mut usize, // the index of start point of the output for this round of calculation
    ) {
        // output_buf is the place to store raw value from CORDIC (via DMA).
        // For buf size, we assume in this round of calculation:
        // all input is 1 arg, and all calculation need 2 output,
        // thus output_buf will always be long enough.
        let mut output_buf = [0u32; INPUT_BUF_MAX_LEN * 2];

        let mut output_buf_size = input_buf.len();
        if !self.config.res1_only {
            // if we need 2 result for 1 input, then output_buf length should be 2x long.
            output_buf_size *= 2;
        };
        if double_input {
            // if input itself is 2 args for 1 calculation, then output_buf length should be /2.
            output_buf_size /= 2;
        }

        let active_output_buf = &mut output_buf[..output_buf_size];

        self.launch_a_dma_transfer(write_dma, read_dma, input_buf, active_output_buf)
            .await;

        for &mut output_u32 in active_output_buf {
            output[*output_start_index] = utils::q1_31_to_f64(output_u32);
            *output_start_index += 1;
        }
    }
}

// q1.15 related
impl<'d, T: Instance> Cordic<'d, T> {
    /// Run a blocking CORDIC calculation in q1.15 format
    pub fn blocking_calc_16bit(
        &mut self,
        arg1s: &[f32],
        arg2s: Option<&[f32]>,
        output: &mut [f32],
    ) -> Result<usize, CordicError> {
        if arg1s.is_empty() {
            return Ok(0);
        }

        let output_length_enough = match self.config.res1_only {
            true => output.len() >= arg1s.len(),
            false => output.len() >= 2 * arg1s.len(),
        };

        if !output_length_enough {
            return Err(CordicError::OutputLengthNotEnough);
        }

        self.check_input_f32(arg1s, arg2s)?;

        // In q1.15 mode, 1 write/read to access 2 arguments/results
        self.peri.set_argument_count(AccessCount::One);
        self.peri.set_result_count(AccessCount::One);

        self.peri.set_data_width(Width::Bits16, Width::Bits16);

        let mut output_count = 0;

        // In q1.15 mode, we always fill 1 pair of 16bit value into WDATA register.
        // If arg2s is None or empty array, we assume arg2 value always 1.0 (as reset value for ARG2).
        // If arg2s has some value, and but not as long as arg1s,
        // we fill the reset of arg2 values with last value from arg2s (as q1.31 version does)

        let arg2_default_value = match arg2s {
            Some(arg2s) if !arg2s.is_empty() => arg2s[arg2s.len() - 1],
            _ => 1.0,
        };

        let mut args = arg1s.iter().zip(
            arg2s
                .unwrap_or(&[])
                .iter()
                .chain(core::iter::repeat(&arg2_default_value)),
        );

        let (&arg1, &arg2) = args.next().unwrap();

        // preloading 1 pair of arguments
        self.blocking_write_f32(arg1, arg2)?;

        for (&arg1, &arg2) in args {
            self.blocking_write_f32(arg1, arg2)?;
            self.blocking_read_f32_to_buf(output, &mut output_count);
        }

        // read last pair of value from cordic
        self.blocking_read_f32_to_buf(output, &mut output_count);

        Ok(output_count)
    }

    fn blocking_write_f32(&mut self, arg1: f32, arg2: f32) -> Result<(), NumberOutOfRange> {
        self.peri.write_argument(utils::f32_args_to_u32(arg1, arg2)?);
        Ok(())
    }

    fn blocking_read_f32_to_buf(&mut self, result_buf: &mut [f32], result_index: &mut usize) {
        let (res1, res2) = utils::u32_to_f32_res(self.peri.read_result());

        result_buf[*result_index] = res1;
        *result_index += 1;

        // We don't care about whether the function return 1 or 2 results,
        // the only thing matter is whether user want 1 or 2 results.
        if !self.config.res1_only {
            result_buf[*result_index] = res2;
            *result_index += 1;
        }
    }

    /// Run a async CORDIC calculation in q1.15 format
    pub async fn async_calc_16bit(
        &mut self,
        write_dma: impl Peripheral<P = impl WriteDma<T>>,
        read_dma: impl Peripheral<P = impl ReadDma<T>>,
        arg1s: &[f32],
        arg2s: Option<&[f32]>,
        output: &mut [f32],
    ) -> Result<usize, CordicError> {
        if arg1s.is_empty() {
            return Ok(0);
        }

        let output_length_enough = match self.config.res1_only {
            true => output.len() >= arg1s.len(),
            false => output.len() >= 2 * arg1s.len(),
        };

        if !output_length_enough {
            return Err(CordicError::OutputLengthNotEnough);
        }

        self.check_input_f32(arg1s, arg2s)?;

        into_ref!(write_dma, read_dma);

        // In q1.15 mode, 1 write/read to access 2 arguments/results
        self.peri.set_argument_count(AccessCount::One);
        self.peri.set_result_count(AccessCount::One);

        self.peri.set_data_width(Width::Bits16, Width::Bits16);

        let mut output_count = 0;
        let mut input_buf = [0u32; INPUT_BUF_MAX_LEN];
        let mut input_buf_len = 0;

        // In q1.15 mode, we always fill 1 pair of 16bit value into WDATA register.
        // If arg2s is None or empty array, we assume arg2 value always 1.0 (as reset value for ARG2).
        // If arg2s has some value, and but not as long as arg1s,
        // we fill the reset of arg2 values with last value from arg2s (as q1.31 version does)

        let arg2_default_value = match arg2s {
            Some(arg2s) if !arg2s.is_empty() => arg2s[arg2s.len() - 1],
            _ => 1.0,
        };

        let args = arg1s.iter().zip(
            arg2s
                .unwrap_or(&[])
                .iter()
                .chain(core::iter::repeat(&arg2_default_value)),
        );

        for (&arg1, &arg2) in args {
            input_buf[input_buf_len] = utils::f32_args_to_u32(arg1, arg2)?;
            input_buf_len += 1;

            if input_buf_len == INPUT_BUF_MAX_LEN {
                self.inner_dma_calc_16bit(&mut write_dma, &mut read_dma, &input_buf, output, &mut output_count)
                    .await;
            }
        }

        if input_buf_len > 0 {
            self.inner_dma_calc_16bit(
                &mut write_dma,
                &mut read_dma,
                &input_buf[..input_buf_len],
                output,
                &mut output_count,
            )
            .await;
        }

        Ok(output_count)
    }

    // this function is highly coupled with async_calc_16bit, and is not intended to use in other place
    async fn inner_dma_calc_16bit(
        &mut self,
        write_dma: impl Peripheral<P = impl WriteDma<T>>,
        read_dma: impl Peripheral<P = impl ReadDma<T>>,
        input_buf: &[u32],  // input_buf, its content should be extact values and length for calculation
        output: &mut [f32], // caller uses should this as a final output array
        output_start_index: &mut usize, // the index of start point of the output for this round of calculation
    ) {
        // output_buf is the place to store raw value from CORDIC (via DMA).
        let mut output_buf = [0u32; INPUT_BUF_MAX_LEN];

        let active_output_buf = &mut output_buf[..input_buf.len()];

        self.launch_a_dma_transfer(write_dma, read_dma, input_buf, active_output_buf)
            .await;

        for &mut output_u32 in active_output_buf {
            let (res1, res2) = utils::u32_to_f32_res(output_u32);

            output[*output_start_index] = res1;
            *output_start_index += 1;

            if !self.config.res1_only {
                output[*output_start_index] = res2;
                *output_start_index += 1;
            }
        }
    }
}

// check input value ARG1, ARG2, SCALE and FUNCTION are compatible with each other
macro_rules! check_input_value {
    ($func_name:ident, $float_type:ty) => {
        impl<'d, T: Instance> Cordic<'d, T> {
            fn $func_name(&self, arg1s: &[$float_type], arg2s: Option<&[$float_type]>) -> Result<(), ArgError> {
                let config = &self.config;

                use Function::*;

                struct Arg1ErrInfo {
                    scale: Option<Scale>,
                    range: [f32; 2],
                    inclusive_upper_bound: bool,
                }

                // check ARG1 value
                let err_info = match config.function {
                    Cos | Sin | Phase | Modulus | Arctan if arg1s.iter().any(|v| !(-1.0..=1.0).contains(v)) => {
                        Some(Arg1ErrInfo {
                            scale: None,
                            range: [-1.0, 1.0],
                            inclusive_upper_bound: true,
                        })
                    }

                    Cosh | Sinh if arg1s.iter().any(|v| !(-0.559..=0.559).contains(v)) => Some(Arg1ErrInfo {
                        scale: None,
                        range: [-0.559, 0.559],
                        inclusive_upper_bound: true,
                    }),

                    Arctanh if arg1s.iter().any(|v| !(-0.403..=0.403).contains(v)) => Some(Arg1ErrInfo {
                        scale: None,
                        range: [-0.403, 0.403],
                        inclusive_upper_bound: true,
                    }),

                    Ln => match config.scale {
                        Scale::Arg1o2Res2 if arg1s.iter().any(|v| !(0.0535..0.5).contains(v)) => Some(Arg1ErrInfo {
                            scale: Some(Scale::Arg1o2Res2),
                            range: [0.0535, 0.5],
                            inclusive_upper_bound: false,
                        }),
                        Scale::Arg1o4Res4 if arg1s.iter().any(|v| !(0.25..0.75).contains(v)) => Some(Arg1ErrInfo {
                            scale: Some(Scale::Arg1o4Res4),
                            range: [0.25, 0.75],
                            inclusive_upper_bound: false,
                        }),
                        Scale::Arg1o8Res8 if arg1s.iter().any(|v| !(0.375..0.875).contains(v)) => Some(Arg1ErrInfo {
                            scale: Some(Scale::Arg1o8Res8),
                            range: [0.375, 0.875],
                            inclusive_upper_bound: false,
                        }),
                        Scale::Arg1o16Res16 if arg1s.iter().any(|v| !(0.4375..0.584).contains(v)) => {
                            Some(Arg1ErrInfo {
                                scale: Some(Scale::Arg1o16Res16),
                                range: [0.4375, 0.584],
                                inclusive_upper_bound: false,
                            })
                        }

                        Scale::Arg1o2Res2 | Scale::Arg1o4Res4 | Scale::Arg1o8Res8 | Scale::Arg1o16Res16 => None,

                        _ => unreachable!(),
                    },

                    Sqrt => match config.scale {
                        Scale::Arg1Res1 if arg1s.iter().any(|v| !(0.027..0.75).contains(v)) => Some(Arg1ErrInfo {
                            scale: Some(Scale::Arg1Res1),
                            range: [0.027, 0.75],
                            inclusive_upper_bound: false,
                        }),
                        Scale::Arg1o2Res2 if arg1s.iter().any(|v| !(0.375..0.875).contains(v)) => Some(Arg1ErrInfo {
                            scale: Some(Scale::Arg1o2Res2),
                            range: [0.375, 0.875],
                            inclusive_upper_bound: false,
                        }),
                        Scale::Arg1o4Res4 if arg1s.iter().any(|v| !(0.4375..0.584).contains(v)) => Some(Arg1ErrInfo {
                            scale: Some(Scale::Arg1o4Res4),
                            range: [0.4375, 0.584],
                            inclusive_upper_bound: false,
                        }),
                        Scale::Arg1Res1 | Scale::Arg1o2Res2 | Scale::Arg1o4Res4 => None,
                        _ => unreachable!(),
                    },

                    Cos | Sin | Phase | Modulus | Arctan | Cosh | Sinh | Arctanh => None,
                };

                if let Some(err) = err_info {
                    return Err(ArgError {
                        func: config.function,
                        scale: err.scale,
                        arg_range: err.range,
                        inclusive_upper_bound: err.inclusive_upper_bound,
                        arg_type: ArgType::Arg1,
                    });
                }

                // check ARG2 value
                if let Some(arg2s) = arg2s {
                    struct Arg2ErrInfo {
                        range: [f32; 2],
                    }

                    let err_info = match config.function {
                        Cos | Sin if arg2s.iter().any(|v| !(0.0..=1.0).contains(v)) => {
                            Some(Arg2ErrInfo { range: [0.0, 1.0] })
                        }

                        Phase | Modulus if arg2s.iter().any(|v| !(-1.0..=1.0).contains(v)) => {
                            Some(Arg2ErrInfo { range: [-1.0, 1.0] })
                        }

                        Cos | Sin | Phase | Modulus | Arctan | Cosh | Sinh | Arctanh | Ln | Sqrt => None,
                    };

                    if let Some(err) = err_info {
                        return Err(ArgError {
                            func: config.function,
                            scale: None,
                            arg_range: err.range,
                            inclusive_upper_bound: true,
                            arg_type: ArgType::Arg2,
                        });
                    }
                }

                Ok(())
            }
        }
    };
}

check_input_value!(check_input_f64, f64);
check_input_value!(check_input_f32, f32);

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

dma_trait!(WriteDma, Instance);
dma_trait!(ReadDma, Instance);
