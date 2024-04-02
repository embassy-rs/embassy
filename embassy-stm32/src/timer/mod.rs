//! Timers, PWM, quadrature decoder.

#[cfg(not(stm32l0))]
pub mod complementary_pwm;
pub mod low_level;
pub mod qei;
pub mod simple_pwm;

use crate::interrupt;
use crate::rcc::RccPeripheral;

/// Timer channel.
#[derive(Clone, Copy)]
pub enum Channel {
    /// Channel 1.
    Ch1,
    /// Channel 2.
    Ch2,
    /// Channel 3.
    Ch3,
    /// Channel 4.
    Ch4,
}

impl Channel {
    /// Get the channel index (0..3)
    pub fn index(&self) -> usize {
        match self {
            Channel::Ch1 => 0,
            Channel::Ch2 => 1,
            Channel::Ch3 => 2,
            Channel::Ch4 => 3,
        }
    }
}

/// Amount of bits of a timer.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum TimerBits {
    /// 16 bits.
    Bits16,
    /// 32 bits.
    #[cfg(not(stm32l0))]
    Bits32,
}

/// Core timer instance.
pub trait CoreInstance: RccPeripheral + 'static {
    /// Interrupt for this timer.
    type Interrupt: interrupt::typelevel::Interrupt;

    /// Amount of bits this timer has.
    const BITS: TimerBits;

    /// Registers for this timer.
    ///
    /// This is a raw pointer to the register block. The actual register block layout varies depending on the timer type.
    fn regs() -> *mut ();
}
/// Cut-down basic timer instance.
pub trait BasicNoCr2Instance: CoreInstance {}
/// Basic timer instance.
pub trait BasicInstance: BasicNoCr2Instance {}

/// General-purpose 16-bit timer with 1 channel instance.
pub trait GeneralInstance1Channel: CoreInstance {}

/// General-purpose 16-bit timer with 2 channels instance.
pub trait GeneralInstance2Channel: GeneralInstance1Channel {}

/// General-purpose 16-bit timer with 4 channels instance.
pub trait GeneralInstance4Channel: BasicInstance + GeneralInstance2Channel {
    // SimplePwm<'d, T> is implemented for T: GeneralInstance4Channel
    // Advanced timers implement this trait, but the output needs to be
    // enabled explicitly.
    // To support general-purpose and advanced timers, this function is added
    // here defaulting to noop and overwritten for advanced timers.
    /// Enable timer outputs.
    fn enable_outputs(&self) {}
}

/// General-purpose 32-bit timer with 4 channels instance.
pub trait GeneralInstance32bit4Channel: GeneralInstance4Channel {}

/// Advanced 16-bit timer with 1 channel instance.
pub trait AdvancedInstance1Channel: BasicNoCr2Instance + GeneralInstance1Channel {
    /// Capture compare interrupt for this timer.
    type CaptureCompareInterrupt: interrupt::typelevel::Interrupt;
}
/// Advanced 16-bit timer with 2 channels instance.

pub trait AdvancedInstance2Channel: BasicInstance + GeneralInstance2Channel + AdvancedInstance1Channel {}

/// Advanced 16-bit timer with 4 channels instance.
pub trait AdvancedInstance4Channel: AdvancedInstance2Channel + GeneralInstance4Channel {}

pin_trait!(Channel1Pin, GeneralInstance4Channel);
pin_trait!(Channel2Pin, GeneralInstance4Channel);
pin_trait!(Channel3Pin, GeneralInstance4Channel);
pin_trait!(Channel4Pin, GeneralInstance4Channel);
pin_trait!(ExternalTriggerPin, GeneralInstance4Channel);

pin_trait!(Channel1ComplementaryPin, AdvancedInstance4Channel);
pin_trait!(Channel2ComplementaryPin, AdvancedInstance4Channel);
pin_trait!(Channel3ComplementaryPin, AdvancedInstance4Channel);
pin_trait!(Channel4ComplementaryPin, AdvancedInstance4Channel);

pin_trait!(BreakInputPin, AdvancedInstance4Channel);
pin_trait!(BreakInput2Pin, AdvancedInstance4Channel);

pin_trait!(BreakInputComparator1Pin, AdvancedInstance4Channel);
pin_trait!(BreakInputComparator2Pin, AdvancedInstance4Channel);

pin_trait!(BreakInput2Comparator1Pin, AdvancedInstance4Channel);
pin_trait!(BreakInput2Comparator2Pin, AdvancedInstance4Channel);

// Update Event trigger DMA for every timer
dma_trait!(UpDma, BasicInstance);

dma_trait!(Ch1Dma, GeneralInstance4Channel);
dma_trait!(Ch2Dma, GeneralInstance4Channel);
dma_trait!(Ch3Dma, GeneralInstance4Channel);
dma_trait!(Ch4Dma, GeneralInstance4Channel);

#[allow(unused)]
macro_rules! impl_core_timer {
    ($inst:ident, $bits:expr) => {
        impl CoreInstance for crate::peripherals::$inst {
            type Interrupt = crate::_generated::peripheral_interrupts::$inst::UP;

            const BITS: TimerBits = $bits;

            fn regs() -> *mut () {
                crate::pac::$inst.as_ptr()
            }
        }
    };
}

