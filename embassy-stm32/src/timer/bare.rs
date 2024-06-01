//! Bare register-level timer driver.
//!
//! This module provides the core functionality for type-erased drivers for all STM32 timers. It
//! provides integration with the RCC and direct access to the timer registers. It is intended to
//! serve as a building block for higher level drivers.
use core::marker::PhantomData;

use embassy_hal_internal::PeripheralRef;

use super::{AdvancedInstance4Channel, Channel, CoreInstance, GeneralInstance4Channel, Info, TimerBits};
use crate::gpio::{AfType, AnyPin, Pin, SealedPin};
use crate::time::Hertz;
use crate::{into_ref, pac, rcc, Peripheral};

trait TimerPinMarkerSealed {}

/// Trait for marker types that represent all possible timer pins.
#[allow(private_bounds)]
pub trait TimerPinMarker: TimerPinMarkerSealed {}

/// Timer pin trait.
///
/// If a pin peripheral implements `TimerPin<T, M>`, it means that it can be used with timer `T` in
/// the role represented by marker type `M`. For example, `TimerPin<TIM1, Ch2>` is implemented for
/// all pins that can be used as channel 2 for timer TIM1.
///
/// This trait is equivalent to the pin-specific traits defined in the [`timer`][super] module; for
/// example `TimerPin<T, Ch1>` is equivalent to [`timer::Channel1Pin<T>`][super::Channel1Pin].
pub trait TimerPin<T: CoreInstance, M: TimerPinMarker>: Pin {
    /// Get the AF number needed to use this pin with timer `T` as pin `M`.
    fn af_num(&self) -> u8;
}

/// Marker type for channel 1 pin.
pub enum Ch1 {}
/// Marker type for channel 2 pin.
pub enum Ch2 {}
/// Marker type for channel 3 pin.
pub enum Ch3 {}
/// Marker type for channel 4 pin.
pub enum Ch4 {}
/// Marker type for external trigger pin.
pub enum Etr {}
/// Marker type for channel 1 complementary pin.
pub enum Ch1N {}
/// Marker type for channel 2 complementary pin.
pub enum Ch2N {}
/// Marker type for channel 3 complementary pin.
pub enum Ch3N {}
/// Marker type for channel 4 complementary pin.
pub enum Ch4N {}
/// Marker type for break input pin.
pub enum Bkin {}
/// Marker type for break input comparator 1 pin.
pub enum BkinComp1 {}
/// Marker type for break input comparator 2 pin.
pub enum BkinComp2 {}
/// Marker type for break 2 input pin.
pub enum Bkin2 {}
/// Marker type for break 2 input comparator 1 pin.
pub enum Bkin2Comp1 {}
/// Marker type for break 2 input comparator 2 pin.
pub enum Bkin2Comp2 {}

macro_rules! impl_pin {
    ($marker_ty:ident, $t:ident, $pin_trait:path, $peri_trait:ident) => {
        impl TimerPinMarkerSealed for $marker_ty {}
        impl TimerPinMarker for $marker_ty {}

        impl<$t: $peri_trait, P> TimerPin<$t, $marker_ty> for P
        where
            P: $pin_trait,
        {
            fn af_num(&self) -> u8 {
                <P as $pin_trait>::af_num(self)
            }
        }
    };
}

impl_pin!(Ch1, T, super::Channel1Pin<T>, GeneralInstance4Channel);
impl_pin!(Ch2, T, super::Channel2Pin<T>, GeneralInstance4Channel);
impl_pin!(Ch3, T, super::Channel3Pin<T>, GeneralInstance4Channel);
impl_pin!(Ch4, T, super::Channel4Pin<T>, GeneralInstance4Channel);
impl_pin!(Etr, T, super::ExternalTriggerPin<T>, GeneralInstance4Channel);

impl_pin!(Ch1N, T, super::Channel1ComplementaryPin<T>, AdvancedInstance4Channel);
impl_pin!(Ch2N, T, super::Channel2ComplementaryPin<T>, AdvancedInstance4Channel);
impl_pin!(Ch3N, T, super::Channel3ComplementaryPin<T>, AdvancedInstance4Channel);
impl_pin!(Ch4N, T, super::Channel4ComplementaryPin<T>, AdvancedInstance4Channel);

impl_pin!(Bkin, T, super::BreakInputPin<T>, AdvancedInstance4Channel);
impl_pin!(
    BkinComp1,
    T,
    super::BreakInputComparator1Pin<T>,
    AdvancedInstance4Channel
);
impl_pin!(
    BkinComp2,
    T,
    super::BreakInputComparator2Pin<T>,
    AdvancedInstance4Channel
);

