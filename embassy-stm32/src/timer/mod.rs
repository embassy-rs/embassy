//! General-purpose timers (TIM).
//!
//! Every STM32 microcontroller contains several general-purpose timer peripherals. A single chip
//! contains several types of timers with different capabilities:
//!
//! - [**Basic timers**][BasicInstance] (TIM6, TIM7) can be used for timekeeping but don't have any inputs or
//! outputs.
//! - **General timers** have input/output channels with output comparison and input capture. They
//! can have 1, 2 or 4 channels:
//!     - [1-channel][General1ChInstance] timers (TIM10, TIM14, ...) and
//!     [2-channel][General2ChInstance] timers (TIM9, TIM12, TIM21, ...) (_channel timers_)
//!     have somewhat limited feature set and reduced channel count.
//!     - [4-channel][General4ChInstance] timers (TIM3, TIM4, ...) (_general-purpose timer_) have the
//!     full feature set.
//!     - [32-bit timer][General32BitInstance] (TIM2, TIM5, ...) timers are a variant of the
//!     4-channel timers with a 32-bit counter instead of the standard 16-bit counter.
//! - **Advanced timers** have additional features not available on general timers. Advanced timers
//! can also have 1, 2 or 4 channels:
//!     - [Advanced 1-channel][Advanced1ChInstance] timers (TIM16, TIM17) and [advanced
//!     2-channel][Advanced2ChInstance] timers (TIM15, TIM21, ...) have one complementary output
//!     and some additional features.
//!     - [Advanced 4-channel][Advanced4ChInstance] timers (TIM1, TIM8) have four complementary
//!     outputs (in addition to the standard four input/output channels) and the most advanced
//!     feature set related to motor control and power conversion applications.
//! - Some chips also have low-power timers (LPTIM) and high-resolution timers (HRTIM), but these
//! are not covered by the drivers in this module.
//!
//! Note that the numbering of timer peripherals is usually consistent across all chip families, so
//! for example `TIM1` is always an advanced 4-channel timer and `TIM6` is always a basic timer.
//!
//! ## Instance traits
//!
//! For each type of timer, we export a trait which is implemented by the timer peripherals of that
//! type:
//!
//! - [`BasicInstance`] for basic timers
//! - For general timers:
//!     - [`General1ChInstance`] for 1-channel timers
//!     - [`General2ChInstance`] for 2-channel timers
//!     - [`General4ChInstance`] for 4-channel timers
//!     - [`General32BitInstance`] for 32-bit, 4-channel timers
//! - For advanced timers:
//!     - [`Advanced1ChInstance`] for advanced 1-channel timers
//!     - [`Advanced2ChInstance`] for advanced 2-channel timers
//!     - [`Advanced4ChInstance`] for advanced 4-channel timers
//!
//! Fortunately, the timers are bit-compatible on the register level, so a more capable timer can
//! be used in place of a less capable timer:
//!
//! - Timers with more channels are more capable than timers with fewer channels (so, for
//! example, [`General2ChInstance`] is more capable than [`General1ChInstance`], and
//! [`Advanced4ChInstance`] is more capable than [`Advanced2ChInstance`]).
//! - Advanced channels are more capable than general timers _with the same number of channels_
//! (so for example [`Advanced2ChInstance`] can be used where [`General2ChInstance`] is expected).
//!
//! These relations are encoded in the type system, so for example [`Advanced2ChInstance`] has
//! [`Advanced1ChInstance`] and [`General2ChInstance`] as supertraits.
//!
//! However, there is not a simple linear ordering of timers from least to most capable: for
//! example, neither of [`Advanced2ChInstance`] and [`General4ChInstance`] is more capable than the
//! other. They have some functionality in common, but either of them has a feature that the other
//! doesn't have. To accomodate these common feature subsets, we introduce "virtual" timer traits:
//!
//! - [`UpDmaInstance`] represents the common parts of [`BasicInstance`], [`General4ChInstance`]
//! and [`Advanced1ChInstance`].
//! - [`MmsInstance`] represents the common parts of [`BasicInstance`] and [`General2ChInstance`].
//! - [`CcDmaInstance`] represents the common parts of [`General4ChInstance`] and
//! [`Advanced1ChInstance`].
//! - [`TrigDmaInstance`] represents the common parts of [`General4ChInstance`] and
//! [`Advanced2ChInstance`].
//!
//! All these relations are also expressed in the type system, so for example
//! [`General4ChInstance`] implies [`MmsInstance`] and [`TrigDmaInstance`] implies
//! [`General2ChInstance`].
//!
//! This means that for every subset of timer features that we might need, there is always a
//! _single_ trait that exactly matches the timer peripherals that support all of these features.
//!
//! ## Marker types
//!
//! The traits described in previous section are implemented for timer peripherals such as
//! [`TIM2`][crate::peripherals::TIM2]. In timer drivers, we often want to have basic functionality
//! that is available for most timers (such as PWM generation in the
//! [`SimplePwm`][self::simple_pwm::SimplePwm] driver, which requires just [`General1ChInstance`]),
//! but we also want to expose some additional features for timers that support them (such as
//! waveform PWM generation using DMA in the `SimplePwm` driver, which needs [`CcDmaInstance`]).
//! However, we also don't want to include the type of the concrete timer peripheral (such as
//! `TIM2`) in the type of the driver, to reduce code duplication from monomorphization and to
//! avoid leaking implementation details to the users of the driver.
//!
//! For these reasons, we introduce marker types. Each peripheral instance trait (such as
//! [`General4ChInstance`]) has an associated marker type ([`General4ChTim`]) and marker type trait
//! ([`IsGeneral4ChTim`]). We use the marker type as a type-level "witness" that the driver was
//! instantiated with a timer peripheral that has at least the given capability. For example, you
//! can [build `SimplePwm<General4ChTim>`][self::simple_pwm::Builder::build_4ch()] only from a
//! timer peripheral that implements [`General4ChInstance`].
//!
//! We also introduce marker type traits to implement the "is at least as capable as" relation on
//! marker types. For example, the marker type trait [`IsGeneral4ChTim`] is implemented for
//! [`General4ChTim`] and [`Advanced4ChTim`], but not for [`General2ChTim`] or [`Advanced2ChTim`].
//! The relation implied by the marker type traits (`Is*Tim`) is the same as the relation implied
//! by the instance traits (`*Instance`). The marker type traits allow us to express that some
//! driver methods are only available for some timers; for example,
//! [`SimplePwm<Tim>::waveform_cc_dma()`][simple_pwm::SimplePwm::waveform_cc_dma()] is only
//! available when `Tim: IsCcDmaTim`, so we can use it on `SimplePwm<General4ChTim>` but not on
//! `SimplePwm<General1ChTim>`.
//!
//! ## Drivers
//!
//! STM32 timers are very versatile and can be used for a wide variety of purposes, from dimming a
//! LED to controlling a brushless motor. Embassy will never be able to handle all use cases, so we
//! expose high-level drivers for simple tasks (such as dimming a LED using PWM) but give you all
//! tools that you need to implement your own type-safe drivers for more advanced tasks (such as
//! controlling a brushless motor using an advanced timer):
//!
//! - [`raw::RawTimer`] and other items of the [`raw`] submodule expose the raw functionality that
//! gives you type-safe access to timer registers and timer pins.
//! - [`low_level::Timer`] is a thin wrapper over the raw driver that wraps register operations
//! with convenient method calls.
//! - Other submodules contain high-level drivers for common timer tasks, such as PWM output
//! ([`simple_pwm`]) or quadrature decoder input ([`qei`]).
use core::marker::PhantomData;

