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
    NBitsTooBig,
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
        self.regs.ctl().write(|w| {
            w.set_func(vals::Func::SINCOS);
            w.set_numiter(precision as u8);
        });

        if rad > PI || rad < -PI {
            return Err(Error::ValueInWrongRange);
        }

        // TODO: make f32 division on CPU
        let native = rad / PI;

        match signed_f32_to_register(native, 0) {
            Ok(val) => self.regs.op1().write(|w| {
                w.set_data(val);
            }),
            Err(er) => return Err(er),
        };

        // check if done
        while self.regs.status().read().busy() == vals::Busy::NOTDONE {}

        match sin {
            true => register_to_signed_f32(self.regs.res2().read().data(), 0),
            false => register_to_signed_f32(self.regs.res1().read().data(), 0),
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
fn signed_f32_to_register(data: f32, n_bits: u8) -> Result<u32, Error> {
    let mut res: u32 = 0;
    // check if negative
    let negative = data < 0.0;

    // absolute value for extraction
    let abs = data.abs();

    // total integer bit count
    let total_bits = 31;

    // Validate n_bits
    if n_bits > 31 {
        return Err(Error::NBitsTooBig);
    }

    // number of fractional bits
    let shift = total_bits - n_bits;

    // Compute masks
    let (n_mask, m_mask) = if n_bits == 0 {
        (0, 0x7FFFFFFF)
    } else if n_bits == 31 {
        (0x7FFFFFFF, 0)
    } else {
        ((1u32 << n_bits) - 1, (1u32 << shift) - 1)
    };

    // calc. integer(n) & fractional(m) parts
    let n = abs.floor() as u32;
    let mut m = ((abs - abs.floor()) * (1u32 << shift) as f32).round() as u32;

    // Handle trimming integer part
    if n_bits == 0 && n > 0 {
        m = 0x7FFFFFFF;
    }

    // calculate result
    if n_bits > 0 {
        res = n << shift & n_mask;
    }
    if shift > 0 {
        res = res | m & m_mask;
    }

    // if negative, do 2’s compliment
    if negative {
        res = !res + 1;
    }
    Ok(res)
}

/// Reversely converts M0-register format to native f32.
fn register_to_signed_f32(data: u32, n_bits: u8) -> Result<f32, Error> {
    // Validate n_bits
    if n_bits > 31 {
        return Err(Error::NBitsTooBig);
    }

    // total integer bit count
    let total_bits = 31;

    let negative = (data >> 31) == 1;

    // number of fractional bits
    let shift = total_bits - n_bits;

    // Compute masks
    let (n_mask, m_mask) = if n_bits == 0 {
        (0, 0x7FFFFFFF)
    } else if n_bits == 31 {
        (0x7FFFFFFF, 0)
    } else {
        ((1u32 << n_bits) - 1, (1u32 << shift) - 1)
    };

    // Compute n and m
    let mut n = if n_bits == 0 {
        0
    } else if shift >= 32 {
        data & n_mask
    } else {
        (data >> shift) & n_mask
    };
    let mut m = data & m_mask;

    // if negative, do 2’s compliment
    if negative {
        n = !n & n_mask;
        m = (!m & m_mask) + 1;
    }

    let mut value = (n as f32) + (m as f32) / (1u32 << shift) as f32;
    if negative {
        value = -value;
    }
    return Ok(value);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mathacl_convert_func_errors() {
        assert_eq!(signed_f32_to_register(0.0, 32), Err(Error::NBitsTooBig));
        assert_eq!(register_to_signed_f32(0, 32), Err(Error::NBitsTooBig));
    }

    #[test]
    fn mathacl_signed_f32_to_register() {
        let mut test_float = 1.0;
        assert_eq!(signed_f32_to_register(test_float, 0).unwrap(), 0x7FFFFFFF);

        test_float = 0.0;
        assert_eq!(signed_f32_to_register(test_float, 0).unwrap(), 0x0);

        test_float = -1.0;
        assert_eq!(signed_f32_to_register(test_float, 0).unwrap(), 0x80000001);
    }

    #[test]
    fn mathacl_register_to_signed_f32() {
        let mut test_u32: u32 = 0x7FFFFFFF;
        assert_eq!(register_to_signed_f32(test_u32, 0u8).unwrap(), 1.0);

        test_u32 = 0x0;
        assert_eq!(register_to_signed_f32(test_u32, 0u8).unwrap(), 0.0);

        test_u32 = 0x80000001;
        assert_eq!(register_to_signed_f32(test_u32, 0u8).unwrap(), -1.0);
    }
}