impl_pin!(Bkin2, T, super::BreakInput2Pin<T>, AdvancedInstance4Channel);
impl_pin!(
    Bkin2Comp1,
    T,
    super::BreakInput2Comparator1Pin<T>,
    AdvancedInstance4Channel
);
impl_pin!(
    Bkin2Comp2,
    T,
    super::BreakInput2Comparator2Pin<T>,
    AdvancedInstance4Channel
);

/// Trait for marker types that represent the four channel pins ([`Ch1`], [`Ch2`], [`Ch3`],
/// [`Ch4`]).
pub trait TimerChannelMarker: TimerPinMarker {
    /// Representation of the channel number.
    const CHANNEL: Channel;
}

macro_rules! impl_channel {
    ($marker_ty:ident, $channel:expr) => {
        impl TimerChannelMarker for $marker_ty {
            const CHANNEL: Channel = $channel;
        }
    };
}

impl_channel!(Ch1, Channel::Ch1);
impl_channel!(Ch2, Channel::Ch2);
impl_channel!(Ch3, Channel::Ch3);
impl_channel!(Ch4, Channel::Ch4);

/// Type-erased timer pin.
///
/// The only purpose of this struct is to correctly initialize the pin in the constructor and
/// deinitialize it (set it as disconnected) in the constructor. It can be used to implement
/// higher-level pin wrappers, like [`simple_pwm::PwmPin`](super::simple_pwm::PwmPin).
pub struct AnyTimerPin<'d> {
    pin: PeripheralRef<'d, AnyPin>,
}

impl<'d> AnyTimerPin<'d> {
    /// Initializes a timer pin `M` for timer instance `T`.
    pub fn new<T: CoreInstance, M: TimerPinMarker>(
        pin: impl Peripheral<P = impl TimerPin<T, M>> + 'd,
        af_type: AfType,
    ) -> Self {
        into_ref!(pin);
        let af_num = pin.af_num();
        let pin = pin.map_into();
        pin.set_as_af(af_num, af_type);
        Self { pin }
    }
}

impl<'d> Drop for AnyTimerPin<'d> {
    fn drop(&mut self) {
        self.pin.set_as_disconnected();
    }
}

/// Type-erased bare driver for timers.
///
/// This driver provides direct access to the timer registers, with the type of the peripheral
/// erased.
pub struct AnyTimer<'d, Regs> {
    info: &'d Info,
    kernel_clock: Hertz,
    _phantom: PhantomData<&'d mut Regs>,
}

macro_rules! impl_regs {
    ($reg_ty:path, $new:ident, $peri_trait:path) => {
        impl<'d> AnyTimer<'d, $reg_ty> {
            /// Initializes timer instance `T`.
            pub fn $new<T: $peri_trait>(_tim: impl Peripheral<P = T> + 'd) -> Self {
                rcc::enable_and_reset::<T>();
                Self {
                    info: T::info(),
                    kernel_clock: T::frequency(),
                    _phantom: PhantomData,
                }
            }

            /// Get the registers for this timer.
            pub fn regs(&self) -> $reg_ty {
                unsafe { <$reg_ty>::from_ptr(self.info.regs) }
            }
        }
    };
}

impl_regs!(pac::timer::TimCore, new_core, CoreInstance);
impl_regs!(pac::timer::TimBasicNoCr2, new_basic_no_cr2, super::BasicNoCr2Instance);
impl_regs!(pac::timer::TimBasic, new_basic, super::BasicInstance);
impl_regs!(pac::timer::Tim1ch, new_1ch, super::GeneralInstance1Channel);
impl_regs!(pac::timer::Tim2ch, new_2ch, super::GeneralInstance2Channel);
impl_regs!(pac::timer::TimGp16, new_gp16, GeneralInstance4Channel);
#[cfg(not(timer_l0))]
impl_regs!(pac::timer::TimGp32, new_gp32, super::GeneralInstance32bit4Channel);
#[cfg(not(timer_l0))]
impl_regs!(pac::timer::Tim1chCmp, new_1ch_cmp, super::AdvancedInstance1Channel);
#[cfg(not(timer_l0))]
impl_regs!(pac::timer::Tim2chCmp, new_2ch_cmp, super::AdvancedInstance2Channel);
#[cfg(not(timer_l0))]
impl_regs!(pac::timer::TimAdv, new_advanced, AdvancedInstance4Channel);

impl<'d, Regs> AnyTimer<'d, Regs> {
    /// Get the kernel clock frequency for the timer peripheral.
    ///
    /// Unless you switch the timer to a different clock source, this is the frequency that is fed
    /// into the prescaler to drive the timer.
    pub fn clock_frequency(&self) -> Hertz {
        self.kernel_clock
    }

    /// Get the number of bits in this timer (16 or 32).
    pub fn bits(&self) -> TimerBits {
        self.info.bits
    }
}

impl<'d, Regs> Drop for AnyTimer<'d, Regs> {
    fn drop(&mut self) {
        self.info.rcc.disable();
    }
}