use embassy_sync::waitqueue::AtomicWaker;

#[allow(unused_imports)] // used in documentation links
use crate::pac;
use crate::rcc::{RccInfo, RccPeripheral, SealedRccPeripheral};
use crate::{dma, gpio, interrupt};

#[cfg(not(timer_l0))]
pub mod complementary_pwm;
pub mod input_capture;
pub mod low_level;
pub mod pwm_input;
pub mod qei;
pub mod raw;
pub mod simple_pwm;

// === Instance traits and taxonomy ===

trait SealedCoreInstance {
    fn info() -> &'static Info;
    fn state() -> &'static State;
}

trait SealedTimMarker {}

/// Peripheral instance trait for all timers.
///
/// This trait is implemented by all general-purpose timer peripherals. It is not implemented for
/// the low-power timer (LPTIM) or for the high-resolution timer (HRTIM). It corresponds to
/// [`TimCore`][crate::pac::timer::TimCore] in the PAC.
///
/// Marker type for this trait is [`CoreTim`] and marker type trait is [`IsCoreTim`].
#[allow(private_bounds)]
pub trait CoreInstance: SealedCoreInstance + RccPeripheral + 'static {
    /// Update interrupt for this timer.
    type UpdateInterrupt: interrupt::typelevel::Interrupt;

    /// Registers for this timer.
    ///
    /// This is a raw pointer to the register block. The actual register block layout varies
    /// depending on the timer type.
    fn regs() -> *mut ();
}

