//! MATHACL
//!
//! This HAL implements mathematical calculations performed by the CPU.

#![macro_use]

use core::f32::consts::PI;
use core::marker::PhantomData;

use embassy_hal_internal::PeripheralType;
use micromath::F32Ext;

use crate::Peri;
use crate::pac::mathacl::{Mathacl as Regs, vals};

const ERROR_TOLERANCE: f32 = 0.00001;

pub enum Precision {
    High = 31,
    Medium = 15,
    Low = 1,
}

/// Error type for Mathacl operations.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    ValueInWrongRange,
    DivideByZero,
    FaultIQTypeFormat,
    IQTypeError(IQTypeError),
}

pub struct Mathacl<'d> {
    regs: &'static Regs,
    _phantom: PhantomData<&'d mut ()>,
}

impl<'d> Mathacl<'d> {
    /// Mathacl initialization.
    pub fn new<T: Instance>(_instance: Peri<'d, T>) -> Self {
        // Init power
        T::regs().gprcm(0).rstctl().write(|w| {
            w.set_resetstkyclr(vals::Resetstkyclr::CLR);
            w.set_resetassert(vals::Resetassert::ASSERT);
            w.set_key(vals::ResetKey::KEY);
        });

        // Enable power
        T::regs().gprcm(0).pwren().write(|w| {
            w.set_enable(true);
            w.set_key(vals::PwrenKey::KEY);
        });

        // init delay, 16 cycles
        cortex_m::asm::delay(16);

        Self {
            regs: T::regs(),
            _phantom: PhantomData,
        }
    }

    /// Internal helper SINCOS function.
    fn sincos(&mut self, rad: f32, precision: Precision, sin: bool) -> Result<f32, Error> {
        if rad > PI || rad < -PI {
            return Err(Error::ValueInWrongRange);
        }

        // make division using mathacl
        let native = self.div_iq(IQType::from_f32(rad, 15, true)?, IQType::from_f32(PI, 15, true)?)?;

        self.regs.ctl().write(|w| {
            w.set_func(vals::Func::SINCOS);
            w.set_numiter(precision as u8);
        });

        // integer part has to be 0 bits
        self.regs.op1().write(|w| {
            w.set_data(IQType::from_f32(native, 0, true).unwrap().to_reg());
        });

        // check if done
        while self.regs.status().read().busy() == vals::Busy::NOTDONE {}

        match sin {
            true => Ok(IQType::from_reg(self.regs.res2().read().data(), 0, true)
                .unwrap()
                .to_f32()),
            false => Ok(IQType::from_reg(self.regs.res1().read().data(), 0, true)
                .unwrap()
                .to_f32()),
        }
    }

    /// Calsulates trigonometric sine operation in the range [-1,1) with a give precision.
    pub fn sin(&mut self, rad: f32, precision: Precision) -> Result<f32, Error> {
        self.sincos(rad, precision, true)
    }

    /// Calsulates trigonometric cosine operation in the range [-1,1) with a give precision.
    pub fn cos(&mut self, rad: f32, precision: Precision) -> Result<f32, Error> {
        self.sincos(rad, precision, false)
    }

    pub fn div_i32(&mut self, dividend: i32, divisor: i32) -> Result<i32, Error> {
        if divisor == 0 {
            return Err(Error::DivideByZero);
        } else if dividend == 0 {
            return Ok(0);
        }
        let signed = true;

        self.regs.ctl().write(|w| {
            w.set_func(vals::Func::DIV);
            w.set_optype(signed);
        });

        self.regs.op2().write(|w| {
            w.set_data(divisor as u32);
        });

        self.regs.op1().write(|w| {
            w.set_data(dividend as u32);
        });

        // check if done
        while self.regs.status().read().busy() == vals::Busy::NOTDONE {}

        // read quotient
        Ok(self.regs.res1().read().data() as i32)
    }

    pub fn div_u32(&mut self, dividend: u32, divisor: u32) -> Result<u32, Error> {
        if divisor == 0 {
            return Err(Error::DivideByZero);
        } else if dividend == 0 {
            return Ok(0);
        }
        let signed = false;

        self.regs.ctl().write(|w| {
            w.set_func(vals::Func::DIV);
            w.set_optype(signed);
        });

        self.regs.op2().write(|w| {
            w.set_data(divisor);
        });

        self.regs.op1().write(|w| {
            w.set_data(dividend);
        });

        // check if done
        while self.regs.status().read().busy() == vals::Busy::NOTDONE {}

        // read quotient
        Ok(self.regs.res1().read().data())
    }

