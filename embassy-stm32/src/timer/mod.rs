//! Timers, PWM, quadrature decoder.

use core::marker::PhantomData;

use embassy_sync::waitqueue::AtomicWaker;

#[cfg(not(stm32l0))]
pub mod complementary_pwm;
pub mod input_capture;
pub mod low_level;
pub mod pwm_input;
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

struct State {
    up_waker: AtomicWaker,
    cc_waker: [AtomicWaker; 4],
}

impl State {
    const fn new() -> Self {
        const NEW_AW: AtomicWaker = AtomicWaker::new();
        Self {
            up_waker: NEW_AW,
            cc_waker: [NEW_AW; 4],
        }
    }
}

trait SealedInstance: RccPeripheral {
    /// Async state for this timer
    fn state() -> &'static State;
}

/// Core timer instance.
#[allow(private_bounds)]
pub trait CoreInstance: SealedInstance + 'static {
    /// Update Interrupt for this timer.
    type UpdateInterrupt: interrupt::typelevel::Interrupt;

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
pub trait GeneralInstance1Channel: CoreInstance {
    /// Capture compare interrupt for this timer.
    type CaptureCompareInterrupt: interrupt::typelevel::Interrupt;
}

/// General-purpose 16-bit timer with 2 channels instance.
pub trait GeneralInstance2Channel: GeneralInstance1Channel {
    /// Trigger event interrupt for this timer.
    type TriggerInterrupt: interrupt::typelevel::Interrupt;
}

// This trait add *extra* methods to GeneralInstance4Channel,
// that GeneralInstance4Channel doesn't use, but the "AdvancedInstance"s need.
// And it's a private trait, so it's content won't leak to outer namespace.
//
// If you want to add a new method to it, please leave a detail comment to explain it.
trait General4ChBlankSealed {
    // SimplePwm<'d, T> is implemented for T: GeneralInstance4Channel
    // Advanced timers implement this trait, but the output needs to be
    // enabled explicitly.
    // To support general-purpose and advanced timers, this function is added
    // here defaulting to noop and overwritten for advanced timers.
    //
    // Enable timer outputs.
    fn enable_outputs(&self) {}
}

/// General-purpose 16-bit timer with 4 channels instance.
#[allow(private_bounds)]
pub trait GeneralInstance4Channel: BasicInstance + GeneralInstance2Channel + General4ChBlankSealed {}

/// General-purpose 32-bit timer with 4 channels instance.
pub trait GeneralInstance32bit4Channel: GeneralInstance4Channel {}

/// Advanced 16-bit timer with 1 channel instance.
pub trait AdvancedInstance1Channel: BasicNoCr2Instance + GeneralInstance1Channel {
    /// Communication interrupt for this timer.
    type CommunicationInterrupt: interrupt::typelevel::Interrupt;
    /// Break input interrupt for this timer.
    type BreakInputInterrupt: interrupt::typelevel::Interrupt;
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
        impl SealedInstance for crate::peripherals::$inst {
            fn state() -> &'static State {
                static STATE: State = State::new();
                &STATE
            }
        }

        impl CoreInstance for crate::peripherals::$inst {
            type UpdateInterrupt = crate::_generated::peripheral_interrupts::$inst::UP;

            const BITS: TimerBits = $bits;

            fn regs() -> *mut () {
                crate::pac::$inst.as_ptr()
            }
        }
    };
}

#[allow(unused)]
macro_rules! impl_general_1ch {
    ($inst:ident) => {
        impl GeneralInstance1Channel for crate::peripherals::$inst {
            type CaptureCompareInterrupt = crate::_generated::peripheral_interrupts::$inst::CC;
        }
    };
}

#[allow(unused)]
macro_rules! impl_general_2ch {
    ($inst:ident) => {
        impl GeneralInstance2Channel for crate::peripherals::$inst {
            type TriggerInterrupt = crate::_generated::peripheral_interrupts::$inst::TRG;
        }
    };
}

#[allow(unused)]
macro_rules! impl_advanced_1ch {
    ($inst:ident) => {
        impl AdvancedInstance1Channel for crate::peripherals::$inst {
            type CommunicationInterrupt = crate::_generated::peripheral_interrupts::$inst::COM;
            type BreakInputInterrupt = crate::_generated::peripheral_interrupts::$inst::BRK;
        }
    };
}