/// Marker type for [`CoreInstance`].
///
/// Please see the [module documentation](super) for details.
pub enum CoreTim {}

/// Marker type trait for [`CoreInstance`].
///
/// Please see the [module documentation](super) for details.
#[allow(private_bounds)]
pub trait IsCoreTim: SealedTimMarker {}
impl IsCoreTim for CoreTim {}
impl SealedTimMarker for CoreTim {}

macro_rules! marker_type_and_trait {
    (
        $instance_trait:ident, $marker_ty:ident, $marker_trait:ident :
        $($marker_super_trait:ident),+ $(,)? ;
        $($marker_implied_trait:ident),* $(,)?
    ) => {
        #[doc = concat!(
            "Marker type for [`", stringify!($instance_trait), "`].\n\n",
            "Please see the [module documentation](super) for details.",
        )]
        pub enum $marker_ty {}

        #[doc = concat!(
            "Marker type trait for [`", stringify!($instance_trait), "`].\n\n",
            "Please see the [module documentation](super) for details.",
        )]
        #[allow(private_bounds)]
        pub trait $marker_trait: $($marker_super_trait +)* SealedTimMarker {}

        impl $marker_trait for $marker_ty {}
        $( impl $marker_super_trait for $marker_ty {} )*
        $( impl $marker_implied_trait for $marker_ty {} )*
        impl SealedTimMarker for $marker_ty {}
    };
    (
        $instance_trait:ident, $marker_ty:ident, $marker_trait:ident :
        $($marker_super_trait:ident),+ $(,)?
    ) => {
        marker_type_and_trait! {
            $instance_trait, $marker_ty, $marker_trait:
            $($marker_super_trait),+ ;
        }
    };
}

/// Peripheral instance trait for basic, 4-channel and advanced timers.
///
/// This trait is the intersection of [`BasicInstance`], [`General4ChInstance`] and
/// [`Advanced1ChInstance`]. It provides access to the UDE register field to enable the update DMA.
///
/// Marker type for this trait is [`UpDmaTim`] and marker type trait is [`IsUpDmaTim`].
pub trait UpDmaInstance: CoreInstance {}
marker_type_and_trait! { UpDmaInstance, UpDmaTim, IsUpDmaTim: IsCoreTim }

/// Peripheral instance trait for basic and 2-channel timers.
///
/// This trait is the intersection of [`BasicInstance`] and [`General2ChInstance`]. It provides
/// access to the master mode selection (MMS) register field.
///
/// Marker type for this trait is [`MmsTim`] and marker type trait is [`IsMmsTim`].
pub trait MmsInstance: CoreInstance {}
marker_type_and_trait! { MmsInstance, MmsTim, IsMmsTim: IsCoreTim }

/// Peripheral instance trait for the basic timers.
///
/// This trait is implemented for basic timer peripherals and corresponds to
/// [`TimBasic`][crate::pac::timer::TimBasic] in the PAC.
///
/// Marker type for this trait is [`BasicTim`] and marker type trait is [`IsBasicTim`].
pub trait BasicInstance: UpDmaInstance + MmsInstance {}
marker_type_and_trait! { BasicInstance, BasicTim, IsBasicTim: IsUpDmaTim, IsMmsTim; IsCoreTim }