    /// Divide function (DIV) computes with a known dividend and divisor.
    pub fn div_iq(&mut self, dividend: IQType, divisor: IQType) -> Result<f32, Error> {
        let divisor_value = divisor.to_f32();
        if -ERROR_TOLERANCE < divisor_value && divisor_value < ERROR_TOLERANCE {
            return Err(Error::DivideByZero);
        }

        // check if both numbers have the same number of bits
        if dividend.f_bits != divisor.f_bits {
            return Err(Error::FaultIQTypeFormat);
        }

        // dividen and divisor must have the same signedness
        if dividend.signed ^ divisor.signed {
            return Err(Error::FaultIQTypeFormat);
        }

        self.regs.ctl().write(|w| {
            w.set_func(vals::Func::DIV);
            w.set_optype(dividend.signed);
            w.set_qval(dividend.f_bits.into());
        });

        self.regs.op2().write(|w| {
            w.set_data(divisor.to_reg());
        });

        self.regs.op1().write(|w| {
            w.set_data(dividend.to_reg());
        });

        // check if done
        while self.regs.status().read().busy() == vals::Busy::NOTDONE {}

        // read quotient
        return Ok(
            IQType::from_reg(self.regs.res1().read().data(), dividend.i_bits.into(), dividend.signed)
                .unwrap()
                .to_f32(),
        );
    }
}

pub(crate) trait SealedInstance {
    fn regs() -> &'static Regs;
}

/// Mathacl instance trait
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType {}

macro_rules! impl_mathacl_instance {
    ($instance: ident) => {
        impl crate::mathacl::SealedInstance for crate::peripherals::$instance {
            fn regs() -> &'static crate::pac::mathacl::Mathacl {
                &crate::pac::$instance
            }
        }

        impl crate::mathacl::Instance for crate::peripherals::$instance {}
    };
}

/// Error type for Mathacl operations.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum IQTypeError {
    FaultySignParameter,
    IntPartIsTrimmed,
}

