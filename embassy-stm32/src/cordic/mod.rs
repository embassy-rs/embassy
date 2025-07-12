//! coordinate rotation digital computer (CORDIC)

use embassy_hal_internal::drop::OnDrop;
use embassy_hal_internal::{Peri, PeripheralType};

use crate::pac::cordic::vals;
use crate::{dma, peripherals, rcc};

mod enums;
pub use enums::*;

mod errors;
pub use errors::*;

pub mod utils;

/// CORDIC driver
pub struct Cordic<'d, T: Instance> {
    peri: Peri<'d, T>,
    config: Config,
}

/// Cordic instance
trait SealedInstance {
    /// Get access to CORDIC registers
    fn regs() -> crate::pac::cordic::Cordic;

    /// Set Function value
    fn set_func(&self, func: Function) {
        Self::regs()
            .csr()
            .modify(|v| v.set_func(vals::Func::from_bits(func as u8)));
    }

    /// Set Precision value
    fn set_precision(&self, precision: Precision) {
        Self::regs()
            .csr()
            .modify(|v| v.set_precision(vals::Precision::from_bits(precision as u8)))
    }

    /// Set Scale value
    fn set_scale(&self, scale: Scale) {
        Self::regs()
            .csr()
            .modify(|v| v.set_scale(vals::Scale::from_bits(scale as u8)))
    }

    /// Enable global interrupt
    #[allow(unused)]
    fn enable_irq(&self) {
        Self::regs().csr().modify(|v| v.set_ien(true))
    }

    /// Disable global interrupt
    fn disable_irq(&self) {
        Self::regs().csr().modify(|v| v.set_ien(false))
    }

    /// Enable Read DMA
    fn enable_read_dma(&self) {
        Self::regs().csr().modify(|v| {
            v.set_dmaren(true);
        })
    }

    /// Disable Read DMA
    fn disable_read_dma(&self) {
        Self::regs().csr().modify(|v| {
            v.set_dmaren(false);
        })
    }

    /// Enable Write DMA
    fn enable_write_dma(&self) {
        Self::regs().csr().modify(|v| {
            v.set_dmawen(true);
        })
    }

    /// Disable Write DMA
    fn disable_write_dma(&self) {
        Self::regs().csr().modify(|v| {
            v.set_dmawen(false);
        })
    }

    /// Set NARGS value
    fn set_argument_count(&self, n: AccessCount) {
        Self::regs().csr().modify(|v| {
            v.set_nargs(match n {
                AccessCount::One => vals::Num::NUM1,
                AccessCount::Two => vals::Num::NUM2,
            })
        })
    }

    /// Set NRES value
    fn set_result_count(&self, n: AccessCount) {
        Self::regs().csr().modify(|v| {
            v.set_nres(match n {
                AccessCount::One => vals::Num::NUM1,
                AccessCount::Two => vals::Num::NUM2,
            });
        })
    }

    /// Set ARGSIZE and RESSIZE value
    fn set_data_width(&self, arg: Width, res: Width) {
        Self::regs().csr().modify(|v| {
            v.set_argsize(match arg {
                Width::Bits32 => vals::Size::BITS32,
                Width::Bits16 => vals::Size::BITS16,
            });
            v.set_ressize(match res {
                Width::Bits32 => vals::Size::BITS32,
                Width::Bits16 => vals::Size::BITS16,
            })
        })
    }

    /// Read RRDY flag
    fn ready_to_read(&self) -> bool {
        Self::regs().csr().read().rrdy()
    }

    /// Write value to WDATA
    fn write_argument(&self, arg: u32) {
        Self::regs().wdata().write_value(arg)
    }

    /// Read value from RDATA
    fn read_result(&self) -> u32 {
        Self::regs().rdata().read()
    }
}

/// CORDIC instance trait
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + crate::rcc::RccPeripheral {}

/// CORDIC configuration
#[derive(Debug)]
pub struct Config {
    function: Function,
    precision: Precision,
    scale: Scale,
}

