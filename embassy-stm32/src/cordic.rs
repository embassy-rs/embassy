//! CORDIC co-processor

use crate::peripherals;
use embassy_hal_internal::{into_ref, Peripheral, PeripheralRef};

pub use enums::*;

mod enums {
    /// CORDIC function
    #[allow(missing_docs)]
    #[derive(Clone, Copy)]
    pub enum Function {
        Cos = 0,
        Sin,
        Phase,
        Modulus,
        Arctan,
        Cosh,
        Sinh,
        Arctanh,
        Ln,
        Sqrt,
    }

    /// CORDIC precision
    #[allow(missing_docs)]
    #[derive(Clone, Copy)]
    pub enum Precision {
        Iters4 = 1,
        Iters8,
        Iters12,
        Iters16,
        Iters20,
        Iters24,
        Iters28,
        Iters32,
        Iters36,
        Iters40,
        Iters44,
        Iters48,
        Iters52,
        Iters56,
        Iters60,
    }

    /// CORDIC scale
    #[allow(non_camel_case_types)]
    #[allow(missing_docs)]
    #[derive(Clone, Copy, Default)]
    pub enum Scale {
        #[default]
        A1_R1 = 0,
        A1o2_R2,
        A1o4_R4,
        A1o8_R8,
        A1o16_R16,
        A1o32_R32,
        A1o64_R64,
        A1o128_R128,
    }

    /// CORDIC argument/result count
    #[allow(missing_docs)]
    #[derive(Clone, Copy, Default)]
    pub enum Count {
        #[default]
        One,
        Two,
    }

    /// CORDIC argument/result data width
    #[allow(missing_docs)]
    #[derive(Clone, Copy)]
    pub enum Width {
        Bits32,
        Bits16,
    }

    /// Cordic driver running mode
    #[derive(Clone, Copy)]
    pub enum Mode {
        /// After caculation start, a read to RDATA register will block AHB until the caculation finished
        ZeroOverhead,

        /// Use CORDIC interrupt to trigger a read result value
        Interrupt,

        /// Use DMA to write/read value
        Dma,
    }
}

/// Low-level CORDIC access.
#[cfg(feature = "unstable-pac")]
pub mod low_level {
    pub use super::sealed::*;
}

pub(crate) mod sealed {
    use super::*;
    use crate::pac::cordic::vals;

    /// Cordic instance
    pub trait Instance {
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
        fn set_argument_count(&self, n: Count) {
            Self::regs().csr().modify(|v| {
                v.set_nargs(match n {
                    Count::One => vals::Num::NUM1,
                    Count::Two => vals::Num::NUM2,
                })
            })
        }

        /// Set NRES value
        fn set_result_count(&self, n: Count) {
            Self::regs().csr().modify(|v| {
                v.set_nres(match n {
                    Count::One => vals::Num::NUM1,
                    Count::Two => vals::Num::NUM2,
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
                        self.cordic.write_argument(f64_to_q1_31(arg));
                        output[cnt] = q1_31_to_f64(self.cordic.read_result());
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

macro_rules! floating_fixed_convert {
    ($f_to_q:ident, $q_to_f:ident, $unsigned_bin_typ:ty, $signed_bin_typ:ty, $float_ty:ty, $offset:literal, $min_positive:literal) => {
        /// convert float point to fixed point format
        pub fn $f_to_q(value: $float_ty) -> $unsigned_bin_typ {
            const MIN_POSITIVE: $float_ty = unsafe { core::mem::transmute($min_positive) };

            assert!(
                (-1.0 as $float_ty) <= value,
                "input value {} should be equal or greater than -1",
                value
            );

            let value = if value == 1.0 as $float_ty{
                (1.0 as $float_ty) - MIN_POSITIVE
            } else {
                assert!(
                    value <= (1.0 as $float_ty) - MIN_POSITIVE,
                    "input value {} should be equal or less than 1-2^(-{})",
                    value, $offset
                );
                value
            };

            (value * ((1 as $unsigned_bin_typ << $offset) as $float_ty)) as $unsigned_bin_typ
        }

        #[inline(always)]
        /// convert fixed point to float point format
        pub fn $q_to_f(value: $unsigned_bin_typ) -> $float_ty {
            // It's needed to convert from unsigned to signed first, for correct result.
            -(value as $signed_bin_typ as $float_ty) / ((1 as $unsigned_bin_typ << $offset) as $float_ty)
        }
    };
}

floating_fixed_convert!(
    f64_to_q1_31,
    q1_31_to_f64,
    u32,
    i32,
    f64,
    31,
    0x3E00_0000_0000_0000u64 // binary form of 1f64^(-31)
);

floating_fixed_convert!(
    f32_to_q1_15,
    q1_15_to_f32,
    u16,
    i16,
    f32,
    15,
    0x3800_0000u32 // binary form of 1f32^(-15)
);
