//! Interpolator driver for RP2040/RP235x
//! The interpolator is a hardware peripheral that can perform linear interpolation and accumulation of values. It has two lanes, each with its own accumulator and base register.
//! The interpolator can be used for various applications, such as generating sequences of values or performing mathematical operations on data streams.

use core::marker::PhantomData;

use embassy_hal_internal::{Peri, PeripheralType};
use rp_pac::sio::regs::{Interp1CtrlLane0, Interp1CtrlLane1};

use crate::peripherals::{INTERP0, INTERP1};

/// Configuration struct for one lane of the interpolator
#[derive(Debug, Clone, Copy)]
pub struct LaneCtrl {
    /// Bit 22 - Only present on INTERP1 on each core. If CLAMP mode is enabled:  
    /// - LANE0 result is shifted and masked ACCUM0, clamped by a lower bound of  
    /// BASE0 and an upper bound of BASE1.  
    /// - Signedness of these comparisons is determined by LANE0_CTRL_SIGNED
    pub clamp: bool,
    /// Bit 21 - Only present on INTERP0 on each core. If BLEND mode is enabled:
    /// - LANE1 result is a linear interpolation between BASE0 and BASE1, controlled
    /// by the 8 LSBs of lane 1 shift and mask value (a fractional number between
    /// 0 and 255/256ths)
    /// - LANE0 result does not have BASE0 added (yields only
    /// the 8 LSBs of lane 1 shift+mask value)
    /// - FULL result does not have lane 1 shift+mask value added (BASE2 + lane 0 shift+mask)
    /// LANE1 SIGNED flag controls whether the interpolation is signed or unsigned.
    pub blend: bool,
    /// Bits 19:20 - ORed into bits 29:28 of the lane result presented to the processor on the bus.  
    /// No effect on the internal 32-bit datapath. Handy for using a lane to generate sequence  
    /// of pointers into flash or SRAM.
    pub force_msb: u8,
    /// Bit 18 - If 1, mask + shift is bypassed for LANE0 result. This does not affect FULL result.
    pub add_raw: bool,
    /// Bit 17 - If 1, feed the opposite lane's result into this lane's accumulator on POP.
    pub cross_result: bool,
    /// Bit 16 - If 1, feed the opposite lane's accumulator into this lane's shift + mask hardware.  
    /// Takes effect even if ADD_RAW is set (the CROSS_INPUT mux is before the shift+mask bypass)
    pub cross_input: bool,
    /// Bit 15 - If SIGNED is set, the shifted and masked accumulator value is sign-extended to 32 bits  
    /// before adding to BASE0, and LANE0 PEEK/POP appear extended to 32 bits when read by processor.
    pub signed: bool,
    /// Bits 10:14 - The most-significant bit allowed to pass by the mask (inclusive)  
    /// Setting MSB < LSB may cause chip to turn inside-out
    pub mask_msb: u8,
    /// Bits 5:9 - The least-significant bit allowed to pass by the mask (inclusive)
    pub mask_lsb: u8,
    /// Bits 0:4 - Logical right-shift applied to accumulator before masking
    pub shift: u8,
}

impl Default for LaneCtrl {
    fn default() -> Self {
        Self::new()
    }
}

impl LaneCtrl {
    /// Default configuration. Normal operation, unsigned, mask keeps all bits, no shift.
    pub const fn new() -> Self {
        Self {
            clamp: false,
            blend: false,
            force_msb: 0,
            add_raw: false,
            cross_result: false,
            cross_input: false,
            signed: false,
            mask_msb: 31,
            mask_lsb: 0,
            shift: 0,
        }
    }

    /// encode the configuration to be loaded in the ctrl register of one lane of an interpolator
    pub const fn encode(&self) -> u32 {
        core::assert!(!(self.blend && self.clamp));
        core::assert!(self.force_msb < 0b100);
        core::assert!(self.mask_msb < 0b100000);
        core::assert!(self.mask_lsb < 0b100000);
        core::assert!(self.mask_msb >= self.mask_lsb);
        core::assert!(self.shift < 0b100000);
        ((self.clamp as u32) << 22)
            | ((self.blend as u32) << 21)
            | ((self.force_msb as u32) << 19)
            | ((self.add_raw as u32) << 18)
            | ((self.cross_result as u32) << 17)
            | ((self.cross_input as u32) << 16)
            | ((self.signed as u32) << 15)
            | ((self.mask_msb as u32) << 10)
            | ((self.mask_lsb as u32) << 5)
            | (self.shift as u32)
    }

    /// decode the configuration from the ctrl register of one lane of an interpolator
    pub const fn decode(v: u32) -> Self {
        Self {
            clamp: v & (1 << 22) != 0,
            blend: v & (1 << 21) != 0,
            force_msb: ((v >> 19) & 0b11) as u8,
            add_raw: v & (1 << 18) != 0,
            cross_result: v & (1 << 17) != 0,
            cross_input: v & (1 << 16) != 0,
            signed: v & (1 << 15) != 0,
            mask_msb: ((v >> 10) & 0b11111) as u8,
            mask_lsb: ((v >> 5) & 0b11111) as u8,
            shift: (v & 0b11111) as u8,
        }
    }
}

pub(crate) trait SealedInstance {
    fn regs() -> crate::pac::sio::Interp;
}

/// Interpolator instance trait.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType {}

macro_rules! interpolator {
    ($interp:ident, $id:expr) => {
        impl SealedInstance for $interp {
            #[inline]
            fn regs() -> crate::pac::sio::Interp {
                crate::pac::SIO.interp($id)
            }
        }
        impl Instance for $interp {}
    };
}