impl Config {
    /// Create a config for Cordic driver
    pub fn new(function: Function, precision: Precision, scale: Scale) -> Result<Self, CordicError> {
        let config = Self {
            function,
            precision,
            scale,
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
    /// If you need a peripheral -> CORDIC -> peripheral mode,  
    /// you may want to set Cordic into [Mode::ZeroOverhead] mode, and add extra arguments with [Self::extra_config]
    pub fn new(peri: Peri<'d, T>, config: Config) -> Self {
        rcc::enable_and_reset::<T>();

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

    fn clean_rrdy_flag(&mut self) {
        while self.peri.ready_to_read() {
            self.peri.read_result();
        }
    }

    /// Disable IRQ and DMA, clean RRDY, and set ARG2 to +1 (0x7FFFFFFF)
    pub fn reconfigure(&mut self) {
        // reset ARG2 to +1
        {
            self.peri.disable_irq();
            self.peri.disable_read_dma();
            self.peri.disable_write_dma();
            self.clean_rrdy_flag();

            self.peri.set_func(Function::Cos);
            self.peri.set_precision(Precision::Iters4);
            self.peri.set_scale(Scale::Arg1Res1);
            self.peri.set_argument_count(AccessCount::Two);
            self.peri.set_data_width(Width::Bits32, Width::Bits32);
            self.peri.write_argument(0x0u32);
            self.peri.write_argument(0x7FFFFFFFu32);

            self.clean_rrdy_flag();
        }

        self.peri.set_func(self.config.function);
        self.peri.set_precision(self.config.precision);
        self.peri.set_scale(self.config.scale);

        // we don't set NRES in here, but to make sure NRES is set each time user call "calc"-ish functions,
        // since each "calc"-ish functions can have different ARGSIZE and RESSIZE, thus NRES should be change accordingly.
    }
}

impl<'d, T: Instance> Drop for Cordic<'d, T> {
    fn drop(&mut self) {
        rcc::disable::<T>();
    }
}

// q1.31 related
impl<'d, T: Instance> Cordic<'d, T> {
    /// Run a blocking CORDIC calculation in q1.31 format  
    ///
    /// Notice:  
    /// If you set `arg1_only` to `true`, please be sure ARG2 value has been set to desired value before.  
    /// This function won't set ARG2 to +1 before or after each round of calculation.  
    /// If you want to make sure ARG2 is set to +1, consider run [.reconfigure()](Self::reconfigure).
    pub fn blocking_calc_32bit(
        &mut self,
        arg: &[u32],
        res: &mut [u32],
        arg1_only: bool,
        res1_only: bool,
    ) -> Result<usize, CordicError> {
        if arg.is_empty() {
            return Ok(0);
        }

        let res_cnt = Self::check_arg_res_length_32bit(arg.len(), res.len(), arg1_only, res1_only)?;

        self.peri
            .set_argument_count(if arg1_only { AccessCount::One } else { AccessCount::Two });

        self.peri
            .set_result_count(if res1_only { AccessCount::One } else { AccessCount::Two });

        self.peri.set_data_width(Width::Bits32, Width::Bits32);

        let mut cnt = 0;

        match arg1_only {
            true => {
                // To use cordic preload function, the first value is special.
                // It is loaded to CORDIC WDATA register out side of loop
                let first_value = arg[0];

                // preload 1st value to CORDIC, to start the CORDIC calc
                self.peri.write_argument(first_value);

                for &arg1 in &arg[1..] {
                    // preload arg1 (for next calc)
                    self.peri.write_argument(arg1);

                    // then read current result out
                    res[cnt] = self.peri.read_result();
                    cnt += 1;
                    if !res1_only {
                        res[cnt] = self.peri.read_result();
                        cnt += 1;
                    }
                }

                // read the last result
                res[cnt] = self.peri.read_result();
                cnt += 1;
                if !res1_only {
                    res[cnt] = self.peri.read_result();
                    // cnt += 1;
                }
            }
            false => {
                // To use cordic preload function, the first and last value is special.
                // They are load to CORDIC WDATA register out side of loop
                let first_value = arg[0];
                let last_value = arg[arg.len() - 1];

                let paired_args = &arg[1..arg.len() - 1];

                // preload 1st value to CORDIC
                self.peri.write_argument(first_value);

                for args in paired_args.chunks(2) {
                    let arg2 = args[0];
                    let arg1 = args[1];

                    // load arg2 (for current calc) first, to start the CORDIC calc
                    self.peri.write_argument(arg2);

                    // preload arg1 (for next calc)
                    self.peri.write_argument(arg1);

                    // then read current result out
                    res[cnt] = self.peri.read_result();
                    cnt += 1;
                    if !res1_only {
                        res[cnt] = self.peri.read_result();
                        cnt += 1;
                    }
                }

                // load last value to CORDIC, and finish the calculation
                self.peri.write_argument(last_value);
                res[cnt] = self.peri.read_result();
                cnt += 1;
                if !res1_only {
                    res[cnt] = self.peri.read_result();
                    // cnt += 1;
                }
            }
        }

        // at this point cnt should be equal to res_cnt

        Ok(res_cnt)
    }

    /// Run a async CORDIC calculation in q.1.31 format
    ///
    /// Notice:  
    /// If you set `arg1_only` to `true`, please be sure ARG2 value has been set to desired value before.  
    /// This function won't set ARG2 to +1 before or after each round of calculation.  
    /// If you want to make sure ARG2 is set to +1, consider run [.reconfigure()](Self::reconfigure).
    pub async fn async_calc_32bit(
        &mut self,
        mut write_dma: Peri<'_, impl WriteDma<T>>,
        mut read_dma: Peri<'_, impl ReadDma<T>>,
        arg: &[u32],
        res: &mut [u32],
        arg1_only: bool,
        res1_only: bool,
    ) -> Result<usize, CordicError> {
        if arg.is_empty() {
            return Ok(0);
        }

        let res_cnt = Self::check_arg_res_length_32bit(arg.len(), res.len(), arg1_only, res1_only)?;

        let active_res_buf = &mut res[..res_cnt];

        self.peri
            .set_argument_count(if arg1_only { AccessCount::One } else { AccessCount::Two });

        self.peri
            .set_result_count(if res1_only { AccessCount::One } else { AccessCount::Two });

        self.peri.set_data_width(Width::Bits32, Width::Bits32);

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
                write_dma.reborrow(),
                write_req,
                arg,
                T::regs().wdata().as_ptr() as *mut _,
                Default::default(),
            );

            let read_transfer = dma::Transfer::new_read(
                read_dma.reborrow(),
                read_req,
                T::regs().rdata().as_ptr() as *mut _,
                active_res_buf,
                Default::default(),
            );

            embassy_futures::join::join(write_transfer, read_transfer).await;
        }

        Ok(res_cnt)
    }

    fn check_arg_res_length_32bit(
        arg_len: usize,
        res_len: usize,
        arg1_only: bool,
        res1_only: bool,
    ) -> Result<usize, CordicError> {
        if !arg1_only && arg_len % 2 != 0 {
            return Err(CordicError::ArgumentLengthIncorrect);
        }

        let mut minimal_res_length = arg_len;

        if !res1_only {
            minimal_res_length *= 2;
        }

        if !arg1_only {
            minimal_res_length /= 2
        }

        if minimal_res_length > res_len {
            return Err(CordicError::ResultLengthNotEnough);
        }

        Ok(minimal_res_length)
    }
}

// q1.15 related
impl<'d, T: Instance> Cordic<'d, T> {
    /// Run a blocking CORDIC calculation in q1.15 format  
    ///
    /// Notice::  
    /// User will take respond to merge two u16 arguments into one u32 data, and/or split one u32 data into two u16 results.
    pub fn blocking_calc_16bit(&mut self, arg: &[u32], res: &mut [u32]) -> Result<usize, CordicError> {
        if arg.is_empty() {
            return Ok(0);
        }

        if arg.len() > res.len() {
            return Err(CordicError::ResultLengthNotEnough);
        }

        let res_cnt = arg.len();

        // In q1.15 mode, 1 write/read to access 2 arguments/results
        self.peri.set_argument_count(AccessCount::One);
        self.peri.set_result_count(AccessCount::One);

        self.peri.set_data_width(Width::Bits16, Width::Bits16);

        // To use cordic preload function, the first value is special.
        // It is loaded to CORDIC WDATA register out side of loop
        let first_value = arg[0];

        // preload 1st value to CORDIC, to start the CORDIC calc
        self.peri.write_argument(first_value);

        let mut cnt = 0;

        for &arg_val in &arg[1..] {
            // preload arg_val (for next calc)
            self.peri.write_argument(arg_val);

            // then read current result out
            res[cnt] = self.peri.read_result();
            cnt += 1;
        }

        // read last result out
        res[cnt] = self.peri.read_result();
        // cnt += 1;

        Ok(res_cnt)
    }