/// Peripheral instance trait for general-purpose 1-channel timers.
///
/// This trait is implemented for all timers with channels and corresponds to
/// [`Tim1ch`][crate::pac::timer::Tim1ch] in the PAC.
pub trait General1ChInstance: CoreInstance {
    /// Capture/compare interrupt for this timer.
    type CaptureCompareInterrupt: interrupt::typelevel::Interrupt;
}
marker_type_and_trait! { General1ChInstance, General1ChTim, IsGeneral1ChTim: IsCoreTim }

/// Peripheral instance trait for 4-channel and advanced timers.
///
/// This trait is the intersection of [`General4ChInstance`] and [`Advanced1ChInstance`]. It
/// provides access to register fields related to capture/compare DMA, timer DMA bursts (DMAR
/// register) and other functionality.
pub trait CcDmaInstance: General1ChInstance + UpDmaInstance {}
marker_type_and_trait! { CcDmaInstance, CcDmaTim, IsCcDmaTim: IsGeneral1ChTim, IsUpDmaTim; IsCoreTim }

/// Peripheral instance trait for general-purpose 2-channel timers.
///
/// This trait is implemented for all timers with at least two channels and corresponds to
/// [`Tim2ch`][crate::pac::timer::Tim2ch] in the PAC.
pub trait General2ChInstance: General1ChInstance + MmsInstance {
    /// Trigger event interrupt for this timer.
    type TriggerInterrupt: interrupt::typelevel::Interrupt;
}
marker_type_and_trait! {
    General2ChInstance, General2ChTim, IsGeneral2ChTim: IsGeneral1ChTim, IsMmsTim;
    IsCoreTim,
}

/// Peripheral instance trait for 4-channel and advanced 2-channel timers.
///
/// This trait is the intersection of [`General4ChInstance`] and [`Advanced2ChInstance`]. It
/// provides access to the TDE register field to enable the trigger DMA.
pub trait TrigDmaInstance: General2ChInstance + CcDmaInstance + BasicInstance {}
marker_type_and_trait! {
    TrigDmaInstance, TrigDmaTim, IsTrigDmaTim:
    IsGeneral2ChTim, IsCcDmaTim, IsBasicTim;
    IsGeneral1ChTim, IsMmsTim, IsUpDmaTim, IsCoreTim,
}

/// Peripheral instance trait for general-purpose 4-channel timers.
///
/// This trait is implemented for all timers with four channels and corresponds to
/// [`Tim4ch`][crate::pac::timer::Tim4ch] in the PAC.
pub trait General4ChInstance: General2ChInstance + UpDmaInstance + TrigDmaInstance + BasicInstance {}
marker_type_and_trait! {
    General4ChInstance, General4ChTim, IsGeneral4ChTim:
    IsGeneral2ChTim, IsUpDmaTim, IsCcDmaTim, IsTrigDmaTim, IsBasicTim;
    IsGeneral1ChTim, IsMmsTim, IsCoreTim,
}

/// Peripheral instance trait for 4-channel timers with 32-bit counter.
///
/// This trait is implemented for the 32-bit timer peripheral and corresponds to
/// [`Tim32bit`][crate::pac::timer::Tim32bit] in the PAC.
pub trait General32BitInstance: General4ChInstance {}
marker_type_and_trait! {
    General32BitInstance, General32BitTim, IsGeneral32BitTim:
    IsGeneral4ChTim;
    IsGeneral2ChTim, IsGeneral1ChTim, IsUpDmaTim, IsCcDmaTim, IsTrigDmaTim, IsMmsTim, IsBasicTim, IsCoreTim,
}

