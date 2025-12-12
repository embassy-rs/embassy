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

/// Serial error
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    ValueInWrongRange,
    DivideByZero,
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
        let native = self.div(rad, PI)?;

        self.regs.ctl().write(|w| {
            w.set_func(vals::Func::SINCOS);
            w.set_numiter(precision as u8);
        });

        // fractional part has to be 31 bits
        match signed_f32_to_register(native, vals::Qval::Q31) {
            Ok(val) => self.regs.op1().write(|w| {
                w.set_data(val);
            }),
            Err(er) => return Err(er),
        };

        // check if done
        while self.regs.status().read().busy() == vals::Busy::NOTDONE {}

        match sin {
            true => register_to_signed_f32(self.regs.res2().read().data(), vals::Qval::Q31),
            false => register_to_signed_f32(self.regs.res1().read().data(), vals::Qval::Q31),
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

    /// Divide function (DIV) computes with a known dividend and divisor.
    pub fn div(&mut self, dividend: f32, divisor: f32) -> Result<f32, Error> {
        if -ERROR_TOLERANCE < divisor && divisor < ERROR_TOLERANCE {
            return Err(Error::DivideByZero);
        }

        // assume all input data is signed &
        // has 16 bits of fractional part
        let optype = true;
        let q_val = vals::Qval::Q16;

        self.regs.ctl().write(|w| {
            w.set_func(vals::Func::DIV);
            w.set_optype(optype);
            w.set_qval(q_val);
        });

        match signed_f32_to_register(divisor, q_val) {
            Ok(val) => self.regs.op2().write(|w| {
                w.set_data(val);
            }),
            Err(er) => return Err(er),
        };

        match signed_f32_to_register(dividend, q_val) {
            Ok(val) => self.regs.op1().write(|w| {
                w.set_data(val);
            }),
            Err(er) => return Err(er),
        };

        // check if done
        while self.regs.status().read().busy() == vals::Busy::NOTDONE {}

        // read quotient
        let res1 = self.regs.res1().read().data();
        register_to_signed_f32(res1, q_val)
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

/// Convert f32 data to understandable by M0 format.
fn signed_f32_to_register(data: f32, m_bits: vals::Qval) -> Result<u32, Error> {
    let mut res: u32 = 0;
    // check if negative
    let negative = data < 0.0;

    // absolute value for extraction
    let abs = data.abs();

    // total integer bit count
    let total_bits = 31;

    // number of fractional bits
    let n_bits = total_bits - m_bits as u8;

    // Compute masks
    let (n_mask, m_mask) = if n_bits == 0 {
        (0, 0x7FFFFFFF)
    } else if n_bits == 31 {
        (0x7FFFFFFF, 0)
    } else {
        ((1u32 << n_bits) - 1, (1u32 << m_bits as u8) - 1)
    };

    let abs_floor = abs.floor();
    let n = abs_floor as u32;
    let mut m = ((abs - abs_floor) * (1u32 << m_bits as u8) as f32).round() as u32;

    // Handle trimming integer part
    if n_bits == 0 && n > 0 {
        m = 0x7FFFFFFF;
    }

    // calculate result
    if n_bits > 0 {
        res = n << m_bits as u8 & (n_mask << m_bits as u8);
    }
    if m_bits as u8 > 0 {
        res = (m & m_mask) | res;
    }

    // if negative, do 2’s compliment
    if negative {
        res = !res + 1;
    }
    Ok(res)
}

/// Reversely converts M0-register format to native f32.
fn register_to_signed_f32(data: u32, m_bits: vals::Qval) -> Result<f32, Error> {
    // total integer bit count
    let total_bits = 31;

    let negative = (data >> 31) == 1;

    // number of fractional bits
    let n_bits = total_bits - m_bits as u8;

    // Compute masks
    let (n_mask, m_mask) = if n_bits == 0 {
        (0, 0x7FFFFFFF)
    } else if n_bits == 31 {
        (0x7FFFFFFF, 0)
    } else {
        ((1u32 << n_bits) - 1, (1u32 << m_bits as u8) - 1)
    };

    // Compute n and m
    let mut n = if n_bits == 0 {
        0
    } else if m_bits as u8 >= 32 {
        data & n_mask
    } else {
        (data >> m_bits as u8) & n_mask
    };
    let mut m = data & m_mask;

    // if negative, do 2’s compliment
    if negative {
        n = !n & n_mask;
        m = (!m & m_mask) + 1;
    }

    let mut value = (n as f32) + (m as f32) / (1u32 << m_bits as u8) as f32;
    if negative {
        value = -value;
    }
    return Ok(value);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mathacl_signed_f32_to_register() {
        let mut test_float = 1.0;
        assert_eq!(signed_f32_to_register(test_float, vals::Qval::Q31).unwrap(), 0x7FFFFFFF);

        test_float = 0.0;
        assert_eq!(signed_f32_to_register(test_float, vals::Qval::Q31).unwrap(), 0x0);

        test_float = -1.0;
        assert_eq!(signed_f32_to_register(test_float, vals::Qval::Q31).unwrap(), 0x80000001);

        test_float = 1.666657;
        assert_eq!(signed_f32_to_register(test_float, vals::Qval::Q16).unwrap(), 0x0001AAAA);

        test_float = -1.666657;
        assert_eq!(signed_f32_to_register(test_float, vals::Qval::Q16).unwrap(), 0xFFFE5556);
    }

    #[test]
    fn mathacl_register_to_signed_f32() {
        let mut test_u32: u32 = 0x7FFFFFFF;

        let mut result = register_to_signed_f32(test_u32, vals::Qval::Q31).unwrap();
        assert!(result < 1.0 + ERROR_TOLERANCE && result > 1.0 - ERROR_TOLERANCE);

        test_u32 = 0x0;
        result = register_to_signed_f32(test_u32, vals::Qval::Q31).unwrap();
        assert!(result < 0.0 + ERROR_TOLERANCE && result > 0.0 - ERROR_TOLERANCE);

        test_u32 = 0x0001AAAA;
        result = register_to_signed_f32(test_u32, vals::Qval::Q16).unwrap();
        assert!(result < 1.666657 + ERROR_TOLERANCE && result > 1.666657 - ERROR_TOLERANCE);

        test_u32 = 0xFFFE5556;
        result = register_to_signed_f32(test_u32, vals::Qval::Q16).unwrap();
        assert!(result < -1.666657 + ERROR_TOLERANCE && result > -1.666657 - ERROR_TOLERANCE);
    }
}