interpolator!(INTERP0, 0);
interpolator!(INTERP1, 1);

/// Interpolator Driver
pub struct Interpolator<'d, T: Instance> {
    phantom: PhantomData<&'d mut T>,
}

impl<'d, T: Instance> Interpolator<'d, T> {
    /// Create a new interpolator instance from the peripheral type.
    pub fn new(_peri: Peri<'d, T>) -> Self {
        Self { phantom: PhantomData }
    }

    /// Read the interpolator result (Result 2 in the datasheet), and simultaneously write lane results to both accumulators.
    pub fn pop(&mut self) -> u32 {
        T::regs().pop_full().read()
    }
    /// Read the interpolator result (Result 2 in the datasheet) without altering any internal state
    pub fn peek(&self) -> u32 {
        T::regs().peek_full().read()
    }
    /// Write to the interpolator Base register (Base2 in the datasheet)
    pub fn set_base(&mut self, v: u32) {
        T::regs().base2().write(|w| *w = v)
    }
    /// Read the interpolator Base register (Base2 in the datasheet)
    pub fn get_base(&self) -> u32 {
        T::regs().base2().read()
    }
    /// Write the lower 16 bits to BASE0 and the upper bits to BASE1 simultaneously. Each half is sign-extended to 32 bits if that lane's SIGNED flag is set
    pub fn set_base_1and0(&mut self, v: u32) {
        T::regs().base_1and0().write(|w| *w = v)
    }

    /// Access Lane 0
    pub fn lane0(&mut self) -> InterpolatorLane0<'_, T> {
        InterpolatorLane0 { phantom: PhantomData }
    }

    /// Access Lane 1
    pub fn lane1(&mut self) -> InterpolatorLane1<'_, T> {
        InterpolatorLane1 { phantom: PhantomData }
    }
}

/// Lane 0
pub struct InterpolatorLane0<'d, T: Instance> {
    phantom: PhantomData<&'d mut T>,
}

// impl<'d, T: Instance> Lane for InterpolatorLane0<'d, T> {
impl<'d, T: Instance> InterpolatorLane0<'d, T> {
    ///Read the lane result, and simultaneously write lane results to both accumulators.
    pub fn pop(&mut self) -> u32 {
        T::regs().pop_lane0().read()
    }

    ///Read the lane result without altering any internal state
    pub fn peek(&self) -> u32 {
        T::regs().peek_lane0().read()
    }

    ///Write a value to the accumulator
    pub fn set_accum(&mut self, v: u32) {
        T::regs().accum0().write(|w| *w = v)
    }

    ///Read the value from the accumulator
    pub fn get_accum(&self) -> u32 {
        T::regs().accum0().read()
    }

    ///Add the value to the accumulator register
    pub fn add_accum(&mut self, v: u32) {
        T::regs().accum0_add().write(|w| w.set_interp1_accum0_add(v))
    }

    ///Write a value to the base register
    pub fn set_base(&mut self, v: u32) {
        T::regs().base0().write(|w| *w = v)
    }

    ///Read the value from the base register
    pub fn get_base(&self) -> u32 {
        T::regs().base0().read()
    }

    ///Write to the control register
    pub fn set_ctrl(&mut self, v: LaneCtrl) {
        T::regs().ctrl_lane0().write(|w| *w = Interp1CtrlLane0(v.encode()))
    }

    ///Read from the control register
    pub fn get_ctrl(&self) -> LaneCtrl {
        LaneCtrl::decode(T::regs().ctrl_lane0().read().0)
    }

    ///Read the raw shift and mask value (BASE register not added)
    pub fn read_raw(&self) -> u32 {
        T::regs().accum0_add().read().interp1_accum0_add()
    }
}

/// Lane 1
pub struct InterpolatorLane1<'d, T: Instance> {
    phantom: PhantomData<&'d mut T>,
}

impl<'d, T: Instance> InterpolatorLane1<'d, T> {
    ///Read the lane result, and simultaneously write lane results to both accumulators.
    pub fn pop(&mut self) -> u32 {
        T::regs().pop_lane1().read()
    }

    ///Read the lane result without altering any internal state
    pub fn peek(&self) -> u32 {
        T::regs().peek_lane1().read()
    }

    ///Write a value to the accumulator
    pub fn set_accum(&mut self, v: u32) {
        T::regs().accum1().write(|w| *w = v)
    }

    ///Read the value from the accumulator
    pub fn get_accum(&self) -> u32 {
        T::regs().accum1().read()
    }

    ///Add the value to the accumulator register
    pub fn add_accum(&mut self, v: u32) {
        T::regs().accum1_add().write(|w| w.set_interp1_accum1_add(v))
    }

    ///Write a value to the base register
    pub fn set_base(&mut self, v: u32) {
        T::regs().base1().write(|w| *w = v)
    }

    ///Read the value from the base register
    pub fn get_base(&self) -> u32 {
        T::regs().base1().read()
    }

    ///Write to the control register
    pub fn set_ctrl(&mut self, v: LaneCtrl) {
        T::regs().ctrl_lane1().write(|w| *w = Interp1CtrlLane1(v.encode()))
    }

    ///Read from the control register
    pub fn get_ctrl(&self) -> LaneCtrl {
        LaneCtrl::decode(T::regs().ctrl_lane1().read().0)
    }

    ///Read the raw shift and mask value (BASE register not added)
    pub fn read_raw(&self) -> u32 {
        T::regs().accum1_add().read().interp1_accum1_add()
    }
}