/// Peripheral instance trait for advanced 1-channel timers (1-channel timers with complementary
/// output).
///
/// This trait is implemented for all advanced timer peripherals and corresponds to
/// [`TimAdv1ch`][crate::pac::timer::TimAdv1ch] in the PAC.
pub trait Advanced1ChInstance: General1ChInstance + UpDmaInstance + CcDmaInstance {
    /// Communication interrupt for this timer.
    type CommunicationInterrupt: interrupt::typelevel::Interrupt;
    /// Break input interrupt for this timer.
    type BreakInputInterrupt: interrupt::typelevel::Interrupt;
}
marker_type_and_trait! {
    Advanced1ChInstance, Advanced1ChTim, IsAdvanced1ChTim:
    IsGeneral1ChTim, IsUpDmaTim, IsCcDmaTim;
    IsCoreTim,
}

/// Peripheral instance trait for advanced 2-channel timers (2-channel timers with complementary
/// output).
///
/// This trait is implemented for all advanced timers with at least two channels and corresponds to
/// [`TimAdv2ch`][crate::pac::timer::TimAdv2ch] in the PAC.
pub trait Advanced2ChInstance: General2ChInstance + Advanced1ChInstance + TrigDmaInstance + BasicInstance {}
marker_type_and_trait! {
    Advanced2ChInstance, Advanced2ChTim, IsAdvanced2ChTim:
    IsGeneral2ChTim, IsAdvanced1ChTim, IsTrigDmaTim, IsBasicTim;
    IsGeneral1ChTim, IsUpDmaTim, IsCcDmaTim, IsMmsTim, IsCoreTim,
}

/// Peripheral instance trait for advanced 4-channel timers (advanced timers).
///
/// This trait is implemented for advanced 4-channel timer peripherals and corresponds to
/// [`TimAdv4ch`][crate::pac::timer::TimAdv4ch] in the PAC.
pub trait Advanced4ChInstance: Advanced2ChInstance + General4ChInstance {}
marker_type_and_trait! {
    Advanced4ChInstance, Advanced4ChTim, IsAdvanced4ChTim:
    IsAdvanced2ChTim, IsGeneral4ChTim;
    IsAdvanced1ChTim, IsGeneral2ChTim, IsGeneral1ChTim,
    IsTrigDmaTim, IsUpDmaTim, IsCcDmaTim, IsMmsTim, IsBasicTim, IsCoreTim,
}

// === Pins ===

trait TimerPinMarkerSealed {}

/// Trait for marker types that represent all possible timer pins.
#[allow(private_bounds)]
pub trait TimerPinMarker: TimerPinMarkerSealed {}

/// Timer pin trait.
///
/// If a pin peripheral implements `TimerPin<T, M>`, it means that it can be used with timer `T` in
/// the role represented by marker type `M`. For example, `TimerPin<TIM1, Ch2>` is implemented for
/// all pin peripherals that can be used as a pin for channel 2 for timer TIM1.
// developer note: implementations of this trait are in code generated by build.rs
pub trait TimerPin<T: CoreInstance, M: TimerPinMarker>: gpio::Pin {
    /// Get the AF number needed to use this pin with timer `T` as pin `M`.
    fn af_num(&self) -> u8;
}

macro_rules! impl_pin_marker {
    ($marker_ty:ident) => {
        impl TimerPinMarkerSealed for $marker_ty {}
        impl TimerPinMarker for $marker_ty {}
    };
}

/// Marker type for channel 1.
pub enum Ch1 {}
impl_pin_marker!(Ch1);

/// Marker type for channel 2.
pub enum Ch2 {}
impl_pin_marker!(Ch2);

/// Marker type for channel 3.
pub enum Ch3 {}
impl_pin_marker!(Ch3);

/// Marker type for channel 4.
pub enum Ch4 {}
impl_pin_marker!(Ch4);

/// Marker type for external trigger pin.
pub enum Etr {}
impl_pin_marker!(Etr);

/// Marker type for channel 1 complementary pin.
pub enum Ch1N {}
impl_pin_marker!(Ch1N);

/// Marker type for channel 2 complementary pin.
pub enum Ch2N {}
impl_pin_marker!(Ch2N);

/// Marker type for channel 3 complementary pin.
pub enum Ch3N {}
impl_pin_marker!(Ch3N);