foreach_interrupt! {
    ($inst:ident, timer, TIM_BASIC, UP, $irq:ident) => {
        impl_core_timer!($inst, TimerBits::Bits16);
        impl BasicNoCr2Instance for crate::peripherals::$inst {}
        impl BasicInstance for crate::peripherals::$inst {}
    };

    ($inst:ident, timer, TIM_1CH, UP, $irq:ident) => {
        impl_core_timer!($inst, TimerBits::Bits16);
        impl BasicNoCr2Instance for crate::peripherals::$inst {}
        impl BasicInstance for crate::peripherals::$inst {}
        impl GeneralInstance1Channel for crate::peripherals::$inst {}
        impl GeneralInstance2Channel for crate::peripherals::$inst {}
        impl GeneralInstance4Channel for crate::peripherals::$inst {}
    };

    ($inst:ident, timer, TIM_2CH, UP, $irq:ident) => {
        impl_core_timer!($inst, TimerBits::Bits16);
        impl BasicNoCr2Instance for crate::peripherals::$inst {}
        impl BasicInstance for crate::peripherals::$inst {}
        impl GeneralInstance1Channel for crate::peripherals::$inst {}
        impl GeneralInstance2Channel for crate::peripherals::$inst {}
        impl GeneralInstance4Channel for crate::peripherals::$inst {}
    };

    ($inst:ident, timer, TIM_GP16, UP, $irq:ident) => {
        impl_core_timer!($inst, TimerBits::Bits16);
        impl BasicNoCr2Instance for crate::peripherals::$inst {}
        impl BasicInstance for crate::peripherals::$inst {}
        impl GeneralInstance1Channel for crate::peripherals::$inst {}
        impl GeneralInstance2Channel for crate::peripherals::$inst {}
        impl GeneralInstance4Channel for crate::peripherals::$inst {}
    };

    ($inst:ident, timer, TIM_GP32, UP, $irq:ident) => {
        impl_core_timer!($inst, TimerBits::Bits32);
        impl BasicNoCr2Instance for crate::peripherals::$inst {}
        impl BasicInstance for crate::peripherals::$inst {}
        impl GeneralInstance1Channel for crate::peripherals::$inst {}
        impl GeneralInstance2Channel for crate::peripherals::$inst {}
        impl GeneralInstance4Channel for crate::peripherals::$inst {}
        impl GeneralInstance32bit4Channel for crate::peripherals::$inst {}
    };

    ($inst:ident, timer, TIM_1CH_CMP, UP, $irq:ident) => {
        impl_core_timer!($inst, TimerBits::Bits16);
        impl BasicNoCr2Instance for crate::peripherals::$inst {}
        impl BasicInstance for crate::peripherals::$inst {}
        impl GeneralInstance1Channel for crate::peripherals::$inst {}
        impl GeneralInstance2Channel for crate::peripherals::$inst {}
        impl GeneralInstance4Channel for crate::peripherals::$inst { fn enable_outputs(&self) { set_moe::<Self>() }}
        impl AdvancedInstance1Channel for crate::peripherals::$inst { type CaptureCompareInterrupt = crate::_generated::peripheral_interrupts::$inst::CC; }
        impl AdvancedInstance2Channel for crate::peripherals::$inst {}
        impl AdvancedInstance4Channel for crate::peripherals::$inst {}
    };

    ($inst:ident, timer, TIM_2CH_CMP, UP, $irq:ident) => {
        impl_core_timer!($inst, TimerBits::Bits16);
        impl BasicNoCr2Instance for crate::peripherals::$inst {}
        impl BasicInstance for crate::peripherals::$inst {}
        impl GeneralInstance1Channel for crate::peripherals::$inst {}
        impl GeneralInstance2Channel for crate::peripherals::$inst {}
        impl GeneralInstance4Channel for crate::peripherals::$inst { fn enable_outputs(&self) { set_moe::<Self>() }}
        impl AdvancedInstance1Channel for crate::peripherals::$inst { type CaptureCompareInterrupt = crate::_generated::peripheral_interrupts::$inst::CC; }
        impl AdvancedInstance2Channel for crate::peripherals::$inst {}
        impl AdvancedInstance4Channel for crate::peripherals::$inst {}
    };

    ($inst:ident, timer, TIM_ADV, UP, $irq:ident) => {
        impl_core_timer!($inst, TimerBits::Bits16);
        impl BasicNoCr2Instance for crate::peripherals::$inst {}
        impl BasicInstance for crate::peripherals::$inst {}
        impl GeneralInstance1Channel for crate::peripherals::$inst {}
        impl GeneralInstance2Channel for crate::peripherals::$inst {}
        impl GeneralInstance4Channel for crate::peripherals::$inst { fn enable_outputs(&self) { set_moe::<Self>() }}
        impl AdvancedInstance1Channel for crate::peripherals::$inst { type CaptureCompareInterrupt = crate::_generated::peripheral_interrupts::$inst::CC; }
        impl AdvancedInstance2Channel for crate::peripherals::$inst {}
        impl AdvancedInstance4Channel for crate::peripherals::$inst {}
    };
}

#[cfg(not(stm32l0))]
#[allow(unused)]
fn set_moe<T: GeneralInstance4Channel>() {
    unsafe { crate::pac::timer::Tim1chCmp::from_ptr(T::regs()) }
        .bdtr()
        .modify(|w| w.set_moe(true));
}