    /// Run a async CORDIC calculation in q1.15 format  
    ///
    /// Notice::  
    /// User will take respond to merge two u16 arguments into one u32 data, and/or split one u32 data into two u16 results.
    pub async fn async_calc_16bit(
        &mut self,
        mut write_dma: Peri<'_, impl WriteDma<T>>,
        mut read_dma: Peri<'_, impl ReadDma<T>>,
        arg: &[u32],
        res: &mut [u32],
    ) -> Result<usize, CordicError> {
        if arg.is_empty() {
            return Ok(0);
        }

        if arg.len() > res.len() {
            return Err(CordicError::ResultLengthNotEnough);
        }

        let res_cnt = arg.len();

        let active_res_buf = &mut res[..res_cnt];

        // In q1.15 mode, 1 write/read to access 2 arguments/results
        self.peri.set_argument_count(AccessCount::One);
        self.peri.set_result_count(AccessCount::One);

        self.peri.set_data_width(Width::Bits16, Width::Bits16);

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
                write_dma.reborrow(),
                write_req,
                arg,
                T::regs().wdata().as_ptr() as *mut _,
                Default::default(),
            );

            let read_transfer = dma::Transfer::new_read(
                read_dma.reborrow(),
                read_req,
                T::regs().rdata().as_ptr() as *mut _,
                active_res_buf,
                Default::default(),
            );