/// Marker type for channel 4 complementary pin.
pub enum Ch4N {}
impl_pin_marker!(Ch4N);

/// Marker type for break input pin.
pub enum Bkin {}
impl_pin_marker!(Bkin);

/// Marker type for break input comparator 1 pin.
pub enum BkinComp1 {}
impl_pin_marker!(BkinComp1);

/// Marker type for break input comparator 2 pin.
pub enum BkinComp2 {}
impl_pin_marker!(BkinComp2);

/// Marker type for break 2 input pin.
pub enum Bkin2 {}
impl_pin_marker!(Bkin2);

/// Marker type for break 2 input comparator 1 pin.
pub enum Bkin2Comp1 {}
impl_pin_marker!(Bkin2Comp1);

/// Marker type for break 2 input comparator 2 pin.
pub enum Bkin2Comp2 {}
impl_pin_marker!(Bkin2Comp2);

// === Channels ===

/// Timer channel.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(u8)]
pub enum Channel {
    /// Channel 1.
    Ch1 = 0,
    /// Channel 2.
    Ch2 = 1,
    /// Channel 3.
    Ch3 = 2,
    /// Channel 4.
    Ch4 = 3,
}

impl Channel {
    /// Get the channel index (0..3)
    pub fn index(self) -> usize {
        self as u8 as usize
    }
}

/// Trait for marker types that represent the four channel pins ([`Ch1`], [`Ch2`], [`Ch3`],
/// [`Ch4`]).
pub trait ChannelMarker: TimerPinMarker {
    /// Representation of the channel number.
    const CHANNEL: Channel;
}

/// Trait for marker types that represent the four complementary channel pins ([`Ch1N`], [`Ch2N`],
/// [`Ch3N`], [`Ch4N`]).
pub trait NChannelMarker: TimerPinMarker {
    /// Representation of the channel number.
    const N_CHANNEL: Channel;
}

macro_rules! impl_channel_marker {
    ($marker_ty:ident, $n_marker_ty:ident, $channel:expr) => {
        impl ChannelMarker for $marker_ty {
            const CHANNEL: Channel = $channel;
        }

        impl NChannelMarker for $n_marker_ty {
            const N_CHANNEL: Channel = $channel;
        }
    };
}
impl_channel_marker!(Ch1, Ch1N, Channel::Ch1);
impl_channel_marker!(Ch2, Ch2N, Channel::Ch2);
impl_channel_marker!(Ch3, Ch3N, Channel::Ch3);
impl_channel_marker!(Ch4, Ch4N, Channel::Ch4);

// === DMAs ===

/// Capture/compare DMA trait.
///
/// If a DMA channel implements `CcDma<T, C>`, it means that it can be used with the
/// capture/compare DMA request for channel `C` of timer `T`. For example, `CcDma<TIM2,
/// Ch1>` is implemented for DMA channels that can be used with capture/compare DMA request of
/// channel 1 on timer TIM2.
pub trait CcDma<T: CoreInstance, C: ChannelMarker>: dma::Channel {
    /// Get the DMA request number needed to use this DMA channel as DMA request for channel `C` of
    /// timer peripheral `T`.
    ///
    /// Note: in some chips, ST calls this the "channel", and calls channels "streams".
    /// `embassy-stm32` always uses the "channel" and "request number" names.
    fn request(&self) -> dma::Request;
}

dma_trait!(UpDma, CoreInstance);
dma_trait!(TrigDma, CoreInstance);

// === Runtime info and state ===

/// Number of channels of a timer.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(u8)]
pub enum ChannelCount {
    /// Zero channels.
    Zero = 0,
    /// One channel.
    One = 1,
    /// Two channels.
    Two = 2,
    /// Four channels.
    Four = 4,
}

impl ChannelCount {
    /// Get the number of channels as an integer.
    pub fn to_usize(self) -> usize {
        self as u8 as usize
    }
}

/// Bit size of a timer.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum TimerBits {
    /// 16 bits.
    Bits16,
    /// 32 bits.
    #[cfg(not(timer_l0))]
    Bits32,
}

