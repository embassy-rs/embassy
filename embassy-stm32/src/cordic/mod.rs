//! CORDIC co-processor

use embassy_hal_internal::drop::OnDrop;
use embassy_hal_internal::{into_ref, Peripheral, PeripheralRef};

use crate::{dma, peripherals};

mod enums;
pub use enums::*;

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
pub struct Config {
    function: Function,
    precision: Precision,
    scale: Scale,
    first_result: bool,
}

impl Config {
    /// Create a config for Cordic driver
    pub fn new(function: Function, precision: Option<Precision>, scale: Option<Scale>, first_result: bool) -> Self {
        Self {
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
    pub fn new(peri: impl Peripheral<P = T> + 'd, config: Config) -> Self {
        T::enable_and_reset();

        into_ref!(peri);

        if !config.check_scale() {
            panic!("Scale value is not compatible with Function")
        }

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
    pub fn extra_config(&mut self, arg_cnt: Count, arg_width: Width, res_width: Width) {
        self.peri.set_argument_count(arg_cnt);
        self.peri.set_data_width(arg_width, res_width);
    }

    fn reconfigure(&mut self) {
        if self.peri.ready_to_read() {
            warn!("At least 1 result hasn't been read, reconfigure will cause DATA LOST");
        };

        // clean RRDY flag
        while self.peri.ready_to_read() {
            self.peri.read_result();
        }

        self.peri.set_func(self.config.function);
        self.peri.set_precision(self.config.precision);
        self.peri.set_scale(self.config.scale);

        // we don't set NRES in here, but to make sure NRES is set each time user call "calc"-ish functions,
        // since each "calc"-ish functions can have different ARGSIZE and RESSIZE, thus NRES should be change accrodingly.
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
    pub fn blocking_calc_32bit(&mut self, arg1s: &[f64], arg2s: Option<&[f64]>, output: &mut [f64]) -> usize {
        if arg1s.is_empty() {
            return 0;
        }

        assert!(
            match self.config.first_result {
                true => output.len() >= arg1s.len(),
                false => output.len() >= 2 * arg1s.len(),
            },
            "Output buf length is not long enough"
        );

        self.check_input_f64(arg1s, arg2s);

        self.peri.set_result_count(if self.config.first_result {
            Count::One
        } else {
            Count::Two
        });

        self.peri.set_data_width(Width::Bits32, Width::Bits32);

        let mut output_count = 0;

        let mut consumed_input_len = 0;

        //
        // handle 2 input args calculation
        //

        if arg2s.is_some() && !arg2s.expect("It's infailable").is_empty() {
            let arg2s = arg2s.expect("It's infailable");

            self.peri.set_argument_count(Count::Two);

            // Skip 1st value from arg1s, this value will be manually "preload" to cordic, to make use of cordic preload function.
            // And we preserve last value from arg2s, since it need to manually write to cordic, and read the result out.
            let double_input = arg1s.iter().skip(1).zip(&arg2s[..arg2s.len() - 1]);
            // Since we preload 1st value from arg1s, the consumed input length is double_input length + 1.
            consumed_input_len = double_input.len() + 1;

            // preload first value from arg1 to cordic
            self.blocking_write_f64(arg1s[0]);

            for (&arg1, &arg2) in double_input {
                // Since we manually preload a value before,
                // we will write arg2 (from the actual last pair) first, (at this moment, cordic start to calculating,)
                // and write arg1 (from the actual next pair), then read the result, to "keep preloading"

                self.blocking_write_f64(arg2);
                self.blocking_write_f64(arg1);
                self.blocking_read_f64_to_buf(output, &mut output_count);
            }

            // write last input value from arg2s, then read out the result
            self.blocking_write_f64(arg2s[arg2s.len() - 1]);
            self.blocking_read_f64_to_buf(output, &mut output_count);
        }

        //
        // handle 1 input arg calculation
        //

        let input_left = &arg1s[consumed_input_len..];

        if !input_left.is_empty() {
            self.peri.set_argument_count(Count::One);

            // "preload" value to cordic (at this moment, cordic start to calculating)
            self.blocking_write_f64(input_left[0]);

            for &arg in input_left.iter().skip(1) {
                // this line write arg for next round caculation to cordic,
                // and read result from last round
                self.blocking_write_f64(arg);
                self.blocking_read_f64_to_buf(output, &mut output_count);
            }

            // read the last output
            self.blocking_read_f64_to_buf(output, &mut output_count);
        }

        output_count
    }

    fn blocking_read_f64_to_buf(&mut self, result_buf: &mut [f64], result_index: &mut usize) {
        result_buf[*result_index] = utils::q1_31_to_f64(self.peri.read_result());
        *result_index += 1;

        // We don't care about whether the function return 1 or 2 results,
        // the only thing matter is whether user want 1 or 2 results.
        if !self.config.first_result {
            result_buf[*result_index] = utils::q1_31_to_f64(self.peri.read_result());
            *result_index += 1;
        }
    }

    fn blocking_write_f64(&mut self, arg: f64) {
        self.peri.write_argument(utils::f64_to_q1_31(arg));
    }

    /// Run a async CORDIC calculation in q.1.31 format
    pub async fn async_calc_32bit(
        &mut self,
        write_dma: impl Peripheral<P = impl WriteDma<T>>,
        read_dma: impl Peripheral<P = impl ReadDma<T>>,
        arg1s: &[f64],
        arg2s: Option<&[f64]>,
        output: &mut [f64],
    ) -> usize {
        if arg1s.is_empty() {
            return 0;
        }

        assert!(
            match self.config.first_result {
                true => output.len() >= arg1s.len(),
                false => output.len() >= 2 * arg1s.len(),
            },
            "Output buf length is not long enough"
        );

        self.check_input_f64(arg1s, arg2s);

        into_ref!(write_dma, read_dma);

        self.peri.set_result_count(if self.config.first_result {
            Count::One
        } else {
            Count::Two
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
            let arg2s = arg2s.expect("It's infailable");

            self.peri.set_argument_count(Count::Two);

            let double_input = arg1s.iter().zip(arg2s);

            consumed_input_len = double_input.len();

            for (&arg1, &arg2) in double_input {
                for &arg in [arg1, arg2].iter() {
                    input_buf[input_buf_len] = utils::f64_to_q1_31(arg);
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

            self.peri.set_argument_count(Count::One);

            for &arg in input_remain {
                input_buf[input_buf_len] = utils::f64_to_q1_31(arg);
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

        output_count
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
        into_ref!(write_dma, read_dma);

        let write_req = write_dma.request();
        let read_req = read_dma.request();

        // output_buf is the place to store raw value from CORDIC (via DMA).
        // For buf size, we assume in this round of calculation:
        // all input is 1 arg, and all calculation need 2 output,
        // thus output_buf will always be long enough.
        let mut output_buf = [0u32; INPUT_BUF_MAX_LEN * 2];

        let mut output_buf_size = input_buf.len();
        if !self.config.first_result {
            // if we need 2 result for 1 input, then output_buf length should be 2x long.
            output_buf_size *= 2;
        };
        if double_input {
            // if input itself is 2 args for 1 calculation, then output_buf length should be /2.
            output_buf_size /= 2;
        }

        let active_output_buf = &mut output_buf[..output_buf_size];

        self.peri.enable_write_dma();
        self.peri.enable_read_dma();

        let on_drop = OnDrop::new(|| {
            self.peri.disable_write_dma();
            self.peri.disable_read_dma();
        });

        unsafe {
            let write_transfer = dma::Transfer::new_write(
                &mut write_dma,
                write_req,
                input_buf,
                T::regs().wdata().as_ptr() as *mut _,
                Default::default(),
            );

            let read_transfer = dma::Transfer::new_read(
                &mut read_dma,
                read_req,
                T::regs().rdata().as_ptr() as *mut _,
                active_output_buf,
                Default::default(),
            );

            embassy_futures::join::join(write_transfer, read_transfer).await;
        }

        drop(on_drop);

        for &mut output_u32 in active_output_buf {
            output[*output_start_index] = utils::q1_31_to_f64(output_u32);
            *output_start_index += 1;
        }
    }
}

// q1.15 related
impl<'d, T: Instance> Cordic<'d, T> {
    /// Run a blocking CORDIC calculation in q1.15 format
    pub fn blocking_calc_16bit(&mut self, arg1s: &[f32], arg2s: Option<&[f32]>, output: &mut [f32]) -> usize {
        if arg1s.is_empty() {
            return 0;
        }

        assert!(
            match self.config.first_result {
                true => output.len() >= arg1s.len(),
                false => output.len() >= 2 * arg1s.len(),
            },
            "Output buf length is not long enough"
        );

        self.check_input_f32(arg1s, arg2s);

        // In q1.15 mode, 1 write/read to access 2 arguments/results
        self.peri.set_argument_count(Count::One);
        self.peri.set_result_count(Count::One);

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

        let (&arg1, &arg2) = args
            .next()
            .expect("This should be infallible, since arg1s is not empty");

        // preloading 1 pair of arguments
        self.blocking_write_f32(arg1, arg2);

        for (&arg1, &arg2) in args {
            self.blocking_write_f32(arg1, arg2);
            self.blocking_read_f32_to_buf(output, &mut output_count);
        }

        // read last pair of value from cordic
        self.blocking_read_f32_to_buf(output, &mut output_count);

        output_count
    }

    fn blocking_write_f32(&mut self, arg1: f32, arg2: f32) {
        let reg_value: u32 = utils::f32_args_to_u32(arg1, arg2);
        self.peri.write_argument(reg_value);
    }

    fn blocking_read_f32_to_buf(&mut self, result_buf: &mut [f32], result_index: &mut usize) {
        let reg_value = self.peri.read_result();

        let (res1, res2) = utils::u32_to_f32_res(reg_value);

        result_buf[*result_index] = res1;
        *result_index += 1;

        // We don't care about whether the function return 1 or 2 results,
        // the only thing matter is whether user want 1 or 2 results.
        if !self.config.first_result {
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
    ) -> usize {
        if arg1s.is_empty() {
            return 0;
        }

        assert!(
            match self.config.first_result {
                true => output.len() >= arg1s.len(),
                false => output.len() >= 2 * arg1s.len(),
            },
            "Output buf length is not long enough"
        );

        self.check_input_f32(arg1s, arg2s);

        into_ref!(write_dma, read_dma);

        // In q1.15 mode, 1 write/read to access 2 arguments/results
        self.peri.set_argument_count(Count::One);
        self.peri.set_result_count(Count::One);

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
            input_buf[input_buf_len] = utils::f32_args_to_u32(arg1, arg2);
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

        output_count
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
        into_ref!(write_dma, read_dma);

        let write_req = write_dma.request();
        let read_req = read_dma.request();

        // output_buf is the place to store raw value from CORDIC (via DMA).
        let mut output_buf = [0u32; INPUT_BUF_MAX_LEN];

        let active_output_buf = &mut output_buf[..input_buf.len()];

        self.peri.enable_write_dma();
        self.peri.enable_read_dma();

        let on_drop = OnDrop::new(|| {
            self.peri.disable_write_dma();
            self.peri.disable_read_dma();
        });

        unsafe {
            let write_transfer = dma::Transfer::new_write(
                &mut write_dma,
                write_req,
                input_buf,
                T::regs().wdata().as_ptr() as *mut _,
                Default::default(),
            );

            let read_transfer = dma::Transfer::new_read(
                &mut read_dma,
                read_req,
                T::regs().rdata().as_ptr() as *mut _,
                active_output_buf,
                Default::default(),
            );

            embassy_futures::join::join(write_transfer, read_transfer).await;
        }

        drop(on_drop);

        for &mut output_u32 in active_output_buf {
            let (res1, res2) = utils::u32_to_f32_res(output_u32);

            output[*output_start_index] = res1;
            *output_start_index += 1;

            if !self.config.first_result {
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
            fn $func_name(&self, arg1s: &[$float_type], arg2s: Option<&[$float_type]>) {
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
                                arg1s.iter().all(|v| (0.4375..0.584).contains(v)),
                                "When SCALE set to 4, ARG1 should be: 0.4375 <= ARG1 < 0.584"
                            ),
                            _ => unreachable!(),
                        };
                    }

                    Sqrt => match config.scale {
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