// This macro only apply to "AdvancedInstance(s)",
// not "GeneralInstance4Channel" itself.
#[allow(unused)]
macro_rules! impl_general_4ch_blank_sealed {
    ($inst:ident) => {
        impl General4ChBlankSealed for crate::peripherals::$inst {
            fn enable_outputs(&self) {
                unsafe { crate::pac::timer::Tim1chCmp::from_ptr(Self::regs()) }
                    .bdtr()
                    .modify(|w| w.set_moe(true));
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
        impl_general_1ch!($inst);
        impl_general_2ch!($inst);
        impl GeneralInstance4Channel for crate::peripherals::$inst {}
        impl General4ChBlankSealed for crate::peripherals::$inst {}
    };

    ($inst:ident, timer, TIM_2CH, UP, $irq:ident) => {
        impl_core_timer!($inst, TimerBits::Bits16);
        impl BasicNoCr2Instance for crate::peripherals::$inst {}
        impl BasicInstance for crate::peripherals::$inst {}
        impl_general_1ch!($inst);
        impl_general_2ch!($inst);
        impl GeneralInstance4Channel for crate::peripherals::$inst {}
        impl General4ChBlankSealed for crate::peripherals::$inst {}
    };

    ($inst:ident, timer, TIM_GP16, UP, $irq:ident) => {
        impl_core_timer!($inst, TimerBits::Bits16);
        impl BasicNoCr2Instance for crate::peripherals::$inst {}
        impl BasicInstance for crate::peripherals::$inst {}
        impl_general_1ch!($inst);
        impl_general_2ch!($inst);
        impl GeneralInstance4Channel for crate::peripherals::$inst {}
        impl General4ChBlankSealed for crate::peripherals::$inst {}
    };

    ($inst:ident, timer, TIM_GP32, UP, $irq:ident) => {
        impl_core_timer!($inst, TimerBits::Bits32);
        impl BasicNoCr2Instance for crate::peripherals::$inst {}
        impl BasicInstance for crate::peripherals::$inst {}
        impl_general_1ch!($inst);
        impl_general_2ch!($inst);
        impl GeneralInstance4Channel for crate::peripherals::$inst {}
        impl GeneralInstance32bit4Channel for crate::peripherals::$inst {}
        impl General4ChBlankSealed for crate::peripherals::$inst {}
    };

    ($inst:ident, timer, TIM_1CH_CMP, UP, $irq:ident) => {
        impl_core_timer!($inst, TimerBits::Bits16);
        impl BasicNoCr2Instance for crate::peripherals::$inst {}
        impl BasicInstance for crate::peripherals::$inst {}
        impl_general_1ch!($inst);
        impl_general_2ch!($inst);
        impl GeneralInstance4Channel for crate::peripherals::$inst {}
        impl_general_4ch_blank_sealed!($inst);
        impl_advanced_1ch!($inst);
        impl AdvancedInstance2Channel for crate::peripherals::$inst {}
        impl AdvancedInstance4Channel for crate::peripherals::$inst {}
    };

    ($inst:ident, timer, TIM_2CH_CMP, UP, $irq:ident) => {
        impl_core_timer!($inst, TimerBits::Bits16);
        impl BasicNoCr2Instance for crate::peripherals::$inst {}
        impl BasicInstance for crate::peripherals::$inst {}
        impl_general_1ch!($inst);
        impl_general_2ch!($inst);
        impl GeneralInstance4Channel for crate::peripherals::$inst {}
        impl_general_4ch_blank_sealed!($inst);
        impl_advanced_1ch!($inst);
        impl AdvancedInstance2Channel for crate::peripherals::$inst {}
        impl AdvancedInstance4Channel for crate::peripherals::$inst {}
    };

    ($inst:ident, timer, TIM_ADV, UP, $irq:ident) => {
        impl_core_timer!($inst, TimerBits::Bits16);
        impl BasicNoCr2Instance for crate::peripherals::$inst {}
        impl BasicInstance for crate::peripherals::$inst {}
        impl_general_1ch!($inst);
        impl_general_2ch!($inst);
        impl GeneralInstance4Channel for crate::peripherals::$inst {}
        impl_general_4ch_blank_sealed!($inst);
        impl_advanced_1ch!($inst);
        impl AdvancedInstance2Channel for crate::peripherals::$inst {}
        impl AdvancedInstance4Channel for crate::peripherals::$inst {}
    };
}

/// Update interrupt handler.
pub struct UpdateInterruptHandler<T: CoreInstance> {
    _phantom: PhantomData<T>,
}

impl<T: CoreInstance> interrupt::typelevel::Handler<T::UpdateInterrupt> for UpdateInterruptHandler<T> {
    unsafe fn on_interrupt() {
        #[cfg(feature = "low-power")]
        crate::low_power::on_wakeup_irq();

        let regs = crate::pac::timer::TimCore::from_ptr(T::regs());

        // Read TIM interrupt flags.
        let sr = regs.sr().read();

        // Mask relevant interrupts (UIE).
        let bits = sr.0 & 0x00000001;

        // Mask all the channels that fired.
        regs.dier().modify(|w| w.0 &= !bits);

        // Wake the tasks
        if sr.uif() {
            T::state().up_waker.wake();
        }
    }
}

/// Capture/Compare interrupt handler.
pub struct CaptureCompareInterruptHandler<T: GeneralInstance1Channel> {
    _phantom: PhantomData<T>,
}

impl<T: GeneralInstance1Channel> interrupt::typelevel::Handler<T::CaptureCompareInterrupt>
    for CaptureCompareInterruptHandler<T>
{
    unsafe fn on_interrupt() {
        #[cfg(feature = "low-power")]
        crate::low_power::on_wakeup_irq();

        let regs = crate::pac::timer::TimGp16::from_ptr(T::regs());

        // Read TIM interrupt flags.
        let sr = regs.sr().read();

        // Mask relevant interrupts (CCIE).
        let bits = sr.0 & 0x0000001E;

        // Mask all the channels that fired.
        regs.dier().modify(|w| w.0 &= !bits);

        // Wake the tasks
        for ch in 0..4 {
            if sr.ccif(ch) {
                T::state().cc_waker[ch].wake();
            }
        }
    }
}