/// Type of a timer peripheral.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum TimerType {
    /// Basic timer ([`BasicInstance`]).
    Basic,
    /// General-purpose 1-channel timer ([`General1ChInstance`]).
    General1Ch,
    /// General-purpose 2-channel timer ([`General2ChInstance`]).
    General2Ch,
    /// General-purpose 4-channel timer ([`General4ChInstance`]).
    General4Ch,
    /// General-purpose 32-bit timer ([`General32BitInstance`]).
    General32Bit,
    /// Advanced 1-channel timer ([`Advanced1ChInstance`]).
    Advanced1Ch,
    /// Advanced 2-channel timer ([`Advanced2ChInstance`]).
    Advanced2Ch,
    /// Advanced 4-channel timer ([`Advanced4ChInstance`]).
    Advanced4Ch,
}

impl TimerType {
    /// Get the number of channels supported by this timer.
    pub const fn channel_count(self) -> ChannelCount {
        match self {
            Self::Basic => ChannelCount::Zero,
            Self::General1Ch | Self::Advanced1Ch => ChannelCount::One,
            Self::General2Ch | Self::Advanced2Ch => ChannelCount::Two,
            Self::General4Ch | Self::General32Bit | Self::Advanced4Ch => ChannelCount::Four,
        }
    }

    /// Get the bit width of the counter for this timer.
    pub const fn bits(self) -> TimerBits {
        match self {
            #[cfg(not(timer_l0))]
            Self::General32Bit => TimerBits::Bits32,
            _ => TimerBits::Bits16,
        }
    }

    /// Is this timer one of the advanced timers?
    pub const fn is_advanced(self) -> bool {
        matches!(self, Self::Advanced1Ch | Self::Advanced2Ch | Self::Advanced4Ch)
    }
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

struct Info {
    regs: *mut (),
    rcc: RccInfo,
    timer_type: TimerType,
}

unsafe impl Sync for Info {}

// === Peripheral impls ===

macro_rules! impl_core {
    ($inst:ident, $timer_type:expr) => {
        impl SealedCoreInstance for crate::peripherals::$inst {
            fn info() -> &'static Info {
                static INFO: Info = Info {
                    regs: crate::pac::$inst.as_ptr(),
                    rcc: crate::peripherals::$inst::RCC_INFO,
                    timer_type: $timer_type,
                };
                &INFO
            }

            fn state() -> &'static State {
                static STATE: State = State::new();
                &STATE
            }
        }

        impl CoreInstance for crate::peripherals::$inst {
            type UpdateInterrupt = crate::_generated::peripheral_interrupts::$inst::UP;

            fn regs() -> *mut () {
                crate::pac::$inst.as_ptr()
            }
        }
    };
}

macro_rules! impl_general_1ch {
    ($inst:ident) => {
        impl General1ChInstance for crate::peripherals::$inst {
            type CaptureCompareInterrupt = crate::_generated::peripheral_interrupts::$inst::CC;
        }
    };
}

macro_rules! impl_general_2ch {
    ($inst:ident) => {
        impl General2ChInstance for crate::peripherals::$inst {
            type TriggerInterrupt = crate::_generated::peripheral_interrupts::$inst::TRG;
        }
    };
}

#[allow(unused_macros)]
macro_rules! impl_advanced_1ch {
    ($inst:ident) => {
        impl Advanced1ChInstance for crate::peripherals::$inst {
            type CommunicationInterrupt = crate::_generated::peripheral_interrupts::$inst::COM;
            type BreakInputInterrupt = crate::_generated::peripheral_interrupts::$inst::BRK;
        }
    };
}

