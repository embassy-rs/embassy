//! Coordinate Rotation Digital Computer (CORDIC)

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
    arg_count: AccessCount,
    res_count: AccessCount,
}

impl Config {
    /// Create a config for Cordic driver
    ///
    /// `arg_count` defaults to `AccessCount::One` and `res_count` defaults to `AccessCount::Two`.
    /// Use the builder methods [`Self::arg_count`] and [`Self::res_count`] to override.
    pub fn new(function: Function, precision: Precision, scale: Scale) -> Result<Self, CordicError> {
        let config = Self {
            function,
            precision,
            scale,
            arg_count: AccessCount::One,
            res_count: AccessCount::Two,
        };

        config.check_scale()?;

        Ok(config)
    }

    /// Set the argument access count.
    ///
    /// `AccessCount::One`: each WDATA write provides one argument (ARG1), reusing the previous ARG2.
    /// `AccessCount::Two`: arguments are written in pairs (ARG1 then ARG2) to WDATA.
    pub fn arg_count(mut self, arg_count: AccessCount) -> Self {
        self.arg_count = arg_count;
        self
    }

    /// Set the result access count.
    ///
    /// `AccessCount::One`: each calculation produces one RDATA read (primary result only).
    /// `AccessCount::Two`: each calculation produces two RDATA reads (primary + secondary).
    pub fn res_count(mut self, res_count: AccessCount) -> Self {
        self.res_count = res_count;
        self
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
    pub fn new(peri: Peri<'d, T>, config: Config) -> Self {
        rcc::enable_and_reset::<T>();

        let mut instance = Self { peri, config };

        instance.reconfigure();

        instance
    }

    /// Set a new config for Cordic driver.
    ///
    /// This calls [`Self::reconfigure`], which resets ARG2 to +1.
    /// To change only `arg_count`/`res_count` without resetting ARG2,
    /// use [`Self::set_access_counts`] instead.
    pub fn set_config(&mut self, config: Config) {
        self.config = config;
        self.reconfigure();
    }

    /// Change `arg_count` and `res_count` without resetting ARG2.
    ///
    /// This is a lightweight CSR update for switching between 1-arg and 2-arg
    /// modes within the same function configuration (e.g. after an initial
    /// 2-arg call sets ARG2, switch to 1-arg mode for the hot loop).
    pub fn set_access_counts(&mut self, arg_count: AccessCount, res_count: AccessCount) {
        self.config.arg_count = arg_count;
        self.config.res_count = res_count;
        T::regs().csr().modify(|v| {
            v.set_nargs(match arg_count {
                AccessCount::One => vals::Num::NUM1,
                AccessCount::Two => vals::Num::NUM2,
            });
            v.set_nres(match res_count {
                AccessCount::One => vals::Num::NUM1,
                AccessCount::Two => vals::Num::NUM2,
            });
        });
    }

    fn clean_rrdy_flag(&mut self) {
        while self.peri.ready_to_read() {
            self.peri.read_result();
        }
    }

    /// Disable IRQ and DMA, clean RRDY, and set ARG2 to +1 (0x7FFFFFFF)
    pub fn reconfigure(&mut self) {
        // Disable IRQ and DMA first
        T::regs().csr().modify(|v| {
            v.set_ien(false);
            v.set_dmaren(false);
            v.set_dmawen(false);
        });
        self.clean_rrdy_flag();

        // Reset ARG2 to +1: configure for 2-arg Cos with minimal precision, feed dummy args.
        T::regs().csr().modify(|v| {
            v.set_func(vals::Func::from_bits(Function::Cos as u8));
            v.set_precision(vals::Precision::from_bits(Precision::Iters4 as u8));
            v.set_scale(vals::Scale::from_bits(Scale::Arg1Res1 as u8));
            v.set_nargs(vals::Num::NUM2);
            v.set_argsize(vals::Size::BITS32);
            v.set_ressize(vals::Size::BITS32);
        });
        self.peri.write_argument(0x0u32);
        self.peri.write_argument(0x7FFFFFFFu32);
        self.clean_rrdy_flag();

        // Apply full user configuration (func, precision, scale, data interface).
        T::regs().csr().modify(|v| {
            v.set_func(vals::Func::from_bits(self.config.function as u8));
            v.set_precision(vals::Precision::from_bits(self.config.precision as u8));
            v.set_scale(vals::Scale::from_bits(self.config.scale as u8));
            v.set_nargs(match self.config.arg_count {
                AccessCount::One => vals::Num::NUM1,
                AccessCount::Two => vals::Num::NUM2,
            });
            v.set_nres(match self.config.res_count {
                AccessCount::One => vals::Num::NUM1,
                AccessCount::Two => vals::Num::NUM2,
            });
            v.set_argsize(vals::Size::BITS32);
            v.set_ressize(vals::Size::BITS32);
        });

        // Changing NRES or other CSR fields above can re-assert RRDY if secondary
        // results from the dummy calc were not fully drained. Clean it again.
        self.clean_rrdy_flag();
    }
}

impl<'d, T: Instance> Drop for Cordic<'d, T> {
    fn drop(&mut self) {
        rcc::disable::<T>();
    }
}

// q1.31 related
impl<'d, T: Instance> Cordic<'d, T> {
    /// Run a blocking CORDIC calculation in q1.31 format.
    ///
    /// Uses `arg_count` and `res_count` from the current [`Config`].
    /// If `arg_count` is `One`, ARG2 must have been set to the desired value
    /// beforehand (e.g. via a prior `Two`-arg call or [`Self::reconfigure`]).
    #[inline]
    pub fn blocking_calc_32bit(&mut self, arg: &[u32], res: &mut [u32]) -> Result<usize, CordicError> {
        if arg.is_empty() {
            return Ok(0);
        }

        let arg1_only = matches!(self.config.arg_count, AccessCount::One);
        let res1_only = matches!(self.config.res_count, AccessCount::One);

        let res_cnt = Self::check_arg_res_length_32bit(arg.len(), res.len(), arg1_only, res1_only)?;

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

    /// Run an async CORDIC calculation in q1.31 format.
    ///
    /// Uses `arg_count` and `res_count` from the current [`Config`].
    /// If `arg_count` is `One`, ARG2 must have been set to the desired value
    /// beforehand (e.g. via a prior `Two`-arg call or [`Self::reconfigure`]).
    #[inline]
    pub async fn async_calc_32bit<'a, W, R>(
        &mut self,
        mut write_dma: Peri<'a, W>,
        mut read_dma: Peri<'a, R>,
        irq: impl crate::interrupt::typelevel::Binding<W::Interrupt, crate::dma::InterruptHandler<W>>
        + crate::interrupt::typelevel::Binding<R::Interrupt, crate::dma::InterruptHandler<R>>
        + 'a,
        arg: &[u32],
        res: &mut [u32],
    ) -> Result<usize, CordicError>
    where
        W: WriteDma<T>,
        R: ReadDma<T>,
    {
        if arg.is_empty() {
            return Ok(0);
        }

        let arg1_only = matches!(self.config.arg_count, AccessCount::One);
        let res1_only = matches!(self.config.res_count, AccessCount::One);

        let res_cnt = Self::check_arg_res_length_32bit(arg.len(), res.len(), arg1_only, res1_only)?;

        let active_res_buf = &mut res[..res_cnt];

        let write_req = write_dma.request();
        let read_req = read_dma.request();

        // DMAWEN and DMAREN must be set in separate register writes;
        // setting both in a single write hangs the CORDIC+DMA on STM32H563.
        T::regs().csr().modify(|v| v.set_dmawen(true));
        T::regs().csr().modify(|v| v.set_dmaren(true));

        // Same H563 constraint: clear DMAWEN and DMAREN in separate writes.
        let _on_drop = OnDrop::new(|| {
            T::regs().csr().modify(|v| v.set_dmawen(false));
            T::regs().csr().modify(|v| v.set_dmaren(false));
        });

        unsafe {
            let mut write_channel = dma::Channel::new(write_dma.reborrow(), irq);
            let write_transfer =
                write_channel.write(write_req, arg, T::regs().wdata().as_ptr() as *mut _, Default::default());

            let mut read_channel = dma::Channel::new(read_dma.reborrow(), irq);
            let read_transfer = read_channel.read(
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
    /// Run a blocking CORDIC calculation in q1.15 format.
    ///
    /// In q1.15 mode, each WDATA write / RDATA read contains two packed 16-bit values,
    /// so `nargs` and `nres` are always 1 (one register access = two values).
    ///
    /// After this call, the CSR is restored to the 32-bit state from the current [`Config`].
    #[inline]
    pub fn blocking_calc_16bit(&mut self, arg: &[u32], res: &mut [u32]) -> Result<usize, CordicError> {
        if arg.is_empty() {
            return Ok(0);
        }

        if arg.len() > res.len() {
            return Err(CordicError::ResultLengthNotEnough);
        }

        let res_cnt = arg.len();

        // In q1.15 mode, 1 write/read to access 2 arguments/results
        T::regs().csr().modify(|v| {
            v.set_nargs(vals::Num::NUM1);
            v.set_nres(vals::Num::NUM1);
            v.set_argsize(vals::Size::BITS16);
            v.set_ressize(vals::Size::BITS16);
        });

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

        // Restore CSR to 32-bit state matching current Config
        self.restore_csr_from_config();

        Ok(res_cnt)
    }

    /// Run an async CORDIC calculation in q1.15 format.
    ///
    /// In q1.15 mode, each WDATA write / RDATA read contains two packed 16-bit values,
    /// so `nargs` and `nres` are always 1 (one register access = two values).
    ///
    /// After this call, the CSR is restored to the 32-bit state from the current [`Config`].
    #[inline]
    pub async fn async_calc_16bit<'a, W, R>(
        &mut self,
        mut write_dma: Peri<'a, W>,
        mut read_dma: Peri<'a, R>,
        irq: impl crate::interrupt::typelevel::Binding<W::Interrupt, crate::dma::InterruptHandler<W>>
        + crate::interrupt::typelevel::Binding<R::Interrupt, crate::dma::InterruptHandler<R>>
        + 'a,
        arg: &[u32],
        res: &mut [u32],
    ) -> Result<usize, CordicError>
    where
        W: WriteDma<T>,
        R: ReadDma<T>,
    {
        if arg.is_empty() {
            return Ok(0);
        }

        if arg.len() > res.len() {
            return Err(CordicError::ResultLengthNotEnough);
        }

        let res_cnt = arg.len();

        let active_res_buf = &mut res[..res_cnt];

        // In q1.15 mode, 1 write/read to access 2 arguments/results
        T::regs().csr().modify(|v| {
            v.set_nargs(vals::Num::NUM1);
            v.set_nres(vals::Num::NUM1);
            v.set_argsize(vals::Size::BITS16);
            v.set_ressize(vals::Size::BITS16);
        });

        let write_req = write_dma.request();
        let read_req = read_dma.request();

        // DMAWEN and DMAREN must be set in separate register writes;
        // setting both in a single write hangs the CORDIC+DMA on STM32H563.
        T::regs().csr().modify(|v| v.set_dmawen(true));
        T::regs().csr().modify(|v| v.set_dmaren(true));

        // Same H563 constraint: clear DMAWEN and DMAREN in separate writes.
        let _on_drop = OnDrop::new(|| {
            T::regs().csr().modify(|v| v.set_dmawen(false));
            T::regs().csr().modify(|v| v.set_dmaren(false));
        });

        unsafe {
            let mut write_channel = dma::Channel::new(write_dma.reborrow(), irq);
            let write_transfer =
                write_channel.write(write_req, arg, T::regs().wdata().as_ptr() as *mut _, Default::default());

            let mut read_channel = dma::Channel::new(read_dma.reborrow(), irq);
            let read_transfer = read_channel.read(
                read_req,
                T::regs().rdata().as_ptr() as *mut _,
                active_res_buf,
                Default::default(),
            );

            embassy_futures::join::join(write_transfer, read_transfer).await;
        }

        // Restore CSR to 32-bit state matching current Config
        self.restore_csr_from_config();

        Ok(res_cnt)
    }

    fn restore_csr_from_config(&self) {
        T::regs().csr().modify(|v| {
            v.set_nargs(match self.config.arg_count {
                AccessCount::One => vals::Num::NUM1,
                AccessCount::Two => vals::Num::NUM2,
            });
            v.set_nres(match self.config.res_count {
                AccessCount::One => vals::Num::NUM1,
                AccessCount::Two => vals::Num::NUM2,
            });
            v.set_argsize(vals::Size::BITS32);
            v.set_ressize(vals::Size::BITS32);
        });
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