            embassy_futures::join::join(write_transfer, read_transfer).await;
        }

        Ok(res_cnt)
    }
}

macro_rules! check_arg_value {
    ($func_arg1_name:ident, $func_arg2_name:ident, $float_type:ty) => {
        impl<'d, T: Instance> Cordic<'d, T> {
            /// check input value ARG1, SCALE and FUNCTION are compatible with each other
            pub fn $func_arg1_name(&self, arg: $float_type) -> Result<(), ArgError> {
                let config = &self.config;

                use Function::*;

                struct Arg1ErrInfo {
                    scale: Option<Scale>,
                    range: [f32; 2], // f32 is ok, it only used in error display
                    inclusive_upper_bound: bool,
                }

                let err_info = match config.function {
                    Cos | Sin | Phase | Modulus | Arctan if !(-1.0..=1.0).contains(arg) => Some(Arg1ErrInfo {
                        scale: None,
                        range: [-1.0, 1.0],
                        inclusive_upper_bound: true,
                    }),

                    Cosh | Sinh if !(-0.559..=0.559).contains(arg) => Some(Arg1ErrInfo {
                        scale: None,
                        range: [-0.559, 0.559],
                        inclusive_upper_bound: true,
                    }),

                    Arctanh if !(-0.403..=0.403).contains(arg) => Some(Arg1ErrInfo {
                        scale: None,
                        range: [-0.403, 0.403],
                        inclusive_upper_bound: true,
                    }),

                    Ln => match config.scale {
                        Scale::Arg1o2Res2 if !(0.0535..0.5).contains(arg) => Some(Arg1ErrInfo {
                            scale: Some(Scale::Arg1o2Res2),
                            range: [0.0535, 0.5],
                            inclusive_upper_bound: false,
                        }),
                        Scale::Arg1o4Res4 if !(0.25..0.75).contains(arg) => Some(Arg1ErrInfo {
                            scale: Some(Scale::Arg1o4Res4),
                            range: [0.25, 0.75],
                            inclusive_upper_bound: false,
                        }),
                        Scale::Arg1o8Res8 if !(0.375..0.875).contains(arg) => Some(Arg1ErrInfo {
                            scale: Some(Scale::Arg1o8Res8),
                            range: [0.375, 0.875],
                            inclusive_upper_bound: false,
                        }),
                        Scale::Arg1o16Res16 if !(0.4375..0.584).contains(arg) => Some(Arg1ErrInfo {
                            scale: Some(Scale::Arg1o16Res16),
                            range: [0.4375, 0.584],
                            inclusive_upper_bound: false,
                        }),

                        Scale::Arg1o2Res2 | Scale::Arg1o4Res4 | Scale::Arg1o8Res8 | Scale::Arg1o16Res16 => None,

                        _ => unreachable!(),
                    },

                    Sqrt => match config.scale {
                        Scale::Arg1Res1 if !(0.027..0.75).contains(arg) => Some(Arg1ErrInfo {
                            scale: Some(Scale::Arg1Res1),
                            range: [0.027, 0.75],
                            inclusive_upper_bound: false,
                        }),
                        Scale::Arg1o2Res2 if !(0.375..0.875).contains(arg) => Some(Arg1ErrInfo {
                            scale: Some(Scale::Arg1o2Res2),
                            range: [0.375, 0.875],
                            inclusive_upper_bound: false,
                        }),
                        Scale::Arg1o4Res4 if !(0.4375..0.584).contains(arg) => Some(Arg1ErrInfo {
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

                Ok(())
            }

            /// check input value ARG2 and FUNCTION are compatible with each other
            pub fn $func_arg2_name(&self, arg: $float_type) -> Result<(), ArgError> {
                let config = &self.config;

                use Function::*;

                struct Arg2ErrInfo {
                    range: [f32; 2], // f32 is ok, it only used in error display
                }

                let err_info = match config.function {
                    Cos | Sin if !(0.0..=1.0).contains(arg) => Some(Arg2ErrInfo { range: [0.0, 1.0] }),

                    Phase | Modulus if !(-1.0..=1.0).contains(arg) => Some(Arg2ErrInfo { range: [-1.0, 1.0] }),

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

                Ok(())
            }
        }
    };
}

check_arg_value!(check_f64_arg1, check_f64_arg2, &f64);
check_arg_value!(check_f32_arg1, check_f32_arg2, &f32);

foreach_interrupt!(
    ($inst:ident, cordic, $block:ident, GLOBAL, $irq:ident) => {
        impl Instance for peripherals::$inst {
        }

        impl SealedInstance for peripherals::$inst {
            fn regs() -> crate::pac::cordic::Cordic {
                crate::pac::$inst
            }
        }
    };
);

dma_trait!(WriteDma, Instance);
dma_trait!(ReadDma, Instance);