foreach_interrupt! {
    ($inst:ident, timer, TIM_BASIC, UP, $irq:ident) => {
        impl_core!($inst, TimerType::Basic);
        impl UpDmaInstance for crate::peripherals::$inst {}
        impl MmsInstance for crate::peripherals::$inst {}
        impl BasicInstance for crate::peripherals::$inst {}
    };

    ($inst:ident, timer, TIM_1CH, UP, $irq:ident) => {
        impl_core!($inst, TimerType::General1Ch);
        impl_general_1ch!($inst);
    };

    ($inst:ident, timer, TIM_2CH, UP, $irq:ident) => {
        impl_core!($inst, TimerType::General2Ch);
        impl_general_1ch!($inst);
        impl_general_2ch!($inst);
        impl MmsInstance for crate::peripherals::$inst {}
    };

    ($inst:ident, timer, TIM_4CH, UP, $irq:ident) => {
        impl_core!($inst, TimerType::General4Ch);
        impl_general_1ch!($inst);
        impl_general_2ch!($inst);
        impl UpDmaInstance for crate::peripherals::$inst {}
        impl MmsInstance for crate::peripherals::$inst {}
        impl TrigDmaInstance for crate::peripherals::$inst {}
        impl CcDmaInstance for crate::peripherals::$inst {}
        impl BasicInstance for crate::peripherals::$inst {}
        impl General4ChInstance for crate::peripherals::$inst {}
    };

    ($inst:ident, timer, TIM_32BIT, UP, $irq:ident) => {
        impl_core!($inst, TimerType::General32Bit);
        impl_general_1ch!($inst);
        impl_general_2ch!($inst);
        impl UpDmaInstance for crate::peripherals::$inst {}
        impl MmsInstance for crate::peripherals::$inst {}
        impl TrigDmaInstance for crate::peripherals::$inst {}
        impl CcDmaInstance for crate::peripherals::$inst {}
        impl BasicInstance for crate::peripherals::$inst {}
        impl General4ChInstance for crate::peripherals::$inst {}
        impl General32BitInstance for crate::peripherals::$inst {}
    };

    ($inst:ident, timer, TIM_ADV1CH, UP, $irq:ident) => {
        impl_core!($inst, TimerType::Advanced1Ch);
        impl_general_1ch!($inst);
        impl_advanced_1ch!($inst);
        impl UpDmaInstance for crate::peripherals::$inst {}
        impl CcDmaInstance for crate::peripherals::$inst {}
    };

    ($inst:ident, timer, TIM_ADV2CH, UP, $irq:ident) => {
        impl_core!($inst, TimerType::Advanced2Ch);
        impl_general_1ch!($inst);
        impl_general_2ch!($inst);
        impl_advanced_1ch!($inst);
        impl UpDmaInstance for crate::peripherals::$inst {}
        impl MmsInstance for crate::peripherals::$inst {}
        impl CcDmaInstance for crate::peripherals::$inst {}
        impl TrigDmaInstance for crate::peripherals::$inst {}
        impl BasicInstance for crate::peripherals::$inst {}
        impl Advanced2ChInstance for crate::peripherals::$inst {}
    };

    ($inst:ident, timer, TIM_ADV4CH, UP, $irq:ident) => {
        impl_core!($inst, TimerType::Advanced4Ch);
        impl_general_1ch!($inst);
        impl_general_2ch!($inst);
        impl_advanced_1ch!($inst);
        impl UpDmaInstance for crate::peripherals::$inst {}
        impl MmsInstance for crate::peripherals::$inst {}
        impl CcDmaInstance for crate::peripherals::$inst {}
        impl TrigDmaInstance for crate::peripherals::$inst {}
        impl BasicInstance for crate::peripherals::$inst {}
        impl Advanced2ChInstance for crate::peripherals::$inst {}
        impl General4ChInstance for crate::peripherals::$inst {}
        impl Advanced4ChInstance for crate::peripherals::$inst {}
    };
}

// === Interrupt handlers ===

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
pub struct CaptureCompareInterruptHandler<T: General1ChInstance> {
    _phantom: PhantomData<T>,
}

impl<T: General1ChInstance> interrupt::typelevel::Handler<T::CaptureCompareInterrupt>
    for CaptureCompareInterruptHandler<T>
{
    unsafe fn on_interrupt() {
        #[cfg(feature = "low-power")]
        crate::low_power::on_wakeup_irq();

        let regs = crate::pac::timer::Tim4ch::from_ptr(T::regs());

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