impl From<IQTypeError> for Error {
    fn from(e: IQTypeError) -> Self {
        Error::IQTypeError(e)
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct IQType {
    i_bits: u8,
    f_bits: u8,
    negative: bool,
    signed: bool,
    i_data: u32,
    f_data: u32,
}

/// IQType implements 32-bit fixed point numbers with configurable integer and fractional parts.
impl IQType {
    pub fn from_reg(data: u32, i_bits: u8, signed: bool) -> Result<Self, IQTypeError> {
        // check if negative
        let negative = signed && ((1u32 << 31) & data != 0);

        // total bit count
        let total_bits = if signed { 31 } else { 32 };

        // number of fractional bits
        let f_bits = total_bits - i_bits;

        // Compute masks
        let max_mask = if signed { 0x7FFFFFFF } else { 0xFFFFFFFF };
        let (i_mask, f_mask) = if i_bits == 0 {
            (0, max_mask)
        } else if i_bits == total_bits {
            (max_mask, 0)
        } else {
            ((1u32 << i_bits) - 1, (1u32 << f_bits) - 1)
        };

        // Compute i_data and f_data
        let mut i_data = if i_bits == 0 {
            0
        } else if i_bits == total_bits {
            data & i_mask
        } else {
            (data >> f_bits) & i_mask
        };
        let mut f_data = data & f_mask;

        // if negative, do 2’s compliment
        if negative {
            i_data = !i_data & i_mask;
            f_data = (!f_data & f_mask) + 1;
        }

        Ok(Self {
            i_bits,
            f_bits,
            negative,
            signed,
            i_data,
            f_data,
        })
    }

    pub fn from_f32(data: f32, i_bits: u8, signed: bool) -> Result<Self, IQTypeError> {
        // check if negative
        let negative = data < 0.0;

        if !signed && negative {
            return Err(IQTypeError::FaultySignParameter);
        }

        // absolute value
        let abs = if data < 0.0 { -data } else { data };

        // total bit count
        let total_bits = if signed { 31 } else { 32 };

        // number of fractional bits
        let f_bits: u8 = total_bits - i_bits;

        let abs_floor = abs.floor();
        let i_data = abs_floor as u32;
        let f_data = ((abs - abs_floor) * (1u32 << f_bits) as f32).round() as u32;

        // Handle trimming integer part
        if i_bits == 0 && i_data > 0 {
            return Err(IQTypeError::IntPartIsTrimmed);
        }

        Ok(Self {
            i_bits,
            f_bits,
            negative,
            signed,
            i_data,
            f_data,
        })
    }

    pub fn to_f32(&self) -> f32 {
        let mut value = (self.i_data as f32) + (self.f_data as f32) / (1u32 << self.f_bits as u8) as f32;
        if self.negative {
            value = -value;
        }
        return value;
    }

    pub fn to_reg(&self) -> u32 {
        let mut res: u32 = 0;

        // total bit count
        let total_bits: u8 = if self.signed { 31 } else { 32 };

        // Compute masks
        let max_mask = if self.signed { 0x7FFFFFFF } else { 0xFFFFFFFF };
        let (i_mask, f_mask) = if self.i_bits == 0 {
            (0, max_mask)
        } else if self.i_bits == total_bits {
            (max_mask, 0)
        } else {
            ((1u32 << self.i_bits) - 1, (1u32 << self.f_bits) - 1)
        };

        // calculate result
        if self.i_bits > 0 {
            res = self.i_data << self.f_bits & (i_mask << self.f_bits);
        }

        if self.f_bits > 0 {
            res = (self.f_data & f_mask) | res;
        }

        // if negative, do 2’s compliment
        if self.negative {
            res = !res + 1;
        }
        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mathacl_iqtype_errors() {
        // integer part trimmed
        let mut test_float = 1.0;
        assert_eq!(
            IQType::from_f32(test_float, 0, true),
            Err(IQTypeError::IntPartIsTrimmed)
        );
        // negative value for unsigned type
        test_float = -1.0;
        assert_eq!(
            IQType::from_f32(test_float, 1, false),
            Err(IQTypeError::FaultySignParameter)
        );
    }

    #[test]
    fn mathacl_iqtype_f32_to_f32() {
        assert_eq!(IQType::from_f32(0.0, 15, true).unwrap().to_f32(), 0.0);
        assert_eq!(IQType::from_f32(0.0, 16, false).unwrap().to_f32(), 0.0);

        assert_eq!(IQType::from_f32(1.5, 16, false).unwrap().to_f32(), 1.5);
        assert_eq!(IQType::from_f32(1.5, 15, true).unwrap().to_f32(), 1.5);
        assert_eq!(IQType::from_f32(-1.5, 15, true).unwrap().to_f32(), -1.5);
    }

    #[test]
    fn mathacl_iqtype_reg_to_reg() {
        assert_eq!(IQType::from_reg(0x0, 15, true).unwrap().to_reg(), 0x0);
        assert_eq!(IQType::from_reg(0x0, 16, false).unwrap().to_reg(), 0x0);

        assert_eq!(IQType::from_reg(0x00018000, 15, true).unwrap().to_reg(), 0x00018000);
        assert_eq!(IQType::from_reg(0x00018000, 16, false).unwrap().to_reg(), 0x00018000);
        assert_eq!(IQType::from_reg(0xFFFE5556, 15, true).unwrap().to_reg(), 0xFFFE5556);
    }

    #[test]
    fn mathacl_iqtype_f32_to_register() {
        let mut test_float = 0.0;
        assert_eq!(IQType::from_f32(test_float, 15, true).unwrap().to_reg(), 0x0);
        assert_eq!(IQType::from_f32(test_float, 16, false).unwrap().to_reg(), 0x0);

        test_float = 1.5;
        assert_eq!(IQType::from_f32(test_float, 15, true).unwrap().to_reg(), 0x00018000);
        assert_eq!(IQType::from_f32(test_float, 16, false).unwrap().to_reg(), 0x00018000);

        test_float = -1.5;
        assert_eq!(IQType::from_f32(test_float, 15, true).unwrap().to_reg(), 0xFFFE8000);

        test_float = 1.666657;
        assert_eq!(IQType::from_f32(test_float, 15, true).unwrap().to_reg(), 0x0001AAAA);
        assert_eq!(IQType::from_f32(test_float, 16, false).unwrap().to_reg(), 0x0001AAAA);

        test_float = -1.666657;
        assert_eq!(IQType::from_f32(test_float, 15, true).unwrap().to_reg(), 0xFFFE5556);
    }

    #[test]
    fn mathacl_iqtype_register_to_signed_f32() {
        let mut test_u32: u32 = 0x7FFFFFFF;

        let mut result = IQType::from_reg(test_u32, 0, true).unwrap().to_f32();
        assert!(result < 1.0 + ERROR_TOLERANCE && result > 1.0 - ERROR_TOLERANCE);

        test_u32 = 0x0;
        result = IQType::from_reg(test_u32, 0, true).unwrap().to_f32();
        assert!(result < 0.0 + ERROR_TOLERANCE && result > 0.0 - ERROR_TOLERANCE);

        test_u32 = 0x0001AAAA;
        result = IQType::from_reg(test_u32, 15, true).unwrap().to_f32();
        assert!(result < 1.666657 + ERROR_TOLERANCE && result > 1.666657 - ERROR_TOLERANCE);

        test_u32 = 0xFFFE5556;
        result = IQType::from_reg(test_u32, 15, true).unwrap().to_f32();
        assert!(result < -1.666657 + ERROR_TOLERANCE && result > -1.666657 - ERROR_TOLERANCE);
    }
}
